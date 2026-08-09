//! Tests of the range reader. The tests use a mock server, because the
//! behaviour depends on real HTTP answers.

use std::io::{Read, Seek, SeekFrom};
use toutui::api::client::error::ApiError;
use toutui::player::engine::http_file::HttpFile;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

/// Makes the content of a test file. Each byte has the value of its position,
/// thus a test can prove that the reader gives the correct bytes.
fn content(size: usize) -> Vec<u8> {
    (0..size).map(|value| (value % 251) as u8).collect()
}

/// Reads the first byte of a `Range` header.
fn range_start(request: &Request) -> usize {
    request
        .headers
        .get("range")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("bytes="))
        .and_then(|value| value.split('-').next())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

/// Answers a range request in the same way as Audiobookshelf.
fn range_answer(body: &[u8], request: &Request) -> ResponseTemplate {
    let total = body.len();
    let start = range_start(request).min(total);
    let end = total.saturating_sub(1);

    ResponseTemplate::new(206)
        .insert_header("accept-ranges", "bytes")
        .insert_header(
            "content-range",
            format!("bytes {}-{}/{}", start, end, total).as_str(),
        )
        .set_body_bytes(body[start..].to_vec())
}

async fn server_with(body: Vec<u8>) -> MockServer {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .respond_with(move |request: &Request| range_answer(&body, request))
        .mount(&server)
        .await;

    server
}

#[tokio::test]
async fn the_reader_gives_the_total_size_from_content_range() {
    let server = server_with(content(5000)).await;
    let uri = server.uri();

    let size = tokio::task::spawn_blocking(move || {
        HttpFile::open(&uri, "test-token", "item1", "ino1")
            .unwrap()
            .len()
    })
    .await
    .unwrap();

    assert_eq!(size, 5000);
}

#[tokio::test]
async fn the_reader_gives_the_correct_bytes() {
    let expected = content(5000);
    let server = server_with(expected.clone()).await;
    let uri = server.uri();

    let got = tokio::task::spawn_blocking(move || {
        let mut file = HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap();
        let mut got = Vec::new();
        file.read_to_end(&mut got).unwrap();
        got
    })
    .await
    .unwrap();

    assert_eq!(got.len(), expected.len());
    assert_eq!(got, expected);
}

#[tokio::test]
async fn a_seek_operation_gives_the_bytes_from_that_position() {
    let expected = content(5000);
    let server = server_with(expected.clone()).await;
    let uri = server.uri();

    let got = tokio::task::spawn_blocking(move || {
        let mut file = HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap();
        file.seek(SeekFrom::Start(4000)).unwrap();
        let mut got = Vec::new();
        file.read_to_end(&mut got).unwrap();
        got
    })
    .await
    .unwrap();

    assert_eq!(got, expected[4000..].to_vec());
}

/// `Decoder` uses `SeekFrom::End` to find the size of a file. An M4B file
/// fails without this behaviour.
#[tokio::test]
async fn a_seek_operation_from_the_end_is_correct() {
    let server = server_with(content(5000)).await;
    let uri = server.uri();

    let position = tokio::task::spawn_blocking(move || {
        let mut file = HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap();
        file.seek(SeekFrom::End(0)).unwrap()
    })
    .await
    .unwrap();

    assert_eq!(position, 5000);
}

/// A movement backward and then a read operation must give the correct bytes.
#[tokio::test]
async fn a_movement_backward_gives_the_correct_bytes() {
    let expected = content(20_000);
    let server = server_with(expected.clone()).await;
    let uri = server.uri();

    let got = tokio::task::spawn_blocking(move || {
        let mut file = HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap();
        file.seek(SeekFrom::Start(10_000)).unwrap();

        let mut ignore = vec![0u8; 10];
        file.read_exact(&mut ignore).unwrap();

        file.seek(SeekFrom::Start(50)).unwrap();
        let mut got = vec![0u8; 10];
        file.read_exact(&mut got).unwrap();
        got
    })
    .await
    .unwrap();

    assert_eq!(got, expected[50..60].to_vec());
}

/// A short movement forward must use the buffer. The reader must not send a
/// new request for a movement of a few bytes.
#[tokio::test]
async fn a_short_movement_forward_sends_no_new_request() {
    let expected = content(200_000);
    let server = MockServer::start().await;
    let body = expected.clone();

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .respond_with(move |request: &Request| range_answer(&body, request))
        .expect(2)
        .mount(&server)
        .await;

    let uri = server.uri();

    let got = tokio::task::spawn_blocking(move || {
        let mut file = HttpFile::open(&uri, "test-token", "item1", "ino1").unwrap();

        // Give the thread time to fill the buffer.
        std::thread::sleep(std::time::Duration::from_millis(600));

        file.seek(SeekFrom::Start(1000)).unwrap();
        let mut got = vec![0u8; 10];
        file.read_exact(&mut got).unwrap();
        got
    })
    .await
    .unwrap();

    assert_eq!(got, expected[1000..1010].to_vec());

    // The mock expects two requests only: the request of `open` that reads
    // the size, and the request of the thread. The movement sends none.
    drop(server);
}

#[tokio::test]
async fn the_reader_sends_the_token_in_the_authorization_header() {
    let server = MockServer::start().await;
    let body = content(1000);

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .and(header("authorization", "Bearer secret-token"))
        .respond_with(move |request: &Request| range_answer(&body, request))
        .mount(&server)
        .await;

    let uri = server.uri();

    let size = tokio::task::spawn_blocking(move || {
        HttpFile::open(&uri, "secret-token", "item1", "ino1")
            .unwrap()
            .len()
    })
    .await
    .unwrap();

    assert_eq!(size, 1000);
}

/// The address must hold no token. This proves the correction of T-6, and it
/// also proves the correction of T-5, because a token in the address goes to
/// the list of processes.
#[tokio::test]
async fn the_address_holds_no_token() {
    let server = MockServer::start().await;
    let body = content(1000);

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .respond_with(move |request: &Request| {
            assert!(
                request.url.query().is_none(),
                "the address must hold no query, but it holds {:?}",
                request.url.query()
            );
            range_answer(&body, request)
        })
        .mount(&server)
        .await;

    let uri = server.uri();

    tokio::task::spawn_blocking(move || {
        HttpFile::open(&uri, "secret-token", "item1", "ino1").unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn a_status_401_gives_an_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let uri = server.uri();

    let result =
        tokio::task::spawn_blocking(move || HttpFile::open(&uri, "bad-token", "item1", "ino1"))
            .await
            .unwrap();

    assert!(matches!(result, Err(ApiError::Unauthorized)));
}

/// The server does not give the header `Content-Range`. The reader must give
/// an error, because it cannot know the size of the file.
#[tokio::test]
async fn an_answer_with_no_content_range_gives_an_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/item1/file/ino1/download"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(content(100)))
        .mount(&server)
        .await;

    let uri = server.uri();

    let result =
        tokio::task::spawn_blocking(move || HttpFile::open(&uri, "test-token", "item1", "ino1"))
            .await
            .unwrap();

    assert!(matches!(result, Err(ApiError::Decode(_))));
}
