//! The message of the login screen that the disk refused. See T-270.
//!
//! **The message of the login screen made a road through the disk.** Every
//! sentence of that screen — a wrong password, an address of no `http://`, a
//! field of no character, and the token that the server refused — went to the
//! column `login_err` of the table `others` with `let _ = update_login_err(...)`,
//! and the render read that column again at each frame. A disk that takes no
//! write of that column therefore gave a login screen with **no word at all**,
//! and no line of the log named it.
//!
//! **The measurement of 2026-08-16** of the real program v0.8.98 inside tmux, on
//! a screen of 160 columns and 45 rows, against the sandbox. The condition is a
//! trigger of SQLite of one column (T-213), and no other write of the program
//! fails:
//!
//! ```bash
//! sqlite3 "$DB" "CREATE TRIGGER the_disk_takes_no_message \
//!     BEFORE UPDATE OF login_err ON others \
//!     BEGIN SELECT RAISE(ABORT, 'the disk takes no message of the login screen'); END;"
//! ```
//!
//! The login screen took the address of the sandbox, the account `toutuitest`,
//! and a password that the server refuses. The screen then held the three lines
//! of the field of the address and nothing else. The control of the same run,
//! with no trigger at all, said
//! `The server refused the username or the password.`
//!
//! The correction gives that sentence a box of the process, and the render reads
//! that box first. The disk keeps the sentence for the **process after this
//! one**, because a token that the server refused starts the program again
//! (T-123): that road is the one reason of the column. A write that the disk
//! refused takes a line of the log.
//!
//! **This test needs no sandbox and no network**: a trigger of SQLite refuses
//! the write of one column, and every other read and every other write of the
//! program answers.
//!
//! **The parts of this test stay in one function.** The box belongs to the
//! process, therefore two test functions of one module fight for it (T-144 and
//! T-157).

use toutui::db::crud::{get_others, update_login_err};
use toutui::logic::auth::auth_input::{say_on_the_login_screen, the_message_of_the_login_screen};

/// The sentence that the process before this one left on the disk. See T-123.
const THE_SENTENCE_OF_THE_DISK: &str = "The token is not valid. Log in again.";

/// The sentence of a login that the server refused.
const THE_SENTENCE_OF_THE_LOGIN: &str = "The server refused the username or the password.";

fn a_statement(sql: &str) {
    let conn = toutui::db::migrate::open_conn().expect("the database of this test");
    conn.execute_batch(sql).expect("the statement of this test");
}

/// Gives the sentence that the column of the disk holds.
fn the_sentence_of_the_disk() -> String {
    match get_others().expect("the row of the messages of this test") {
        Some(value) => value.login_err,
        None => String::new(),
    }
}

#[test]
fn the_login_screen_says_a_message_that_the_disk_did_not_take() {
    let home = tempfile::tempdir().expect("the directory of this test");
    std::env::set_var("XDG_CONFIG_HOME", home.path());
    std::fs::create_dir_all(home.path().join("toutui")).expect("the directory of the program");

    {
        let conn = toutui::db::migrate::open_conn().expect("the database of this test");
        toutui::db::migrate::run_migrations(&conn).expect("the migration of this test");
    }

    // ## The first road: the box of this process holds nothing, therefore the
    // sentence of the process before this one comes of the disk.
    //
    // A token that the server refused writes that sentence and it starts the
    // program again (T-123). The first frame of the new program must hold it.
    update_login_err(THE_SENTENCE_OF_THE_DISK).expect("the sentence of the disk of this test");

    assert_eq!(
        the_message_of_the_login_screen(),
        THE_SENTENCE_OF_THE_DISK,
        "the first frame of a program that started again holds the sentence of the disk"
    );

    // ## The second road: the disk takes no write of that column.
    //
    // **A trigger `BEFORE UPDATE OF` fails the write of one column alone**
    // (T-213): every other read and every other write of the program answers,
    // therefore the login screen stands and the account of a login that works
    // reaches the disk.
    a_statement(
        "CREATE TRIGGER the_disk_takes_no_message \
         BEFORE UPDATE OF login_err ON others \
         BEGIN SELECT RAISE(ABORT, 'the disk takes no message of the login screen'); END;",
    );

    say_on_the_login_screen(THE_SENTENCE_OF_THE_LOGIN);

    assert_eq!(
        the_message_of_the_login_screen(),
        THE_SENTENCE_OF_THE_LOGIN,
        "the login screen says the reason of a login that failed, and the disk holds none of it"
    );

    assert_eq!(
        the_sentence_of_the_disk(),
        THE_SENTENCE_OF_THE_DISK,
        "the disk refused that write, therefore it keeps the sentence before it"
    );

    // ## The third road: the disk answers again, and a screen with no fault
    // holds no sentence at all.
    a_statement("DROP TRIGGER the_disk_takes_no_message;");

    say_on_the_login_screen("");

    assert_eq!(
        the_message_of_the_login_screen(),
        "",
        "a field that the program accepted takes the sentence of the fault away"
    );

    assert_eq!(
        the_sentence_of_the_disk(),
        "",
        "the disk answers again, therefore the process after this one reads no old sentence"
    );
}
