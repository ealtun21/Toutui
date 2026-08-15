//! The queue holds one media one time, and the number of the key `n` says it.
//! See T-231.
//!
//! **The key `n` on a media that stands in the queue already named a number
//! that no line of the queue held.** The measurement of the real program
//! v0.8.59 inside tmux, against the sandbox (podman on :13399), of the library
//! `Books`: the user pressed `n` on `A Long Test Book`, `j`, `n` on
//! `A Big Book Of A Scan`, and `n` on that same book again. The three messages
//! said `number 1`, `number 2`, and `number 3`, and the key `q` of that same
//! second said:
//!
//! ```text
//! The queue [2 items]
//! 50% 1. 📕 A Long Test Book — Long Author  (30m)
//! 77% 2. 📕 A Big Book Of A Scan — Big Author  (0m)
//! ```
//!
//! The queue of the disk of that same moment held two rows, of the places 0 and
//! 2: `save_the_queue` writes one row for one media, because the primary key of
//! the table `queue` is the account, the server, the item, and the episode.
//! **The disk is the truth of the queue** (T-147), therefore the queue of the
//! process holds one entry for one media too.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The structure `Queue` holds no lock and no global value, therefore this test
//! needs no database, no server, and no screen. **Two builds of the fault each
//! fail it**: a key `n` that puts the media in a second time, and a media of a
//! playback that did not start that goes to the front beside the entry of
//! before.

use std::collections::BTreeSet;
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{Entry, Queue};

fn book(id: &str, title: &str) -> Entry {
    Entry {
        target: PlaybackTarget::Book {
            item_id: id.to_string(),
            whole_book_duration: Some(1800.0),
        },
        title: title.to_string(),
        author: "An Author".to_string(),
        duration: Some(1800.0),
    }
}

fn episode(item: &str, id: &str, title: &str) -> Entry {
    Entry {
        target: PlaybackTarget::Episode {
            item_id: item.to_string(),
            episode_id: id.to_string(),
        },
        title: title.to_string(),
        author: "A Podcast".to_string(),
        duration: Some(1320.0),
    }
}

/// The rule of the queue that one media takes one place.
///
/// The disk writes one row for one media. Therefore the number that the key `n`
/// says is the number of the lines of the view of that same second, and the
/// media of that key stands at that place.
fn the_queue_of_the_disk(queue: &Queue) -> BTreeSet<String> {
    queue.entries().iter().map(|entry| entry.key()).collect()
}

#[test]
fn the_queue_holds_a_media_one_time() {
    let mut queue = Queue::default();

    // The two keys `n` of two media give the two numbers of the two lines.
    assert_eq!(queue.add(book("long", "A Long Test Book")).place, 1);
    assert_eq!(queue.add(book("scan", "A Big Book Of A Scan")).place, 2);

    // **The key `n` on the media that stands in the queue already**: the disk
    // holds two rows after it, therefore the number of the message is 2 and the
    // view holds two lines.
    assert_eq!(queue.add(book("scan", "A Big Book Of A Scan")).place, 2);
    assert_eq!(queue.len(), 2);
    assert_eq!(the_queue_of_the_disk(&queue).len(), queue.len());

    // The media of that key stands at the end, as the row of the disk of the
    // place 2 stood after the row of the place 0.
    assert_eq!(
        queue
            .entries()
            .iter()
            .map(|entry| entry.title.as_str())
            .collect::<Vec<_>>(),
        vec!["A Long Test Book", "A Big Book Of A Scan"]
    );

    // Two episodes of one podcast are two media (T-223 and T-229), therefore
    // both of them keep their place.
    assert_eq!(
        queue.add(episode("pym", "chapter-00", "Chapter 00")).place,
        3
    );
    assert_eq!(
        queue.add(episode("pym", "chapter-01", "Chapter 01")).place,
        4
    );
    assert_eq!(queue.len(), 4);
    assert_eq!(the_queue_of_the_disk(&queue).len(), 4);

    // The key `n` on one of those two episodes moves that episode alone.
    assert_eq!(
        queue.add(episode("pym", "chapter-00", "Chapter 00")).place,
        4
    );
    assert_eq!(
        queue
            .entries()
            .iter()
            .map(|entry| entry.key())
            .collect::<Vec<_>>(),
        vec!["long", "scan", "pym/chapter-01", "pym/chapter-00"]
    );

    // **A playback that did not start puts its media at the front** (T-146),
    // and the user can have pressed the key `n` on that same media: the queue
    // holds it one time there too, and the disk then holds it at the front.
    queue.put_at_the_front(book("scan", "A Big Book Of A Scan"));
    assert_eq!(queue.len(), 4);
    assert_eq!(the_queue_of_the_disk(&queue).len(), 4);
    assert_eq!(
        queue
            .entries()
            .iter()
            .map(|entry| entry.key())
            .collect::<Vec<_>>(),
        vec!["scan", "long", "pym/chapter-01", "pym/chapter-00"]
    );

    // A media of the front that no entry of the queue holds keeps every other
    // media where it stood.
    queue.put_at_the_front(book("hours", "A Book Of Many Hours"));
    assert_eq!(queue.len(), 5);
    assert_eq!(queue.entries()[0].key(), "hours");
    assert_eq!(the_queue_of_the_disk(&queue).len(), 5);
}
