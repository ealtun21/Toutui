//! A change of the disk that the disk did not take is no change. See T-206.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME`, and that variable belongs to the process: two test
//! functions of one binary fight for it. See T-144 and T-157.
//!
//! # The condition
//!
//! Every measurement of a fault of the database since T-199 took the **whole**
//! file away: `docs/harness/hold_the_lock.py` holds the write lock, and a file
//! that holds no database gives the same fault with no wait. Each of those two
//! stops the read and the write together, therefore each of them hides the road
//! where the program **reads** the disk of the user and the disk then refuses the
//! write.
//!
//! **A database that the program reads and cannot write is that road** (T-206): a
//! file of a database with no permission of a write, a disk that is full, and a
//! file system that a machine gave back as read-only each give it. SQLite opens
//! such a file, every `SELECT` of the program answers, and every `INSERT` gives
//! `attempt to write a readonly database`.
//!
//! # The measurement of 2026-08-14
//!
//! The program stood in the Library view of the sandbox, and `chmod 444` of
//! `db.sqlite3` gave the condition. The key `n`:
//!
//! ```text
//! "A Book Of An Epub With No Container" is number 1 of the queue. Press q to see the queue.
//! ```
//!
//! `select count(*) from queue` said **0**, and the key `q` of that same sentence
//! then said `The queue is empty. Press n on a media to put it in the queue.`
//! The one word of the fault stood in the log.
//!
//! The key `X` of the view of the queue holds the other direction: it said
//! `"A Book Of An Epub With No Container" is not in the queue now.`, the line
//! went out of the view, the disk kept both media, and the keys `h` and `q`
//! brought the media back.
//!
//! The keys `O` and `I` of the player hold the same shape with no word at all:
//! `let _ = update_speed_rate(...)` and a read of that same row after it gave the
//! engine the speed of before, therefore the key changed no speed, it said
//! nothing, and it wrote no line of the log.
//!
//! # The rule
//!
//! **The disk is the truth of the queue** (T-147), therefore a change that stands
//! in this program alone is a change that no program of the account reads: the
//! queue of the process goes back to the queue of before, and the key says why.
//! The row of the account is the truth of the speed in the same way.

use toutui::logic::queue::{self, TheDiskDidNotAnswer};

/// The media of the disk of this test.
fn a_book(id: &str, title: &str) -> queue::Entry {
    queue::Entry {
        target: toutui::logic::playback::PlaybackTarget::Book {
            item_id: id.to_string(),
            whole_book_duration: Some(60.0),
        },
        title: title.to_string(),
        author: "An Author".to_string(),
        duration: Some(60.0),
    }
}

