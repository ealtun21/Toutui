//! The mark of a line and the panel of that same line say one thing of the end
//! of a media. See T-290.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! **A percent of 100 is not the mark of the end.** The field `isFinished` of
//! the row of the server is the one truth of a media that the user finished,
//! and the percent of that same row can stand at 100 beside `Not finished`:
//! the server clamps `progress` at 1, it never takes that value down, and
//! `crate::api::me::update_media_progress` sends the place of the end in one
//! request and the mark of the end in a second one.
//!
//! The measurement of 2026-08-16, of the real program v0.8.118 against the
//! sandbox. The row of the server of the episode `Chapter 01` of the podcast
//! `Arthur Gordon Pym` held `progress: 1`, `currentTime: 300`, and
//! `isFinished: false`, and one frame of the Home view said two things of that
//! one media:
//!
//! ```text
//!   ▌ Continue Listening
//! ➤ ✓   Chapter 01
//!   89% Chapter 02
//!   3%  Letter 1
//! [Arthur Gordon Pym] - Author: LibriVox - Episode: 1 - Duration: 22m
//! Progress: 100%, 17m left, Not finished
//! ```
//!
//! The line held the mark of a media that the user finished, the panel of that
//! same line said `Not finished`, and the shelf above the two of them holds the
//! media that the user did not finish.

use toutui::logic::the_panel_of_a_line::the_place_of_the_panel;
use toutui::ui::marks;

/// The mark of a media that the user finished.
const FINISHED: &str = "✓";

#[test]
fn the_line_and_the_panel_agree_on_the_end() {
    // The row of the server of the measurement: the percent of the whole
    // length, and the field of the end that says that the user did not finish
    // the media.
    let percent = "100";
    let the_end = "Not finished";

    let mark = marks::of_progress(percent, the_end, false);
    let panel = the_place_of_the_panel(
        false,
        None,
        None,
        Some(1319.601633),
        percent,
        "17m left,",
        the_end,
    );

    // The panel says the truth of the server already.
    assert_eq!(panel.the_end, "Not finished");

    // **The line must say it too** (T-290): the mark of a media that the user
    // finished belongs to the field of the server alone.
    assert!(
        !mark.contains(FINISHED),
        "the line of a media of 100 percent that the user did not finish holds \
         the mark of the end: {:?}, and the panel of that same line says {:?}",
        mark,
        panel.the_end,
    );

    // A media that the server does call finished keeps that mark, at every
    // percent.
    assert!(marks::of_progress("100", "Finished", false).contains(FINISHED));
    assert!(marks::of_progress("42", "Finished", false).contains(FINISHED));

    // **Every mark holds the same number of columns** (T-44): a mark of a
    // percent of 100 that takes the whole of the width leaves no space between
    // itself and the title, and the titles of the list then do not stand under
    // each other.
    let width = marks::WIDTH;
    for value in ["0", "5", "50", "99", "100", "101", "-1", " N/A"] {
        let mark = marks::of_progress(value, "Not finished", false);
        assert_eq!(
            mark.chars().count(),
            width,
            "the mark of the percent {:?} is not {} columns wide: {:?}",
            value,
            width,
            mark,
        );
        assert!(
            mark.ends_with(' '),
            "the mark of the percent {:?} holds no space before the title: {:?}",
            value,
            mark,
        );
    }

    // The line of a media at the whole of its length says the number of that
    // percent, and not the mark of a media of another place.
    assert!(marks::of_progress("100", "Not finished", false).starts_with("100"));
    assert!(marks::of_progress("99", "Not finished", false).starts_with("99%"));
}
