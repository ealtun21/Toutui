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
//! **The words of the fault named nothing.** Every one of the four faults gave
//! `The answer of the server is not valid: error decoding response body`, and
//! those words tell a user, a maintainer, and the log the same thing for three
//! different causes.
//!
//! **This test needs no sandbox and no network.** A host of a raw socket
//! answers the libraries with a body of this file.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::error::ApiError;
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_all_libraries::get_all_libraries;

/// The body of a server of another version: the three fields that the program
/// reads, and nothing else.
const THE_LIBRARY_OF_THREE_FIELDS: &str = r#"{"libraries": [
    {"id": "a-library", "name": "Books", "mediaType": "book"},
    {"id": "a-second-library", "name": "Podcasts", "mediaType": "podcast",
     "aFieldOfALaterVersion": {"of": "a later version"}}
]}"#;

/// The body of a server that gives no name to its library. The row of the
/// account of the database holds that name (T-173).
const THE_LIBRARY_OF_NO_NAME: &str = r#"{"libraries": [
    {"id": "a-library", "mediaType": "book", "icon": "database"}
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

    // **A body that holds no name is not the answer of this endpoint**, and the
    // words of that fault name the field.
    let address = a_host_of_a_body(THE_LIBRARY_OF_NO_NAME).await;
    let api = a_client(&address);

    let error = get_all_libraries(&api)
        .await
        .expect_err("a library of no name must give a fault");

    let ApiError::Decode(detail) = error else {
        panic!(
            "the fault of a body must be a fault of the decode, and it is {:?}",
            error
        );
    };

    assert!(
        detail.contains("missing field `name`"),
        "the words of the fault must name the field, and they say {:?}",
        detail
    );
}
