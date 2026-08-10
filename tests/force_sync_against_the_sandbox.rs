//! The command that forces the sync, against a real server. See T-32 and
//! upstream issue #37.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test force_sync_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The application sends the position every ten seconds during a playback. The
//! key `F` asks for the sync now. The test must show two things: the server
//! holds the new position at once, and the session stays open.
//!
//! `docs/TEST-SERVER.md` section 8 says that the ALSA device `null` plays a
//! book of 60 seconds in a few milliseconds. Therefore this test writes the
//! state of the engine itself, as `sync_against_the_sandbox.rs` does.
//!
//! The test changes the progress of one book on the sandbox server. It changes
//! nothing on a server of a user, and it changes nothing in the configuration
//! of the user.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::library_items::play_lib_item_or_pod::post_start_playback_session_book;
use toutui::logic::playback::follow_playback;
use toutui::logic::sync_session::force_sync;
use toutui::player::engine::{PlaybackStatus, PlayerHandle};

/// The sandbox server of `docs/TEST-SERVER.md`.
const SERVER: &str = "http://127.0.0.1:13399";
const USER: &str = "toutuitest";
const PASSWORD: &str = "toutuitest";

/// The book of the test. It has three audio files and 60 seconds.
const TITLE: &str = "Multi File Test Book";

/// The identity of the playback.
const PLAYBACK: u64 = 11;

/// Where the book plays, in seconds. The value is not a round number, thus no
/// other test gives it by chance.
const POSITION: f64 = 37.0;

/// The application sends the position every ten seconds. The test asks for the
/// sync long before that.
const BEFORE_THE_PERIOD: u64 = 2200;

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

    panic!(
        "the sandbox server must hold the book \"{}\". See docs/TEST-SERVER.md.",
        title
    );
}

/// Reads a number that the server gives as a number or as a text.
///
/// Audiobookshelf gives `currentTime` as a text in `/api/me/progress/:id`. A
/// test that reads a number only finds nothing there.
fn number(value: &serde_json::Value) -> f64 {
    match value {
        serde_json::Value::Number(number) => number.as_f64().unwrap_or(-1.0),
        serde_json::Value::String(text) => text.parse::<f64>().unwrap_or(-1.0),
        _ => -1.0,
    }
}

async fn session_is_open(token: &str, session_id: &str) -> bool {
    let open = get(token, "/api/sessions/open").await;

    open["sessions"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|session| session["id"].as_str() == Some(session_id))
}

/// The key `F` gives the position to the server at once, and the session
/// stays open.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_forced_sync_sends_the_position_and_keeps_the_session() {
    // No line of this test may touch the database of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    let token = token().await;
    let item = item_of(&token, TITLE).await;

    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = ApiClient::new(Arc::new(pool), token.clone()).unwrap();

    let session = post_start_playback_session_book(&api, &item)
        .await
        .expect("the server must open a session");
    let session_id = session[3].clone();

    let (player, _receiver) = PlayerHandle::without_engine();
    let shared = player.shared_state();

    {
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK;
        state.item_id = item.clone();
        state.position = POSITION;
        state.duration = 60.0;
        state.status = PlaybackStatus::Playing;
    }

    let session_of_the_loop = session_id.clone();
    let item_of_the_loop = item.clone();

    let playback_loop = tokio::spawn(async move {
        follow_playback(
            &api,
            &player,
            session_of_the_loop,
            item_of_the_loop,
            None,
            USER.to_string(),
            "60".to_string(),
            PLAYBACK,
            POSITION,
        )
        .await;
    });

    // The loop reads the state one time each second.
    tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;

    // The user presses the key `F`.
    assert!(force_sync::ask(PLAYBACK), "the request must go to the loop");

    tokio::time::sleep(tokio::time::Duration::from_millis(BEFORE_THE_PERIOD)).await;

    let report = force_sync::take_report();
    assert!(
        report.is_some(),
        "the loop must tell the user what the server did"
    );
    let report = report.unwrap_or_default();
    assert!(
        report.contains("the server has the position"),
        "the server did not take the position: {}",
        report
    );

    let progress = get(&token, &format!("/api/me/progress/{}", item)).await;

    // Audiobookshelf gives `currentTime` as a text, and not as a number.
    let current = number(&progress["currentTime"]);
    assert!(
        (current - POSITION).abs() < 1.5,
        "the server must hold {} seconds, and it holds {}",
        POSITION,
        current
    );

    assert!(
        session_is_open(&token, &session_id).await,
        "the forced sync must not close the session"
    );

    // Stop the loop, so that the session does not stay open on the server.
    {
        let mut state = shared.write().unwrap();
        state.status = PlaybackStatus::Stopped;
    }

    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(10), playback_loop).await;
}
