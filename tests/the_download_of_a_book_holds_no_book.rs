//! The program writes the parts of a book to the disk, and it holds no whole
//! book in its memory. See T-116.
//!
//! `download_to_file` read `response.bytes()`, therefore the whole answer stood
//! in the memory of the program of the user. The sweep of a book of a scan of
//! **502 megabytes** on 2026-08-12 measured `VmHWM` of the real program:
//!
//! ```text
//! the peak of the program of the user: 1031420 kB (1007 MB)
//! ```
//!
//! **1007 megabytes for a file of 502**: the buffer of the answer grows by a copy
//! of itself. T-62 moved the parse of a PDF into a child for the same reason, and
//! the download of the same book stayed in the program that the user reads.
//!
//! **A test of that memory needs a server outside this process.** A measurement
//! of 2026-08-12 read `VmHWM` around a download of 96 megabytes of a mock server
//! of `wiremock`: the mock makes its answer inside the process of the test, and
//! the two forms of the code both gave 192 megabytes. The memory of the answer of
//! the server hides the memory of the client, therefore **the number of this
//! fault comes from the real program**, and it stands in T-116 of
//! `docs/TAKEOVER-BACKLOG.md`.
//!
//! This test holds the two rules that a test can hold:
//!
//! 1. A book of 96 megabytes comes to the disk complete, through the loop of the
//!    parts.
//! 2. **`download_to_file` holds no `response.bytes()`.** The test reads the
//!    source of the program, in the same way as
//!    `every_key_of_the_handler_stands_in_the_list`.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The size of the book of this test.
const BOOK: usize = 96 * 1024 * 1024;

/// The byte of every page of the book of this test.
const THE_BYTE: u8 = 7;

/// The source of the client of the requests.
const THE_SOURCE_OF_THE_CLIENT: &str = include_str!("../src/api/client/mod.rs");

#[tokio::test(flavor = "multi_thread")]
async fn a_large_book_comes_to_the_disk_complete() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/big/ebook"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![THE_BYTE; BOOK]))
        .mount(&server)
        .await;

    let pool = EndpointPool::new(vec![Endpoint::new(&server.uri(), 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    let dir = tempfile::tempdir().unwrap();

    api.download_to_file("/api/items/big/ebook", dir.path(), "big.epub")
        .await
        .expect("the book must come");

    // **Every part stands in the file, and the parts keep their sequence.** A
    // loop that writes a part in the wrong place gives a file of the right size
    // and a book that no reader opens.
    let bytes = std::fs::read(dir.path().join("big.epub")).expect("the file of the book");

    assert_eq!(bytes.len(), BOOK, "the file holds every byte");
    assert!(
        bytes.iter().all(|byte| *byte == THE_BYTE),
        "every byte of the file is a byte of the answer"
    );
}

/// **The download of a book holds no whole book in the memory.**
///
/// The test reads the source, because no test of this process can measure that
/// memory: a mock server makes its answer in the memory of the same process. See
/// T-116 and the head of this file.
#[test]
fn the_client_writes_the_parts_of_a_download() {
    let start = THE_SOURCE_OF_THE_CLIENT
        .find("pub async fn download_to_file")
        .expect("the client holds download_to_file");

    let end = THE_SOURCE_OF_THE_CLIENT[start..]
        .find("\n    }\n")
        .map(|place| start + place)
        .expect("the function has an end");

    // **A comment is not code.** The comment of this correction names
    // `response.bytes()`, and a rule that reads the whole text would find it.
    let body: String = THE_SOURCE_OF_THE_CLIENT[start..end]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n");

    assert!(
        body.contains(".chunk()"),
        "download_to_file must write the parts of the answer: {}",
        body
    );
    assert!(
        !body.contains(".bytes()"),
        "download_to_file holds the whole book in the memory of the user: {}",
        body
    );
}
