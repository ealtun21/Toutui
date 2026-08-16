//! A book that the reader refuses at its open says the road back. See T-284.
//!
//! `ReaderError::BookTooLarge` and `ReaderError::TooManyEntries` of
//! `src/logic/reader/book.rs` come at the open of a book, and each of them gave
//! one sentence of numbers alone.
//!
//! The measurement, of the real program v0.8.112 inside tmux against the
//! sandbox on 2026-08-16. **The data of each fault is a book, and neither needs
//! a proxy at all**: `docs/harness/a_book_of_too_many_files.py` writes an EPUB
//! whose manifest names 4201 files in an archive of 1211260 bytes, and
//! `docs/harness/a_book_that_is_too_large.py` writes an EPUB of 269486151
//! bytes. Each book went into the cache of the ebooks of the account
//! `toutuitest`, under the name of the item of `Alice in Wonderland`. The keys
//! `Tab`, 15 keys `j`, and `e` gave
//!
//! ```text
//! This book holds too many files. It has 4201 files, and the limit is 4096 files.
//! This book is too large. It has 269486151 bytes, and the limit is 268435456 bytes.
//! ```
//!
//! and the log held 7 lines and 6 lines after the key, and `grep -c reader` of
//! it gave **0** for each of the two: no line of the reader at all.
//!
//! Three faults.
//!
//! 1. **Neither sentence names a key.** The view of the reader with no book
//!    stands at that moment — `get_the_book` of `src/app.rs` writes
//!    `AppView::Reader` before it starts the task — and the footer of it says
//!    `h/Esc: back`. The key `h` is the one key of these two faults: no key of
//!    this program makes a book smaller or takes a file out of its manifest.
//!    See T-170.
//! 2. **Neither road takes a line of the log**, and the name of the file of
//!    that book stands in no view of the user.
//! 3. **The sentence of the size says a wall of digits.** The bar of a download
//!    of the same program says "1.2 MB" already.
//!
//! A fourth fault of the same two values stood in the source: the arms of
//! `Display` named the constants of `src/logic/reader/book.rs`, and
//! `src/logic/reader/pdf.rs` makes the same two values with a limit of 512
//! megabytes and a limit of 5000 pages. A sentence of that road would say a
//! number that the program did not measure (T-91). The two values therefore
//! carry their limit.

use toutui::logic::reader::book::{Book, ReaderError, MAX_BOOK_BYTES, MAX_ENTRIES};

/// The real book of the measurement gives the value of the fault.
///
/// The file of the test holds the files of the spine alone: `Book::open` counts
/// the manifest of the OPF, and it opens no file of it. The book of the
/// measurement holds every file of its manifest, and it takes 1211260 bytes.
#[test]
fn a_book_of_too_many_files_gives_that_value() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/hostile/13-a-book-of-too-many-files.epub");

    let fault = Book::open(&path)
        .err()
        .unwrap_or_else(|| panic!("the reader must refuse the book of {}", path.display()));

    assert_eq!(
        ReaderError::TooManyEntries {
            count: 4201,
            limit: MAX_ENTRIES,
        },
        fault
    );
}

/// The sentence of a book that is too large says the two sizes in megabytes,
/// and it names no wall of digits.
#[test]
fn the_sentence_of_a_book_that_is_too_large_says_megabytes() {
    let text = ReaderError::BookTooLarge {
        size: 269_486_151,
        limit: MAX_BOOK_BYTES,
    }
    .to_string();

    assert!(text.contains("257.0 MB"), "{text}");
    assert!(text.contains("256.0 MB"), "{text}");
    assert!(
        !text.contains("269486151") && !text.contains("268435456"),
        "the sentence holds the bytes of before the correction: {text}"
    );
}

/// The sentence of a book of too many files says the count and the limit.
#[test]
fn the_sentence_of_a_book_of_too_many_files_says_the_two_numbers() {
    let text = ReaderError::TooManyEntries {
        count: 4201,
        limit: MAX_ENTRIES,
    }
    .to_string();

    assert!(text.contains("4201 files"), "{text}");
    assert!(text.contains("4096 files"), "{text}");
}

/// Each of the two sentences names the key `h` and the file of the log.
///
/// **The keys `n` and `p` do no work of these two faults**: the reader holds no
/// book at all, therefore the view of the reader draws the message alone and
/// the footer of it says `h/Esc: back`.
#[test]
fn each_sentence_names_the_key_of_the_view_and_the_log() {
    let sentences = [
        ReaderError::BookTooLarge {
            size: 1,
            limit: MAX_BOOK_BYTES,
        }
        .to_string(),
        ReaderError::TooManyEntries {
            count: 1,
            limit: MAX_ENTRIES,
        }
        .to_string(),
    ];

    for text in sentences {
        assert!(
            text.contains("Press h "),
            "the sentence names no key: {text}"
        );
        assert!(
            !text.contains("Press n ") && !text.contains("or p "),
            "the view of this fault holds neither of those keys: {text}"
        );
        assert!(
            text.contains("file of the log"),
            "the sentence names no log: {text}"
        );
    }
}

/// The two values carry the limit of the reader that measured them.
///
/// `src/logic/reader/pdf.rs` holds a limit of 512 megabytes and a limit of 5000
/// pages, and the arms of `Display` named the constants of
/// `src/logic/reader/book.rs` alone.
#[test]
fn the_sentence_says_the_limit_of_the_value_and_not_the_limit_of_one_file() {
    let text = ReaderError::BookTooLarge {
        size: 600 * 1024 * 1024,
        limit: 512 * 1024 * 1024,
    }
    .to_string();
    assert!(text.contains("512.0 MB"), "{text}");
    assert!(!text.contains("256.0 MB"), "{text}");

    let text = ReaderError::TooManyEntries {
        count: 6000,
        limit: 5000,
    }
    .to_string();
    assert!(text.contains("5000 files"), "{text}");
    assert!(!text.contains("4096 files"), "{text}");
}

/// The one form of a size of this program. The bar of a download and the reader
/// say a size in the same way.
#[test]
fn the_program_says_a_size_in_one_form() {
    assert_eq!("1.0 MB", toutui::ui::keys::megabytes(1_048_576));
    assert_eq!("0.5 MB", toutui::ui::keys::megabytes(524_288));
    assert_eq!("257.0 MB", toutui::ui::keys::megabytes(269_484_032));
}
