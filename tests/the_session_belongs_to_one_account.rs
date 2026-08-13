//! The listening session of one account never goes to the server of another
//! account. See T-138 and T-139.
//!
//! **The sweep of two accounts of two servers while a media plays, 2026-08-13,
//! found two faults of one shape.** The account `toutuitest` of the server of
//! the port 13399 played a book at the minute 13, and the key `c` gave the start
//! to the account `secondtest` of the port 13400:
//!
//! 1. **The position of that playback never reached its server.** The key starts
//!    the program again with `exec`, and no line of the program sent the position
//!    first: the server held the minute 13:23 of the last sync, and the program
//!    held 13:31.
//! 2. **The next playback of the second account sent the position of the first
//!    account to the second server.** One row stood for the whole program, and
//!    that row held no account at all: the second server answered "The server
//!    does not have this item", and the close of that session then removed the
//!    row. **The place of the user went away**, and no line of the screen said
//!    it.
//!
//! The row holds the account and the server now, as the queue of version 7 of
//! the schema does. A test of the database and a test of the source of the loop
//! hold the two rules.

use rusqlite::Connection;
use toutui::db::crud::{
    delete_the_session_of_a_playback, get_listening_session, insert_listening_session,
};

/// The database of one test: a configuration directory of its own, and the turn
/// of that test.
///
/// **`XDG_CONFIG_HOME` belongs to the process and not to the test**, and three
/// tests of this binary write it. `cargo test` runs them in threads of one
/// process, therefore a test read the database of another one: the row of the
/// account "second" stood in the database while the first test held that account
/// to no row, and a row with no account at all belongs to **every** account
/// (T-138). nextest gives each test a process of its own, and it hides that
/// fault. **One test of this binary runs at one time.** See T-144.
struct TheDatabaseOfTheTest {
    _turn: std::sync::MutexGuard<'static, ()>,
    directory: tempfile::TempDir,
}

impl TheDatabaseOfTheTest {
    fn path(&self) -> &std::path::Path {
        self.directory.path()
    }
}

fn a_database_of_the_test() -> TheDatabaseOfTheTest {
    static THE_TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());

    // A test that stopped inside its turn must not stop every test after it.
    let turn = THE_TURN
        .lock()
        .unwrap_or_else(|of_a_test| of_a_test.into_inner());

    let directory = tempfile::tempdir().expect("a directory");
    std::env::set_var("XDG_CONFIG_HOME", directory.path());
    std::fs::create_dir_all(directory.path().join("toutui")).expect("the directory of the program");

    TheDatabaseOfTheTest {
        _turn: turn,
        directory,
    }
}

/// Writes one session of one account.
fn a_session_of(id_session: &str, id_item: &str, at: u32, username: &str, server: &str) {
    insert_listening_session(
        id_session.to_string(),
        id_item.to_string(),
        at,
        "1800".to_string(),
        String::new(),
        0,
        "A Book".to_string(),
        "An Author".to_string(),
        true,
        String::new(),
        username,
        server,
    )
    .expect("the session of the test");
}

/// **The fault of T-138.** The program of the second account read the row of the
/// first one, and it sent that position to its own server.
#[test]
fn the_session_of_one_account_does_not_go_to_another_account() {
    let _dir = a_database_of_the_test();

    a_session_of(
        "a-session",
        "a-book-of-the-first-server",
        810,
        "first",
        "one",
    );

    // The account of the row takes it.
    let of_the_first = get_listening_session("first", "one")
        .expect("the database")
        .expect("the account of the row must take it");
    assert_eq!(of_the_first.id_item, "a-book-of-the-first-server");
    assert_eq!(of_the_first.current_time, 810);

    // **Another account takes nothing at all.** The old program gave this row to
    // every account, and the position of 810 seconds then went to a server that
    // does not hold that item.
    assert!(
        get_listening_session("second", "two")
            .expect("the database")
            .is_none(),
        "the session of the account \"first\" went to the account \"second\""
    );

    // The same name on another server is another account.
    assert!(
        get_listening_session("first", "two")
            .expect("the database")
            .is_none(),
        "the session of one server went to another server"
    );
}

