//! A write of the disk that no caller reads is a fact of the user. See T-207.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME` and it reads two boxes of the process, and each of them
//! belongs to the process: two test functions of one binary fight for them. See
//! T-144 and T-157.
//!
//! # The condition
//!
//! The harness of T-206 gives it, and it is one command: `chmod 444` of the file
//! of the database. SQLite opens such a file for a read, every `SELECT` of the
//! program answers, and every write gives `attempt to write a readonly database`.
//! A disk that is full, a database with no permission of a write, and a file
//! system that a machine gave back as read-only each give that condition of the
//! user.
//!
//! # The measurement of 2026-08-14
//!
//! The program of the sandbox played "A Book Of Many Hours" (eight hours), and
//! `chmod 444` came after the first frame. The key `l` of a second media closed
//! the session of the book: the server took the place 646 seconds, and
//! `let _ = delete_the_session_of_a_playback(...)` **left the row**. A second
//! client of the account then wrote 6000 seconds, and the next key `l` of the
//! program read that same row again and sent 646 seconds over it:
//!
//! ```text
//! the server holds 6000
//! [handle_key (l)][book] Item 6ba57b9a-… closed at 646s (not finished)
//! the server holds 646
//! ```
//!
//! **The book of the user lost 89 minutes**, and no word of the screen and no
//! line of the log named it.
//!
//! The same run held the second fault. `let _ = update_is_loop_break("1", …)` of
//! the end of the loop of the playback failed in the same way, therefore the row
//! of the disk stayed at `0` and the key `l` after it waited the whole
//! `THE_LONGEST_WAIT` of 30 seconds:
//!
//! ```text
//! 18:14:22 [wait_prev_session_finished][is_loop_break] Some("0")
//! 18:14:52 [wait_prev_session_finished] the playback before this one wrote no end in 30 s.
//! ```
//!
//! # The rule
//!
//! **A place that this program gave to the server goes to that server no second
//! time**, and **the wait of a playback is a question about the program that
//! waits**. The disk cannot hold either fact while it takes no write, therefore a
//! box of the process holds each of them: a program that stops takes both with
//! it, and the row of the disk is then the row of a program that died, which the
//! rules of T-140 and T-145 hold already.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use toutui::logic::sync_session::the_rows_that_the_disk_kept::{
    the_box_of_the_sessions_goes_empty, the_row_of_a_closed_session_goes_away,
    the_server_holds_this_session_already,
};
use toutui::logic::sync_session::wait_prev_session_finished::{
    a_loop_of_this_program_wrote_its_end, the_loop_of_this_program_ended,
    the_mark_of_the_loop_goes_away, the_wait_of_a_playback,
};

/// The longest wait of this test. The wait of the program is 30 seconds, and a
/// test of 30 seconds holds a session of continuous integration for nothing.
const THE_LONGEST_WAIT: Duration = Duration::from_secs(2);

/// Gives the wait a thread of its own, and it reads the end of that thread with a
/// limit of time.
///
/// **A test must not call a function that may never come back.** The wait blocks
/// the thread that calls it, therefore a limit of time on the call itself says
/// nothing. See T-158 and T-167.
fn the_time_of_the_wait(username: &str) -> Duration {
    let (to_the_test, of_the_wait) = mpsc::channel();
    let of_the_account = username.to_string();

    std::thread::spawn(move || {
        let start = Instant::now();
        the_wait_of_a_playback(of_the_account, THE_LONGEST_WAIT);
        let _ = to_the_test.send(start.elapsed());
    });

    of_the_wait
        .recv_timeout(THE_LONGEST_WAIT + Duration::from_secs(20))
        .expect("the wait of the playback must come back")
}

