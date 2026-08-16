//! T-277: a chapter that the book did not give says why.
//!
//! The measurement, of the real program v0.8.105 inside tmux against the
//! sandbox, of the book `Alice in Wonderland`: 64 bytes of the deflate stream
//! of the file of the chapter 3 of 14 went the other way, and the reader then
//! said `Reading…` for ever. The book was good: every other chapter of it read.
//!
//! The two faults of that one screen:
//!
//! 1. `chapter_bytes` gave `ReaderError::ChapterAbsent` for every fault of
//!    `copy_bytes`, and the archive held that chapter. That is a reason that the
//!    program does not have (T-91), and the fault of the crate took no line of
//!    the log at all.
//! 2. `render_for` read `self.lines.is_empty()` for "the render came back". A
//!    chapter that gave a fault gives no line, therefore the render started
//!    again at every frame: `take_the_answer` wrote the message of the fault,
//!    and `render_for` of the same frame wrote `Reading…` over it before the
//!    draw.
//!
//! T-286 holds the same condition, and it reads the keys of that sentence. The
//! measurement, of the real program v0.8.114 inside tmux against the sandbox,
//! pressed the three keys of the view of the reader on this fault: the key `n`
//! gave the chapter after it and the key `p` gave the chapter before it, and
//! each of the two read at once, and the key `h` gave the Library view. The
//! sentence named the key `n` alone.
//!
//! These tests need no network and no sandbox. The book comes from
//! `tests/data/alice.epub` of this repository.

use std::path::{Path, PathBuf};

use toutui::logic::reader::book::{Book, ReaderError};

/// The chapter of the spine that the measurement damaged. It is the file
/// `OEBPS/6260297267691793459_11-h-1.htm.html`, and the header of the reader
/// calls it "chapter 3 of 14".
const THE_CHAPTER_OF_THE_MEASUREMENT: usize = 2;

/// The book of this repository.
fn the_good_book() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/alice.epub")
}

/// Gives a copy of the book with the deflate stream of one file of the archive
/// damaged.
///
/// The central directory and the local header of the entry keep every number,
/// therefore the archive still names the chapter: the read of the bytes is the
/// one thing that fails. That is the condition of a disk with a bad sector and
/// of a download that a machine stopped in the middle.
fn a_book_with_one_file_damaged(directory: &Path, name: &str) -> PathBuf {
    let mut bytes = std::fs::read(the_good_book()).expect("the book of the repository reads");

    let start = the_bytes_of_the_entry(&bytes, name).expect("the archive holds that file");

    // The stream of deflate holds more than 2064 bytes for this file. A change
    // in the middle of it keeps the head of the stream, therefore the fault
    // comes of the read and not of the shape of the entry.
    for byte in bytes.iter_mut().skip(start + 2000).take(64) {
        *byte ^= 0xFF;
    }

    let path = directory.join("a_book_of_one_damaged_chapter.epub");
    std::fs::write(&path, &bytes).expect("the disk takes the book");

    path
}

/// Gives the place of the first byte of the data of one entry of a zip archive.
///
/// The local header of an entry holds `PK\x03\x04`, the length of the name at
/// the byte 26, and the length of the extra field at the byte 28.
fn the_bytes_of_the_entry(bytes: &[u8], name: &str) -> Option<usize> {
    let head = b"PK\x03\x04";

    for place in 0..bytes.len().saturating_sub(30) {
        if &bytes[place..place + 4] != head {
            continue;
        }

        let of_the_name = u16::from_le_bytes([bytes[place + 26], bytes[place + 27]]) as usize;
        let of_the_extra = u16::from_le_bytes([bytes[place + 28], bytes[place + 29]]) as usize;
        let start_of_the_name = place + 30;

        if start_of_the_name + of_the_name > bytes.len() {
            continue;
        }

        if &bytes[start_of_the_name..start_of_the_name + of_the_name] == name.as_bytes() {
            return Some(start_of_the_name + of_the_name + of_the_extra);
        }
    }

    None
}

