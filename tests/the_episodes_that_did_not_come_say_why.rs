//! The view of the episodes of a podcast says why it holds no line. See T-168.
//!
//! **The server went away while the program stood, and the view lied.** A
//! measurement of 2026-08-14 with the sandbox and tmux: the program started with
//! the server up, `podman stop -t 0 abs-test` took the server away, and the key
//! `l` of a podcast gave
//! `The program gets the episodes of this podcast…`. That sentence stood on the
//! screen 28 seconds later, and the log held
//! `[podcast] the server gave no episode of the podcast …: No server address
//! answered.` **The program had stopped that work at the first second.**
//!
//! `is_offline` of `App` holds the offline mode of the **start** (T-25). A
//! program that started with a server that answers therefore holds `false` for
//! ever, and the three conditions of T-91 did not reach this one: the request of
//! the key went, it did not come back, and the view had no word for that.
//!
//! **This test needs no sandbox.** A host of a raw socket answers `404` to every
//! request: the request of the program therefore comes back with a fault, as it
//! does for a server that went away.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::library_items::get_pod_ep::get_pod_ep;
use toutui::logic::the_episodes;

/// Starts a host that answers `404 Not Found` to every request.
///
/// A mock server is a crate of its own, and a raw socket needs none: the answer
/// holds a length, therefore the client of the program reads it and stops.
async fn a_host_that_holds_nothing() -> String {
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

                let body = b"Item not found";
                let head = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\
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

/// A request of the episodes that did not come back gives the view one
/// sentence, and that sentence names what the server said.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary fight for the boxes of the process (T-144 and T-157), and
/// `logic::the_episodes` holds such a box.
#[tokio::test(flavor = "multi_thread")]
async fn the_view_of_the_episodes_says_that_the_request_did_not_come_back() {
    let address = a_host_that_holds_nothing().await;

    let pool = EndpointPool::new(vec![Endpoint::new(&address, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    the_episodes::forget();

    // The place of the podcast in the lists of the library. The user opened it,
    // therefore the program asked the server for its episodes.
    let the_place_of_the_podcast = 3;

    let error = get_pod_ep(&api, "9fa45bd1-66bc-4c17-ba49-a5a6a5ec8806")
        .await
        .expect_err("the host of this test holds no podcast");

    // This is the line of the task of `ask_the_server_for_the_episodes`.
    the_episodes::keep_the_fault(the_place_of_the_podcast, error.to_string().as_str());
    the_episodes::keep_the_flag(false);

    // And this is the line of the render of that view.
    let what_the_server_said = the_episodes::the_fault_of(the_place_of_the_podcast);

    let what_the_server_said = what_the_server_said.expect(
        "the request did not come back, therefore the view must know it. \
         It knew nothing at all: this is T-168.",
    );

    let text = the_episodes::the_reason_of_no_episode(false, false, Some(&what_the_server_said));

    assert!(
        text.starts_with("The server did not give the episodes of this podcast:"),
        "the view must say that the server gave no episode, and it says {:?}",
        text
    );

    assert!(
        !text.contains("gets the episodes"),
        "the view must not promise a work that the program stopped (T-118), \
         and it says {:?}",
        text
    );

    assert!(
        text.len() > "The server did not give the episodes of this podcast:".len() + 1,
        "the sentence must name what the server said, and it says {:?}",
        text
    );

    // **A user who opens a second podcast must not read the fault of the first
    // one**: the box holds the place of its own podcast.
    assert_eq!(
        the_episodes::the_fault_of(the_place_of_the_podcast + 1),
        None
    );

    // A new request of that podcast takes the fault away, therefore the view
    // says "The program gets the episodes of this podcast…" again while that
    // request runs.
    the_episodes::forget_the_fault_of(the_place_of_the_podcast);

    assert_eq!(the_episodes::the_fault_of(the_place_of_the_podcast), None);

    assert!(
        the_episodes::the_reason_of_no_episode(false, false, None).contains("gets the episodes")
    );

    the_episodes::forget();
}
