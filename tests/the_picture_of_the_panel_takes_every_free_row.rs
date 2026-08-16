//! The picture of the panel 5 takes every row that the facts and the
//! description leave. See T-330.3.
//!
//! **The report of the maintainer of 2026-08-16, the part 3.** The panel 5 gave
//! the picture a **share** of its height, therefore a tall panel held a small
//! picture over rows of nothing at all.
//!
//! **The measurement of the real program v0.8.161 inside tmux**, of 160 columns
//! and 60 rows, of `Alice in Wonderland` of the library `Books` of the sandbox.
//! The panel held 27 rows inside its border:
//!
//! ```text
//! │              ▄▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀              │   the picture: 14 rows
//! │              ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀              │
//! │Author    Lewis Carroll                         │   the facts: 8 rows
//! │Progress  0%, Not finished                      │
//! │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
//! │No description available                        │   the description: 5 rows
//! │                                                │   ← four rows of nothing
//! │                                                │
//! │                                                │
//! │                                                │
//! ```
//!
//! **The corrected program of the same harness**: the picture takes 18 rows,
//! the facts take 8, the description of one line takes 1, and **no row of the
//! panel holds nothing at all**.
//!
//! ```text
//! │           ▄▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀           │   the picture: 18 rows
//! │           ▀▀▀▀▀▄▄▀▀▀▄▀▀▀▄▄▀▄▀▀▀▀▀▀▀▀           │
//! │Author    Lewis Carroll                         │   the facts: 8 rows
//! │Progress  0%, Not finished                      │
//! │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
//! │No description available                        │   the description: 1 row
//! └────────────────────────────────────────────────┘
//! ```
//!
//! **The control of the same run**, of 160 columns and 45 rows: the panel of 20
//! rows held a picture of 10 rows, 8 rows of the facts, and a description of 2
//! rows for a text of one line; the corrected program holds a picture of 11
//! rows and a description of 1 row, and the rows of the facts do not move.
//!
//! **A description of many lines does not take the picture away**: the picture
//! keeps `THE_SMALLEST_PICTURE` rows, and the keys `J` and `K` scroll the text
//! of the description (`crate::logic::the_scroll_of_a_panel`).

use ratatui::layout::Rect;
use toutui::ui::cover::MIN_HEIGHT_FOR_COVER;
use toutui::ui::the_panel_of_the_cover::{
    the_parts_of_the_panel, THE_SMALLEST_PANEL_OF_THE_WORDS, THE_SMALLEST_PICTURE,
};

/// The picture takes every row that the facts and the description leave.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_picture_takes_every_row_that_the_words_leave() {
    // The panel of the measurement: 27 rows, 8 rows of the facts, and a
    // description of one line.
    let inside = Rect::new(111, 3, 48, 27);
    let parts = the_parts_of_the_panel(inside, true, 8, 1);

    let cover = parts.cover.expect("a picture comes");

    assert_eq!(
        cover.height, 18,
        "the picture takes the 27 rows of the panel, less the 8 of the facts \
         and the 1 of the description"
    );
    assert_eq!(parts.facts.height, 8);
    assert_eq!(parts.description.height, 1);

    // **No row of the panel holds nothing at all**: the four empty rows of the
    // fault came of a share of the height.
    assert_eq!(
        cover.height + parts.facts.height + parts.description.height,
        inside.height
    );

    // The gate of the report of the maintainer: a panel of 22 rows, of 8 rows
    // of the facts and 3 of a description, gives a picture of 11 rows.
    let parts = the_parts_of_the_panel(Rect::new(111, 3, 48, 22), true, 8, 3);

    assert_eq!(parts.cover.map(|of| of.height), Some(11));
    assert_eq!(parts.facts.height, 8);
    assert_eq!(parts.description.height, 3);
}

/// A share of the height gives the picture the same rows at every height, and
/// the picture of this program does not.
///
/// **A test of one panel cannot tell a share from the rule of the free rows**:
/// a share of 55 percent of 20 rows is 11 rows, and the rule of the free rows
/// gives 11 rows of that same panel too. This test therefore reads **five**
/// heights of one panel, and it holds that the picture grows one row for one
/// row of the panel, which no share of a percent does.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_picture_grows_one_row_for_one_row_of_the_panel() {
    let of_the_facts = 8;
    let of_the_description = 1;

    for height in 20..=40u16 {
        let parts = the_parts_of_the_panel(
            Rect::new(111, 3, 48, height),
            true,
            of_the_facts,
            of_the_description,
        );

        assert_eq!(
            parts.cover.map(|of| of.height),
            Some(height - of_the_facts - of_the_description),
            "the picture of a panel of {height} rows takes every free row"
        );
    }
}

/// A description of many lines keeps the picture on the screen.
///
/// **The rule of the free rows must not give the whole panel to the words**: a
/// description of the server can hold 40 lines, and a picture of no row at all
/// is the fault of T-319 turned around.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_description_of_many_lines_keeps_the_picture() {
    let inside = Rect::new(111, 3, 48, 27);
    let parts = the_parts_of_the_panel(inside, true, 8, 40);

    assert_eq!(
        parts.cover.map(|of| of.height),
        Some(THE_SMALLEST_PICTURE),
        "the picture keeps the rows of a cover, and the description scrolls"
    );
    assert_eq!(THE_SMALLEST_PICTURE, MIN_HEIGHT_FOR_COVER);

    // The facts stand before the description, and the words fill the rest.
    assert_eq!(parts.facts.height, 8);
    assert_eq!(
        parts.description.height,
        inside.height - THE_SMALLEST_PICTURE - 8
    );
}

/// The two rules of T-319 stand after this correction.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_rules_of_the_panel_of_the_cover_stand() {
    // A media that the server holds with no cover gives every row to the words.
    let inside = Rect::new(111, 3, 48, 27);
    let parts = the_parts_of_the_panel(inside, false, 8, 1);

    assert_eq!(parts.cover, None);
    assert_eq!(
        parts.facts.height + parts.description.height,
        inside.height,
        "a picture that never comes must take no row of the screen"
    );

    // A panel that is not tall holds the picture alone, and the words then stay
    // under the list.
    let parts = the_parts_of_the_panel(
        Rect::new(111, 3, 48, THE_SMALLEST_PANEL_OF_THE_WORDS - 1),
        true,
        8,
        1,
    );

    assert!(!parts.the_words_stand_here());
    assert_eq!(
        parts.cover.map(|of| of.height),
        Some(THE_SMALLEST_PANEL_OF_THE_WORDS - 1)
    );

    // A panel of no cell at all holds no part.
    let parts = the_parts_of_the_panel(Rect::default(), true, 8, 1);

    assert_eq!(parts.cover, None);
    assert!(!parts.the_words_stand_here());
}
