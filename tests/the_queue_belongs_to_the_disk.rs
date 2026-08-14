//! A second program of the account must not take the media of the queue away.
//! See T-147.
//!
//! **Every change of the queue writes every row again** (T-56). The queue of the
//! process stood beside the queue of the disk, therefore the second program wrote
//! its own memory over the media of the first one.
//!
//! The measurement of 2026-08-13, with the sandbox and two sessions of tmux of
//! one `XDG_CONFIG_HOME`: the window A put "One Chapter Book" in the queue with
//! the key `n`, the window B then put "Multi File Test Book" with the same key,
//! **each screen said "The queue [1 item]" with its own media**, and the table
//! `queue` of the disk held the media of B alone.
//!
//! This test needs no server. It writes the queue of the second program with
//! `save_the_queue`, which is the one function that every program of the account
//! calls.

use toutui::db::crud::{read_the_queue, save_the_queue, QueueRow};
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

/// The row that a second program of the account writes.
fn a_row(id: &str, title: &str) -> QueueRow {
    QueueRow {
        id_item: id.to_string(),
        id_pod: String::new(),
        title: title.to_string(),
        author: "An Author".to_string(),
        duration: Some(60.0),
    }
}

/// The queue belongs to the process, therefore the parts of this measurement
/// stay in one test function. Two test functions of one binary would fight for
/// it, and for the database beside it (T-144).
#[test]
fn a_second_program_of_the_account_keeps_every_media_of_the_queue() {
    // No line of this test may touch the files of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    queue::read_the_queue_of_the_account(USER, SERVER);
    queue::clear();

    // The window A puts one book in the queue with the key `n`.
    queue::add(a_book("the-book-of-a", "The Book Of A"));

    // The window B does the same. It holds the queue of its own start, and that
    // queue is empty: its write therefore holds one row.
    save_the_queue(USER, SERVER, &[a_row("the-book-of-b", "The Book Of B")]).unwrap();

    // The window A puts a second book in the queue. **The disk is the truth**:
    // the write of A must not take the book of B away.
    queue::add(a_book("the-second-book-of-a", "The Second Book Of A"));

    let titles: Vec<String> = read_the_queue(USER, SERVER)
        .unwrap()
        .into_iter()
        .map(|row| row.title)
        .collect();

    assert_eq!(
        titles,
        vec![
            "The Book Of B".to_string(),
            "The Second Book Of A".to_string()
        ],
        "the queue of the disk must hold the media of both programs"
    );

    // The view of A takes the queue of the disk, therefore the user of that
    // window reads the media of both programs.
    queue::read_the_queue_again();

    assert_eq!(
        queue::snapshot()
            .entries()
            .iter()
            .map(|entry| entry.title.clone())
            .collect::<Vec<String>>(),
        vec![
            "The Book Of B".to_string(),
            "The Second Book Of A".to_string()
        ]
    );

    // The key `X` of a view that is older than the disk takes the media of its
    // own line, and not the media of that place.
    let entries = queue::snapshot();
    let of_the_line = entries.entries()[1].key();

    // The line 0 of the view of A held "The Second Book Of A" before the disk
    // moved under it. The key takes that media, and not "The Book Of B".
    let taken = queue::take_the_media(0, &of_the_line)
        .expect("the disk answered")
        .expect("the media of the line");

    assert_eq!(taken.title, "The Second Book Of A");
    assert_eq!(
        read_the_queue(USER, SERVER)
            .unwrap()
            .into_iter()
            .map(|row| row.title)
            .collect::<Vec<String>>(),
        vec!["The Book Of B".to_string()]
    );

    // A media that stands in the queue no more gives nothing, and the key must
    // still say what happened to the media of that line. **A key that does
    // nothing says why** (T-79), and this key said nothing at all before T-151.
    assert!(queue::take_the_media(0, "a-media-of-no-queue")
        .expect("the disk answered")
        .is_none());

    let text = queue::text_of_the_key_that_takes(Some("The Book Of B"), None)
        .expect("the key must say what happened to the media of its line");

    assert!(text.contains("The Book Of B"), "{}", text);
    assert!(text.contains("is not in the queue now"), "{}", text);

    // **No unit test reaches a key handler of `src/app.rs`**, therefore this
    // part reads the source, as the tests of T-131, T-143, T-149, and T-150 do:
    // the key `X` of the view of the queue must say the sentence on both roads.
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
        of_the_key.contains("text_of_the_key_that_takes("),
        "the key X of the view of the queue must say a sentence on both roads. See T-151."
    );

    queue::clear();
    queue::forget_the_account();
}
