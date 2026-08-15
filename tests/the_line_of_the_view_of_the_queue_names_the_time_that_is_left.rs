//! The line of the view of the queue names the time that is left. See T-234.
//!
//! **The time at the end of a line said the length of the media.** The user of
//! this view chooses the media that comes after the media that plays, and the
//! length of a media that stands at 90 percent tells that user nothing.
//!
//! The measurement of the real program v0.8.62 inside tmux, against the sandbox
//! (podman on :13399), of the library `Books`. The server held
//! `A Second Book Of Many Hours` at 7200 seconds of 28800, `A Long Test Book`
//! at 900 of 1800, and `A Book Of Many Hours` at 7200 of 28800. The user
//! pressed the key `n` on each of the three and then the key `q`:
//!
//! ```text
//! ➤     1. 📕 A Second Book Of Many Hours — Many Hours Author  (8h)
//!   50% 2. 📕 A Long Test Book — Long Author  (30m)
//!   90% 3. 📕 A Book Of Many Hours — Many Hours Author  (8h)
//! ```
//!
//! **The control of the same run** (the trap 206): the panel of the Home view
//! of that same program said `Duration: 8h` and
//! `Progress: 0%, 6h left, Not finished` of `A Second Book Of Many Hours`.
//! Therefore the program holds the time that is left already, and one view of
//! it says that time while the other one says the length.
//!
//! The same measurement of the corrected program:
//!
//! ```text
//! ➤     1. 📕 A Second Book Of Many Hours — Many Hours Author  (6h left)
//!   50% 2. 📕 A Long Test Book — Long Author  (15m left)
//!   90% 3. 📕 A Book Of Many Hours — Many Hours Author  (6h left)
//! ```
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The function is pure, therefore this test needs no server and no screen.
//! **Three builds of the fault each fail it**: a line that says the length of
//! every media, a line that says `0m left` for a media that came to its end,
//! and a line that reads the place of its neighbour.

use std::collections::BTreeMap;
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{the_lines_of_the_queue, Entry};

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

fn place(percent: &str, finished: &str, seconds: &str) -> Vec<String> {
    vec![
        percent.to_string(),
        finished.to_string(),
        seconds.to_string(),
    ]
}

#[test]
fn the_line_of_the_view_of_the_queue_names_the_time_that_is_left() {
    let entries = vec![
        book("many-hours", "A Book Of Many Hours", 28800.0),
        book("long", "A Long Test Book", 1800.0),
        book("never", "A Book That Never Played", 1800.0),
        book("at-the-start", "A Book At The Start", 1800.0),
        book("at-the-end", "A Book At Its End", 1800.0),
        book("of-an-old-version", "A Book Of An Old Row", 1800.0),
    ];

    let mut places = BTreeMap::new();
    places.insert(
        "many-hours".to_string(),
        place(" 25", " Not finished", "7200"),
    );
    places.insert("long".to_string(), place(" 50", " Not finished", "900"));
    places.insert(
        "at-the-start".to_string(),
        place(" 0", " Not finished", "0"),
    );
    places.insert("at-the-end".to_string(), place(" 100", " Finished", "1800"));
    // A row of a version before T-234 holds the percent and the mark of the end
    // alone. The line of that media keeps the length of it.
    places.insert(
        "of-an-old-version".to_string(),
        vec![" 40".to_string(), " Not finished".to_string()],
    );

    let lines = the_lines_of_the_queue(&entries, &places, None);

    assert_eq!(lines.len(), 6, "the lines are {:?}", lines);

    // **The time of a media that the user began is the time that is left of
    // it.** The line said the length before T-234, and the panel of the Home
    // view of that same media said `6h left`.
    assert!(
        lines[0].contains("(6h left)"),
        "the line of a book of 8h at 2h must say 6h left, and it is {:?}",
        lines[0]
    );
    assert!(
        lines[1].contains("(15m left)"),
        "the line of a book of 30m at 15m must say 15m left, and it is {:?}",
        lines[1]
    );

    // A media of no place is a media that the user did not begin. The length of
    // it is the time that is left, therefore the line keeps the length and it
    // says no word of a time that is left.
    assert!(
        lines[2].contains("(30m)") && !lines[2].contains("left"),
        "a media of no place must keep its length, and the line is {:?}",
        lines[2]
    );

    // A place of 0 is the place of a media that the user did not begin (T-177
    // and T-188): the server gives that place to a media that never played.
    assert!(
        lines[3].contains("(30m)") && !lines[3].contains("left"),
        "a media of the place 0 must keep its length, and the line is {:?}",
        lines[3]
    );

    // A place at the end of the media, or after it, keeps the length: the mark
    // of that line says already that the media came to its end, and a line of
    // `(0m left)` says nothing more.
    assert!(
        lines[4].contains("(30m)") && !lines[4].contains("left"),
        "a media at its end must keep its length, and the line is {:?}",
        lines[4]
    );

    // A row of a version before this item, and a row whose place is no number,
    // each say nothing of the place. The line of that media keeps the length.
    assert!(
        lines[5].contains("(30m)") && !lines[5].contains("left"),
        "a row of no place must keep the length of its media, and the line is \
         {:?}",
        lines[5]
    );

    // **The place stands on the line of its own media** (T-230). A line that
    // reads the row of its neighbour gives the time of another book.
    assert!(
        lines[0].contains("A Book Of Many Hours") && lines[1].contains("A Long Test Book"),
        "the lines keep their names, and they are {:?}",
        lines
    );

    // A caller that holds no place of the user gives none, and every line then
    // holds the length of its media. That is the view of the offline mode, and
    // it is the message of a test.
    let nothing = the_lines_of_the_queue(&entries, &BTreeMap::new(), None);

    assert!(
        nothing.iter().all(|line| !line.contains("left")),
        "a queue of no place must say no time that is left, and the lines are \
         {:?}",
        nothing
    );
}
