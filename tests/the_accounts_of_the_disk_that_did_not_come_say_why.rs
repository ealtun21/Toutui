//! A read of the accounts that failed is not a database with no account.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME`, and that variable belongs to the process: two test
//! functions of one binary fight for it. See T-144 and T-157.
//!
//! The measurement of T-199: a second program of one account held the database
//! of the program, therefore `select_default_usr` came back with a fault. The
//! old code read that fault with `if let Ok(...)`, therefore `Database::new`
//! gave a list of no account, and `src/main.rs` drew the login screen of a first
//! start. The row of the account stood on the disk all the time.
//!
//! The condition of this test is a file that holds no database. It gives the
//! same fault of `open_conn` with no wait at all: a lock of a second program
//! needs the busy timeout of five seconds of rusqlite first.

use toutui::db::database_struct::Database;

#[tokio::test]
async fn a_read_of_the_accounts_that_failed_is_not_a_database_with_no_account() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());

    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    // A file of bytes that hold no database. SQLite says "file is not a
    // database" for it, and it says that with no wait.
    std::fs::write(
        dir.path().join("toutui").join("db.sqlite3"),
        b"this file holds no database at all",
    )
    .unwrap();

    let Err(report) = Database::new().await else {
        panic!("a database that says nothing must not give a list of no account");
    };

    // The fault holds its own type, because the words of the user must name the
    // database and not the server.
    let of_the_accounts = report
        .chain()
        .find_map(|cause| cause.downcast_ref::<toutui::db::TheAccountsDidNotCome>());

    assert!(
        of_the_accounts.is_some(),
        "the fault must say that the accounts did not come: {:?}",
        report
    );

    // The words of the program that stops name the database of the program. The
    // words of a fault of the server say that the program cannot read the lists
    // of the server, and a program that did not read its own database must not
    // say that. See T-91 and T-172.
    let words = toutui::api::client::error::the_words_of_a_program_that_stops(
        &report,
        "toutuitest",
        "127.0.0.1:13399",
    );

    assert!(
        words.contains("it cannot read the accounts of its database"),
        "the words must name the database: {}",
        words
    );
    assert!(
        !words.contains("the lists of the server"),
        "the words must not name the lists of the server: {}",
        words
    );
    assert!(
        words.contains("Stop a second Toutui of this account"),
        "the words must say what the user does: {}",
        words
    );
}
