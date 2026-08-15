//! A refresh that cannot read the configuration file keeps the program of the
//! user. See T-266.
//!
//! **The parts of this test stay in one function.** See T-144 and T-157.
//!
//! The measurement of 2026-08-15 of the real program v0.8.94 inside tmux,
//! against the sandbox, with a program that stood in the Home view. The user
//! changed one line of `config.toml` and left one bracket out
//! (`log_background_color = [40, 40, 40`), and then pressed `R`:
//!
//! | The key | The program of v0.8.94 |
//! |---|---|
//! | `R` | the program went away with the status 1: `Toutui stops: it cannot read its configuration file.` |
//! | `R`, with a file of no fault | the program stands, and the Home view comes again |
//!
//! **A refresh is not a start** (T-205). T-265 stops the program when the read
//! of `src/main.rs` fails, and at the start that is right: the program holds no
//! value of the user at all. The key `R` reads that file again (T-142), and the
//! application of the user holds the account, every list, and the playback
//! already; the values of the file that it read at its start stay good.
//! Therefore that application stays, and the row of the message says why the
//! screen did not change.
//!
//! **The two other readers of that file hold this rule already**:
//! `take_the_limit_of_the_cache_of_the_file` of `src/app.rs` and
//! `read_the_limit_of_the_configuration_again` of `src/logic/reader/cache.rs`
//! each say "A file that the program cannot read changes nothing".

#[test]
fn a_refresh_of_a_file_that_the_program_cannot_read_keeps_the_program() {
    // 1. The whole chain: a file of a shape that the crate refuses gives a
    //    report, and the refresh reads the category of that report.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, b"[colors]\nbackground_color = [40, 40, 40\n").unwrap();

    let report = toutui::config::load_config_from(&path)
        .expect_err("a file of a shape that the crate refuses gives no value of the user");

    assert!(
        toutui::api::client::error::the_configuration_file_did_not_come(&report),
        "a report of a file that the program cannot read says so"
    );

    // The same report, inside a report of a caller. The function reads every
    // cause of the chain.
    let of_a_caller =
        color_eyre::eyre::Report::new(toutui::config::TheConfigurationFileDidNotCome {
            path: "/home/one/.config/toutui/config.toml".to_string(),
            reason: "TOML parse error at line 40, column 1".to_string(),
        })
        .wrap_err("the configuration of the refresh");

    assert!(
        toutui::api::client::error::the_configuration_file_did_not_come(&of_a_caller),
        "a report that holds the fault of the file says so, at every depth"
    );

    // 2. A fault of the server and a fault of the database are no fault of the
    //    file, and each of them keeps the road of its own item.
    let of_the_server =
        color_eyre::eyre::Report::new(toutui::api::client::error::ApiError::Server(500));

    assert!(
        !toutui::api::client::error::the_configuration_file_did_not_come(&of_the_server),
        "a fault of the server is no fault of the file, and it keeps the road of T-172"
    );

    let of_the_database = color_eyre::eyre::Report::new(toutui::db::TheAccountsDidNotCome(
        "database is locked".into(),
    ));

    assert!(
        !toutui::api::client::error::the_configuration_file_did_not_come(&of_the_database),
        "a fault of the database is no fault of the file, and it keeps the road of T-205"
    );

    assert!(
        !toutui::api::client::error::the_accounts_did_not_come(&report),
        "a fault of the file is no fault of the database"
    );

    // 3. The refresh of `src/main.rs` reads that category, and the arm that stops
    //    the program stands behind it. **A test of the source is the road of a
    //    decision of the loop of the program** (T-135, T-143, and T-204).
    let of_the_loop = std::fs::read_to_string("src/main.rs").unwrap();

    let the_refresh = of_the_loop
        .split("let the_new_application")
        .nth(1)
        .expect("the refresh makes the new application");

    let the_category = the_refresh
        .find("the_configuration_file_did_not_come")
        .expect("the refresh reads the fault of the configuration file");
    let the_stop = the_refresh
        .find("the_program_stops_with_words")
        .expect("the refresh keeps the road of T-172 for a fault of the server");

    assert!(
        the_category < the_stop,
        "the fault of the configuration file stands before the stop of the program"
    );
    assert!(
        the_refresh.contains("THE_REFRESH_DID_NOT_READ_THE_CONFIGURATION_FILE"),
        "the row of the message says why the screen did not change"
    );

    // 4. The words for the user. They name the file and the key of the view that
    //    the user sees at that moment (T-183), and **they say nothing of the
    //    server and nothing of an account** (T-91): the file belongs to the user.
    let the_words = toutui::ui::keys::THE_REFRESH_DID_NOT_READ_THE_CONFIGURATION_FILE;

    for the_word in ["configuration file", "press R again"] {
        assert!(
            the_words.contains(the_word),
            "the sentence of the refresh says \"{}\": {}",
            the_word,
            the_words
        );
    }

    for the_word in ["server", "account", "database", "administrator"] {
        assert!(
            !the_words.to_lowercase().contains(the_word),
            "a fault of the file of the user says no reason that the program does not have \
             (\"{}\"): {}",
            the_word,
            the_words
        );
    }
}
