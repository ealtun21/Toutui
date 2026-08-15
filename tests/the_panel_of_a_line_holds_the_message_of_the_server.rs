//! The panel of a line holds the place of a live message of the server.
//! See T-240.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! The measurement of 2026-08-15 against the sandbox: the server held
//! `A Book Of Many Hours` at 10800 seconds of 28800 with the percent 52, the
//! user opened the Home view, and a second client of that same account moved
//! that media to 21600 seconds with the percent 75. The line of the media took
//! the message at the next frame (T-47), and the panel of that same line kept
//! the answer of the request of the start:
//!
//! ```text
//! ➤ 75% A Book Of Many Hours
//! Author: Many Hours Author - Year: N/A - Duration: 8h
//! Progress: 52%, 5h left, Not finished
//! ```

use toutui::api::live::Progress;
use toutui::logic::the_panel_of_a_line::the_place_of_the_panel;

/// The row of a message, in the shape of `crate::api::live::Progress`.
fn a_message(percent: &str, finished: &str, place: &str) -> Progress {
    Progress {
        percent: percent.to_string(),
        finished: finished.to_string(),
        place: place.to_string(),
    }
}

#[test]
fn the_panel_of_a_line_holds_the_message_of_the_server() {
    // The measurement itself: no playback of this program holds the media, and
    // the message of the server names 21600 seconds of 28800.
    let live = a_message("75", "Not finished", "21600");
    let panel = the_place_of_the_panel(
        false,
        None,
        Some(&live),
        Some(28800.0),
        "52",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "75");
    assert_eq!(panel.the_time_that_is_left, "2h left,");
    assert_eq!(panel.the_end, "Not finished");

    // **A media with no message keeps the row of the request**: the message
    // carries the whole account, and a media that stands in no row of it holds
    // the value of the request of the view (T-184).
    let panel = the_place_of_the_panel(
        false,
        None,
        None,
        Some(28800.0),
        "52",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "52");
    assert_eq!(panel.the_time_that_is_left, "5h left,");

    // **The place of the engine of this program wins over the message**
    // (T-239): the server sends no message of a place to the client that wrote
    // it (T-235), therefore a message of a media that plays here is older than
    // the engine.
    let live = a_message("75", "Not finished", "21600");
    let panel = the_place_of_the_panel(
        true,
        Some(15192.0),
        Some(&live),
        Some(28800.0),
        "52",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "53");
    assert_eq!(panel.the_time_that_is_left, "3h47m left,");

    // **A place of 0 of the engine is a playback that did not begin** (T-238),
    // and the message is then the newest value that the program holds.
    let live = a_message("75", "Not finished", "21600");
    let panel = the_place_of_the_panel(
        true,
        Some(0.0),
        Some(&live),
        Some(28800.0),
        "52",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "75");
    assert_eq!(panel.the_time_that_is_left, "2h left,");

    // **The mark of the end of the message belongs to the message**: a second
    // client of the account finished the media, and the panel says it.
    let live = a_message("100", "Finished", "28800");
    let panel = the_place_of_the_panel(
        false,
        None,
        Some(&live),
        Some(28800.0),
        "52",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "100");
    assert_eq!(panel.the_end, "Finished");
    assert_eq!(panel.the_time_that_is_left, "0m left,");

    // **A place of 0 of a message is the start of the media** (T-234): the line
    // of the view of the queue reads that same value in that same way. The
    // panel of a media that the user did not begin names no time that is left,
    // as `convert_seconds_for_prg` says.
    let live = a_message("0", "Not finished", "0");
    let panel = the_place_of_the_panel(
        false,
        None,
        Some(&live),
        Some(28800.0),
        "52",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "0");
    assert_eq!(panel.the_time_that_is_left, "");

    // **A place that is no number keeps the time of the row**, and the percent
    // and the mark of the end of the message stay: a message of a version of
    // the program before T-235 holds no such value.
    let live = a_message("75", "Not finished", "");
    let panel = the_place_of_the_panel(
        false,
        None,
        Some(&live),
        Some(28800.0),
        "52",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "75");
    assert_eq!(panel.the_time_that_is_left, "5h left,");

    // **A length that the server did not give keeps the time of the row too**
    // (T-180): the program makes no time of a place with no length.
    for length in [Some(0.0), None] {
        let live = a_message("75", "Not finished", "21600");
        let panel = the_place_of_the_panel(
            false,
            None,
            Some(&live),
            length,
            "52",
            "5h left,",
            "Not finished",
        );
        assert_eq!(panel.percent, "75");
        assert_eq!(panel.the_time_that_is_left, "5h left,");
    }

    // The panel of an episode of a podcast names no time that is left (T-229),
    // therefore the row of it holds no such text and the message gives the
    // percent and the mark of the end alone.
    let live = a_message("65", "Not finished", "200");
    let panel = the_place_of_the_panel(false, None, Some(&live), None, "20", "", "Not finished");
    assert_eq!(panel.percent, "65");
    assert_eq!(panel.the_time_that_is_left, "");
}
