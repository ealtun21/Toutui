//! The time that one call of the database waits for a disk that a second
//! program of the account holds. See T-208.
//!
//! **A key of the user paid the wait of the disk one time for each call of its
//! road.** rusqlite waits five seconds for the lock of the file (T-199), and a
//! key of the program holds more than one call: a measurement of 2026-08-14 with
//! `docs/harness/hold_the_lock.py` and the key `l` of the Home view gave **35
//! seconds** for a playback that did not start, and the log held seven calls of
//! the database at five seconds each — the read of the account of the wait, the
//! two writes of that wait, the read of the sessions to close, the read of the
//! files of the download, the write of the row of the session, and the write of
//! the end of the loop. **Each of them met the same lock, and each of them paid
//! the whole five seconds again.**
//!
//! This box holds the moment at which a call did not reach the disk. A call that
//! stands inside `THE_TIME_OF_A_FAULT` after that moment waits
//! `THE_WAIT_OF_A_DISK_THAT_DID_NOT_ANSWER` and no longer, therefore the road of
//! one key pays the whole wait one time.
//!
//! **The box arms after a wait of five seconds and not before it.** Two programs
//! of one account (T-140) write the database in some milliseconds, and a call
//! that meets such a write answers inside its own wait: no fault reaches this
//! box at all. A lock that stands longer than the whole wait is the condition of
//! a second program that stopped inside a write, of a disk that does not answer,
//! and of the harness of T-199 — and there the program must say why, and it must
//! say it at once.
//!
//! **A call inside the window can therefore fail where the old code waited and
//! answered**: a lock that goes away between the short wait and the whole one
//! gives a fault now. That is the decision of T-208, and the fork holds the
//! words of every such fault already (T-199 to T-207): a key of the user says
//! why, and a work with no key of the user takes a line of the log.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The time that a call waits for the lock of the file. It is the value of
/// rusqlite, and T-199 measured it.
pub const THE_WAIT_OF_A_DISK_THAT_ANSWERS: Duration = Duration::from_secs(5);

/// The time that a call waits while the fault of a call before it stands.
///
/// A disk of this machine answers in some hundreds of microseconds (T-204),
/// therefore 200 milliseconds is the wait of a lock that goes away and not the
/// wait of a disk that is slow.
pub const THE_WAIT_OF_A_DISK_THAT_DID_NOT_ANSWER: Duration = Duration::from_millis(200);

/// The time in which the fault of one call stands for the calls after it.
pub const THE_TIME_OF_A_FAULT: Duration = THE_WAIT_OF_A_DISK_THAT_ANSWERS;

/// The moment at which a call did not reach the disk. A program that stops takes
/// it with it, and no other program of the account reads it.
static THE_MOMENT_OF_THE_FAULT: Mutex<Option<Instant>> = Mutex::new(None);

/// Gives the box, and a box that a thread of a panic left gives its value too.
fn the_box() -> std::sync::MutexGuard<'static, Option<Instant>> {
    THE_MOMENT_OF_THE_FAULT
        .lock()
        .unwrap_or_else(|of_a_panic| of_a_panic.into_inner())
}

/// A call did not reach the disk of this account.
pub fn the_disk_did_not_answer() {
    *the_box() = Some(Instant::now());
}

/// A call reached the disk. The calls after it wait the whole time again.
pub fn the_disk_answered() {
    *the_box() = None;
}

/// The moment of the fault that stands, and nothing when no fault stands.
pub fn the_moment_of_the_fault() -> Option<Instant> {
    *the_box()
}

/// The time that the next call of the database waits for the lock of the file.
pub fn the_wait_of_the_next_call() -> Duration {
    the_wait_after(the_moment_of_the_fault(), Instant::now())
}

/// The work of `the_wait_of_the_next_call`, with the moment of the fault and the
/// moment of now. The function is pure, therefore a test needs no database.
pub fn the_wait_after(of_the_fault: Option<Instant>, now: Instant) -> Duration {
    match of_the_fault {
        Some(moment) if now.duration_since(moment) < THE_TIME_OF_A_FAULT => {
            THE_WAIT_OF_A_DISK_THAT_DID_NOT_ANSWER
        }
        _ => THE_WAIT_OF_A_DISK_THAT_ANSWERS,
    }
}

/// Says that a fault of rusqlite is a disk that did not answer.
///
/// **A database with no permission of a write and a file that holds no database
/// are other conditions** (T-199 and T-206): each of them answers at once, and a
/// wait of the next call helps nobody. The two codes below are the lock of the
/// file alone.
pub fn the_fault_is_a_lock_of_the_file(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(of_sqlite, _)
            if matches!(
                of_sqlite.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            )
    )
}

/// Reads the answer of one open of the database, and it keeps the moment of a
/// lock that stood.
pub fn the_answer_of_an_open<T>(answer: rusqlite::Result<T>) -> rusqlite::Result<T> {
    match answer {
        Ok(value) => {
            the_disk_answered();
            Ok(value)
        }
        Err(error) => {
            if the_fault_is_a_lock_of_the_file(&error) {
                the_disk_did_not_answer();
            }

            Err(error)
        }
    }
}
