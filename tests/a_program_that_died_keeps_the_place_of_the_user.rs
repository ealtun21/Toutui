//! A row that the program does not close must not go away. See T-145.
//!
//! **The measurement of 2026-08-13.** A book of eight hours played, and the
//! terminal of the user went away (`SIGHUP`): the program died with 1024 seconds
//! of that book on the disk, and the server held 866. The user started the
//! program again at once and played a second book, and the key `Q` of that
//! program then left **no row at all**: the row of the book of the dead program
//! went away with no request, and the place of the user stayed at 866 seconds on
//! the server for ever.
//!
//! The two rules of `sync_session_from_database` did not agree:
//!
//! - it closes **one** session (`get_listening_session` gives one row), and
//! - it removes **every** row that this program may take.
//!
//! A user who starts the program again inside `THE_LIMIT_OF_THE_HEARTBEAT`
//! seconds meets both: the row of the program that died is too young for the
//! rule of T-140 at the moment of the play, and it is old enough for the removal
//! at the moment of the key `Q`.
//!
//! **The program closes every row that it may take now, and it removes a row
//! after that row reached the server.** The rows of a program that died go
//! first, and the row of this program goes last: two rows of one media then
//! leave the newest position on the server.

use rusqlite::params;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::db::crud::{insert_listening_session, THE_LIMIT_OF_THE_HEARTBEAT};
use toutui::logic::sync_session::sync_session_from_database::sync_session_from_database;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// **The two tests of this binary share one database and one
/// `XDG_CONFIG_HOME`**, therefore each test holds an account of its own. See
/// T-144.
const THE_ACCOUNT_OF_THE_DEATH: &str = "the-account-of-the-death";
const THE_SERVER_OF_THE_DEATH: &str = "the-server-of-the-death";
const THE_ACCOUNT_OF_THE_ORDER: &str = "the-account-of-the-order";
const THE_SERVER_OF_THE_ORDER: &str = "the-server-of-the-order";

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

fn client(url: &str) -> ApiClient {
    let pool = EndpointPool::new(vec![Endpoint::new(url, 0)]);
    ApiClient::new(Arc::new(pool), "test-token".to_string()).unwrap()
}

/// The row of the playback of **this** program, as `play_media` writes it.
fn the_row_of_this_program(id_session: &str, id_item: &str, at: u32, username: &str, server: &str) {
    insert_listening_session(
        id_session.to_string(),
        id_item.to_string(),
        at,
        "28800".to_string(),
        String::new(),
        0,
        "A Book".to_string(),
        "An Author".to_string(),
        true,
        String::new(),
        username,
        server,
    )
    .expect("the row of the test");
}

/// The row of a program that **died**: the terminal went away, and the last
/// second of that playback stands in the row.
fn the_row_of_a_program_that_died(
    id_session: &str,
    id_item: &str,
    at: u32,
    moments_ago: u64,
    username: &str,
    server: &str,
) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock of the machine")
        .as_secs();

    let conn = toutui::db::migrate::open_conn().expect("the database of the test");

    conn.execute(
        "INSERT INTO listening_session (id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter, username, server, owner, heartbeat)
         VALUES (?1, ?2, ?3, '28800', 0, '', 0, 'A Book', 'An Author', 1, '', ?4, ?5, 'a-program-that-died', ?6)",
        params![
            id_session,
            id_item,
            at,
            username,
            server,
            now.saturating_sub(moments_ago) as i64
        ],
    )
    .expect("the row of the program that died");
}

fn the_rows_of(username: &str, server: &str) -> i64 {
    let conn = toutui::db::migrate::open_conn().expect("the database of the test");
    conn.query_row(
        "SELECT COUNT(*) FROM listening_session WHERE username = ?1 AND server = ?2",
        [username, server],
        |row| row.get(0),
    )
    .expect("the count of the rows")
}

/// The media of each `PATCH /api/me/progress/:id` of the server, in the
/// sequence of the requests.
async fn the_media_that_reached_the_server(server: &MockServer) -> Vec<(String, u64)> {
    server
        .received_requests()
        .await
        .expect("the requests of the server")
        .iter()
        .filter(|request| request.url.path().starts_with("/api/me/progress/"))
        .map(|request| {
            let item = request
                .url
                .path()
                .trim_start_matches("/api/me/progress/")
                .to_string();

            let body: serde_json::Value =
                serde_json::from_slice(&request.body).expect("the body of the request");

            let at = body
                .get("currentTime")
                .and_then(|value| value.as_f64())
                .expect("the position of the request") as u64;

            (item, at)
        })
        .collect()
}

