//! The reader sends no place when the server did not give the place. See
//! T-178.
//!
//! **The key `e` reads a state of the server and the program then writes it.**
//! `place_of_the_server` reads `ebookLocation` and `ebookProgress` of
//! `GET /api/me/progress/:id`, and the loop of the program sends the place of
//! the reader to `PATCH /api/me/progress/:id` every 30 seconds and at the
//! moment that the user leaves the book. The old code held `.ok()?`: **every**
//! fault of that read gave the reader the first page of the book, and the send
//! after it took the place of the user away.
//!
//! A measurement of 2026-08-14 against the sandbox with
//! `docs/harness/one_method_fails.py`, which answered `500` to
//! `GET /api/me/progress/:id` and which forwarded the `PATCH` of that same
//! path:
//!
//! | The moment | The answer |
//! |---|---|
//! | The server holds `Alice in Wonderland` | `ebookLocation toutui:12:300`, `ebookProgress 0.6` |
//! | The reader of the key `e` | `chapter 2 of 14 — 0%` |
//! | The words of the reader | none |
//! | The key `h`, and the send that comes with it | `PATCH /api/me/progress/:id` |
//! | The server after it | `ebookProgress 0.0041284304384330275` |
//!
//! **The user lost their place in a book of 14 chapters, on every machine of
//! that account**, and one request of the status 500 did that work.
//!
//! **A status of 404 is a different answer**: the server holds no progress for
//! this media, therefore the user never opened this book. The reader starts at
//! the first page, and the send of that place is the truth. That road stays,
//! and this test holds it too.
//!
//! **This test needs no sandbox.** A host of a raw socket gives the fault to
//! the read alone (T-167).

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::logic::reader::session::{
    place_of_the_server, the_sentence_of_a_place_that_did_not_come,
    the_sentence_of_a_place_that_stays_here, ThePlaceOfTheBook,
};
use toutui::logic::reader::Reader;

/// The media of this measurement.
const THE_MEDIA: &str = "8fda6e43-0728-46ad-98bc-4c8634e299ad";

/// Starts a host that gives one status to the read of the progress, and that
/// answers every other request with `200`.
async fn a_host_whose_read_gives(the_status: u16) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
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

                // **The body of the request must leave the socket** (T-220). A host
                // that closes a connection which still holds bytes of the request
                // gives that connection a `RST`, and the client then loses the answer
                // that this host wrote already.
                let of_the_head_of_the_request = String::from_utf8_lossy(&head).to_string();
                let mut the_length_of_the_body = 0usize;
                for line in of_the_head_of_the_request.lines() {
                    if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                        the_length_of_the_body = value.trim().parse().unwrap_or(0);
                    }
                }
                if the_length_of_the_body > 0 {
                    let mut the_body_of_the_request = vec![0u8; the_length_of_the_body];
                    let _ = socket.read_exact(&mut the_body_of_the_request).await;
                }

                let words = match the_status {
                    404 => "404 Not Found",
                    _ => "500 Internal Server Error",
                };
                let body: &[u8] = b"this host said no.";

                let answer = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    words,
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

/// A read of the place that came back with a fault sends no place, and it says
/// why.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn the_reader_sends_no_place_and_it_says_what_the_server_said() {
    // The road of the fault: the read gives the status 500.
    let api = a_client(&a_host_whose_read_gives(500).await);

    let fault = place_of_the_server(&api, THE_MEDIA)
        .await
        .expect_err("a read of the status 500 must not give a place");

    let text = the_sentence_of_a_place_that_did_not_come(&fault);

    assert!(
        text.starts_with("The server did not give your place:"),
        "the reader must say that the server did not give the place, and it says {:?}",
        text
    );

    assert!(
        text.contains("Status 500"),
        "the sentence must name what the server said (T-91), and it says {:?}",
        text
    );

    assert!(
        text.contains("The program writes no place."),
        "the sentence must say that the program writes nothing, and it says {:?}",
        text
    );

    assert!(
        text.contains("Press h and then e"),
        "the sentence must name a key that does this work again (T-170), and it says {:?}",
        text
    );

    // The road of the media that the user never opened: the server says 404,
    // and such a book has no place. The reader starts at the first page, and
    // its send is the truth of that book.
    let api = a_client(&a_host_whose_read_gives(404).await);

    let place = place_of_the_server(&api, THE_MEDIA)
        .await
        .expect("a status of 404 is the answer of a book that no user opened");

    assert!(
        place.is_none(),
        "a book that the user never opened holds no place, and the read gives {:?}",
        place
    );

    // The reader of a place that did not come sends nothing at all, and the
    // reader of a book of the server sends.
    let path = std::path::Path::new("tests/data/alice.epub");
    let mut reader = Reader::open(path, THE_MEDIA).expect("the book must open");

    assert_eq!(
        reader.the_place_of_the_book(),
        ThePlaceOfTheBook::GoesToTheServer,
        "the book of the server sends its place"
    );
    assert!(reader.sends_the_place());

    reader.the_server_did_not_give_the_place();

    assert!(
        !reader.sends_the_place(),
        "a place that the program did not read must not go to the server"
    );
    assert!(
        !reader.wants_to_send(),
        "the rule of the loop must ask for no send of such a book"
    );
    assert!(
        !reader.wants_to_send_at_the_end(),
        "the key h must send no place of such a book"
    );

    // **The two roads that send no place say two different things** (T-91).
    let of_the_server = the_sentence_of_a_place_that_stays_here(reader.the_place_of_the_book())
        .expect("a reader that sends no place says why");
    let of_another_book =
        the_sentence_of_a_place_that_stays_here(ThePlaceOfTheBook::AnotherBookOfTheItem)
            .expect("a book of another file of the item says why too");

    assert_ne!(
        of_the_server, of_another_book,
        "the sentence of a fault of the server must not name a book of this machine"
    );
    assert!(
        of_the_server.contains("The server did not give your place in this book."),
        "{}",
        of_the_server
    );
    assert!(
        the_sentence_of_a_place_that_stays_here(ThePlaceOfTheBook::GoesToTheServer).is_none(),
        "a reader that sends its place says nothing"
    );
}
