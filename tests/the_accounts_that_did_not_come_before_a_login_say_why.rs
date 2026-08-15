//! A read of the accounts that failed before every account says why.
//!
//! **The parts of this test stay in one function.** The test reads the source of
//! `src/main.rs`, and it holds one rule of one road. See T-144 and T-157.
//!
//! The measurement of T-268, of the real program v0.8.96 inside tmux, with
//! `docs/harness/hold_the_lock.py`: `Database::new` of `src/main.rs` came back
//! with the fault of T-199, the `?` of that line gave the report to the runtime
//! of Rust, and the terminal of the user held
//!
//! ```text
//! Error: The program did not read the accounts of its database: database is locked
//! Location:
//!     src/db/database_struct.rs:68:27
//! ```
//!
//! That text names a line of the source of this program (T-172), it holds no
//! sentence of Toutui and no road back, and `grep -c 'the program stops'` of the
//! log of that run gave 0.
//!
//! The two reads of `Database::new` of `src/main.rs` stand before the login
//! screen, therefore the words of that stop name no account and no server.

/// The read of the accounts of the start must not give its fault to the runtime.
#[test]
fn the_accounts_that_did_not_come_before_a_login_say_why() {
    let source = include_str!("../src/main.rs");

    // **A bare `?` of `main` is the words of the runtime** (T-267 and T-268).
    // No read of the accounts of `src/main.rs` may hold one.
    assert!(
        !source.contains("Database::new().await?"),
        "the read of the accounts of src/main.rs must not give its fault to the runtime of Rust"
    );

    // The report of that read goes to the words of a program that stops, and
    // those words name no account and no server, because the read stands before
    // the login screen (T-91).
    let of_the_start = source
        .matches("Err(report) => the_program_stops_with_words(report, \"\", \"\"),")
        .count();

    assert!(
        of_the_start >= 3,
        "the two reads of Database::new and the login screen each give their report to the words \
         of a program that stops; the source holds {} of them",
        of_the_start
    );

    // The words of a read that failed before every account name the database,
    // and they name no account at all: a name of no character gives the sentence
    // "The account is .", and that sentence names nobody.
    let report = color_eyre::eyre::Report::new(toutui::db::TheAccountsDidNotCome(
        "database is locked".to_string(),
    ));

    let words = toutui::api::client::error::the_words_of_a_program_that_stops(&report, "", "");

    assert!(
        words.contains("it cannot read the accounts of its database"),
        "the words must name the database: {}",
        words
    );
    assert!(
        !words.contains("The account is"),
        "the words of a read before every account must name no account: {}",
        words
    );
    assert!(
        words.contains("Stop a second Toutui that uses this database"),
        "the words must say what the user does: {}",
        words
    );

    // The account that the program knows keeps its sentence.
    let of_an_account =
        toutui::api::client::error::the_words_of_a_program_that_stops(&report, "toutuitest", "");

    assert!(
        of_an_account.contains("The account is toutuitest."),
        "the words of a read of a program that holds an account must name it: {}",
        of_an_account
    );
}
