//! A disk that took no page of a PDF is not a PDF that gives no page. See
//! T-274.
//!
//! **The measurement of 2026-08-16**, of the real program v0.8.102 inside tmux
//! against the sandbox: the directory of the cache of the ebooks took no write
//! (`chmod 555`), the user pressed the key `e` on a book of 60 pages that the
//! same program read 90 seconds before, and the reader said
//!
//! ```text
//! This PDF gives no page. The file can be damaged. Press h to go back.
//! ```
//!
//! The log of that run held the truth:
//!
//! ```text
//! [pdf] the child stopped with the code 3: toutui: the program did not write
//! the pages: Permission denied (os error 13)
//! ```
//!
//! The book of the user was good. **A view never says a reason that the program
//! does not have** (T-91), and this program had the reason in its hand.
//!
//! These tests need no network and no sandbox.

use std::path::Path;
use toutui::logic::reader::book::ReaderError;
use toutui::logic::reader::pdf_of_a_child::{
    the_fault_of_the_answer_of_the_child, the_reason_of_the_child, the_work_of_the_child,
    THE_CODE_OF_A_BOOK_THAT_GIVES_NO_PAGE, THE_CODE_OF_A_DISK_THAT_TOOK_NO_PAGE,
};

/// Gives the bytes of a PDF of one page that `lopdf` reads.
///
/// The file holds the five objects of a book, a table of the places of them,
/// and one line of text. It needs 537 bytes, therefore this test needs no book
/// of the sandbox and no network.
fn a_pdf_of_one_page() -> Vec<u8> {
    let place = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/a_pdf_of_one_page.pdf");
    std::fs::read(&place).expect("the fixture of the PDF of one page is absent")
}

/// Makes a directory of the test, and it gives the path of it.
fn a_directory(name: &str) -> std::path::PathBuf {
    let place = std::env::temp_dir().join(format!("toutui-t274-{}-{}", name, std::process::id()));

    let _ = std::fs::remove_dir_all(&place);
    std::fs::create_dir_all(&place).expect("the test made no directory");
    place
}

/// Gives the directory the permission of a read alone. It tells if the
/// machine took that permission.
fn the_directory_takes_no_write(place: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut how = std::fs::metadata(place)
            .expect("the test read no directory")
            .permissions();
        how.set_mode(0o555);
        std::fs::set_permissions(place, how).expect("the test set no permission");

        // The user of the root takes every write. A machine of that user gives
        // this test nothing, and the test says so.
        std::fs::write(place.join("a-file-of-the-test"), b"x").is_err()
    }

    #[cfg(not(unix))]
    {
        let _ = place;
        false
    }
}

/// Gives the directory its write again, so the removal of it works.
fn the_directory_takes_a_write(place: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(how) = std::fs::metadata(place) {
            let mut how = how.permissions();
            how.set_mode(0o755);
            let _ = std::fs::set_permissions(place, how);
        }
    }

    #[cfg(not(unix))]
    let _ = place;
}

/// **The condition of the measurement.** The child reads the book, and the disk
/// takes no page of it: the code of that child is the code of the disk, and
/// never the code of a book that gives no page.
#[test]
fn the_child_of_a_disk_that_takes_no_write_gives_the_code_of_the_disk() {
    let place = a_directory("the-disk");
    let book = place.join("a-book.pdf");
    std::fs::write(&book, a_pdf_of_one_page()).expect("the test wrote no book");

    // The child reads the book of this directory first, therefore the book must
    // give a page before the disk refuses anything.
    let good = a_directory("the-disk-good");
    let code = the_work_of_the_child(&book, &good.join("the-pages"));
    assert_eq!(
        code, 0,
        "the child of a good disk read no page of the book of this test"
    );

    if !the_directory_takes_no_write(&place) {
        eprintln!(
            "this machine takes every write of a directory of 555, \
             therefore this test measured nothing"
        );
        the_directory_takes_a_write(&place);
        return;
    }

    let code = the_work_of_the_child(&book, &place.join("a-book.pdf.pages"));

    the_directory_takes_a_write(&place);
    let _ = std::fs::remove_dir_all(&place);
    let _ = std::fs::remove_dir_all(&good);

    assert_eq!(
        code, THE_CODE_OF_A_DISK_THAT_TOOK_NO_PAGE,
        "the child of a disk that takes no write did not give the code of the disk"
    );
    assert_ne!(
        code, THE_CODE_OF_A_BOOK_THAT_GIVES_NO_PAGE,
        "the child said that the book of the user gives no page, and the disk is the fault"
    );
}

/// A file that is no PDF at all keeps the code of a book that gives no page.
#[test]
fn the_child_of_a_book_that_gives_no_page_keeps_the_code_of_the_book() {
    let place = a_directory("the-book");
    let book = place.join("a-book.pdf");
    std::fs::write(&book, b"%PDF-1.4\nthis file holds no object of a book\n")
        .expect("the test wrote no book");

    let code = the_work_of_the_child(&book, &place.join("a-book.pdf.pages"));

    let _ = std::fs::remove_dir_all(&place);

    assert_eq!(
        code, THE_CODE_OF_A_BOOK_THAT_GIVES_NO_PAGE,
        "a file that holds no page did not give the code of a book that gives no page"
    );
}

