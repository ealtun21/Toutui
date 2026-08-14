//! One shelf with no name keeps every shelf of the Home view. See T-190.
//!
//! **The answer of `GET /api/libraries/:id/personalized` is the list of the
//! shelves itself**, therefore serde gave the fault of one row to the whole
//! answer: a shelf that holds no `label` took the six shelves of a library of
//! books and the three shelves of a library of podcasts away together.
//!
//! A measurement of 2026-08-14 with
//! `docs/harness/a_field_of_one_row_goes_away.py`, which answered the path of
//! the personalized view with the body of the sandbox and no `label` of the
//! row 1:
//!
//! | The library | The Home view |
//! |---|---|
//! | `Books`, the row 1 of the shelves holds no `label` | **The server did not give the shelves of this library: The answer of the server is not valid: missing field `label`** |
//! | `Podcasts`, the row 1 of the shelves holds no `label` | the same sentence |
//!
//! The four other shelves of that answer held their media, and the user saw
//! none of them. **The label of a shelf is a name for the user, and it is no
//! address**: the id of each media of that shelf still reaches every request
//! of the program.
//!
//! **This test needs no sandbox and no network.** A host of a raw socket
//! answers the personalized view with a body of this file.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_library_perso_view::get_the_shelves;
use toutui::api::libraries::get_library_perso_view_pod::get_the_shelves_pod;
use toutui::logic::home_view::{group_home, group_home_pod, HomeRow};

/// The shelves of a library of books. The row 1 holds no `label`, and the row
/// 2 holds a name of no character.
const THE_SHELVES_OF_A_BOOK: &str = r#"[
    {"id": "continue-listening", "label": "Continue Listening",
     "entities": [{"id": "a", "media": {}}]},
    {"id": "recently-added",
     "entities": [{"id": "b", "media": {}}, {"id": "c", "media": {}}]},
    {"label": "   ", "entities": [{"id": "d", "media": {}}]}
]"#;

/// The shelves of a library of podcasts. The row 0 holds no `label`.
const THE_SHELVES_OF_A_PODCAST: &str = r#"[
    {"id": "newest-episodes",
     "entities": [{"id": "a", "media": {}, "recentEpisode": {}}]},
    {"id": "listen-again", "label": "Listen Again",
     "entities": [{"id": "b", "media": {}, "recentEpisode": {}}]}
]"#;

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

/// A shelf with no name keeps every shelf of the Home view, and the line of
/// that shelf holds the identity of it.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn one_shelf_with_no_name_keeps_the_media_of_every_shelf() {
    let address = a_host_of_a_body(THE_SHELVES_OF_A_BOOK).await;
    let api = a_client(&address);

    let shelves = get_the_shelves(&api, "a-library")
        .await
        .expect("a shelf with no name must not take the answer away");

    assert_eq!(shelves.len(), 3, "every shelf of the answer stays");

    let rows = group_home(&shelves, &[]);

    assert_eq!(
        rows,
        vec![
            HomeRow::Shelf {
                label: "Continue Listening".to_string()
            },
            HomeRow::Media { item: 0 },
            // The name of the server is absent, therefore the line holds the
            // identity of the shelf.
            HomeRow::Shelf {
                label: "recently-added".to_string()
            },
            HomeRow::Media { item: 1 },
            HomeRow::Media { item: 2 },
            // No name and no identity: the program names the shelf.
            HomeRow::Shelf {
                label: "A shelf with no name".to_string()
            },
            HomeRow::Media { item: 3 },
        ]
    );

    // The same answer of a library of podcasts.
    let address = a_host_of_a_body(THE_SHELVES_OF_A_PODCAST).await;
    let api = a_client(&address);

    let shelves = get_the_shelves_pod(&api, "a-library")
        .await
        .expect("a shelf with no name must not take the answer away");

    assert_eq!(shelves.len(), 2);

    assert_eq!(
        group_home_pod(&shelves),
        vec![
            HomeRow::Shelf {
                label: "newest-episodes".to_string()
            },
            HomeRow::Media { item: 0 },
            HomeRow::Shelf {
                label: "Listen Again".to_string()
            },
            HomeRow::Media { item: 1 },
        ]
    );
}
