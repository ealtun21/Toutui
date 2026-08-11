//! The reports `9bacac`, `86384e`, and `dd9a649` against a real server.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test sync_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The three reports describe one condition: the user plays the book X, the
//! user plays the book Y quickly, and then the progress of X is wrong or the
//! session of X stays open.
//!
//! One mechanism explains all three reports. The state of the engine is one
//! value for the whole application, and the loop that follows a playback read
//! that state always. The loop of X then reported the position of Y, and it
//! never saw that its own playback stopped.
//!
//! This test gives the loop of X a state that belongs to Y, and it then asks
//! the real server what happened. `docs/TEST-SERVER.md` section 8 says that the
//! ALSA device `null` plays a book of 60 seconds in a few milliseconds.
//! Therefore a test of this condition cannot use a real playback and a clock.
//! The test writes the state of the engine itself.
//!
//! The test changes the progress of one book on the sandbox server. It changes
//! nothing on a server of a user, and it changes nothing in the configuration
//! of the user.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::library_items::play_lib_item_or_pod::post_start_playback_session_book;
use toutui::logic::playback::follow_playback;
use toutui::player::engine::{PlaybackStatus, PlayerHandle};

/// The sandbox server of `docs/TEST-SERVER.md`.
const SERVER: &str = "http://127.0.0.1:13399";

/// The title of the book X. The book has three audio files and 60 seconds.
const TITLE_X: &str = "Multi File Test Book";

/// The title of the book Y.
const TITLE_Y: &str = "The Test Chronicles Volume 1";

/// The identity of the playback of the book X.
const PLAYBACK_X: u64 = 7;

/// The identity of the playback of the book Y. A later playback has a larger
/// identity.
const PLAYBACK_Y: u64 = 8;

/// Where the book X plays, in seconds.
const POSITION_X: f64 = 30.0;

/// Where the book Y plays, in seconds. The book Y starts, therefore its
/// position is small. This is the value that the old code reported for X.
const POSITION_Y: f64 = 2.0;

/// Gives a token of the sandbox server.
mod common;
use common::{token, USER};

/// Reads a path of the server with a token.
async fn get(token: &str, path: &str) -> serde_json::Value {
    reqwest::Client::new()
        .get(format!("{}{}", SERVER, path))
        .bearer_auth(token)
        .send()
        .await
        .expect("the sandbox server must answer")
        .json()
        .await
        .expect("the answer must hold JSON")
}

/// Gives the identity of the book that has a title.
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

/// Tells if the server holds an open session that has an identity.
async fn session_is_open(token: &str, session_id: &str) -> bool {
    let open = get(token, "/api/sessions/open").await;

    open["sessions"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|session| session["id"].as_str() == Some(session_id))
}

/// A different playback takes the engine. The real server must then hold the
/// position of the book X, and it must hold no open session of X.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_session_of_x_closes_and_keeps_the_position_of_x() {
    // No line of this test may touch the database of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    let token = token().await;
    let item_x = item_of(&token, TITLE_X).await;
    let item_y = item_of(&token, TITLE_Y).await;

    let pool = EndpointPool::new(vec![Endpoint::new(SERVER, 0)]);
    let api = ApiClient::new(Arc::new(pool), token.clone()).unwrap();

    // Open a real session for the book X.
    let session = post_start_playback_session_book(&api, &item_x)
        .await
        .expect("the server must open a session");
    let session_id = session[3].clone();

    assert!(
        session_is_open(&token, &session_id).await,
        "the server must hold the new session as an open session"
    );

    let (player, _receiver) = PlayerHandle::without_engine();
    let shared = player.shared_state();

    {
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK_X;
        state.item_id = item_x.clone();
        state.position = POSITION_X;
        state.duration = 60.0;
        state.status = PlaybackStatus::Playing;
    }

    let session_of_the_loop = session_id.clone();
    let item_of_the_loop = item_x.clone();

    let loop_of_x = tokio::spawn(async move {
        follow_playback(
            &api,
            &player,
            session_of_the_loop,
            item_of_the_loop,
            None,
            USER.to_string(),
            "60".to_string(),
            PLAYBACK_X,
            POSITION_X,
        )
        .await;
    });

    // The loop reads the state one time each second.
    tokio::time::sleep(tokio::time::Duration::from_millis(2200)).await;

    // The user plays the book Y. The engine plays Y now.
    {
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK_Y;
        state.item_id = item_y.clone();
        state.position = POSITION_Y;
        state.duration = 3.0;
        state.status = PlaybackStatus::Playing;
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(2500)).await;

    assert!(
        loop_of_x.is_finished(),
        "the loop of the book X must stop when the engine plays the book Y"
    );

    // The report `dd9a649`: the session of X must not stay open.
    assert!(
        !session_is_open(&token, &session_id).await,
        "the session {} of the book X must not stay open",
        session_id
    );

    // The reports `9bacac` and `86384e`: the progress of X must hold the
    // position of X, and not the position of Y.
    let progress = get(&token, &format!("/api/me/progress/{}", item_x)).await;
    let current_time = progress["currentTime"].as_f64().unwrap_or(-1.0);

    assert!(
        (current_time - POSITION_X).abs() < 1.0,
        "the server must hold the position of the book X ({} seconds), but it \
         holds {} seconds",
        POSITION_X,
        current_time
    );

    // The media did not come to its end.
    assert_eq!(progress["isFinished"], false);

    // The progress of the book Y must not change at all, because no loop of Y
    // ran in this test.
    let progress_y = get(&token, &format!("/api/me/progress/{}", item_y)).await;

    assert_ne!(
        progress_y["currentTime"].as_f64().unwrap_or(0.0),
        POSITION_X,
        "the loop of the book X must not write the progress of the book Y"
    );
}
