//! A playback that does not start must not move the position. See T-38.
//!
//! The user reported this on 2026-08-10: the screen said "buffer overrun", the
//! book started at the beginning, and the position went to 0 on the disk and
//! on the server.
//!
//! **The mechanism.** `rodio` gives the position inside the source, and
//! `get_pos` gives 0 until the seek finishes. A book of one file that starts
//! at 1227 seconds therefore reports 0 for a short time, and a playback that
//! never starts reports 0 for the whole wait. The loop of the playback wrote
//! that 0 in the database every second, and it gave that 0 to the server when
//! the session closed.
//!
//! This test gives the loop an engine that reports 0 and never moves. The row
//! of the download must keep the position of the start.
//!
//! The test needs no server: the address is a port that nothing listens on.
//!
//! # The clock of this test
//!
//! The loop reads the state one time each second, and this test must give the
//! fault the time to appear: a poll of "the position is the position of the
//! user" answers `true` before the loop ever ran, because the row holds that
//! value already. Such a poll is a false pass.
//!
//! Therefore the test holds a clock of its own (`start_paused = true`). Each
//! `sleep` below still gives the loop its steps — three steps for 3500
//! milliseconds — and it takes no real time. The test took 8.01 seconds with
//! the clock of the machine, and it takes about 0.02 seconds now.
//!
//! The test makes no request and it opens no socket. A test that waits for a
//! server must not take this clock: the clock would move to the timeout of the
//! request while the request is still on its way.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::{get_download, insert_download};
use toutui::logic::playback::follow_playback;
use toutui::player::engine::{PlaybackStatus, PlayerHandle};

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";
const USER: &str = "a-user";
const ITEM: &str = "an-item";

/// Where the book starts. The user listened to 1227 seconds before.
const START: f64 = 1227.0;

/// The identity of this playback.
const PLAYBACK: u64 = 21;

#[tokio::test(start_paused = true)]
async fn an_engine_that_stays_at_zero_does_not_move_the_position() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("XDG_DATA_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    // The disk holds the book, and the row holds the place of the user.
    insert_download(
        ITEM,
        USER,
        "A Book",
        "An Author",
        "/a/path.mp3",
        3600.0,
        ITEM,
        "a-server",
    )
    .unwrap();

    // The row starts with the place of the user.
    toutui::db::crud::update_download_current_time(ITEM, USER, START as u32).unwrap();

    let pool = EndpointPool::new(vec![Endpoint::new(NO_SERVER, 0)]);
    let api = ApiClient::new(Arc::new(pool), "token".to_string()).unwrap();

    let (player, _receiver) = PlayerHandle::without_engine();
    let shared = player.shared_state();

    // The engine holds this playback, and it says 0: the seek did not finish,
    // or the playback never started.
    {
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK;
        state.item_id = ITEM.to_string();
        state.position = 0.0;
        state.duration = 3600.0;
        state.status = PlaybackStatus::Playing;
    }

    let loop_of_the_book = tokio::spawn(async move {
        follow_playback(
            &api,
            &player,
            "a-session".to_string(),
            ITEM.to_string(),
            None,
            USER.to_string(),
            "3600".to_string(),
            PLAYBACK,
            START,
        )
        .await;
    });

    // The loop reads the state one time each second. This time gives it three
    // steps, and each of them can write the wrong position.
    tokio::time::sleep(tokio::time::Duration::from_millis(3500)).await;

    let position = get_download(ITEM, USER)
        .expect("the program reads its database")
        .expect("the row of the download must stay")
        .1 as f64;

    assert!(
        (position - START).abs() < 1.0,
        "the position went to {} seconds. The engine said 0 while it did not \
         start, and the loop wrote that 0. The user then loses their place. \
         See T-38.",
        position
    );

    // ---- The second part: the engine reaches the place of the seek. ----
    //
    // The two parts stand in one test, because `XDG_CONFIG_HOME` is one value
    // for the whole process. Two tests would then share one database.

    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    // The seek finished. The engine plays at the place of the user now.
    {
        let mut state = shared.write().unwrap();
        state.position = START + 10.0;
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    let position = get_download(ITEM, USER)
        .expect("the program reads its database")
        .expect("the row of the download must stay")
        .1 as f64;
    assert!(
        (position - (START + 10.0)).abs() < 1.5,
        "the loop must follow the engine after the seek. It holds {} seconds.",
        position
    );

    // The user goes back by 300 seconds. The loop follows that too.
    {
        let mut state = shared.write().unwrap();
        state.position = START - 300.0;
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    let position = get_download(ITEM, USER)
        .expect("the program reads its database")
        .expect("the row of the download must stay")
        .1 as f64;
    assert!(
        (position - (START - 300.0)).abs() < 1.5,
        "the user went back, and the loop must follow. It holds {} seconds.",
        position
    );

    loop_of_the_book.abort();
}
