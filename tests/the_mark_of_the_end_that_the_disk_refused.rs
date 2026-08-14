//! The mark of the end of a media that the disk refused. See T-213.
//!
//! **The mark of the end of a playback stands in the row of its session**, and
//! that row is the one copy of it for a program that dies (T-140 and T-145). The
//! loop of the playback wrote it with `let _ = update_is_finished(...)`: a disk
//! that took no write of that one column therefore gave the next program of the
//! account a media that the user finished and that the row of it says is **not**
//! finished, and no line of the log and no word of the screen named it.
//!
//! **The measurement of 2026-08-14** of the real program of the sandbox. The
//! condition holds two halves, and each of them takes one half away:
//!
//! ```bash
//! python3 docs/harness/one_method_fails.py 13500 13399 requests.log \
//!     PATCH:/api/me/progress
//! sqlite3 "$DB" "CREATE TRIGGER the_disk_takes_no_mark \
//!     BEFORE UPDATE OF is_finished ON listening_session \
//!     BEGIN SELECT RAISE(ABORT, 'the disk takes no mark of the end'); END;"
//! ```
//!
//! The book of 30 minutes came to its end on the null device in 25 seconds, and
//! the log of the program held two lines and no word of the mark:
//!
//! ```text
//! [INFO] [follow_playback] the playback stopped at 1800 seconds, finished=true
//! [WARN] [follow_playback] the server did not accept the position:
//!     The server reported a fault. Status 500.
//! ```
//!
//! The row of `listening_session` then held `current_time_playback=1800` and
//! `is_finished=0`. The next program of the account read that row, and the key
//! `Q` of it said:
//!
//! ```text
//! [INFO] [handle_key (Q)][book][Quit] Item 70a3cade-… closed at 1800s (not finished)
//! ```
//!
//! `grep -c "PATCH /api/me/progress"` of the log of the proxy said **1**: the
//! second request of `update_media_progress2_book`, which holds the body
//! `{"isFinished": true}`, never left the program. The sandbox then held
//! `currentTime 1800` and `isFinished false` of a book that the user finished,
//! and the same request by hand gave `isFinished true` with a `finishedAt`.
//!
//! **The server forgives the last ten seconds of a media, and no more.** Its own
//! log says the rule — `Marking media progress as finished because time remaining
//! (5) is less than 10 seconds` — therefore the arithmetic of the server covers a
//! media that ends at its length, and the mark of this program is the one carrier
//! for every media that the engine finishes earlier than that: a book whose
//! progress record holds the duration 0 (the shape of T-180) lost the mark at
//! 1800 seconds of 1800.
//!
//! **This test needs no sandbox.** A host of `wiremock` refuses the place of the
//! user, a trigger of SQLite refuses the write of one column, and
//! `std::fs::set_permissions` of `0o444` gives the disk of T-206.
//!
//! **The parts of this test stay in one function.** Two test functions of one
//! module fight for the row of the account and for the file of the database, and
//! `cargo test` gives them one process (T-144 and T-157).

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::{get_pending_progress, insert_listening_session};
use toutui::logic::playback::follow_playback;
use toutui::player::engine::{PlaybackStatus, PlayerHandle};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const THE_ACCOUNT: &str = "the-account-of-the-mark-of-the-end";
const THE_SERVER: &str = "the-server-of-the-mark-of-the-end";
const THE_BOOK: &str = "the-book-of-the-mark-of-the-end";

/// The length of the media, in seconds. The engine stops at that place.
const THE_LENGTH: f64 = 1800.0;

fn the_database() -> std::path::PathBuf {
    let home = std::env::var("XDG_CONFIG_HOME").expect("the home of this test");

    std::path::Path::new(&home)
        .join("toutui")
        .join("db.sqlite3")
}

fn the_permission_of_the_database(mode: u32) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(the_database(), std::fs::Permissions::from_mode(mode))
        .expect("the permission of the database of this test");
}

fn a_statement(sql: &str) {
    let conn = toutui::db::migrate::open_conn().expect("the database of this test");
    conn.execute_batch(sql).expect("the statement of this test");
}

/// Says how many rows of that playback the table of the sessions holds.
fn the_rows_of_the_session(id_session: &str) -> i64 {
    let conn = toutui::db::migrate::open_conn().expect("the database of this test");

    conn.query_row(
        "SELECT COUNT(*) FROM listening_session WHERE id_session = ?1",
        [id_session],
        |row| row.get(0),
    )
    .expect("the count of the rows")
}

/// Writes the row of a playback, as `play_media` does.
fn the_row_of_a_playback(id_session: &str) {
    insert_listening_session(
        id_session.to_string(),
        THE_BOOK.to_string(),
        THE_LENGTH as u32,
        THE_LENGTH.to_string(),
        String::new(),
        0,
        "A Book That Came To Its End".to_string(),
        "An Author".to_string(),
        true,
        String::new(),
        THE_ACCOUNT,
        THE_SERVER,
    )
    .expect("the row of the playback of this test");
}

/// The engine that came to the end of the media, with no audio at all.
#[allow(clippy::type_complexity)]
fn the_engine_at_the_end(
    id_playback: u64,
) -> (
    PlayerHandle,
    std::sync::mpsc::Receiver<toutui::player::engine::PlayerCommand>,
) {
    let (player, receiver) = PlayerHandle::without_engine();

    {
        let shared = player.shared_state();
        let mut state = shared.write().unwrap();
        state.playback_id = id_playback;
        state.item_id = THE_BOOK.to_string();
        state.position = THE_LENGTH;
        state.duration = THE_LENGTH;
        state.status = PlaybackStatus::Stopped;
        state.finished = true;
    }

    (player, receiver)
}

