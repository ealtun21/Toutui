//! The header and the contents of the reader keep one line each. See T-313.
//!
//! **The data of this fault is a book**, and it needs no proxy, no build of the
//! fault of the source, and no change of the sandbox at all. The `dc:title` of
//! an EPUB and the name of a chapter of the nav of it hold the text of the
//! maker of the book. **A literal end of a line in them does not reach the
//! program**: the parser of the XML normalizes the whitespace of the content of
//! an element. **A character reference of `&#10;` keeps that end**, because a
//! parser of XML gives a character reference its character and it does not
//! normalize it (XML 1.0, section 2.11 and section 3.3.3).
//!
//! `docs/harness/a_book_of_an_end_of_a_line.py` writes that book, and the same
//! file stands at `tests/data/hostile/15-a-book-of-an-end-of-a-line.epub`.
//!
//! The measurement of 2026-08-16, of the real program v0.8.141 inside tmux
//! against the sandbox on `:13399` with the account `toutuitest`, of a terminal
//! of **80** columns and 45 rows. The book went into the cache of the ebooks of
//! that account under the name of the item of `Alice in Wonderland`
//! (`8fda6e43-0728-46ad-98bc-4c8634e299ad.epub`), and the keys `Tab`, 15 keys
//! `j`, and `e` gave the two faults.
//!
//! **1. The header.** `line_of_the_top` of `src/ui/reader_tui.rs` draws in a
//! `Paragraph` of **one** row with no wrap, at `Constraint::Length(1)` of the
//! layout of the view. The row 3 of the screen held
//!
//! ```text
//! Alpha
//! ```
//!
//! and the number of the chapter, the count of the chapters, and the percent of
//! the place of the user stood on a second line that falls outside the area.
//!
//! **2. The contents.** The key `t` then gave the panel of the table of
//! contents, and it held **four** rows for a book of **three** chapters:
//!
//! ```text
//! │➤   One                                                                       │
//! │    Beta                                                                      │
//! │  GAMMAEND                                                                    │
//! │    Three                                                                     │
//! ```
//!
//! The row `GAMMAEND` names a chapter that the book does not hold, it carries
//! no space of the depth and no sign `➤` of the cursor, and the key `j` of the
//! line after it therefore stands on a row that the user does not see. That is
//! the rule of T-311 for a `List` of ratatui.
//!
//! **The control of the same run**: the same book of the same keys, with the
//! title `AlphaPLACEHOLDEROMEGAEND` and the name of the chapter `Two` of one
//! line each, gave the row 3
//! `AlphaPLACEHOLDEROMEGAEND — chapter 3 of 3 — 89%` and the three rows `One`,
//! `Two`, and `Three` of the contents.
//!
//! These tests draw the real `render` of the reader into a `Buffer` of ratatui
//! with no terminal and no screen (T-256), and they need no network and no
//! sandbox.

use std::path::PathBuf;

use ratatui::{buffer::Buffer, layout::Rect};
use toutui::logic::reader::Reader;
use toutui::ui::reader_tui::render;

/// The screen of the measurement: 80 columns and 45 rows.
const WIDTH: u16 = 80;
const HEIGHT: u16 = 45;

/// The header of the reader stands on the first row of its area, and it holds
/// one row. `render` of `src/ui/reader_tui.rs` reads the same number.
const THE_ROW_OF_THE_HEADER: u16 = 0;

/// The book of this repository. Its `dc:title` is `Alpha&#10;OMEGAEND`, and the
/// name of its second chapter is `Beta&#10;GAMMAEND`.
fn the_book() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/hostile/15-a-book-of-an-end-of-a-line.epub")
}

