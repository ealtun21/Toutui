//! The login against a real server. This test needs the sandbox server.
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test login_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The test writes in a temporary directory only. It changes nothing on the
//! server, and it changes nothing in the configuration of the user.

use toutui::api::server::auth_process::auth_process;

/// The sandbox server of `docs/TEST-SERVER.md`.
const SERVER: &str = "http://127.0.0.1:13399";
const USER: &str = "toutuitest";
const PASSWORD: &str = "toutuitest";

/// The first login writes the user, and the program can read it with no wait.
///
/// This is the test of T-15. That report says that a login with correct
/// credentials fails, and that the second attempt works. One mechanism
/// explains such a report: the program reads the database before the login
/// writes the user. The comment of `main.rs` named that mechanism, and the
/// program waited one second for it.
///
/// The test reads the database with no wait at all. If the race exists, the
/// list of the users is empty here.
#[tokio::test]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn the_first_login_writes_the_user_with_no_wait() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("TOUTUI_SECRET_KEY", "a-key-for-the-test-of-t15");

    // The program makes this directory at its start, therefore the test makes
    // it too.
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    let before = toutui::db::crud::select_default_usr().unwrap_or_default();
    assert!(before.is_empty(), "the database must hold no user at the start");

    auth_process(USER, PASSWORD, SERVER)
        .await
        .expect("a login with correct credentials must succeed");

    // No sleep. The race of T-15 would show itself here.
    let after = toutui::db::crud::select_default_usr().unwrap_or_default();

    assert_eq!(after.first().map(String::as_str), Some(USER));
    assert_eq!(after.get(1).map(String::as_str), Some(SERVER));
}
