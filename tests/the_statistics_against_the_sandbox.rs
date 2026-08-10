//! The statistics of the user come from the server. See T-24.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_statistics_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The test reads only. It changes nothing on the server.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::me::listening_stats::{get_listening_stats, human_time, top_items, week};
use toutui::logic::stats::State;
use toutui::ui::stats_tui::lines;

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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_server_gives_the_statistics_and_the_screen_reads_them() {
    let token = token().await;
    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), token).unwrap());

    let stats = get_listening_stats(&api)
        .await
        .expect("the server must give the statistics");

    println!(
        "in total {}, today {}, {} days, {} media, {} sessions",
        human_time(stats.total_time),
        human_time(stats.today),
        stats.days.len(),
        stats.items.len(),
        stats.recent_sessions.len()
    );

    // The sandbox played media. Therefore the answer holds a time, and it
    // holds a media.
    assert!(
        stats.total_time > 0.0,
        "the sandbox must hold a time. Play a media of docs/TEST-SERVER.md first."
    );
    assert!(!stats.items.is_empty(), "the answer must name a media");
    assert_eq!(week(&stats).len(), 7);

    let top = top_items(&stats, 5);
    assert!(!top.is_empty(), "the list of the media must hold a line");
    println!("the media of the largest time: {}", top[0].title);

    // The screen reads the answer of a real server.
    let text = lines(&State::Ready(Box::new(stats)), 80)
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("\n");

    println!("{}", text);

    assert!(text.contains("In total:"));
    assert!(text.contains("The days of the week"));
    assert!(text.contains(&top[0].title));
}
