//! T-282: a chapter that the book holds no file of says which chapter it is.
//!
//! The measurement, of the real program v0.8.110 inside tmux against the
//! sandbox. The book comes of
//! `docs/harness/a_book_of_a_chapter_that_is_absent.py`: the spine of it names
//! three chapters, and the manifest of the same file holds the file of two of
//! them. That book went into the cache of the ebooks of the account
//! `toutuitest` under the name of the item of `Alice in Wonderland`, and the
//! keys `Tab`, 15 keys `j`, `e`, and `p` gave
//!
//! ```text
//! This chapter is absent.
//! ```
//!
//! The three faults of that one sentence:
//!
//! 1. It named no chapter at all, and the program measured which chapter of
//!    the spine holds no file of the manifest.
//! 2. It named no key, and the view of the reader holds `n`, `p`, and `h`, and
//!    each of the three does the work of that fault (T-170).
//! 3. The log held 30 lines before the key and 30 lines after it: **no line of
//!    the reader at all**, while the two arms of the copy beside this road each
//!    write one already.
//!
//! These tests need no network and no sandbox. The book comes from
//! `tests/data/hostile/11-a-chapter-with-no-file.epub` of this repository.

use std::path::PathBuf;

use toutui::logic::reader::book::{Book, ReaderError};

/// The chapter of the spine that the manifest of the book does not hold. The
/// header of the reader calls it "chapter 2 of 3".
const THE_CHAPTER_OF_NO_FILE: usize = 1;

/// The book of this repository.
fn the_book() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/hostile/11-a-chapter-with-no-file.epub")
}

/// The spine names the chapter, and the manifest holds no file of it. The
/// value of the fault must therefore carry the number of that chapter, and the
/// sentence must name it and the keys of the view of the reader.
///
/// The parts of this test stay in one function: the two chapters of the same
/// book that read are the control of the chapter of no file, and the three of
/// them read one file.

#[test]
fn a_chapter_that_the_manifest_holds_no_file_of_names_that_chapter() {
    let book = Book::open(&the_book())
        .expect("the book opens: a spine of one idref of no item is no fault of the open");

    assert_eq!(
        3,
        book.chapter_count(),
        "the spine of the book holds three chapters"
    );

    let fault = book
        .chapter_xhtml(THE_CHAPTER_OF_NO_FILE)
        .expect_err("the chapter of no file gives no text");

    assert_eq!(
        ReaderError::ChapterAbsent(THE_CHAPTER_OF_NO_FILE),
        fault,
        "the fault carries the number of the chapter of the spine"
    );

    let sentence = fault.to_string();

    assert!(
        sentence.contains("chapter 2"),
        "the sentence names the chapter that the user reads: {sentence}"
    );
    assert!(
        sentence.contains("holds no file"),
        "the sentence says what the program measured: {sentence}"
    );

    // **A sentence of a fault must name a key that does the work of that
    // fault** (T-170). The view of the reader holds the three of them.
    for key in ["Press n ", "or p ", "Press h "] {
        assert!(
            sentence.contains(key),
            "the sentence names the key: {key} is not in {sentence}"
        );
    }
    assert!(
        sentence.contains("log"),
        "the sentence names the file of the log: {sentence}"
    );

    // The control of the same book: the chapter before this one and the
    // chapter after it each read. The book is good, and one chapter of it has
    // no file.
    for index in [THE_CHAPTER_OF_NO_FILE - 1, THE_CHAPTER_OF_NO_FILE + 1] {
        let text = book
            .chapter_xhtml(index)
            .unwrap_or_else(|fault| panic!("the chapter {index} reads: {fault}"));

        assert!(
            text.contains("This is a chapter of plain text"),
            "the chapter {index} holds its text"
        );
    }
}

/// The screen of that chapter holds the sentence of that fault, and the render
/// does not start again.
///
/// The test draws the real widget of the reader into a `Buffer`, with no
/// terminal at all, and it does the work of the loop of the screen: it takes
/// the answer of the task and it draws.
///
/// The parts of this test stay in one function: the frames of the loop are one
/// measurement.
#[tokio::test]
async fn the_screen_of_a_chapter_of_no_file_names_that_chapter() {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    let mut reader = toutui::logic::reader::Reader::open(&the_book(), "an item of no server")
        .expect("the reader opens the book");

    reader.go_to_chapter(THE_CHAPTER_OF_NO_FILE);

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
        words.contains("This book names a chapter 2, and it holds no file of that chapter"),
        "the screen says what the program measured: {words}"
    );
    assert!(
        words.contains("Press n for the chapter after this one"),
        "the screen names the key that does the work of that fault: {words}"
    );
    assert!(
        words.contains("Press h to leave the book"),
        "the screen names the key that goes back: {words}"
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
