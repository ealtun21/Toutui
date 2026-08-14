//! The account of this program stands on the disk, and the program reads that
//! disk at the moment of the use. See T-159.
//!
//! **The measurement of 2026-08-14, with the sandbox and two sessions of tmux
//! of one `XDG_CONFIG_HOME`.** The window A stood in the Home view of
//! `toutuitest`, and the window B logged out of that account with the key `l`
//! of the view of the accounts. The key `R` of the window A then made a new
//! application of an account that the disk does not hold:
//!
//! - the header said `👋 Connected as ` — **the program named nobody**;
//! - the log said `Failed to decrypt the token`, and the program went on with
//!   the token of the start, because the client of the start holds it;
//! - the key `S` of the library said `The library has been updated. Please
//!   refresh the app to apply the changes.` for a write of **0 rows**: the user
//!   asked for `Books`, and the header said `📖 Podcasts` after it.
//!
//! **A logout that leaves a program of that account is no logout.** The program
//! reads the accounts of the disk at every key now, and a program whose account
//! stands in no row starts again: the login screen of that program says why,
//! and the place of a playback reaches the server first (T-139). The
//! measurement after the correction: the key `j` at the minute 14:44 of an
//! episode gave `Item … closed at 884s`, and `GET /api/me` of `curl` holds
//! `884`.
//!
//! This test needs no server and no sound card.

use toutui::db::crud::{db_insert_usr, get_others, update_id_selected_lib};
use toutui::db::database_struct::User;

/// The directory of configuration of this test binary. No line of a test may
/// touch the database of the user.
static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();

fn temporary_home() {
    HOME.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        std::env::set_var("XDG_DATA_HOME", dir.path());
        std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();
        drop(conn);

        dir
    });
}

fn an_account(username: &str) -> User {
    User {
        username: username.to_string(),
        server_address: "http://127.0.0.1:1".to_string(),
        token: "the-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Books".to_string(),
        id_selected_lib: "lib-1".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// The write of the library gives the number of the rows that it changed, and
/// it says the sentence of an account that a second program removed when that
/// number is 0.
#[test]
fn the_library_of_an_account_that_holds_no_row_changes_no_row_and_it_says_so() {
    temporary_home();
    db_insert_usr(&vec![an_account("the-account-of-the-library")]).unwrap();

    let rows = update_id_selected_lib("lib-2", "the-account-of-the-library").unwrap();
    assert_eq!(rows, 1, "the account of a row must take the library");
    let of_the_account = toutui::logic::message::for_the_screen().unwrap_or_default();
    assert!(
        of_the_account.contains("library has been updated"),
        "the account of a row must read that its library changed: {}",
        of_the_account
    );

    toutui::logic::message::forget();

    let rows = update_id_selected_lib("lib-2", "an-account-that-a-second-program-removed").unwrap();
    assert_eq!(rows, 0, "a name that no row holds must change no row");

    let of_no_account = toutui::logic::message::for_the_screen().unwrap_or_default();
    assert!(
        !of_no_account.contains("library has been updated"),
        "the program must not say that it kept a choice that no row of the disk holds: {}",
        of_no_account
    );
    assert!(
        of_no_account.contains("an-account-that-a-second-program-removed"),
        "the sentence must name the account: {}",
        of_no_account
    );
}

/// The login screen of the program that starts again says why it comes.
#[test]
fn the_login_screen_takes_the_reason_of_the_account_that_is_gone() {
    temporary_home();

    toutui::db::crud::update_login_err(
        toutui::logic::the_accounts::the_text_of_an_account_that_is_gone(
            "the-account-of-the-login",
        )
        .as_str(),
    )
    .unwrap();

    let of_the_login = get_others()
        .unwrap()
        .map(|others| others.login_err)
        .unwrap_or_default();

    assert!(
        of_the_login.contains("the-account-of-the-login"),
        "the login screen must say which account went away: {}",
        of_the_login
    );
    assert!(
        !of_the_login.contains("Press"),
        "the sentence must promise no key: {}",
        of_the_login
    );
}

/// **No unit test reaches the loop of `src/main.rs`**, therefore the rule of
/// that loop stands here as a rule of the source, as the rule of T-131 does.
///
/// The read of the accounts must stand **after** the key of the user and
/// **before** the block that starts the program again: a key of the view of the
/// accounts writes that block itself, and the read of the disk must not take
/// its place.
#[test]
fn every_key_reads_the_accounts_of_the_disk() {
    let source = include_str!("../src/main.rs");

    let of_the_key = source
        .find("app.handle_key(key);")
        .expect("the loop of main.rs gives every key to the application");
    let of_the_read = source
        .find("select_every_usr()")
        .expect("the loop of main.rs must read the accounts of the disk after a key. See T-159");
    let of_the_start_again = source
        .find("if let Some(request) = app.the_program_starts_again.take()")
        .expect("the loop of main.rs holds the block that starts the program again");

    assert!(
        of_the_key < of_the_read && of_the_read < of_the_start_again,
        "the read of the accounts must stand between the key of the user and the block that \
         starts the program again"
    );
    assert!(
        source.contains("the_account_of_this_program_is_gone()"),
        "the loop must ask for a new program when the account of this program stands in no row \
         of the disk"
    );
}