#[test]
fn a_change_that_the_disk_did_not_take_is_no_change() {
    const USER: &str = "toutuitest";
    const SERVER: &str = "http://127.0.0.1:13399";

    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let of_the_disk = dir.path().join("toutui").join("db.sqlite3");

    {
        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();
    }

    // The disk of the account holds one media, and this program reads it.
    toutui::db::crud::save_the_queue(
        USER,
        SERVER,
        &[toutui::db::crud::QueueRow {
            id_item: "the-book-of-the-disk".to_string(),
            id_pod: String::new(),
            title: "The Book Of The Disk".to_string(),
            author: "An Author".to_string(),
            duration: Some(60.0),
        }],
    )
    .unwrap();

    queue::read_the_queue_of_the_account(USER, SERVER);

    assert_eq!(
        queue::len(),
        1,
        "the queue of the process must hold the media of the disk"
    );

    // **The disk reads, and it takes no write.**
    let mut how = std::fs::metadata(&of_the_disk).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut how, 0o444);
    std::fs::set_permissions(&of_the_disk, how).unwrap();

    // The read of the disk answers still. Therefore the fault below is the write
    // alone, and no other condition of this road gives it.
    assert!(
        toutui::db::crud::read_the_queue(USER, SERVER).is_ok(),
        "the program must read a database that takes no write"
    );

    // **The key `n`: a media that the disk did not take is no media of the
    // queue.** The old shape gave `Some(1)`, and the program then said that the
    // media is number 1 of the queue.
    assert_eq!(
        queue::add(a_book("the-book-of-the-key", "The Book Of The Key")),
        Err(TheDiskDidNotAnswer::TheWrite),
        "a media that the disk did not take must not reach the queue of the user"
    );

    assert_eq!(
        queue::snapshot()
            .entries()
            .iter()
            .map(|entry| entry.title.clone())
            .collect::<Vec<String>>(),
        vec!["The Book Of The Disk".to_string()],
        "the queue of the process must hold the queue of the disk, and no media more"
    );

    // **The key `X`: a media that the disk still holds waits still.** The old
    // shape gave `Ok(Some(...))`, the line went out of the view, and the next
    // read of the disk brought it back.
    assert_eq!(
        queue::take_the_media(0, "the-book-of-the-disk"),
        Err(TheDiskDidNotAnswer::TheWrite),
        "a media that the disk still holds must not go out of the queue of the user"
    );

    assert_eq!(
        queue::len(),
        1,
        "the media of the disk must stay in the queue of the process"
    );

    // The queue of the disk changed with none of the two keys.
    assert_eq!(
        toutui::db::crud::read_the_queue(USER, SERVER)
            .unwrap()
            .len(),
        1,
        "the queue of the disk must hold the media of before"
    );

    // **The speed of the account.** The write gives the fault, therefore the key
    // reads it. The old shape was `let _ = update_speed_rate(...)`.
    assert!(
        toutui::db::crud::update_speed_rate(USER, true).is_err(),
        "a write of the speed that the disk did not take must give a fault"
    );

    // The disk takes a write again, therefore the temporary directory goes away.
    let mut how = std::fs::metadata(&of_the_disk).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut how, 0o644);
    std::fs::set_permissions(&of_the_disk, how).unwrap();

    // # The words of the three keys
    //
    // **A read and a write are two conditions** (T-206), and the sentence of the
    // key names the one that happened: the words of a read name a second Toutui
    // (T-202), and the words of a write name the queue that does not change.
    // Each of them names the key of the view that the user sees at that moment
    // (T-183).
    let of_the_read =
        queue::the_words_of_a_queue_that_the_disk_did_not_hold(TheDiskDidNotAnswer::TheRead, "n");

    assert!(
        of_the_read.contains("did not read"),
        "the words of a read must name the read: {}",
        of_the_read
    );

    let of_the_write =
        queue::the_words_of_a_queue_that_the_disk_did_not_hold(TheDiskDidNotAnswer::TheWrite, "n");

    assert!(
        of_the_write.contains("did not write") && of_the_write.contains("Press n again"),
        "the words of a write must name the write and the key: {}",
        of_the_write
    );

    let of_the_key_x =
        queue::the_words_of_a_queue_that_the_disk_did_not_hold(TheDiskDidNotAnswer::TheWrite, "X");

    assert!(
        of_the_key_x.contains("Press X again"),
        "the words of the view of the queue must name the key of that view: {}",
        of_the_key_x
    );

    let of_the_speed =
        toutui::player::integrated::handle_key_player::the_words_of_a_speed_that_the_disk_did_not_hold(
            "O",
        );

    assert!(
        of_the_speed.contains("did not write") && of_the_speed.contains("Press O again"),
        "the words of the speed must name the write and the key: {}",
        of_the_speed
    );

    // # The source of the key of the speed
    //
    // The engine takes the speed of the row of the account, therefore a test of
    // the value needs a player and a database of a playback. **The sequence of
    // the two lines is the correction**, and a test of the source holds it, as
    // T-135, T-143, T-204, and T-205 hold.
    let of_the_file = include_str!("../src/player/integrated/handle_key_player.rs");

    assert!(
        !of_the_file.contains("let _ = update_speed_rate("),
        "the key of the speed must read the answer of the write of the disk"
    );

    assert!(
        of_the_file.contains("if let Err(error) = update_speed_rate("),
        "the key of the speed must give its fault a sentence for the user"
    );
}