/// Opens the book and draws the reader into a buffer of ratatui.
///
/// The render of the text of a chapter asks a task of tokio for the lines of
/// it (`src/logic/reader/session.rs`), therefore the call stands inside a
/// runtime. The header and the panel of the contents need no line of the text:
/// the render draws them at the first frame, and the words `Reading…` then
/// stand in the place of the text.
fn the_screen_of_the_reader(contents_open: bool) -> Buffer {
    let mut reader = Reader::open(&the_book(), "the-item-of-the-test")
        .expect("the book opens: an end of a line of a title is no fault of the open");

    assert_eq!(
        "Alpha\nOMEGAEND",
        reader.title.as_str(),
        "the character reference of the title reaches the program"
    );

    reader.contents_open = contents_open;

    let area = Rect::new(0, 0, WIDTH, HEIGHT);
    let mut buffer = Buffer::empty(area);
    // **The footer of the reader stands under the band of the player** (T-343),
    // therefore the caller gives its area beside the area of the book.
    let of_the_book = Rect::new(0, 0, WIDTH, HEIGHT.saturating_sub(2));
    let of_the_footer = Rect::new(0, HEIGHT.saturating_sub(2), WIDTH, 2);
    render(&mut reader, of_the_book, of_the_footer, &mut buffer);
    buffer
}

/// Gives the text of one row of the buffer, with no space of the end.
fn the_row(buffer: &Buffer, row: u16) -> String {
    let mut text = String::new();

    for column in 0..WIDTH {
        text.push_str(buffer[(column, row)].symbol());
    }

    text.trim_end().to_string()
}

/// The header of the reader holds one row, therefore the whole title and the
/// whole place of the user stand on that row.
///
/// The parts of this test stay in one function: the row of the header, the
/// title in it, and the place of the user after it are one measurement of one
/// row.
#[tokio::test]
async fn the_header_of_the_reader_holds_the_title_and_the_place_on_one_row() {
    let buffer = the_screen_of_the_reader(false);
    let header = the_row(&buffer, THE_ROW_OF_THE_HEADER);

    assert!(
        header.contains("Alpha OMEGAEND"),
        "the row of the header holds the whole title: {header:?}"
    );

    assert!(
        header.contains("chapter 1 of 3"),
        "the row of the header holds the number of the chapter and the count of \
         the chapters: {header:?}"
    );

    assert!(
        !header.contains('\n'),
        "the row of the header holds no end of a line: {header:?}"
    );

    // The row after the header belongs to the text of the book, and no part of
    // the title of the header may stand in it.
    let after = the_row(&buffer, THE_ROW_OF_THE_HEADER + 1);

    assert!(
        !after.contains("OMEGAEND"),
        "the row after the header holds no part of the title: {after:?}"
    );
}

/// A `List` of ratatui gives one `ListItem` the rows of the ends of the lines
/// of its text. The panel of the contents of a book of three chapters must
/// therefore hold three rows of a name, and the name of the second chapter must
/// stand whole on its own row.
///
/// The parts of this test stay in one function: the three rows of the panel and
/// the name of the second chapter in one of them are one measurement of one
/// panel.
#[tokio::test]
async fn the_contents_of_the_reader_hold_one_row_of_each_chapter() {
    let buffer = the_screen_of_the_reader(true);

    // The panel of the contents stands under the header, and the border of it
    // takes the row after the header. The three names then follow.
    let rows: Vec<String> = (0..HEIGHT).map(|row| the_row(&buffer, row)).collect();

    let of_the_second = rows
        .iter()
        .position(|row| row.contains("Beta"))
        .expect("the panel of the contents names the second chapter");

    assert!(
        rows[of_the_second].contains("Beta GAMMAEND"),
        "the row of the second chapter holds its whole name: {:?}",
        rows[of_the_second]
    );

    assert!(
        rows[of_the_second + 1].contains("Three"),
        "the row after the second chapter names the third chapter: {:?}",
        rows[of_the_second + 1]
    );

    let of_the_end = rows.iter().filter(|row| row.contains("GAMMAEND")).count();

    assert_eq!(
        1, of_the_end,
        "the name of the second chapter stands on one row alone"
    );

    // A book of three chapters gives three names, and no row of the panel names
    // a chapter that the book does not hold.
    let names = ["One", "Beta GAMMAEND", "Three"];

    for name in names {
        assert_eq!(
            1,
            rows.iter().filter(|row| row.contains(name)).count(),
            "the panel of the contents holds one row of {name:?}"
        );
    }
}
