//! A media that came to its end leaves no row behind. See T-141.
//!
//! **The measurement of 2026-08-13.** A book of eight hours played to its end,
//! and the program stayed open. The row of `listening_session` then held
//! `t=28800` and `finished=1`, and the server held the same values: **the
//! position was safe, and the row stayed.** A different client wrote a new place
//! of that book, and the key `Q` of this program sent 28800 seconds and
//! "finished" again — **the new place of the user went away.**
//!
//! That is the fault that T-4 named for the start of the program, and the answer
//! is the same one: **the row goes away when the server holds the position.** A
//! server that did not accept it keeps the row, because the position then lives
//! in that row only.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::{get_listening_session, insert_listening_session};
use toutui::logic::playback::follow_playback;
use toutui::player::engine::{PlaybackStatus, PlayerHandle};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The account and the server of every row of this test.
const THE_ACCOUNT: &str = "the-account";
const THE_SERVER: &str = "the-server";

/// The identity of the playback of each test. The flag of the forced sync holds
/// one identity for the whole process, therefore each test needs its own.
const PLAYBACK_OF_THE_END: u64 = 21;
const PLAYBACK_OF_THE_SERVER_THAT_REFUSES: u64 = 22;

fn client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// The directory of configuration of this test binary. The two tests run at the
/// same time in one process, therefore they share one directory: the rule of
/// `tests/playback_ownership.rs`.
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

/// Writes the row of a playback, as `play_media` does.
fn the_row_of_a_playback(id_session: &str, id_item: &str) {
    insert_listening_session(
        id_session.to_string(),
        id_item.to_string(),
        100,
        "1000".to_string(),
        String::new(),
        0,
        "A Book".to_string(),
        "An Author".to_string(),
        true,
        String::new(),
        THE_ACCOUNT,
        THE_SERVER,
    )
    .expect("the row of the test");
}

/// Says if the table holds the row of that playback.
fn the_table_holds(id_session: &str) -> bool {
    let conn = toutui::db::migrate::open_conn().expect("the database of the test");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM listening_session WHERE id_session = ?1",
            [id_session],
            |row| row.get(0),
        )
        .expect("the count of the rows");

    count > 0
}

/// Waits while the loop of the playback runs.
async fn the_loop_stops(loop_of_the_media: tokio::task::JoinHandle<()>) {
    tokio::time::timeout(std::time::Duration::from_secs(20), loop_of_the_media)
        .await
        .expect("the loop of the playback must stop with the media")
        .expect("the loop of the playback");
}

/// **The fault of T-141.** The row of a media that came to its end stayed, and
/// the key `Q` then sent that end again over a newer place of the user.
#[tokio::test(flavor = "multi_thread")]
async fn a_media_that_came_to_its_end_leaves_no_row() {
    temporary_home();
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/session/the-session-of-the-end/close"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/the-book-of-the-end"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    the_row_of_a_playback("the-session-of-the-end", "the-book-of-the-end");

    let (player, _receiver) = PlayerHandle::without_engine();

    {
        let shared = player.shared_state();
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK_OF_THE_END;
        state.item_id = "the-book-of-the-end".to_string();
        state.position = 1000.0;
        state.duration = 1000.0;
        state.status = PlaybackStatus::Stopped;
        state.finished = true;
    }

    let api = client(&server.uri());

    the_loop_stops(tokio::spawn(async move {
        follow_playback(
            &api,
            &player,
            "the-session-of-the-end".to_string(),
            "the-book-of-the-end".to_string(),
            None,
            THE_ACCOUNT.to_string(),
            "1000".to_string(),
            PLAYBACK_OF_THE_END,
            1000.0,
        )
        .await;
    }))
    .await;

    assert!(
        !the_table_holds("the-session-of-the-end"),
        "the media came to its end and the server holds its position, therefore \
         the row must go away. A row that stays sends that end again at the next \
         key `Q`, and it destroys the place that a different client wrote. See \
         T-4."
    );

    assert!(
        get_listening_session(THE_ACCOUNT, THE_SERVER)
            .expect("the database")
            .is_none(),
        "no session waits after a media that came to its end"
    );
}

/// A server that does not accept the position keeps the row: the position of
/// the user then lives in that row only, and the next program sends it. See
/// T-25 and T-4.
#[tokio::test(flavor = "multi_thread")]
async fn a_server_that_refuses_the_position_keeps_the_row() {
    temporary_home();
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/session/the-session-that-waits/close"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/me/progress/the-book-that-waits"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    the_row_of_a_playback("the-session-that-waits", "the-book-that-waits");

    let (player, _receiver) = PlayerHandle::without_engine();

    {
        let shared = player.shared_state();
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK_OF_THE_SERVER_THAT_REFUSES;
        state.item_id = "the-book-that-waits".to_string();
        state.position = 640.0;
        state.duration = 1000.0;
        state.status = PlaybackStatus::Stopped;
        state.finished = false;
    }

    let api = client(&server.uri());

    the_loop_stops(tokio::spawn(async move {
        follow_playback(
            &api,
            &player,
            "the-session-that-waits".to_string(),
            "the-book-that-waits".to_string(),
            None,
            THE_ACCOUNT.to_string(),
            "1000".to_string(),
            PLAYBACK_OF_THE_SERVER_THAT_REFUSES,
            640.0,
        )
        .await;
    }))
    .await;

    assert!(
        the_table_holds("the-session-that-waits"),
        "the server did not take the position, therefore the row must stay: that \
         row is the only place of the user."
    );
}
