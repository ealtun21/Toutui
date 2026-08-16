//! The box of a message counts the ends of the lines of its text. See T-310.
//!
//! **The two counts of a wrap of this program read one loop** (T-307 and
//! T-309), and that loop counts **one** line: its documentation says that a
//! caller with an end of a line splits its text at every `\n` and adds the
//! answers. The panel of a description did that (T-306), and the box of a
//! message did **not**: `the_rows_of_a_message` gave the whole text to
//! `the_rows_of_one_line`. A `\n` is a character of no column, therefore that
//! count read the word after an end of a line as a word of the same row, and it
//! gave a number of the rows that is smaller than the number that ratatui
//! draws.
//!
//! The measurement of 2026-08-16, of the real program v0.8.138 inside tmux
//! against the sandbox on `:13399`, with the account `toutuitest` and a
//! terminal of **80** columns and 45 rows. **The data of this fault is the text
//! of the server**: it needs no proxy, no build of the fault of the source, and
//! no change of the source at all. A
//! `PATCH /api/items/a4d8b9b2-c4a4-4e80-8ed0-07662933fa71/media` gave the book
//! `A Book Of An Epub With No Container` of the library `Books` the title
//! `Alpha\nOMEGAEND`, and the row of the account took that library with
//! `sqlite3` before the start (the trap 203 and the trap 204). The key `Tab`
//! gave the Library view, and the key `D` of the first line of it downloaded
//! that book of 5220 bytes. The message of the end of that download:
//!
//! ```text
//! (the row 42)
//! (the row 43)                       "Alpha
//! (the row 44)  j/k: move  l: play or open  Tab: home/library  S-Tab: the next
//! ```
//!
//! The sentence of the program is `"Alpha\nOMEGAEND" is now available
//! offline.`, and ratatui draws it on **two** rows. The count of the program
//! said **one**, therefore the box of the message held one row, and
//! `OMEGAEND" is now available offline.` had no road at all: the user read a
//! sentence of no end, and no three points said that the program cut it.
//!
//! **The control of the same run** (the trap 206): the key `j` gave the book
//! after it, `A Book Of A Broken Epub`, whose title holds no end of a line, and
//! the key `D` of that line said
//! `"A Book Of A Broken Epub" is now available offline.` on one whole row.
//!
//! **The gate below asks ratatui.** The box of a message draws a `Paragraph`
//! with `Wrap { trim: true }`, therefore the number of the rows that such a
//! paragraph takes in a `Buffer` of a width is the one truth of this count. The
//! test draws into a buffer with no terminal at all (T-256).
//!
//! **The build of the fault**: `the_rows_of_one_line(text, usize::from(width))`
//! in the place of `the_rows_of_a_text(text, usize::from(width))` of
//! `the_rows_of_a_message` gives one row where ratatui draws two.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::widgets::{Paragraph, Wrap};

use toutui::logic::message::the_rows_of_a_message;

/// The sentence of the measurement.
const OF_THE_MEASUREMENT: &str = "\"Alpha\nOMEGAEND\" is now available offline.";

/// The sentence of the control of the same run.
const OF_THE_CONTROL: &str = "\"A Book Of A Broken Epub\" is now available offline.";

/// Gives the number of the rows that ratatui draws a text on.
///
/// The row of the last character is the number of the rows. **A text whose last
/// line holds no character therefore stands outside this measure**, and every
/// text of this gate ends with a character.
fn the_rows_of_ratatui(text: &str, width: u16) -> u16 {
    // The rows of the buffer are the most rows that this text can need: one
    // word and one end of a line each start one row at the most.
    let columns = text.chars().count() * 2;
    let words = text.split_whitespace().count() + text.matches('\n').count();
    let height = u16::try_from(columns / usize::from(width) + words + 2).unwrap_or(u16::MAX);
    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);

    Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .render(area, &mut buffer);

    let mut rows = 0u16;

    for row in 0..area.height {
        let holds_a_character = (0..area.width).any(|column| buffer[(column, row)].symbol() != " ");

        if holds_a_character {
            rows = row + 1;
        }
    }

    rows
}

/// **The parts of this test stay in one function**: two test functions of one
/// module fight for the slot of that module (T-144 and T-157).
#[test]
fn the_count_of_the_rows_of_a_message_holds_the_ends_of_the_lines() {
    // The sentence of the measurement, and the sentence of its control.
    for text in [OF_THE_MEASUREMENT, OF_THE_CONTROL] {
        for width in 20u16..=200 {
            assert_eq!(
                the_rows_of_a_message(text, width),
                the_rows_of_ratatui(text, width),
                "the rows of the message of the width {width}: {text:?}"
            );
        }
    }

    // Every place of an end of a line, and every number of them: at the start of
    // a word, after a word, between two ends of a line, and with the spaces of a
    // text that wraps too. **A line of no character takes a row of its own**,
    // therefore every text below ends with a word.
    let of_the_words = "Removed the local copy of a book of the library of the user.";
    let words: Vec<&str> = of_the_words.split(' ').collect();

    for at in 0..words.len() {
        for number in [1usize, 2, 3] {
            let mut text = words.clone().join(" ");
            let end = "\n".repeat(number);

            // The end of a line comes in the place of the space after the word
            // `at`, and the last word keeps its place.
            if at + 1 < words.len() {
                text = format!(
                    "{}{}{}",
                    words[..=at].join(" "),
                    end,
                    words[at + 1..].join(" ")
                );
            }

            for width in [20u16, 27, 34, 41, 55, 62, 80, 111, 160] {
                assert_eq!(
                    the_rows_of_a_message(&text, width),
                    the_rows_of_ratatui(&text, width),
                    "the rows of the width {width}: {text:?}"
                );
            }
        }
    }

    // A message of no character and a width of 0 hold no row (T-299).
    assert_eq!(the_rows_of_a_message("\n\n", 80), 0);
    assert_eq!(the_rows_of_a_message(OF_THE_MEASUREMENT, 0), 0);
}
