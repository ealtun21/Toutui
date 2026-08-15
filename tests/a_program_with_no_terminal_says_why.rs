//! A program that has no terminal says why. See T-273.
//!
//! **A terminal that went away is not a terminal that never came.** T-271 and
//! T-272 hold the first condition. This test holds the second one: a process of
//! this program with no controlling terminal at all, which is the condition of a
//! unit of systemd, of a task of cron, and of a program of `setsid`.
//!
//! `ratatui::init()` is `try_init().expect("failed to initialize terminal")`,
//! therefore the old program panicked, and the hook of the panic of T-197 then
//! said that a part of the program had an internal fault. The machine gave the
//! program no terminal: that is no fault of Toutui, and a view never says a
//! reason that the program does not have (T-91).
//!
//! The first test starts the real binary with no controlling terminal. The
//! second one reads `src/main.rs`, because the second terminal of the program
//! stands after a login of a real server and no test reaches it.

use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use toutui::utils::the_terminal_of_the_program::{
    the_line_of_a_program_with_no_terminal, the_words_of_a_program_with_no_terminal,
};

/// The place of the sources of this project.
fn the_place_of_the_sources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `setsid` gives the child a session of its own, and a session of its own holds
/// no controlling terminal: the open of `/dev/tty` then gives `No such device or
/// address`. A machine with no `setsid` gives this test nothing at all.
fn the_machine_holds_setsid() -> bool {
    Command::new("setsid")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[test]
fn a_program_with_no_terminal_stops_with_words_of_its_own() {
    if !the_machine_holds_setsid() {
        eprintln!("this machine holds no setsid, therefore this test measured nothing");
        return;
    }

    // A directory of nothing gives the login screen (the trap 135), and it keeps
    // the database and the configuration file of the user away from this test.
    let place = std::env::temp_dir().join(format!(
        "toutui-a-program-with-no-terminal-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&place);
    std::fs::create_dir_all(place.join("config")).expect("the directory of the test");
    std::fs::create_dir_all(place.join("data")).expect("the directory of the test");

    let outcome = Command::new("setsid")
        // `setsid` makes a child of its own when the caller leads a process
        // group already. `--wait` gives the status of that child.
        .arg("--wait")
        .arg(env!("CARGO_BIN_EXE_toutui"))
        .env("XDG_CONFIG_HOME", place.join("config"))
        .env("XDG_DATA_HOME", place.join("data"))
        .env("TOUTUI_AUDIO_DEVICE", "null")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("the program of this test");

    let _ = std::fs::remove_dir_all(&place);

    // The words of the user stand on the standard error, and the hook of the
    // panic of T-197 writes to the two streams. This test reads both of them,
    // therefore no word of the old program hides.
    let words = format!(
        "{}{}",
        String::from_utf8_lossy(&outcome.stdout),
        String::from_utf8_lossy(&outcome.stderr)
    );

    assert!(
        words.contains("Toutui stops: it found no terminal."),
        "the program said no word of the terminal. It said: {}",
        words
    );
    assert!(
        words.contains("Start Toutui in a terminal."),
        "the words of the program name no road back. They are: {}",
        words
    );

    // **The words of the old program named a reason that the program does not
    // have.** The hook of the panic of T-197 gave `a part of the program had an
    // internal fault`, and it named a line of the source of a crate (T-172).
    assert!(
        !words.contains("panicked"),
        "the program panicked. It said: {}",
        words
    );
    assert!(
        !words.contains("internal fault"),
        "the program said that it had an internal fault. It said: {}",
        words
    );

    assert_eq!(
        outcome.status.code(),
        Some(1),
        "a program that stops with words of its own gives the status 1"
    );
}

#[test]
fn the_two_terminals_of_the_program_hold_the_words_of_a_machine_with_no_terminal() {
    let main = std::fs::read_to_string(the_place_of_the_sources().join("src/main.rs"))
        .expect("src/main.rs");

    assert!(
        // The name stands in the comments of that file too, therefore the test
        // reads the call and not the name.
        !main.contains("= ratatui::init()"),
        "the call of `ratatui::init` panics for a machine that gives no terminal"
    );

    // The terminal of the login screen, and the terminal of the application of
    // the user.
    assert_eq!(
        main.matches("the_terminal_of_the_program::the_terminal_of_the_program()")
            .count(),
        2,
        "the program makes two terminals, and each of them holds the words of T-273"
    );
}

#[test]
fn the_words_and_the_line_of_the_log_name_the_terminal_and_the_reason() {
    let reason = io::Error::from_raw_os_error(6);

    let words = the_words_of_a_program_with_no_terminal(&reason);
    assert!(words.contains("it found no terminal"));
    assert!(words.contains("No such device or address"));
    assert!(words.contains("Start Toutui in a terminal."));
    // A user must read no line of the source of this program (T-172).
    assert!(!words.contains(".rs"));

    let line = the_line_of_a_program_with_no_terminal(&reason);
    assert!(line.contains("[the terminal]"));
    assert!(line.contains("No such device or address"));
}
