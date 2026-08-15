//! A login screen that did not reach the terminal says why. See T-275.
//!
//! **A terminal that never came is not a screen that did not reach it.** T-273
//! holds a program with no terminal at all, and T-271 and T-272 hold a terminal
//! that went away. This item holds the third condition: the program found its
//! terminal, it drew the login screen, and the standard output of it then
//! failed.
//!
//! `let _ = self.auth()` of the render of the login screen dropped the fault of
//! that screen, and `let _app_result = app_login.run(terminal)` of `src/main.rs`
//! dropped the fault of the frame. The loop of the login then waited one second
//! and it made a terminal again: the words of the user came of T-273, and they
//! said `Toutui stops: it found no terminal.` while the user stood in a terminal
//! already (T-91). The log held no word of the login screen at all.
//!
//! The measurement of 2026-08-16, of the real program v0.8.103 inside tmux, with
//! the standard output of the program in a pipe whose reader went away after
//! three seconds. The corrected program said
//!
//! ```text
//! Toutui stops: the login screen did not reach the terminal.
//! Broken pipe (os error 32)
//! ```
//!
//! and it stopped with the status 1.

use std::io;
use std::path::PathBuf;

use toutui::login_app::the_end_of_a_frame_of_the_login;
use toutui::utils::the_terminal_of_the_program::{
    the_line_of_a_screen_that_did_not_reach_the_terminal,
    the_words_of_a_screen_that_did_not_reach_the_terminal,
};

/// The place of the sources of this project.
fn the_place_of_the_sources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The broken pipe of the measurement.
fn the_reason_of_the_machine() -> io::Error {
    io::Error::from_raw_os_error(32)
}

#[test]
fn the_words_name_the_screen_the_reason_and_the_road_back() {
    let words = the_words_of_a_screen_that_did_not_reach_the_terminal(&the_reason_of_the_machine());

    assert!(
        words.contains("the login screen did not reach the terminal"),
        "the words name no screen. They are: {}",
        words
    );
    assert!(
        words.contains("Broken pipe"),
        "the words hold no reason of the machine. They are: {}",
        words
    );
    assert!(
        words.contains("standard output"),
        "the words name no stream of the fault. They are: {}",
        words
    );
    assert!(
        words.contains("Start Toutui in a terminal."),
        "the words name no road back. They are: {}",
        words
    );

    // **The words of T-273 say a reason that this program does not have.** The
    // machine gave this program a terminal, and the user stands in one.
    assert!(
        !words.contains("it found no terminal"),
        "the words say that the program found no terminal. They are: {}",
        words
    );

    // A user must read no line of the source of this program or of a crate
    // (T-172).
    assert!(
        !words.contains(".rs"),
        "the words name a file of the source"
    );
}

#[test]
fn the_line_of_the_log_names_the_screen_and_the_reason() {
    let line = the_line_of_a_screen_that_did_not_reach_the_terminal(&the_reason_of_the_machine());

    assert!(line.contains("[the terminal]"));
    assert!(line.contains("the login screen did not reach the terminal"));
    assert!(line.contains("Broken pipe"));
}

/// **The login screen draws inside the frame of the loop of `run`**, and it
/// holds a terminal of its own. A standard output that failed gives the two
/// faults together, and the user must read the one of the screen that they
/// looked at.
#[test]
fn the_fault_of_the_screen_comes_before_the_fault_of_the_frame() {
    let of_the_screen = the_end_of_a_frame_of_the_login(
        Some(io::Error::other("the screen of the login")),
        Err(io::Error::other("the frame of the loop")),
    );

    assert_eq!(
        of_the_screen.unwrap_err().to_string(),
        "the screen of the login"
    );

    // A screen that gave no fault leaves the fault of the frame, because that
    // frame stands on the same standard output.
    let of_the_frame =
        the_end_of_a_frame_of_the_login(None, Err(io::Error::other("the frame of the loop")));

    assert_eq!(
        of_the_frame.unwrap_err().to_string(),
        "the frame of the loop"
    );

    // The login screen comes back for a login that succeeded, for a login that
    // failed, and for the key Esc. That road holds no fault at all.
    assert!(the_end_of_a_frame_of_the_login(None, Ok(())).is_ok());
}

/// The two lines that dropped the fault. A test of the real binary needs a
/// terminal for the program and a standard output that fails, and no machine of
/// a gate gives the two of them together: this test reads the two call sites
/// instead, as the second test of T-273 reads the second terminal of the
/// program.
#[test]
fn the_two_call_sites_of_the_login_screen_keep_the_fault() {
    let login_tui = std::fs::read_to_string(the_place_of_the_sources().join("src/ui/login_tui.rs"))
        .expect("src/ui/login_tui.rs");

    assert!(
        !login_tui.contains("let _ = self.auth();"),
        "the render of the login screen drops the fault of that screen"
    );

    let main = std::fs::read_to_string(the_place_of_the_sources().join("src/main.rs"))
        .expect("src/main.rs");

    assert!(
        !main.contains("let _app_result = app_login.run("),
        "the loop of the login drops the fault of the login screen"
    );
    assert!(
        main.contains("the_program_stops_for_a_screen_that_did_not_reach_the_terminal(&fault)"),
        "the loop of the login says nothing of a screen that did not reach the terminal"
    );
}

/// **`ratatui::restore()` writes words of its own to the standard error.** The
/// measurement of T-275 read `Failed to restore terminal: Broken pipe (os error
/// 32)` above the words of Toutui: a user must read no word of a crate (T-172),
/// and that sentence names no road back at all.
#[test]
fn the_program_gives_the_terminal_back_with_no_word_of_a_crate() {
    for file in [
        "src/main.rs",
        "src/utils/the_terminal_of_the_program.rs",
        "src/utils/exit_app.rs",
    ] {
        let source =
            std::fs::read_to_string(the_place_of_the_sources().join(file)).expect("the source");

        assert!(
            !source.contains("ratatui::restore();"),
            "{} calls the restore of the crate, and that call writes words of the crate",
            file
        );
    }
}
