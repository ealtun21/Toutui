//! The line of the view of the queue of the media that plays holds the place of
//! the engine. See T-238.
//!
//! **The place of the row of that view is the place of the moment of the key
//! `q`**: the request of the places runs at that key (T-230) and at a media that
//! came into the queue (T-237), and a live message of the server is the one
//! other road to a newer place (T-235). **The playback of this same program
//! takes neither road**, therefore the line of the media that plays stood at the
//! place of the key while the row of the player of that same frame said the new
//! one.
//!
//! The measurement of the real program v0.8.66 inside tmux, against the sandbox
//! (podman on :13399). The server held `A Book Of Many Hours` at 7200 seconds of
//! 28800. The user played it with the key `l`, put it in the queue with the key
//! `n`, and pressed the key `q`. The line said `(7h58m left)`, and 45 seconds
//! later (the null device plays 8 hours in about 6 minutes) that same line said:
//!
//! ```text
//! ➤ ▶   1. 📕 A Book Of Many Hours — Many Hours Author  (7h58m left)
//!                        ▶ 1:04:23 / 8:00:00 | Left: 6:55:37 (13%) | Speed: 1.00x
//! ```
//!
//! **The control of the same run** (the trap 206): the mark `▶` of that line
//! stood on it at each frame, therefore the line reads the state of the playback
//! already; and the row of the player of that same screen said the true place.
//!
//! The same measurement of the corrected program, at two moments of one run:
//!
//! ```text
//! ➤ ▶   1. 📕 A Book Of Many Hours — Many Hours Author  (5h53m left)
//!                        ▶ 2:07:26 / 8:00:00 | Left: 5:52:34 (27%) | Speed: 1.00x
//! ➤ ▶   1. 📕 A Book Of Many Hours — Many Hours Author  (5h15m left)
//!                        ▶ 2:44:54 / 8:00:00 | Left: 5:15:06 (34%) | Speed: 1.00x
//! ```
//!
//! The engine holds that place already, therefore the correction costs no
//! request at all.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The function of the lines is pure, therefore this test needs no server and no
//! screen. **Five builds of the fault each fail it**: a line that keeps the
//! place of the row, a line of every media that takes the place of the engine, a
//! place of 0 of a playback that did not begin, a mark of the end of the row
//! that stands over the place of the engine, and a render that gives the place
//! of the engine to no line.

use std::collections::BTreeMap;
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{the_lines_of_the_queue, Entry};

/// A book of eight hours of the queue.
fn book(id: &str) -> Entry {
    Entry {
        target: PlaybackTarget::Book {
            item_id: id.to_string(),
            whole_book_duration: Some(28800.0),
        },
        title: id.to_string(),
        author: "Many Hours Author".to_string(),
        duration: Some(28800.0),
    }
}

/// A row of the box of the places: the percent, the mark of the end, and the
/// place of the user in seconds. See T-234.
fn row(percent: &str, the_end: &str, place: &str) -> Vec<String> {
    vec![percent.to_string(), the_end.to_string(), place.to_string()]
}

/// Gives the block of a function of a file of the source. See the trap 209.
///
/// A window of a number of characters is a window of the comments of the
/// function after it: the words of a correction take a line out of that window,
/// and the gate then says that the program lost a rule that it holds. The block
/// ends at the comment or at the head of the function that comes after this one.
fn the_block_of(source: &str, head: &str) -> String {
    let start = source
        .find(head)
        .unwrap_or_else(|| panic!("the source holds no function `{}`", head));
    let body = &source[start + head.len()..];

    let end = body
        .find("\n    /// ")
        .into_iter()
        .chain(body.find("\n    pub fn "))
        .chain(body.find("\n    fn "))
        .min()
        .unwrap_or(body.len());

    body[..end].to_string()
}

