//! The three other views of the start say why they hold no line. See T-170.
//!
//! **A server can answer some requests and fail others.** A measurement of
//! 2026-08-14 with the sandbox, tmux, and `docs/harness/one_path_fails.py` gave
//! the status 500 to `/personalized`, to `/series`, and to `/items` of the
//! library `Books`, and it forwarded every other request. That library holds 17
//! books and three series, and the program said:
//!
//! - `This library holds no media.` and `Press L to tell the server to examine
//!   the library.`
//! - `This library has no series.`
//! - `The server gave no shelf for this library.`
//!
//! `is_offline` of `App` holds the offline mode of the **start** (T-25), and
//! the server of this condition answers: the words of T-91 for a server that
//! does not answer never came. This is the cause of T-168 and of T-169 too.
//!
//! **This test needs no sandbox.** A host of a raw socket answers `500` to
//! every request.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_all_series::get_all_series;
use toutui::logic::the_requests_of_the_start::{
    keep_the_fault, the_fault_of, the_reason_of_no_series, TheRequest,
};
use toutui::ui::keys::{
    the_text_of_the_home_view_with_no_line, the_text_of_the_library_view_with_no_line,
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

/// The three requests of the start that came back with a fault give their views
/// one sentence, and each sentence names what the server said.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary fight for the boxes of the process (T-144 and T-157), and
/// `logic::the_requests_of_the_start` holds such a box.
#[tokio::test(flavor = "multi_thread")]
async fn the_three_views_of_the_start_say_that_the_request_came_back_with_a_fault() {
    let address = a_host_that_fails().await;

    let pool = EndpointPool::new(vec![Endpoint::new(&address, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    toutui::logic::the_requests_of_the_start::forget();

    let the_library = "1b090ea8-91c5-4591-ac9d-716985e61faf";

    let error = get_all_series(&api, the_library)
        .await
        .expect_err("the host of this test gives a fault to every request");

    // These are the lines of the three `unwrap_or_else` of `App::new`. The
    // fault of one request is the fault of every one of them: the server of
    // this test answers the same way.
    for which in [TheRequest::Shelves, TheRequest::Series, TheRequest::Items] {
        keep_the_fault(the_library, which, error.to_string().as_str());
    }

    // And these are the lines of the render of the three views.
    let of_the_series = the_fault_of(the_library, TheRequest::Series).expect(
        "the request came back with a fault, therefore the view must know it. \
         It knew nothing at all: this is T-170.",
    );

    let text = the_reason_of_no_series(false, Some(&of_the_series));

    assert!(
        text.starts_with("The server did not give the series of this library:"),
        "the view says {:?}",
        text
    );
    assert!(
        !text.contains("has no series"),
        "the view must not say a reason that the program does not have (T-91), \
         and it says {:?}",
        text
    );
    assert!(text.contains("500"), "the view says {:?}", text);

    let of_the_items =
        the_fault_of(the_library, TheRequest::Items).expect("the box holds the fault of the items");

    let text = the_text_of_the_library_view_with_no_line(false, false, false, Some(&of_the_items));

    assert!(
        text.starts_with("The server did not give the media of this library:"),
        "the Library view says {:?}",
        text
    );
    assert!(
        !text.contains("Press L"),
        "a text must not promise a key that does no work of this fault (T-118), \
         and it says {:?}",
        text
    );

    let of_the_shelves = the_fault_of(the_library, TheRequest::Shelves)
        .expect("the box holds the fault of the shelves");

    let text = the_text_of_the_home_view_with_no_line(false, Some(&of_the_shelves));

    assert!(
        text.starts_with("The server did not give the shelves of this library:"),
        "the Home view says {:?}",
        text
    );

    // **A user who takes the key `S` to another library must not read the fault
    // of the library before it**: the box holds one library.
    assert_eq!(the_fault_of("another-library", TheRequest::Items), None);

    // The key `R` makes the application again, and its first line takes the
    // faults of this library away: the three views say the truth of the library
    // then.
    toutui::logic::the_requests_of_the_start::forget_the_faults_of(the_library);

    assert_eq!(the_fault_of(the_library, TheRequest::Items), None);
    assert_eq!(
        the_reason_of_no_series(false, None),
        "This library has no series.\nPress h to go back."
    );

    toutui::logic::the_requests_of_the_start::forget();
}
