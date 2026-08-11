//! The description of a series comes with the list of the series. See T-24.
//!
//! `GET /api/series/:id` gives the description of one series. The reference
//! names that endpoint, and a reader of the reference adds a request for each
//! series. **A measurement on 2026-08-11 shows that the request is not
//! necessary:** `GET /api/libraries/:id/series`, which the program asks for
//! already, gives the same `description` for every series of the page.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_series_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! **The test writes.** It gives a description to one series of the sandbox
//! with `PATCH /api/series/:id`, because a series of a new sandbox has none and
//! an empty value shows no shape. It touches no other server.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_all_series::get_all_series;
use toutui::api::utils::collect_series::collect_series;

const SERVER: &str = "http://127.0.0.1:13399";

/// The text that the test writes on the server.
const DESCRIPTION: &str = "Three books of a test. The series has a description.";

mod common;
use common::token;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399, and it writes"]
async fn the_list_of_the_series_carries_the_description() {
    let token = token().await;
    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token.clone()).unwrap());

    let libraries: serde_json::Value = api
        .get_json("/api/libraries")
        .await
        .expect("the server must give the libraries");

    let library = libraries["libraries"]
        .as_array()
        .and_then(|all| all.iter().find(|one| one["mediaType"] == "book"))
        .and_then(|one| one["id"].as_str())
        .expect("the sandbox must hold a library of books")
        .to_string();

    // The data must exist first. An empty value shows no shape.
    let answer = get_all_series(&api, &library)
        .await
        .expect("the server must give the series");
    let first = collect_series(&answer);
    let first = first
        .first()
        .expect("the sandbox must hold a series. See docs/TEST-SERVER.md, 6b.");

    let outcome = reqwest::Client::new()
        .patch(format!("{}/api/series/{}", SERVER, first.id))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "description": DESCRIPTION }))
        .send()
        .await
        .expect("the server must answer");

    assert_eq!(200, outcome.status().as_u16(), "PATCH /api/series/:id");

    // The list that the program asks for already must hold the same text.
    let answer = get_all_series(&api, &library)
        .await
        .expect("the server must give the series");
    let series = collect_series(&answer);

    let one = series
        .iter()
        .find(|series| series.id == first.id)
        .expect("the list must hold the series again");

    println!("the series {:?} gives {:?}", one.name, one.description);

    assert_eq!(
        DESCRIPTION, one.description,
        "the list of the series must carry the description"
    );

    // The screen shows that text, and not the description of the first book.
    assert_eq!(DESCRIPTION, one.description_for_the_screen());

    // A series with no description shows the description of its first book.
    // See T-43.
    for other in &series {
        if other.id == first.id {
            continue;
        }

        println!(
            "the series {:?} has no description, and the screen shows {:?}",
            other.name,
            other.description_for_the_screen()
        );
    }
}
