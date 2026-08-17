//! The start of the program names the filter that stands. See T-380.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME`, and that variable belongs to the process: two test
//! functions of one binary fight for it. See T-144 and T-157.
//!
//! # The condition
//!
//! The value of a filter of an author and of a series holds an identity in
//! base64, and no arithmetic gives the name of it back. T-379 gave the header
//! the box `the_name_that_stands`, and that box lives in the process: the
//! measurement of v0.8.210 inside tmux, with the filter of the author Lewis
//! Carroll in the row of the account, gave the header
//! `⇅ The sequence of the server ▣ An author` at the start — the group, and
//! not the name that the user took.
//!
//! # The correction
//!
//! The version 11 of the database gives the row of the account the column
//! `library_filter_name`. The write of the sequence takes the name of the box
//! (`the_name_for_the_disk`), and the start seeds the box out of the row: the
//! header of the next start then names the filter, and a write of the sequence
//! alone keeps the name that stands.

use toutui::db::{crud, migrate};
use toutui::logic::sort_filter;

#[test]
fn the_start_names_the_filter_of_the_disk() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = migrate::open_conn().unwrap();

    assert_eq!(
        migrate::schema_version(&conn).unwrap(),
        migrate::LATEST_VERSION,
        "a fresh database holds the latest schema"
    );

    // The version 4 of the schema renamed the column of VLC to
    // `has_played_before` and it removed `is_vlc_running`.
    conn.execute(
        "INSERT INTO users (username, server_address, token, name_selected_lib,
             id_selected_lib, is_loop_break, has_played_before,
             speed_rate, is_show_key_bindings)
         VALUES ('a', '', '', '', '', '', '0', 1.0, '1')",
        [],
    )
    .unwrap();

    // **The write takes the name beside the value, and the read gives it
    // back.** That pair is what the start of the program reads: without it,
    // the header names the group of an author and of a series alone.
    crud::update_library_sort(
        "a",
        "media.metadata.title",
        true,
        "authors.QUFBQQ==",
        "Lewis Carroll",
    )
    .unwrap();

    let (sort, desc, filter, name) = crud::get_library_sort("a").unwrap();

    assert_eq!(sort, "media.metadata.title");
    assert!(desc);
    assert_eq!(filter, "authors.QUFBQQ==");
    assert_eq!(
        name, "Lewis Carroll",
        "the row of the account holds the name that the user read"
    );

    // **A removal of the filter takes the name away too**: a row of no filter
    // must not keep the name of a filter that went away.
    crud::update_library_sort("a", "media.metadata.title", true, "", "").unwrap();

    let (_, _, filter, name) = crud::get_library_sort("a").unwrap();

    assert_eq!(filter, "");
    assert_eq!(name, "", "a row of no filter holds no name");

    // **The name of the write comes of the box of the last application.** The
    // start seeds that box out of the row, therefore a write of the sequence
    // alone — after a restart — keeps the name that stands.
    assert_eq!(
        sort_filter::the_name_for_the_disk(""),
        "",
        "a filter of no value takes no name"
    );
    assert_eq!(
        sort_filter::the_name_for_the_disk("authors.QkJCQg=="),
        "",
        "a value that no application named takes no name"
    );

    sort_filter::the_name_that_stands::keep("authors.QkJCQg==", "Long Author");

    assert_eq!(
        sort_filter::the_name_for_the_disk("authors.QkJCQg=="),
        "Long Author",
        "the write takes the name of the application that stands"
    );

    // **The migration reaches a database of the version 10.** The column
    // comes with the default of no character: an account of an older database
    // holds no name, and the header then reads the value itself for the five
    // kinds of `decode_base64`, and it names the group for an author and for
    // a series, as it did before this version.
    conn.execute("ALTER TABLE users DROP COLUMN library_filter_name", [])
        .unwrap();
    conn.execute_batch("PRAGMA user_version = 10").unwrap();

    migrate::run_migrations(&conn).unwrap();

    assert_eq!(
        migrate::schema_version(&conn).unwrap(),
        migrate::LATEST_VERSION,
        "the migration moves a database of the version 10 forward"
    );

    let (_, _, _, name) = crud::get_library_sort("a").unwrap();

    assert_eq!(
        name, "",
        "an account of an older database holds no name of its filter"
    );

    // **The migration is safe to run two times**, as the rule of the head of
    // `src/db/migrate.rs` says.
    migrate::run_migrations(&conn).unwrap();
}