#[test]
fn the_line_of_the_queue_holds_the_place_of_the_playback() {
    let entries = [book("a-book"), book("a-second-book")];

    // The two media stood at 7200 seconds of 28800 at the moment of the key
    // `q`, therefore each of them held six hours.
    let mut places = BTreeMap::new();
    places.insert("a-book".to_string(), row("25%", "", "7200"));
    places.insert("a-second-book".to_string(), row("25%", "", "7200"));

    // ---------------------------------------------------------------------
    // The line of the media that plays.
    // ---------------------------------------------------------------------

    // The engine stands at 2:44:54 of the eight hours, therefore 5h15m stay.
    let lines = the_lines_of_the_queue(&entries, &places, Some("a-book"), Some(9894.0));

    assert!(
        lines[0].contains("(5h15m left)"),
        "the line of the media that plays must hold the place of the engine, \
         and it said: {}",
        lines[0]
    );
    assert!(
        !lines[0].contains("(6h left)"),
        "the place of the moment of the key q must reach no line of a media \
         that this program moved, and the line said: {}",
        lines[0]
    );

    // **The place of the engine belongs to the line of the media that plays,
    // and to no other line**: one playback moves one media, and the other
    // media of the queue keep the place of the request.
    assert!(
        lines[1].contains("(6h left)"),
        "the line of a media that does not play must keep the place of the \
         request, and it said: {}",
        lines[1]
    );

    // ---------------------------------------------------------------------
    // A playback that did not begin.
    // ---------------------------------------------------------------------

    // The engine gives the place 0 while the media loads (the screen says
    // `Loading the media...`), and a place of 0 is no place of the user: the
    // line then keeps the place of the request.
    let of_a_media_that_loads =
        the_lines_of_the_queue(&entries, &places, Some("a-book"), Some(0.0));

    assert!(
        of_a_media_that_loads[0].contains("(6h left)"),
        "a playback that did not begin must take no place from the line, and \
         it said: {}",
        of_a_media_that_loads[0]
    );

    // A playback that stopped gives no place at all, and no line of the view
    // then reads the engine.
    let of_no_playback = the_lines_of_the_queue(&entries, &places, None, None);

    assert!(
        of_no_playback[0].contains("(6h left)"),
        "a view with no playback must keep the place of the request, and it \
         said: {}",
        of_no_playback[0]
    );

    // ---------------------------------------------------------------------
    // A media that the user finished and that plays again.
    // ---------------------------------------------------------------------

    // **The mark of the end of the row belongs to the place of the row**
    // (T-236): a media that plays stands at the place of the engine, therefore
    // the line of it says the time that is left of that place and not the
    // length of the whole media.
    let mut of_the_end = BTreeMap::new();
    of_the_end.insert("a-book".to_string(), row("100%", "Finished", "28800"));

    let lines = the_lines_of_the_queue(&entries[..1], &of_the_end, Some("a-book"), Some(9894.0));

    assert!(
        lines[0].contains("(5h15m left)"),
        "a media that the user finished and that plays again must hold the \
         place of the engine, and the line said: {}",
        lines[0]
    );

    // The mark of the end of a media that does not play stays: the line of it
    // keeps the length of the media.
    let of_a_media_that_stands = the_lines_of_the_queue(&entries[..1], &of_the_end, None, None);

    assert!(
        of_a_media_that_stands[0].contains("(8h)"),
        "a media that the user finished and that plays no more keeps the \
         length, and the line said: {}",
        of_a_media_that_stands[0]
    );

    // ---------------------------------------------------------------------
    // The render of the view gives that place to the line.
    // ---------------------------------------------------------------------

    let source = include_str!("../src/app.rs");

    let of_the_lines = the_block_of(
        source,
        "pub fn queue_lines(&self, entries: &[crate::logic::queue::Entry]) -> Vec<String> {",
    );

    assert!(
        of_the_lines.contains("the_place_of_the_playback()"),
        "the render of the view of the queue must give the place of the engine \
         to the line of the media that plays"
    );

    // **A playback that stopped gives no place**, as `playing_media` gives no
    // media: a line of a media of the queue would else hold the place of the
    // playback before it.
    let of_the_place = the_block_of(
        source,
        "pub fn the_place_of_the_playback(&self) -> Option<f64> {",
    );

    assert!(
        of_the_place.contains("PlaybackStatus::Stopped"),
        "the place of the playback must come of a playback that stands"
    );
}
