//! The bookmarks of the user, against a real server. See T-24.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_bookmarks_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The test writes two bookmarks on the sandbox server and it removes them.
//! It changes nothing on a server of a user.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::me::bookmarks::{add_bookmark, get_bookmarks, of_item, remove_bookmark};

const SERVER: &str = "http://127.0.0.1:13399";
const TITLE: &str = "A Long Test Book";

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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_program_writes_reads_and_removes_a_bookmark() {
    let token = token().await;
    let item = item_of(&token, TITLE).await;

    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token.clone()).unwrap());

    // A test before this one can leave a bookmark. The two places go away
    // first, and a place that does not exist gives an error that this test
    // accepts.
    let _ = remove_bookmark(&api, &item, 61.0).await;
    let _ = remove_bookmark(&api, &item, 123.0).await;

    let start = of_item(&get_bookmarks(&api).await.unwrap(), &item).len();

    add_bookmark(&api, &item, 123.4, "The second place")
        .await
        .expect("the server must take the bookmark");
    add_bookmark(&api, &item, 61.0, "The first place")
        .await
        .expect("the server must take the bookmark");

    let mine = of_item(&get_bookmarks(&api).await.unwrap(), &item);
    println!("{:?}", mine);

    assert_eq!(mine.len(), start + 2);

    // The first place comes first, and the server gives no sequence.
    let places: Vec<f64> = mine.iter().map(|one| one.time).collect();
    let mut sorted = places.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(places, sorted);

    // `123.4` goes to the server as `123`, because the address of the delete
    // holds a whole number.
    let written = mine
        .iter()
        .find(|one| one.title == "The second place")
        .expect("the server must hold the bookmark");
    assert_eq!(written.time, 123.0);

    // The same place a second time changes the name, and it makes no second
    // line.
    add_bookmark(&api, &item, 123.0, "A new name")
        .await
        .expect("the server must take the bookmark");

    let mine = of_item(&get_bookmarks(&api).await.unwrap(), &item);
    assert_eq!(mine.len(), start + 2, "the same place must give one line");
    assert!(mine.iter().any(|one| one.title == "A new name"));

    remove_bookmark(&api, &item, 123.0)
        .await
        .expect("the server must remove the bookmark");
    remove_bookmark(&api, &item, 61.0)
        .await
        .expect("the server must remove the bookmark");

    let mine = of_item(&get_bookmarks(&api).await.unwrap(), &item);
    assert_eq!(mine.len(), start);

    // A place that does not exist gives `404`, and the program must give an
    // error and not stop.
    assert!(remove_bookmark(&api, &item, 999_999.0).await.is_err());
}
