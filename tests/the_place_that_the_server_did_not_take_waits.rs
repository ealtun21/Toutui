//! A place that the server did not take stays on the disk. See T-189.
//!
//! **The row of a listening session goes away after the place of the user is
//! safe**: the server holds it, or the table `pending_progress` holds it. That
//! is the rule of T-145. The old code asked `is_offline` for that second road,
//! therefore a server that **answered** with a fault gave neither: the place
//! went away for ever, and the log said the words of a success.
//!
//! **The measurement of 2026-08-14**, with `docs/harness/one_path_fails.py` of
//! the path `/api/me/progress` and a row of a program that died at 1234 seconds
//! of `A Book Of Many Hours`:
//!
//! ```text
//! [sync_session_from_database] the server did not accept the position: The
//!     server reported a fault. Status 500.
//! [handle_key (Q)][book][Quit] Item 6ba57b9a-… closed at 1234s (not finished)
//! ```
//!
//! `listening_session` held no row, `pending_progress` held no row, and the
//! place of the server stayed **0**.
//!
//! **Two faults say that a place reaches a server never**: the status 404, of a
//! media that the server does not hold (T-187), and the status 400, of a request
//! that the server refused (T-87). Every other fault can pass: a status of 500
//! or more is the fault of one machine (T-128), a token that is not valid holds
//! until the user logs in again, and a permission of an account can come back
//! (T-136).
//!
//! **This test needs no sandbox.** A host of `wiremock` gives the fault to the
//! write of the place.

use rusqlite::params;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::error::ApiError;
use toutui::api::client::ApiClient;
use toutui::db::crud::{get_pending_progress, insert_pending_progress, PendingProgress};
use toutui::logic::offline::{flush_pending_progress, the_place_can_wait};
use toutui::logic::sync_session::sync_session_from_database::sync_session_from_database;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const THE_ACCOUNT: &str = "the-account-of-the-place-that-waits";
const THE_SERVER: &str = "the-server-of-the-place-that-waits";
const THE_BOOK: &str = "the-book-of-the-place-that-waits";

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

fn a_client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// The row of a program that died, with the place of the user in it.
fn the_row_of_a_program_that_died(at: u32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock of the machine")
        .as_secs();

    let conn = toutui::db::migrate::open_conn().expect("the database of the test");

    conn.execute("DELETE FROM listening_session", [])
        .expect("the table of the test");
    conn.execute("DELETE FROM pending_progress", [])
        .expect("the table of the test");

    conn.execute(
        "INSERT INTO listening_session (id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter, username, server, owner, heartbeat)
         VALUES ('the-session-that-died', ?1, ?2, '28800', 0, '', 0, 'A Book', 'An Author', 1, '', ?3, ?4, 'a-program-that-died', ?5)",
        params![
            THE_BOOK,
            at,
            THE_ACCOUNT,
            THE_SERVER,
            now.saturating_sub(3600) as i64
        ],
    )
    .expect("the row of the program that died");
}

fn the_rows_of_the_sessions() -> i64 {
    let conn = toutui::db::migrate::open_conn().expect("the database of the test");
    conn.query_row(
        "SELECT COUNT(*) FROM listening_session WHERE username = ?1 AND server = ?2",
        [THE_ACCOUNT, THE_SERVER],
        |row| row.get(0),
    )
    .expect("the count of the rows")
}

/// A host that closes a session and that fails the write of the place.
async fn a_host_that_fails_the_place(status: u16) -> MockServer {
    let host = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_regex(r"^/api/session/.+/close$"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&host)
        .await;

    Mock::given(method("PATCH"))
        .and(path_regex(r"^/api/me/progress/.+$"))
        .respond_with(ResponseTemplate::new(status))
        .mount(&host)
        .await;

    host
}

/// The place of a session that the server did not take waits on the disk.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn the_place_of_a_session_that_the_server_refused_stays_on_the_disk() {
    temporary_home();

    // ── The road of the fault: the server answers 500 to the write of the
    // place. The row of the session goes away, therefore the disk must hold that
    // place.
    let host = a_host_that_fails_the_place(500).await;
    the_row_of_a_program_that_died(1234);

    let api = a_client(&host.uri());
    sync_session_from_database(
        &api,
        THE_ACCOUNT.to_string(),
        THE_SERVER.to_string(),
        false,
        "Q",
    )
    .await;

    let waiting = get_pending_progress(THE_ACCOUNT, THE_SERVER);

    assert_eq!(
        waiting.len(),
        1,
        "the row of the session goes away, therefore a place that the server did \
         not take must wait on the disk: the place of the user is gone for ever \
         without it"
    );

    assert_eq!(
        waiting[0].current_time, 1234.0,
        "the place of the disk is the place of that session"
    );

    assert_eq!(
        the_rows_of_the_sessions(),
        0,
        "the place is safe on the disk, therefore the row of the session goes \
         away (T-145)"
    );

    // ── A media that the server does not hold: the status 404. Such a place
    // belongs to nothing (T-187), therefore no row waits.
    let host = a_host_that_fails_the_place(404).await;
    the_row_of_a_program_that_died(1234);

    let api = a_client(&host.uri());
    sync_session_from_database(
        &api,
        THE_ACCOUNT.to_string(),
        THE_SERVER.to_string(),
        false,
        "Q",
    )
    .await;

    assert!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER).is_empty(),
        "the server does not hold this media, therefore its place waits for \
         nothing"
    );

    // ── The same rule holds for the flush of the positions that wait. The write
    // of the flush answered 500, and the old code removed the row: the task of
    // every 30 seconds then had nothing to send.
    let host = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/api/me/progress/.+$"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&host)
        .await;

    Mock::given(method("PATCH"))
        .and(path_regex(r"^/api/me/progress/.+$"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&host)
        .await;

    insert_pending_progress(
        THE_ACCOUNT,
        THE_SERVER,
        &PendingProgress {
            id_item: THE_BOOK.to_string(),
            id_pod: String::new(),
            current_time: 1234.0,
            duration: 28800.0,
            is_finished: false,
            updated_at: toutui::logic::offline::now_ms(),
        },
    )
    .expect("the row of the test");

    let api = a_client(&host.uri());
    let sent = flush_pending_progress(&api, THE_ACCOUNT, THE_SERVER).await;

    assert_eq!(sent, 0, "the server did not take the position");

    assert_eq!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER).len(),
        1,
        "a write that came back with the status 500 keeps the row: the task of \
         the flush tries again every 30 seconds"
    );

    // ── The rule of the two faults, apart from the two roads above.
    for fault in [
        ApiError::Unreachable,
        ApiError::Timeout,
        ApiError::Server(500),
        ApiError::Server(502),
        ApiError::Unauthorized,
        ApiError::Forbidden,
        ApiError::Decode("no field".to_string()),
    ] {
        assert!(
            the_place_can_wait(&fault),
            "this fault can pass at a later attempt, therefore the place waits: {}",
            fault
        );
    }

    for fault in [ApiError::NotFound, ApiError::Server(400)] {
        assert!(
            !the_place_can_wait(&fault),
            "this fault gives the same answer at every attempt, therefore the \
             place waits for nothing: {}",
            fault
        );
    }
}
