//! An account that the database keeps after a token that the server refused
//! says why.
//!
//! **The parts of this test stay in one function.** The test reads the source of
//! `src/main.rs`, and it holds one rule of one road. See T-144 and T-157.
//!
//! The measurement of T-269, of the real program v0.8.97 inside tmux, against
//! the sandbox behind `docs/harness/a_status_of_one_path.py` with the status
//! 401 for `/api/libraries`, and with a trigger of SQLite of
//! `BEFORE DELETE ON users` (T-213). The server said that the token is not
//! valid, `the_program_needs_a_new_token` did not remove the row of the
//! account, and the `map_err(color_eyre::eyre::Report::msg)?` of `src/main.rs`
//! gave the report to the runtime of Rust:
//!
//! ```text
//! Error: The account toutuitest stays in the database: the disk takes no removal of the account
//! Location:
//!     …/library/core/src/ops/function.rs:250:5
//! ```
//!
//! That text names a line of a file of the standard library of Rust, which no
//! user must read (T-172), it holds no sentence of Toutui and no road back, and
//! `grep -c 'the program stops'` of the log of that run gave 0.

/// The removal of the account of a token that the server refused must not give
/// its fault to the runtime.
#[test]
fn an_account_that_did_not_go_away_says_why() {
    let source = include_str!("../src/main.rs");

    // **A bare `?` of `main` is the words of the runtime** (T-267, T-268, and
    // T-269). The two roads of the token that the server refused — the start and
    // the key `R` — held one each.
    assert!(
        !source.contains("map_err(color_eyre::eyre::Report::msg)?"),
        "the removal of the account of src/main.rs must not give its fault to the runtime of Rust"
    );

    // The two roads stay: the start of the session and the key `R`.
    let of_the_two_roads = source.matches("the_program_needs_a_new_token(").count();

    assert_eq!(
        of_the_two_roads, 2,
        "the start and the key `R` each ask for a new token; the source holds {} of them",
        of_the_two_roads
    );

    // Each of the two gives its fault to the words of a program that stops.
    let of_the_words = source.matches("if let Err(fault) =").count();

    assert!(
        of_the_words >= 2,
        "the two roads each give the fault of the removal to the words of a program that stops; \
         the source holds {} of them",
        of_the_words
    );

    // The words name the database, the account, the server, and the road back.
    let report = color_eyre::eyre::Report::new(toutui::db::TheAccountDidNotGoAway {
        username: "toutuitest".to_string(),
        reason: "attempt to write a readonly database".to_string(),
    });

    let words = toutui::api::client::error::the_words_of_a_program_that_stops(
        &report,
        "toutuitest",
        "http://127.0.0.1:13399",
    );

    assert!(
        words.contains("it did not remove an account of its database"),
        "the words must name the database: {}",
        words
    );
    assert!(
        words.contains("The account toutuitest stays in the database"),
        "the words must name the account of the row that stays: {}",
        words
    );
    assert!(
        words.contains("attempt to write a readonly database"),
        "the words must hold what the database said: {}",
        words
    );
    assert!(
        words.contains("http://127.0.0.1:13399"),
        "the server did not accept the token, and that is a reason that the program has: {}",
        words
    );
    assert!(
        words.contains("Correct the database, and start Toutui again."),
        "the words must name the road back: {}",
        words
    );
    assert!(
        words.contains("Toutui changed nothing"),
        "the user must know that the program wrote nothing: {}",
        words
    );

    // **This is not a fault of the lists of the server.** The server answered
    // the request, and a view never says a reason that the program does not have
    // (T-91).
    assert!(!words.contains("lists of the server"), "{}", words);

    // **No line of the source of this program, and no line of a file of the
    // standard library of Rust** (T-172).
    assert!(!words.contains("Location"), "{}", words);
    assert!(!words.contains("src/"), "{}", words);
    assert!(!words.contains(".rs"), "{}", words);
}
