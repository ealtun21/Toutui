//! The key `N` takes a media away from Continue Listening. See T-24.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_shelf_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The test changes the progress of one book on the sandbox server, and it
//! puts the value back. It changes nothing on a server of a user.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_library_perso_view::{
    get_the_shelves, is_the_shelf_of_continue_listening,
};
use toutui::app::hide_the_media;

const SERVER: &str = "http://127.0.0.1:13399";
const USER: &str = "toutuitest";
const PASSWORD: &str = "toutuitest";
const TITLE: &str = "A Long Test Book";

async fn token() -> String {
    let answer: serde_json::Value = reqwest::Client::new()
        .post(format!("{}/login", SERVER))
        .json(&serde_json::json!({ "username": USER, "password": PASSWORD }))
        .send()
        .await
        .expect("the sandbox server must answer")
        .json()
        .await
        .expect("the answer of the login must hold JSON");

    answer["user"]["token"]
        .as_str()
        .expect("the answer must hold a token")
        .to_string()
}

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

async fn library_and_item(token: &str, title: &str) -> (String, String) {
    let libraries = get(token, "/api/libraries").await;

    for library in libraries["libraries"].as_array().unwrap_or(&Vec::new()) {
        if library["mediaType"].as_str() != Some("book") {
            continue;
        }

        let id = library["id"].as_str().unwrap_or_default();
        let items = get(token, &format!("/api/libraries/{}/items?limit=200", id)).await;

        for item in items["results"].as_array().unwrap_or(&Vec::new()) {
            if item["media"]["metadata"]["title"].as_str() == Some(title) {
                return (
                    id.to_string(),
                    item["id"].as_str().unwrap_or_default().to_string(),
                );
            }
        }
    }

    panic!("the sandbox must hold the book \"{}\".", title);
}

async fn is_hidden(token: &str, item: &str) -> bool {
    get(token, &format!("/api/me/progress/{}", item))
        .await
        .get("hideFromContinueListening")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

/// Gives the titles of the shelf of Continue Listening.
async fn on_the_shelf(api: &Arc<ApiClient>, library: &str) -> Vec<String> {
    get_the_shelves(api, library)
        .await
        .expect("the server must give the shelves")
        .iter()
        .filter(|shelf| is_the_shelf_of_continue_listening(shelf))
        .flat_map(|shelf| shelf.entities.iter().flatten())
        .filter_map(|entity| entity.media.as_ref())
        .filter_map(|media| media.metadata.as_ref())
        .filter_map(|metadata| metadata.title.clone())
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_key_takes_a_media_away_from_the_shelf_and_it_puts_it_back() {
    let token = token().await;
    let (library, item) = library_and_item(&token, TITLE).await;

    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token.clone()).unwrap());

    // The book starts on the shelf, whatever a test before it did. A book
    // with no position does not stand on the shelf, therefore the test gives
    // it one.
    reqwest::Client::new()
        .patch(format!("{}/api/me/progress/{}", SERVER, item))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "progress": 0.5,
            "currentTime": 900,
            "hideFromContinueListening": false
        }))
        .send()
        .await
        .expect("the server must take the first position");

    assert!(!is_hidden(&token, &item).await);
    assert!(
        on_the_shelf(&api, &library)
            .await
            .contains(&TITLE.to_string()),
        "the book must stand on the shelf before the key"
    );

    let text = hide_the_media(&api, &item).await;
    println!("{}", text);
    assert!(text.contains("away from Continue Listening"));
    assert!(is_hidden(&token, &item).await);
    assert!(
        !on_the_shelf(&api, &library)
            .await
            .contains(&TITLE.to_string()),
        "the shelf of the server must not hold the book now"
    );

    let text = hide_the_media(&api, &item).await;
    println!("{}", text);
    assert!(text.contains("on Continue Listening again"));
    assert!(!is_hidden(&token, &item).await);
    assert!(
        on_the_shelf(&api, &library)
            .await
            .contains(&TITLE.to_string()),
        "the book must come back to the shelf"
    );
}
