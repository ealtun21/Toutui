//! The key `X` takes the disk of a download, and not the row of the database.
//! See T-150.
//!
//! **The database holds a row after the last byte of the last file**, therefore
//! the bytes of a download that stopped stand in no row at all: `remove_download`
//! read the database, it found nothing, and the program said "holds no local
//! copy" while some megabytes of a `.part` file stayed on the disk for ever. No
//! key of the program removed them.
//!
//! **A download that runs holds its files**, and the key must take none of them:
//! a removal under a writer gives that writer the fault of T-148 from the other
//! side.
//!
//! **This test writes `XDG_DATA_HOME` and `XDG_CONFIG_HOME`, therefore it must
//! stay alone in its binary and it must hold every part in one function.** A
//! variable of the environment belongs to the process. See the trap 25 and the
//! trap 8 of the harness of `docs/HANDOVER.md`.

use std::fs;
use std::time::{Duration, SystemTime};

use toutui::logic::download::{
    a_program_downloads, remove_download, remove_the_directory_of_the_download,
    the_work_of_the_key_that_removes, TheAudioOfTheRemoval, TheWorkOfTheKeyThatRemoves,
};

const THE_USER: &str = "a user";
const THE_KEY: &str = "an item of a download";
const THE_LOCK: &str = ".the-program-of-the-download";

#[test]
fn the_key_x_takes_the_disk_of_a_download_that_the_database_does_not_hold() {
    let directory = std::env::temp_dir().join(format!(
        "toutui-the-key-x-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|when| when.as_nanos())
            .unwrap_or(0)
    ));

    fs::create_dir_all(&directory).expect("the test must make its directory");

    // The variables must come before the first call of `downloads_base_dir`.
    unsafe {
        std::env::set_var("XDG_DATA_HOME", &directory);
        std::env::set_var("XDG_CONFIG_HOME", &directory);
    }

    let of_the_download = directory
        .join("toutui")
        .join("downloads")
        .join(THE_USER)
        .join(THE_KEY);

    // 1. A download that runs: the lock stands, and the writer wrote now.
    fs::create_dir_all(&of_the_download).expect("the test must make the directory of the download");
    fs::write(of_the_download.join("001 - a.mp3.part"), vec![b'x'; 1000])
        .expect("the test must write the part of the download");
    fs::write(of_the_download.join(THE_LOCK), b"").expect("the test must write the lock");

    assert!(
        a_program_downloads(THE_KEY, THE_USER),
        "a lock of this second belongs to a program that writes these files"
    );

    assert_eq!(
        the_work_of_the_key_that_removes(false, a_program_downloads(THE_KEY, THE_USER)),
        TheWorkOfTheKeyThatRemoves::ADifferentProgramDownloads,
        "the key must take no file of a download that runs"
    );

    // 2. The program of that lock died: the lock and the part of the download
    //    stood still for more than 30 seconds.
    let long_ago = SystemTime::now() - Duration::from_secs(600);

    for name in [THE_LOCK, "001 - a.mp3.part"] {
        let file = fs::File::open(of_the_download.join(name)).expect("the test must open the file");
        file.set_modified(long_ago)
            .expect("the test must give the file its time");
    }

    assert!(
        !a_program_downloads(THE_KEY, THE_USER),
        "a lock that stood still belongs to a program that is gone"
    );

    assert_eq!(
        the_work_of_the_key_that_removes(false, a_program_downloads(THE_KEY, THE_USER)),
        TheWorkOfTheKeyThatRemoves::TakeTheDisk
    );

    // 3. The key takes the disk. The database holds no row of this download,
    //    and the bytes must go away all the same.
    let (title, of_the_audio) = remove_download(THE_KEY, THE_USER);

    assert_eq!(title, None, "the database holds no row of this download");
    assert_eq!(
        of_the_audio,
        TheAudioOfTheRemoval::ThePartOfADownload(1000),
        "the key must remove the bytes of a download that did not come to its end"
    );
    assert!(
        !of_the_download.exists(),
        "the directory of the download must go away, with its lock"
    );

    // 4. A media that the disk does not hold at all gives no fault, and it
    //    gives no false sentence.
    let (title, of_the_audio) = remove_download(THE_KEY, THE_USER);
    assert_eq!(title, None);
    assert_eq!(of_the_audio, TheAudioOfTheRemoval::Nothing);
    assert_eq!(remove_the_directory_of_the_download(THE_KEY, THE_USER), 0);

    // 5. The handler of the key `X` must ask the lock before it removes a file.
    //    **No unit test reaches a key handler of `src/app.rs`**, therefore this
    //    test reads the source, as the tests of T-131, T-143, and T-149 do.
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("the test must read src/app.rs");

    let of_the_key = source
        .split_once("KeyCode::Char('X') => {")
        .expect("the handler of the key X must stand in src/app.rs")
        .1;

    let place_of_the_removal = of_the_key
        .find("remove_download(")
        .expect("the key X must remove the download");
    let place_of_the_question = of_the_key
        .find("the_work_of_the_key_that_removes(")
        .expect("the key X must ask what it may remove");

    assert!(
        place_of_the_question < place_of_the_removal,
        "the key X must ask the lock before it removes a file. See T-150."
    );

    let _ = fs::remove_dir_all(&directory);
}
