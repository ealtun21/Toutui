//! A function of the database that got no connection gives a fault. See T-200.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME`, and that variable belongs to the process: two test
//! functions of one binary fight for it. See T-144 and T-157.
//!
//! The measurement of T-200: 21 functions of `src/db/crud.rs` held the shape
//! `if let Ok(conn) = open_conn() { … } else { say("Error connecting to the
//! database."); error!(…) }` with `Ok(…)` after it. **A caller that read the
//! answer of a write therefore got the answer of a write that never happened.**
//! A download of the sandbox with a second program of one account on the
//! database: the file of the episode stood on the disk, the three writes each
//! wrote a line of the log, and the program said `Downloaded "Letter 45"` and
//! `"Letter 45" is now available offline.` while the database held no row of it.
//!
//! The condition of this test is a file that holds no database. It gives the
//! same fault of `open_conn` with no wait at all: a lock of a second program
//! needs the busy timeout of five seconds of rusqlite first
//! (`docs/harness/hold_the_lock.py`).

use toutui::db::crud;

#[test]
fn a_function_of_the_database_that_got_no_connection_gives_a_fault() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());

    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    std::fs::write(
        dir.path().join("toutui").join("db.sqlite3"),
        b"this file holds no database at all",
    )
    .unwrap();

    // The writes of a download. Each of them held the shape of T-200, and the
    // three of them together hold one download.
    assert!(
        crud::insert_download(
            "the-key",
            "toutuitest",
            "A Book",
            "An Author",
            "/a/path",
            60.0,
            "the-item",
            "the-server",
        )
        .is_err(),
        "the row of a download of a database that says nothing is no row"
    );
    assert!(
        crud::insert_download_file("the-key", "toutuitest", 1, "the-ino", "/a/path", 100, 60.0)
            .is_err(),
        "the row of a file of a database that says nothing is no row"
    );
    assert!(
        crud::keep_the_files_of_the_download("the-key", "toutuitest", &[1]).is_err(),
        "the removal of the rows of the files of a database that says nothing is no removal"
    );
    assert!(
        crud::delete_download("the-key", "toutuitest").is_err(),
        "the removal of a download of a database that says nothing is no removal"
    );

    // The writes and the reads of a session of a playback. A read that gives
    // `Ok(None)` is "no session of this program", and the close of a session
    // then reaches no row: the place of the user stays on the disk (T-145).
    assert!(
        crud::get_listening_session("toutuitest", "the-server").is_err(),
        "a database that says nothing holds no answer of a session"
    );
    assert!(
        crud::get_the_sessions_to_close("toutuitest", "the-server").is_err(),
        "a database that says nothing holds no list of the sessions to close"
    );
    assert!(
        crud::update_current_time(120, "the-session").is_err(),
        "the place of a session of a database that says nothing is no place"
    );

    // The row of the message of the login screen, and the account of the start.
    assert!(
        crud::update_login_err("a sentence").is_err(),
        "the row of the message of a database that says nothing is no row"
    );
    assert!(
        crud::update_is_show_key_bindings("1", "toutuitest").is_err(),
        "the setting of a database that says nothing is no setting"
    );

    // **The rows of a download come together, or the download is no download of
    // this program** (T-200). The caller of the download reads this answer, and
    // it says the words of a download that the database did not take.
    let plan = toutui::logic::download::plan::DownloadPlan {
        item_id: "the-item".to_string(),
        key: "the-key".to_string(),
        title: "A Book".to_string(),
        author: "An Author".to_string(),
        files: vec![toutui::logic::download::plan::AudioFilePlan {
            index: 1,
            ino: "the-ino".to_string(),
            filename: "001.mp3".to_string(),
            size: 100,
            duration: 60.0,
        }],
    };

    assert!(
        toutui::logic::download::the_rows_of_the_download(
            &plan,
            "toutuitest",
            "A Book",
            "An Author",
            "/a/path",
            "the-server",
            &[std::path::PathBuf::from("/a/path")],
        )
        .is_err(),
        "a download whose rows did not come is no download of the program"
    );

    // The words of that fault name the key that does the work of it (T-170).
    let words =
        toutui::logic::download::the_words_of_a_download_that_the_database_did_not_take("A Book");

    assert!(
        words.contains("A Book"),
        "the words must name the media: {}",
        words
    );
    assert!(
        words.contains("press the key D again"),
        "the words must name the key of the work: {}",
        words
    );
}