/// A playback of the second account must keep the session of the first one: that
/// position belongs to the first server, and the program sends it when that
/// account plays again.
#[test]
fn a_playback_of_another_account_keeps_the_session_that_waits() {
    let _dir = a_database_of_the_test();

    a_session_of(
        "a-session",
        "a-book-of-the-first-server",
        810,
        "first",
        "one",
    );
    a_session_of(
        "another",
        "a-book-of-the-second-server",
        42,
        "second",
        "two",
    );

    let of_the_first = get_listening_session("first", "one")
        .expect("the database")
        .expect("the session of the first account must stay");
    assert_eq!(of_the_first.current_time, 810);

    let of_the_second = get_listening_session("second", "two")
        .expect("the database")
        .expect("the session of the second account");
    assert_eq!(of_the_second.id_item, "a-book-of-the-second-server");

    // The close of one session leaves the other one. The close removes the row of
    // that session alone (T-145).
    delete_the_session_of_a_playback("another").expect("the database");

    assert!(
        get_listening_session("second", "two")
            .expect("the database")
            .is_none(),
        "the close of a session left its row"
    );
    assert_eq!(
        get_listening_session("first", "one")
            .expect("the database")
            .expect("the session of the first account must stay")
            .current_time,
        810,
        "the close of one session removed the session of another account"
    );
}

/// A row of a program of an older version holds no account, and the account that
/// asks takes it: such a database holds the row of the one account that program
/// had.
#[test]
fn a_row_of_an_older_program_belongs_to_the_account_that_asks() {
    let dir = a_database_of_the_test();

    // The two columns of T-138 come with the version 8 of the schema. A row of
    // an older program holds no value in them, therefore this test writes such a
    // row itself.
    a_session_of("a-session", "a-book", 300, "", "");

    let path = dir.path().join("toutui").join("db.sqlite3");
    let conn = Connection::open(&path).expect("the database of the test");
    let empty: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM listening_session WHERE username = '' AND server = ''",
            [],
            |row| row.get(0),
        )
        .expect("the count of the rows");
    assert_eq!(empty, 1, "the row of the test must hold no account");

    let session = get_listening_session("the-one-account", "a-server")
        .expect("the database")
        .expect("a row of an older program belongs to the account that asks");
    assert_eq!(session.current_time, 300);
}

/// **The loop of the program must send the position before it starts the program
/// again.** The rule lives in `src/main.rs`, and no unit test reaches that loop:
/// a test may read the source of the program, as the tests of T-131 and T-135
/// do. See T-139.
#[test]
fn the_loop_sends_the_position_before_the_program_starts_again() {
    let source = include_str!("../src/main.rs");

    assert!(
        source.contains("app.the_program_starts_again.take()"),
        "the loop of the program must take the request of a key that starts the \
         program again. A key handler cannot wait for the server, therefore the \
         handler writes the request and the loop does the work."
    );

    let of_the_accounts = source
        .split("app.the_program_starts_again.take()")
        .nth(1)
        .expect("the loop must hold that request");

    let the_sync = of_the_accounts
        .find("sync_session_from_database")
        .expect("the loop must close the session of the playback");
    let the_start_again = of_the_accounts
        .find("start_the_program_again_with")
        .expect("the loop must start the program again");

    assert!(
        the_sync < the_start_again,
        "the loop must send the position of the playback **before** it starts the \
         program again: `exec` takes every task of this process away, therefore a \
         position that waits for the server never arrives."
    );

    assert!(
        of_the_accounts[..the_sync].contains("PlayerCommand::Stop"),
        "the loop must stop the engine before it sends the position, so that no \
         later second of the playback stands in the database."
    );
}
