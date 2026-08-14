//! A place that no machine holds keeps the row of its session. See T-212.
//!
//! **The row of a listening session goes away after the place of the user stands
//! somewhere else**: the server holds it, or the table `pending_progress` holds
//! it (T-145 and T-189). `close_one_session` wrote that second row with
//! `remember_progress`, and it read **no answer** of that write: a disk that took
//! no row therefore left the place of the user on no machine, and the removal
//! under the same block took the last copy of it.
//!
//! **The measurement of 2026-08-14** of the real program of the sandbox, with
//! `docs/harness/one_path_fails.py` of the two paths `/api/me/progress` and
//! `/api/session`, and one command for a table of the disk that takes no write:
//!
//! ```bash
//! sqlite3 "$DB" "ALTER TABLE pending_progress
//!     RENAME COLUMN position_s TO position_s_of_an_old_version;"
//! ```
//!
//! The user heard 757 seconds of `A Book Of Many Hours`, and the key `Q` said:
//!
//! ```text
//! [WARN] [sync_session_from_database] the server did not accept the position:
//!     The server reported a fault. Status 500.
//! [WARN] [offline] the application did not keep the position 757s of 6ba57b9a-…:
//!     table pending_progress has no column named position_s. The place of that
//!     playback goes away.
//! [INFO] [handle_key (Q)][book][Quit] Item 6ba57b9a-… closed at 757s (not finished)
//! ```
//!
//! `listening_session` held **0** rows, `pending_progress` held **0** rows, and
//! `GET /api/me/progress` of the sandbox gave `currentTime 0`: the place of the
//! user stood on no machine at all.
//!
//! **The second road of the same fault is the box of T-207.** A `chmod 444` of
//! the file of the database (T-206) makes the removal fail, and the box then held
//! the identity of that session with the words "The server holds the place of
//! that media already" — one millisecond after the status 500 of the write of
//! that place. The next key `l` of the program read that box, and it removed the
//! row with **no request at all**:
//!
//! ```text
//! [handle_key] the server holds the place of the session fae44ca0-… already. The
//!     disk kept its row, and this program sends it no second time.
//! ```
//!
//! **This test needs no sandbox.** A host of `wiremock` gives the fault of the
//! server, one `ALTER TABLE` gives a table of the disk that takes no write, and
//! `std::fs::set_permissions` of `0o444` gives the disk of T-206.

use rusqlite::params;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::get_pending_progress;
use toutui::logic::sync_session::sync_session_from_database::sync_session_from_database;
use toutui::logic::sync_session::the_rows_that_the_disk_kept::{
    the_box_of_the_sessions_goes_empty, the_place_of_this_session_stands_somewhere,
};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const THE_ACCOUNT: &str = "the-account-of-a-place-that-no-machine-holds";
const THE_SERVER: &str = "the-server-of-a-place-that-no-machine-holds";
const THE_BOOK: &str = "the-book-of-a-place-that-no-machine-holds";
const THE_SESSION: &str = "the-session-of-a-place-that-no-machine-holds";

static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();