#[test]
fn a_write_that_the_disk_did_not_take_is_no_fact_of_the_user() {
    const USER: &str = "toutuitest";
    const SERVER: &str = "http://127.0.0.1:13399";
    const THE_SESSION: &str = "the-session-of-the-book-of-many-hours";

    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let of_the_disk = dir.path().join("toutui").join("db.sqlite3");

    {
        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();

        // The account of this program. The two marks say "a loop of a playback
        // stands before this one", therefore the wait of the test waits.
        conn.execute(
            "INSERT INTO users (username, server_address, token, is_default_usr, \
             name_selected_lib, id_selected_lib, is_loop_break, has_played_before, speed_rate, \
             is_show_key_bindings) VALUES (?1, ?2, '', 1, '', '', '0', '0', 1.0, '0')",
            rusqlite::params![USER, SERVER],
        )
        .unwrap();

        // The row of the session of the book of eight hours, at 646 seconds.
        conn.execute(
            "INSERT INTO listening_session (id_session, id_item, current_time_playback, \
             duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter, \
             username, server, owner, heartbeat) \
             VALUES (?1, 'the-book-of-many-hours', 646, '28800', 0, '', 646, \
             'A Book Of Many Hours', 'Many Hours Author', 1, '', ?2, ?3, 'a-program', 0)",
            rusqlite::params![THE_SESSION, USER, SERVER],
        )
        .unwrap();
    }

    the_box_of_the_sessions_goes_empty();
    the_mark_of_the_loop_goes_away();

    // **The disk reads, and it takes no write.** Every assertion after this line
    // holds the write alone.
    let mut how = std::fs::metadata(&of_the_disk).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut how, 0o444);
    std::fs::set_permissions(&of_the_disk, how).unwrap();

    assert_eq!(
        toutui::db::crud::get_the_sessions_to_close(USER, SERVER)
            .expect("the program must read a database that takes no write")
            .len(),
        1,
        "the read of the sessions must answer with the row of the disk"
    );

    // **The place of the user goes to the server no second time** (T-207). The
    // server holds the place of this session already, and the removal of its row
    // fails: the old shape was `let _ = delete_the_session_of_a_playback(...)`,
    // therefore the program read that row again and it sent 646 seconds over the
    // 6000 seconds of a second client of the account.
    the_row_of_a_closed_session_goes_away(THE_SESSION);

    assert_eq!(
        toutui::db::crud::get_the_sessions_to_close(USER, SERVER)
            .unwrap()
            .len(),
        1,
        "the disk that takes no write must keep the row of the session"
    );

    assert!(
        the_server_holds_this_session_already(THE_SESSION),
        "a session whose place the server took, and whose row the disk kept, must go to that \
         server no second time"
    );

    // A session of another program of the account stands in no box of this
    // program: the rule of T-140 and of T-145 holds that row.
    assert!(
        !the_server_holds_this_session_already("the-session-of-a-program-that-died"),
        "the box must hold the sessions of this program alone"
    );

    // **The wait of a playback is a question about the program that waits**
    // (T-207). The two marks of the disk say `0`, therefore the wait waits, and
    // the disk takes no write of the end of the loop.
    assert!(
        !a_loop_of_this_program_wrote_its_end(),
        "no loop of this program wrote its end before this line"
    );

    let of_the_disk_alone = the_time_of_the_wait(USER);

    assert!(
        of_the_disk_alone >= THE_LONGEST_WAIT,
        "the row of the disk that says `0` must hold the wait for the whole limit of time, and \
         the wait took {:?}",
        of_the_disk_alone
    );

    // The loop of the playback of this program came to its end, and the disk took
    // no word of it. **The playback after it must not wait.**
    the_loop_of_this_program_ended();

    let of_the_mark = the_time_of_the_wait(USER);

    assert!(
        of_the_mark < Duration::from_secs(1),
        "the mark of the loop of this program must end the wait at once, and the wait took {:?}",
        of_the_mark
    );

    // The wait that came back takes the mark away: the playback that begins holds
    // the loop after it.
    assert!(
        !a_loop_of_this_program_wrote_its_end(),
        "the wait that came back must take the mark of the loop before it away"
    );

    let mut how = std::fs::metadata(&of_the_disk).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut how, 0o644);
    std::fs::set_permissions(&of_the_disk, how).unwrap();
}
