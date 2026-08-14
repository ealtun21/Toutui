//! A read of the disk that failed is not a media with no copy on the disk. See
//! T-203.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME` and `XDG_DATA_HOME`, and those variables belong to the
//! process: two test functions of one binary fight for them. See T-144 and T-157.
//!
//! T-202 gave the fault of the disk to three reads of the queue and of the
//! account, and it left the reads of the **downloads** open. Every one of them
//! gave a fact of the user: an empty list is "this account holds no media on the
//! disk", and `false` of `a_program_keeps_the_place_of_this_media` is "no program
//! of this account plays that media".
//!
//! The measurement of 2026-08-14 with `docs/harness/hold_the_lock.py`: a row of
//! `pending_progress` of this second stood for `Multi File Test Book`, the key `X`
//! of the program said "A program of this account plays "Multi File Test Book" from
//! the disk now." with no lock, and the same key with the lock **removed the three
//! files of that book**:
//!
//! ```text
//! [ERROR] - [delete_download] the program did not open its database: database is locked
//! [INFO]  - [remove_download] the application removed 481839 bytes of the download ac365248-…
//! ```
//!
//! The rows of that download stayed, and no word of the screen named the fault.
//!
//! The second road of the measurement: a database whose statement of the table
//! `downloads` fails. The Library view of the offline mode of T-25 then said
//!
//! ```text
//! The server gave no media: the server does not answer.
//! A media of the disk plays in this mode. Press R when the server answers again.
//! ```
//!
//! while nine downloads stood on the disk and in the database, and no line of the
//! log named that fault.
//!
//! The condition of this test is a file that holds no database: it gives the same
//! fault of `open_conn` with no wait at all (T-200).

use toutui::db::crud;
use toutui::logic::download::{
    remove_download, the_work_of_the_key_that_removes, TheRemovalOfADownload,
    TheWorkOfTheKeyThatRemoves,
};

const THE_USER: &str = "toutuitest";
const THE_SERVER: &str = "http://127.0.0.1:13399";
const THE_KEY: &str = "a book of the disk";

