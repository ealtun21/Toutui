//! A character is not a column. See T-305.
//!
//! **The program measures every text of the screen with a number of
//! characters, and the screen has a number of columns.** A character of the
//! Han script, of Hiragana, or of Katakana takes **two** columns of the
//! terminal, and a mark of a combination takes none. `str::chars().count()`
//! therefore gives a number that the screen does not have, and a text that the
//! program cut to that number is wider than the room that it has. ratatui then
//! cuts it a second time, and the rules of T-299, of T-300, and of T-304 — the
//! rules that say **which** part of a text the user can spare — go away.
//!
//! The measurement of 2026-08-16, of the real program v0.8.133 inside tmux
//! against the sandbox, with the account `toutuitest` and a terminal of 40
//! columns (`COLUMNS_OF_THE_SCREEN=40` of `docs/harness/drive.sh`). The keys
//! `/`, eighteen characters of Japanese, and `Enter` of the Home view gave the
//! view of the search, and the header of it said:
//!
//! ```text
//! und nothing for "日本語日本語日本語…────
//! ```
//!
//! The title is `The server found nothing for "日本語日本語日本語日本語日本語
//! 日本語". Press / to write other words.`. `in_one_row` kept 39 characters of
//! it and it wrote the three points: 30 characters of ASCII and 9 characters of
//! Japanese, which is 40 **characters** and **49 columns**. The screen holds
//! 40, therefore ratatui took the road of a title that it must cut
//! (T-304): `offset = (49 − 40) / 2 = 4`, the area of the title is `40 − 4 =
//! 36` columns, and the right-aligned draw of it kept the last 36 columns —
//! `und nothing for "日本語日本語日本語…`, the string of the screen character
//! for character. The start that went away, `The server fo`, names the view.
//!
//! **The control of the same run**: a query of eighteen characters of ASCII, of
//! the same count of characters, gave `The server found nothing for
//! "abcdefghi…` — the whole start of the title, and the three points at the
//! end of it.
//!
//! The correction: `crate::logic::message::the_columns_of` is the one measure
//! of a text of this program, and it is the crate `unicode-width` that ratatui
//! itself measures with. The corrected program, of the same keys, said `The
//! server found nothing for "日本語日…` at 39 and at 40 columns, `The server
//! found nothing for "日本語日本…` at 41, and the whole of its title at 80.

use toutui::logic::message::{in_one_row, the_columns_of, the_rows_of_a_message};
use toutui::ui::reader_tui::the_line_that_stands;

/// Eighteen characters of Japanese, of thirty-six columns.
const WIDE: &str = "日本語日本語日本語日本語日本語日本語";

/// The title of the measurement of T-305, of the real program.
fn the_title_of_the_measurement() -> String {
    format!("The server found nothing for \"{WIDE}\". Press / to write other words.")
}

/// The parts of this test stay in one function: a box of the process holds no
/// state here, but the rule of this repository keeps one test of one module.
#[test]
fn a_text_of_one_row_never_stands_wider_than_its_row() {
    // The measure itself. A character of Japanese takes two columns, and the
    // count of the characters gives one half of the truth.
    assert_eq!(WIDE.chars().count(), 18);
    assert_eq!(the_columns_of(WIDE), 36);
    assert_eq!(the_columns_of("abcdefghijklmnopqr"), 18);

    // The title of the measurement. The old rule gave 40 characters of 49
    // columns to a screen of 40 columns.
    let title = the_title_of_the_measurement();
    assert_eq!(the_columns_of(&title), 30 + 36 + 32);

    // **The rule of T-305**: no width of a screen takes a text that is wider
    // than it. The widths of the measurement stand in this sweep, and every
    // width from 0 to twice the title with them.
    for width in 0..=u16::try_from(the_columns_of(&title) * 2).unwrap() {
        let row = in_one_row(&title, width);

        assert!(
            the_columns_of(&row) <= usize::from(width),
            "the row of {width} columns took {} columns: {row}",
            the_columns_of(&row)
        );
    }

    // **The rule of T-304 stands with it**: the text keeps its start. The
    // numbers of the corrected program of the measurement.
    assert_eq!(
        in_one_row(&title, 40),
        "The server found nothing for \"日本語日…"
    );
    assert_eq!(
        in_one_row(&title, 39),
        "The server found nothing for \"日本語日…"
    );
    assert_eq!(
        in_one_row(&title, 41),
        "The server found nothing for \"日本語日本…"
    );

    // A character of two columns that meets the last column of the row stays
    // outside the row: the row of 40 columns holds 39 of them.
    assert_eq!(the_columns_of(&in_one_row(&title, 40)), 39);
    assert_eq!(the_columns_of(&in_one_row(&title, 41)), 41);

    // A text that stands takes no three points at all.
    let whole = in_one_row(&title, 200);
    assert_eq!(whole, title);

    // **The line at the top of the reader keeps the place of the user**
    // (T-300). The title of a book comes of the server, therefore a character
    // of two columns reaches that line.
    let place = " 3/12 · 41%";

    for width in 1..=120u16 {
        let line = the_line_that_stands(WIDE, place, width);

        assert!(
            the_columns_of(&line) <= usize::from(width),
            "the line of {width} columns took {} columns: {line}",
            the_columns_of(&line)
        );

        // While one column stays for the place, the place stands whole.
        if usize::from(width) > the_columns_of(place) {
            assert!(
                line.ends_with(place),
                "the line of {width} columns lost the place of the user: {line}"
            );
        }
    }

    // **A message of a view stands on the rows that it needs** (T-299 and
    // T-302). A message names a media, therefore a character of two columns
    // reaches the count of its rows.
    assert_eq!(the_rows_of_a_message(WIDE, 36), 1);
    assert_eq!(the_rows_of_a_message(WIDE, 18), 2);
    assert_eq!(the_rows_of_a_message(&format!("{WIDE} {WIDE}"), 36), 2);

    // A message of ASCII of the same count of characters needs one row of 18
    // columns, and the message of Japanese needs two: the two counts differ,
    // therefore this measures the columns and not the characters.
    assert_eq!(the_rows_of_a_message("abcdefghijklmnopqr", 18), 1);
}

/// **`the_columns_of` is the one measure of a text of this program.** A second
/// count of the characters for a width of the screen gives the fault of T-305
/// again, therefore no file of `src/` names the crate `unicode-width` outside
/// that one function.
#[test]
fn the_columns_of_a_text_come_of_one_function() {
    let mut the_files_that_measure = Vec::new();

    for file in walk("src") {
        let text = std::fs::read_to_string(&file).unwrap();

        if text.contains("unicode_width") {
            the_files_that_measure.push(file);
        }
    }

    assert_eq!(
        the_files_that_measure,
        vec![std::path::PathBuf::from("src/logic/message.rs")],
        "the crate of the columns belongs to `the_columns_of` alone"
    );
}

/// Every file of Rust under a directory.
fn walk(directory: &str) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut rooms = vec![std::path::PathBuf::from(directory)];

    while let Some(room) = rooms.pop() {
        for entry in std::fs::read_dir(&room).unwrap() {
            let path = entry.unwrap().path();

            if path.is_dir() {
                rooms.push(path);
            } else if path.extension().is_some_and(|it| it == "rs") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}
