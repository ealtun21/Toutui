//! One program writes the files of one download. See T-148.
//!
//! A user can start the program in two terminals, and the two programs hold one
//! account, one database, and **one directory of the downloads**. The key `D` of
//! each of them wrote the same file at the same time: the first program wrote
//! the file from its start, the second program read the bytes that stood on the
//! disk and it added its own bytes to the end of them, and **the file that
//! stayed was not the file of the server**.
//!
//! The rule of this module is the rule of T-142 and of T-147: **the disk is the
//! truth**. A lock file stands in the directory of the download while a program
//! writes it, and a second program that meets that lock writes nothing.
//!
//! **A program that died leaves its lock**, therefore a lock is not for ever.
//! The time of the lock and the time of the file that is not complete both say
//! when that program last worked: a lock that stood still for
//! [`THE_TIME_OF_A_LOCK_THAT_STOOD_STILL`] belongs to a program that is gone,
//! and the next program takes it. This is the rule of the heartbeat of T-140,
//! and it needs no call of the system and no dependency.

use log::info;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The name of the lock inside the directory of the download.
///
/// The name starts with a point, therefore it stands beside the audio files and
/// no name of an audio file agrees with it.
pub const THE_NAME_OF_THE_LOCK: &str = ".the-program-of-the-download";

/// The time after which the lock of a program that died goes away.
///
/// The writer touches the file of the download at each block, therefore a
/// download that lives moves this time. The number is the number of T-140: a
/// program that stood still for 30 seconds is a program that is gone.
pub const THE_TIME_OF_A_LOCK_THAT_STOOD_STILL: Duration = Duration::from_secs(30);

/// The lock of one download.
///
/// The file of the lock goes away with this value: a `return` of the caller, a
/// fault of the download, and a task that stops all remove it.
#[derive(Debug)]
pub struct TheLockOfTheDownload {
    path: PathBuf,
}

