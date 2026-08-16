//! The box of a message counts the columns of its text. See T-307.
//!
//! **A character is not a column** (T-305), and T-306 gave that rule to
//! `the_number_of_the_lines` of `src/logic/the_scroll_of_a_panel.rs`, which
//! counts the lines of the panel of a description. `the_rows_of_a_message` of
//! `src/logic/message.rs` is the **second** count of a wrap of this program:
//! the box of the message of a view stands on the rows that this count gives
//! (T-299), and `in_the_rows` cuts the end of a message that needs more rows
//! than the screen holds. That count measured with `the_columns_of` already,
//! and it did **not** hold the rule of the last character of a row (T-306):
//! ratatui draws the last character of a row while one column of that row
//! stays. A message of the Han script, of Hiragana, or of Katakana therefore
//! said that it needs one row more than the render takes.
//!
//! The measurement of 2026-08-16, of the real program v0.8.135 inside tmux
//! against the sandbox, with the account `toutuitest` and a terminal of **40**
//! columns. **The data of this fault is the text of the server**, therefore it
//! needs no proxy and no build of the fault of the source: a
//! `PATCH /api/items/040e9d69-…/media` gave a book of the library `Books` the
//! title `日本語書籍説明 日本語書籍説明 日本語書籍説明` (three words of seven
//! characters, of fourteen columns each). The keys `/`, `日本語書籍説明`, and
//! `Enter` gave the view of the search, the key `D` wrote the row of the
//! download with that title, and the key `X` said:
//!
//! ```text
//! (the row 39)
//! (the row 40)  Removed the local copy of "日本語書籍説
//! (the row 41)       日本語書籍説明 日本語書籍説明".
//! (the row 42)
//! (the row 43) j/k: move  l: play or open  h: back  /:
//! ```
//!
//! The count of the program said **three** rows and the render of ratatui took
//! **two**: the box of the message therefore stood on the rows 40, 41, and 42,
//! its last row held no character, and **the row 39 of the list went away for
//! nothing**. The last row of a message stands above the footer (T-299), and
//! that message stood one row over it.
//!
//! **The control of the same run**: the same book, the same keys, and the title
//! `ABCDEFGHIJKLMN ABCDEFGHIJKLMN ABCDEFGHIJKLMN` — three words of fourteen
//! characters, of the same fourteen columns. The message took the rows 40, 41,
//! and 42, every row of it held characters, and its last row stood above the
//! footer.
//!
//! **The gate below asks ratatui.** The box of a message draws a `Paragraph`
//! with `Wrap { trim: true }`, therefore the number of the rows that such a
//! paragraph takes in a `Buffer` of a width is the one truth of this count. The
//! test draws into a buffer with no terminal at all (T-256), and it asks for
//! the sentences of the measurement at every width from 20 to 120 columns, and
//! for words of the Han script of every length, for the words of the control of
//! ASCII, and for the two spaces of a footer (T-302), at the widths of 20 to
//! 200 columns of a step of seven.
//!
//! **The build of the fault**: `column + spaces + length` in the place of
//! `column + spaces + room` of `the_rows_of_a_message` gives three rows where
//! ratatui draws two.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::widgets::{Paragraph, Wrap};

use toutui::logic::message::the_rows_of_a_message;

/// The characters of the title of the measurement.
const OF_THE_HAN: &str = "日本語書籍説明文段落章節版";

/// The characters of the control of the same run, of two columns for one
/// character of the Han script.
const OF_ASCII: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";

/// Gives the number of the rows that ratatui draws a text on.
///
/// The box of a message draws this same paragraph, therefore this number is the
/// one truth of `the_rows_of_a_message`.
fn the_rows_of_ratatui(text: &str, width: u16) -> u16 {
    // The rows of the buffer are the most rows that this text can need: one
    // word starts one row at the most, and a word that is longer than the width
    // takes the columns of the text. **A buffer that is shorter than the text
    // cuts the render**, and a buffer of a fixed height of 250 rows made this
    // gate take 455 seconds.
    let columns = text.chars().count() * 2;
    let words = text.split_whitespace().count();
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
fn the_count_of_the_rows_of_a_message_is_the_count_of_ratatui() {
    let of_the_han: Vec<char> = OF_THE_HAN.chars().collect();

    // The sentence of the measurement, and the sentence of its control.
    let of_the_measurement = [
        "Removed the local copy of \"日本語書籍説明 日本語書籍説明 日本語書籍説明\".",
        "Removed the local copy of \"ABCDEFGHIJKLMN ABCDEFGHIJKLMN ABCDEFGHIJKLMN\".",
    ];

    for text in of_the_measurement {
        for width in 20u16..=120 {
            assert_eq!(
                the_rows_of_a_message(text, width),
                the_rows_of_ratatui(text, width),
                "the rows of the message of the width {width}: {text}"
            );
        }
    }

    // Every length of a word, every number of the words, and the three forms of
    // a text: the Han script alone, ASCII alone, and the two of them together.
    for length in 1..=8usize {
        let of_the_han: String = of_the_han[..length.min(of_the_han.len())].iter().collect();
        let of_ascii = &OF_ASCII[..(length * 2).min(OF_ASCII.len())];

        for number in [1usize, 2, 3, 5, 8] {
            let of_the_han = vec![of_the_han.clone(); number].join(" ");
            let of_ascii = vec![of_ascii; number].join(" ");
            let of_the_two = format!("{of_the_han} {of_ascii} {of_the_han}");

            for name in [&of_the_han, &of_ascii, &of_the_two] {
                for form in [
                    "{}",
                    "This program downloads \"{}\" now.",
                    "The program shows the library \"{}\" now.",
                    // **The spaces between two words keep their width**
                    // (T-302): every footer of this program holds two of them.
                    "a {} b  c   {} d",
                ] {
                    let text = form.replace("{}", name);

                    for width in (20u16..=200).step_by(7) {
                        assert_eq!(
                            the_rows_of_a_message(&text, width),
                            the_rows_of_ratatui(&text, width),
                            "the rows of the width {width}: {text}"
                        );
                    }
                }
            }
        }
    }
}
