//! The media of a collection and of a playlist. See T-84.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! cargo test --test the_lists_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! **The test changes the lists of the sandbox**, and it gives them back: it
//! puts one book in the first collection and in the first playlist, and it takes
//! that book out again. A sandbox with no collection and no playlist gives a
//! line of text, and the test does not fail.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::libraries::get_lists::{get_all_collections, get_all_playlists};
use toutui::api::lists::{put_in_the_list, take_out_of_the_list};
use toutui::api::utils::collect_lists::{collect_lists, ListKind, ListView};

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

/// Gives the library of the books, its lists, and one book that stands in no
/// list.
async fn the_library(api: &Arc<ApiClient>) -> Option<(String, Vec<ListView>, String, String)> {
    let libraries: serde_json::Value = api
        .get_json("/api/libraries")
        .await
        .expect("the server must give the libraries");

    for library in libraries["libraries"].as_array()? {
        if library["mediaType"].as_str() != Some("book") {
            continue;
        }

        let library_id = library["id"].as_str()?.to_string();

        let collections = get_all_collections(api, &library_id).await.ok()?;
        let playlists = get_all_playlists(api, &library_id).await.ok()?;
        let lists = collect_lists(&collections, &playlists);

        if lists.is_empty() {
            continue;
        }

        let items: serde_json::Value = api
            .get_json(&format!("/api/libraries/{}/items?limit=50", library_id))
            .await
            .ok()?;

        // A book that no list of this library holds. The test gives the lists
        // back at its end, and a book that stands in a list already would give
        // "the list holds it already" and no measurement.
        let of_the_lists: Vec<String> = lists
            .iter()
            .flat_map(|list| list.entries.iter().map(|entry| entry.id.clone()))
            .collect();

        for item in items["results"].as_array()? {
            let id = item["id"].as_str()?.to_string();

            if of_the_lists.contains(&id) {
                continue;
            }

            let title = item["media"]["metadata"]["title"]
                .as_str()
                .unwrap_or("A book")
                .to_string();

            return Some((library_id, lists, id, title));
        }
    }

    None
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox of docs/TEST-SERVER.md on :13399"]
async fn the_program_puts_a_book_in_a_list_and_it_takes_it_out() {
    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token().await).unwrap());

    let Some((library_id, lists, item_id, title)) = the_library(&api).await else {
        println!(
            "this sandbox holds no list, or every book stands in a list already. \
             See docs/TEST-SERVER.md, section 6d."
        );
        return;
    };

    println!("the test puts \"{}\" in {} list(s)", title, lists.len());

    for list in &lists {
        // The book goes in the list.
        let came = put_in_the_list(&api, list.kind, &list.id, &item_id, None)
            .await
            .unwrap_or_else(|error| panic!("the server did not take the book: {}", error));

        assert!(came, "the list \"{}\" held the book already", list.name);

        // **The server answers 400 for a book that stands in the list**, and
        // that answer is not a fault of the program. See T-84.
        let again = put_in_the_list(&api, list.kind, &list.id, &item_id, None)
            .await
            .expect("the second request must give an answer");

        assert!(
            !again,
            "the server must say that the {} \"{}\" holds the book already",
            list.kind.name().to_lowercase(),
            list.name
        );
    }

    // The lists of the server hold the book now.
    let collections = get_all_collections(&api, &library_id).await.unwrap();
    let playlists = get_all_playlists(&api, &library_id).await.unwrap();
    let now = collect_lists(&collections, &playlists);

    for list in &lists {
        let of_the_server = now
            .iter()
            .find(|one| one.id == list.id)
            .unwrap_or_else(|| panic!("the list \"{}\" must stay", list.name));

        assert!(
            of_the_server
                .entries
                .iter()
                .any(|entry| entry.id == item_id),
            "the {} \"{}\" must hold the book",
            list.kind.name().to_lowercase(),
            list.name
        );
    }

    // The test gives the lists back.
    for list in &lists {
        take_out_of_the_list(&api, list.kind, &list.id, &item_id, None)
            .await
            .unwrap_or_else(|error| panic!("the server did not take the book out: {}", error));
    }

    let collections = get_all_collections(&api, &library_id).await.unwrap();
    let playlists = get_all_playlists(&api, &library_id).await.unwrap();
    let at_the_end = collect_lists(&collections, &playlists);

    for list in &lists {
        let of_the_server = at_the_end
            .iter()
            .find(|one| one.id == list.id)
            .unwrap_or_else(|| panic!("the list \"{}\" must stay", list.name));

        assert!(
            !of_the_server
                .entries
                .iter()
                .any(|entry| entry.id == item_id),
            "the {} \"{}\" must not hold the book any more",
            list.kind.name().to_lowercase(),
            list.name
        );

        assert_eq!(
            of_the_server.entries.len(),
            list.entries.len(),
            "the {} \"{}\" must hold the media that it held before the test",
            list.kind.name().to_lowercase(),
            list.name
        );
    }

    // A collection holds books, and the program must not offer an episode.
    assert_eq!(ListKind::Collection.name(), "Collection");
}
