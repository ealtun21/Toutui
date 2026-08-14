//! The label `[Downloaded]` of a media whose copy of the disk is not whole. See
//! T-217.
//!
//! **T-215 and T-216 gave the playback the rule of the disk, and the line of the
//! user kept the label of before.** The label comes of the table `downloads` and
//! of the box of T-204, and no read of the file system stood behind it: the line
//! of a book whose file went away, or whose file lost its bytes, said
//! `[Downloaded]` while every playback of that book took the road of the server.
//!
//! **The measurement of 2026-08-14** of the real program of the sandbox, of
//! `Multi File Test Book` of three files, with the second file at half of its
//! bytes:
//!
//! ```text
//! ➤ Multi File Test Book
//! Author: Test Author - Year: N/A - [Downloaded]
//! ```
//!
//! ```text
//! [INFO] [play] the download ac365248-… gives 0 of 3 track(s) from the disk
//! ```
//!
//! **The program said that the media stands on the disk, and it read the book of
//! the server.** The offline mode of the same book says "The disk does not hold
//! every file of this media." and it plays nothing at all.
//!
//! **The box reads the file system now**, and it stands outside the render
//! (T-204): the program calls it at the start, at the key `R`, at the end of a
//! download, and at the key `X`.
//!
//! **This test needs no sandbox and no server.** A directory of the machine holds
//! the files of the book.

use toutui::logic::the_copies_of_the_disk::{
    read_the_disk, the_copy_of_this_media, TheCopyOfTheDisk,
};
use toutui::ui::keys::{
    the_label_of_the_copy_of_the_disk, THE_COPY_OF_THE_DISK, THE_COPY_THAT_IS_NOT_WHOLE,
};

const THE_ACCOUNT: &str = "the-account-of-the-label-of-a-copy";
const THE_SERVER: &str = "the-server-of-the-label-of-a-copy";
const THE_BOOK: &str = "the-book-of-the-label-of-a-copy";
const THE_BOOK_OF_ONE_FILE: &str = "the-second-book-of-the-label-of-a-copy";

/// The bytes of a whole file of these books.
const THE_BYTES: &[u8] = b"the bytes of a file of a book";

static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();

fn temporary_home() -> std::path::PathBuf {
    let dir = HOME.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        std::env::set_var("XDG_DATA_HOME", dir.path());
        std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

        let conn = toutui::db::migrate::open_conn().unwrap();
        toutui::db::migrate::run_migrations(&conn).unwrap();
        drop(conn);

        dir
    });

    dir.path().to_path_buf()
}

/// Writes the row of a download of one media, and the row of each of its files.
fn the_download_of(key: &str, directory: &std::path::Path, files: u32) -> Vec<std::path::PathBuf> {
    std::fs::create_dir_all(directory).expect("the directory of the book");

    let mut paths = Vec::new();

    for index in 1..=files {
        let path = directory.join(format!("00{index} - Part {index}.mp3"));
        std::fs::write(&path, THE_BYTES).expect("the file of the book");

        toutui::db::crud::insert_download_file(
            key,
            THE_ACCOUNT,
            index,
            &format!("ino-{index}"),
            &path.to_string_lossy(),
            THE_BYTES.len() as u64,
            20.0,
        )
        .expect("the row of the file");

        paths.push(path);
    }

    toutui::db::crud::insert_download(
        key,
        THE_ACCOUNT,
        "A Book Of The Disk",
        "An Author",
        &paths[0].to_string_lossy(),
        20.0 * files as f64,
        key,
        THE_SERVER,
    )
    .expect("the row of the download");

    paths
}

/// The label of the line of one media.
fn the_label_of(key: &str) -> &'static str {
    the_label_of_the_copy_of_the_disk(the_copy_of_this_media(key))
}

/// The label of a line says what the disk holds.
///
/// **The parts of this test stay in one function**: the box is a box of the
/// process, and two test functions of one binary fight for it (T-144 and T-157).
#[test]
fn the_label_of_a_media_whose_copy_is_not_whole_says_what_the_disk_holds() {
    let home = temporary_home();
    let directory = home.join("the-files-of-the-books");

    let paths = the_download_of(THE_BOOK, &directory.join("the-book"), 3);
    the_download_of(THE_BOOK_OF_ONE_FILE, &directory.join("the-second-book"), 1);

    // The disk holds every file of the two books.
    read_the_disk(THE_ACCOUNT);

    assert_eq!(
        the_copy_of_this_media(THE_BOOK),
        TheCopyOfTheDisk::AWholeCopy
    );
    assert_eq!(the_label_of(THE_BOOK), THE_COPY_OF_THE_DISK);
    assert_eq!(the_label_of(THE_BOOK_OF_ONE_FILE), THE_COPY_OF_THE_DISK);

    // **One file of the first book loses its bytes** (T-216). The playback of that
    // book takes the road of the server, therefore the label of it must not say
    // that the disk holds the book.
    let shorter = std::fs::OpenOptions::new()
        .write(true)
        .open(&paths[1])
        .expect("the file of the middle");
    shorter.set_len(10).expect("the file loses its bytes");
    drop(shorter);

    read_the_disk(THE_ACCOUNT);

    assert_eq!(
        the_copy_of_this_media(THE_BOOK),
        TheCopyOfTheDisk::ACopyThatIsNotWhole
    );
    assert_eq!(the_label_of(THE_BOOK), THE_COPY_THAT_IS_NOT_WHOLE);

    // **The other book keeps its label**: the label of one media says the files of
    // that media alone.
    assert_eq!(the_label_of(THE_BOOK_OF_ONE_FILE), THE_COPY_OF_THE_DISK);

    // **A file that went away holds the same road** (T-215).
    std::fs::write(&paths[1], THE_BYTES).expect("the bytes come back");
    std::fs::remove_file(&paths[2]).expect("the file of the end goes away");

    read_the_disk(THE_ACCOUNT);

    assert_eq!(the_label_of(THE_BOOK), THE_COPY_THAT_IS_NOT_WHOLE);

    // The file comes back, and the label of the whole copy comes with it.
    std::fs::write(&paths[2], THE_BYTES).expect("the file comes back");

    read_the_disk(THE_ACCOUNT);

    assert_eq!(the_label_of(THE_BOOK), THE_COPY_OF_THE_DISK);

    // **A download with no row of a file is a download of one half** (T-214), and
    // no second of it plays from the disk.
    toutui::db::crud::keep_the_files_of_the_download(THE_BOOK, THE_ACCOUNT, &[])
        .expect("the rows of the files go away");

    read_the_disk(THE_ACCOUNT);

    assert_eq!(the_label_of(THE_BOOK), THE_COPY_THAT_IS_NOT_WHOLE);

    // A media of no download at all holds no label.
    assert_eq!(the_label_of("the-book-of-no-download"), "");
}
