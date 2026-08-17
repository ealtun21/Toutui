//! The title of a view whose name holds an end of a line stands in one row.
//! See T-376.
//!
//! **A title of the server can hold an end of a line**, and a ratatui `Span`
//! draws no control character: the words of the two lines then glue together
//! on the screen, with no space between them and no mark at all.
//!
//! The measurement of the real program v0.8.206 inside tmux, against the
//! sandbox (podman on :13399): `PATCH /api/items/:id/media` gave the book
//! `A Long Test Book` the title `Alpha\nOMEGAEND`, and the key `V` of that
//! book then gave the view `The bookmarks of "AlphaOMEGAEND" [1 item]` — the
//! two words glued. The row of the same book in the view of the search said
//! `Alpha OMEGAEND`, because the road of the lines collapses an end of a line
//! already (T-374): the two roads of one name said two names.
//!
//! The three sinks of a title of a view — `render_the_table_of_a_panel`,
//! `render_the_reason`, and `render_the_message` — each make the title with
//! [`toutui::logic::message::in_one_row`] alone, therefore the correction
//! stands in that function: it collapses every end of a line first, as
//! [`toutui::logic::message::in_one_line`] does for the row of a list.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! **The build of the fault fails it**: with the collapse of the ends of the
//! lines removed from `in_one_row`, a text of two lines that stands in the
//! width comes back with its `\n`, and the first assertion fails.

use toutui::logic::message::in_one_row;

#[test]
fn a_title_of_a_name_of_two_lines_stands_in_one_row() {
    // A text of two lines that stands in the width takes one space for the
    // end of the line, therefore the screen holds the two words apart.
    assert_eq!(in_one_row("Alpha\nOMEGAEND", 80), "Alpha OMEGAEND");

    // A `\r\n` takes one space together, as the rule of `in_one_line` says.
    assert_eq!(
        in_one_row("The bookmarks of \"Alpha\r\nOMEGAEND\" [1 item]", 80),
        "The bookmarks of \"Alpha OMEGAEND\" [1 item]"
    );

    // A text of two lines that is wider than the width collapses first and
    // the cut then reads the columns of one row: the three points say that
    // the screen cut it (T-304).
    assert_eq!(in_one_row("Alpha\nOMEGAEND", 8), "Alpha O…");

    // A text of one line keeps the work that it had.
    assert_eq!(in_one_row("Alpha OMEGAEND", 80), "Alpha OMEGAEND");
    assert_eq!(in_one_row("Alpha OMEGAEND", 8), "Alpha O…");
}