/// **The rule of this item.** The parent reads the code of the child, and the
/// user of a disk that took no page never reads that their book can be damaged.
#[test]
fn the_words_of_a_disk_that_took_no_page_name_the_disk_and_not_the_book() {
    let words = "toutui: the program did not write the pages: Permission denied (os error 13)\n";
    let fault =
        the_fault_of_the_answer_of_the_child(Some(THE_CODE_OF_A_DISK_THAT_TOOK_NO_PAGE), words);

    assert_eq!(
        fault,
        ReaderError::TheDiskTookNoPageOfThePdf("Permission denied (os error 13)".to_string()),
        "the code of a disk that took no page gave another fault"
    );

    let said = fault.to_string();

    assert!(
        said.contains("disk"),
        "the words of a disk that took no page name no disk: {said}"
    );
    assert!(
        said.contains("Permission denied (os error 13)"),
        "the words say nothing of what the machine said: {said}"
    );
    assert!(
        said.contains("The book is good"),
        "the words do not say that the book of the user is good: {said}"
    );
    assert!(
        !said.contains("can be damaged"),
        "the words say that the book of the user can be damaged, and the disk is the fault: {said}"
    );
    assert!(
        said.contains("Press h"),
        "the words name no key of the view that the user sees (T-170): {said}"
    );
}

/// The code of a book that gives no page keeps the words of a book that gives
/// no page. **A correction of the words must not take the true sentence away.**
#[test]
fn the_words_of_a_book_that_gives_no_page_stay_the_words_of_the_book() {
    let fault = the_fault_of_the_answer_of_the_child(
        Some(THE_CODE_OF_A_BOOK_THAT_GIVES_NO_PAGE),
        "toutui: this PDF gives no page: This file is not an EPUB.\n",
    );

    assert_eq!(fault, ReaderError::ThePdfGivesNoPage);
    assert!(
        fault.to_string().contains("can be damaged"),
        "the words of a book that gives no page went away: {fault}"
    );
}

/// Every other code of a child is a part of the program that did not do its
/// work, and the book of the user is not the reason.
#[test]
fn every_other_code_of_a_child_says_that_the_book_can_be_good() {
    for code in [1, -1, 101] {
        let fault = the_fault_of_the_answer_of_the_child(Some(code), "");
        let said = fault.to_string();

        assert!(
            matches!(fault, ReaderError::ThePartThatReadsAPdfFailed(_)),
            "the code {code} gave the fault of a book of the user"
        );
        assert!(
            !said.contains("can be damaged"),
            "the code {code} said that the book of the user can be damaged: {said}"
        );
        assert!(
            said.contains("Press h"),
            "the code {code} named no key of the view that the user sees: {said}"
        );
    }
}

/// A child that did not come back is a part of the program that took too long,
/// and the book of the user is not the reason.
#[test]
fn a_child_that_did_not_come_back_says_that_the_book_can_be_good() {
    let fault = the_fault_of_the_answer_of_the_child(None, "");
    let said = fault.to_string();

    assert!(
        matches!(fault, ReaderError::ThePartThatReadsAPdfFailed(_)),
        "a child that did not come back gave the fault of a book of the user"
    );
    assert!(
        said.contains("minutes"),
        "the words say nothing of the time that the program waited: {said}"
    );
    assert!(
        !said.contains("can be damaged"),
        "the words say that the book of the user can be damaged: {said}"
    );
}

/// A child that said nothing gives no reason, and the sentence of the user
/// stays true and whole.
#[test]
fn a_child_that_said_nothing_gives_a_sentence_with_no_reason() {
    for said in ["", "\n", "a line of another program"] {
        assert_eq!(
            the_reason_of_the_child(said),
            "",
            "the words {said:?} gave a reason of the machine"
        );
    }

    let words = ReaderError::TheDiskTookNoPageOfThePdf(String::new()).to_string();

    assert!(
        !words.contains("The machine said"),
        "a fault with no reason names a machine that said nothing: {words}"
    );
    assert!(
        words.contains("disk") && words.contains("Press h"),
        "the words of a fault with no reason are not whole: {words}"
    );
}

/// **A user must read no line of a source** (T-172). The three sentences of
/// this item name no file of a crate and no number of a line.
#[test]
fn the_words_of_this_item_name_no_file_of_a_source() {
    let all = [
        ReaderError::ThePdfGivesNoPage.to_string(),
        ReaderError::TheDiskTookNoPageOfThePdf("Permission denied (os error 13)".to_string())
            .to_string(),
        ReaderError::ThePartThatReadsAPdfFailed("The part that reads a PDF stopped.".to_string())
            .to_string(),
    ];

    for words in all {
        for mark in [".rs", "src/", "panicked", "unwrap"] {
            assert!(
                !words.contains(mark),
                "the words hold {mark}, and a user reads no line of a source: {words}"
            );
        }
    }
}
