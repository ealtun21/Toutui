//! An author of the server with no identity takes no line. See T-386.
//!
//! The measurement of the real program v0.8.216 inside tmux against the
//! sandbox, with `docs/harness/a_field_of_one_row_goes_away.py` on the path
//! `/api/libraries/:id/authors` (the row 0 loses `id`): the view of the key
//! `a` kept the line `Test Author [2 book(s)]`, the key `l` of that line
//! wrote the filter `authors.` (an identity of no character) into the row of
//! the account, the request `filter=authors.` gave 0 items, and the view
//! said `No media of this library agrees with the filter.` The two books of
//! that author stood in the library, and no view of the program could name
//! or show the filter that hid them.
//!
//! **The identity of an author is the address of its one key** (the rule of
//! T-183 and of T-192): the key `l` of the line is the filter of that
//! identity, therefore an author with no identity belongs to no line. A
//! narrator holds no row of its own on the server, and the filter of a
//! narrator takes the name (T-73): a narrator with no name belongs to no
//! line for the same reason.
//!
//! The parts of this test stay in one function: two test functions of one
//! module fight for the slot of that module in the run of `cargo test`.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_authors::{get_authors, get_narrators};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(url: &str) -> ApiClient {
    ApiClient::new(
        Arc::new(EndpointPool::new(vec![Endpoint::new(url, 0)])),
        "test-token".to_string(),
    )
    .unwrap()
}

#[tokio::test]
async fn an_author_with_no_identity_takes_no_line() {
    let server = MockServer::start().await;

    // The answer of the server: one author lost its identity, in the shape
    // of the measurement of the harness of T-181.
    Mock::given(method("GET"))
        .and(path("/api/libraries/lib1/authors"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "authors": [
                { "name": "A Ghost Author", "numBooks": 2 },
                { "id": "id-of-the-real-author", "name": "A Real Author", "numBooks": 1 }
            ]
        })))
        .mount(&server)
        .await;

    // One narrator lost its name, which is the address of its filter.
    Mock::given(method("GET"))
        .and(path("/api/libraries/lib1/narrators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "narrators": [
                { "id": "", "name": "", "numBooks": 1 },
                { "id": "QQ==", "name": "A Real Narrator", "numBooks": 1 }
            ]
        })))
        .mount(&server)
        .await;

    let client = client(&server.uri());

    // The author with no identity takes no line, and the author with an
    // identity keeps its line.
    let authors = get_authors(&client, "lib1").await.unwrap();
    assert_eq!(
        authors.len(),
        1,
        "the author with no identity must take no line: {authors:?}"
    );
    assert_eq!(authors[0].name, "A Real Author");

    // The narrator with no name takes no line, and the narrator with a name
    // keeps its line.
    let narrators = get_narrators(&client, "lib1").await.unwrap();
    assert_eq!(
        narrators.len(),
        1,
        "the narrator with no name must take no line: {narrators:?}"
    );
    assert_eq!(narrators[0].name, "A Real Narrator");
}
