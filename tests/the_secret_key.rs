//! The secret key that the program makes for itself. See T-133.
//!
//! **A program that has no key keeps no token.** `install.sh` writes the key,
//! and a user who builds the program with `cargo` gets no such file: the login
//! then took the password of the user, it asked the server, and it wrote
//! nothing at all. The program makes the key itself now.
//!
//! The test stands in its own file, because it writes the variable of the
//! environment of the whole process. The tests of one file run on threads of one
//! process, therefore this file holds one test.

use std::path::Path;
use toutui::utils::encrypt_token::{
    encrypt_token, the_program_makes_a_key_if_it_has_none, THE_KEY,
};

/// The whole life of the key: the program makes it, it writes it, and it reads
/// it again.
#[test]
fn the_program_makes_the_key_that_it_has_not() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("toutui");

    // The process of the test holds no key.
    std::env::remove_var(THE_KEY);
    assert!(encrypt_token("a-token").is_err());

    // The first start makes the key.
    assert_eq!(the_program_makes_a_key_if_it_has_none(&config), Ok(true));

    let path = config.join(".env");
    let text = std::fs::read_to_string(&path).unwrap();
    let line = text
        .lines()
        .find(|line| line.starts_with(THE_KEY))
        .expect("the file holds the key");
    let key = line.trim_start_matches(THE_KEY).trim_start_matches('=');

    // 32 bytes of the machine, in the form of the hexadecimal.
    assert_eq!(key.len(), 64);
    assert!(key.chars().all(|of_the_key| of_the_key.is_ascii_hexdigit()));

    // Nobody else reads the key.
    the_file_belongs_to_the_user_alone(&path);

    // The program of this moment holds the key, therefore the login of this
    // start keeps its token.
    assert_eq!(std::env::var(THE_KEY).unwrap(), key);
    assert!(encrypt_token("a-token").is_ok());

    // The start after it makes no second key.
    assert_eq!(the_program_makes_a_key_if_it_has_none(&config), Ok(false));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), text);

    // **A start reads the key of the file.** The program of a new start holds
    // no variable, and the token of the database needs the key that wrote it.
    std::env::remove_var(THE_KEY);
    assert_eq!(the_program_makes_a_key_if_it_has_none(&config), Ok(false));
    assert_eq!(std::env::var(THE_KEY).unwrap(), key);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), text);

    // A file that holds other lines keeps them, and the key stands on its own
    // line.
    std::env::remove_var(THE_KEY);
    let other = config.join("other");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join(".env"), "TOUTUI_OTHER=1").unwrap();

    assert_eq!(the_program_makes_a_key_if_it_has_none(&other), Ok(true));

    let text = std::fs::read_to_string(other.join(".env")).unwrap();
    assert!(text.starts_with("TOUTUI_OTHER=1\n"));
    assert!(text.contains(THE_KEY));
    assert!(encrypt_token("a-token").is_ok());
}

#[cfg(unix)]
fn the_file_belongs_to_the_user_alone(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mode = std::fs::metadata(path).unwrap().permissions().mode();
    assert_eq!(mode & 0o077, 0, "the mode of the file is {:o}", mode);
}

#[cfg(not(unix))]
fn the_file_belongs_to_the_user_alone(_path: &Path) {}
