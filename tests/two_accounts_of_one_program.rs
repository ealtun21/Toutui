//! The program holds more than one account. See T-124.
//!
//! **The measurement of 2026-08-12 (the sweep of two accounts) found that the
//! condition could not exist**: the view of the login came only when the
//! database held no account, the view of the accounts listed the account of the
//! start alone, and no key gave the start to a different account. The database
//! held the values of a second account already.
//!
//! This test holds the three rules of the database that the keys need:
//!
//! - every account of the database comes to the view;
//! - one account starts the program, and one only;
//! - the account of the line takes that work, and every other row loses it.
//!
//! The test writes `XDG_CONFIG_HOME` and `XDG_DATA_HOME`, therefore it stands
//! alone in its binary (the trap 8 of the harness).

use toutui::db::crud::{
    db_insert_usr, make_this_account_the_default, remove_the_account, select_default_usr,
    select_every_usr,
};
use toutui::db::database_struct::User;

/// An account of the database, with the values that a login writes.
fn an_account(username: &str, server_address: &str, starts: bool) -> User {
    User {
        username: username.to_string(),
        server_address: server_address.to_string(),
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

#[test]
fn the_program_holds_two_accounts_and_one_of_them_starts() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("XDG_DATA_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    // Two accounts of two servers. The first login writes the first row, and
    // the account of the newest login starts the program.
    db_insert_usr(&vec![an_account(
        "toutuitest",
        "http://127.0.0.1:13399",
        true,
    )])
    .unwrap();
    db_insert_usr(&vec![an_account(
        "secondtest",
        "http://127.0.0.1:13400",
        true,
    )])
    .unwrap();
    make_this_account_the_default("secondtest").unwrap();

    // **Every account comes to the view.** `select_default_usr` gives one
    // account, and the view of the accounts held that one line before T-124.
    let accounts = select_every_usr().unwrap();
    assert_eq!(
        accounts.len(),
        2,
        "the database holds two accounts, and the view must hold both: {:?}",
        accounts
    );

    // The mark stands on one account, and on one only. Two rows with the mark
    // let the rowid decide which account the program takes.
    let with_the_mark: Vec<&String> = accounts
        .iter()
        .filter(|(_, _, starts)| *starts)
        .map(|(name, _, _)| name)
        .collect();
    assert_eq!(
        with_the_mark,
        vec!["secondtest"],
        "one account starts the program: {:?}",
        accounts
    );

    // The lines of the view name the account and the address, and the mark
    // stands on the account of the start.
    let lines: Vec<String> = accounts
        .iter()
        .map(|(name, address, starts)| {
            toutui::logic::the_accounts::the_line_of_an_account(name, address, *starts)
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "  toutuitest — http://127.0.0.1:13399".to_string(),
            "▶ secondtest — http://127.0.0.1:13400".to_string(),
        ],
        "the lines of the view: {:?}",
        lines
    );

    // The key `c` of the view gives the start to the account of the line.
    let rows = make_this_account_the_default("toutuitest").unwrap();
    assert_eq!(rows, 1, "one row takes the start");

    let starts = select_default_usr().unwrap();
    assert_eq!(
        starts.first().map(String::as_str),
        Some("toutuitest"),
        "the program starts with the account of the key: {:?}",
        starts
    );

    // Every other row lost the mark. The program that reads
    // `WHERE is_default_usr = 1 LIMIT 1` therefore finds one row only.
    let accounts = select_every_usr().unwrap();
    let count = accounts.iter().filter(|(_, _, starts)| *starts).count();
    assert_eq!(count, 1, "one account starts the program: {:?}", accounts);

    // A log out of the account that starts leaves the account that stays, and
    // the rule of the program gives it the start.
    let the_next_account =
        toutui::logic::the_accounts::the_account_after_a_log_out(&accounts, "toutuitest");
    assert_eq!(
        the_next_account,
        toutui::logic::the_accounts::AfterALogOut::ThisAccountStarts("secondtest".to_string())
    );

    remove_the_account("toutuitest").unwrap();
    make_this_account_the_default("secondtest").unwrap();

    let accounts = select_every_usr().unwrap();
    assert_eq!(
        accounts,
        vec![(
            "secondtest".to_string(),
            "http://127.0.0.1:13400".to_string(),
            true
        )],
        "the account that stays holds the start: {:?}",
        accounts
    );
}
