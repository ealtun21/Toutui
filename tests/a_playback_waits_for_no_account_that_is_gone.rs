//! A playback of an account that stands in no row of the disk must not wait,
//! and no wait of a playback lasts for ever. See T-158.
//!
//! **The measurement of 2026-08-14, with the sandbox and two sessions of tmux
//! of one `XDG_CONFIG_HOME`.** The window A stood in the Home view of
//! `toutuitest`. The window B logged out of that account with the key `l` of
//! the view of the accounts, therefore the row of `users` went away. The key
//! `l` of the window A on a book then gave the message "Syncing your last
//! listening session. Please wait..." and **the media never played**: the
//! message stood on the screen 78 seconds later, and no line of the log came
//! after `[wait_prev_session_finished][is_loop_break] No db found`.
//!
//! The two reads of the wait gave the text "No db found" for a row that does
//! not exist, and that text is not `1`. **No loop of a playback of a row that
//! does not exist can ever write `is_loop_break`**, therefore the wait had no
//! end. Every press of the key `l` after it took a worker of the runtime with a
//! `std::thread::sleep`: 40 presses took every worker, and the program then
//! answered no key at all — the key `j` moved no line, and **the key `Q` did
//! not stop the program**.
//!
//! This test needs no server and no sound card. It writes `XDG_CONFIG_HOME`.

use std::time::{Duration, Instant};
use toutui::db::crud::{db_insert_usr, get_has_played_before, get_is_loop_break};
use toutui::db::database_struct::User;
use toutui::logic::sync_session::wait_prev_session_finished::the_wait_of_a_playback;

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

/// Writes an account that played a media before, and whose loop of the
/// playback wrote no end.
fn an_account_of_a_playback_that_wrote_no_end(username: &str) {
    let user = User {
        username: username.to_string(),
        server_address: "http://127.0.0.1:1".to_string(),
        token: "the-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Books".to_string(),
        id_selected_lib: "lib-1".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "0".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    };

    db_insert_usr(&vec![user]).unwrap();
}

/// Gives the wait to a thread of its own, and it says how long that thread
/// took. `None` says that the wait stood after the limit of the test: **a test
/// that calls the wait itself never comes back**, and the fault of T-158 is a
/// wait with no end.
fn the_time_of_the_wait(
    username: &str,
    longest_wait: Duration,
    the_limit_of_the_test: Duration,
) -> Option<Duration> {
    let name = username.to_string();
    let start = Instant::now();
    let the_wait_came_to_its_end = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let of_the_thread = std::sync::Arc::clone(&the_wait_came_to_its_end);

    std::thread::spawn(move || {
        the_wait_of_a_playback(name, longest_wait);
        of_the_thread.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    while start.elapsed() < the_limit_of_the_test {
        if the_wait_came_to_its_end.load(std::sync::atomic::Ordering::SeqCst) {
            return Some(start.elapsed());
        }

        std::thread::sleep(Duration::from_millis(20));
    }

    None
}

/// The two reads of the wait say that the account holds no row, and they say it
/// with `None`. A text of a fault stands for a value of the database, and the
/// caller of that text cannot tell the two conditions apart.
#[test]
fn the_reads_of_the_wait_say_that_an_account_holds_no_row() {
    temporary_home();

    assert_eq!(
        get_is_loop_break("an-account-of-no-row").expect("the database answered"),
        None,
        "an account of no row must give no value of is_loop_break"
    );
    assert_eq!(
        get_has_played_before("an-account-of-no-row").expect("the database answered"),
        None,
        "an account of no row must give no value of has_played_before"
    );
}

/// A second program of the account logged out, therefore this account stands in
/// no row. The playback must start.
#[test]
fn a_playback_of_an_account_that_is_gone_does_not_wait() {
    temporary_home();

    let waited = the_time_of_the_wait(
        "the-account-that-a-second-program-removed",
        Duration::from_secs(60),
        Duration::from_secs(5),
    );

    assert!(
        matches!(waited, Some(time) if time < Duration::from_secs(2)),
        "a playback of an account of no row must start at once: {:?}. The old code waited for \
         ever.",
        waited
    );
}

/// A program that dies inside the loop of its playback writes `is_loop_break`
/// never. The wait of the playback after it must hold a limit of time.
#[test]
fn the_wait_of_a_playback_holds_a_limit_of_time() {
    temporary_home();

    let username = "the-account-of-a-loop-with-no-end";
    an_account_of_a_playback_that_wrote_no_end(username);

    let waited = the_time_of_the_wait(username, Duration::from_secs(2), Duration::from_secs(10));

    assert!(
        matches!(waited, Some(time) if time >= Duration::from_secs(2)),
        "the wait must hold the playback while the limit stands, and it must then end: {:?}",
        waited
    );
}
