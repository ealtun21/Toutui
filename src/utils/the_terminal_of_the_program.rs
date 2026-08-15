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
            ratatui::restore();

            eprintln!("{}", the_words_of_a_program_with_no_terminal(&reason));

            std::process::exit(1);
        }
    }
}