#[test]
fn the_downloads_of_a_disk_that_said_nothing_are_no_fact_of_the_user() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("XDG_DATA_HOME", dir.path());

    // The copy of the disk of one download stands under `XDG_DATA_HOME`, and the
    // program must keep every byte of it.
    let of_the_download = toutui::logic::download::downloads_base_dir(THE_USER).join(THE_KEY);
    std::fs::create_dir_all(&of_the_download).unwrap();
    std::fs::write(of_the_download.join("001 - a.mp3"), b"the audio of a book").unwrap();

    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    std::fs::write(
        dir.path().join("toutui").join("db.sqlite3"),
        b"this file holds no database at all",
    )
    .unwrap();

    // **A read that failed is not an account with no download.** The offline mode
    // of T-25 holds the media of the disk alone, therefore an empty list takes
    // every line of every view away.
    assert!(
        crud::get_all_downloads(THE_USER, THE_SERVER).is_err(),
        "a read of the downloads that failed must not give an account of no download"
    );

    // **A read that failed is not a media with no copy on the disk.**
    assert!(
        crud::get_download(THE_KEY, THE_USER).is_err(),
        "a read of the row of a download that failed must not give a media of no copy"
    );
    assert!(
        crud::get_download_row(THE_KEY, THE_USER).is_err(),
        "a read of the row of a download that failed must not give a media of no copy"
    );
    assert!(
        crud::get_download_files(THE_KEY, THE_USER).is_err(),
        "a read of the files of a download that failed must not give a download of no file"
    );
    assert!(
        toutui::logic::offline::tracks_from_downloads(THE_KEY, THE_USER).is_err(),
        "the track list of a download that the program did not read is no track list"
    );

    // **A read that failed is not a disk with no place that waits** (T-189).
    assert!(
        crud::get_pending_progress(THE_USER, THE_SERVER).is_err(),
        "a read of the places that wait must not give a disk of no place"
    );
    assert!(
        crud::count_pending_progress(THE_USER, THE_SERVER).is_err(),
        "a count of the places that wait must not give the number 0"
    );

    // **A read that failed is not a media that no program plays** (T-156).
    assert!(
        crud::a_program_keeps_the_place_of_this_media(THE_USER, THE_KEY, "").is_err(),
        "a read of the place of a media must not say that no program plays it"
    );

    // The key `X` therefore removes nothing at all: the program does not know
    // which program of this account holds those files.
    assert_eq!(
        the_work_of_the_key_that_removes(false, false, Err(())),
        TheWorkOfTheKeyThatRemoves::TheDatabaseDidNotAnswer,
        "the key must take no file of a media of a database that says nothing"
    );

    assert_eq!(
        remove_download(THE_KEY, THE_USER),
        TheRemovalOfADownload::TheDatabaseSaidNothing,
        "a removal that did not read its database is no removal"
    );

    assert!(
        of_the_download.join("001 - a.mp3").exists(),
        "the file of the user must stay on the disk. The old shape removed the whole \
         directory of the download, and the rows of that download stayed"
    );

    // The words of that key name the database and the key of the work (T-79 and
    // T-170), and they say nothing of the disk of the user.
    let words = toutui::logic::download::text_of_the_database_that_did_not_answer("A Book");

    assert!(words.contains("A Book"), "{}", words);
    assert!(words.contains("database"), "{}", words);
    assert!(words.contains("Press X again"), "{}", words);
    assert!(
        words.contains("removed no file"),
        "the words must say that the disk of the user did not change: {}",
        words
    );

    let words = toutui::logic::download::text_of_the_rows_that_stay("A Book");

    assert!(words.contains("A Book"), "{}", words);
    assert!(words.contains("Press X again"), "{}", words);

    // The view of the offline mode names the disk, and not the server (T-91 and
    // T-172).
    let text =
        toutui::ui::keys::the_text_of_the_library_view_with_no_line(true, true, false, false, None);

    assert_eq!(
        text,
        toutui::ui::keys::THE_LIBRARY_WITH_NO_MEDIA_OF_THE_DISK,
        "the Library view of a disk that said nothing must not name the server"
    );
    assert!(
        !text.contains("the server does not answer"),
        "the program must not say a reason that it does not have: {}",
        text
    );

    // The row of the detail of the views reads the table `downloads` at each
    // frame, and the label of a read that failed says nothing of a media with no
    // copy on the disk.
    use toutui::logic::the_copies_of_the_disk::TheCopyOfTheDisk;

    assert_eq!(
        toutui::ui::keys::the_label_of_the_copy_of_the_disk(TheCopyOfTheDisk::TheDiskDidNotAnswer),
        toutui::ui::keys::THE_DISK_DID_NOT_ANSWER
    );
    assert_eq!(
        toutui::ui::keys::the_label_of_the_copy_of_the_disk(TheCopyOfTheDisk::AWholeCopy),
        toutui::ui::keys::THE_COPY_OF_THE_DISK
    );
    assert_eq!(
        toutui::ui::keys::the_label_of_the_copy_of_the_disk(TheCopyOfTheDisk::NoCopy),
        ""
    );
    // **A copy of the disk that is not whole is no copy of the disk** (T-217).
    assert_eq!(
        toutui::ui::keys::the_label_of_the_copy_of_the_disk(TheCopyOfTheDisk::ACopyThatIsNotWhole),
        toutui::ui::keys::THE_COPY_THAT_IS_NOT_WHOLE
    );

    // **No unit test reaches a key handler of `src/app.rs`**, therefore this part
    // reads the source, as the tests of T-131, T-143, T-149, T-150, T-151, T-156,
    // and T-202 do.
    let source = std::fs::read_to_string("src/app.rs").expect("the test must read src/app.rs");

    assert!(
        source.contains("TheWork::TheDatabaseDidNotAnswer"),
        "the handler of the key `X` must hold the road of a database that says nothing"
    );
    assert!(
        source.contains("TheRemoval::TheRowsOfTheDatabaseStay"),
        "the handler of the key `X` must say that the rows of the database stay"
    );
    assert!(
        source.contains("the_media_of_the_disk_did_not_come"),
        "the offline mode must name the read of the disk that failed"
    );
}
