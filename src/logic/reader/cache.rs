//! The limit of the cache of the ebooks. See T-67.
//!
//! The reader of T-10 keeps the file of an ebook on the disk: a second visit of
//! the book then needs no request, and the reader also works with no server.
//! **The program removed no such file of its own.** T-65 gave the user the key
//! `X` for one media, and a user of twenty books of a scan still held twenty
//! files: the measurement of T-62 used a PDF of 137 megabytes.
//!
//! The rule of this module:
//!
//! - The cache stands at or below `LIMIT_OF_THE_CACHE`.
//! - The program removes the book of the oldest use first.
//! - **The book that the user reads now never goes away**, and the limit does
//!   not hold for it: one book of 500 megabytes is a correct cache of one book.
//! - The time of the file is the time of the **last use**, and not the time of
//!   the download. `the_book_is_in_use` writes that time each time the reader
//!   opens the book, therefore a book that the user reads every day stays.
//!
//! The program looks at the limit after a new book came, because that is the
//! one moment when the cache grows.

use log::{error, info};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The largest cache of the ebooks, in bytes.
///
/// One gigabyte holds some hundred EPUB books, or seven books of a scan of the
/// size of the measurement of T-62. `MAX_BOOK_BYTES` of the reader is 512
/// megabytes for one book, therefore this limit holds two of the largest books
/// that the reader opens.
pub const LIMIT_OF_THE_CACHE: u64 = 1024 * 1024 * 1024;

/// One ebook of the cache on the disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InTheCache {
    pub path: PathBuf,
    pub bytes: u64,
    /// The time of the last use of the book.
    pub used: SystemTime,
}

/// Gives the ebooks that must go away, so the cache stands at or below the
/// limit.
///
/// The book of the oldest use goes first. `keep` never goes away: it is the
/// book that the user reads now, and a cache that removes it would ask the
/// server for it again at once.
///
/// The function is pure, therefore a test needs no file.
pub fn the_ebooks_that_must_go(books: &[InTheCache], keep: &Path, limit: u64) -> Vec<PathBuf> {
    let mut total: u64 = books.iter().map(|book| book.bytes).sum();

    if total <= limit {
        return Vec::new();
    }

    let mut of_the_oldest_use: Vec<&InTheCache> =
        books.iter().filter(|book| book.path != keep).collect();

    of_the_oldest_use.sort_by_key(|book| book.used);

    let mut answer: Vec<PathBuf> = Vec::new();

    for book in of_the_oldest_use {
        if total <= limit {
            break;
        }

        total -= book.bytes;
        answer.push(book.path.clone());
    }

    answer
}

/// Says that the reader uses this book now.
///
/// The function writes the time of the file. A book that the user reads every
/// day therefore stays in the cache, and the book of the oldest use goes away
/// first. A fault of the write is not a fault of the program: the book then
/// keeps the time of its download.
pub fn the_book_is_in_use(path: &Path) {
    let times = std::fs::FileTimes::new()
        .set_accessed(SystemTime::now())
        .set_modified(SystemTime::now());

    let answer = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .and_then(|file| file.set_times(times));

    if let Err(error) = answer {
        info!(
            "[cache] the time of {} did not change: {}",
            path.display(),
            error
        );
    }
}

/// Gives every ebook of the cache of one user.
///
/// The ebooks stand in the directory of the downloads, and the audio of a
/// download stands in a directory of that directory. Therefore this function
/// takes the files of the two forms of an ebook only.
pub fn the_cache_of(username: &str) -> Vec<InTheCache> {
    let directory = crate::logic::download::downloads_base_dir(username);

    let Ok(rows) = std::fs::read_dir(&directory) else {
        return Vec::new();
    };

    let mut books: Vec<InTheCache> = Vec::new();

    for row in rows.flatten() {
        let path = row.path();
        let is_an_ebook = matches!(
            path.extension().and_then(|name| name.to_str()),
            Some("epub") | Some("pdf")
        );

        if !is_an_ebook {
            continue;
        }

        let Ok(data) = row.metadata() else {
            continue;
        };

        if !data.is_file() {
            continue;
        }

        books.push(InTheCache {
            path,
            bytes: data.len(),
            used: data.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        });
    }

    books
}