impl Drop for TheLockOfTheDownload {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Gives the path of the lock of the download of this directory.
pub fn the_path_of_the_lock(dest_dir: &Path) -> PathBuf {
    dest_dir.join(THE_NAME_OF_THE_LOCK)
}

/// Takes the lock of the download of `dest_dir`.
///
/// It gives `None` when a different program writes these files now. The caller
/// must then write no byte at all.
pub fn take_the_lock(dest_dir: &Path) -> Option<TheLockOfTheDownload> {
    let path = the_path_of_the_lock(dest_dir);

    if make_the_file(&path) {
        return Some(TheLockOfTheDownload { path });
    }

    // A lock stands there already. It belongs to a program that writes these
    // files now, or to a program that died.
    let age = the_time_since_the_last_work(dest_dir);

    if !the_lock_stood_still(age, THE_TIME_OF_A_LOCK_THAT_STOOD_STILL) {
        return None;
    }

    info!(
        "[the-lock-of-the-download] the lock of {} stood still for {:?}, therefore this program takes it",
        dest_dir.display(),
        age
    );

    // The program of that lock is gone. The file of the lock goes away, and
    // this program makes its own. A second program of this same moment makes
    // the file first, and this program then gets nothing: one program wins.
    let _ = std::fs::remove_file(&path);

    if make_the_file(&path) {
        return Some(TheLockOfTheDownload { path });
    }

    None
}

/// Says that a program of this account writes these files now.
///
/// The key `X` needs this answer: **a removal that takes the file of a writer
/// gives that writer a fault, and it gives the user a message of a download
/// that works** (T-150). The rule is the rule of [`take_the_lock`]: a lock that
/// stood still for [`THE_TIME_OF_A_LOCK_THAT_STOOD_STILL`] belongs to a program
/// that is gone, and its files belong to no writer.
pub fn a_program_writes_the_files(dest_dir: &Path) -> bool {
    if !the_path_of_the_lock(dest_dir).exists() {
        return false;
    }

    !the_lock_stood_still(
        the_time_since_the_last_work(dest_dir),
        THE_TIME_OF_A_LOCK_THAT_STOOD_STILL,
    )
}

/// Makes the file of the lock. It gives `false` when the file stands already.
fn make_the_file(path: &Path) -> bool {
    if let Some(directory) = path.parent() {
        let _ = std::fs::create_dir_all(directory);
    }

    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .is_ok()
}

/// The rule of a lock that a program that died left.
///
/// The function is pure, therefore a test needs no file and no time.
pub fn the_lock_stood_still(age: Option<Duration>, limit: Duration) -> bool {
    match age {
        // The program cannot say how old the lock is. It must not take a lock
        // of a program that can live: a file of the user is worth more than a
        // download that the user asks for a second time.
        None => false,
        Some(age) => age >= limit,
    }
}

/// Gives the time since the program of the lock last did work.
///
/// The lock says when that program started, and **the file that is not complete
/// says when it last wrote a block**: a download of an hour therefore holds its
/// lock for that hour. The newest of the two decides.
pub fn the_time_since_the_last_work(dest_dir: &Path) -> Option<Duration> {
    let mut newest: Option<SystemTime> = None;

    let mut hold = |time: SystemTime| {
        if newest.is_none_or(|other| time > other) {
            newest = Some(time);
        }
    };

    if let Ok(data) = std::fs::metadata(the_path_of_the_lock(dest_dir)) {
        if let Ok(time) = data.modified() {
            hold(time);
        }
    }

    if let Ok(rows) = std::fs::read_dir(dest_dir) {
        for row in rows.flatten() {
            let name = row.file_name();
            let is_a_part = name.to_str().is_some_and(|name| name.ends_with(".part"));

            if !is_a_part {
                continue;
            }

            if let Ok(time) = row.metadata().and_then(|data| data.modified()) {
                hold(time);
            }
        }
    }

    newest?.elapsed().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lock stands while the value lives, and it goes away with it.
    #[test]
    fn a_second_program_gets_no_lock_while_the_first_one_holds_it() {
        let dir = tempfile::tempdir().unwrap();
        let download = dir.path().join("item-1");

        let first = take_the_lock(&download).expect("the first program takes the lock");
        assert!(the_path_of_the_lock(&download).exists());

        assert!(
            take_the_lock(&download).is_none(),
            "the second program must get no lock"
        );

        drop(first);

        assert!(
            !the_path_of_the_lock(&download).exists(),
            "the lock goes away with the value"
        );
        assert!(
            take_the_lock(&download).is_some(),
            "the next program takes the lock of a download that ended"
        );
    }

    /// Two downloads of two media do not meet at all.
    #[test]
    fn two_downloads_hold_two_locks() {
        let dir = tempfile::tempdir().unwrap();

        let one = take_the_lock(&dir.path().join("item-1"));
        let two = take_the_lock(&dir.path().join("item-2"));

        assert!(one.is_some());
        assert!(two.is_some());
    }

    #[test]
    fn a_lock_that_stood_still_belongs_to_a_program_that_is_gone() {
        let limit = Duration::from_secs(30);

        // A program that writes now.
        assert!(!the_lock_stood_still(Some(Duration::from_secs(0)), limit));
        assert!(!the_lock_stood_still(Some(Duration::from_secs(29)), limit));

        // A program that is gone.
        assert!(the_lock_stood_still(Some(Duration::from_secs(30)), limit));
        assert!(the_lock_stood_still(Some(Duration::from_secs(600)), limit));

        // The program cannot say. The lock stays.
        assert!(!the_lock_stood_still(None, limit));
    }

    /// A program that died leaves its lock, and the next program takes it. The
    /// user must not lose the key `D` for ever.
    #[test]
    fn the_next_program_takes_the_lock_of_a_program_that_died() {
        let dir = tempfile::tempdir().unwrap();
        let download = dir.path().join("item-1");

        // The lock of the program that died. The value goes away, and the file
        // stays: this is the disk of a program that stopped with no exit.
        let died = take_the_lock(&download).unwrap();
        std::mem::forget(died);

        assert!(
            take_the_lock(&download).is_none(),
            "a lock of this second is the lock of a program that lives"
        );

        // The disk of that program, some minutes later.
        let old = SystemTime::now() - Duration::from_secs(600);
        let file = std::fs::File::open(the_path_of_the_lock(&download)).unwrap();
        file.set_modified(old).unwrap();
        drop(file);

        assert!(
            take_the_lock(&download).is_some(),
            "the lock of a program that is gone must not stop the user"
        );
    }

    /// The key `X` must know if a program writes these files now. See T-150.
    #[test]
    fn a_directory_of_no_lock_holds_no_writer() {
        let dir = tempfile::tempdir().unwrap();
        let download = dir.path().join("item-1");

        // A directory that no program ever wrote, and a directory of a
        // download that ended: no writer stands in either of them.
        assert!(!a_program_writes_the_files(&download));

        let held = take_the_lock(&download).unwrap();
        assert!(a_program_writes_the_files(&download));

        drop(held);
        assert!(!a_program_writes_the_files(&download));
    }

    /// The lock of a program that died must not keep the files of the user for
    /// ever: the key `X` takes them.
    #[test]
    fn the_lock_of_a_program_that_died_holds_no_writer() {
        let dir = tempfile::tempdir().unwrap();
        let download = dir.path().join("item-1");

        let died = take_the_lock(&download).unwrap();
        std::mem::forget(died);

        let old = SystemTime::now() - Duration::from_secs(600);
        let file = std::fs::File::open(the_path_of_the_lock(&download)).unwrap();
        file.set_modified(old).unwrap();
        drop(file);

        assert!(!a_program_writes_the_files(&download));
    }

    /// A download of an hour writes blocks the whole hour, and no second
    /// program may take that lock: the file that is not complete holds the time
    /// of the last block.
    #[test]
    fn the_file_that_grows_holds_the_lock_of_a_long_download() {
        let dir = tempfile::tempdir().unwrap();
        let download = dir.path().join("item-1");

        let held = take_the_lock(&download).unwrap();

        // The lock came one hour before, and the writer wrote a block now.
        let old = SystemTime::now() - Duration::from_secs(3600);
        let lock = std::fs::File::open(the_path_of_the_lock(&download)).unwrap();
        lock.set_modified(old).unwrap();
        drop(lock);

        std::fs::write(download.join("001 - one.mp3.part"), b"a block").unwrap();

        assert!(
            take_the_lock(&download).is_none(),
            "the program of this download works, therefore its lock stays"
        );

        drop(held);
    }
}