fn temporary_home() -> std::path::PathBuf {
    let dir = HOME.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();
        drop(conn);

        dir
    });

    dir.path().join("toutui").join("db.sqlite3")
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

    conn.execute(
        "INSERT INTO listening_session (id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter, username, server, owner, heartbeat)
         VALUES (?1, ?2, ?3, '28800', 0, '', 0, 'A Book', 'An Author', 1, '', ?4, ?5, 'a-program-that-died', ?6)",
        params![
            THE_SESSION,
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

/// The table of the places that wait takes no write, and every other table of
/// the disk answers. This is the harness of T-203 for one table.
fn the_places_that_wait_take_no_write(broken: bool) {
    let conn = toutui::db::migrate::open_conn().expect("the database of the test");

    let statement = if broken {
        "ALTER TABLE pending_progress RENAME COLUMN position_s TO position_s_of_an_old_version"
    } else {
        "ALTER TABLE pending_progress RENAME COLUMN position_s_of_an_old_version TO position_s"
    };

    conn.execute(statement, []).expect("the table of the test");
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

async fn the_program_closes_the_sessions(url: &str) {
    let api = a_client(url);

    sync_session_from_database(
        &api,
        THE_ACCOUNT.to_string(),
        THE_SERVER.to_string(),
        false,
        "Q",
    )
    .await;
}

/// A place that reached no machine keeps the row of its session.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[tokio::test(flavor = "multi_thread")]
async fn a_place_that_reached_no_machine_keeps_the_row_of_its_session() {
    let the_file_of_the_database = temporary_home();

    // ── The road of the fault. The server answers 500 to the write of the place,
    // and the table of the places that wait takes no row: the place of the user
    // stands on no machine, therefore the row of the session must stay.
    the_box_of_the_sessions_goes_empty();
    let host = a_host_that_fails_the_place(500).await;
    the_row_of_a_program_that_died(757);
    the_places_that_wait_take_no_write(true);

    the_program_closes_the_sessions(&host.uri()).await;

    the_places_that_wait_take_no_write(false);

    assert_eq!(
        the_rows_of_the_sessions(),
        1,
        "the server did not take the place of the user and the disk did not keep \
         it, therefore the row of that session is the last copy of that place: a \
         removal of it takes the place of the user away for ever"
    );

    assert!(
        get_pending_progress(THE_ACCOUNT, THE_SERVER)
            .unwrap()
            .is_empty(),
        "the table of the places that wait took no row"
    );

    assert!(
        the_place_of_this_session_stands_somewhere(THE_SESSION).is_none(),
        "no machine holds that place, therefore the box of the sessions holds \
         nothing: a program that says that the place of a session is safe sends \
         that place no second time (T-207)"
    );

    // ── The second road of the same fault: the disk that takes no write at all
    // (T-206). The removal fails too, therefore the row stays; and the box must
    // not say that a machine holds a place that the server refused.
    the_box_of_the_sessions_goes_empty();
    let host = a_host_that_fails_the_place(500).await;
    the_row_of_a_program_that_died(599);

    let of_the_disk = std::fs::metadata(&the_file_of_the_database)
        .expect("the file of the database")
        .permissions();

    let mut readonly = of_the_disk.clone();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        readonly.set_mode(0o444);
    }
    readonly.set_readonly(true);
    std::fs::set_permissions(&the_file_of_the_database, readonly).expect("the disk of the test");

    the_program_closes_the_sessions(&host.uri()).await;

    std::fs::set_permissions(&the_file_of_the_database, of_the_disk).expect("the disk of the test");

    assert_eq!(
        the_rows_of_the_sessions(),
        1,
        "the disk took no write at all, therefore the row of that session stays"
    );

    assert!(
        the_place_of_this_session_stands_somewhere(THE_SESSION).is_none(),
        "the server refused that place with the status 500, therefore no machine \
         holds it: the box of T-207 said \"the server holds the place of that \
         media already\" for such a row, and the next key of the user then \
         removed it with no request at all"
    );

    // ── The place that the server takes. The row goes away, and the box holds
    // nothing.
    the_box_of_the_sessions_goes_empty();
    let host = a_host_that_fails_the_place(200).await;
    the_row_of_a_program_that_died(1234);

    the_program_closes_the_sessions(&host.uri()).await;

    assert_eq!(
        the_rows_of_the_sessions(),
        0,
        "the server holds that place, therefore the row goes away (T-145)"
    );

    // ── The place that the disk keeps. The row goes away, because the task of
    // the flush sends that row to the server again (T-189).
    the_box_of_the_sessions_goes_empty();
    let host = a_host_that_fails_the_place(500).await;
    the_row_of_a_program_that_died(2345);

    the_program_closes_the_sessions(&host.uri()).await;

    assert_eq!(
        the_rows_of_the_sessions(),
        0,
        "the table of the places that wait holds that place, therefore the row of \
         the session goes away"
    );

    let waiting = get_pending_progress(THE_ACCOUNT, THE_SERVER).unwrap();
    assert_eq!(waiting.len(), 1, "the disk holds that place");
    assert_eq!(waiting[0].current_time, 2345.0, "and it is that place");

    // ── A place that this server takes never: the status 404 of a media that the
    // server does not hold, and the status 400 of a request that it refused
    // (T-189). No machine keeps such a place, therefore the row must not stay for
    // ever.
    for status in [404, 400] {
        the_box_of_the_sessions_goes_empty();
        let host = a_host_that_fails_the_place(status).await;
        the_row_of_a_program_that_died(3456);

        the_program_closes_the_sessions(&host.uri()).await;

        assert_eq!(
            the_rows_of_the_sessions(),
            0,
            "the status {} gives the same answer at every attempt, therefore the \
             row of that session waits for nothing",
            status
        );
    }
}
