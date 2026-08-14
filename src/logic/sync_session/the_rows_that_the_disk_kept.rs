//! The rows of a session that the disk did not remove. See T-207.
//!
//! **A row of `listening_session` and the place of the server hold one thing
//! together.** `close_one_session` sends the place of the user to the server and
//! it then removes the row, because the rule of T-4 says that a place of a
//! program that died goes to the server one time. The removal was `let _ =`, and
//! a disk that takes no write therefore left a row that the server holds
//! already.
//!
//! A measurement of 2026-08-14 with the harness of T-206
//! (`chmod 444` of the file of the database) took the place of the user
//! **backward**: the program closed a book of eight hours at 646 seconds, the
//! row stayed, a second client of the account then wrote 6000 seconds, and the
//! next key `l` of the program sent 646 seconds over it. The book of the user
//! lost 89 minutes, and no line of the log and no word of the screen named it.
//!
//! **A place that this program gave to the server goes to that server no second
//! time.** The disk cannot hold that fact, because the disk takes no write;
//! therefore the box of this module holds it, and `sync_session_from_database`
//! reads it. A program that stops takes the box with it, and the row of the disk
//! is then the row of a program that died: the rule of T-140 and of T-145 holds
//! it again, and that rule is correct for a program that this one did not see.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use crate::db::crud::delete_the_session_of_a_playback;
use log::error;

/// The sessions of this program whose place the server holds and whose row the
/// disk kept.
fn the_box() -> &'static Mutex<HashSet<String>> {
    static THE_BOX: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

    THE_BOX.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Removes the row of a session whose place the server holds already.
///
/// **The caller reads the answer of this removal** (T-200 and T-207): a removal
/// that the disk refused leaves a row that the program sends again, therefore
/// the box of this module holds the identity of that session and the log names
/// the fault. The removal holds no key of the user and no view of its own
/// (T-177), therefore it takes a line of the log and no word for the user: the
/// place of the user is safe, because the server took it before this call.
/// **A disk that answers again takes the row away.** The condition of this box is
/// the condition of a disk that is full and of a file system that a machine gave
/// back as read-only, and each of them can go away while the program runs:
/// `sync_session_from_database` calls this function again for a session of the
/// box, and the row and the identity of it then go away together.
pub fn the_row_of_a_closed_session_goes_away(id_session: &str) {
    match delete_the_session_of_a_playback(id_session) {
        Ok(()) => {
            if let Ok(mut the_sessions) = the_box().lock() {
                the_sessions.remove(id_session);
            }
        }

        Err(error) => {
            error!(
                "[the row of a closed session] the disk kept the row of the session {}: {}. The \
                 server holds the place of that media already, therefore this program sends it no \
                 second time.",
                id_session, error
            );

            if let Ok(mut the_sessions) = the_box().lock() {
                the_sessions.insert(id_session.to_string());
            }
        }
    }
}

/// Says that this program gave the place of that session to the server already.
pub fn the_server_holds_this_session_already(id_session: &str) -> bool {
    the_box()
        .lock()
        .map(|the_sessions| the_sessions.contains(id_session))
        .unwrap_or(false)
}

/// Empties the box. A test of this box needs the box of the test alone.
pub fn the_box_of_the_sessions_goes_empty() {
    if let Ok(mut the_sessions) = the_box().lock() {
        the_sessions.clear();
    }
}
