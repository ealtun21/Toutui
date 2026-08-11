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

/// The name of the variable that changes the limit of the cache.
///
/// **A measurement of the real program needs a small limit.** A cache of one
/// gigabyte needs some hundred books, and a session cannot make them. The
/// variable holds a number of bytes, and `TOUTUI_NO_COVERS` and
/// `TOUTUI_AUDIO_DEVICE` hold the same shape. See T-71.
pub const LIMIT_VARIABLE: &str = "TOUTUI_EBOOK_CACHE_BYTES";

/// The limit that `config.toml` gives, in bytes.
///
/// The cache runs inside a task, and that task holds no `App`. Therefore the
/// start of the program writes the value of the configuration file here, and this
/// is the shape of `logic::live` and of `logic::message`. See T-72.
fn box_of_the_limit() -> &'static std::sync::Mutex<Option<u64>> {
    static LIMIT: std::sync::OnceLock<std::sync::Mutex<Option<u64>>> = std::sync::OnceLock::new();
    LIMIT.get_or_init(|| std::sync::Mutex::new(None))
}

/// Writes the limit of the configuration file. The start of the program calls
/// this one time. See T-72.
pub fn keep_the_limit_of_the_configuration(megabytes: u64) {
    let bytes = megabytes.saturating_mul(1024 * 1024);

    if let Ok(mut place) = box_of_the_limit().lock() {
        *place = if bytes > 0 { Some(bytes) } else { None };
    }
}

/// Forgets the limit of the configuration file. A test calls this.
pub fn forget_the_limit_of_the_configuration() {
    if let Ok(mut place) = box_of_the_limit().lock() {
        *place = None;
    }
}

/// Gives the limit of the cache of the ebooks.
///
/// The sequence of the three sources: the variable of the environment, then
/// `config.toml`, then the value of the program. **The variable comes first**,
/// because a measurement of the real program needs a small limit and it must not
/// change the file of the user. See T-71 and T-72.
pub fn the_limit() -> u64 {
    let of_the_config = box_of_the_limit().lock().ok().and_then(|place| *place);

    the_limit_of(std::env::var(LIMIT_VARIABLE).ok().as_deref(), of_the_config)
}

/// Gives the limit for one value of the variable and one value of the
/// configuration file.
///
/// A value that is not a number, and the value 0, give the source that comes
/// after it. A cache of 0 bytes would remove every book of the disk, therefore
/// that value cannot mean itself.
///
/// The function is pure, therefore a test needs no variable of the environment
/// and no file. A test that writes such a variable must stand alone in its
/// binary. See the trap 29 of `docs/HANDOVER.md`.
pub fn the_limit_of(value: Option<&str>, of_the_config: Option<u64>) -> u64 {
    if let Ok(bytes) = value.map(str::trim).unwrap_or("").parse::<u64>() {
        if bytes > 0 {
            return bytes;
        }
    }

    match of_the_config {
        Some(bytes) if bytes > 0 => bytes,
        _ => LIMIT_OF_THE_CACHE,
    }
}

/// The values that the view of the settings offers, in megabytes. See T-77.
///
/// The user writes any value in `config.toml`. The view offers these values
/// only, therefore the user needs no line of text and no examination of what
/// they wrote.
pub const THE_VALUES_OF_THE_SETTINGS: &[u64] = &[256, 512, 1024, 2048, 4096, 8192];

/// Gives the line of one value of the view of the settings.
///
/// The line says which value the program uses now, and which value it uses when
/// `config.toml` names none. The function is pure, therefore a test needs no
/// screen. See T-77.
pub fn line_of_a_value(megabytes: u64, of_the_config: u64) -> String {
    let mark = if megabytes == of_the_config {
        "✓"
    } else {
        " "
    };

    let of_the_program = if megabytes.saturating_mul(1024 * 1024) == LIMIT_OF_THE_CACHE {
        " (the value of the program)"
    } else {
        ""
    };

    format!(
        "{} {}{}",
        mark,
        crate::logic::download::text_of_the_size(megabytes.saturating_mul(1024 * 1024)),
        of_the_program
    )
}

/// Gives the sentence of the removal for the user.
///
/// **The program removed a book of the user, therefore the user must read it.**
/// T-67 wrote that work in the log only, and a user who kept a book of a scan for
/// a journey then lost it with no word.
///
/// The function is pure, therefore a test needs no file. See T-71.
pub fn the_sentence_of_the_cache(books: usize, bytes: u64) -> String {
    format!(
        "The cache of the ebooks was full. The program removed {} book(s) of the \
         disk, and it gave {} back. Press e to get one again.",
        books,
        crate::logic::download::text_of_the_size(bytes)
    )
}

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
    the_removal(username, keep, limit).1
}

