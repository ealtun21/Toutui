//! The command that forces the sync. See T-32 and upstream issue #37.
//!
//! A user wants to give the position to the server at once, for example
//! before they take a different device. The application sends the position
//! every `SYNC_PERIOD` seconds during a playback, therefore the user waits.
//!
//! **The key is `F`.** The design named the key `S`, and `S` was not free: it
//! opens the settings. `F` reads as "force the sync".
//!
//! # Why a flag, and not a request of the key
//!
//! The endpoint `POST /api/session/:id/sync` takes the time that the user
//! listened since the last sync. Two senders would give that time two times,
//! and the server would then hold too much listened time.
//!
//! Therefore the key writes a flag only, and the loop of the playback does the
//! work at its next second. That loop holds the bookkeeping of the position
//! and of the listened time.
//!
//! The flag carries the identity of the playback. A loop takes the flag only
//! when the identity is its own. Two playbacks can run at the same time, and a
//! loop must never read a value of a different media. See `9bacac` in
//! `known_bugs.md`.
//!
//! The command does not close the session. `GET /api/sessions/open` still
//! holds the session after the key.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// The identity of the playback that must send its position now. The value 0
/// means that no user asked for it.
static REQUEST: AtomicU64 = AtomicU64::new(0);

/// What the loop did with the last request. The key shows this text to the
/// user.
fn report_box() -> &'static Mutex<Option<String>> {
    static REPORT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    REPORT.get_or_init(|| Mutex::new(None))
}

/// The user asks the playback to send its position now.
///
/// The function gives `false` when nothing plays. The caller then tells the
/// user that there is nothing to send.
pub fn ask(playback_id: u64) -> bool {
    if playback_id == 0 {
        return false;
    }

    // An answer of an older request has no value now.
    if let Ok(mut report) = report_box().lock() {
        *report = None;
    }

    REQUEST.store(playback_id, Ordering::SeqCst);
    true
}

/// The loop of the playback asks for the flag of its own playback.
///
/// The function gives `true` one time for each request, and it then takes the
/// flag away. A loop of a different playback gives `false` and leaves the flag
/// for its owner.
pub fn take_request(playback_id: u64) -> bool {
    // The value 0 means "no request". A loop with the identity 0 must not
    // find a request in that value.
    if playback_id == 0 {
        return false;
    }

    REQUEST
        .compare_exchange(playback_id, 0, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}

/// The loop tells the user what the server did.
pub fn report(text: String) {
    if let Ok(mut report) = report_box().lock() {
        *report = Some(text);
    }
}

/// Makes the text that the user reads after a forced sync.
pub fn message<E: std::fmt::Display>(outcome: &Result<(), E>, position: u32) -> String {
    match outcome {
        Ok(()) => format!(
            "Sync: the server has the position {}.",
            crate::utils::convert_seconds::convert_seconds(vec![f64::from(position)])
                .first()
                .cloned()
                .unwrap_or_default()
        ),
        Err(error) => format!("Sync: the server did not take the position. {}", error),
    }
}

/// Gives the answer of the last request, one time.
pub fn take_report() -> Option<String> {
    report_box().lock().ok()?.take()
}

/// Forgets a request and an answer. A test starts from a known condition.
#[cfg(test)]
fn clear() {
    REQUEST.store(0, Ordering::SeqCst);
    let _ = take_report();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tests of one file run at the same time on many threads, and this
    /// module holds one value for the whole process. This lock gives each
    /// test the module for itself.
    fn guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }

    #[test]
    fn a_playback_takes_its_own_request() {
        let _guard = guard();
        clear();

        assert!(ask(7));
        assert!(take_request(7));
    }

    #[test]
    fn the_request_comes_one_time_only() {
        let _guard = guard();
        clear();

        assert!(ask(7));
        assert!(take_request(7));
        assert!(!take_request(7), "the loop must not sync a second time");
    }

    #[test]
    fn a_different_playback_does_not_take_the_request() {
        let _guard = guard();
        clear();

        assert!(ask(7));
        assert!(
            !take_request(8),
            "the loop of a different media took the flag"
        );
        assert!(take_request(7), "the owner lost its flag");
    }

    #[test]
    fn a_loop_with_no_request_gives_nothing() {
        let _guard = guard();
        clear();

        assert!(!take_request(7));
    }

    #[test]
    fn nothing_plays_gives_no_request() {
        let _guard = guard();
        clear();

        assert!(!ask(0), "the identity 0 means that nothing plays");
        assert!(!take_request(0));
    }

    #[test]
    fn the_answer_comes_one_time_only() {
        let _guard = guard();
        clear();

        report("The server has the position.".to_string());
        assert_eq!(
            take_report(),
            Some("The server has the position.".to_string())
        );
        assert_eq!(take_report(), None);
    }

    #[test]
    fn a_new_request_forgets_the_old_answer() {
        let _guard = guard();
        clear();

        report("An old answer.".to_string());
        assert!(ask(7));
        assert_eq!(take_report(), None);
        assert!(take_request(7));
    }
}
