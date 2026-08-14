//! The header says that the server reports a fault, and not that it is away.
//! See T-171.
//!
//! **A server that answers `500` is not a server that does not answer.** The
//! pool marks that address down, because a second address of the same server
//! can answer it (T-87), and `active()` then gives nothing at all: the header
//! read the same value as a server that no machine reaches.
//!
//! The measurement of 2026-08-14, with the sandbox, tmux, and
//! `docs/harness/one_path_fails.py`. The proxy gave the status 500 to
//! `GET /api/libraries/:id/authors` alone, and it forwarded every other request
//! to the sandbox. The key `a` of the Library view of the library `Books` then
//! gave the header:
//!
//! ```text
//! ⚠ toutuitest: the server does not answer      📖 Books (book)
//! 🔗 127.0.0.1:13500 does not answer                     R: the media of the disk
//! ```
//!
//! That header stood for **10.5 seconds**, until the probe task ran, and it can
//! stand for the whole 60 seconds of `PROBE_INTERVAL`. In the middle of it
//! `curl` got an answer of that same address in **1.4 milliseconds**, and the
//! key `W` of the program gave the 114 sessions of the account. The two
//! sentences are a reason that the program does not have (T-91), and the notice
//! offers the media of the disk (T-107) to a user whose server holds every list
//! (T-170: a sentence of a fault must name a key that does the work of that
//! fault).
//!
//! **This test needs no sandbox.** A host of a raw socket answers `500` to
//! every request, in the way of `tests/the_lists_that_did_not_come_say_why.rs`.
//! A port that no program holds is a different condition — that road is the
//! offline mode of T-25 — and the second half of this rule stands in the tests
//! of `src/api/client/endpoint.rs`.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::ui::keys::{
    the_lines_of_the_connection, THE_SERVER_DOES_NOT_ANSWER, THE_SERVER_REPORTS_A_FAULT,
};

/// Starts a host that answers `500 Internal Server Error` to every request.
async fn a_host_that_fails() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                let mut request = Vec::new();
                let mut byte = [0u8; 1];
                while socket.read(&mut byte).await.unwrap_or(0) == 1 {
                    request.push(byte[0]);
                    if request.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }

                // **The body of the request must leave the socket** (T-220). A host
                // that closes a connection which still holds bytes of the request
                // gives that connection a `RST`, and the client then loses the answer
                // that this host wrote already.
                let of_the_head_of_the_request = String::from_utf8_lossy(&request).to_string();
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

                let body = b"the database of this server is away";
                let head = format!(
                    "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );

                let _ = socket.write_all(head.as_bytes()).await;
                let _ = socket.write_all(body).await;
                let _ = socket.flush().await;
            });
        }
    });

    address
}

/// One request that came back with the status 500 gives the header the words of
/// a server that reports a fault, and not the words of a server that is away.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary fight for the boxes of the process (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn the_header_says_that_the_server_reports_a_fault() {
    let address = a_host_that_fails().await;

    let pool = Arc::new(EndpointPool::new(vec![Endpoint::new(&address, 0)]));
    let api = ApiClient::new(Arc::clone(&pool), "token".to_string()).unwrap();

    // The pool holds one address, and that address is well.
    assert!(pool.active().is_some());
    assert!(!pool.every_address_answers_with_a_fault());

    // This is the road of every request of a key of the user.
    let answer: Result<serde_json::Value, _> = api.get_json("/api/libraries/x/authors").await;

    assert!(answer.is_err(), "the host answers 500 to every request");

    // **The address goes down, and that decision stays**: a second address of
    // the same server can answer this request (T-87 and T-97).
    assert!(
        pool.active().is_none(),
        "the pool must hold no address with the state Up"
    );

    // And this is the value that the header of T-171 needs.
    assert!(
        pool.every_address_answers_with_a_fault(),
        "the address answered, and the answer holds a fault. The pool knew the \
         state Down alone, and the header therefore said that the server is \
         away: this is T-171."
    );

    let text = the_lines_of_the_connection("toutuitest", None, "127.0.0.1:13500", false, true, 160);

    assert!(
        !text.contains("does not answer"),
        "the server answered, therefore the header must not say that it does \
         not answer. It says {:?}",
        text
    );

    assert!(
        text.contains("the server reports a fault"),
        "the header must say what the program measured, and it says {:?}",
        text
    );

    assert!(
        text.contains("127.0.0.1:13500 reports a fault"),
        "the second line must name the address, and it says {:?}",
        text
    );

    // **The notice must name a key that does the work of that fault** (T-170).
    // The key `R` asks the server again, and the server holds every list.
    assert_eq!(THE_SERVER_REPORTS_A_FAULT, "R: ask the server again");

    assert!(
        !THE_SERVER_REPORTS_A_FAULT.contains("the media of the disk"),
        "the media of the disk are the road of a server that is away (T-107)"
    );

    // A narrow terminal keeps the two parts of the header apart (T-115), and it
    // says the same thing.
    let short = the_lines_of_the_connection("toutuitest", None, "127.0.0.1:13500", false, true, 60);

    assert!(!short.contains("does not answer"), "{}", short);
    assert!(short.contains("reports a fault"), "{}", short);

    // **The words of a server that gives no answer stay as they are.** A pool
    // that no machine reaches keeps the sentence of T-107 and the media of the
    // disk.
    let away =
        the_lines_of_the_connection("toutuitest", None, "127.0.0.1:13500", false, false, 160);

    assert!(away.contains("the server does not answer"), "{}", away);
    assert_eq!(THE_SERVER_DOES_NOT_ANSWER, "R: the media of the disk");

    // The offline mode of the start holds its own words (T-25), and the fault
    // of the server does not change them.
    let offline =
        the_lines_of_the_connection("toutuitest", None, "127.0.0.1:13500", true, true, 160);

    assert!(offline.contains("📴 Offline as toutuitest"), "{}", offline);
}
