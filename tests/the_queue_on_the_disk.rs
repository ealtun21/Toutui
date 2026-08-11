//! The queue of the media stands on the disk, therefore a new start keeps it.
//! See T-56.
//!
//! The queue lived in the memory of the process before this work. A user who
//! stopped the program lost every media that waited.
//!
//! **This test sets `XDG_CONFIG_HOME`, therefore it stands alone in its binary.**
//! That variable belongs to the process, and a second test of the same binary
//! would then read the database of the first one. The test needs no server and no
//! sound card.

use toutui::db::crud::{read_the_queue, save_the_queue, QueueRow};

/// A book of the queue.
fn book(id: &str, title: &str) -> QueueRow {
    QueueRow {
        id_item: id.to_string(),
        id_pod: String::new(),
        title: title.to_string(),
        author: "An Author".to_string(),
        duration: Some(1800.0),
    }
}

/// One episode of a podcast. Two episodes of one podcast hold the same item,
/// therefore the identity of the episode belongs to the key of the row.
fn episode(id: &str, episode: &str) -> QueueRow {
    QueueRow {
        id_item: id.to_string(),
        id_pod: episode.to_string(),
        title: format!("The episode {}", episode),
        author: "A Podcast".to_string(),
        duration: None,
    }
}

#[test]
fn the_queue_of_one_account_survives_a_new_start() {
    let directory = std::env::temp_dir().join(format!(
        "toutui-queue-{}-{}",
        std::process::id(),
        u64::from(std::process::id()) * 7 + 13
    ));

    // The database stands in `<XDG_CONFIG_HOME>/toutui`. The program makes that
    // directory at its start, therefore the test makes it here.
    std::fs::create_dir_all(directory.join("toutui"))
        .expect("the test must make its own directory");
    std::env::set_var("XDG_CONFIG_HOME", &directory);

    // The database of this test stands in that directory, and the migration of
    // the version 7 makes the table of the queue.
    let conn = toutui::db::migrate::open_conn().expect("the database must open");
    let version = toutui::db::migrate::schema_version(&conn).expect("the version must come");
    assert!(
        version >= 7,
        "the migration of the queue must run: the version is {}",
        version
    );
    drop(conn);

    let user = "toutuitest";
    let server = "http://127.0.0.1:13399";

    // A queue with no media gives no row, and it is not a fault.
    assert!(read_the_queue(user, server).is_empty());

    let rows = vec![
        book("book-1", "The First Book"),
        episode("podcast-1", "episode-7"),
        book("book-2", "The Second Book"),
    ];

    save_the_queue(user, server, &rows).expect("the program must write the queue");

    // A new start of the program reads the rows again, **in the sequence of the
    // queue**. The sequence is the whole value of a queue.
    let read = read_the_queue(user, server);
    assert_eq!(read, rows);

    // The account of a different server holds its own queue.
    let other = "http://192.168.1.10:13378";
    assert!(read_the_queue(user, other).is_empty());

    save_the_queue(
        other,
        other,
        &[book("book-9", "A Book Of The Other Server")],
    )
    .expect("the program must write the queue of the other server");
    assert_eq!(read_the_queue(user, server).len(), 3);

    // A user of a different account holds its own queue.
    assert!(read_the_queue("a different user", server).is_empty());

    // The write of the queue gives every row again. A media that went out of the
    // queue must go out of the disk as well.
    save_the_queue(user, server, &rows[1..]).expect("the program must write the queue");
    let after = read_the_queue(user, server);
    assert_eq!(after.len(), 2);
    assert_eq!(after[0].id_pod, "episode-7");
    assert_eq!(after[1].id_item, "book-2");

    // An empty queue takes every row away.
    save_the_queue(user, server, &[]).expect("the program must write the queue");
    assert!(read_the_queue(user, server).is_empty());
}
