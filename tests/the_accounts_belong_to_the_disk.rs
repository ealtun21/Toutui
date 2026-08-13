//! The view of the accounts belongs to the disk, and the mark of the start
//! never stands on nobody. See T-155.
//!
//! **The measurement of 2026-08-14, with the sandbox and two sessions of tmux of
//! one `XDG_CONFIG_HOME`.** The database held `toutuitest` (the account of the
//! start) and `toutuilimited`. The window B logged out of `toutuilimited` with
//! the key `l`; the window A stood open, and its view of the accounts still held
//! the line of that account. The user of A pressed `c` on that line:
//!
//! - `make_this_account_the_default` took the mark from every account, and it
//!   gave the mark to no account: the name of that write named no row;
//! - the program started again, and it drew the **login screen**;
//! - `toutuitest` stood on the disk with its token, and every new window of the
//!   program drew that login screen too.
//!
//! The user must give a server, a name, and a password again for an account of
//! which they never logged out. That is the shape of T-136: no key of the
//! program gives the account back, in any view and after every start.
//!
//! This test needs no server. It writes `XDG_CONFIG_HOME`, therefore it stands
//! alone in its binary (the trap 8 of the harness, and T-144).

use toutui::db::crud::{
    an_account_takes_the_start_when_none_holds_it, db_insert_usr, make_this_account_the_default,
    remove_the_account, select_default_usr, select_every_usr,
};
use toutui::db::database_struct::User;
use toutui::logic::the_accounts::{
    the_account_of_the_line, the_text_of_an_account_that_is_gone, TheAccountOfTheLine,
};

/// An account of the database, with the values that a login writes.
fn an_account(username: &str, starts: bool) -> User {
    User {
        username: username.to_string(),
        server_address: "http://127.0.0.1:13399".to_string(),
        token: format!("the-token-of-{}", username),
        is_default_usr: starts,
        name_selected_lib: "Books".to_string(),
        id_selected_lib: "lib-1".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// The account of the start stays when a key names an account of no row, and a
/// database that holds no account of a start gives that work to the first
/// account.
#[test]
fn the_mark_of_the_start_never_stands_on_nobody() {
    // No line of this test may touch the files of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("XDG_DATA_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    db_insert_usr(&vec![an_account("toutuitest", true)]).unwrap();
    db_insert_usr(&vec![an_account("toutuilimited", false)]).unwrap();
    make_this_account_the_default("toutuitest").unwrap();

    // The window B logs out of the second account. The list of the window A
    // holds its line still: that list comes of `App::new`.
    let of_the_view_of_a = select_every_usr().unwrap();
    remove_the_account("toutuilimited").unwrap();

    // **The key of a line reads the disk before it acts** (the rule of T-142 and
    // of T-147), and it acts on the name of its own line.
    assert_eq!(
        the_account_of_the_line(&of_the_view_of_a, "toutuilimited"),
        TheAccountOfTheLine::ItStays,
        "the list of the window A is older than the disk"
    );
    assert_eq!(
        the_account_of_the_line(&select_every_usr().unwrap(), "toutuilimited"),
        TheAccountOfTheLine::ItIsGone,
        "the disk is the truth, and the key must find the account gone"
    );

    // The sentence names the account of the line. A key that does nothing says
    // why (T-79).
    let text = the_text_of_an_account_that_is_gone("toutuilimited");
    assert!(text.contains("toutuilimited"), "{}", text);

    // **The write of the mark of the start.** It named an account of no row, and
    // it took the mark from `toutuitest` before T-155.
    let rows = make_this_account_the_default("toutuilimited").unwrap();
    assert_eq!(rows, 0, "the database holds no such account");

    let of_the_start = select_default_usr().unwrap();
    assert_eq!(
        of_the_start.first().map(String::as_str),
        Some("toutuitest"),
        "the account of the start stays: the program drew the login screen here before T-155"
    );

    // **A database that already met that fault must find its account again.**
    // The user of such a database holds no key that gives the mark back.
    let conn = toutui::db::migrate::open_conn().unwrap();
    conn.execute("UPDATE users SET is_default_usr = 0", [])
        .unwrap();
    drop(conn);

    assert!(
        select_default_usr().unwrap().is_empty(),
        "the database of the fault holds no account of a start"
    );

    assert_eq!(
        an_account_takes_the_start_when_none_holds_it().unwrap(),
        Some("toutuitest".to_string()),
        "the first account takes the start of the program"
    );

    assert_eq!(
        select_default_usr().unwrap().first().map(String::as_str),
        Some("toutuitest"),
        "the program finds the account of the user again"
    );

    // An account holds the mark now, therefore the start changes no row.
    assert_eq!(
        an_account_takes_the_start_when_none_holds_it().unwrap(),
        None
    );

    // **No unit test reaches a key handler of `src/app.rs`**, therefore this
    // part reads the source, as the tests of T-131, T-143, T-149, T-150, and
    // T-151 do: the view of the accounts and its two keys read the disk.
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("the test must read src/app.rs");

    let of_the_key = source
        .split_once("pub fn this_account_starts(&mut self) {")
        .expect("the handler of the key c must stand in src/app.rs")
        .1
        .split_once("\n    }\n")
        .expect("the handler must end")
        .0;

    assert!(
        of_the_key.contains("the_accounts_come_from_the_disk()"),
        "the key c must read the accounts of the disk before it writes the mark of the start"
    );
    assert!(
        of_the_key.contains("the_account_of_the_line("),
        "the key c must find an account that a second program removed"
    );

    assert!(
        source.contains(
            "Some(0) => {\n                            self.the_accounts_come_from_the_disk();"
        ),
        "the view of the accounts must read the disk when it opens"
    );
}
