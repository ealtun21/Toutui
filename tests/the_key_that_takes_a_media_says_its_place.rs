//! The key `X` of the view of the queue says the place of the media that it
//! took. See T-233.
//!
//! **The measurement of 2026-08-15**, with the real program v0.8.61 inside tmux
//! and the sandbox: the user put three books in the queue with the key `n`,
//! pressed `q`, moved to the line 2, and pressed `X`. The message said:
//!
//! ```text
//! "A Big Book Of A Scan" is not in the queue now.
//! ```
//!
//! The title alone. **A media that goes out of the queue changes the number of
//! every media after it**: the book of the line 3 became the line 2, and the
//! user who reads that sentence cannot tell which number went away. The program
//! held that number: `take_the_media` knew the place at the moment that it
//! removed the entry, and it gave the entry alone.
//!
//! **The place of the disk is not the number of the line** (T-147): a second
//! program of the account moves the media under the view, and the key then takes
//! the media of its line at another place. That place is the truth of the
//! sentence.
//!
//! The queue belongs to the process, therefore the parts of this test stay in
//! one function. Two test functions of one binary would fight for it, and for
//! the database beside it (T-144 and T-157).

use toutui::db::crud::{save_the_queue, QueueRow};
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{self, Entry};

const SERVER: &str = "http://127.0.0.1:1";
const USER: &str = "toutuitest";

fn a_book(id: &str, title: &str) -> Entry {
    Entry {
        target: PlaybackTarget::Book {
            item_id: id.to_string(),
            whole_book_duration: Some(60.0),
        },
        title: title.to_string(),
        author: "An Author".to_string(),
        duration: Some(60.0),
    }
}

fn a_row(id: &str, title: &str) -> QueueRow {
    QueueRow {
        id_item: id.to_string(),
        id_pod: String::new(),
        title: title.to_string(),
        author: "An Author".to_string(),
        duration: Some(60.0),
    }
}

#[test]
fn the_key_that_takes_a_media_out_of_the_queue_says_the_place_of_it() {
    // The words are pure, therefore this part needs no queue and no database.
    let text = queue::text_of_the_key_that_takes(
        2,
        Some("A Big Book Of A Scan"),
        Some("A Big Book Of A Scan"),
    )
    .expect("the key must say what happened to the media of its line");

    assert!(
        text.contains("A Big Book Of A Scan"),
        "the sentence must name the media: {}",
        text
    );
    assert!(
        text.contains("number 2"),
        "the sentence must name the place that the media held. See T-233: {}",
        text
    );
    assert!(
        text.contains("is not in the queue now"),
        "the sentence must say that the media waits no more. See T-151: {}",
        text
    );

    // **A media that a second program took out first gives no place at all**
    // (T-151), and the number of the line of the view is then the number that
    // the user saw. The two roads give one sentence.
    let of_the_other_road = queue::text_of_the_key_that_takes(3, Some("The Book Of B"), None)
        .expect("the key must say what happened to the media of its line");

    assert!(
        of_the_other_road.contains("number 3") && of_the_other_road.contains("The Book Of B"),
        "the two roads of the key give one sentence, and it names the place. See T-151 and \
         T-233: {}",
        of_the_other_road
    );

    assert!(
        queue::text_of_the_key_that_takes(1, None, None).is_none(),
        "a line of no media gives no sentence"
    );

    // **The place of the answer is the place of the disk, and not the number of
    // the line of the view.** No line of this test may touch the files of the
    // user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    queue::read_the_queue_of_the_account(USER, SERVER);
    queue::clear();

    queue::add(a_book("the-first-book", "The First Book")).unwrap();
    queue::add(a_book("the-second-book", "The Second Book")).unwrap();

    // A second program of the account puts a media at the front of the queue of
    // the disk. The view of this program still holds "The Second Book" at the
    // line 2, and the disk holds it at the line 3.
    save_the_queue(
        USER,
        SERVER,
        &[
            a_row("the-book-of-b", "The Book Of B"),
            a_row("the-first-book", "The First Book"),
            a_row("the-second-book", "The Second Book"),
        ],
    )
    .unwrap();

    let went_out = queue::take_the_media(1, "the-second-book")
        .expect("the disk answered")
        .expect("the media of the line");

    assert_eq!(went_out.entry.title, "The Second Book");
    assert_eq!(
        went_out.place, 3,
        "the place of the answer is the place of the disk, and the line of the view held the \
         number 2. See T-147 and T-233"
    );

    let text = queue::text_of_the_key_that_takes(
        went_out.place,
        Some("The Second Book"),
        Some(went_out.entry.title.as_str()),
    )
    .expect("the key must say what happened to the media of its line");

    assert!(
        text.contains("number 3"),
        "the sentence must hold the place of the disk: {}",
        text
    );

    // **No unit test reaches a key handler of `src/app.rs`**, therefore this
    // part reads the source, as the test of T-151 does: the key `X` must give
    // the place of the answer to the words, and the number of its line when the
    // answer holds no media. The block ends at the function after this one.
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("the test must read src/app.rs");

    let of_the_key = source
        .split_once("pub fn remove_from_the_queue(&mut self) {")
        .expect("the handler of the key X of the queue must stand in src/app.rs")
        .1;

    let of_the_key = of_the_key
        .split_once("\n    }\n")
        .expect("the handler must end")
        .0;

    assert!(
        of_the_key.contains("went_out.place"),
        "the key X must say the place of the media that it took. See T-233."
    );
    assert!(
        of_the_key.contains("index + 1"),
        "a media that a second program took out first gives no place, and the number of the line \
         of the view is then the number that the user saw. See T-151 and T-233."
    );

    queue::clear();
    queue::forget_the_account();
}
