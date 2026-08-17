//! A refresh that did not read the accounts keeps the program of the user. See
//! T-205.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME`, and that variable belongs to the process: two test
//! functions of one binary fight for it. See T-144 and T-157.
//!
//! The measurement of 2026-08-14 with `docs/harness/hold_the_lock.py` (T-199),
//! against the sandbox, with a program that stood in the Library view:
//!
//! | The key | The program of v0.8.34 |
//! |---|---|
//! | `R` | the program went away: `Toutui stops: it cannot read the accounts of its database.` |
//! | Shift-Tab | the same, after `The program shows the library "Podcasts" now.` |
//!
//! **A refresh is not a start.** T-199 stops the program when the read of the
//! accounts of `main` fails, and at the start that is right: a program with no
//! account can do no work. A refresh holds the account, the token, every list,
//! and the playback of the user already, and **two programs of one account is a
//! condition of this fork** (T-140): a second Toutui that writes for six seconds
//! must take none of them away.
//!
//! The condition of this test is a file that holds no database, which gives the
//! same fault of `open_conn` with no wait at all.

use toutui::db::crud;

#[test]
fn a_refresh_of_a_database_that_said_nothing_keeps_the_program() {
    // 1. The report of a read of the accounts that failed holds its category,
    //    and a report of a fault of the server does not.
    let of_the_database = color_eyre::eyre::Report::new(toutui::db::TheAccountsDidNotCome(
        "database is locked".into(),
    ))
    .wrap_err("the accounts of the start");

    assert!(
        toutui::api::client::error::the_accounts_did_not_come(&of_the_database),
        "a report that holds the fault of the accounts says so, at every depth"
    );

    let of_the_server =
        color_eyre::eyre::Report::new(toutui::api::client::error::ApiError::Server(500));

    assert!(
        !toutui::api::client::error::the_accounts_did_not_come(&of_the_server),
        "a fault of the server is no fault of the database, and it keeps the road of T-172"
    );

    // 2. The refresh of `src/main.rs` reads that category, and the arm that
    //    stops the program stands behind it. **A test of the source is the road
    //    of a decision of the loop of the program** (T-135, T-143, and T-204).
    let of_the_loop = std::fs::read_to_string("src/main.rs").unwrap();

    let the_refresh = of_the_loop
        .split("let the_new_application")
        .nth(1)
        .expect("the refresh makes the new application");

    let the_category = the_refresh
        .find("the_accounts_did_not_come")
        .expect("the refresh reads the fault of the accounts of the database");
    let the_stop = the_refresh
        .find("the_program_stops_with_words")
        .expect("the refresh keeps the road of T-172 for a fault of the server");

    assert!(
        the_category < the_stop,
        "the fault of the database stands before the stop of the program"
    );
    assert!(
        the_refresh.contains("THE_REFRESH_DID_NOT_READ_THE_DATABASE"),
        "the row of the message says why the screen did not change"
    );
    assert!(
        the_refresh.contains("app.must_refresh = false"),
        "the mark of the refresh goes away, or every key after this one refreshes again"
    );

    // 3. The two functions of the database that the sweep of T-200 did not
    //    reach. Each of them said `Ok(())` for a connection that it did not get,
    //    therefore no correction of a caller could reach them.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());

    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    std::fs::write(
        dir.path().join("toutui").join("db.sqlite3"),
        b"this file holds no database at all",
    )
    .unwrap();

    assert!(
        crud::update_library_sort("toutuitest", "title", false, "", "").is_err(),
        "the sequence of a library of a database that says nothing is no sequence"
    );
    assert!(
        crud::save_the_queue("toutuitest", "the-server", &[]).is_err(),
        "the queue of a database that says nothing is no queue: the disk is the truth of it (T-147)"
    );

    // 4. The three keys of the user that write these rows say why. **A key of
    //    the user that writes the disk takes a sentence** (T-199), and that
    //    sentence names a key of the view that the user sees at that moment
    //    (T-183).
    let of_the_application = std::fs::read_to_string("src/app.rs").unwrap();

    for (the_key, the_words) in [
        (
            "fn take_the_next_library",
            "THE_NEXT_LIBRARY_DID_NOT_REACH_THE_DISK",
        ),
        (
            "fn the_disk_takes_the_sequence_of_the_library",
            "THE_SEQUENCE_DID_NOT_REACH_THE_DISK",
        ),
    ] {
        let the_body = of_the_application
            .split(the_key)
            .nth(1)
            .unwrap_or_else(|| panic!("{} stands in the application", the_key));

        assert!(
            the_body[..the_body.len().min(2600)].contains(the_words),
            "{} says {} when the disk did not take the write",
            the_key,
            the_words
        );
    }

    assert!(
        of_the_application.contains("THE_LIBRARY_DID_NOT_REACH_THE_DISK"),
        "the view of the libraries of the settings says why too"
    );

    // 5. No key of these three writes the disk with `let _ =` any more.
    for the_line in of_the_application
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
    {
        assert!(
            !the_line.contains("let _ = update_id_selected_lib")
                && !the_line.contains("let _ = crate::db::crud::update_library_sort"),
            "a write of the disk of a key of the user reads its answer: {}",
            the_line.trim()
        );
    }
}