/// Holds the limit, and it gives the number of the books and the number of the
/// bytes that went away. See T-71.
pub fn the_removal(username: &str, keep: &Path, limit: u64) -> (usize, u64) {
    let books = the_cache_of(username);
    let must_go = the_ebooks_that_must_go(&books, keep, limit);

    if must_go.is_empty() {
        return (0, 0);
    }

    let mut bytes = 0u64;
    let mut count = 0usize;

    for path in must_go {
        let size = match std::fs::metadata(&path) {
            Ok(data) => data.len(),
            Err(_) => continue,
        };

        match std::fs::remove_file(&path) {
            Ok(()) => {
                bytes += size;
                count += 1;
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

    (count, bytes)
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

    /// The variable of the limit gives a number of bytes. A value that is not a
    /// number, and the value 0, give the limit of the program: a cache of 0 bytes
    /// would remove every book of the disk. See T-71.
    #[test]
    fn the_variable_of_the_limit_gives_a_number_of_bytes() {
        assert_eq!(the_limit_of(Some("4096"), None), 4096);
        assert_eq!(the_limit_of(Some("  4096  "), None), 4096);

        for wrong in [
            None,
            Some(""),
            Some("0"),
            Some("a lot"),
            Some("-1"),
            Some("4096.5"),
        ] {
            assert_eq!(
                the_limit_of(wrong, None),
                LIMIT_OF_THE_CACHE,
                "the value {:?} must give the limit of the program",
                wrong
            );
        }
    }

    /// The sequence of the three sources: the variable, then the configuration
    /// file, then the value of the program. See T-72.
    #[test]
    fn the_variable_comes_before_the_configuration_file() {
        // The variable comes first: a measurement must not need the file of the
        // user.
        assert_eq!(the_limit_of(Some("4096"), Some(9999)), 4096);

        // No variable, therefore the file of the user decides.
        assert_eq!(the_limit_of(None, Some(9999)), 9999);

        // A variable that is not a number gives the file, and not the value of
        // the program.
        assert_eq!(the_limit_of(Some("a lot"), Some(9999)), 9999);

        // Neither source gives a value.
        assert_eq!(the_limit_of(None, None), LIMIT_OF_THE_CACHE);
        assert_eq!(the_limit_of(None, Some(0)), LIMIT_OF_THE_CACHE);
    }

    /// The slot of the configuration file takes megabytes, and it gives bytes. A
    /// value of 0 megabytes gives the value of the program.
    ///
    /// The view of the settings names the value that the program uses now, and
    /// the value that it uses when the file names none. See T-77.
    #[test]
    fn the_line_of_the_view_of_the_settings_names_the_value_of_now() {
        let lines: Vec<String> = THE_VALUES_OF_THE_SETTINGS
            .iter()
            .map(|value| line_of_a_value(*value, 2048))
            .collect();

        assert_eq!(
            lines.iter().filter(|line| line.contains('✓')).count(),
            1,
            "one value of the list is the value of now: {:?}",
            lines
        );

        assert!(lines
            .iter()
            .any(|line| line.contains("2048 MB") && line.contains('✓')));
        assert!(lines
            .iter()
            .any(|line| line.contains("1024 MB") && line.contains("the value of the program")));

        // A value that no line of the list holds gives a list with no mark. The
        // user wrote that value in the file, and the view must not say that a
        // different value is the value of now.
        let lines: Vec<String> = THE_VALUES_OF_THE_SETTINGS
            .iter()
            .map(|value| line_of_a_value(*value, 3000))
            .collect();

        assert_eq!(lines.iter().filter(|line| line.contains('✓')).count(), 0);
    }

    /// **The slot belongs to the process, therefore every part of this test
    /// stays in one function.** See the trap 29 of `docs/HANDOVER.md`.
    #[test]
    fn the_slot_of_the_configuration_holds_megabytes() {
        forget_the_limit_of_the_configuration();
        assert_eq!(the_limit(), LIMIT_OF_THE_CACHE);

        keep_the_limit_of_the_configuration(2);
        assert_eq!(the_limit(), 2 * 1024 * 1024);

        keep_the_limit_of_the_configuration(0);
        assert_eq!(the_limit(), LIMIT_OF_THE_CACHE);

        // A number of megabytes that would go outside a `u64` must not stop the
        // program.
        keep_the_limit_of_the_configuration(u64::MAX);
        assert_eq!(the_limit(), u64::MAX);

        forget_the_limit_of_the_configuration();
    }

    /// The sentence names the number of the books, the size, and the key that
    /// gets a book again. See T-71.
    #[test]
    fn the_sentence_of_the_cache_names_the_key() {
        let sentence = the_sentence_of_the_cache(2, 5 * 1024 * 1024);

        assert!(sentence.contains('2'), "{}", sentence);
        assert!(sentence.contains("5 MB"), "{}", sentence);
        assert!(sentence.contains("Press e"), "{}", sentence);
        assert!(
            sentence.chars().count() <= 150,
            "the row of the screen holds one line: {} letters",
            sentence.chars().count()
        );
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
