//! A log out that keeps the place of the user gives that place to the server
//! later. See T-296.
//!
//! **The measurement of 2026-08-16, of the real program v0.8.124 inside tmux
//! against the sandbox**, with `docs/harness/one_method_fails.py 13500 13399
//! requests.log PATCH:/api/me/progress` and the one address
//! `http://127.0.0.1:13500` of the account (the trap 129):
//!
//! - the user read two chapters of `Alice in Wonderland`, and the key `h` said
//!   "The server did not take the place: The server reported a fault. Status
//!   500." The row of `pending_ebook_progress` then held
//!   `epubcfi(/6/8!/4/2/2/1:0)` at 0.0916 (T-294);
//! - the key `l` of the view of the accounts, two times, logged out of the one
//!   account: `users` held **0** rows, and the row of the place **stayed**;
//! - a second machine of the user read to the half of the book, and the server
//!   held `epubcfi(/6/30!/4/2/2/1:0)` at 0.5;
//! - the user logged in again with the same account and the same server. The
//!   start of that program said "1 place(s) of a book wait for the server", and
//!   it sent the place of the program before the log out. **The server then held
//!   `epubcfi(/6/8!/4/2/2/1:0)` at 0.0916, and the place of the second machine
//!   went away.**
//!
//! **A token that the server refused takes the other road** (T-123): that
//! account comes back at once with the same name, therefore the places of it
//! stay. This test guards the two roads together.
//!
//! This test writes `XDG_CONFIG_HOME`, therefore the parts of it stay in one
//! function (the trap 8 of the harness, and T-144).

use toutui::db::crud::{
    count_pending_progress, db_insert_usr, delete_user, get_pending_ebook_progress,
    insert_listening_session, insert_pending_ebook_progress, insert_pending_progress,
    remove_the_account, select_every_usr, the_words_of_a_log_out, PendingEbookProgress,
    PendingProgress,
};
use toutui::db::database_struct::User;

const SERVER: &str = "http://127.0.0.1:13399";

/// An account of the database, with the values that a login writes.
fn an_account(username: &str) -> User {
    User {
        username: username.to_string(),
        server_address: SERVER.to_string(),
        token: format!("the-token-of-{}", username),
        is_default_usr: username == "toutuitest",
        name_selected_lib: "Books".to_string(),
        id_selected_lib: "lib-1".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// Writes the three places of the user of one account.
fn the_places_of(username: &str) {
    insert_pending_progress(
        username,
        SERVER,
        &PendingProgress {
            id_item: format!("the-media-of-{}", username),
            id_pod: String::new(),
            current_time: 314.0,
            duration: 1800.0,
            is_finished: false,
            updated_at: 1_786_855_602_891,
        },
    )
    .unwrap();

    insert_pending_ebook_progress(
        username,
        SERVER,
        &PendingEbookProgress {
            id_item: format!("the-book-of-{}", username),
            location: "epubcfi(/6/8!/4/2/2/1:0)".to_string(),
            fraction: 0.091_630_833_716_182_39,
            updated_at: 1_786_855_602_891,
        },
    )
    .unwrap();

    insert_listening_session(
        format!("the-session-of-{}", username),
        format!("the-media-of-{}", username),
        314,
        "1800".to_string(),
        String::new(),
        0,
        "A Long Test Book".to_string(),
        "The Author".to_string(),
        true,
        String::new(),
        username,
        SERVER,
    )
    .unwrap();
}

#[test]
fn a_log_out_takes_the_places_of_the_user() {
    // No line of this test may touch the files of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("XDG_DATA_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    db_insert_usr(&vec![an_account("toutuitest")]).unwrap();
    db_insert_usr(&vec![an_account("toutuilimited")]).unwrap();

    the_places_of("toutuitest");
    the_places_of("toutuilimited");

    // The user logs out of the second account.
    delete_user("toutuilimited").unwrap();

    // The row of the account went away, and the row of the other account stays.
    let the_accounts = select_every_usr().unwrap();
    assert_eq!(the_accounts.len(), 1, "the log out took one account away");
    assert_eq!(the_accounts[0].0, "toutuitest");

    // **The places of that account went away with it.** A row that stays reaches
    // the server at the start of the program after a login with the same name,
    // and it stands over the place of another machine of the user.
    assert_eq!(
        count_pending_progress("toutuilimited", SERVER).unwrap(),
        0,
        "the position of a playback of an account that went away stays on the disk"
    );
    assert_eq!(
        get_pending_ebook_progress("toutuilimited", SERVER)
            .unwrap()
            .len(),
        0,
        "the place of a book of an account that went away stays on the disk"
    );
    assert_eq!(
        the_number_of_the_sessions("toutuilimited"),
        0,
        "the session of an account that went away stays on the disk"
    );

    // **The places of every other account stay.** The name of an account is the
    // primary key of `users`, therefore the rows of one name belong to one
    // account.
    assert_eq!(
        count_pending_progress("toutuitest", SERVER).unwrap(),
        1,
        "the log out took the position of a second account"
    );
    assert_eq!(
        get_pending_ebook_progress("toutuitest", SERVER)
            .unwrap()
            .len(),
        1,
        "the log out took the place of a book of a second account"
    );
    assert_eq!(
        the_number_of_the_sessions("toutuitest"),
        1,
        "the log out took the session of a second account"
    );

    // **A token that the server refused is not a log out** (T-123). That account
    // comes back with the same name at once, therefore its places must stay.
    remove_the_account("toutuitest").unwrap();

    assert_eq!(
        select_every_usr().unwrap().len(),
        0,
        "the row of the account of a token that is not valid must go away"
    );
    assert_eq!(
        count_pending_progress("toutuitest", SERVER).unwrap(),
        1,
        "a token that a session renews took the position of the user away"
    );
    assert_eq!(
        get_pending_ebook_progress("toutuitest", SERVER)
            .unwrap()
            .len(),
        1,
        "a token that a session renews took the place of a book away"
    );

    // **The words name the places that went away** (T-118): the user read that
    // media while the server did not answer, and no machine holds that place now.
    let one = the_words_of_a_log_out("toutuilimited", 1);
    assert!(one.contains("toutuilimited"), "{}", one);
    assert!(one.contains("1 place of the user"), "{}", one);
    assert!(one.contains("it went away"), "{}", one);

    let three = the_words_of_a_log_out("toutuilimited", 3);
    assert!(three.contains("3 places of the user"), "{}", three);
    assert!(three.contains("they went away"), "{}", three);

    // An account that holds no place of the user keeps the words of the start.
    let none = the_words_of_a_log_out("toutuilimited", 0);
    assert!(!none.contains("went away with the account"), "{}", none);
    assert!(none.contains("Start the program again."), "{}", none);
}

/// The number of rows of `listening_session` of one account.
fn the_number_of_the_sessions(username: &str) -> usize {
    let conn = toutui::db::migrate::open_conn().unwrap();

    conn.query_row(
        "SELECT COUNT(*) FROM listening_session WHERE username = ?1",
        [username],
        |row| row.get::<_, i64>(0),
    )
    .unwrap() as usize
}
