//! The sentence of the key `n` names the condition of the media. See T-232.
//!
//! **The key said one sentence for three conditions.** The measurement of the
//! real program v0.8.60 inside tmux, against the sandbox (podman on :13399), of
//! the library `Books`, with the table `queue` of the account empty at the
//! start: the user pressed `n` on `A Long Test Book`, `j`, `n` on
//! `A Big Book Of A Scan`, `k`, and `n` on `A Long Test Book` again.
//!
//! ```text
//! "A Long Test Book" is number 1 of the queue. Press q to see the queue.
//! "A Big Book Of A Scan" is number 2 of the queue. Press q to see the queue.
//! "A Long Test Book" is number 2 of the queue. Press q to see the queue.
//! ```
//!
//! The queue held two media before the third key and two media after it, and
//! `A Long Test Book` went from the place 1 to the place 2. The sentence of that
//! key holds the shape of the sentence of a media that came in, therefore a user
//! who does not press the key `q` reads a queue of three media.
//!
//! **The program has the reason** (T-91): `Queue::take_the_key_out` finds the
//! place of the media before the push, and `ThePlaceOfTheMedia` carries it to
//! the words. **A key that does nothing must say why** (T-79), and the key `n`
//! on the media of the last line of the queue changes nothing at all.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The structure `Queue` holds no lock and no global value and the words are a
//! pure function, therefore this test needs no database, no server, and no
//! screen. **A build of the fault fails it**: an `add` that gives no place of
//! before, and words that give one sentence for the three conditions.

use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{the_words_of_the_key_that_adds, Entry, Queue};

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

#[test]
fn the_key_of_the_queue_says_that_the_media_moved() {
    let mut queue = Queue::default();

    // **The media came in.** The queue held no media of that identity, therefore
    // the number of the sentence is the number of its new line.
    let came_in = queue.add(book("long", "A Long Test Book"));
    assert_eq!(came_in.place, 1);
    assert_eq!(came_in.the_place_before, None);
    assert_eq!(
        the_words_of_the_key_that_adds("A Long Test Book", came_in),
        "\"A Long Test Book\" is number 1 of the queue. Press q to see the queue."
    );

    let second = queue.add(book("scan", "A Big Book Of A Scan"));
    assert_eq!(second.place, 2);
    assert_eq!(second.the_place_before, None);

    // **The media moved.** This is the third key of the measurement: the queue
    // holds two media before it and two media after it, and the sentence names
    // the two places.
    let it_moved = queue.add(book("long", "A Long Test Book"));
    assert_eq!(it_moved.place, 2);
    assert_eq!(it_moved.the_place_before, Some(1));
    assert_eq!(queue.len(), 2);
    assert_eq!(
        the_words_of_the_key_that_adds("A Long Test Book", it_moved),
        "\"A Long Test Book\" waits in the queue already. It moves from number 1 to number 2. \
         Press q to see the queue."
    );

    // The sentence of a media that came in must not stand for a media that
    // moved: a user reads the first one as a queue that grew.
    assert_ne!(
        the_words_of_the_key_that_adds("A Long Test Book", it_moved),
        the_words_of_the_key_that_adds("A Long Test Book", came_in)
    );

    // **The key changed nothing at all**: the media stands at the last place of
    // the queue already, therefore the sentence says that it waits there (T-79).
    let it_stands = queue.add(book("long", "A Long Test Book"));
    assert_eq!(it_stands.place, 2);
    assert_eq!(it_stands.the_place_before, Some(2));
    assert_eq!(queue.len(), 2);
    assert_eq!(
        the_words_of_the_key_that_adds("A Long Test Book", it_stands),
        "\"A Long Test Book\" waits at number 2 of the queue already. Press q to see the queue."
    );

    // A queue of one media gives that same condition at the place 1.
    let mut one = Queue::default();
    one.add(book("scan", "A Big Book Of A Scan"));
    let again = one.add(book("scan", "A Big Book Of A Scan"));
    assert_eq!(again.place, 1);
    assert_eq!(again.the_place_before, Some(1));
    assert_eq!(
        the_words_of_the_key_that_adds("A Big Book Of A Scan", again),
        "\"A Big Book Of A Scan\" waits at number 1 of the queue already. Press q to see the queue."
    );

    // The place of before is the place of the line of the view, and the first
    // line is 1: a media of the front of a queue of three gives 1 and not 0.
    let mut three = Queue::default();
    three.add(book("a", "First"));
    three.add(book("b", "Second"));
    three.add(book("c", "Third"));
    let front = three.add(book("a", "First"));
    assert_eq!(front.the_place_before, Some(1));
    assert_eq!(front.place, 3);
}
