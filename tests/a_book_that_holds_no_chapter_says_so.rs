//! T-283: a book that holds no chapter says so, and the header says the number
//! that the program measured.
//!
//! The measurement, of the real program v0.8.111 inside tmux against the
//! sandbox. The book comes of `docs/harness/a_book_of_no_chapter.py`: the
//! manifest of it holds the files of two chapters, and the spine of the same
//! file names no chapter at all. That book went into the cache of the ebooks of
//! the account `toutuitest` under the name of the item of
//! `Alice in Wonderland`, and the keys `Tab`, 15 keys `j`, and `e` gave
//!
//! ```text
//! The Book Of No Chapter — chapter 1 of 1 — 0%
//!                     This book has no chapter 0.
//! ```
//!
//! **A book of no chapter is the one road of the real program to
//! `ReaderError::NoSuchChapter`**: `go_to_chapter` of
//! `src/logic/reader/session.rs` guards every other road with
//! `chapter >= self.chapter_count()`, and a book of no chapter keeps the reader
//! at the chapter 0 that `Reader::open_with_the_title` writes.
//!
//! The three faults of that screen:
//!
//! 1. The header said `chapter 1 of 1` for a book of no chapter. The program
//!    measured 0, and a `count.max(1)` of an older version, which kept a
//!    division by zero away, told the user that the book holds one chapter and
//!    that the reader stands in it.
//! 2. The sentence said `chapter 0`, and no view of this program says that
//!    number: the header calls the index 0 "chapter 1".
//! 3. The sentence named no key at all, and the log held no line of the reader:
//!    `grep -c reader` of it gave 0.
//!
//! These tests need no network and no sandbox. The book comes from
//! `tests/data/hostile/12-a-book-of-no-chapter.epub` of this repository.

use std::path::PathBuf;

use toutui::logic::reader::book::{Book, ReaderError};

/// The book of this repository.
fn the_book() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/hostile/12-a-book-of-no-chapter.epub")
}

/// The spine of the book names no chapter. The value of the fault must
/// therefore carry the number of the chapters of the book, and the sentence
/// must say that the book holds none and name the key of the view.
///
/// The parts of this test stay in one function: the open of the book, the
/// count, and the fault of the chapter 0 are one measurement of one file.

#[test]
fn a_book_that_names_no_chapter_says_so_and_it_says_no_number() {
    let book = Book::open(&the_book())
        .expect("the book opens: a spine of no itemref is no fault of the open");

    assert_eq!(
        0,
        book.chapter_count(),
        "the spine of the book names no chapter"
    );

    let fault = book
        .chapter_xhtml(0)
        .expect_err("a book of no chapter gives no text of the chapter 0");

    assert_eq!(
        ReaderError::NoSuchChapter { asked: 0, count: 0 },
        fault,
        "the fault carries the number of the chapters of the book"
    );

    let sentence = fault.to_string();

    assert!(
        sentence.contains("names no chapter"),
        "the sentence says what the program measured: {sentence}"
    );

    // **A number of a sentence of a fault is the number that the user reads,
    // and not the index of the program** (T-283). The header of the reader
    // calls the index 0 "chapter 1", therefore no sentence of this program
    // says "chapter 0".
    assert!(
        !sentence.contains("chapter 0"),
        "the sentence names no index of the program: {sentence}"
    );

    // **A sentence of a fault must name a key that does the work of that
    // fault** (T-170). The keys `n` and `p` do nothing here, because
    // `go_to_chapter` holds a book of no chapter at the chapter 0. The key `h`
    // is the one key of this fault.
    assert!(
        sentence.contains("Press h to leave the book"),
        "the sentence names the key that goes back: {sentence}"
    );
    assert!(
        !sentence.contains("Press n"),
        "the sentence promises no key that does nothing (T-143): {sentence}"
    );
    assert!(
        sentence.contains("log"),
        "the sentence names the file of the log: {sentence}"
    );
}

