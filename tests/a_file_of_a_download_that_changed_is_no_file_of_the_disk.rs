//! A file of the disk that is not the file of the row is no file of the download.
//! See T-216.
//!
//! **T-215 asks one question of the file system: does the path stand?** A file
//! that lost bytes stands, therefore it passed that check: the row of
//! `download_files` said 160613 bytes, the disk held 80000 of them, and the
//! program called that copy of the disk whole.
//!
//! **The measurement of 2026-08-14** of the real program of the sandbox, of
//! `Multi File Test Book` of three files of 20 seconds, with the second file at
//! half of its bytes (`truncate -s 80000`) and the server up:
//!
//! ```text
//! [INFO] [play] the download ac365248-… gives 3 of 3 track(s) from the disk
//! [INFO] [worker] the playback starts at 0 seconds
//! [INFO] [follow_playback] the playback stopped at 60 seconds, finished=true
//! ```
//!
//! `ffmpeg` says that the file of the disk holds **9.95 seconds** of audio, and
//! the row of it says 20. **The user heard 50 seconds of a book of 60**, no word
//! came to the screen, and the program then told the server:
//!
//! ```text
//! {'currentTime': 60, 'progress': 1, 'isFinished': True, 'duration': 60}
//! ```
//!
//! **A value that the program sends to the server outlives the program that sent
//! it** (T-193): every client of that account holds the book as read, and the ten
//! seconds that no machine played go away with it.
//!
//! **A size of 0 is a size that the server did not give** (T-179), therefore a row
//! of that size keeps its file: the program has no length to compare.
//!
//! **This test needs no sandbox and no server.** A directory of the machine holds
//! the files of the book, and one `set_len` takes the bytes of one of them away.

use toutui::player::engine::source::{select_sources, TrackSource};
use toutui::player::engine::track::Track;

const THE_ACCOUNT: &str = "the-account-of-a-file-that-changed";
const THE_SERVER: &str = "the-server-of-a-file-that-changed";
const THE_BOOK: &str = "the-book-of-a-file-that-changed";

/// The bytes of a whole file of this book.
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

/// Writes the row of one file of the book. The size is the size of the server.
fn the_row_of_the_file(index: u32, path: &std::path::Path, size: u64) {
    toutui::db::crud::insert_download_file(
        THE_BOOK,
        THE_ACCOUNT,
        index,
        &format!("ino-{index}"),
        &path.to_string_lossy(),
        size,
        20.0,
    )
    .expect("the row of the file");
}

/// The three files of the book, on the disk and in the rows of the database.
fn the_book_of_three_files(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::create_dir_all(directory).expect("the directory of the book");

    let mut paths = Vec::new();

    for index in 1..=3u32 {
        let path = directory.join(format!("00{index} - Part {index}.mp3"));
        std::fs::write(&path, THE_BYTES).expect("the file of the book");
        the_row_of_the_file(index, &path, THE_BYTES.len() as u64);
        paths.push(path);
    }

    toutui::db::crud::insert_download(
        THE_BOOK,
        THE_ACCOUNT,
        "A Book Of Three Files",
        "An Author",
        &paths[0].to_string_lossy(),
        60.0,
        THE_BOOK,
        THE_SERVER,
    )
    .expect("the row of the download");

    paths
}

/// The tracks of the book of the server.
fn the_tracks_of_the_book() -> Vec<Track> {
    (1..=3u32)
        .map(|index| Track {
            index,
            ino: format!("10{index}"),
            filename: format!("0{index} - Part {index}.mp3"),
            mime_type: None,
            size: Some(THE_BYTES.len() as u64),
            duration: 20.0,
            start_offset: 20.0 * (index - 1) as f64,
        })
        .collect()
}

/// The number of the tracks that take the disk, and the number that take the
/// server.
fn the_sources_of_the_playback() -> (usize, usize) {
    let sources = select_sources(
        THE_BOOK,
        THE_BOOK,
        THE_ACCOUNT,
        "http://the-server-of-this-test",
        &the_tracks_of_the_book(),
    );

    let of_the_disk = sources
        .iter()
        .filter(|source| matches!(source, TrackSource::Local(_)))
        .count();

    (of_the_disk, sources.len() - of_the_disk)
}

/// A file that lost bytes takes the road of the server.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[test]
fn a_file_whose_size_is_not_the_size_of_its_row_takes_the_road_of_the_server() {
    let home = temporary_home();
    let directory = home.join("the-files-of-the-book");

    let paths = the_book_of_three_files(&directory);

    // The whole book stands on the disk, therefore every track takes the disk.
    assert_eq!(
        the_sources_of_the_playback(),
        (3, 0),
        "a whole copy of the disk plays from the disk"
    );

    // **The file of the middle loses its bytes, and its path stands.** This is the
    // condition that T-215 does not reach.
    let shorter = std::fs::OpenOptions::new()
        .write(true)
        .open(&paths[1])
        .expect("the file of the middle");
    shorter.set_len(10).expect("the file loses its bytes");
    drop(shorter);

    assert_eq!(
        the_sources_of_the_playback(),
        (0, 3),
        "a file that is not the file of its row takes the road of the server"
    );

    let files = toutui::db::crud::get_download_files(THE_BOOK, THE_ACCOUNT)
        .expect("the rows of the files of the download");

    assert_eq!(files.len(), 3, "the rows of the database stay");
    assert_eq!(
        toutui::logic::offline::the_files_that_stand_on_the_disk(files).len(),
        2,
        "the disk holds two files of the three of the book"
    );

    // **A file of more bytes than its row is no file of that row either.**
    std::fs::write(&paths[1], b"the bytes of a file of a book and more of them")
        .expect("a file of more bytes");

    assert_eq!(
        the_sources_of_the_playback(),
        (0, 3),
        "a file of more bytes than its row takes the road of the server"
    );

    // The bytes of the file come back, and the whole book plays from the disk.
    std::fs::write(&paths[1], THE_BYTES).expect("the bytes come back");

    assert_eq!(
        the_sources_of_the_playback(),
        (3, 0),
        "a file of the bytes of its row gives the road of the disk again"
    );

    // **A size of 0 is a size that the server did not give** (T-179). The program
    // holds no length of that file, therefore the file of the disk stands.
    the_row_of_the_file(2, &paths[1], 0);
    std::fs::write(&paths[1], b"a file of another number of bytes").expect("another file");

    assert_eq!(
        the_sources_of_the_playback(),
        (3, 0),
        "a row of no size keeps the file of the disk"
    );
}
