//! The key `M` marks a media as finished, or as not finished. See T-24.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_mark_of_finished_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The test changes the progress of one book on the sandbox server. It changes
//! nothing on a server of a user.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::mark_the_media;

const SERVER: &str = "http://127.0.0.1:13399";
const TITLE: &str = "Multi File Test Book";

mod common;
use common::token;

async fn get(token: &str, path: &str) -> serde_json::Value {
    reqwest::Client::new()
        .get(format!("{}{}", SERVER, path))
        .bearer_auth(token)
        .send()
        .await
        .expect("the sandbox server must answer")
        .json()
        .await
        .unwrap_or(serde_json::Value::Null)
}

async fn item_of(token: &str, title: &str) -> String {
    let libraries = get(token, "/api/libraries").await;

    for library in libraries["libraries"].as_array().unwrap_or(&Vec::new()) {
        if library["mediaType"].as_str() != Some("book") {
            continue;
        }

        let id = library["id"].as_str().unwrap_or_default();
        let items = get(token, &format!("/api/libraries/{}/items?limit=200", id)).await;

        for item in items["results"].as_array().unwrap_or(&Vec::new()) {
            if item["media"]["metadata"]["title"].as_str() == Some(title) {
                return item["id"].as_str().unwrap_or_default().to_string();
            }
        }
    }

    panic!("the sandbox must hold the book \"{}\".", title);
}

async fn is_finished(token: &str, item: &str) -> bool {
    get(token, &format!("/api/me/progress/{}", item))
        .await
        .get("isFinished")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_key_marks_a_media_and_it_marks_it_back() {
    let token = token().await;
    let item = item_of(&token, TITLE).await;

    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token.clone()).unwrap());

    // The book starts as "not finished", whatever a test before it did.
    reqwest::Client::new()
        .patch(format!("{}/api/me/progress/{}", SERVER, item))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "isFinished": false }))
        .send()
        .await
        .expect("the server must take the first mark");

    assert!(!is_finished(&token, &item).await);

    let text = mark_the_media(&api, &item, None).await;
    assert!(
        text.contains("is finished now"),
        "the message must say what happened: {}",
        text
    );
    assert!(
        is_finished(&token, &item).await,
        "the server must hold the media as finished"
    );

    let text = mark_the_media(&api, &item, None).await;
    assert!(
        text.contains("not finished"),
        "the message must say what happened: {}",
        text
    );
    assert!(
        !is_finished(&token, &item).await,
        "the server must hold the media as not finished"
    );
}
