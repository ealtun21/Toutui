//! The line of the view of the queue holds the place of its media. See T-230.
//!
//! **A line of that view is one media**, and no line of it held one word of the
//! place of that media: no percent of the user, no mark of the media that the
//! user finished, and no mark of the media that plays. The view took the title,
//! the author, and the length of the media and it put them on the screen.
//!
//! The measurement of the real program v0.8.58 inside tmux, against the sandbox
//! (podman on :13399), of the library `Books`. The server held
//! `A Book Of Many Hours` at 90 percent, `A Second Book Of Many Hours`
//! finished, and `A Long Test Book` at 50 percent, and the user pressed the key
//! `n` on each of the three and then the key `q`:
//!
//! ```text
//! 1. 📕 A Book Of Many Hours — Many Hours Author  (8h)
//! 2. 📕 A Long Test Book — Long Author  (30m)
//! 3. 📕 A Second Book Of Many Hours — Many Hours Author  (8h)
//! ```
//!
//! **The control of the same run** (the trap 206): the Home view of that same
//! program gave `90% A Book Of Many Hours`, `50% A Long Test Book`, and
//! `✓   A Second Book Of Many Hours`. A second run played
//! `A Second Book Of Many Hours` and put it in the queue with the key `n`: the
//! Home view said `▶   A Second Book Of Many Hours` and `42% A Big Book Of A
//! Scan`, and the two lines of the queue of that same second said
//! `1. 📕 A Big Book Of A Scan — Big Author  (0m)` and
//! `2. 📕 A Second Book Of Many Hours — Many Hours Author  (8h)`.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The function is pure, therefore this test needs no server and no screen.
//! **Three builds of the fault each fail it**: a line that reads no place at
//! all, a key of a line that drops the episode, and a line that takes the row of
//! its neighbour.

use std::collections::BTreeMap;
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{the_lines_of_the_queue, Entry};

fn book(id: &str, title: &str) -> Entry {
    Entry {
        target: PlaybackTarget::Book {
            item_id: id.to_string(),
            whole_book_duration: Some(28800.0),
        },
        title: title.to_string(),
        author: "An Author".to_string(),
        duration: Some(28800.0),
    }
}

fn episode(item: &str, id: &str, title: &str) -> Entry {
    Entry {
        target: PlaybackTarget::Episode {
            item_id: item.to_string(),
            episode_id: id.to_string(),
        },
        title: title.to_string(),
        author: String::new(),
        duration: None,
    }
}

fn place(percent: &str, finished: &str) -> Vec<String> {
    vec![percent.to_string(), finished.to_string()]
}

#[test]
fn the_line_of_the_view_of_the_queue_holds_its_place() {
    let entries = vec![
        book("many-hours", "A Book Of Many Hours"),
        book("second-many-hours", "A Second Book Of Many Hours"),
        book("never", "A Book That Never Played"),
        episode("pym", "chapter-00", "Chapter 00"),
        episode("pym", "chapter-01", "Chapter 01"),
    ];

    let mut places = BTreeMap::new();
    places.insert("many-hours".to_string(), place(" 90", " Not finished"));
    places.insert("second-many-hours".to_string(), place(" 100", " Finished"));
    places.insert("pym/chapter-00".to_string(), place(" 30", " Not finished"));
    places.insert("pym/chapter-01".to_string(), place(" 0", " Finished"));

    let lines = the_lines_of_the_queue(&entries, &places, Some("pym/chapter-00"));

    assert_eq!(lines.len(), 5, "the lines are {:?}", lines);

    // **The percent of the user stands on the line of its own media.** The
    // whole view said nothing of the place before T-230.
    assert!(
        lines[0].starts_with("90% "),
        "the line of a book at 90 percent must say it, and it is {:?}",
        lines[0]
    );

    // **The mark of the end stands on the line of the media that the user
    // finished**, as it stands on the line of the Home view (T-44).
    assert!(
        lines[1].starts_with("✓"),
        "the line of a book that the user finished must hold the mark of the \
         end, and it is {:?}",
        lines[1]
    );

    // A media of no row of the server takes no mark, as a media that never
    // played takes none in the Home view.
    assert!(
        lines[2].trim_start().starts_with("3. "),
        "a media of no place must keep its title alone, and the line is {:?}",
        lines[2]
    );
    assert!(
        !lines[2].contains('%') && !lines[2].contains('✓') && !lines[2].contains('▶'),
        "a media of no place must hold no mark at all, and the line is {:?}",
        lines[2]
    );

    // **The key of a line names the episode after the item** (T-223, T-228, and
    // T-229). Two episodes of one podcast hold the identity of that podcast: a
    // key of the item alone gives the mark of the media that plays to every
    // episode of it, and it gives the place of one episode to every line.
    assert!(
        lines[3].starts_with("▶"),
        "the line of the episode that plays must hold the mark of it, and it \
         is {:?}",
        lines[3]
    );
    assert!(
        !lines[4].contains('▶'),
        "a second episode of that same podcast must hold no mark of the media \
         that plays, and the line is {:?}",
        lines[4]
    );
    assert!(
        lines[4].starts_with("✓"),
        "the line of the second episode must hold the place of that episode \
         alone, and it is {:?}",
        lines[4]
    );

    // The number of the place and the name of the media stay: the mark stands
    // before them, and it takes no word of the line away.
    assert!(
        lines[0].contains("1. ") && lines[0].contains("A Book Of Many Hours"),
        "the line keeps the number of the place and the name, and it is {:?}",
        lines[0]
    );

    // A caller that holds no place of the user gives none, and every line then
    // holds its title alone. That is the view of the offline mode, and it is
    // the message of a test.
    let nothing = the_lines_of_the_queue(&entries, &BTreeMap::new(), None);

    assert!(
        nothing
            .iter()
            .all(|line| !line.contains('%') && !line.contains('✓') && !line.contains('▶')),
        "a queue of no place must hold no mark at all, and the lines are {:?}",
        nothing
    );
}
