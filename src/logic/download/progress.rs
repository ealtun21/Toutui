//! The progress of a download.
//!
//! The download task writes to this state. The user interface reads it and
//! draws. The task does not draw.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The condition of one download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadState {
    /// The task gets the files now.
    Running,
    /// The task got all the files.
    Finished,
    /// The task stopped. The text gives the cause.
    Failed(String),
}

/// The progress of one download.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// The identity of the download: the item of a book, or the episode of a
    /// podcast.
    pub key: String,
    /// The title of the book, or the title of the episode.
    pub title: String,
    /// The file that the task gets now. The first file has the number 1.
    pub file_index: usize,
    /// The number of files in the book.
    pub file_count: usize,
    /// The number of bytes on the disk, for all the files together.
    pub bytes_done: u64,
    /// The number of bytes of all the files together.
    pub bytes_total: u64,
    /// The condition of the download.
    pub state: DownloadState,
}

impl DownloadProgress {
    /// Gives the progress in percent, from 0 to 100.
    ///
    /// The function gives 0 when the total is 0. A division by zero is not
    /// possible.
    pub fn percent(&self) -> u16 {
        if self.bytes_total == 0 {
            return 0;
        }

        let value = self.bytes_done.saturating_mul(100) / self.bytes_total;
        value.min(100) as u16
    }
}

/// The progress of every download. The key is the field `key` of the
/// progress.
pub type ProgressMap = Arc<RwLock<HashMap<String, DownloadProgress>>>;

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(done: u64, total: u64) -> DownloadProgress {
        DownloadProgress {
            key: "item-1".to_string(),
            title: "A Book".to_string(),
            file_index: 1,
            file_count: 1,
            bytes_done: done,
            bytes_total: total,
            state: DownloadState::Running,
        }
    }

    #[test]
    fn no_bytes_give_zero_percent() {
        assert_eq!(progress(0, 100).percent(), 0);
    }

    #[test]
    fn one_half_gives_fifty_percent() {
        assert_eq!(progress(50, 100).percent(), 50);
    }

    #[test]
    fn all_the_bytes_give_one_hundred_percent() {
        assert_eq!(progress(100, 100).percent(), 100);
    }

    /// A book with no size must not cause a division by zero.
    #[test]
    fn a_total_of_zero_gives_zero_percent() {
        assert_eq!(progress(0, 0).percent(), 0);
        assert_eq!(progress(10, 0).percent(), 0);
    }

    /// A large book must not overflow the multiplication.
    #[test]
    fn a_large_book_gives_the_correct_percent() {
        assert_eq!(progress(350_000_000, 700_000_000).percent(), 50);
    }
}