async fn a_server_that_takes_every_request() -> MockServer {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_regex(r"^/api/session/.+/close$"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("PATCH"))
        .and(path_regex(r"^/api/me/progress/.+$"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    server
}

/// **The fault of T-145.** The program closed one row and it removed every row,
/// therefore the place of the user of a program that died went away with no
/// request.
#[tokio::test(flavor = "multi_thread")]
async fn the_place_of_a_program_that_died_reaches_the_server() {
    temporary_home();
    let server = a_server_that_takes_every_request().await;

    // **The sequence of the two rows is the condition** (the trap 94 of the
    // handover). The user starts the program again at once, therefore the row of
    // the program that died is younger than `THE_LIMIT_OF_THE_HEARTBEAT` at the
    // moment of the play: `insert_listening_session` leaves it, and the play
    // never closes it. The row grows old while the second book plays.
    the_row_of_this_program(
        "the-session-of-now",
        "the-book-of-now",
        1842,
        THE_ACCOUNT_OF_THE_DEATH,
        THE_SERVER_OF_THE_DEATH,
    );

    // The program of the terminal that went away: 1024 seconds of its book stand
    // on the disk, and no request carried them.
    the_row_of_a_program_that_died(
        "the-session-that-died",
        "the-book-that-died",
        1024,
        THE_LIMIT_OF_THE_HEARTBEAT + 5,
        THE_ACCOUNT_OF_THE_DEATH,
        THE_SERVER_OF_THE_DEATH,
    );

    let api = client(&server.uri());

    sync_session_from_database(
        &api,
        THE_ACCOUNT_OF_THE_DEATH.to_string(),
        THE_SERVER_OF_THE_DEATH.to_string(),
        false,
        "Q",
    )
    .await;

    let media = the_media_that_reached_the_server(&server).await;

    assert!(
        media
            .iter()
            .any(|(item, at)| item == "the-book-that-died" && *at == 1024),
        "the row of the program that died holds the place of the user, and the \
         program removed that row with no request: the place is gone for ever. \
         The requests of the server: {:?}",
        media
    );

    assert!(
        media
            .iter()
            .any(|(item, at)| item == "the-book-of-now" && *at == 1842),
        "the row of this program must reach the server too. The requests of the \
         server: {:?}",
        media
    );

    assert_eq!(
        the_rows_of(THE_ACCOUNT_OF_THE_DEATH, THE_SERVER_OF_THE_DEATH),
        0,
        "every row reached the server, therefore no row stays: a row that stays \
         sends its position again over the place of a different client (T-4)."
    );
}

/// Two rows of **one media**: the row of the program that died goes first, and
/// the row of this program goes last. The newest position is then the last write
/// of the server.
#[tokio::test(flavor = "multi_thread")]
async fn the_row_of_this_program_reaches_the_server_last() {
    temporary_home();
    let server = a_server_that_takes_every_request().await;

    // The user plays the same book again, and this program stands further in it.
    the_row_of_this_program(
        "the-session-of-now-of-the-order",
        "one-book",
        3000,
        THE_ACCOUNT_OF_THE_ORDER,
        THE_SERVER_OF_THE_ORDER,
    );

    the_row_of_a_program_that_died(
        "the-session-of-the-terminal",
        "one-book",
        1024,
        THE_LIMIT_OF_THE_HEARTBEAT + 5,
        THE_ACCOUNT_OF_THE_ORDER,
        THE_SERVER_OF_THE_ORDER,
    );

    let api = client(&server.uri());

    sync_session_from_database(
        &api,
        THE_ACCOUNT_OF_THE_ORDER.to_string(),
        THE_SERVER_OF_THE_ORDER.to_string(),
        false,
        "Q",
    )
    .await;

    let media = the_media_that_reached_the_server(&server).await;
    let places: Vec<u64> = media
        .iter()
        .filter(|(item, _)| item == "one-book")
        .map(|(_, at)| *at)
        .collect();

    assert_eq!(
        places,
        vec![1024, 3000],
        "the row of a program that died holds an older place of the same media, \
         therefore it must reach the server **before** the row of this program: \
         the newest place is then the last write of the server."
    );
}
