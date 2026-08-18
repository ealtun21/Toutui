//! **The value of a filter of an author and of a series holds an identity of
//! one library** (T-383). The program of v0.8.213 kept such a filter when the
//! user took the next library: the server answered 0 items for the identity
//! of Lewis Carroll in a second library of 2056 books, and the view said
//! that no media agrees with the filter. The version 12 of the database
//! holds the library of the filter beside it, and the start keeps a filter
//! of another library out of the request and out of the header. The row of
//! the disk keeps the filter: the library of the filter gives it back.
//!
//! The parts of this test stay in one function: the test writes
//! `XDG_CONFIG_HOME`, and that variable belongs to the process (T-144).

use toutui::db::{crud, migrate};
use toutui::logic::sort_filter::is_a_filter_of_another_library;

#[test]
fn a_filter_of_an_author_stays_in_its_library() {
    // The two kinds whose value holds an identity of one library.
    for filter in [
        "authors.MzEyYzQyZmYtZTgwMC00YjI5LTk5NzQtZDJkODk5ZDBiYmE5",
        "series.YzE0YzYxYzM",
    ] {
        assert!(
            is_a_filter_of_another_library(filter, "lib-a", "lib-b"),
            "the filter {} of one library must stay out of another library",
            filter
        );

        // The filter acts in the library where the user took it.
        assert!(
            !is_a_filter_of_another_library(filter, "lib-a", "lib-a"),
            "the filter {} acts in its own library",
            filter
        );

        // A row that an older program wrote holds no library of the filter:
        // the filter then stays, as it did before the version 12.
        assert!(
            !is_a_filter_of_another_library(filter, "", "lib-b"),
            "a filter of no known library stays"
        );
    }

    // The five other kinds hold a name or the position, and a name acts in
    // every library: they ride with their meaning.
    for filter in [
        "genres.RmFpcnkgVGFsZXM=",
        "tags.YS10ZXN0LXRhZw==",
        "languages.ZW4=",
        "narrators.U29tZSBOYXJyYXRvcg==",
        "publishers.U29tZSBQdWJsaXNoZXI=",
        "progress.ZmluaXNoZWQ=",
    ] {
        assert!(
            !is_a_filter_of_another_library(filter, "lib-a", "lib-b"),
            "the filter {} rides with its meaning",
            filter
        );
    }

    // A filter of no value is no filter.
    assert!(!is_a_filter_of_another_library("", "lib-a", "lib-b"));

    // **The migration of the version 12 writes the library of the row beside
    // a filter that stands**, because that filter acted in the library of the
    // row. A row of no filter takes no library.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = migrate::open_conn().unwrap();

    conn.execute(
        "INSERT INTO users (username, server_address, token, name_selected_lib,
             id_selected_lib, is_loop_break, has_played_before,
             speed_rate, is_show_key_bindings)
         VALUES ('a', '', '', 'Books', 'lib-of-the-row', '', '0', 1.0, '1')",
        [],
    )
    .unwrap();

    crud::update_library_sort(
        "a",
        "",
        false,
        "authors.QUFBQQ==",
        "Lewis Carroll",
        "lib-of-the-row",
    )
    .unwrap();

    conn.execute("ALTER TABLE users DROP COLUMN library_filter_lib", [])
        .unwrap();
    conn.execute_batch("PRAGMA user_version = 11").unwrap();

    migrate::run_migrations(&conn).unwrap();

    assert_eq!(
        migrate::schema_version(&conn).unwrap(),
        migrate::LATEST_VERSION,
        "the migration moves a database of the version 11 forward"
    );

    let (_, _, filter, _, lib) = crud::get_library_sort("a").unwrap();

    assert_eq!(filter, "authors.QUFBQQ==");
    assert_eq!(
        lib, "lib-of-the-row",
        "the migration writes the library of the row beside the filter"
    );

    // A row of no filter takes no library of a filter.
    crud::update_library_sort("a", "", false, "", "", "").unwrap();
    conn.execute("ALTER TABLE users DROP COLUMN library_filter_lib", [])
        .unwrap();
    conn.execute_batch("PRAGMA user_version = 11").unwrap();

    migrate::run_migrations(&conn).unwrap();

    let (_, _, _, _, lib) = crud::get_library_sort("a").unwrap();

    assert_eq!(lib, "", "a row of no filter holds no library of a filter");

    // **The migration is safe to run two times**, as the rule of the head of
    // `src/db/migrate.rs` says.
    migrate::run_migrations(&conn).unwrap();
}
