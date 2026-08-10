//! The sequence and the filter of a library come from the server. See T-24.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_sequence_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The test reads only. It changes nothing on the server.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_all_books::get_all_books;
use toutui::api::libraries::get_filter_data::{choices, get_filter_data};
use toutui::logic::sort_filter::query;

const SERVER: &str = "http://127.0.0.1:13399";
const USER: &str = "toutuitest";
const PASSWORD: &str = "toutuitest";

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

async fn api(token: &str) -> Arc<ApiClient> {
    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    Arc::new(ApiClient::new(Arc::new(pool), token.to_string()).unwrap())
}

async fn library_of_books(token: &str) -> String {
    let libraries: serde_json::Value = reqwest::Client::new()
        .get(format!("{}/api/libraries", SERVER))
        .bearer_auth(token)
        .send()
        .await
        .expect("the sandbox server must answer")
        .json()
        .await
        .expect("the answer must hold JSON");

    for library in libraries["libraries"]
        .as_array()
        .expect("the answer must hold the libraries")
    {
        if library["mediaType"].as_str() == Some("book")
            && library["name"].as_str() == Some("Books")
        {
            return library["id"].as_str().unwrap_or_default().to_string();
        }
    }

    panic!("the sandbox must hold the library \"Books\".");
}

fn titles(root: &toutui::api::libraries::get_all_books::Root) -> Vec<String> {
    root.results
        .as_ref()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.media.as_ref())
                .filter_map(|media| media.metadata.as_ref())
                .filter_map(|metadata| metadata.title.clone())
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_sequence_of_the_user_reaches_the_server() {
    let token = token().await;
    let api = api(&token).await;
    let library = library_of_books(&token).await;

    let by_the_title = titles(
        &get_all_books(&api, &library, &query("media.metadata.title", false, ""))
            .await
            .expect("the server must give the items"),
    );

    let the_other_way = titles(
        &get_all_books(&api, &library, &query("media.metadata.title", true, ""))
            .await
            .expect("the server must give the items"),
    );

    println!("the first three by the title: {:?}", &by_the_title[..3]);
    println!("the first three the other way: {:?}", &the_other_way[..3]);

    assert!(by_the_title.len() > 2, "the sandbox must hold books");
    assert_eq!(by_the_title.len(), the_other_way.len());

    let mut turned = the_other_way.clone();
    turned.reverse();
    assert_eq!(
        by_the_title, turned,
        "`desc=1` must give the other direction"
    );

    // The request with no choice is the request of before this work.
    let of_the_server = titles(
        &get_all_books(&api, &library, "")
            .await
            .expect("the server must give the items"),
    );
    assert_eq!(of_the_server.len(), by_the_title.len());
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_filter_of_the_user_reaches_the_server() {
    let token = token().await;
    let api = api(&token).await;
    let library = library_of_books(&token).await;

    let data = get_filter_data(&api, &library)
        .await
        .expect("the server must give the values of the filter");

    let all = choices(&data);
    println!("the values of the filter: {}", all.len());

    let author = all
        .iter()
        .find(|one| one.group == "The authors")
        .expect("the sandbox must hold an author");

    println!("the filter of {}: {}", author.label, author.value);

    let whole = titles(
        &get_all_books(&api, &library, "")
            .await
            .expect("the server must give the items"),
    );

    let filtered = titles(
        &get_all_books(&api, &library, &query("", false, &author.value))
            .await
            .expect("the server must give the items"),
    );

    println!("the books of that author: {:?}", filtered);

    assert!(!filtered.is_empty(), "the author must have a book");
    assert!(
        filtered.len() < whole.len(),
        "the filter must give fewer books than the whole library"
    );

    for title in &filtered {
        assert!(whole.contains(title));
    }
}
