//! The word of a place of a playback that the disk did not take. See T-210.
//!
//! **The place of the user of a playback reaches the disk at each second**: the
//! row of the download and, for a playback with no server, the row of
//! `pending_progress` that waits for that server (T-152). A disk that takes no
//! write — a disk that is full, a database with no permission of a write, a file
//! system that a machine gave back as read-only (T-206) — takes every one of
//! those seconds away.
//!
//! **A word that names work that stands must live while that work stands.** A
//! message of the program lives six seconds (`crate::logic::message::LIFE`), and
//! this condition stands for the whole playback: a user who looks at the screen
//! two minutes after the first second of the fault reads nothing. The row of the
//! player stands while the media plays, therefore the word of this condition
//! belongs to that row, beside the word of the engine.
//!
//! This module is a box of the process, in the shape of `crate::logic::message`
//! and of `crate::logic::the_copies_of_the_disk`: the loop of the playback writes
//! it, and the render reads it. **The render reads no disk** (T-204).

use std::sync::atomic::{AtomicBool, Ordering};

/// The words of the row of the player while the disk takes no place.
///
/// **The words name the thing that failed** (T-91 and T-199): the disk of this
/// machine, and not the server. The row of the player is short, therefore this
/// text is short too, and the log holds the fault of the database itself.
pub const THE_DISK_TAKES_NO_PLACE: &str = "The disk keeps no place of this media";

/// The one value of the process.
static THE_DISK_TOOK_NO_PLACE: AtomicBool = AtomicBool::new(false);

/// The loop of the playback says what the disk did with the place of a second.
///
/// `true` says that a write of this second failed. A write that the disk took
/// again gives `false`, and the word goes away: the place of the user reaches
/// the disk again.
pub fn the_disk_says(it_took_no_place: bool) {
    THE_DISK_TOOK_NO_PLACE.store(it_took_no_place, Ordering::Relaxed);
}

/// Tells if the disk takes no place now.
pub fn the_disk_takes_no_place() -> bool {
    THE_DISK_TOOK_NO_PLACE.load(Ordering::Relaxed)
}

/// The word of the row of the player, of the engine and of the disk together.
///
/// The function is pure, therefore a test needs no box of the process.
pub fn the_notice(of_the_engine: Option<String>, the_disk_takes_no_place: bool) -> Option<String> {
    match (of_the_engine, the_disk_takes_no_place) {
        (Some(of_the_engine), false) => Some(of_the_engine),
        (Some(of_the_engine), true) => {
            Some(format!("{} | {}", of_the_engine, THE_DISK_TAKES_NO_PLACE))
        }
        (None, true) => Some(THE_DISK_TAKES_NO_PLACE.to_string()),
        (None, false) => None,
    }
}

/// The word of the row of the player. The render calls this one.
pub fn the_notice_of_the_player(of_the_engine: Option<String>) -> Option<String> {
    the_notice(of_the_engine, the_disk_takes_no_place())
}
