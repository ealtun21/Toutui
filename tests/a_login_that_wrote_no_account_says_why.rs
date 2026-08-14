//! A login that writes no row is a login that failed. See T-199.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME` and `TOUTUI_SECRET_KEY`, and those variables belong to the
//! process: two test functions of one binary fight for them. See T-144 and
//! T-157.
//!
//! The test needs the sandbox server of `docs/TEST-SERVER.md`, because the login
//! must reach a server before the program writes the row:
//!
//! ```text
//! cargo test --test a_login_that_wrote_no_account_says_why \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The measurement of T-199: a second program of one account held the database
//! while the login of this program wrote the row. `db_insert_usr` came back with
//! `database is locked`, the old code read that fault with `let _ = ...`, and the
//! log said `Login successful`. The program then drew the login screen of a first
//! start, and the row of the message held no character: the user wrote the
//! address, the name, and the password again, for ever.
//!
//! The condition of this test is a file that holds no database. It gives the
//! same fault of the write with no wait at all: a lock of a second program needs
//! the busy timeout of five seconds of rusqlite first.

use toutui::api::server::auth_process::{
    auth_process, THE_SENTENCE_OF_A_LOGIN_THAT_KEPT_NO_ACCOUNT,
};

const SERVER: &str = "http://127.0.0.1:13399";
const USER: &str = "toutuitest";
const PASSWORD: &str = "toutuitest";

#[tokio::test]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399"]
async fn a_login_that_wrote_no_row_of_the_account_says_why() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("TOUTUI_SECRET_KEY", "a-key-for-the-test-of-t199");

    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    // A file of bytes that hold no database. Every request of the login comes
    // through, and the writes of the row alone fail.
    //
    // **The login holds two writes**, and the rule covers the two of them: the
    // row of the account (`db_insert_usr`) and the mark of the account that
    // starts the program (`make_this_account_the_default`, T-124). A build with
    // one of the two corrections removed still gives this sentence, therefore
    // the build of the fault of this test removes both of them (the trap 147).
    let db = dir.path().join("toutui").join("db.sqlite3");
    std::fs::write(&db, b"this file holds no database at all").unwrap();

    let report = auth_process(USER, PASSWORD, SERVER)
        .await
        .expect_err("a login that wrote no row of the account must fail");

    assert_eq!(
        report.to_string(),
        THE_SENTENCE_OF_A_LOGIN_THAT_KEPT_NO_ACCOUNT,
        "the login must say that the program kept no account"
    );

    // The sentence stands in the row of the message of the login screen, and
    // that row holds one line. See the trap 11 of the harness.
    assert!(
        !THE_SENTENCE_OF_A_LOGIN_THAT_KEPT_NO_ACCOUNT.contains('\n'),
        "the sentence of the login holds one line"
    );
}
