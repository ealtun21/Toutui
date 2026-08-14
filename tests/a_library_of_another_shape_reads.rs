//! A library of a field that the program does not read starts the program.
//! See T-176.
//!
//! **The old `Library` asked for every field of Audiobookshelf 2.36.0**, and one
//! field fewer stopped the whole program. A measurement of 2026-08-14 with
//! `docs/harness/another_body_of_the_libraries.py`, which answered
//! `GET /api/libraries` with the body of the sandbox and one field fewer:
//!
//! | The body | The program |
//! |---|---|
//! | No `icon` of the first library | **it stopped** |
//! | No `settings.autoScanCronExpression` | **it stopped** |
//! | No `name` | it stopped |
//! | No JSON at all | it stopped |
//! | A field of a later version | it started |
//!
//! **`icon` and `autoScanCronExpression` reach no line of this program.** A
//! server of another version can hold neither, and the user of that server then
//! reads `Toutui stops: it cannot read the lists of the server.`
//!
//! **One library of a list took every library away** (T-191). A measurement of
//! 2026-08-14 with `docs/harness/a_field_of_one_row_goes_away.py`, which took
//! the `name` of the row 1 of the five libraries of the sandbox away:
//!
//! | The body | The program |
//! |---|---|
//! | No `name` of the row 1 of five | **it stopped**: `the program stops: The answer of the server is not valid: missing field name` |
//! | No `mediaType` of the row 2 of five | it stopped |
//!
//! The four other libraries of that answer held every field, and the user
//! reached none of them. **The id and the media type are the two values that a
//! library needs**, and the name is a word for the user: a library with no
//! name keeps its line now, and that line holds the id.
//!
//! **The words of the fault named nothing.** Every one of the four faults gave
//! `The answer of the server is not valid: error decoding response body`, and
//! those words tell a user, a maintainer, and the log the same thing for three
//! different causes.
//!
//! **This test needs no sandbox and no network.** A host of a raw socket
//! answers the libraries with a body of this file.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_all_libraries::get_all_libraries;

/// The body of a server of another version: the three fields that the program
/// reads, and nothing else.
const THE_LIBRARY_OF_THREE_FIELDS: &str = r#"{"libraries": [
    {"id": "a-library", "name": "Books", "mediaType": "book"},
    {"id": "a-second-library", "name": "Podcasts", "mediaType": "podcast",
     "aFieldOfALaterVersion": {"of": "a later version"}}
]}"#;

/// The body of a server that gives no name to the first of its two libraries.
/// The row of the account of the database holds that name (T-173).
const THE_LIBRARY_OF_NO_NAME: &str = r#"{"libraries": [
    {"id": "a-library", "mediaType": "book", "icon": "database"},
    {"id": "a-second-library", "name": "A Second Library", "mediaType": "book"}
]}"#;

/// The body of a server that holds three libraries of which the program can
/// use one: no id, no media type, and a folder with no address. See T-191.
const THE_LIBRARY_THAT_THE_PROGRAM_CANNOT_USE: &str = r#"{"libraries": [
    {"name": "A Library With No Address", "mediaType": "book"},
    {"id": "a-library-of-no-view", "name": "A Library Of No View"},
    {"id": "the-one-library-of-a-line", "name": "The Library", "mediaType": "book",
     "folders": [{"fullPath": "/a/path"}]}
]}"#;

/// Starts a host that answers every request with one body.
async fn a_host_of_a_body(body: &'static str) -> String {
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

                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );

                let _ = socket.write_all(head.as_bytes()).await;
                let _ = socket.write_all(body.as_bytes()).await;
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

/// The libraries of a body of three fields come, and a body of no name says
/// which field it holds not.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn a_body_of_the_fields_that_the_program_reads_gives_the_libraries() {
    let address = a_host_of_a_body(THE_LIBRARY_OF_THREE_FIELDS).await;
    let api = a_client(&address);

    let all = get_all_libraries(&api)
        .await
        .expect("a library of the three fields of this program must come");

    assert_eq!(all.libraries.len(), 2);
    assert_eq!(all.libraries[0].id, "a-library");
    assert_eq!(all.libraries[0].name, "Books");
    assert_eq!(all.libraries[0].media_type, "book");

    // The fields that the program does not read take a value of nothing, and no
    // line of the program asks for them.
    assert!(all.libraries[0].folders.is_empty());
    assert_eq!(all.libraries[0].icon, "");

    // A field of a later version changes nothing.
    assert_eq!(all.libraries[1].name, "Podcasts");
    assert_eq!(all.libraries[1].media_type, "podcast");

    // **A library with no name keeps its line, and that line holds the id.**
    // T-176 gave the fault of that one row to the whole answer, and a
    // measurement of T-191 with a list of five libraries showed the cost: the
    // program did not start at all.
    let address = a_host_of_a_body(THE_LIBRARY_OF_NO_NAME).await;
    let api = a_client(&address);

    let all = get_all_libraries(&api)
        .await
        .expect("a library with no name must not take the answer away");

    assert_eq!(
        all.libraries.len(),
        2,
        "the library with no name keeps its line"
    );
    assert_eq!(all.libraries[0].name, "a-library", "the line holds the id");
    assert_eq!(all.libraries[1].name, "A Second Library");

    // **A library with no id has no address**, and a library with no media
    // type gives no view: neither belongs to a line, and every other library
    // of the same answer stays. See T-191.
    let address = a_host_of_a_body(THE_LIBRARY_THAT_THE_PROGRAM_CANNOT_USE).await;
    let api = a_client(&address);

    let all = get_all_libraries(&api)
        .await
        .expect("a library that the program cannot use must not take the answer away");

    assert_eq!(all.libraries.len(), 1);
    assert_eq!(all.libraries[0].id, "the-one-library-of-a-line");

    // A folder with no address is no folder, and the library keeps its line.
    assert!(all.libraries[0].folders.is_empty());
}
