//! A disk that gave no byte of a book is not a file of another form. See
//! T-276.
//!
//! **The measurement of 2026-08-16**, of the real program v0.8.104 inside tmux
//! against the sandbox: the file of the cache of the ebooks of "Alice in
//! Wonderland" took no read (`chmod 000`), the user pressed the key `e` on that
//! book, and the reader said
//!
//! ```text
//! This file is not an EPUB.
//! ```
//!
//! The book was a good EPUB: the same program read chapter 3 of 14 of it 90
//! seconds before. The log of that run held one line of the cache and no word
//! of the reader at all, because `Epub::open(path).map_err(|_| NotAnEpub)`
//! dropped every reason.
//!
//! **A view never says a reason that the program does not have** (T-91). The
//! first read of `Book::open` is the five bytes that say the form of the file,
//! and that read holds the reason of the machine.
//!
//! These tests need no network and no sandbox.

use std::path::{Path, PathBuf};
use toutui::logic::reader::book::{the_file_starts_as_a_pdf, Book, ReaderError};

/// Makes a directory of the test, and it gives the path of it.
fn a_directory(name: &str) -> PathBuf {
    let place = std::env::temp_dir().join(format!("toutui-t276-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&place);
    std::fs::create_dir_all(&place).expect("the directory of the test");
    place
}

/// The path of the EPUB of the tests. It is a good book of Project Gutenberg.
fn a_good_epub() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/alice.epub")
}

/// A disk that gives the program no byte of the file says the reason of the
/// machine, and it never says that the file is not an EPUB.
///
/// **The parts of this test stay in one function**: a path that does not exist
/// and a path of a file that takes no read are the two roads of one condition,
/// and each of them reaches the same value.
#[test]
fn a_disk_that_gave_no_book_names_the_reason_of_the_machine() {
    let place = a_directory("no-book");

    // The first road: the file went away between the look of the cache and the
    // open of the reader.
    let absent = place.join("a-book-that-is-not-there.epub");

    match Book::open(&absent) {
        Err(ReaderError::TheDiskGaveNoBook(reason)) => {
            assert!(
                !reason.is_empty(),
                "the value holds the reason of the machine"
            );
        }
        other => panic!("a file that is not there gives the disk: {:?}", other.err()),
    }

    // The second road: the disk holds the file, and it gives the program no
    // byte of it. This is the condition of the measurement of T-276.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let of_the_book = place.join("alice.epub");
        std::fs::copy(a_good_epub(), &of_the_book).expect("the copy of the book");

        // The control comes first: the same file, and the same call, gives a
        // book. A test of a fault that never had a book says nothing.
        Book::open(&of_the_book).expect("the book of the disk that answers");

        std::fs::set_permissions(&of_the_book, std::fs::Permissions::from_mode(0o000))
            .expect("the file of no read");

        let outcome = Book::open(&of_the_book);

        // The user of the gate can be root, and root reads a file of 0o000.
        let the_disk_refused = std::fs::File::open(&of_the_book).is_err();

        if the_disk_refused {
            match outcome {
                Err(ReaderError::TheDiskGaveNoBook(reason)) => {
                    assert!(
                        reason.to_lowercase().contains("permission"),
                        "the value names the reason of the machine: {reason}"
                    );
                }
                other => panic!("a file of no read gives the disk: {:?}", other.err()),
            }
        }

        let _ = std::fs::set_permissions(&of_the_book, std::fs::Permissions::from_mode(0o644));
    }

    let _ = std::fs::remove_dir_all(&place);
}

/// A file that the disk gives and that no reader opens keeps the words of a
/// book that is not an EPUB.
#[test]
fn a_file_that_is_no_book_keeps_the_words_of_a_book_that_is_no_epub() {
    let place = a_directory("no-epub");
    let of_the_file = place.join("a-file-that-is-no-book.epub");

    std::fs::write(&of_the_file, b"this file holds no archive of a book at all")
        .expect("the file of the test");

    match Book::open(&of_the_file) {
        Err(ReaderError::NotAnEpub) => {}
        other => panic!("a file that is no book gives NotAnEpub: {:?}", other.err()),
    }

    let _ = std::fs::remove_dir_all(&place);
}

/// The words of the disk say that the book can be good, they hold the reason of
/// the machine, and they name the key that goes back.
#[test]
fn the_words_of_a_disk_that_gave_no_book_name_the_reason_and_the_key() {
    let words =
        ReaderError::TheDiskGaveNoBook("Permission denied (os error 13)".to_string()).to_string();

    assert!(
        words.contains("The book can be good"),
        "the words keep the book of the user: {words}"
    );
    assert!(
        words.contains("Permission denied (os error 13)"),
        "the words hold the reason of the machine: {words}"
    );
    assert!(
        words.contains("Press h to go back"),
        "the words name the key of the view of the reader: {words}"
    );
    assert!(
        !words.contains("not an EPUB"),
        "the words say no reason that the program does not have: {words}"
    );
}

/// A file of fewer than five bytes is no PDF, and it is no fault of the disk.
///
/// **The parts of this test stay in one function**: the three answers of
/// `the_file_starts_as_a_pdf` are one rule.
#[test]
fn the_five_bytes_of_the_form_of_a_file_tell_the_disk_from_the_book() {
    let place = a_directory("five-bytes");

    let of_the_pdf = place.join("a-book.pdf");
    std::fs::write(&of_the_pdf, b"%PDF-1.7 and the rest of the book").expect("the PDF of the test");
    assert_eq!(
        Ok(true),
        the_file_starts_as_a_pdf(&of_the_pdf).map_err(|_| ())
    );

    // Fewer than five bytes: the read stops at the end of the file, and that
    // end is no fault of the disk.
    let of_the_short = place.join("a-short-file.epub");
    std::fs::write(&of_the_short, b"%PD").expect("the short file of the test");
    assert_eq!(
        Ok(false),
        the_file_starts_as_a_pdf(&of_the_short).map_err(|_| ())
    );

    // A file that is not there is a fault of the disk, and never `false`.
    let absent = place.join("a-file-that-is-not-there.epub");
    assert!(
        the_file_starts_as_a_pdf(&absent).is_err(),
        "a file that is not there gives the fault of the disk"
    );

    let _ = std::fs::remove_dir_all(&place);
}
