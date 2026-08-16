//! The row of the message of a view stands on the rows that it needs. See
//! T-299, and the same rule of the login screen in T-297.
//!
//! **The measurement of 2026-08-16, of the real program v0.8.127 inside tmux
//! against the sandbox**: the database held `toutuitest` (11 copies of the disk,
//! and 239.7 MB) and `toutuilimited`, and `toutuitest` was the account of the
//! start. The keys `S`, `Enter`, `l`, and `l` logged out of it, the program
//! started again with `toutuilimited` (the road
//! `AfterALogOut::ThisAccountStarts`), and the Home view of that account said,
//! in a terminal of **160** columns:
//!
//! ```text
//! The program removed the account toutuitest. The disk keeps the copies of that account: 11 media, and 239.7 MB. Log in again with the same name and the same…
//! ```
//!
//! The road back of that sentence stood outside the screen. The row of the
//! message of every view held one row, and it cut every sentence that was
//! longer than the width of the terminal. A terminal of 80 columns loses much
//! more.
//!
//! The place of the message and the text of it are pure, therefore these tests
//! need no terminal and no `App`.

use toutui::logic::message::{in_the_rows, the_place_of_a_message, the_rows_of_a_message};

/// The number of rows of the header of every view, of `src/ui/tui.rs`.
const HEADER: u16 = 2;

/// The number of rows of the footer of every view, of `src/ui/tui.rs`.
const FOOTER: u16 = 2;

/// The sentence of the measurement above, of a log out that keeps the copies of
/// the disk (T-297).
const THE_WORDS_OF_THE_LOG_OUT: &str =
    "The program removed the account toutuitest. The disk keeps the copies of that account: \
     11 media, and 239.7 MB. Log in again with the same name and the same server: the key X \
     then removes a copy.";

/// The sentence of the measurement is longer than the screen, and the rows of
/// the message hold the whole of it.
#[test]
fn the_sentence_of_a_log_out_takes_the_rows_that_it_needs() {
    assert!(
        THE_WORDS_OF_THE_LOG_OUT.chars().count() > 160,
        "the sentence of the measurement must be longer than the screen"
    );

    // A terminal of 160 columns needs two rows, and one of 80 needs three.
    assert_eq!(the_rows_of_a_message(THE_WORDS_OF_THE_LOG_OUT, 160), 2);
    assert_eq!(the_rows_of_a_message(THE_WORDS_OF_THE_LOG_OUT, 80), 3);

    // The whole sentence stands in those rows, and no three points cut it.
    for (width, rows) in [(160u16, 2u16), (80, 3)] {
        let text = in_the_rows(THE_WORDS_OF_THE_LOG_OUT, width, rows);

        assert_eq!(
            text, THE_WORDS_OF_THE_LOG_OUT,
            "the screen of {width} columns cut the sentence"
        );
        assert!(
            text.ends_with("then removes a copy."),
            "the road back of the sentence went away: {text}"
        );
    }
}

/// The last row of the message stays above the footer, and the rows before it
/// grow upward over the view.
#[test]
fn the_rows_of_a_message_grow_upward_and_they_keep_the_header() {
    // The screen of the measurement: 45 rows, and a message of two rows.
    let (y, rows) = the_place_of_a_message(0, 45, HEADER, FOOTER, 2).unwrap();
    assert_eq!((y, rows), (41, 2));
    // The last row of it stays where one row of a message stood.
    assert_eq!(y + rows - 1, 45 - FOOTER - 1);

    // A message of one row keeps the place that it had.
    assert_eq!(
        the_place_of_a_message(0, 45, HEADER, FOOTER, 1),
        Some((42, 1))
    );

    // The header of the screen keeps its rows: a message of more rows than the
    // room takes the room alone.
    let (y, rows) = the_place_of_a_message(0, 8, HEADER, FOOTER, 6).unwrap();
    assert_eq!((y, rows), (2, 4));
    assert!(y >= HEADER, "the message took a row of the header");

    // A screen that holds no row above the footer draws no message.
    assert_eq!(the_place_of_a_message(0, 2, HEADER, FOOTER, 1), None);
}

/// A message of more rows than the screen holds loses its end, and the three
/// points then say that the screen cut it.
#[test]
fn a_message_of_more_rows_than_the_screen_says_that_it_lost_its_end() {
    let text = in_the_rows(THE_WORDS_OF_THE_LOG_OUT, 80, 1);

    assert!(text.ends_with('…'), "{text}");
    assert!(text.starts_with("The program removed"), "{text}");
    assert_eq!(the_rows_of_a_message(&text, 80), 1);

    // A message of no row and a screen of no width draw nothing.
    assert_eq!(in_the_rows(THE_WORDS_OF_THE_LOG_OUT, 80, 0), "");
    assert_eq!(in_the_rows(THE_WORDS_OF_THE_LOG_OUT, 0, 3), "");
}