/// The archive holds the chapter, and it gave no byte of it. The value of the
/// fault must therefore hold the reason of the crate, and never the sentence of
/// a chapter that the archive does not hold.
///
/// The parts of this test stay in one function: the good chapter of the same
/// book is the control of the damaged one, and the two of them read one file.
#[test]
fn a_chapter_that_the_archive_did_not_give_holds_the_reason() {
    let directory = tempfile::tempdir().expect("the disk gives a directory");
    let path = a_book_with_one_file_damaged(
        directory.path(),
        "OEBPS/6260297267691793459_11-h-1.htm.html",
    );

    let book = Book::open(&path).expect("the book opens: one damaged file is no fault of the open");

    let fault = book
        .chapter_xhtml(THE_CHAPTER_OF_THE_MEASUREMENT)
        .expect_err("the chapter of the damaged file gives no text");

    let reason = match &fault {
        ReaderError::TheArchiveGaveNoChapter(reason) => reason.clone(),
        other => panic!("the archive that gave no chapter says: {other:?}"),
    };

    assert!(
        !reason.trim().is_empty(),
        "the fault of the archive holds the reason of the crate"
    );

    let sentence = fault.to_string();

    assert!(
        sentence.contains(&reason),
        "the user reads the reason of the machine: {sentence}"
    );
    assert!(
        !sentence.contains("This chapter is absent."),
        "the archive holds this chapter: {sentence}"
    );

    // **The sentence names the three keys of the view of the reader** (T-286).
    // The measurement pressed each of the three on this fault, and each of them
    // did its work: a sentence of a fault must name a key that does the work of
    // that fault (T-170).
    for key in ["Press n", "or p for", "Press h to leave the book"] {
        assert!(
            sentence.contains(key),
            "the sentence names the key {key} of the view of the reader: {sentence}"
        );
    }

    // The control of the same book: the chapter after the damaged one and the
    // chapter before it each read. The book is good, and one file of it is not,
    // therefore the keys `n` and `p` of the sentence each do their work.
    let after = book
        .chapter_xhtml(THE_CHAPTER_OF_THE_MEASUREMENT + 1)
        .expect("the chapter after the damaged one reads");

    assert!(
        after.len() > 1000,
        "the chapter after the damaged one holds its text"
    );

    let before = book
        .chapter_xhtml(THE_CHAPTER_OF_THE_MEASUREMENT - 1)
        .expect("the chapter before the damaged one reads");

    assert!(
        before.len() > 1000,
        "the chapter before the damaged one holds its text"
    );
}

/// The screen of a chapter that gave a fault holds the sentence of that fault.
///
/// The test draws the real widget of the reader into a `Buffer`, with no
/// terminal at all, and it does the work of the loop of the screen: it takes
/// the answer of the task and it draws, for 200 frames. The fault of T-277
/// gives `Reading…` at every one of them.
///
/// The parts of this test stay in one function: the frames of the loop are one
/// measurement.
#[tokio::test]
async fn the_screen_of_a_chapter_that_gave_a_fault_says_why() {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    let directory = tempfile::tempdir().expect("the disk gives a directory");
    let path = a_book_with_one_file_damaged(
        directory.path(),
        "OEBPS/6260297267691793459_11-h-1.htm.html",
    );

    let mut reader = toutui::logic::reader::Reader::open(&path, "an item of no server")
        .expect("the reader opens the book");

    reader.go_to_chapter(THE_CHAPTER_OF_THE_MEASUREMENT);

    // A width of 100 columns is the width of the text of the reader
    // (`MAX_TEXT_WIDTH`), and the sentence of this fault is longer than one line
    // of it.
    let area = Rect::new(0, 0, 100, 30);
    let mut words = String::new();

    for _ in 0..200 {
        let mut buffer = Buffer::empty(area);

        reader.take_the_answer();
        toutui::ui::reader_tui::render(&mut reader, area, &mut buffer);

        words = the_words_of(&buffer);

        if !words.contains("Reading") {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    assert!(
        words.contains("The book gave no text of this chapter"),
        "the screen says why the chapter gave no text: {words}"
    );
    assert!(
        words.contains("corrupt deflate stream"),
        "the screen holds the reason of the machine: {words}"
    );
    // **The screen names the three keys of the view of the reader** (T-286).
    // The wrap of the paragraph puts a sentence of this length on three rows,
    // and `the_words_of` gives the whole of it as one line.
    for key in [
        "Press n for the chapter after this one",
        "or p for the chapter before it",
        "Press h to leave the book",
    ] {
        assert!(
            words.contains(key),
            "the screen names the key {key} of the view of the reader: {words}"
        );
    }

    // The message stays. The fault of T-277 wrote `Reading…` over it in the
    // frame after it.
    for _ in 0..20 {
        let mut buffer = Buffer::empty(area);

        reader.take_the_answer();
        toutui::ui::reader_tui::render(&mut reader, area, &mut buffer);

        assert!(
            !the_words_of(&buffer).contains("Reading"),
            "the render of a chapter that gave a fault does not start again"
        );

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
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
