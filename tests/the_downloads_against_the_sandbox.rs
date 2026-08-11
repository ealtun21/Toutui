//! The queue of the episodes that the server downloads. See T-81.
//!
//! Continuous integration does not run this test, because it needs a server and
//! the network of that server. Start the sandbox of `docs/TEST-SERVER.md`, and
//! then run:
//!
//! ```text
//! cargo test --test the_downloads_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! **The test gives the server work of the network**: it puts three episodes of
//! the feed of the sandbox in the queue, and it empties that queue at once. The
//! server can hold one of them before the clear comes, and that file stays in
//! the library of the sandbox.
//!
//! A sandbox with no podcast gives a line of text, and the test does not fail.

use std::sync::Arc;
use std::time::{Duration, Instant};
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::podcasts::the_downloads::{empty_the_queue, the_downloads_of_the_library};

const SERVER: &str = "http://127.0.0.1:13399";
const USER: &str = "toutuitest";
const PASSWORD: &str = "toutuitest";

/// The feed of the sandbox. `docs/TEST-SERVER.md` names it.
const FEED: &str = "https://librivox.org/rss/52";

/// How long a poll of the queue waits.
///
/// **The queue of the server does not fill at once.** A measurement of
/// 2026-08-11 read an empty queue two seconds after the request, and the server
/// held nine episodes three seconds later. See T-81.
const LIMIT: Duration = Duration::from_secs(30);

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

/// Gives the library of the podcasts and its first podcast.
async fn the_podcast(api: &Arc<ApiClient>) -> Option<(String, String)> {
    let libraries: serde_json::Value = api
        .get_json("/api/libraries")
        .await
        .expect("the server must give the libraries");

    for library in libraries["libraries"].as_array()? {
        if library["mediaType"].as_str() != Some("podcast") {
            continue;
        }

        let library_id = library["id"].as_str()?.to_string();

        let items: serde_json::Value = api
            .get_json(&format!("/api/libraries/{}/items?limit=10", library_id))
            .await
            .ok()?;

        if let Some(item) = items["results"].as_array()?.first() {
            return Some((library_id, item["id"].as_str()?.to_string()));
        }
    }

    None
}

/// Gives the episodes of the feed that the server does not hold.
async fn the_episodes_that_are_missing(
    api: &Arc<ApiClient>,
    item_id: &str,
    how_many: usize,
) -> Vec<serde_json::Value> {
    let item: serde_json::Value = api
        .get_json(&format!("/api/items/{}", item_id))
        .await
        .expect("the server must give the podcast");

    let of_the_server: Vec<String> = item["media"]["episodes"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|episode| episode["title"].as_str().map(str::to_string))
        .collect();

    let feed: serde_json::Value = api
        .post_json(
            "/api/podcasts/feed",
            &serde_json::json!({ "rssFeed": FEED }),
        )
        .await
        .expect("the server must read the feed");

    feed["podcast"]["episodes"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter(|episode| {
            let title = episode["title"].as_str().unwrap_or_default().to_string();
            !of_the_server.contains(&title)
        })
        .take(how_many)
        .cloned()
        .collect()
}

/// Waits until the queue of the library holds a line, and it gives the lines.
async fn wait_for_the_queue(
    api: &Arc<ApiClient>,
    library_id: &str,
) -> Vec<toutui::api::podcasts::the_downloads::OneDownload> {
    let start = Instant::now();

    loop {
        let all = the_downloads_of_the_library(api, library_id)
            .await
            .expect("the server must give the queue");

        if !all.is_empty() {
            return all;
        }

        assert!(
            start.elapsed() < LIMIT,
            "the server put no episode in the queue in {:?}",
            LIMIT
        );

        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox of docs/TEST-SERVER.md on :13399, and its network"]
async fn the_program_reads_the_queue_of_the_downloads_and_it_empties_it() {
    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token().await).unwrap());

    let Some((library_id, item_id)) = the_podcast(&api).await else {
        println!("this sandbox holds no podcast. See docs/TEST-SERVER.md, section 5.");
        return;
    };

    let episodes = the_episodes_that_are_missing(&api, &item_id, 3).await;

    if episodes.is_empty() {
        println!("the server holds every episode of the feed. The test needs one that is missing.");
        return;
    }

    println!("the test gives {} episodes to the server", episodes.len());

    api.post_no_content(
        &format!("/api/podcasts/{}/download-episodes", item_id),
        &serde_json::Value::Array(episodes),
    )
    .await
    .expect("the server must take the episodes");

    let all = wait_for_the_queue(&api, &library_id).await;

    println!("the queue holds {} line(s):", all.len());

    for one in &all {
        println!("  {}", one.line());
    }

    // The episode that downloads now stands first, and it carries its mark.
    assert!(
        all.iter().filter(|one| one.now).count() <= 1,
        "the server downloads one episode at a time"
    );

    if let Some(first) = all.first() {
        assert!(!first.item_id.is_empty(), "a line names its podcast");
        assert!(!first.title.is_empty(), "a line names its episode");
    }

    // The key `X` of the view empties the queue of one podcast.
    empty_the_queue(&api, &item_id)
        .await
        .expect("the server must empty the queue");

    // **The clear does not stop the episode that downloads now.** Therefore the
    // test waits for a queue that holds that episode only, or nothing.
    let start = Instant::now();

    loop {
        let all = the_downloads_of_the_library(&api, &library_id)
            .await
            .expect("the server must give the queue");

        let waiting = all.iter().filter(|one| !one.now).count();

        if waiting == 0 {
            println!(
                "the queue is empty, and {} episode(s) of now stay",
                all.len()
            );
            return;
        }

        assert!(
            start.elapsed() < LIMIT,
            "the queue still holds {} episode(s) that wait, {:?} after the clear",
            waiting,
            LIMIT
        );

        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}
