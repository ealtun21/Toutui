//! A copy of the disk that is not whole is no copy of the disk. See T-215.
//!
//! **A row of `download_files` is no file of the disk.** The row holds the path of
//! a file that the program wrote, and a file goes away outside this program: the
//! user removes it, a directory of the machine goes away, or a removal of a
//! download takes the files of the disk and leaves the rows (T-214). Every caller
//! of that table read the rows and called them the copy of the disk.
//!
//! **The measurement of 2026-08-14** of the real program of the sandbox, of
//! `Multi File Test Book` of three files of 20 seconds, with the second file away
//! and the server up:
//!
//! ```text
//! [INFO] [play] the download ac365248-… gives 3 of 3 track(s) from the disk
//! [WARN] [worker] the engine cannot open the track 2 of 3: The application cannot
//!     open the file: No such file or directory (os error 2). The tracks before it play.
//! [INFO] [follow_playback] the playback stopped at 20 seconds, finished=false
//! ```
//!
//! **The program played 20 seconds of a book of 60 and it said nothing at all**,
//! while the whole book stood on the server. The row of the player held the length
//! of the whole book, and the place 20 seconds went to the server.
//!
//! **The offline mode said a playback that did not exist.** With
//! `podman stop -t 0 abs-test`, the place of the user at 20 seconds, and the same
//! file away, the key `l` of the Library view of the offline mode said:
//!
//! ```text
//!    Offline: "Multi File Test Book" plays from the disk.
//! ```
//!
//! ```text
//! [INFO] [play] the offline mode plays Multi File Test Book at 20 seconds with 3 track(s)
//! [ERROR] [worker] the engine cannot start the book: The application cannot open
//!     the file: No such file or directory (os error 2)
//! ```
//!
//! **No sound came, and no word of the fault came.** The check of `play_offline`
//! holds the sentence of that condition already — "The disk does not hold every
//! file of this media." — and it compared the files of the book with the files of
//! **the same table**: a file that went away passed that check every time.
//!
//! **This test needs no sandbox and no server.** A directory of the machine holds
//! the files of the book, and one `remove_file` takes one of them away.

use toutui::player::engine::source::{select_sources, TrackSource};
use toutui::player::engine::track::Track;

const THE_ACCOUNT: &str = "the-account-of-a-copy-that-is-not-whole";
const THE_SERVER: &str = "the-server-of-a-copy-that-is-not-whole";
const THE_BOOK: &str = "the-book-of-a-copy-that-is-not-whole";

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

/// The three files of the book, on the disk and in the rows of the database.
fn the_book_of_three_files(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::create_dir_all(directory).expect("the directory of the book");

    let mut paths = Vec::new();

    for index in 1..=3u32 {
        let path = directory.join(format!("00{index} - Part {index}.mp3"));
        std::fs::write(&path, b"the bytes of a file of a book").expect("the file of the book");

        toutui::db::crud::insert_download_file(
            THE_BOOK,
            THE_ACCOUNT,
            index,
            &format!("ino-{index}"),
            &path.to_string_lossy(),
            28,
            20.0,
        )
        .expect("the row of the file");

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
            size: Some(28),
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

/// A copy of the disk that is not whole takes the road of the server.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary take a thread each, and `cargo test` finds a fault of that shape at
/// one run of six (T-144 and T-157).
#[test]
fn a_file_of_a_download_that_went_away_takes_the_road_of_the_server() {
    let home = temporary_home();
    let directory = home.join("the-files-of-the-book");

    let paths = the_book_of_three_files(&directory);

    // The whole book stands on the disk, therefore every track takes the disk.
    assert_eq!(
        the_sources_of_the_playback(),
        (3, 0),
        "a whole copy of the disk plays from the disk"
    );

    // **One file of the book goes away, and the rows of the database stay.**
    std::fs::remove_file(&paths[1]).expect("the file of the middle goes away");

    assert_eq!(
        the_sources_of_the_playback(),
        (0, 3),
        "a copy of the disk that is not whole takes the road of the server"
    );

    // **The offline mode reads the same rows** (`play_offline`), and its check of
    // the files of the disk compared the rows with the rows. The files that stand
    // on the disk are two of the three now, therefore that check finds the media
    // that the disk does not hold whole.
    let files = toutui::db::crud::get_download_files(THE_BOOK, THE_ACCOUNT)
        .expect("the rows of the files of the download");

    assert_eq!(files.len(), 3, "the rows of the database stay");
    assert_eq!(
        toutui::logic::offline::the_files_that_stand_on_the_disk(files).len(),
        2,
        "the disk holds two files of the three of the book"
    );

    // The file comes back, and the whole book plays from the disk again.
    std::fs::write(&paths[1], b"the bytes of a file of a book").expect("the file comes back");

    assert_eq!(
        the_sources_of_the_playback(),
        (3, 0),
        "a file that came back gives the road of the disk again"
    );
}