/// A book that holds chapters says the number that the user reads.
///
/// This value stands behind the guard of `go_to_chapter` in the real program,
/// and the number of it is therefore the number of a caller of the library.
#[test]
fn a_chapter_that_is_past_the_end_names_the_number_of_the_view() {
    let sentence = ReaderError::NoSuchChapter {
        asked: 14,
        count: 14,
    }
    .to_string();

    assert!(
        sentence.contains("14 chapters"),
        "the sentence names the number of the chapters of the book: {sentence}"
    );
    assert!(
        sentence.contains("the chapter 15"),
        "the sentence adds one to the index, as the header of the reader does: {sentence}"
    );
    assert!(
        sentence.contains("Press h to leave the book"),
        "the sentence names the key that goes back: {sentence}"
    );
}

/// The screen of that book holds the sentence of that fault, and the header of
/// it names no chapter at all.
///
/// The test draws the real widget of the reader into a `Buffer`, with no
/// terminal at all, and it does the work of the loop of the screen: it takes
/// the answer of the task and it draws.
///
/// The parts of this test stay in one function: the frames of the loop are one
/// measurement.
#[tokio::test]
async fn the_screen_of_a_book_of_no_chapter_names_no_chapter() {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    let mut reader = toutui::logic::reader::Reader::open(&the_book(), "an item of no server")
        .expect("the reader opens the book");

    // A width of 100 columns is the width of the text of the reader
    // (`MAX_TEXT_WIDTH`), and the sentence of this fault is longer than one
    // line of it (T-278).
    let area = Rect::new(0, 0, 100, 30);
    let mut words = String::new();

    for _ in 0..200 {
        let mut buffer = Buffer::empty(area);

        reader.take_the_answer();
        let (of_the_book, of_the_footer) = the_two_areas(area);
        toutui::ui::reader_tui::render(&mut reader, of_the_book, of_the_footer, &mut buffer);

        words = the_words_of(&buffer);

        if !words.contains("Reading") {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    assert!(
        words.contains("This book names no chapter"),
        "the screen says what the program measured: {words}"
    );
    assert!(
        words.contains("Press h to leave the book"),
        "the screen names the key that goes back: {words}"
    );

    // **The header says the number that the program measured** (T-283).
    assert!(
        words.contains("this book holds no chapter"),
        "the header says that the book holds no chapter: {words}"
    );
    assert!(
        !words.contains("chapter 1 of 1"),
        "the header names no chapter of a book that holds none: {words}"
    );
    assert!(
        !words.contains("chapter 0"),
        "no row of the screen holds the index of the program: {words}"
    );
}

/// The words of a screen, with no line and no colour.
///
/// The wrap of the paragraph breaks a sentence at a space, and the render puts
/// that sentence in the middle of the width. Therefore this function makes one
/// space of every run of spaces and of every end of a line: a sentence of two
/// rows then reads as one sentence.
fn the_words_of(buffer: &ratatui::buffer::Buffer) -> String {
    let mut words = String::new();

    for row in 0..buffer.area.height {
        for column in 0..buffer.area.width {
            words.push_str(buffer[(column, row)].symbol());
        }
        words.push(' ');
    }

    words.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// The two areas of the render of the reader: the book, and the footer under
/// it. **The band of the player stands between the two of them** (T-343),
/// therefore the caller of `reader_tui::render` gives it the area of the footer.
fn the_two_areas(area: ratatui::layout::Rect) -> (ratatui::layout::Rect, ratatui::layout::Rect) {
    let of_the_footer = ratatui::layout::Rect::new(
        area.x,
        area.y + area.height.saturating_sub(2),
        area.width,
        2.min(area.height),
    );
    let of_the_book =
        ratatui::layout::Rect::new(area.x, area.y, area.width, area.height.saturating_sub(2));

    (of_the_book, of_the_footer)
}
