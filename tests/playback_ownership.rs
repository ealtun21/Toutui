//! The loop that follows a playback must report its own playback only.
//!
//! The state of the engine is one value for the whole application. The key
//! that starts a media gives its work to a new task, therefore two playbacks
//! can run at the same time.
//!
//! The old code read that state always. The loop of the book X then read the
//! position of the book Y, and it reported that position for X. The loop also
//! never saw the status `Stopped`, thus the session of X stayed open. These
//! tests hold the rule that closes the reports `9bacac`, `86384e`, and
//! `dd9a649`.
//!
//! The tests need no sound card and no server. `PlayerHandle::without_engine`
//! gives the state, and `wiremock` gives the server.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::logic::playback::follow_playback;
use toutui::player::engine::{PlaybackStatus, PlayerHandle};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The identity of the playback of the book X.
const PLAYBACK_X: u64 = 7;

/// The identity of the playback of the book Y. A later playback has a larger
/// identity.
const PLAYBACK_Y: u64 = 8;

/// The identity of the playback of the second test.
///
/// The flag of the forced sync is one value for the whole process, and it
/// carries the identity of a playback. The two tests run at the same time,
/// therefore each of them needs an identity of its own. See `force_sync`.
const PLAYBACK_Z: u64 = 9;

/// Where the book X plays, in seconds.
const POSITION_X: f64 = 100.0;

/// Where the book Y plays, in seconds. The book Y starts, therefore its
/// position is small.
const POSITION_Y: f64 = 4.0;

fn client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// The directory of configuration of this test binary.
///
/// No line of a test may touch the database of the user. Therefore the tests
/// give `XDG_CONFIG_HOME` a temporary directory.
///
/// The two tests run at the same time in one process, and the variable is one
/// value for the whole process. Therefore all the tests share one directory.
/// A directory for each test gave the other test a directory that was already
/// removed, and the functions of the database then wrote a message on the
/// screen.
static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();

fn temporary_home() {
    HOME.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();
        drop(conn);

        dir
    });
}

/// Gives the requests that the server received, as a list of the method, the
/// path, and the body.
async fn requests(server: &MockServer) -> Vec<(String, String, serde_json::Value)> {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .map(|request| {
            (
                request.method.to_string(),
                request.url.path().to_string(),
                serde_json::from_slice(&request.body).unwrap_or(serde_json::Value::Null),
            )
        })
        .collect()
}

/// How long a poll waits before it gives up.
const LIMIT: std::time::Duration = std::time::Duration::from_secs(20);

/// How long a poll waits between two examinations.
const STEP: std::time::Duration = std::time::Duration::from_millis(20);

/// Waits until the loop made one step with the state of its own playback.
///
/// **Why a forced sync, and not a sleep.** The loop reads the state one time
/// each second, and it writes nothing while it agrees with the state. A sleep
/// of 2200 milliseconds gave it two steps, and it cost 2.2 seconds of each
/// run. The key `F` asks the loop to send its position at its next step
/// (`force_sync`), therefore the request `POST /api/session/:id/sync` is the
/// evidence that the loop read the state of this playback and took its
/// position. The evidence comes after about one second.
///
/// The function gives the position that the loop sent to the server.
async fn the_loop_made_a_step(server: &MockServer, playback_id: u64) -> serde_json::Value {
    let start = std::time::Instant::now();

    loop {
        // The flag holds one identity for the whole process, and the two tests
        // run at the same time: the request of the other test can take the
        // place of this one. Therefore the poll asks again at each step.
        assert!(
            toutui::logic::sync_session::force_sync::ask(playback_id),
            "the flag of the forced sync takes no identity 0"
        );

        let sync = requests(server)
            .await
            .into_iter()
            .find(|(method, path, _)| method == "POST" && path.ends_with("/sync"));

        if let Some((_, _, body)) = sync {
            return body["currentTime"].clone();
        }

        assert!(
            start.elapsed() < LIMIT,
            "the loop of the playback {} made no step in {:?}. It sent {:?}",
            playback_id,
            LIMIT,
            requests(server).await
        );

        tokio::time::sleep(STEP).await;
    }
}

/// Waits until the loop stops.
///
/// A sleep of 2500 milliseconds held this wait before. The loop stops at its
/// next step, and it then closes its session: the poll gives the same
/// measurement after about one second.
async fn the_loop_stops(loop_of_x: &tokio::task::JoinHandle<()>, why: &str) {
    let start = std::time::Instant::now();

    while !loop_of_x.is_finished() {
        assert!(start.elapsed() < LIMIT, "{}", why);
        tokio::time::sleep(STEP).await;
    }
}

