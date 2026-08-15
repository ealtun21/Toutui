//! The line of the view of the queue keeps the time that is left when a
//! message of the server comes. See T-235.
//!
//! **One message of one media took the time that is left away from every line
//! of that view.** T-234 gave the row of the places a third value, the place of
//! the user in seconds, and the line says `6h left` with it. The render of that
//! view writes the row of a live message over the row of the request, and that
//! row held two values: the percent and the mark of the end. A message of the
//! server therefore gave every line the length of its media again.
//!
//! **The message carries the position of every media of the account** (T-184),
//! therefore a message of one media reached every line of the queue.
//!
//! The measurement of the real program v0.8.63 inside tmux, against the sandbox
//! (podman on :13399), of the library `Books`. The user pressed the key `n` on
//! `A Long Test Book` (900 seconds of 1800), on `A Second Book Of Many Hours`
//! (7200 of 28800), and on `A Book Of Many Hours` (7200 of 28800), and then the
//! key `q`:
//!
//! ```text
//! ➤ 50% 1. 📕 A Long Test Book — Long Author  (15m left)
//!       2. 📕 A Second Book Of Many Hours — Many Hours Author  (6h left)
//!   90% 3. 📕 A Book Of Many Hours — Many Hours Author  (6h left)
//! ```
//!
//! A second client of that same account then moved in **one** media of the
//! queue (`PATCH /api/me/progress/e2b76945…` with `{"progress":0.42}`), and the
//! log said `[live] user_updated: the position of 27 media`:
//!
//! ```text
//! ➤ 50% 1. 📕 A Long Test Book — Long Author  (30m)
//!   42% 2. 📕 A Second Book Of Many Hours — Many Hours Author  (8h)
//!   90% 3. 📕 A Book Of Many Hours — Many Hours Author  (8h)
//! ```
//!
//! **The control of the same run** (the trap 206): the percent of the line 2
//! moved from nothing to `42%` in that same frame. The message came, and the
//! render read it: it took the percent of that message and it threw the place
//! of the user away.
//!
//! The same measurement of the corrected program said `(6h left)`, `(15m
//! left)`, and `(6h left)` after the message, with the percent at `55%`; and a
//! `PATCH` of `{"currentTime":14400}` of that same media gave `(4h left)` at
//! the next frame.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! The functions are pure, therefore this test needs no server and no screen.
//! **Two builds of the fault each fail it**: a row of a message of two values
//! (the shape before this item), and a row that reads the percent of the
//! message for the place of the user.

use std::collections::BTreeMap;
use toutui::api::live::{progress_of_the_user, Progress};
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{the_lines_of_the_queue, the_row_of_a_live_message, Entry};

fn book(id: &str, title: &str, length: f64) -> Entry {
    Entry {
        target: PlaybackTarget::Book {
            item_id: id.to_string(),
            whole_book_duration: Some(length),
        },
        title: title.to_string(),
        author: "An Author".to_string(),
        duration: Some(length),
    }
}

#[test]
fn the_line_of_the_queue_keeps_the_time_that_is_left_after_a_message() {
    // The message of the server holds the place of the user of each media.
    let body = serde_json::json!({
        "mediaProgress": [
            {
                "libraryItemId": "the second book of many hours",
                "progress": 0.42,
                "currentTime": 7200.0,
                "isFinished": false,
            },
            {
                "libraryItemId": "a long test book",
                "progress": 0.5,
                "currentTime": 900.0,
                "isFinished": false,
            },
        ]
    });

    let rows = progress_of_the_user(&body);
    assert_eq!(rows.len(), 2);

    let of_the_second_book = rows
        .iter()
        .find(|(key, _)| key == "the second book of many hours")
        .map(|(_, live)| live.clone())
        .expect("the message names that media");

    // **The place of the user comes with the percent.** A message that gives
    // the percent alone leaves the line with the length of the media.
    assert_eq!(of_the_second_book.percent, "42");
    assert_eq!(of_the_second_book.place, "7200");

    let of_the_long_book = rows
        .iter()
        .find(|(key, _)| key == "a long test book")
        .map(|(_, live)| live.clone())
        .expect("the message names that media");

    // The row of the render holds the three values of the row of the request:
    // the percent, the mark of the end, and the place of the user in seconds.
    let row = the_row_of_a_live_message(&of_the_second_book);
    assert_eq!(
        row,
        vec![
            "42".to_string(),
            "Not finished".to_string(),
            "7200".to_string()
        ]
    );

    let mut places: BTreeMap<String, Vec<String>> = BTreeMap::new();
    places.insert(
        "a long test book".to_string(),
        the_row_of_a_live_message(&of_the_long_book),
    );
    places.insert("the second book of many hours".to_string(), row);

    let entries = vec![
        book("a long test book", "A Long Test Book", 1800.0),
        book(
            "the second book of many hours",
            "A Second Book Of Many Hours",
            28800.0,
        ),
    ];

    let lines = the_lines_of_the_queue(&entries, &places, None, None);

    // **The line says the time that is left, and not the length.** The fault of
    // this item gave `(30m)` and `(8h)` here.
    assert!(
        lines[0].contains("(15m left)"),
        "the line of a media of a message must say the time that is left: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("(6h left)"),
        "the line of a media of a message must say the time that is left: {}",
        lines[1]
    );

    // A message that holds no place of the user gives the text of 0, and the
    // line of that media keeps the length of it: the program says no time that
    // it does not have (T-91).
    let with_no_place = progress_of_the_user(&serde_json::json!({
        "mediaProgress": [
            {
                "libraryItemId": "a long test book",
                "progress": 0.5,
                "isFinished": false,
            },
        ]
    }));

    let of_no_place = with_no_place
        .first()
        .map(|(_, live)| live.clone())
        .expect("the message names that media");

    assert_eq!(of_no_place.place, "0");

    let mut places_of_no_place: BTreeMap<String, Vec<String>> = BTreeMap::new();
    places_of_no_place.insert(
        "a long test book".to_string(),
        the_row_of_a_live_message(&of_no_place),
    );

    let lines = the_lines_of_the_queue(&entries[..1], &places_of_no_place, None, None);
    assert!(
        lines[0].contains("(30m)"),
        "a media of no place keeps the length of it: {}",
        lines[0]
    );

    // The row of a message that a caller makes by hand holds the same three
    // values, in the same sequence.
    let by_hand = Progress {
        percent: "90".to_string(),
        finished: "Not finished".to_string(),
        place: "7200".to_string(),
    };

    assert_eq!(
        the_row_of_a_live_message(&by_hand),
        vec![
            "90".to_string(),
            "Not finished".to_string(),
            "7200".to_string()
        ]
    );
}
