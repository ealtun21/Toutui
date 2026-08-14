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
//!
//! **The box holds the machine that took the place, and not the fact alone**
//! (T-212). The first form of it said "the server holds the place of that media
//! already" for every row, and `close_one_session` called it for a place that no
//! machine took: the log of a measurement of 2026-08-14 said that sentence one
//! millisecond after the status 500 of the write of that place, and the program
//! then removed the row with no request at all.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::db::crud::delete_the_session_of_a_playback;
use log::error;

/// Where the place of a closed session stands.
///
/// **The row of a session goes away only after the place of the user stands
/// somewhere else** (T-145 and T-212). A caller of
/// `the_row_of_a_closed_session_goes_away` says which machine holds that place,
/// and the words of the log then name it: the first form of this box said "the
/// server holds the place of that media already" for **every** row, and a
/// measurement of 2026-08-14 read that sentence for a place that the server
/// refused with the status 500.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThePlaceOfTheSession {
    /// The server took the place of the user.
    TheServerHoldsIt,

    /// The row of `pending_progress` holds it, and the task of the flush sends
    /// it to the server again. See T-189.
    TheDiskHoldsIt,

    /// The server refuses that place at every attempt: the status 404 of a media
    /// that the server does not hold, and the status 400 of a request that the
    /// server refused. No machine can carry it. See T-189.
    NoServerTakesItEver,
}

impl ThePlaceOfTheSession {
    /// The words of the log for the machine that holds the place.
    fn the_words(self) -> &'static str {
        match self {
            Self::TheServerHoldsIt => "the server holds the place of that media already",
            Self::TheDiskHoldsIt => {
                "the row of the places that wait holds the place of that media already"
            }
            Self::NoServerTakesItEver => "this server takes the place of that media never",
        }
    }
}

/// The sessions of this program whose place stands somewhere else and whose row
/// the disk kept.
fn the_box() -> &'static Mutex<HashMap<String, ThePlaceOfTheSession>> {
    static THE_BOX: OnceLock<Mutex<HashMap<String, ThePlaceOfTheSession>>> = OnceLock::new();

    THE_BOX.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Removes the row of a session whose place stands somewhere else already.
///
/// **The caller reads the answer of this removal** (T-200 and T-207): a removal
/// that the disk refused leaves a row that the program sends again, therefore
/// the box of this module holds the identity of that session and the log names
/// the fault. The removal holds no key of the user and no view of its own
/// (T-177), therefore it takes a line of the log and no word for the user: the
/// place of the user is safe, because `the_place` names the machine that holds
/// it.
/// **A disk that answers again takes the row away.** The condition of this box is
/// the condition of a disk that is full and of a file system that a machine gave
/// back as read-only, and each of them can go away while the program runs:
/// `sync_session_from_database` calls this function again for a session of the
/// box, and the row and the identity of it then go away together.
///
/// **A row whose place no machine holds does not reach this function** (T-212):
/// the caller keeps that row, and the next program of the account sends that
/// place.
pub fn the_row_of_a_closed_session_goes_away(id_session: &str, the_place: ThePlaceOfTheSession) {
    match delete_the_session_of_a_playback(id_session) {
        Ok(()) => {
            if let Ok(mut the_sessions) = the_box().lock() {
                the_sessions.remove(id_session);
            }
        }

        Err(error) => {
            error!(
                "[the row of a closed session] the disk kept the row of the session {}: {}. {}, \
                 therefore this program sends it no second time.",
                id_session,
                error,
                the_place.the_words()
            );

            if let Ok(mut the_sessions) = the_box().lock() {
                the_sessions.insert(id_session.to_string(), the_place);
            }
        }
    }
}

/// Gives the machine that holds the place of that session already, or nothing.
///
/// **A place that this program gave away goes away one time** (T-207), and the
/// answer of this function says which machine took it (T-212).
pub fn the_place_of_this_session_stands_somewhere(
    id_session: &str,
) -> Option<ThePlaceOfTheSession> {
    the_box()
        .lock()
        .ok()
        .and_then(|the_sessions| the_sessions.get(id_session).copied())
}

/// Empties the box. A test of this box needs the box of the test alone.
pub fn the_box_of_the_sessions_goes_empty() {
    if let Ok(mut the_sessions) = the_box().lock() {
        the_sessions.clear();
    }
}
