//! The removal of the cache of the ebooks must keep the book that a second
//! program of the account reads. See T-153.
//!
//! **`keep` is a fact of the process.** The window A reads a book, and the
//! removal of the window B names its own book in `keep` and no other: B knew
//! nothing of the book of A, therefore it took that book of the disk while A
//! read it. That is the shape of T-148 and of T-150 — a state that one process
//! holds is not a state that the account holds — and the answer is the answer
//! of those items: **the disk is the truth**.
//!
//! The measurement of 2026-08-14, with two windows of one `XDG_CONFIG_HOME`:
//! the window A read "A Huge Book Of A Scan", the window B pressed `e` on a
//! book of 100 megabytes, and the log of B says
//! `the cache of the ebooks gave 545898521 bytes of 2 book(s) back` — the PDF
//! of 502745447 bytes of A, and the 43016313 bytes of its pages of T-62.
//!
//! The reader of A writes the time of its file every 15 seconds
//! (`say_that_a_program_reads_this_book`), and the removal keeps every book of
//! a time inside `THE_LIMIT_OF_THE_USE`. This test writes the two times with no
//! reader and no second program: the time of the file is the whole word that
//! the two programs share.
//!
//! **This test writes `XDG_DATA_HOME`, therefore it stays alone in its binary
//! and it holds every part in one function.** See the trap 25 of
//! `docs/HANDOVER.md`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use toutui::logic::reader::cache::{the_cache_of, the_removal, THE_LIMIT_OF_THE_USE};

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
fn the_removal_keeps_the_book_that_a_second_window_reads() {
    let directory = std::env::temp_dir().join(format!(
        "toutui-cache-of-two-windows-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or(0)
    ));

    fs::create_dir_all(&directory).expect("the test must make its directory");
    std::env::set_var("XDG_DATA_HOME", &directory);

    let user = "a-user";
    let of_the_user = toutui::logic::download::downloads_base_dir(user);
    fs::create_dir_all(&of_the_user).expect("the test must make the directory of the user");

    // The book of a journey of the user. No program reads it now.
    let old = a_book(&of_the_user, "the oldest.epub", 1000, 300);

    // **The book that the window A reads now.** The reader of that window wrote
    // the time of the file some seconds ago, and this program cannot see that
    // window in any other way.
    let of_the_window_a = a_book(&of_the_user, "the book of the window A.pdf", 1000, 5);

    // The book that the window B took from the server now. It is the `keep` of
    // the removal of B.
    let came_now = a_book(&of_the_user, "the book of the window B.epub", 1000, 0);

    assert_eq!(
        the_cache_of(user).len(),
        3,
        "the cache must hold the three books"
    );

    // The limit takes one book of 1000 bytes. The removal must therefore look
    // for two books to remove, and one of them is the book of the window A.
    let (books, bytes) = the_removal(user, &came_now, 1000);

    assert!(
        of_the_window_a.exists(),
        "the removal took the book that the window A reads. `keep` names the \
         book of this program alone, and the time of the file is the one word \
         that the two programs share: a book of a time inside {} seconds \
         belongs to a reader that stands open. The measurement of 2026-08-14 \
         lost 545898521 bytes of one key. See T-153.",
        THE_LIMIT_OF_THE_USE.as_secs()
    );

    assert!(
        came_now.exists(),
        "the book that came now is the `keep` of the removal, and it stays"
    );

    assert!(
        !old.exists(),
        "the book of no reader must go away. The cache holds its limit still, \
         and a rule that keeps every book is no rule at all."
    );

    assert_eq!(books, 1, "one book went away, and it is the oldest");
    assert_eq!(bytes, 1000);

    // The window A goes away. Its book stands still, and the next removal takes
    // it: the mark of a reader is not for ever, in the same way as the lock of
    // T-148.
    let long_ago = SystemTime::now() - THE_LIMIT_OF_THE_USE - Duration::from_secs(5);
    let times = fs::FileTimes::new()
        .set_accessed(long_ago)
        .set_modified(long_ago);
    fs::OpenOptions::new()
        .write(true)
        .open(&of_the_window_a)
        .and_then(|file| file.set_times(times))
        .expect("the test must give the file its time");

    let (books, _) = the_removal(user, &came_now, 1000);

    assert_eq!(
        books, 1,
        "the book of a window that went away must go away too"
    );
    assert!(!of_the_window_a.exists());

    let _ = fs::remove_dir_all(&directory);
}
