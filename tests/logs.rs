//! Tests of the directory that holds the file of the log.
//!
//! The program makes this directory at the start. A new user has no
//! directory, and the program stops with a panic if the directory is absent.

use toutui::utils::logs::make_parent_dir;

/// The function makes a directory that has more than one level.
#[test]
fn the_function_makes_the_directory_of_the_file() {
    let home = tempfile::tempdir().unwrap();
    let file = home.path().join("toutui").join("toutui.log");

    make_parent_dir(&file).unwrap();

    assert!(file.parent().unwrap().is_dir());
    assert!(!file.exists());
}

/// A directory that is present already gives no error.
#[test]
fn a_directory_that_is_present_gives_no_error() {
    let home = tempfile::tempdir().unwrap();
    let file = home.path().join("toutui.log");

    make_parent_dir(&file).unwrap();
    make_parent_dir(&file).unwrap();

    assert!(home.path().is_dir());
}