/// Holds the cache of the ebooks at or below the limit.
///
/// `keep` is the book that the user reads now, and it never goes away. The
/// function gives the number of bytes that it removed, therefore a caller can
/// write that number in the log.
pub fn hold_the_limit(username: &str, keep: &Path, limit: u64) -> u64 {
    let books = the_cache_of(username);
    let must_go = the_ebooks_that_must_go(&books, keep, limit);

    if must_go.is_empty() {
        return 0;
    }

    let mut bytes = 0u64;

    for path in must_go {
        let size = match std::fs::metadata(&path) {
            Ok(data) => data.len(),
            Err(_) => continue,
        };

        match std::fs::remove_file(&path) {
            Ok(()) => {
                bytes += size;
                info!(
                    "[cache] the cache of the ebooks is full. The program \
                     removed {} of {} bytes.",
                    path.display(),
                    size
                );
            }
            Err(error) => error!("[cache] the ebook {} stays: {}", path.display(), error),
        }
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn a_book(name: &str, bytes: u64, seconds: u64) -> InTheCache {
        InTheCache {
            path: PathBuf::from(name),
            bytes,
            used: SystemTime::UNIX_EPOCH + Duration::from_secs(seconds),
        }
    }

    #[test]
    fn a_cache_below_the_limit_loses_no_book() {
        let books = vec![a_book("a.epub", 10, 1), a_book("b.pdf", 20, 2)];

        assert!(the_ebooks_that_must_go(&books, Path::new("b.pdf"), 30).is_empty());
        assert!(the_ebooks_that_must_go(&[], Path::new("a.epub"), 30).is_empty());
    }

    /// The book of the oldest use goes first, and the program stops at the
    /// limit. It must not empty the cache.
    #[test]
    fn the_book_of_the_oldest_use_goes_first() {
        let books = vec![
            a_book("old.pdf", 100, 1),
            a_book("newer.pdf", 100, 2),
            a_book("now.epub", 100, 3),
        ];

        assert_eq!(
            the_ebooks_that_must_go(&books, Path::new("now.epub"), 250),
            vec![PathBuf::from("old.pdf")],
            "one book of 100 bytes makes 200 of a limit of 250"
        );

        assert_eq!(
            the_ebooks_that_must_go(&books, Path::new("now.epub"), 100),
            vec![PathBuf::from("old.pdf"), PathBuf::from("newer.pdf")]
        );
    }

    /// The book that the user reads now stays, even when that one book is
    /// larger than the whole limit. A cache that removes it asks the server for
    /// it again at once.
    #[test]
    fn the_book_of_the_user_never_goes_away() {
        let books = vec![
            a_book("small.epub", 10, 1),
            a_book("the book of the user.pdf", 900, 2),
        ];

        assert_eq!(
            the_ebooks_that_must_go(&books, Path::new("the book of the user.pdf"), 100),
            vec![PathBuf::from("small.epub")],
            "the program removes every other book, and it stops"
        );

        // One book, and it is larger than the limit. Nothing goes away.
        let one = vec![a_book("the book of the user.pdf", 900, 2)];

        assert!(
            the_ebooks_that_must_go(&one, Path::new("the book of the user.pdf"), 100).is_empty()
        );
    }

    /// A cache of a directory that does not exist gives no book and no fault.
    #[test]
    fn a_cache_that_does_not_exist_gives_no_book() {
        let books = the_cache_of("a user of no directory of this machine");

        assert!(books.is_empty());
        assert_eq!(
            hold_the_limit(
                "a user of no directory of this machine",
                Path::new("a.epub"),
                1
            ),
            0
        );
    }

    /// The whole path of the file names the book that stays. Two users can hold
    /// the same item, and the directory of each user is its own.
    #[test]
    fn the_book_that_stays_is_the_whole_path() {
        let books = vec![
            a_book("/of/a/user/the same book.epub", 100, 1),
            a_book("/of/a/second user/the same book.epub", 100, 2),
        ];

        assert_eq!(
            the_ebooks_that_must_go(
                &books,
                Path::new("/of/a/second user/the same book.epub"),
                100
            ),
            vec![PathBuf::from("/of/a/user/the same book.epub")]
        );
    }
}
