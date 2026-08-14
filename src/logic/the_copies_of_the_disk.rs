//! The copies of the disk of the account, between the disk and the render. See
//! T-204.
//!
//! **A read of the disk that stands inside the render is a read of every
//! frame.** The row of the detail of six views held
//! `get_download_of_a_frame(...)` (T-203), therefore the thread of the screen
//! asked the database at each frame. A second program of the account that holds
//! the write lock then gave that read the busy timeout of rusqlite, and the
//! measurement of T-204 says what that costs: **the program drew one frame in
//! five seconds, five presses of the key `j` moved no cursor for 30 seconds, and
//! the row of the player stood 13 minutes behind the playback.**
//!
//! The box holds the answer of the disk, and the render reads the box. The
//! program reads the disk at the moments that it needs it (the rule of T-142):
//! a new `App` (the start and the key `R`), the end of a download of this
//! program, and the key `X` that takes a copy of the disk away.
//!
//! **A download of a second program of the account reaches this window with the
//! key `R`**, and that is the cost of this decision: the label of a line is a
//! word of the screen, and no word of the screen is worth the thread of the
//! screen. Every road that **removes** a file of the user keeps its own read of
//! the disk at the moment of the use (T-203), because a label that is old
//! destroys nothing and a removal that is old destroys the media of the user.

use log::error;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// The copies of the disk of one account.
#[derive(Debug, Default)]
struct TheCopies {
    /// The key of each media of the account that stands on the disk.
    ///
    /// `None` says that the program did not read the disk: the label of a line
    /// then says that the disk did not answer, and it does not say that the
    /// media holds no copy (T-203).
    keys: Option<HashSet<String>>,
}

fn the_box() -> &'static Mutex<TheCopies> {
    static THE_BOX: OnceLock<Mutex<TheCopies>> = OnceLock::new();
    THE_BOX.get_or_init(|| Mutex::new(TheCopies::default()))
}

/// Reads the copies of the disk of one account, and it keeps them for the
/// render.
///
/// **The box holds one account**, therefore this call takes the place of every
/// key of the account before it: a log out and a second account of the same
/// program then read no copy of the account before them (the rule of T-184 and
/// of T-185).
///
/// A read that failed writes one line of the log and it leaves the box with no
/// key at all. **That is not a media with no copy**: `a_copy_stands_on_the_disk`
/// gives `None` then, and the label of the line says that the disk did not
/// answer.
pub fn read_the_disk(username: &str) {
    let keys = match crate::db::crud::the_keys_of_the_downloads(username) {
        Ok(keys) => Some(keys.into_iter().collect::<HashSet<String>>()),
        Err(why) => {
            error!(
                "[the copies of the disk] the program did not read the downloads of the disk: {}. \
                 The line of a media says that the disk did not answer.",
                why
            );
            None
        }
    };

    if let Ok(mut copies) = the_box().lock() {
        copies.keys = keys;
    }
}

/// Says whether a copy of this media stands on the disk of the account.
///
/// `None` says that the program did not read the disk. The render calls this
/// function at each frame, therefore it reads no disk and it writes no line of
/// the log (T-185).
pub fn a_copy_stands_on_the_disk(key: &str) -> Option<bool> {
    let copies = the_box().lock().ok()?;

    copies.keys.as_ref().map(|keys| keys.contains(key))
}

/// Gives the box the keys of a measurement. The tests of this program use it.
#[cfg(test)]
fn the_keys_of_a_measurement(keys: Option<Vec<&str>>) {
    if let Ok(mut copies) = the_box().lock() {
        copies.keys = keys.map(|keys| keys.into_iter().map(String::from).collect());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The parts of this test stay in one function: the box is a box of the
    /// process, and two test functions of one module fight for it (T-144 and
    /// T-157).
    #[test]
    fn the_box_says_what_the_disk_holds_and_what_it_did_not_say() {
        // The program did not read the disk. The label of a line then says that
        // the disk did not answer, and it does not say that the media holds no
        // copy.
        the_keys_of_a_measurement(None);
        assert_eq!(a_copy_stands_on_the_disk("a-book"), None);
        assert_eq!(
            crate::ui::keys::the_label_of_the_copy_of_the_disk(a_copy_stands_on_the_disk("a-book")),
            crate::ui::keys::THE_DISK_DID_NOT_ANSWER,
        );

        // The disk answered, and it holds one media of the two.
        the_keys_of_a_measurement(Some(vec!["a-book"]));
        assert_eq!(a_copy_stands_on_the_disk("a-book"), Some(true));
        assert_eq!(a_copy_stands_on_the_disk("a-second-book"), Some(false));
        assert_eq!(
            crate::ui::keys::the_label_of_the_copy_of_the_disk(a_copy_stands_on_the_disk("a-book")),
            crate::ui::keys::THE_COPY_OF_THE_DISK,
        );

        // The disk answered, and it holds no media at all. A read of the disk
        // that gave no row is not a read that failed.
        the_keys_of_a_measurement(Some(vec![]));
        assert_eq!(a_copy_stands_on_the_disk("a-book"), Some(false));

        // The box holds one account: the keys of a second account take the
        // place of the keys before them.
        the_keys_of_a_measurement(Some(vec!["the-book-of-the-second-account"]));
        assert_eq!(a_copy_stands_on_the_disk("a-book"), Some(false));
        assert_eq!(
            a_copy_stands_on_the_disk("the-book-of-the-second-account"),
            Some(true)
        );
    }
}
