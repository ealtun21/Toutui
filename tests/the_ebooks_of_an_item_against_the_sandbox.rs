//! An item can hold more than one ebook. See T-76.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! cargo test --test the_ebooks_of_an_item_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The sandbox needs an item with two ebooks. `docs/TEST-SERVER.md` holds the
//! commands, in the section "An item with two ebooks". A sandbox with no such
//! item gives a line of text, and the test does not fail: the program takes the
//! book of the server then, and that is the work of T-10.
//!
//! The test writes no value on the server. It reads the list, and it gets the
//! file of each book.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::library_items::the_ebooks::the_ebooks_of_the_item;
use toutui::logic::reader::session::get_the_ebook_of;

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

/// Gives the identity of every item of every library of books.
async fn every_item(api: &Arc<ApiClient>) -> Vec<String> {
    let libraries: serde_json::Value = api
        .get_json("/api/libraries")
        .await
        .expect("the server must give the libraries");

    let mut all = Vec::new();

    for library in libraries["libraries"].as_array().unwrap_or(&Vec::new()) {
        if library["mediaType"].as_str() != Some("book") {
            continue;
        }

        let id = library["id"].as_str().unwrap_or_default();

        let items: serde_json::Value = api
            .get_json(&format!("/api/libraries/{}/items?limit=50", id))
            .await
            .expect("the server must give the items");

        for item in items["results"].as_array().unwrap_or(&Vec::new()) {
            all.push(item["id"].as_str().unwrap_or_default().to_string());
        }
    }

    all
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox of docs/TEST-SERVER.md on :13399"]
async fn the_program_lists_every_ebook_of_an_item_and_it_gets_each_of_them() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("XDG_DATA_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token().await).unwrap());

    let mut of_two_books = None;

    for item in every_item(&api).await {
        let all = the_ebooks_of_the_item(&api, &item)
            .await
            .expect("the server must give the item");

        if all.len() > 1 {
            of_two_books = Some((item, all));
            break;
        }
    }

    let Some((item_id, all)) = of_two_books else {
        println!(
            "no item of this sandbox holds two ebooks. See the section \"An \
             item with two ebooks\" of docs/TEST-SERVER.md."
        );
        return;
    };

    println!("the item {} holds {} ebooks", item_id, all.len());

    assert!(
        all[0].is_the_book_of_the_server,
        "the book of the server must stand first, and the list holds {:?}",
        all
    );

    assert_eq!(
        all.iter()
            .filter(|one| one.is_the_book_of_the_server)
            .count(),
        1,
        "one book of the item is the book of the server"
    );

    // Each book must give its own file, and the two files must not be the same
    // file.
    let mut sizes = Vec::new();

    for one in &all {
        let ino = if one.is_the_book_of_the_server {
            None
        } else {
            Some(one.ino.as_str())
        };

        let path = get_the_ebook_of(&api, USER, &item_id, ino)
            .await
            .unwrap_or_else(|message| panic!("the book {} did not come: {}", one.name, message));

        let size = std::fs::metadata(&path)
            .expect("the file of the book must stand on the disk")
            .len();

        println!(
            "  {} gives {} of {} bytes",
            one.line(),
            path.display(),
            size
        );

        assert!(size > 0, "the file of the book {} is empty", one.name);

        sizes.push((path, size));
    }

    let names: std::collections::BTreeSet<_> = sizes.iter().map(|(path, _)| path).collect();

    assert_eq!(
        names.len(),
        all.len(),
        "each book of the item must take its own name on the disk, and the \
         program wrote {:?}",
        sizes
    );

    // The key `X` removes every book of the item. See T-65 and T-76.
    let bytes = toutui::logic::download::remove_the_ebook_of_the_item(&item_id, USER);

    assert_eq!(
        bytes,
        sizes.iter().map(|(_, size)| size).sum::<u64>(),
        "the key X must remove every book of the item"
    );

    for (path, _) in &sizes {
        assert!(!path.exists(), "the file {} stays", path.display());
    }
}
