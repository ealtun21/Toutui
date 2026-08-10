//! Tests of the paths of the program.
//!
//! The tests use the pure functions that take the parent directory. A
//! variable of the environment is common to all the tests of one process,
//! and therefore a test that writes a variable is not safe.

use std::path::{Path, PathBuf};
use abstui::paths;

/// The directory of configuration is the parent directory and the name of the
/// program.
#[test]
fn the_directory_holds_the_name_of_the_program() {
    let home = Path::new("/tmp/example");
    assert_eq!(
        paths::config_dir_in(home),
        PathBuf::from("/tmp/example/abstui")
    );
}

/// The old directory holds the name of the program before the fork.
#[test]
fn the_old_directory_holds_the_old_name() {
    let home = Path::new("/tmp/example");
    assert_eq!(
        paths::old_config_dir_in(home),
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

use std::fs;

/// Makes a parent directory that holds an old directory with the three files.
fn old_installation(config_home: &Path) {
    let old = paths::old_config_dir_in(config_home);
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("config.toml"), "[colors]\n").unwrap();
    fs::write(old.join(".env"), "TOUTUI_SECRET_KEY=abc123\n").unwrap();
    fs::write(old.join("db.sqlite3"), b"not a real database").unwrap();
}

/// The program copies the three files when only the old directory is present.
#[test]
fn the_program_copies_the_old_files() {
    let home = tempfile::tempdir().unwrap();
    old_installation(home.path());

    let copied = paths::migrate_old_config(home.path()).unwrap();

    let new = paths::config_dir_in(home.path());
    assert!(copied);
    assert_eq!(fs::read_to_string(new.join("config.toml")).unwrap(), "[colors]\n");
    assert_eq!(fs::read(new.join("db.sqlite3")).unwrap(), b"not a real database");
}

/// The copy writes the new name of the key, and it keeps the value.
#[test]
fn the_copy_changes_the_name_of_the_key() {
    let home = tempfile::tempdir().unwrap();
    old_installation(home.path());

    paths::migrate_old_config(home.path()).unwrap();

    let env = fs::read_to_string(paths::config_dir_in(home.path()).join(".env")).unwrap();
    assert_eq!(env, "ABSTUI_SECRET_KEY=abc123\n");
}

/// The old directory does not change.
#[test]
fn the_old_directory_stays_complete() {
    let home = tempfile::tempdir().unwrap();
    old_installation(home.path());

    paths::migrate_old_config(home.path()).unwrap();

    let old = paths::old_config_dir_in(home.path());
    assert_eq!(
        fs::read_to_string(old.join("config.toml")).unwrap(),
        "[colors]\n"
    );
    assert_eq!(
        fs::read_to_string(old.join(".env")).unwrap(),
        "TOUTUI_SECRET_KEY=abc123\n"
    );
    assert!(old.join("db.sqlite3").exists());
}

/// The program does not write when the new directory is present.
#[test]
fn the_program_does_not_write_on_a_new_installation() {
    let home = tempfile::tempdir().unwrap();
    old_installation(home.path());
    let new = paths::config_dir_in(home.path());
    fs::create_dir_all(&new).unwrap();
    fs::write(new.join("config.toml"), "the file of the user\n").unwrap();

    let copied = paths::migrate_old_config(home.path()).unwrap();

    assert!(!copied);
    assert_eq!(
        fs::read_to_string(new.join("config.toml")).unwrap(),
        "the file of the user\n"
    );
    assert!(!new.join(".env").exists());
    assert!(!new.join("db.sqlite3").exists());
}

/// The program makes the new directory when no directory is present.
#[test]
fn the_program_makes_the_directory_for_a_first_installation() {
    let home = tempfile::tempdir().unwrap();

    let copied = paths::migrate_old_config(home.path()).unwrap();

    assert!(!copied);
    assert!(paths::config_dir_in(home.path()).is_dir());
}

/// A file that the old directory does not have gives no error.
#[test]
fn a_file_that_is_absent_gives_no_error() {
    let home = tempfile::tempdir().unwrap();
    let old = paths::old_config_dir_in(home.path());
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("config.toml"), "[colors]\n").unwrap();

    let copied = paths::migrate_old_config(home.path()).unwrap();

    assert!(copied);
    assert!(!paths::config_dir_in(home.path()).join(".env").exists());
}

/// The new `.env` keeps the mode of the old `.env`. A user can set that mode
/// to keep other users of the computer from reading the secret key.
#[cfg(unix)]
#[test]
fn the_copy_keeps_the_permissions_of_the_env_file() {
    use std::os::unix::fs::PermissionsExt;

    let home = tempfile::tempdir().unwrap();
    old_installation(home.path());
    let old_env = paths::old_config_dir_in(home.path()).join(".env");
    fs::set_permissions(&old_env, fs::Permissions::from_mode(0o600)).unwrap();

    paths::migrate_old_config(home.path()).unwrap();

    let new_env = paths::config_dir_in(home.path()).join(".env");
    let mode = fs::metadata(&new_env).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

/// The copy changes the name of the key on its own line only. A comment, a
/// second variable, and a longer name that holds the old name inside it
/// must not change.
#[test]
fn the_copy_changes_only_the_line_of_the_key() {
    let home = tempfile::tempdir().unwrap();
    let old = paths::old_config_dir_in(home.path());
    fs::create_dir_all(&old).unwrap();
    fs::write(
        old.join(".env"),
        "# a comment about TOUTUI_SECRET_KEY\nTOUTUI_SECRET_KEY=abc123\nSECOND_VAR=xyz\nMY_TOUTUI_SECRET_KEY=decoy\n",
    )
    .unwrap();

    paths::migrate_old_config(home.path()).unwrap();

    let env = fs::read_to_string(paths::config_dir_in(home.path()).join(".env")).unwrap();
    assert_eq!(
        env,
        "# a comment about TOUTUI_SECRET_KEY\nABSTUI_SECRET_KEY=abc123\nSECOND_VAR=xyz\nMY_TOUTUI_SECRET_KEY=decoy\n"
    );
}
