//! The limit of the cache of the ebooks, with real files. See T-67.
//!
//! The pure function `the_ebooks_that_must_go` holds the rule, and
//! `src/logic/reader/cache.rs` tests it. This test measures the work on the
//! disk: it makes three files, it gives them three times of use, and it reads
//! the directory after the removal.
//!
//! **This test writes `XDG_DATA_HOME`, therefore it must stay alone in its
//! binary and it must hold every part in one function.** A variable of the
//! environment belongs to the process. See the trap 25 of `docs/HANDOVER.md`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use toutui::logic::reader::cache::{
    hold_the_limit, the_book_is_in_use, the_cache_of, the_removal, THE_LIMIT_OF_THE_USE,
};

/// Makes a file of `bytes` bytes, and gives it a time of use of `ago` seconds
/// before now.
fn a_book(directory: &Path, name: &str, bytes: usize, ago: u64) -> PathBuf {
    let path = directory.join(name);
    fs::write(&path, vec![b'x'; bytes]).expect("the test must write the file");

    let when = SystemTime::now() - Duration::from_secs(ago);
    let times = fs::FileTimes::new().set_accessed(when).set_modified(when);

    fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .and_then(|file| file.set_times(times))
        .expect("the test must give the file its time");

    path
}

#[test]
fn the_cache_of_the_ebooks_holds_its_limit() {
    let directory = std::env::temp_dir().join(format!(
        "toutui-cache-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|when| when.as_nanos())
            .unwrap_or(0)
    ));

    fs::create_dir_all(&directory).expect("the test must make its directory");

    // The variable must come before the first call of `downloads_base_dir`.
    unsafe {
        std::env::set_var("XDG_DATA_HOME", &directory);
    }

    let of_the_user = directory.join("toutui").join("downloads").join("a user");
    fs::create_dir_all(&of_the_user).expect("the test must make the directory of the user");

    // The audio of a download stands in a directory of that directory, and the
    // cache must not count it.
    let of_the_audio = of_the_user.join("a download of the audio");
    fs::create_dir_all(&of_the_audio).expect("the test must make the directory of the audio");
    fs::write(of_the_audio.join("01 - Part 1.mp3"), vec![b'x'; 4000])
        .expect("the test must write the audio");

    let old = a_book(&of_the_user, "the oldest.pdf", 1000, 300);
    let newer = a_book(&of_the_user, "newer.pdf", 1000, 200);
    let now = a_book(&of_the_user, "the book of the user.epub", 1000, 100);

    // A file that is not an ebook stays outside the cache.
    let a_note = a_book(&of_the_user, "a note.txt", 5000, 10);

    let cache = the_cache_of("a user");
    assert_eq!(
        cache.len(),
        3,
        "the cache holds the three ebooks, and no audio and no note: {:?}",
        cache
    );
    assert_eq!(cache.iter().map(|book| book.bytes).sum::<u64>(), 3000);

    // A limit of 2500 bytes needs 500 bytes back, therefore the book of the
    // oldest use goes away and no other book goes away.
    let bytes = hold_the_limit("a user", &now, 2500);

    assert_eq!(bytes, 1000, "the program removed one book of 1000 bytes");
    assert!(!old.exists(), "the book of the oldest use went away");
    assert!(newer.exists(), "the program must stop at the limit");
    assert!(now.exists(), "the book of the user never goes away");
    assert!(a_note.exists(), "a file that is not an ebook stays");
    assert!(
        of_the_audio.join("01 - Part 1.mp3").exists(),
        "the audio of a download stays"
    );

    // A cache below the limit loses no book, and it removes nothing.
    assert_eq!(hold_the_limit("a user", &now, 2500), 0);
    assert!(newer.exists());

    // The removal gives the number of the books too. The sentence of the user
    // holds that number. See T-71.
    assert_eq!(the_removal("a user", &now, 2500), (0, 0));

    // The reader says that it uses the older book, and that word is the time of
    // the file.
    the_book_is_in_use(&newer);

    let of_the_use = fs::metadata(&newer)
        .and_then(|data| data.modified())
        .expect("the file must hold a time");
    assert!(
        of_the_use > SystemTime::now() - Duration::from_secs(30),
        "the time of the file is the time of the use now"
    );

    // **A book of a time inside `THE_LIMIT_OF_THE_USE` belongs to a reader that
    // stands open**, and that reader can be the reader of a second window of
    // this account: `keep` names the book of this program alone. The removal
    // therefore keeps this book, although the cache stands above its limit. See
    // T-153.
    assert_eq!(hold_the_limit("a user", &now, 1000), 0);
    assert!(
        newer.exists(),
        "a book that a program said it uses now must stay"
    );

    // The window of that reader goes away, therefore the book stands still. A
    // mark of a reader is not for ever, in the same way as the lock of T-148.
    let long_ago = SystemTime::now() - THE_LIMIT_OF_THE_USE - Duration::from_secs(5);
    let times = fs::FileTimes::new()
        .set_accessed(long_ago)
        .set_modified(long_ago);
    fs::OpenOptions::new()
        .write(true)
        .open(&newer)
        .and_then(|file| file.set_times(times))
        .expect("the test must give the file its time");

    // A limit of 1000 bytes needs 1000 bytes back of a cache of 2000. The book
    // of the user never goes away, therefore the other book goes.
    assert_eq!(hold_the_limit("a user", &now, 1000), 1000);
    assert!(!newer.exists(), "the book of no reader went away");
    assert!(now.exists(), "the book of the user never goes away");

    // One book, and it is larger than the limit. The program removes nothing
    // and it asks the server for nothing.
    assert_eq!(hold_the_limit("a user", &now, 10), 0);
    assert!(now.exists());

    fs::remove_dir_all(&directory).ok();
}
