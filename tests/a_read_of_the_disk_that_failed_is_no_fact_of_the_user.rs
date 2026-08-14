//! A read of the disk that failed is not a fact of the user. See T-202.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME`, and that variable belongs to the process: two test
//! functions of one binary fight for it. See T-144 and T-157.
//!
//! T-200 gave the fault of the disk to the **writes** of the module of the
//! database. **The reads hold the other half of that sweep**, and the default of
//! such a read is a fact of the user: an empty list is "the queue of the user
//! holds no media", and `None` is "the account stands in no row of the disk".
//!
//! The measurement of 2026-08-14 with `docs/harness/hold_the_lock.py`: two media
//! stood in the queue of the disk (`select title from queue` gave `Letter 48` and
//! `Letter 47`), the lock stood, and the key `q` of the view of the queue gave
//!
//! ```text
//! ─────── The queue is empty. Press n on a media to put it in the queue. ───────
//! [ERROR] - [read_the_queue] the program cannot open the database.
//! ```
//!
//! **The view said a reason that the program does not have** (T-91 and T-171),
//! and every change of the queue after that read writes the queue of the process
//! on the disk: the media of the user of a second program then goes away with it
//! (T-147).
//!
//! The wait of a playback holds the same shape: the log of the measurement of
//! T-201 held `[wait_prev_session_finished] the account toutuitest stands in no
//! row of the disk` while `select count(*) from users` said **1**, and the program
//! said the sentence of an account that is gone to the user.
//!
//! The condition of this test is a file that holds no database: it gives the same
//! fault of `open_conn` with no wait at all (T-200).

use toutui::db::crud;

#[test]
fn a_read_of_the_disk_that_failed_is_not_a_fact_of_the_user() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());

    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    std::fs::write(
        dir.path().join("toutui").join("db.sqlite3"),
        b"this file holds no database at all",
    )
    .unwrap();

    // **The queue of a database that says nothing is no queue with no media.**
    assert!(
        crud::read_the_queue("toutuitest", "the-server").is_err(),
        "a read of the queue that failed must not give a queue of no media"
    );

    // **A read of the account that failed is not an account that stands in no
    // row.** `None` of these two reads is the account that a second program
    // removed (T-155 and T-158), and the program says that to the user.
    assert!(
        crud::get_has_played_before("toutuitest").is_err(),
        "a read of the account that failed must not give an account of no row"
    );
    assert!(
        crud::get_is_loop_break("toutuitest").is_err(),
        "a read of the loop of the account that failed must not give an account of no row"
    );

    // The queue of this program takes no media of a disk that says nothing, and
    // **a key of the user that cannot read the disk changes nothing**: the disk is
    // the truth of the queue (T-147).
    toutui::logic::queue::read_the_queue_of_the_account("toutuitest", "the-server");

    let of_the_media = toutui::logic::queue::Entry {
        target: toutui::logic::playback::PlaybackTarget::Book {
            item_id: "the-book".to_string(),
            whole_book_duration: Some(60.0),
        },
        title: "A Book".to_string(),
        author: "An Author".to_string(),
        duration: Some(60.0),
    };

    assert_eq!(
        toutui::logic::queue::add(of_the_media),
        None,
        "a media of a queue that the program did not read reaches no disk"
    );

    assert_eq!(
        toutui::logic::queue::take_the_media(0, "the-book"),
        Err(toutui::logic::queue::TheDiskDidNotAnswer),
        "a key that did not read the queue of the disk takes no media of it"
    );

    // The words of that key name the key of the work of the fault (T-79 and
    // T-170).
    let words = toutui::logic::queue::the_words_of_a_queue_that_the_disk_did_not_give();

    assert!(
        words.contains("press the key again"),
        "the words must name the key of the work: {}",
        words
    );
    assert!(
        !words.contains("empty"),
        "the words must not say that the queue holds no media: {}",
        words
    );

    // The queue of this program holds no media of that disk, therefore no line of
    // the view says a media that the program did not read.
    assert_eq!(
        toutui::logic::queue::len(),
        0,
        "the queue of this program holds the media that it read, and it read none"
    );

    // A test must leave no account in the box of the queue of this process.
    toutui::logic::queue::forget_the_account();
}
