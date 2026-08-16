//! The panel of a media under a list keeps its two lines. See T-315.
//!
//! **The name of an author is a text of the server, and a user of that server
//! gives it.** `PATCH /api/items/:id/media` of Audiobookshelf takes an author
//! of a name that holds an end of a line, and `GET /api/libraries/:id/items`
//! then gives that name in `media.metadata.authorName` to every client.
//!
//! The panel of a media stands under the list of the view, and
//! `the_areas_of_a_list` of `src/ui/tui.rs` gives it **two** rows in a terminal
//! that is not tall. The first line of that panel names the author, the year,
//! and the length of the media, and the second line names the place of the
//! user. An author of a name of two lines therefore gives the area three lines,
//! and the row of the place of the user goes away.
//!
//! The measurement of 2026-08-16, of the real program v0.8.143 inside tmux
//! against the sandbox on `:13399` with the account `toutuitest`, of a terminal
//! of 80 columns and **18** rows. It needs no proxy and no build of the fault:
//! one request gave the data of the fault.
//!
//! ```bash
//! curl -X PATCH http://localhost:13399/api/items/a4d8b9b2-... \
//!     -H "Authorization: Bearer $TOK" -H 'Content-Type: application/json' \
//!     -d '{"metadata":{"authors":[{"name":"Alpha\nOMEGAEND"}]}}'
//! ```
//!
//! The keys `Tab` and three `j` gave that book of the Library view, and the two
//! rows of the panel then held
//!
//! ```text
//! Author: Alpha
//! OMEGAEND - Year: N/A - Duration: 0m
//! ```
//!
//! No row of the screen said the place of the user. **The control of the same
//! run** (the trap 206): the same book of the same keys, with the author
//! `AlphaOMEGAEND` of one line, held
//!
//! ```text
//! Author: AlphaOMEGAEND - Year: N/A - Duration: 0m
//! Progress: 100%, 0m left, Finished
//! ```
//!
//! **Six panels of `src/ui/tui.rs` hold this shape**: the panel of a book of a
//! series, the panel of an entry of a list, the panel of an episode of a
//! podcast, and the panels of the Home view, of the Library view, and of the
//! view of a search. Each of them takes
//! `crate::ui::keys::the_panel_of_a_media`.
//!
//! These tests need no network and no sandbox: the first reads the text that
//! the function makes, and the second draws it with the widget of the panel
//! into a `Buffer` of ratatui with no terminal (T-256).

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget, Wrap},
};

/// The width of the measurement, and the rows that `the_areas_of_a_list` gives
/// the panel in a terminal that is not tall.
const WIDTH: u16 = 80;
const ROWS_OF_THE_PANEL: u16 = 2;

/// The rows of the panel of a media, of the widget of `src/ui/tui.rs`.
///
/// The widget is the widget of the six panels: a `Paragraph` that wraps, at the
/// left.
fn the_rows_of_the_panel(of_the_media: &str, of_the_place: &str) -> Vec<String> {
    let area = Rect::new(0, 0, WIDTH, ROWS_OF_THE_PANEL);
    let mut buf = Buffer::empty(area);

    Paragraph::new(toutui::ui::keys::the_panel_of_a_media(
        of_the_media,
        of_the_place,
    ))
    .wrap(Wrap { trim: true })
    .left_aligned()
    .render(area, &mut buf);

    (0..ROWS_OF_THE_PANEL)
        .map(|row| {
            (0..WIDTH)
                .map(|column| buf[(column, row)].symbol())
                .collect::<String>()
                .trim()
                .to_string()
        })
        .collect()
}

/// The text of the panel holds two lines, and no more. See T-315.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_text_of_the_panel_holds_two_lines() {
    let the_panel = toutui::ui::keys::the_panel_of_a_media(
        "Author: Alpha\nOMEGAEND - Year: N/A - Duration: 0m",
        "Progress: 100%, 0m left, Finished",
    );

    assert_eq!(
        the_panel.lines().count(),
        2,
        "the panel of the media holds more than two lines: {the_panel:?}"
    );

    assert_eq!(
        the_panel,
        "Author: Alpha OMEGAEND - Year: N/A - Duration: 0m\nProgress: 100%, 0m left, Finished"
    );

    // The end of a line of the answer of the server takes every shape of RFC
    // 5322: the program reads a `\r` and a `\r\n` in the same way.
    assert_eq!(
        toutui::ui::keys::the_panel_of_a_media("Author: Alpha\r\nOMEGAEND", "Progress: 1%")
            .lines()
            .count(),
        2
    );

    // **The line of the place of the user holds a text of the server too**: the
    // mark of the end and the time that is left come of the answer of the
    // server for a media that this program does not play.
    assert_eq!(
        toutui::ui::keys::the_panel_of_a_media("Author: A", "Progress: 1%, 2m left,\nFinished"),
        "Author: A\nProgress: 1%, 2m left, Finished"
    );

    // The control: two lines of one line each keep every character.
    assert_eq!(
        toutui::ui::keys::the_panel_of_a_media("Author: AlphaOMEGAEND", "Progress: 100%"),
        "Author: AlphaOMEGAEND\nProgress: 100%"
    );
}

/// The place of the user stands on the second row of the panel of two rows. See
/// T-315.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_place_of_the_user_stands_in_the_panel_of_two_rows() {
    let rows = the_rows_of_the_panel(
        "Author: Alpha\nOMEGAEND - Year: N/A - Duration: 0m",
        "Progress: 100%, 0m left, Finished",
    );

    assert_eq!(
        rows[0], "Author: Alpha OMEGAEND - Year: N/A - Duration: 0m",
        "the first row of the panel says no whole line of the media: {:?}",
        rows[0]
    );

    // **The second row of the panel belongs to the place of the user.** The row
    // of the fault held `OMEGAEND - Year: N/A - Duration: 0m`, and the place of
    // the user stood on no row of the screen.
    assert_eq!(
        rows[1], "Progress: 100%, 0m left, Finished",
        "the second row of the panel says no place of the user: {:?}",
        rows[1]
    );

    // The control: an author of one line gives the same two rows.
    let rows = the_rows_of_the_panel(
        "Author: AlphaOMEGAEND - Year: N/A - Duration: 0m",
        "Progress: 100%, 0m left, Finished",
    );
    assert_eq!(rows[0], "Author: AlphaOMEGAEND - Year: N/A - Duration: 0m");
    assert_eq!(rows[1], "Progress: 100%, 0m left, Finished");
}
