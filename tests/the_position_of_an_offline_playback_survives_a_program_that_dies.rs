//! An offline playback that a program does not end must keep its position for
//! the server. See T-152.
//!
//! **The sharp form of T-145.** The item T-145 measured a program that dies
//! while the server answers: the row of `listening_session` then holds the
//! position, and the next program of the account sends it. An offline playback
//! has no such row at all — `play_offline` opens no session on the server —
//! and no request of that playback ever reached the server. **The row of the
//! disk is therefore the one copy of the whole playback.**
//!
//! The loop wrote the position of each second to the row of the download, and
//! it kept that position for the server at its **end** only. A program that
//! dies reaches no end: the table `pending_progress` then held no row, and the
//! server kept the position of the day before for ever. The playback of the
//! user afterward took the position of the server, and it wrote that position
//! over the row of the download: **the one copy went away too.**
//!
//! A measurement of 2026-08-14, with the server away and a book of eight
//! hours: the disk held 1731 seconds, `pending_progress` held no row, and the
//! server held 100 seconds after the program started again.
//!
//! This test gives the loop an engine that plays, it takes the loop away in the
//! middle of the playback (`abort` is the death of the program), and it asks
//! for the position that waits.
//!
//! # The clock of this test
//!
//! The loop reads the state one time each second. The test holds a clock of its
//! own (`start_paused = true`), in the same way as
//! `the_position_survives_a_playback_that_does_not_start`: each `sleep` gives
//! the loop its steps and it takes no real time. The test makes no request and
//! it opens no socket.

use toutui::db::crud::{get_pending_progress, insert_download};
use toutui::logic::playback::follow_playback_offline;
use toutui::player::engine::{PlaybackStatus, PlayerHandle};

const USER: &str = "a-user";
const SERVER: &str = "a-server";
const ITEM: &str = "an-item";

/// Where the book stands when the offline playback starts.
const START: f64 = 100.0;

/// Where the engine plays while the program dies.
const REACHED: f64 = 300.0;

/// The identity of this playback.
const PLAYBACK: u64 = 34;

#[tokio::test(start_paused = true)]
async fn a_program_that_dies_offline_keeps_the_position_for_the_server() {
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
        28800.0,
        ITEM,
        SERVER,
    )
    .unwrap();
    toutui::db::crud::update_download_current_time(ITEM, USER, START as u32).unwrap();

    // No position waits before the playback.
    assert!(
        get_pending_progress(USER, SERVER).is_empty(),
        "the test must start with no position that waits"
    );

    let (player, _receiver) = PlayerHandle::without_engine();
    let shared = player.shared_state();

    // The engine plays this playback, and it stands after the place of the
    // start: the seek finished.
    {
        let mut state = shared.write().unwrap();
        state.playback_id = PLAYBACK;
        state.item_id = ITEM.to_string();
        state.position = REACHED;
        state.duration = 28800.0;
        state.status = PlaybackStatus::Playing;
    }

    let loop_of_the_book = tokio::spawn(async move {
        follow_playback_offline(
            &player,
            ITEM.to_string(),
            ITEM.to_string(),
            None,
            USER.to_string(),
            SERVER.to_string(),
            28800.0,
            PLAYBACK,
            START,
        )
        .await;
    });

    // Three steps of the loop. Each of them reads the engine and writes the
    // place of the user.
    tokio::time::sleep(tokio::time::Duration::from_millis(3500)).await;

    // **The program dies here.** The loop reaches no end, therefore it writes
    // nothing more: this is the terminal that goes away, and it is the kill of
    // the machine of the user.
    loop_of_the_book.abort();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let waiting = get_pending_progress(USER, SERVER);

    assert_eq!(
        waiting.len(),
        1,
        "the position of an offline playback must wait for the server. The \
         loop kept it at its end only, and a program that dies reaches no end: \
         the whole playback then went away, because no request of an offline \
         playback ever reaches the server. See T-152."
    );

    let position = waiting[0].current_time;

    assert!(
        (position - REACHED).abs() < 1.5,
        "the position that waits is {} seconds, and the engine played at {}. \
         The loop must keep the place of the user at each second. See T-152.",
        position,
        REACHED
    );
}