/// A different playback takes the engine. The loop of the book X must stop, it
/// must close its own session, and it must report its own position.
///
/// A measurement with the old code sent
/// `{"currentTime":"4","timeListened":"0"}` to the session of X, and the loop
/// did not stop.
#[tokio::test(flavor = "multi_thread")]
async fn the_loop_stops_when_a_different_playback_takes_the_engine() {
    temporary_home();
    let server = MockServer::start().await;

    for route in [
        "/api/session/session-X/sync",
        "/api/session/session-X/close",
    ] {
        Mock::given(method("POST"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
    }

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/item-X"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let (player, _receiver) = PlayerHandle::without_engine();
    let shared = player.shared_state();

    // The engine plays the book X at 100 seconds.
    {
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK_X;
        state.item_id = "item-X".to_string();
        state.position = POSITION_X;
        state.duration = 1000.0;
        state.status = PlaybackStatus::Playing;
    }

    let api = client(&server.uri());

    let loop_of_x = tokio::spawn(async move {
        follow_playback(
            &api,
            &player,
            "session-X".to_string(),
            "item-X".to_string(),
            None,
            "user".to_string(),
            "1000".to_string(),
            PLAYBACK_X,
            POSITION_X,
        )
        .await;
    });

    // The loop must read the state of the book X before the book Y takes the
    // engine. The forced sync gives the evidence of that step.
    let position = the_loop_made_a_step(&server, PLAYBACK_X).await;
    assert_eq!(
        position, "100",
        "the loop must hold the position of the book X after its first step"
    );

    // The user starts the book Y. The engine plays Y now.
    {
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK_Y;
        state.item_id = "item-Y".to_string();
        state.position = POSITION_Y;
        state.duration = 2000.0;
        state.status = PlaybackStatus::Playing;
    }

    the_loop_stops(
        &loop_of_x,
        "the loop of the book X must stop when the engine plays the book Y",
    )
    .await;

    let requests = requests(&server).await;

    assert!(
        requests
            .iter()
            .any(|(method, path, _)| method == "POST" && path == "/api/session/session-X/close"),
        "the loop must close the session of the book X, but it sent {:?}",
        requests
    );

    let progress: Vec<&serde_json::Value> = requests
        .iter()
        .filter(|(method, path, _)| method == "PATCH" && path == "/api/me/progress/item-X")
        .map(|(_, _, body)| body)
        .collect();

    assert_eq!(
        progress.len(),
        1,
        "the loop must report the position of the book X one time, but it sent {:?}",
        requests
    );

    assert_eq!(
        progress[0]["currentTime"], 100,
        "the loop must report the position of the book X, and not the position \
         of the book Y"
    );

    // The media did not come to its end. Therefore the request holds no mark.
    assert!(progress[0].get("isFinished").is_none());

    // No request may hold the position of the book Y.
    for (method, path, body) in &requests {
        assert_ne!(
            body["currentTime"], "4",
            "the request {} {} holds the position of the book Y",
            method, path
        );
        assert_ne!(
            body["currentTime"], 4,
            "the request {} {} holds the position of the book Y",
            method, path
        );
    }
}

/// The engine plays the book X. The loop must report the position of the book
/// X, and the identity in the state must not stop it.
#[tokio::test(flavor = "multi_thread")]
async fn the_loop_reports_its_own_playback() {
    temporary_home();
    let server = MockServer::start().await;

    for route in [
        "/api/session/session-X/sync",
        "/api/session/session-X/close",
    ] {
        Mock::given(method("POST"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
    }

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/item-X"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let (player, _receiver) = PlayerHandle::without_engine();
    let shared = player.shared_state();

    {
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK_Z;
        state.item_id = "item-X".to_string();
        state.position = POSITION_X;
        state.duration = 1000.0;
        state.status = PlaybackStatus::Playing;
    }

    let api = client(&server.uri());

    let loop_of_x = tokio::spawn(async move {
        follow_playback(
            &api,
            &player,
            "session-X".to_string(),
            "item-X".to_string(),
            None,
            "user".to_string(),
            "1000".to_string(),
            PLAYBACK_Z,
            POSITION_X,
        )
        .await;
    });

    // The loop must read the state one time before the playback stops.
    let position = the_loop_made_a_step(&server, PLAYBACK_Z).await;
    assert_eq!(position, "100", "the loop must follow the book X");

    // The user stops the playback. The engine keeps the identity and the
    // position of the book X.
    {
        let mut state = shared.write().unwrap();
        state.position = 250.0;
        state.status = PlaybackStatus::Stopped;
    }

    the_loop_stops(&loop_of_x, "the loop must stop with the media").await;

    let requests = requests(&server).await;

    let progress: Vec<&serde_json::Value> = requests
        .iter()
        .filter(|(method, path, _)| method == "PATCH" && path == "/api/me/progress/item-X")
        .map(|(_, _, body)| body)
        .collect();

    assert_eq!(progress.len(), 1, "the requests are {:?}", requests);
    assert_eq!(progress[0]["currentTime"], 250);
}
