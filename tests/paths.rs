//! Tests of the paths of the program.
//!
//! The tests use the pure functions that take the parent directory. A
//! variable of the environment is common to all the tests of one process,
//! and therefore a test that writes a variable is not safe.

use std::path::{Path, PathBuf};
use toutui::paths;

/// The directory of configuration is the parent directory and the name of the
/// program.
#[test]
fn the_directory_holds_the_name_of_the_program() {
    let home = Path::new("/tmp/example");
    assert_eq!(
        paths::config_dir_in(home),
        PathBuf::from("/tmp/example/toutui")
    );
}

/// On a system that is not macOS, the parent directory is `.config`.
#[test]
fn the_parent_directory_is_config_on_linux() {
    let home = Path::new("/home/example");
    let expected = if cfg!(target_os = "macos") {
        PathBuf::from("/home/example/Library/Preferences")
    } else {
        PathBuf::from("/home/example/.config")
    };
    assert_eq!(paths::default_config_home(home), expected);
}

/// `config_home` must give the home directory when `XDG_CONFIG_HOME` is
/// empty, and not the current directory. The specification of XDG says that
/// an empty value has the same meaning as an absent value. A program that
/// used the empty value as a real path would resolve the configuration to a
/// path relative to the working directory, and that is a fault.
///
/// This test covers all three states of the variable — absent, empty, and a
/// real path — in one function, because the variable of the environment is
/// common to every test of this process.
#[test]
fn an_empty_variable_gives_the_home_directory_and_not_the_working_directory() {
    let home = dirs::home_dir().expect("Unable to find the user's home directory");
    let expected_default = paths::default_config_home(&home);

    // The variable is absent.
    std::env::remove_var("XDG_CONFIG_HOME");
    assert_eq!(paths::config_home(), expected_default);

    // The variable is present, but it holds no value.
    std::env::set_var("XDG_CONFIG_HOME", "");
    assert_eq!(paths::config_home(), expected_default);

    // The variable holds a real path. That path takes the highest importance.
    std::env::set_var("XDG_CONFIG_HOME", "/tmp/example");
    assert_eq!(paths::config_home(), PathBuf::from("/tmp/example"));

    // Restore the environment for the tests that run after this one.
    std::env::remove_var("XDG_CONFIG_HOME");
}
