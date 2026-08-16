//! The words of the three filters of the position say what they filter, and no
//! word more. See T-330.2, of the second report of the maintainer of
//! 2026-08-16.
//!
//! **The three labels of the start each began with fourteen columns that say
//! nothing.** Every row of a library is a media, therefore those columns tell
//! the user nothing at all, and they cost the words that do.
//!
//! **The measurement of the real program v0.8.160 inside tmux**, of the Home
//! view of the library `Large` of the sandbox. The panel 3 of the stack, at 160
//! columns:
//!
//! ```text
//! ┌3 Filter ───────────────────────┐
//! │➤ ✓ No filter                   │
//! │    The media that you finished │
//! │    The media that you started  │
//! │    The media that you did not… │
//! └────────────────────────────────┘
//! ```
//!
//! The third row is cut, and the user cannot read whether the filter gives the
//! media that they did not start or the media that they did not finish.
//!
//! The second row of the header of the same program, with that third filter on:
//!
//! ```text
//! | 100 | 🔗 localhost:13399  ⇅ The sequence of the server ▣ The media that you did not start |
//! |  84 | 🔗 localhost:13399 |
//! ```
//!
//! A row of 84 columns holds none of the words at all, because the whole of
//! them does not fit beside the address (T-329).
//!
//! **The corrected program** gives the four values `No filter`, `Finished`,
//! `Started, not finished`, and `Not started`, and every one of them stands
//! whole in the panel 3 and in the row of 84 columns.

use toutui::logic::message::the_columns_of;
use toutui::logic::sort_filter::{self, Row};
use toutui::ui::the_panels_of_the_stack::{
    the_column_of_the_words, the_lines_of_a_panel, the_name_of_a_filter, THE_GAP_OF_THE_WORDS,
};

/// The columns of a panel of the stack. The panel of the measurement above held
/// 32 of them, and the border takes two.
const THE_PANEL: u16 = 32;

/// The columns of the row of the address of the sandbox, `🔗 localhost:13399`.
const THE_ADDRESS: u16 = 18;

/// The four values that the user reads, and the sweep that holds them.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_four_words_of_the_filter_say_what_they_filter() {
    assert_eq!(the_name_of_a_filter("", &[]), "No filter");

    for (value, name) in [
        ("finished", "Finished"),
        ("in-progress", "Started, not finished"),
        ("not-started", "Not started"),
    ] {
        let of_the_place = sort_filter::filter_value("progress", value);
        assert_eq!(
            the_name_of_a_filter(&of_the_place, &[]),
            name,
            "the filter {value} says {name}"
        );
    }

    // **The sweep of this item.** The words of the start began with `The media
    // that`, and no label of the position may say those columns again: every
    // row of a library is a media already.
    for (label, value) in sort_filter::PROGRESS {
        assert!(
            !label.starts_with("The media"),
            "the label of {value} says {label}, which begins with words that say nothing"
        );
    }

    // **`Started, not finished` is not `Started`**: the filter of the server
    // gives the media that the user started and did not finish, therefore a
    // word that says less than the filter does is a word that lies.
    assert!(sort_filter::PROGRESS[1].0.contains("not finished"));
}

/// Every one of the four values stands whole in a line of the panel 3.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_panel_of_the_filter_cuts_no_word_of_the_four() {
    let mut rows = vec![Row::NoFilter];
    for one in sort_filter::progress_choices() {
        rows.push(Row::Filter {
            label: one.label.clone(),
            value: one.value.clone(),
        });
    }

    let lines = the_lines_of_a_panel(&rows, THE_PANEL, "media.metadata.title", false, "");
    assert_eq!(lines.len(), 4);

    // `in_one_row` puts the mark `…` at the end of a line that it cuts.
    for line in &lines {
        assert!(!line.contains('…'), "the panel cuts the line `{line}`");
    }

    for (line, word) in lines.iter().zip([
        "No filter",
        "Finished",
        "Started, not finished",
        "Not started",
    ]) {
        assert!(line.contains(word), "the line `{line}` says `{word}`");
    }
}

/// The second row of the header of 84 columns holds the words of the longest of
/// the four beside the address of the server.
///
/// The words of the start took 60 columns of that row, and the row then held
/// none of them (T-329).
///
/// **The parts of this test stay in one function.**
#[test]
fn the_header_of_84_columns_holds_the_longest_of_the_four() {
    // The longest row of the words: the sequence of the server, which is the
    // value of the start of every account, and the longest of the four filters.
    let words = format!(
        "⇅ The sequence of the server ▣ {}",
        sort_filter::PROGRESS[1].0
    );

    // **The mark takes three bytes and one column** (the trap 245).
    let columns = u16::try_from(the_columns_of(&words)).expect("a header holds few columns");
    assert_eq!(columns, 52);

    let at_84 = the_column_of_the_words(84, THE_ADDRESS, 0, columns)
        .expect("84 columns hold the longest of the four beside the address");
    assert_eq!(at_84, THE_ADDRESS + THE_GAP_OF_THE_WORDS);
    assert!(
        u32::from(at_84) + u32::from(columns) <= 84,
        "the words end inside the row: {at_84} + {columns}"
    );

    // The words of the start of the same row took 32 columns more, and 84
    // columns held none of them.
    let of_the_start = u16::try_from(the_columns_of(
        "⇅ The sequence of the server ▣ The media that you did not start",
    ))
    .expect("a header holds few columns");
    assert_eq!(
        the_column_of_the_words(84, THE_ADDRESS, 0, of_the_start),
        None
    );
}
