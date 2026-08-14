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
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

/// What the disk holds of one media. See T-217.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheCopyOfTheDisk {
    /// The program did not read the disk. The label of a line says that, and it
    /// does not say that the media holds no copy (T-203).
    TheDiskDidNotAnswer,

    /// The disk answered, and it holds no copy of this media.
    NoCopy,

    /// The disk holds every file of this media, with the bytes of every row.
    AWholeCopy,

    /// The disk holds the rows of this download and not every file of it
    /// (T-215 and T-216). The playback of that media takes the road of the
    /// server, therefore the label must not say `[Downloaded]`.
    ACopyThatIsNotWhole,
}

/// The copies of the disk of one account.
#[derive(Debug, Default)]
struct TheCopies {
    /// The key of each media of the account that stands whole on the disk.
    ///
    /// `None` says that the program did not read the disk.
    keys: Option<HashSet<String>>,

    /// The key of each media of the account whose copy of the disk is not whole.
    not_whole: HashSet<String>,
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
/// key at all. **That is not a media with no copy**: `the_copy_of_this_media`
/// says that the disk did not answer then, and the label of the line says it.
///
/// **The box holds the file system too** (T-217). The rows of `downloads` say
/// that a media stands on the disk, and a row is no file (T-215 and T-216): this
/// function asks the disk for every file of every download of the account, and
/// the label of a copy that is not whole says what the disk holds.
pub fn read_the_disk(username: &str) {
    let (keys, not_whole) = match the_copies_of_the_account(username) {
        Ok((keys, not_whole)) => (Some(keys), not_whole),
        Err(why) => {
            error!(
                "[the copies of the disk] the program did not read the downloads of the disk: {}. \
                 The line of a media says that the disk did not answer.",
                why
            );
            (None, HashSet::new())
        }
    };

    if let Ok(mut copies) = the_box().lock() {
        copies.keys = keys;
        copies.not_whole = not_whole;
    }
}

/// Reads the rows of the downloads of one account, and it asks the file system
/// for every file of them. See T-217.
///
/// The first set holds the key of each download, and the second one holds the key
/// of each download whose files do not all stand on the disk. **This function
/// stands outside the render** (T-204): the program calls it at the start, at the
/// key `R`, at the end of a download, and at the key `X`.
fn the_copies_of_the_account(
    username: &str,
) -> rusqlite::Result<(HashSet<String>, HashSet<String>)> {
    let keys: HashSet<String> = crate::db::crud::the_keys_of_the_downloads(username)?
        .into_iter()
        .collect();

    // The value says whether every file of that download stands on the disk.
    let mut of_the_disk: HashMap<String, bool> = HashMap::new();

    for (key, index, path, size) in crate::db::crud::the_files_of_the_downloads(username)? {
        if !keys.contains(&key) {
            continue;
        }

        let stands = crate::logic::offline::the_file_stands_on_the_disk(index, &path, size);
        let whole = of_the_disk.entry(key).or_insert(true);
        *whole &= stands;
    }

    // **A download with no row of a file is a download of one half** (T-214), and
    // the media of it plays no second from the disk: that copy is not whole.
    let not_whole = keys
        .iter()
        .filter(|key| of_the_disk.get(*key).copied() != Some(true))
        .cloned()
        .collect();

    Ok((keys, not_whole))
}

/// Says what the disk holds of this media.
///
/// The render calls this function at each frame, therefore it reads no disk and
/// it writes no line of the log (T-185).
pub fn the_copy_of_this_media(key: &str) -> TheCopyOfTheDisk {
    let Ok(copies) = the_box().lock() else {
        return TheCopyOfTheDisk::TheDiskDidNotAnswer;
    };

    let Some(keys) = copies.keys.as_ref() else {
        return TheCopyOfTheDisk::TheDiskDidNotAnswer;
    };

    if !keys.contains(key) {
        return TheCopyOfTheDisk::NoCopy;
    }

    if copies.not_whole.contains(key) {
        return TheCopyOfTheDisk::ACopyThatIsNotWhole;
    }

    TheCopyOfTheDisk::AWholeCopy
}

/// Gives the box the keys of a measurement. The tests of this program use it.
#[cfg(test)]
fn the_keys_of_a_measurement(keys: Option<Vec<&str>>, not_whole: Vec<&str>) {
    if let Ok(mut copies) = the_box().lock() {
        copies.keys = keys.map(|keys| keys.into_iter().map(String::from).collect());
        copies.not_whole = not_whole.into_iter().map(String::from).collect();
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
        the_keys_of_a_measurement(None, Vec::new());
        assert_eq!(
            the_copy_of_this_media("a-book"),
            TheCopyOfTheDisk::TheDiskDidNotAnswer
        );
        assert_eq!(
            crate::ui::keys::the_label_of_the_copy_of_the_disk(the_copy_of_this_media("a-book")),
            crate::ui::keys::THE_DISK_DID_NOT_ANSWER,
        );

        // The disk answered, and it holds one media of the two.
        the_keys_of_a_measurement(Some(vec!["a-book"]), Vec::new());
        assert_eq!(
            the_copy_of_this_media("a-book"),
            TheCopyOfTheDisk::AWholeCopy
        );
        assert_eq!(
            the_copy_of_this_media("a-second-book"),
            TheCopyOfTheDisk::NoCopy
        );
        assert_eq!(
            crate::ui::keys::the_label_of_the_copy_of_the_disk(the_copy_of_this_media("a-book")),
            crate::ui::keys::THE_COPY_OF_THE_DISK,
        );

        // **The disk holds the rows of that download and not every file of it**
        // (T-217). The playback of it takes the road of the server, therefore the
        // label says what the disk holds.
        the_keys_of_a_measurement(Some(vec!["a-book"]), vec!["a-book"]);
        assert_eq!(
            the_copy_of_this_media("a-book"),
            TheCopyOfTheDisk::ACopyThatIsNotWhole
        );
        assert_eq!(
            crate::ui::keys::the_label_of_the_copy_of_the_disk(the_copy_of_this_media("a-book")),
            crate::ui::keys::THE_COPY_THAT_IS_NOT_WHOLE,
        );

        // The disk answered, and it holds no media at all. A read of the disk
        // that gave no row is not a read that failed.
        the_keys_of_a_measurement(Some(vec![]), Vec::new());
        assert_eq!(the_copy_of_this_media("a-book"), TheCopyOfTheDisk::NoCopy);

        // The box holds one account: the keys of a second account take the
        // place of the keys before them.
        the_keys_of_a_measurement(Some(vec!["the-book-of-the-second-account"]), Vec::new());
        assert_eq!(the_copy_of_this_media("a-book"), TheCopyOfTheDisk::NoCopy);
        assert_eq!(
            the_copy_of_this_media("the-book-of-the-second-account"),
            TheCopyOfTheDisk::AWholeCopy
        );
    }
}
