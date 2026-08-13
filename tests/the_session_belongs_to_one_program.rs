//! A listening session belongs to **one program**, and not to every program of
//! one account. See T-140.
//!
//! **The sweep of two programs of one account on one database, 2026-08-13, found
//! one fault of three effects.** The user started the program in two terminals
//! with the same account, and each of them played a book of its own:
//!
//! 1. **The playback of the second program closed the session of the first
//!    one.** `play_media` closes the session that the database holds before it
//!    opens its own (the rule of T-4), and the row held the live session of the
//!    other program: the server closed that session, and every sync of the first
//!    program then said "The server does not have this item".
//! 2. **The key `Q` of the first program sent the position of the book of the
//!    second one**, and it then removed that row: one row stands for one account,
//!    and the two programs of that account share it.
//! 3. **The key `Q` of the second program found no session at all**, therefore
//!    the place of that user never reached the server. The program said 2:00 of
//!    the book, and the server held 0:00.
//!
//! The row holds the program that owns it now (the version 9 of the schema), and
//! the second of a playback writes the moment of that program. A program takes a
//! row of its own, and a row that no program touched for
//! `THE_LIMIT_OF_THE_HEARTBEAT` seconds: **that is the row of a program that
//! stopped without a correct exit**, and the rule of T-4 keeps it.

use rusqlite::params;
use toutui::db::crud::{
    delete_listening_session, get_listening_session, insert_listening_session, update_current_time,
    THE_LIMIT_OF_THE_HEARTBEAT,
};

/// The database of one test: a configuration directory of its own, and the turn
/// of that test.
///
/// **`XDG_CONFIG_HOME` belongs to the process and not to the test**, and every
/// test of this binary writes it. `cargo test` runs them in threads of one
/// process: the tests then share one database, and a test that counts every row
/// of `listening_session` counts the rows of another test. nextest gives each
/// test a process of its own, and it hides that fault. **One test of this binary
/// runs at one time.** See T-144.
struct TheDatabaseOfTheTest {
    _turn: std::sync::MutexGuard<'static, ()>,
    _directory: tempfile::TempDir,
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
        _directory: directory,
    }
}

/// Writes the session of this program, as a playback does.
fn a_session_of_this_program(id_session: &str, id_item: &str, at: u32) {
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
        "the-account",
        "the-server",
    )
    .expect("the session of the test");
}

/// Writes the session of **another program of the same account**. `moments_ago`
/// gives the age of the last second of that playback: a program that lives holds
/// a moment of now, and a program that stopped without a correct exit holds an
/// old one.
fn a_session_of_another_program(id_session: &str, id_item: &str, at: u32, moments_ago: u64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock of the machine")
        .as_secs();

    let conn = toutui::db::migrate::open_conn().expect("the database of the test");

    conn.execute(
        "INSERT INTO listening_session (id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter, username, server, owner, heartbeat)
         VALUES (?1, ?2, ?3, '1800', 0, '', 0, 'A Book', 'An Author', 1, '', 'the-account', 'the-server', 'another-program', ?4)",
        params![id_session, id_item, at, now.saturating_sub(moments_ago) as i64],
    )
    .expect("the row of the other program");
}

/// Counts the rows of the table, and no rule of the program stands between the
/// test and that number.
fn the_rows_of_the_table() -> i64 {
    let conn = toutui::db::migrate::open_conn().expect("the database of the test");
    conn.query_row("SELECT COUNT(*) FROM listening_session", [], |row| {
        row.get(0)
    })
    .expect("the count of the rows")
}

/// The program takes the session that it opened itself.
#[test]
fn the_program_takes_the_session_that_it_owns() {
    let _dir = a_database_of_the_test();

    a_session_of_this_program("a-session", "a-book", 810);

    let session = get_listening_session("the-account", "the-server")
        .expect("the database")
        .expect("the program must take its own session");

    assert_eq!(session.id_session, "a-session");
    assert_eq!(session.current_time, 810);
}

/// **The first effect of T-140.** The playback of the second program closed the
/// live session of the first one, because `play_media` closes the session that
/// the database holds.
#[test]
fn the_session_of_a_program_that_lives_stays_away_from_another_program() {
    let _dir = a_database_of_the_test();

    a_session_of_another_program("the-session-of-the-other", "the-book-of-the-other", 120, 1);

    assert!(
        get_listening_session("the-account", "the-server")
            .expect("the database")
            .is_none(),
        "the program took the live session of another program of the same \
         account. It then closes that session on the server, and the playback of \
         the other program loses every sync."
    );
}

/// **The second effect of T-140.** The key `Q` of one program removed the row of
/// the other one, and the position of that user then reached no server at all.
#[test]
fn the_close_of_a_session_leaves_the_session_of_a_program_that_lives() {
    let _dir = a_database_of_the_test();

    a_session_of_another_program("the-session-of-the-other", "the-book-of-the-other", 120, 1);

    delete_listening_session("the-account", "the-server").expect("the database");

    assert_eq!(
        the_rows_of_the_table(),
        1,
        "the close of one program removed the session of another program that \
         lives. That program then holds no row, and its position never reaches \
         the server."
    );
}

/// A playback of the second program keeps the row of the first one, therefore
/// each program holds the session that it opened.
#[test]
fn a_playback_of_a_second_program_keeps_the_session_of_the_first() {
    let _dir = a_database_of_the_test();

    a_session_of_another_program("the-session-of-the-other", "the-book-of-the-other", 120, 1);
    a_session_of_this_program("my-session", "my-book", 5);

    assert_eq!(
        the_rows_of_the_table(),
        2,
        "a playback of one program removed the session of another program that \
         lives"
    );

    let mine = get_listening_session("the-account", "the-server")
        .expect("the database")
        .expect("the program must take its own session");

    assert_eq!(
        mine.id_session, "my-session",
        "the program took the session of another program, and not its own"
    );
}

/// **The rule of T-4 stays.** A program that stopped without a correct exit
/// leaves its row, and the next program sends that position one time.
#[test]
fn the_session_of_a_program_that_died_belongs_to_the_program_that_asks() {
    let _dir = a_database_of_the_test();

    a_session_of_another_program(
        "the-session-that-waits",
        "the-book-that-waits",
        640,
        THE_LIMIT_OF_THE_HEARTBEAT + 5,
    );

    let session = get_listening_session("the-account", "the-server")
        .expect("the database")
        .expect(
            "a row that no program touched belongs to a program that stopped \
             without a correct exit. The next program must send that position: \
             the rule of T-4.",
        );

    assert_eq!(session.id_session, "the-session-that-waits");
    assert_eq!(session.current_time, 640);
}

/// The second of a playback says that the program lives. A playback that stands
/// still holds its row, and a program that comes after it therefore takes
/// nothing.
#[test]
fn the_second_of_a_playback_says_that_the_program_lives() {
    let _dir = a_database_of_the_test();

    a_session_of_another_program(
        "the-session-of-the-other",
        "the-book-of-the-other",
        640,
        THE_LIMIT_OF_THE_HEARTBEAT + 5,
    );

    // The loop of that playback writes one second of the position.
    update_current_time(641, "the-session-of-the-other").expect("the database");

    assert!(
        get_listening_session("the-account", "the-server")
            .expect("the database")
            .is_none(),
        "the second of a playback must say that the program lives, therefore \
         `update_current_time` writes the moment of that second."
    );
}
