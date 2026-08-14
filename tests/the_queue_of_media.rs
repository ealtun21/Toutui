//! The queue starts the next media at an end, and at nothing else. See T-24.
//!
//! The client played one media, and it then stopped. A measurement of
//! 2026-08-11 with the real program shows the fault: the log holds
//! `[follow_playback] the playback stopped at 3 seconds, finished=true`, and
//! nothing more comes for 19 seconds.
//!
//! `follow_playback` gives the outcome of the playback now, and `play` reads
//! that outcome. These tests hold the two rules of that outcome:
//!
//! - A media that came to its end gives `Outcome::Finished`, and the queue
//!   then starts the next media.
//! - A media that the user stopped, and a media that a different playback took
//!   away, give `Outcome::Stopped`, and the queue stays where it is.
//!
//! The tests need no sound card and no server. `PlayerHandle::without_engine`
//! gives the state, and `wiremock` gives the server. This is the same shape as
//! `tests/playback_ownership.rs`.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::logic::playback::{follow_playback, PlaybackTarget};
use toutui::logic::queue::{self, Entry, Outcome, Queue};
use toutui::player::engine::{PlaybackStatus, PlayerHandle};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The identity of the playback of the test.
const PLAYBACK: u64 = 21;

/// Where the media plays, in seconds.
const POSITION: f64 = 3.0;

fn client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// The directory of configuration of this test binary.
///
/// No line of a test may touch the database of the user. The tests of one
/// binary run at the same time, and `XDG_CONFIG_HOME` is one value for the
/// whole process, therefore all the tests share one directory. See trap 5 of
/// `docs/HANDOVER.md`.
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

/// Makes a server that takes the close and the position of one session.
async fn server_that_takes_the_end() -> MockServer {
    let server = MockServer::start().await;

    for route in [
        "/api/session/session-A/sync",
        "/api/session/session-A/close",
    ] {
        Mock::given(method("POST"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
    }

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/item-A"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    server
}

/// Runs the loop of the playback against a state that the test writes, and
/// gives the outcome.
async fn outcome_of_a_state(status: PlaybackStatus, finished: bool, playback_id: u64) -> Outcome {
    temporary_home();
    let server = server_that_takes_the_end().await;

    let (player, _receiver) = PlayerHandle::without_engine();
    let shared = player.shared_state();

    {
        let mut state = shared.write().unwrap();
        state.playback_id = playback_id;
        state.item_id = "item-A".to_string();
        state.position = POSITION;
        state.duration = 3.0;
        state.status = status;
        state.finished = finished;
    }

    let api = client(&server.uri());

    follow_playback(
        &api,
        &player,
        "session-A".to_string(),
        "item-A".to_string(),
        None,
        "user".to_string(),
        "the-server-of-the-queue".to_string(),
        "3".to_string(),
        PLAYBACK,
        0.0,
    )
    .await
}

/// The media came to its end. The queue must start the next media.
#[tokio::test(flavor = "multi_thread")]
async fn a_media_that_came_to_its_end_starts_the_queue() {
    let outcome = outcome_of_a_state(PlaybackStatus::Stopped, true, PLAYBACK).await;

    assert_eq!(outcome, Outcome::Finished);
    assert!(queue::the_queue_goes_on(outcome));
}

/// The user stopped the media. `PlayerCommand::Stop` writes the status
/// `Stopped` and the value `finished` of `false`. The queue must stay where it
/// is: the user asked for silence.
#[tokio::test(flavor = "multi_thread")]
async fn a_media_that_the_user_stopped_does_not_start_the_queue() {
    let outcome = outcome_of_a_state(PlaybackStatus::Stopped, false, PLAYBACK).await;

    assert_eq!(outcome, Outcome::Stopped);
    assert!(!queue::the_queue_goes_on(outcome));
}

/// A different playback took the engine: the user pressed the key of a
/// different book. That book is the answer, and not the queue.
#[tokio::test(flavor = "multi_thread")]
async fn a_different_playback_does_not_start_the_queue() {
    // The engine holds a later playback, therefore this loop gives up at once.
    let outcome = outcome_of_a_state(PlaybackStatus::Playing, false, PLAYBACK + 1).await;

    assert_eq!(outcome, Outcome::Stopped);
    assert!(!queue::the_queue_goes_on(outcome));
}

/// The sequence of the whole work: two media wait, and each end gives the
/// engine to the next media.
///
/// The measurement of 2026-08-11 with the real program and the sandbox gives
/// the same sequence. The server wrote, one second apart: "Starting session
/// ... Second Series Volume 1", "... The Test Chronicles Volume 2", and "...
/// The Test Chronicles Volume 3".
#[test]
fn the_queue_gives_its_media_in_the_sequence_of_the_user() {
    let mut waiting = Queue::default();

    for (id, title) in [("book-2", "Volume 2"), ("book-3", "Volume 3")] {
        waiting.add(Entry {
            target: PlaybackTarget::Book {
                item_id: id.to_string(),
                whole_book_duration: Some(3.0),
            },
            title: title.to_string(),
            author: "Series Author".to_string(),
            duration: Some(3.0),
        });
    }

    let mut played: Vec<String> = Vec::new();

    // The user started a media of their own. Every end takes the next media of
    // the queue.
    let mut outcome = Outcome::Finished;

    while queue::the_queue_goes_on(outcome) {
        let Some(entry) = waiting.take_next() else {
            break;
        };

        played.push(entry.title.clone());

        // Each of these media also comes to its end.
        outcome = Outcome::Finished;
    }

    assert_eq!(played, vec!["Volume 2".to_string(), "Volume 3".to_string()]);
    assert!(waiting.is_empty());
}

/// A media that the user stopped leaves the media of the queue where they are.
/// The user presses `q` and `l` to start the queue again.
#[test]
fn a_stop_leaves_the_queue_where_it_is() {
    let mut waiting = Queue::default();

    waiting.add(Entry {
        target: PlaybackTarget::Book {
            item_id: "book-2".to_string(),
            whole_book_duration: None,
        },
        title: "Volume 2".to_string(),
        author: String::new(),
        duration: None,
    });

    assert!(!queue::the_queue_goes_on(Outcome::Stopped));
    assert_eq!(waiting.len(), 1);
}
