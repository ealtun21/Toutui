//! The panel of a description counts the columns of its text. See T-306.
//!
//! **A character is not a column** (T-305), and T-305 corrected the three
//! functions that make a text of one row. `the_number_of_the_lines` of
//! `src/logic/the_scroll_of_a_panel.rs` stayed with `str::chars().count()`:
//! it is the one road to the length of the text of a panel (T-252), therefore
//! the largest scroll of the key `J` (T-252) and the bar of the scroll of the
//! panel (T-253) each come of that number. **A description of the Han script,
//! of Hiragana, or of Katakana takes twice the rows that the program counts**,
//! and the panel then says that it holds a text that the screen cut.
//!
//! The measurement of 2026-08-16, of the real program v0.8.134 inside tmux
//! against the sandbox, with the account `toutuitest` and a terminal of 80
//! columns. **The data of this fault is the text of the server**, therefore it
//! needs no proxy and no book of a harness: a `PATCH /api/items/:id/media`
//! gave `Alice in Wonderland` a description of 118 words of nine characters of
//! Japanese (`日本語書籍説明文段`, eighteen columns) and the word
//! `THEENDOFTHETEXT` after them. The keys `/`, `Alice`, and `Enter` of the
//! Home view gave the view of the search, and its panel of eighteen rows drew
//! **four** words of every row:
//!
//! ```text
//! 日本語書籍説明文段 日本語書籍説明文段 日本語書籍説明文段 日本語書籍説明文段
//! ```
//!
//! The text takes **30** rows of that panel, and the program counted **17**:
//! at a width of 78 columns the old rule fitted seven words of nine characters
//! in a row. Seventeen is fewer than the eighteen rows of the panel, therefore
//! `the_last_scroll` gave 0, **no bar of the scroll came**, and **forty presses
//! of the key `J` changed no character of the screen**. The word
//! `THEENDOFTHETEXT` stood on the row 30, and no key of the program reached it.
//!
//! **The control of the same run**: the same panel, with a description of 118
//! words of eighteen characters of ASCII (`ABCDEFGHIJKLMNOPQR`) and the same
//! word after them — the same number of columns. The bar of the scroll came at
//! once, the key `J` moved the panel, and `THEENDOFTHETEXT` stood on the last
//! row of it.
//!
//! **The gate below asks ratatui.** The panels of this program draw a
//! `Paragraph` with `Wrap { trim: true }`, therefore the number of the rows
//! that such a paragraph takes in a `Buffer` of a width is the one truth of
//! this count. The test draws into a buffer with no terminal at all (T-256),
//! and it asks for every text of the measurement at every width of a screen
//! that this fork measures.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::widgets::{Paragraph, Wrap};

use toutui::logic::the_scroll_of_a_panel::{the_last_scroll, the_number_of_the_lines};

/// One word of the description of the measurement: nine characters of
/// Japanese, of eighteen columns.
const WORD: &str = "日本語書籍説明文段";

/// The word of ASCII of the control, of eighteen characters and eighteen
/// columns.
const WORD_OF_ASCII: &str = "ABCDEFGHIJKLMNOPQR";

/// The last word of the description of the measurement. The user of the fault
/// never read it.
const THE_END: &str = "THEENDOFTHETEXT";

/// The description of the measurement of T-306, of `count` words.
fn the_description(word: &str, count: usize) -> String {
    let mut words = vec![word; count];
    words.push(THE_END);

    words.join(" ")
}

/// The number of the rows that ratatui takes for this text in a panel of
/// `width` columns.
///
/// The panels of this program draw with `Wrap { trim: true }`, therefore this
/// is the number that the screen of the user has. The buffer holds many more
/// rows than the text needs, and the count is the row after the last row that
/// holds a character.
///
/// **A text of one column of width takes one row of every column of it**,
/// therefore the buffer holds the columns of the whole text and the rows that
/// its own ends of a line need.
fn the_rows_of_ratatui(text: &str, width: u16) -> usize {
    let room = text.chars().count() + text.split('\n').count() + 4;
    let area = Rect::new(0, 0, width, u16::try_from(room).unwrap());
    let mut buffer = Buffer::empty(area);

    Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .render(area, &mut buffer);

    let mut rows = 0usize;

    for row in 0..area.height {
        let holds_a_character =
            (0..area.width).any(|column| buffer[(column, row)].symbol().trim() != "");

        if holds_a_character {
            rows = usize::from(row) + 1;
        }
    }

    rows
}

/// The parts of this test stay in one function: the rule of this repository
/// keeps one test of one module.
#[test]
fn the_lines_of_a_panel_are_the_rows_of_the_screen() {
    // The measure itself. The word of Japanese and the word of ASCII hold the
    // same number of columns, and a different number of characters.
    assert_eq!(WORD.chars().count(), 9);
    assert_eq!(WORD_OF_ASCII.chars().count(), 18);

    let of_japanese = the_description(WORD, 118);
    let of_ascii = the_description(WORD_OF_ASCII, 118);

    // **The numbers of the measurement of T-306.** The panel of the view of
    // the search of a terminal of 80 columns holds 78 columns of text and 18
    // rows.
    assert_eq!(the_number_of_the_lines(&of_japanese, 78), 30);
    assert_eq!(the_rows_of_ratatui(&of_japanese, 78), 30);

    // The old rule counted 17 rows, which is fewer than the 18 rows of the
    // panel: the bar of the scroll did not come, and the key `J` moved
    // nothing. The corrected program leaves 12 rows of the text under the
    // panel.
    assert_eq!(the_last_scroll(&of_japanese, 78, 18), 12);

    // The control of the measurement: the text of ASCII of the same columns
    // takes the same rows, and it took them before this correction too.
    assert_eq!(the_number_of_the_lines(&of_ascii, 78), 30);
    assert_eq!(the_last_scroll(&of_ascii, 78, 18), 12);

    // **The rule of T-306**: the number of the lines of a text is the number
    // of the rows that ratatui draws it on.
    //
    // The sweep takes the short form of the two descriptions: a buffer of a
    // width of two columns holds one character of every row, therefore the
    // whole description of the measurement costs 1070 rows of every width.
    //
    // **The words of the sweep are shorter than the narrowest width of it.**
    // A word that is longer than the panel takes a road of ratatui that
    // overflows the area (the item T-306 of `docs/TAKEOVER-BACKLOG.md` holds
    // the numbers): a `Paragraph` of `Wrap { trim: true }` of 3 columns drew
    // two characters of the Han script on one row, which is four columns.
    // That is the arithmetic of the crate and not of this program, and no
    // count of this program says what it does.
    let short_of_japanese = the_description(WORD, 12);
    let short_of_ascii = the_description(WORD_OF_ASCII, 12);
    let mixed = format!("{WORD} {WORD_OF_ASCII}\n\n{WORD} a 日 {WORD_OF_ASCII}");

    let the_texts = [
        short_of_japanese.as_str(),
        short_of_ascii.as_str(),
        mixed.as_str(),
        WORD,
        WORD_OF_ASCII,
        "",
        "a",
        "日",
    ];

    // The widths that are not even stand in this sweep beside the widths that
    // are: **a character of two columns that meets the last column of a row
    // stays outside that row** (T-305), because a half of a character is no
    // character. The width of 40 columns is the narrowest terminal that this
    // fork measures (T-301), and the sweep starts under it.
    for text in the_texts {
        for width in 20..=200u16 {
            assert_eq!(
                the_number_of_the_lines(text, width),
                the_rows_of_ratatui(text, width).max(1),
                "the panel of {width} columns counted the characters of: {text}"
            );
        }
    }
}