/// Runs the loop of the playback to its end, and it gives no more than 20
/// seconds to it: a test must not call a function that may never come back.
async fn the_loop_of_the_playback(api: ApiClient, player: PlayerHandle, id_session: &str, id: u64) {
    let of_the_session = id_session.to_string();

    let loop_of_the_media = tokio::spawn(async move {
        follow_playback(
            &api,
            &player,
            of_the_session,
            THE_BOOK.to_string(),
            None,
            THE_ACCOUNT.to_string(),
            THE_SERVER.to_string(),
            THE_LENGTH.to_string(),
            id,
            THE_LENGTH,
        )
        .await;
    });

    tokio::time::timeout(std::time::Duration::from_secs(20), loop_of_the_media)
        .await
        .expect("the loop of the playback must stop with the media")
        .expect("the loop of the playback");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_mark_of_the_end_reaches_a_machine_or_the_program_says_so() {
    let home = tempfile::tempdir().expect("the directory of this test");
    std::env::set_var("XDG_CONFIG_HOME", home.path());
    std::fs::create_dir_all(home.path().join("toutui")).expect("the directory of the program");

    {
        let conn = toutui::db::migrate::open_conn().expect("the database of this test");
        toutui::db::migrate::run_migrations(&conn).expect("the migration of this test");
    }

    // The server refuses the place of the user, therefore the row of the session
    // stays and the next program of the account reads the mark of that row.
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/session/the-session-of-the-mark/close"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/session/the-session-of-no-machine/close"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("PATCH"))
        .and(path(format!("/api/me/progress/{}", THE_BOOK)))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let a_client = || {
        let pool = EndpointPool::new(vec![Endpoint::new(&server.uri(), 0)]);

        ApiClient::new(Arc::new(pool), "the-token-of-this-test".to_string())
            .expect("the client of this test")
    };

    // ## The first road: the disk refuses the mark, and the table of the places
    // that wait takes it.
    //
    // **A table of the disk that takes no write is the statement of T-203 for
    // one column** (the trap 180). A trigger `BEFORE UPDATE OF` fails the write
    // of `is_finished` alone: every other read and every other write of the
    // program answers, therefore the row of the session, the place of each
    // second, and the row of the place that waits all stand.
    the_row_of_a_playback("the-session-of-the-mark");
    a_statement(
        "CREATE TRIGGER the_disk_takes_no_mark \
         BEFORE UPDATE OF is_finished ON listening_session \
         BEGIN SELECT RAISE(ABORT, 'the disk takes no mark of the end'); END;",
    );

    let (player, _receiver) = the_engine_at_the_end(213);
    the_loop_of_the_playback(a_client(), player, "the-session-of-the-mark", 213).await;

    let waiting = get_pending_progress(THE_ACCOUNT, THE_SERVER).expect("the table of the places");

    let of_the_book = waiting
        .iter()
        .find(|row| row.id_item == THE_BOOK)
        .unwrap_or_else(|| {
            panic!(
                "the disk took no mark of the end and the server took no place, therefore \
                 the place of the user and the mark of the end belong to the table of the \
                 places that wait. The old code read no answer of the write of the mark: \
                 the row of the session then said \"not finished\" of a media that came to \
                 its end, and the next program of the account sent that place with no mark \
                 at all. See T-213. The table holds: {:?}",
                waiting.len()
            )
        });

    assert!(
        of_the_book.is_finished,
        "the media came to its end, therefore the row that waits holds the mark of that \
         end: the flush of the positions then sends `{{\"isFinished\": true}}` of it. A row \
         with no mark gives the server a book of the shelf `Continue Listening` at the last \
         second of it. See T-213."
    );

    assert_eq!(
        of_the_book.current_time.round(),
        THE_LENGTH,
        "the row that waits holds the place of the user too"
    );

    assert_eq!(
        the_rows_of_the_session("the-session-of-the-mark"),
        0,
        "the table of the places that wait holds the place and the mark, therefore the row \
         of the session goes away: a row that stays sends that place a second time over a \
         newer place of a different client (T-4, T-141, and T-212)."
    );

    // ## The second road: no machine takes the mark, and the row of the session
    // stays with the place of the user.
    //
    // **A disk that answers every read and that takes no write** is the
    // condition of T-206, and one command gives it.
    a_statement("DROP TRIGGER the_disk_takes_no_mark;");
    a_statement("DELETE FROM pending_progress;");
    the_row_of_a_playback("the-session-of-no-machine");
    the_permission_of_the_database(0o444);

    let (player, _receiver) = the_engine_at_the_end(214);
    the_loop_of_the_playback(a_client(), player, "the-session-of-no-machine", 214).await;

    the_permission_of_the_database(0o644);

    assert!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER)
            .expect("the table of the places")
            .is_empty(),
        "a disk that takes no write keeps that table empty"
    );

    assert_eq!(
        the_rows_of_the_session("the-session-of-no-machine"),
        1,
        "no machine holds the mark of the end, therefore the row of the session stays: it \
         holds the place of the user, and that place is worth more than the mark that went \
         away with it (T-201). See T-213."
    );
}
