//! The exit of the application, and the state of the terminal.
//!
//! The application takes the terminal: it puts the terminal in the raw mode,
//! it changes to the alternate screen, and it hides the cursor. Every exit
//! must give those three things back. An exit that does not give them back
//! leaves a terminal that shows no character that the user types, that holds
//! the screen of the application, and that has no cursor.

use crossterm::cursor::Show;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use std::io::{self, Write};
use std::process;

/// Gives the terminal back to the shell, on the given writer.
///
/// The writer is an argument, so that a test can read the bytes that this
/// function writes. `restore_terminal` gives it the standard output.
pub fn restore_terminal_on(writer: &mut impl Write) {
    let _ = disable_raw_mode();
    let _ = crossterm::execute!(writer, Show, LeaveAlternateScreen);
    let _ = writer.flush();
}

/// Gives the terminal back to the shell.
pub fn restore_terminal() {
    let mut stdout = io::stdout();
    restore_terminal_on(&mut stdout);
}

/// Gives the terminal back to the shell when the program panics.
///
/// A panic inside the application left the terminal in the raw mode and on
/// the alternate screen. The shell then showed no character that the user
/// typed, and it had no cursor. The message of the panic also went on to the
/// alternate screen, therefore the user did not read it. See `06e548` and
/// `40f48d` in `known_bugs.md`.
///
/// The hook gives the terminal back first, and it then calls the hook that
/// was present. Therefore the message of the panic arrives on the screen of
/// the shell, where the user can read it and give it to a report.
///
/// `restore` is an argument, so that a test can give a function of its own and
/// confirm that the hook calls it.
pub fn install_panic_hook_with(restore: fn()) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        previous(info);
    }));
}

/// Gives the terminal back to the shell when the program panics.
pub fn install_panic_hook() {
    install_panic_hook_with(restore_terminal);
}

/// Gives the terminal back and stops the program.
pub fn clean_exit() {
    restore_terminal();
    process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// The function writes the sequence that shows the cursor and the sequence
    /// that leaves the alternate screen.
    ///
    /// A terminal reads `ESC [ ? 25 h` as "show the cursor", and
    /// `ESC [ ? 1049 l` as "leave the alternate screen".
    #[test]
    fn the_program_writes_the_sequences_that_give_the_terminal_back() {
        let mut bytes: Vec<u8> = Vec::new();

        restore_terminal_on(&mut bytes);

        let written = String::from_utf8_lossy(&bytes);
        assert!(
            written.contains("\x1b[?25h"),
            "no show of the cursor: {:?}",
            written
        );
        assert!(
            written.contains("\x1b[?1049l"),
            "no exit of the alternate screen: {:?}",
            written
        );
    }

    static RESTORED: AtomicUsize = AtomicUsize::new(0);

    fn count_one_restore() {
        RESTORED.fetch_add(1, Ordering::SeqCst);
    }

    /// A panic calls the function that gives the terminal back.
    ///
    /// The test gives the hook a function of its own, therefore it does not
    /// touch the terminal of the test. The hook writes the message of the panic
    /// as before, and that message belongs to this test.
    #[test]
    fn a_panic_gives_the_terminal_back() {
        install_panic_hook_with(count_one_restore);

        let before = RESTORED.load(Ordering::SeqCst);
        let result = std::panic::catch_unwind(|| panic!("a panic of the test"));

        assert!(result.is_err(), "the panic must arrive");
        assert_eq!(
            RESTORED.load(Ordering::SeqCst),
            before + 1,
            "the hook did not give the terminal back"
        );
    }
}
