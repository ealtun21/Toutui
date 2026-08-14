//! The view of the collections and of the playlists says why it holds no line.
//! See T-169.
//!
//! **A server that answers some requests and that fails others** put a sentence
//! on the screen that the program does not have. A measurement of 2026-08-14
//! with the sandbox, tmux, and `docs/harness/one_path_fails.py`: the proxy gave
//! the status 500 to `GET /api/libraries/:id/collections` and to
//! `GET /api/libraries/:id/playlists` of the library `Books`, and it forwarded
//! every other request to the sandbox. The key `c` of that program then said
//! `This library has no collection and no playlist.` for a library of one
//! collection and of one playlist, and the key `m` said
//! `This library holds no collection and no playlist. Press c or p to make
//! one.`
//!
//! `is_offline` of `App` holds the offline mode of the **start** (T-25). The
//! server answered the libraries, the shelves, and the items, therefore that
//! value holds `false` and the words of T-91 for a server that does not answer
//! never came.
//!
//! **This test needs no sandbox.** A host of a raw socket answers `500` to
//! every request: the request of the program therefore comes back with a fault,
//! as it does for a server that fails these two endpoints.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::logic::the_lists;

/// Starts a host that answers `500 Internal Server Error` to every request.
///
/// **The status of the fault must come from the server**, and not from a port
/// that no program holds: that road is the offline mode of T-25, and the words
/// of that mode are right there (T-167).
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

/// A request of the lists that came back with a fault gives the view one
/// sentence, and that sentence names what the server said.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary fight for the boxes of the process (T-144 and T-157), and
/// `logic::the_lists` holds such a box.
#[tokio::test(flavor = "multi_thread")]
async fn the_view_of_the_lists_says_that_the_request_came_back_with_a_fault() {
    let address = a_host_that_fails().await;

    let pool = EndpointPool::new(vec![Endpoint::new(&address, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    the_lists::forget();

    let the_library = "1b090ea8-91c5-4591-ac9d-716985e61faf";

    // This is the task of `ask_for_the_lists`, and the two `unwrap_or_else` of
    // `App::new` hold the same road.
    the_lists::ask(&api, the_library).await;

    assert!(
        the_lists::take().is_none(),
        "no answer came, therefore the render must keep the lists that it holds"
    );

    // And this is the line of the render of that view.
    let what_the_server_said = the_lists::the_fault_of(the_library).expect(
        "the request came back with a fault, therefore the view must know it. \
         It knew nothing at all: this is T-169.",
    );

    let text = the_lists::the_reason_of_no_list(false, Some(&what_the_server_said));

    assert!(
        text.starts_with("The server did not give the collections and the playlists:"),
        "the view must say that the server gave no list, and it says {:?}",
        text
    );

    assert!(
        !text.contains("has no collection"),
        "the view must not say a reason that the program does not have (T-91), \
         and it says {:?}",
        text
    );

    assert!(
        text.contains("500"),
        "the sentence must name what the server said, and it says {:?}",
        text
    );

    // The title of the view of the key `m` holds the same rule, and it must not
    // ask the user to make a list of a server that gave no list.
    let title = the_lists::the_title_of_no_list(false, Some(&what_the_server_said));

    assert!(
        !title.contains("Press c or p"),
        "the title says {:?}",
        title
    );

    // **A user who takes the key `S` to another library must not read the fault
    // of the library before it**: the box holds the library of its own request.
    assert_eq!(the_lists::the_fault_of("another-library"), None);

    // A new request of that library takes the fault away, therefore the view
    // says the truth of the library again.
    the_lists::forget_the_fault_of(the_library);

    assert_eq!(the_lists::the_fault_of(the_library), None);
    assert_eq!(
        the_lists::the_reason_of_no_list(false, None),
        "This library has no collection and no playlist.\nPress h to go back."
    );

    the_lists::forget();
}
