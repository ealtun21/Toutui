//! Add a podcast, against a real server. See T-24.
//!
//! Continuous integration does not run this test, because it needs a server,
//! and because the server needs the network for the search. Start the sandbox
//! of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_new_podcast_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! **The test writes a new podcast in the library of the sandbox, and it
//! removes it at the end.** It changes nothing on a server of a user.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::podcasts::{body_for, create_podcast, get_feed, lines, search_podcast};

const SERVER: &str = "http://127.0.0.1:13399";
const USER: &str = "toutuitest";
const PASSWORD: &str = "toutuitest";
const WORDS: &str = "balzac";

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

async fn library_of_podcasts(token: &str) -> (String, String, String) {
    let libraries = get(token, "/api/libraries").await;

    for library in libraries["libraries"].as_array().unwrap_or(&Vec::new()) {
        if library["mediaType"].as_str() != Some("podcast") {
            continue;
        }

        let id = library["id"].as_str().unwrap_or_default().to_string();
        let whole = get(token, &format!("/api/libraries/{}", id)).await;
        let folder = &whole["folders"][0];

        return (
            id,
            folder["id"].as_str().unwrap_or_default().to_string(),
            folder["fullPath"].as_str().unwrap_or_default().to_string(),
        );
    }

    panic!("the sandbox must hold a library of podcasts.");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399, and the network"]
async fn the_program_finds_a_podcast_and_it_adds_it() {
    let token = token().await;
    let (library, folder_id, folder_path) = library_of_podcasts(&token).await;

    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token.clone()).unwrap());

    let all = search_podcast(&api, WORDS)
        .await
        .expect("the server must search");

    println!("the server found {} podcasts", all.len());
    for text in lines(&all).iter().take(3) {
        println!("  {}", text);
    }

    assert!(!all.is_empty(), "the server must find a podcast");
    assert!(all.len() <= toutui::api::podcasts::LIMIT);

    let one = all
        .iter()
        .find(|one| !one.feed_url.is_empty())
        .expect("an answer must hold a feed");

    let feed = get_feed(&api, &one.feed_url)
        .await
        .expect("the server must read the feed");

    println!(
        "the feed of \"{}\" holds {} episode(s)",
        feed.metadata.title.clone().unwrap_or_default(),
        feed.episodes.len()
    );

    assert!(feed.metadata.title.is_some());
    assert!(feed.metadata.feed_url.is_some());

    let body = body_for(&feed, &library, &folder_id, &folder_path);
    println!("the path of the new podcast: {}", body["path"]);

    // The path must stay inside the folder of the library.
    let path = body["path"].as_str().unwrap_or_default();
    assert!(path.starts_with(&folder_path));
    assert!(!path.contains(".."));

    let made = create_podcast(&api, &body)
        .await
        .expect("the server must add the podcast");

    assert!(!made.id.is_empty());
    println!("the new podcast has the identity {}", made.id);

    // The library holds it now.
    let items = get(
        &token,
        &format!("/api/libraries/{}/items?limit=200", library),
    )
    .await;
    let titles: Vec<String> = items["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|item| item["media"]["metadata"]["title"].as_str())
        .map(|title| title.to_string())
        .collect();

    assert!(titles.contains(&feed.metadata.title.clone().unwrap_or_default()));

    // The test puts the library back as it was.
    let removed = reqwest::Client::new()
        .delete(format!("{}/api/items/{}?hard=1", SERVER, made.id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("the server must answer");

    assert!(removed.status().is_success(), "the test must clean up");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399, and the network"]
async fn the_server_gets_the_episodes_that_it_does_not_hold() {
    let token = token().await;
    let (library, folder_id, folder_path) = library_of_podcasts(&token).await;

    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token.clone()).unwrap());

    // A podcast that the program adds now holds no episode. The feed then
    // holds every episode, and the server holds none.
    let all = search_podcast(&api, WORDS)
        .await
        .expect("the server must search");
    let one = all
        .iter()
        .find(|one| one.track_count > 0 && one.track_count < 6 && !one.feed_url.is_empty())
        .expect("the search must find a short podcast");

    let feed = get_feed(&api, &one.feed_url)
        .await
        .expect("the server must read the feed");
    let body = body_for(&feed, &library, &folder_id, &folder_path);
    let made = create_podcast(&api, &body)
        .await
        .expect("the server must add the podcast");

    // `checknew` gives nothing for a podcast that came now: that endpoint
    // compares with the time of the last examination. Therefore the program
    // reads the feed and it compares itself. See T-24.
    let answer = get(&token, &format!("/api/podcasts/{}/checknew", made.id)).await;
    let new_of_the_server = answer["episodes"]
        .as_array()
        .map(|all| all.len())
        .unwrap_or(0);
    println!("checknew gives {} episode(s)", new_of_the_server);

    let item = get(&token, &format!("/api/items/{}", made.id)).await;
    let held: Vec<serde_json::Value> = item["media"]["episodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let asked = toutui::api::podcasts::missing(&feed.episodes, &held);
    println!(
        "the feed holds {}, the server holds {}, the program asks for {}",
        feed.episodes.len(),
        held.len(),
        asked.len()
    );

    assert_eq!(asked.len(), feed.episodes.len());
    assert_eq!(
        new_of_the_server, 0,
        "checknew gives nothing for a new podcast"
    );

    toutui::api::podcasts::download_episodes(&api, &made.id, &asked[..1])
        .await
        .expect("the server must take the request");

    let mut count = 0;

    for _ in 0..40 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let item = get(&token, &format!("/api/items/{}", made.id)).await;
        count = item["media"]["episodes"]
            .as_array()
            .map(|all| all.len())
            .unwrap_or(0);

        if count > 0 {
            break;
        }
    }

    println!("the server holds {} episode(s) now", count);

    // The test puts the library back as it was, whatever the download did.
    let removed = reqwest::Client::new()
        .delete(format!("{}/api/items/{}?hard=1", SERVER, made.id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("the server must answer");

    assert!(count > 0, "the server must hold the episode");
    assert!(removed.status().is_success(), "the test must clean up");
}
