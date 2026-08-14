//! The render of the program reads no disk. See T-204.
//!
//! **A read of the disk that stands inside the render is a read of every
//! frame.** The row of the detail of six views held `get_download_of_a_frame`
//! (T-203) and the row of the player held `get_is_show_key_bindings`, therefore
//! the thread of the screen asked the database at each frame.
//!
//! A measurement of 2026-08-14 with `docs/harness/hold_the_lock.py`, which takes
//! the write lock of the database of the program (T-199): **five presses of the
//! key `j` moved no cursor for the 30 seconds of the lock**, the row of the
//! player stood 13 minutes behind the playback of the book, and the row of the
//! keys of the player went away while the user turned nothing off. rusqlite
//! holds a busy timeout of five seconds, therefore each of those frames cost
//! five seconds.
//!
//! The box of `logic::the_copies_of_the_disk` and two values of the `App` hold
//! those answers now. This test reads the source of the render, as the tests of
//! T-135 and of T-143 do: a later session that puts a call of the database back
//! in the render fails here.

use std::path::{Path, PathBuf};

/// The files of the program that the render calls at each frame.
///
/// `src/ui/` holds every screen, and `player_info` stands in the loop of
/// `src/main.rs` before the draw of the frame.
fn the_files_of_the_render() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut files = vec![root.join("src/player/integrated/player_info.rs")];

    for entry in std::fs::read_dir(root.join("src/ui")).expect("the directory src/ui") {
        let path = entry.expect("one file of src/ui").path();

        if path.extension().is_some_and(|kind| kind == "rs") {
            files.push(path);
        }
    }

    files
}

/// The words of a call of the database.
const THE_WORDS_OF_THE_DATABASE: [&str; 2] = ["db::crud", "crate::db::"];

/// No file of the render calls the database.
///
/// **The disk holds the thread of the screen.** A second program of the account
/// that writes the database gives every call of this program the busy timeout of
/// rusqlite, and a call of each frame then draws one frame in five seconds: no
/// key of the user comes to the screen inside the life of a message of six
/// seconds (T-203 and T-204).
#[test]
fn no_file_of_the_render_calls_the_database() {
    for file in the_files_of_the_render() {
        let source = std::fs::read_to_string(&file).expect("the source of a file of the render");

        for words in THE_WORDS_OF_THE_DATABASE {
            assert!(
                !source.contains(words),
                "the file {} of the render holds \"{}\": the render reads the box of \
                 logic::the_copies_of_the_disk or a value of the App, and it reads no \
                 disk. See T-204.",
                file.display(),
                words,
            );
        }
    }
}

/// The key `B` writes the value that the `App` holds.
///
/// **A key that reads a state of the disk and that then writes it** (the shape
/// of T-175) did nothing at all when that read failed: the value was neither
/// "0" nor "1", therefore no branch wrote the disk, and no word told the user.
/// A measurement of 2026-08-14 pressed that key while a second program held the
/// database, and the disk kept the value of the start with no sentence and no
/// line of the log.
#[test]
fn the_key_that_shows_the_keys_of_the_player_reads_no_disk() {
    let source = include_str!("../src/app.rs");

    let start = source
        .find("KeyCode::Char('B') => {")
        .expect("the handler of the key B");
    let block = &source[start..start + 1200];

    assert!(
        !block.contains("get_is_show_key_bindings"),
        "the key B must read the value of the App and not the disk: a read that \
         failed made the key do nothing at all. See T-204."
    );

    assert!(
        block.contains("THE_KEYS_OF_THE_PLAYER_DID_NOT_REACH_THE_DISK"),
        "the key B is a key of the user, therefore a write that failed takes a \
         sentence for that user. See T-199 and T-204."
    );
}
