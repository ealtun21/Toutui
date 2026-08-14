//! The keys `M` and `N` write nothing when the server did not give the state.
//! See T-175.
//!
//! **The two keys read a state of the server, and they then write the opposite
//! of it.** `mark_the_media` reads `isFinished` of
//! `GET /api/me/progress/:id`, and `hide_the_media` reads
//! `hideFromContinueListening` of the same request. The old code read **every**
//! fault of that request as "the server has no progress for this media",
//! therefore a read that did not come back gave both keys the value `false`,
//! and the write of the key went to one direction alone.
//!
//! A measurement of 2026-08-14 against the sandbox with
//! `docs/harness/one_method_fails.py`, which answered `500` to
//! `GET /api/me/progress/:id` and which forwarded the `PATCH` of that same
//! path:
//!
//! | The moment | The answer |
//! |---|---|
//! | The server holds `A Long Test Book` as finished | `isFinished true` |
//! | The key `M` of the user, who wants no mark | `The media is finished now.` |
//! | The server after it | `isFinished true` |
//! | The server holds `A Big Book Of A Scan` away from the shelf | `hideFromContinueListening true` |
//! | The key `N` of the user, who wants it back | `The media is away from Continue Listening now.` |
//! | The server after it | `hideFromContinueListening true` |
//!
//! **The words of the program named a state that it did not read**, and the key
//! of the user did the opposite of its work.
//!
//! **A status of 404 is a different answer**: the server says that it holds no
//! progress for this media, and such a media is not finished and it is not away
//! from the shelf. That road stays, and this test holds it too.
//!
//! **This test needs no sandbox.** A host of a raw socket gives the fault to
//! the read alone, and it writes down every request that came.

use std::sync::{Arc, Mutex};
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{hide_the_media, mark_the_media};

/// The media of this measurement.
const THE_MEDIA: &str = "9485e88c-ab57-471a-bc05-d4fe01be1607";

/// Starts a host that gives one status to the read of the progress, and that
/// answers every other request with `200`.
///
/// The list holds the method and the path of every request that came, therefore
/// the test says whether a write went to the server.
async fn a_host_whose_read_fails(
    the_status_of_the_read: u16,
    the_requests: Arc<Mutex<Vec<String>>>,
) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let the_requests = Arc::clone(&the_requests);

            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                let mut head = Vec::new();
                let mut byte = [0u8; 1];
                while socket.read(&mut byte).await.unwrap_or(0) == 1 {
                    head.push(byte[0]);
                    if head.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }

                let head = String::from_utf8_lossy(&head).to_string();
                let first = head.lines().next().unwrap_or("").to_string();
                let mut words = first.split(' ');
                let method = words.next().unwrap_or("").to_string();
                let path = words.next().unwrap_or("").to_string();

                // The body of a `PATCH` must leave the socket. A host that
                // closes the connection before it gives the client a fault of
                // the network, and this test measures the fault of the server.
                let mut length = 0usize;
                for line in head.lines() {
                    if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse().unwrap_or(0);
                    }
                }
                if length > 0 {
                    let mut body = vec![0u8; length];
                    let _ = socket.read_exact(&mut body).await;
                }

                if let Ok(mut list) = the_requests.lock() {
                    list.push(format!("{} {}", method, path));
                }

                let it_is_the_read = method == "GET" && path.starts_with("/api/me/progress");

                let (status, body): (String, &[u8]) = if it_is_the_read {
                    let words = match the_status_of_the_read {
                        404 => "404 Not Found",
                        _ => "500 Internal Server Error",
                    };
                    (words.to_string(), b"this host said no.")
                } else {
                    ("200 OK".to_string(), b"{}")
                };

                let answer = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    status,
                    body.len()
                );

                let _ = socket.write_all(answer.as_bytes()).await;
                let _ = socket.write_all(body).await;
                let _ = socket.flush().await;
            });
        }
    });

    address
}

/// Gives a client of the program for one address.
fn a_client(address: &str) -> Arc<ApiClient> {
    let pool = EndpointPool::new(vec![Endpoint::new(address, 0)]);
    Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap())
}

/// A read that came back with a fault stops the write, and it says why.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn the_keys_write_nothing_and_they_say_what_the_server_said() {
    // The road of the fault: the read gives the status 500.
    let the_requests = Arc::new(Mutex::new(Vec::new()));
    let address = a_host_whose_read_fails(500, Arc::clone(&the_requests)).await;
    let api = a_client(&address);

    let text = mark_the_media(&api, THE_MEDIA).await;

    assert!(
        text.starts_with("The server did not give the mark:"),
        "the key M must say that the server did not give the mark, and it says {:?}",
        text
    );

    assert!(
        text.contains("Status 500"),
        "the sentence must name what the server said (T-91), and it says {:?}",
        text
    );

    assert!(
        text.contains("The program changed nothing."),
        "the sentence must say that the program wrote nothing, and it says {:?}",
        text
    );

    assert!(
        text.contains("Press M"),
        "the sentence must name a key that does this work again (T-170), and it says {:?}",
        text
    );

    assert!(
        !text.contains("is finished now"),
        "the program must not say a mark that it did not write, and it says {:?}",
        text
    );

    let text = hide_the_media(&api, THE_MEDIA).await;

    assert!(
        text.starts_with("The server did not give the state of this media:"),
        "the key N must say that the server did not give the state, and it says {:?}",
        text
    );

    assert!(
        text.contains("The program changed nothing."),
        "the sentence must say that the program wrote nothing, and it says {:?}",
        text
    );

    assert!(
        text.contains("Press N"),
        "the sentence must name a key that does this work again (T-170), and it says {:?}",
        text
    );

    assert!(
        !text.contains("Continue Listening now"),
        "the program must not say a state that it did not write, and it says {:?}",
        text
    );

    // **No write went to the server.** This is the larger half of the item: the
    // old code wrote `isFinished: true` and `hideFromContinueListening: true`
    // for a state that it did not read.
    let list = the_requests.lock().unwrap().clone();

    assert!(
        !list.iter().any(|request| request.starts_with("PATCH")),
        "the program must write nothing after a read that did not come back, \
         and the requests are {:?}",
        list
    );

    // The road of a media that never played stays: the server says `404`, and
    // such a media is not finished.
    let the_requests = Arc::new(Mutex::new(Vec::new()));
    let address = a_host_whose_read_fails(404, Arc::clone(&the_requests)).await;
    let api = a_client(&address);

    let text = mark_the_media(&api, THE_MEDIA).await;

    assert_eq!(
        text, "The media is finished now.",
        "a media that the server does not hold in the progress is not finished, \
         therefore the key M must mark it"
    );

    let list = the_requests.lock().unwrap().clone();

    assert!(
        list.iter().any(|request| request.starts_with("PATCH")),
        "the key M of a media that never played must write to the server, and \
         the requests are {:?}",
        list
    );
}
