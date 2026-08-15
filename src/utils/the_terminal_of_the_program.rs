//! The program has no terminal, and it says why. See T-273.
//!
//! **A terminal that went away is not a terminal that never came.** T-271 and
//! T-272 hold the first condition: the program drew its screen, the terminal
//! then went away, and a watch of one second stops the program. This module
//! holds the second one: a process of this program that has no terminal at all.
//!
//! A unit of systemd, a task of cron, a program of `setsid`, and every program
//! whose shell gave it no terminal stand there. `crossterm` reads the keys of
//! the user from the standard input when that input is a terminal, and from
//! `/dev/tty` when it is not. A process with no controlling terminal has no
//! `/dev/tty`, therefore that open gives `No such device or address`.
//!
//! `ratatui::init()` is `try_init().expect("failed to initialize terminal")`.
//! The measurement of 2026-08-16, of the real program v0.8.101, with
//! `setsid env … ./target/debug/toutui < /dev/null`:
//!
//! ```text
//! thread 'main' (2322268) panicked at …/ratatui-0.30.2/src/init.rs:366:16:
//! failed to initialize terminal: Os { code: 6, kind: Uncategorized, message: "No such device or address" }
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! Toutui stopped: a part of the program had an internal fault. The lines above
//! name that fault, and the file of the log holds them too. Start Toutui again
//! to listen again.
//! ```
//!
//! **Those words name a reason that the program does not have** (T-91). The
//! fault is no internal fault of Toutui: the machine gave the program no
//! terminal, and the user must start it in one. The words also name a line of
//! the source of a crate, and a user must read no such line (T-172).
//!
//! **A third condition stands beside those two** (T-275): the program found its
//! terminal, and the standard output of it then failed. `let _ = self.auth()` of
//! the render of the login screen and `let _app_result = app_login.run(terminal)`
//! of `src/main.rs` dropped that fault, and the loop of the login then made a
//! terminal again: the words of the user came of the condition above, and they
//! said that the program found no terminal. The measurement of 2026-08-16, of
//! the real program v0.8.102 inside tmux, with the standard output in a pipe
//! whose reader went away after three seconds, gave `Toutui stops: it found no
//! terminal.` with the reason `Broken pipe (os error 32)` and the road back
//! `Start Toutui in a terminal.` The user stood in a terminal already, and the
//! log held no word of the login screen at all.

use std::io;

/// The words for the user of a program that has no terminal.
///
/// The words name the terminal, the reason of the machine, the work that needs
/// a terminal, and the road back. They name no file and no line of the source
/// (T-172), and they promise no function that the program does not have
/// (T-118).
pub fn the_words_of_a_program_with_no_terminal(reason: &io::Error) -> String {
    format!(
        "Toutui stops: it found no terminal.\n\
         {}\n\
         Toutui draws its screen in a terminal, and it reads the keys of the user from that \
         terminal.\n\
         Start Toutui in a terminal. A unit of systemd, a task of cron, and a program of the \
         background give no terminal.",
        reason
    )
}

/// The line of the log of a program that has no terminal.
///
/// **The user reads the words above on the screen of their shell**, because
/// that shell is the one place that this program has. The log keeps the same
/// fact for the maintainer.
pub fn the_line_of_a_program_with_no_terminal(reason: &io::Error) -> String {
    format!(
        "[the terminal] this program has no terminal, therefore it draws no screen and it reads \
         no key. Toutui stops. The machine said: {}",
        reason
    )
}

/// The words for the user of a login screen that did not reach the terminal.
/// See T-275.
///
/// **A terminal that never came is not a screen that did not reach it.** The
/// two functions above hold a program with no terminal at all. This one holds a
/// program that found its terminal, that drew the login screen, and whose
/// standard output then failed.
///
/// The words name the screen, the reason of the machine, the work of that
/// stream, and the road back. They name no file and no line of the source
/// (T-172).
pub fn the_words_of_a_screen_that_did_not_reach_the_terminal(reason: &io::Error) -> String {
    format!(
        "Toutui stops: the login screen did not reach the terminal.\n\
         {}\n\
         Toutui writes the login screen to its standard output, and it reads the keys of the \
         user from the terminal.\n\
         Start Toutui in a terminal. Do not send the standard output of Toutui to a file or to \
         a different program.",
        reason
    )
}

/// The line of the log of a login screen that did not reach the terminal. See
/// T-275.
pub fn the_line_of_a_screen_that_did_not_reach_the_terminal(reason: &io::Error) -> String {
    format!(
        "[the terminal] the login screen did not reach the terminal, therefore the user reads no \
         field of it and no message of it. Toutui stops. The machine said: {}",
        reason
    )
}

/// Gives the terminal of the user back. See T-275.
///
/// **`ratatui::restore()` writes words of its own to the standard error**, and
/// those words are `Failed to restore terminal: …`. The measurement of T-275
/// read them above the words of Toutui: a user must read no word of a crate
/// (T-172), and that sentence names no road back at all.
///
/// The raw mode and the alternate screen stand on the standard output, therefore
/// a standard output that failed keeps the shell of the user in them. The
/// program says nothing of that here: it has one shell, and the words of the
/// fault of the user belong to it.
pub fn the_program_gives_the_terminal_back() {
    if let Err(reason) = ratatui::try_restore() {
        log::error!(
            "[the terminal] the program did not give the terminal back: {}. \
             The shell of the user can stay in the raw mode.",
            reason
        );
    }
}

/// Stops the program for a login screen that did not reach the terminal. See
/// T-275.
///
/// The function never comes back.
pub fn the_program_stops_for_a_screen_that_did_not_reach_the_terminal(reason: &io::Error) -> ! {
    log::error!(
        "{}",
        the_line_of_a_screen_that_did_not_reach_the_terminal(reason)
    );

    // The program stands on the alternate screen, and the words of the user
    // must stand on the screen of their shell.
    the_program_gives_the_terminal_back();

    eprintln!(
        "{}",
        the_words_of_a_screen_that_did_not_reach_the_terminal(reason)
    );

    std::process::exit(1);
}

/// The terminal of this program, or the words of a program that has none.
///
/// `ratatui::init()` panics for a machine that gives no terminal, and the hook
/// of the panic of T-197 then says that a part of the program had an internal
/// fault. This function takes the fault of `ratatui::try_init()` instead, and
/// it stops the program with words of its own.
///
/// The function never comes back for a program that has no terminal.
pub fn the_terminal_of_the_program() -> ratatui::DefaultTerminal {
    match ratatui::try_init() {
        Ok(terminal) => terminal,
        Err(reason) => {
            log::error!("{}", the_line_of_a_program_with_no_terminal(&reason));

            // `try_init` enables the raw mode and it enters the alternate
            // screen before it makes the terminal. A fault of the last of those
            // three steps therefore leaves the screen of the user in the
            // alternate screen, and the words of the user must stand on the
            // screen of their shell.
            the_program_gives_the_terminal_back();

            eprintln!("{}", the_words_of_a_program_with_no_terminal(&reason));

            std::process::exit(1);
        }
    }
}
