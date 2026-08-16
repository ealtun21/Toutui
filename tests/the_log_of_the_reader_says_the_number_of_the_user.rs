//! T-287: a line of the log of the reader says the number of the user.
//!
//! The measurement, of the real program v0.8.115 inside tmux against the
//! sandbox. The book `docs/harness/a_book_of_a_damaged_chapter.py` went in the
//! cache of the ebooks of the account `toutuitest`, and the keys `Tab`, 15 keys
//! `j`, `e`, and `p` gave the view of the reader on the chapter that the
//! archive does not give. The screen said:
//!
//! ```text
//! The Book Of A Damaged Chapter — chapter 2 of 3 — 50%
//! The book gave no text of this chapter. …
//! ```
//!
//! and the one line of the log of that same key said:
//!
//! ```text
//! [reader] the archive gave no chapter 1 of the book: …corrupt deflate stream
//! ```
//!
//! The book `docs/harness/a_book_of_no_chapter.py` gave the second line of the
//! same class: `[reader] the book holds 0 chapters, and the program asked for
//! the chapter 0`. No book of any user holds a chapter 0.
//!
//! **The index of the spine is not the number of the user** (the rule of
//! T-283): the header of the reader calls the index 0 "chapter 1". The two
//! arms beside the first line say `index + 1` already, and the three lines of
//! `src/logic/reader/session.rs` say `chapter + 1` already. These two lines
//! said the index, therefore the one fault of one key read two different
//! numbers in two places.
//!
//! **The parts of this test stay in one function.** The logger of the crate
//! `log` is a box of the process, and two test functions of one module fight
//! for that slot (the shape of T-144 and of T-157).
//!
//! This test needs no network and no sandbox. The two books come from
//! `tests/data/hostile/` of this repository.

use std::path::PathBuf;
use std::sync::Mutex;

use toutui::logic::reader::book::{Book, ReaderError};

/// The lines that the program wrote while this test ran.
static THE_LINES: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// A logger that keeps the lines in the memory.
struct TheLinesOfTheTest;

impl log::Log for TheLinesOfTheTest {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if let Ok(mut lines) = THE_LINES.lock() {
            lines.push(record.args().to_string());
        }
    }

    fn flush(&self) {}
}

/// Gives the one line of the log that holds a text, of the lines that came
/// after `from`.
fn the_line_that_holds(from: usize, text: &str) -> String {
    let lines = THE_LINES.lock().expect("the box of the lines opens");
    let of_the_fault: Vec<&String> = lines
        .iter()
        .skip(from)
        .filter(|line| line.contains(text))
        .collect();

    assert_eq!(
        of_the_fault.len(),
        1,
        "the log must hold one line of \"{text}\", and it holds {:?}",
        of_the_fault
    );

    of_the_fault[0].clone()
}

/// Gives the number of the lines of the log at this moment.
fn the_count_of_the_lines() -> usize {
    THE_LINES.lock().expect("the box of the lines opens").len()
}

/// A book of `tests/data/hostile/`.
fn the_hostile_book(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/hostile")
        .join(name)
}

#[test]
fn the_log_of_the_reader_says_the_number_of_the_user() {
    log::set_boxed_logger(Box::new(TheLinesOfTheTest)).expect("this test holds the slot alone");
    log::set_max_level(log::LevelFilter::Trace);

    // 1. The chapter that the archive does not give. The spine of the book
    //    names three chapters, and the second of them is the damaged one:
    //    the index 1 of the program is the chapter 2 of the user.
    let book = Book::open(&the_hostile_book("14-a-book-of-a-damaged-chapter.epub"))
        .expect("the archive of this book opens");
    assert_eq!(book.chapter_count(), 3, "the spine names three chapters");

    let before = the_count_of_the_lines();
    let fault = book
        .chapter_xhtml(1)
        .expect_err("the archive gives no byte of the chapter 2");
    assert!(
        matches!(fault, ReaderError::TheArchiveGaveNoChapter(_)),
        "the damaged stream takes that arm, and it gave {fault:?}"
    );

    let line = the_line_that_holds(before, "the archive gave no chapter");
    assert!(
        line.contains("the archive gave no chapter 2 of the book"),
        "the line must say the chapter 2 of the user, and it says: {line}"
    );

    // 2. The book that names no chapter at all. `go_to_chapter` holds such a
    //    book at the index 0, and that index is the chapter 1 of the user.
    let book = Book::open(&the_hostile_book("12-a-book-of-no-chapter.epub"))
        .expect("the archive of this book opens");
    assert_eq!(book.chapter_count(), 0, "the spine names no chapter");

    let before = the_count_of_the_lines();
    let fault = book
        .chapter_xhtml(0)
        .expect_err("a book of no chapter gives no chapter 1");
    assert!(
        matches!(fault, ReaderError::NoSuchChapter { .. }),
        "a book of no chapter takes that arm, and it gave {fault:?}"
    );

    let line = the_line_that_holds(before, "the program asked for the chapter");
    assert!(
        line.contains("the program asked for the chapter 1"),
        "the line must say the chapter 1 of the user, and it says: {line}"
    );
    assert!(
        !line.contains("the chapter 0"),
        "no book of any user holds a chapter 0, and the line says: {line}"
    );
}
