//! A key of the user pays the wait of the disk one time. See T-208.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME` and it writes the box of the wait, and each of the two
//! belongs to the process: two test functions of one binary fight for them. See
//! T-144 and T-157.
//!
//! # The condition
//!
//! rusqlite waits five seconds for the lock of the file (T-199), and a key of
//! the user holds more than one call of the database. A second program of the
//! account that holds the lock (T-140, and `docs/harness/hold_the_lock.py`)
//! therefore gives the key the whole five seconds for **each** of its calls.
//!
//! # The measurement of 2026-08-14
//!
//! The real program of the sandbox played a book of eight hours with the null
//! device of ALSA, and a second writer took the lock for 40 seconds. The key `l`
//! of the Home view:
//!
//! ```text
//!  15370 ms: Loading the media...
//!  23671 ms:
//!  35812 ms: The program did not keep the session on its disk: database is locked.
//! ```
//!
//! The log held seven calls of the database at five seconds each — the read of
//! the account of the wait, the two writes of that wait, the read of the sessions
//! to close, the read of the files of the download, the write of the row of the
//! session, and the write of the end of the loop. **The user waited 35 seconds
//! for a playback that never started, and the screen said nothing for 20 of
//! them.**
//!
//! The same key with the correction, the same lock, and the same script:
//!
//! ```text
//!   5462 ms: Loading the media...
//!   6473 ms: The program did not keep the session on its disk: database is locked.
//! ```
//!
//! The seven calls stand in 1.4 seconds of the log now, and the first of them
//! holds the whole wait.
//!
//! # The rule
//!
//! **The road of one key pays the wait of the disk one time.** A call that did
//! not reach the disk keeps its moment, and every call inside the time of that
//! fault waits the short time and no more. A call that reached the disk takes the
//! moment away, therefore a lock that goes away gives the whole wait back to the
//! program.

use std::time::{Duration, Instant};
use toutui::db::the_wait_of_the_disk::{
    self, THE_TIME_OF_A_FAULT, THE_WAIT_OF_A_DISK_THAT_ANSWERS,
    THE_WAIT_OF_A_DISK_THAT_DID_NOT_ANSWER,
};

#[test]
fn a_key_pays_the_wait_of_the_disk_one_time() {
    // # The pure function of the wait.
    let now = Instant::now();

    // A program that met no fault waits the whole time of rusqlite.
    assert_eq!(
        the_wait_of_the_disk::the_wait_after(None, now),
        THE_WAIT_OF_A_DISK_THAT_ANSWERS,
        "a disk that answered must give the whole wait"
    );

    // A call that stands after a fault of the disk waits the short time.
    assert_eq!(
        the_wait_of_the_disk::the_wait_after(Some(now), now),
        THE_WAIT_OF_A_DISK_THAT_DID_NOT_ANSWER,
        "a call that follows a fault of the disk must not wait the whole time"
    );

    // The fault of a call holds the calls of that road alone. A fault that is
    // older than the time of a fault gives the whole wait back.
    assert_eq!(
        the_wait_of_the_disk::the_wait_after(
            Some(now - THE_TIME_OF_A_FAULT - Duration::from_millis(1)),
            now
        ),
        THE_WAIT_OF_A_DISK_THAT_ANSWERS,
        "a fault that went by must give the whole wait back"
    );

    // # The answer of an open, and the fault that it keeps.
    //
    // A lock of the file arms the box, and no other fault of the database does:
    // **a database with no permission of a write and a file that holds no
    // database each answer at once** (T-199 and T-206), therefore a shorter wait
    // of the call after them helps nobody.
    the_wait_of_the_disk::the_disk_answered();

    let of_a_lock: Result<(), rusqlite::Error> = Err(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(5),
        Some("database is locked".to_string()),
    ));

    assert!(the_wait_of_the_disk::the_answer_of_an_open(of_a_lock).is_err());
    assert!(
        the_wait_of_the_disk::the_moment_of_the_fault().is_some(),
        "a lock of the file must keep the moment of the fault"
    );

    assert!(the_wait_of_the_disk::the_answer_of_an_open(Ok::<(), rusqlite::Error>(())).is_ok());
    assert_eq!(
        the_wait_of_the_disk::the_moment_of_the_fault(),
        None,
        "an open that answered must take the moment of the fault away"
    );

    let of_no_write: Result<(), rusqlite::Error> = Err(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(8),
        Some("attempt to write a readonly database".to_string()),
    ));

    assert!(the_wait_of_the_disk::the_answer_of_an_open(of_no_write).is_err());
    assert_eq!(
        the_wait_of_the_disk::the_moment_of_the_fault(),
        None,
        "a disk that takes no write is no lock of the file, and it holds no wait"
    );

    // # The database of a second writer.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let of_the_disk = dir.path().join("toutui").join("db.sqlite3");

    // The database of the account stands, and the box holds no fault: an open of
    // this program answers.
    the_wait_of_the_disk::the_disk_answered();
    toutui::db::migrate::open_conn().expect("the database of this test must answer");
    assert_eq!(
        the_wait_of_the_disk::the_moment_of_the_fault(),
        None,
        "an open that answered must take the moment of a fault away"
    );

    // A second program of the account takes the write lock of the file, and it
    // holds it for the whole of this test.
    let of_the_second_program = rusqlite::Connection::open(&of_the_disk).unwrap();
    of_the_second_program
        .execute_batch("BEGIN EXCLUSIVE")
        .unwrap();

    // The first call of the road met that lock and it paid the whole wait. The
    // measurement of the real program above holds those five seconds, and this
    // test writes the moment of that call: **a test that waits for the whole of
    // it holds the gate of the machine for five seconds** (T-158).
    the_wait_of_the_disk::the_disk_did_not_answer();

    // The call after it stands in the same road of the same key. **This is the
    // rule**: it meets the same lock, and it waits the short time alone.
    let start = Instant::now();
    let of_the_second_call = toutui::db::migrate::open_conn();
    let of_the_second_wait = start.elapsed();

    assert!(
        of_the_second_call.is_err(),
        "the lock stands still, therefore the second call fails too"
    );
    assert!(
        of_the_second_wait < THE_WAIT_OF_A_DISK_THAT_ANSWERS / 2,
        "the second call of the road waited {:?}: a key of the user must pay the wait of the \
         disk one time",
        of_the_second_wait
    );

    // The lock goes away, and the disk gives the whole wait back to the program.
    of_the_second_program.execute_batch("ROLLBACK").unwrap();
    toutui::db::migrate::open_conn().expect("a disk with no lock must answer");
    assert_eq!(
        the_wait_of_the_disk::the_moment_of_the_fault(),
        None,
        "a call that reached the disk must take the moment of the fault away"
    );
}
