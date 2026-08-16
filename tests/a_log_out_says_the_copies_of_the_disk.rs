//! A log out that keeps the copies of the disk says them. See T-297.
//!
//! **The measurement of 2026-08-16, of the real program v0.8.125 inside tmux
//! against the sandbox**, with the account `toutuitest` and the one library
//! `Books`:
//!
//! - before the key, the disk of that account held **11 rows of `downloads`**,
//!   13 rows of `download_files`, **19 files**, and **251382273 bytes**;
//! - the keys `S`, `Enter`, `l`, and `l` logged out. The row of `users` went
//!   away, and every one of those rows and every one of those bytes **stayed**;
//! - the program started again by itself, and the login screen came. **The words
//!   of the log out reached no user at all**: the row of the message went away
//!   with the process, and the screen of the login held the field of the address
//!   alone;
//! - a login with the same name and the same server gave the account and the 11
//!   rows again. That is the road back, and the words of the log out name it.
//!
//! The old words said "Start the program again." The program of that measurement
//! started again by itself, and a log out of an account that does not start the
//! program leaves the program at the view of the accounts: no road of the log out
//! asks the user for a start.
//!
//! This test writes `XDG_CONFIG_HOME` and `XDG_DATA_HOME`, therefore the parts of
//! it stay in one function (the trap 8 of the harness, and T-144).

use toutui::db::crud::{
    db_insert_usr, delete_user, insert_download, insert_download_file, the_words_of_a_log_out,
};
use toutui::db::database_struct::User;
use toutui::logic::download::{
    downloads_base_dir, the_copies_of_the_disk_that_stay, TheCopiesThatStay,
};

const SERVER: &str = "http://127.0.0.1:13399";

/// An account of the database, with the values that a login writes.
fn an_account(username: &str) -> User {
    User {
        username: username.to_string(),
        server_address: SERVER.to_string(),
        token: format!("the-token-of-{}", username),
        is_default_usr: username == "toutuitest",
        name_selected_lib: "Books".to_string(),
        id_selected_lib: "lib-1".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// Writes one download of an account: the row, the row of its file, and the file
/// of the disk.
fn a_download_of(username: &str, key: &str, bytes: usize) {
    let directory = downloads_base_dir(username).join(key);
    std::fs::create_dir_all(&directory).unwrap();

    let file = directory.join("001 - book.mp3");
    std::fs::write(&file, vec![b'0'; bytes]).unwrap();

    insert_download(
        key,
        username,
        "A Test Book",
        "The Author",
        file.to_str().unwrap(),
        1800.0,
        key,
        SERVER,
    )
    .unwrap();

    insert_download_file(
        key,
        username,
        1,
        "the-ino",
        file.to_str().unwrap(),
        bytes as u64,
        1800.0,
    )
    .unwrap();
}

#[test]
fn a_log_out_says_the_copies_of_the_disk() {
    // No line of this test may touch the files of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("XDG_DATA_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    db_insert_usr(&vec![an_account("toutuitest")]).unwrap();
    db_insert_usr(&vec![an_account("toutuilimited")]).unwrap();

    // An account of no copy of the disk holds no directory of its own at all.
    assert_eq!(
        the_copies_of_the_disk_that_stay("toutuitest"),
        TheCopiesThatStay::default(),
        "an account of no download holds no copy of the disk"
    );

    a_download_of("toutuilimited", "the-first-media", 3_000_000);
    a_download_of("toutuilimited", "the-second-media", 1_000_000);

    // The cache of the ebooks stands under the same directory of the account,
    // therefore the bytes of it belong to the copies of the disk too. See T-77.
    std::fs::write(
        downloads_base_dir("toutuilimited").join("the-book-of-the-cache.epub"),
        vec![b'0'; 500_000],
    )
    .unwrap();

    // **The two numbers come of two places** (T-297): the rows of the database
    // say the media, and the file system says the bytes.
    let copies = the_copies_of_the_disk_that_stay("toutuilimited");
    assert_eq!(copies.media, 2, "the database holds two downloads");
    assert_eq!(
        copies.bytes, 4_500_000,
        "the disk holds the audio of the two downloads and the book of the cache"
    );
    assert!(copies.they_stand());

    // **The copies of one account are not the copies of another one.**
    assert_eq!(
        the_copies_of_the_disk_that_stay("toutuitest"),
        TheCopiesThatStay::default(),
        "the copies of the disk hold the account of their own directory"
    );

    // **The words of the log out name them, and they name the road back**
    // (T-297). The old words said "The program removed the account
    // toutuilimited. Start the program again.", and 4500000 bytes of the disk
    // stood in no word at all.
    let the_words = delete_user("toutuilimited").unwrap();

    assert!(
        the_words.contains("toutuilimited"),
        "the words name the account: {}",
        the_words
    );
    assert!(
        the_words.contains("2 media"),
        "the words say the number of the media of the disk: {}",
        the_words
    );
    assert!(
        the_words.contains("4.3 MB"),
        "the words say the bytes of the disk in megabytes: {}",
        the_words
    );
    assert!(
        the_words.contains("Log in again with the same name and the same server"),
        "the words name the road back: {}",
        the_words
    );
    assert!(
        the_words.contains("the key X"),
        "the words name the key that removes a copy: {}",
        the_words
    );

    // **No road of the log out asks the user for a start** (T-297).
    assert!(
        !the_words.contains("Start the program again"),
        "the program starts again by itself, or it stays at the view: {}",
        the_words
    );

    // **The log out keeps the copies of the disk.** The key is a log out, and the
    // road back gives them again: the words say that, and the disk holds it.
    assert_eq!(
        std::fs::read_dir(downloads_base_dir("toutuilimited"))
            .unwrap()
            .count(),
        3,
        "the two downloads and the book of the cache stay on the disk"
    );

    // A place of the user and a copy of the disk stand in one sentence.
    let both = the_words_of_a_log_out("toutuilimited", 1, copies);
    assert!(both.contains("1 place of the user"), "{}", both);
    assert!(both.contains("2 media"), "{}", both);

    // An account of a copy of the disk and of no media of the database says the
    // bytes alone: the cache of the ebooks holds no row of `downloads`.
    let the_cache_alone = the_words_of_a_log_out(
        "toutuilimited",
        0,
        TheCopiesThatStay {
            media: 0,
            bytes: 500_000,
        },
    );
    assert!(!the_cache_alone.contains("media"), "{}", the_cache_alone);
    assert!(the_cache_alone.contains("0.5 MB"), "{}", the_cache_alone);
}
