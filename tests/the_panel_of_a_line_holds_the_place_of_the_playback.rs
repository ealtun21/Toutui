//! The panel of a line of a media that plays says the place of the engine of
//! this program. See T-239.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! The measurement of 2026-08-15 against the sandbox: the server held
//! `A Book Of Many Hours` at 10800 seconds of 28800, the user played it with
//! the key `l` of the Home view, and 75 seconds later one frame of one screen
//! said `Progress: 37%, 5h left, Not finished` in the panel while the row of
//! the player of that same frame said
//! `▶ 4:13:12 / 8:00:00 | Elapsed: 4:13:12 | Left: 3:46:48 (53%)`.

use toutui::logic::the_panel_of_a_line::the_place_of_the_panel;

#[test]
fn the_panel_of_a_line_holds_the_place_of_the_playback() {
    // The measurement itself: the media plays, the engine stands at 4:13:12 of
    // eight hours, and the row of the request holds the place of the start.
    let panel = the_place_of_the_panel(
        true,
        Some(15192.0),
        Some(28800.0),
        "37",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "53");
    assert_eq!(panel.the_time_that_is_left, "3h47m left,");
    assert_eq!(panel.the_end, "Not finished");

    // **A line whose media does not play keeps the row.** The place of the
    // engine belongs to one media, and every other line of the view holds the
    // answer of the request of the server.
    let panel = the_place_of_the_panel(
        false,
        Some(15192.0),
        Some(28800.0),
        "37",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "37");
    assert_eq!(panel.the_time_that_is_left, "5h left,");
    assert_eq!(panel.the_end, "Not finished");

    // **A place of 0 is a playback that did not begin**: the screen says
    // `Loading the media...` in that moment, and the panel keeps the row.
    let panel = the_place_of_the_panel(
        true,
        Some(0.0),
        Some(28800.0),
        "37",
        "5h left,",
        "Not finished",
    );
    assert_eq!(panel.percent, "37");
    assert_eq!(panel.the_time_that_is_left, "5h left,");

    // A playback that stopped holds no place at all.
    let panel = the_place_of_the_panel(true, None, Some(28800.0), "37", "5h left,", "Not finished");
    assert_eq!(panel.percent, "37");
    assert_eq!(panel.the_time_that_is_left, "5h left,");

    // **A length of 0, and a length that the server did not give, are one
    // thing** (T-180): the program makes no percent and no time of a place
    // with no length, therefore the panel keeps the row.
    for length in [Some(0.0), None] {
        let panel = the_place_of_the_panel(
            true,
            Some(15192.0),
            length,
            "37",
            "5h left,",
            "Not finished",
        );
        assert_eq!(panel.percent, "37");
        assert_eq!(panel.the_time_that_is_left, "5h left,");
    }

    // **The mark of the end of the row belongs to the place of the row**
    // (T-238): a media that the user finished and that plays again stands at
    // the place of the engine, and the panel says the percent of that place.
    let panel = the_place_of_the_panel(
        true,
        Some(1440.0),
        Some(28800.0),
        "100",
        "0m left,",
        "Finished",
    );
    assert_eq!(panel.percent, "5");
    assert_eq!(panel.the_time_that_is_left, "7h36m left,");
    assert_eq!(panel.the_end, "Not finished");

    // A media of the same line that no playback moves keeps the mark of its
    // end.
    let panel = the_place_of_the_panel(
        false,
        Some(1440.0),
        Some(28800.0),
        "100",
        "0m left,",
        "Finished",
    );
    assert_eq!(panel.the_end, "Finished");

    // The panel of a line that the lists of a view do not hold says `N/A`, and
    // this function gives that word back with no change (T-177).
    let panel = the_place_of_the_panel(false, Some(15192.0), None, "N/A", "", "N/A");
    assert_eq!(panel.percent, "N/A");
    assert_eq!(panel.the_end, "N/A");
}
