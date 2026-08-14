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

// A panic that the caller expects, and that the caller catches.
//
// The hook must not give the terminal back for such a panic, because the
// application continues. The hook must also write no message on the screen,
// because the screen holds the application.
//
// One place uses this: the Opus decoder. A crate that decodes audio can stop
// with an arithmetic fault on data that it does not expect. The engine catches
// that panic, it stops the one track, and the application continues. See T-17.
thread_local! {
    static EXPECTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Tells the hook that this thread expects a panic.
///
/// The old value comes back when the guard goes away. Therefore a function
/// inside another function does no damage.
pub struct ExpectedPanic(bool);

impl ExpectedPanic {
    /// Makes the guard.
    pub fn new() -> ExpectedPanic {
        ExpectedPanic(EXPECTED.replace(true))
    }
}

impl Default for ExpectedPanic {
    fn default() -> Self {
        ExpectedPanic::new()
    }
}

impl Drop for ExpectedPanic {
    fn drop(&mut self) {
        EXPECTED.set(self.0);
    }
}

/// Tells if the thread expects a panic.
pub fn panic_is_expected() -> bool {
    EXPECTED.with(|value| value.get())
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
/// `restore` and `stop` are arguments, so that a test can give functions of its
/// own and confirm that the hook calls them.
///
/// A panic that the caller expects is different. The hook then writes a line in
/// the log only: it keeps the terminal, and it keeps the screen, because the
/// application continues. See `ExpectedPanic`.
///
/// **A panic of a thread that is not the main thread stops the program too**
/// (T-197). The hook gave the terminal back and it then came back to a program
/// that lives: the render of the application kept its work, it wrote to a
/// terminal of the shell with no raw mode, and the message of the panic went
/// under the characters of a screen that no terminal reads. A measurement of
/// 2026-08-14 with a panic in the loop of the playback: the words of the panic
/// stood under `▶ 11 20 … Left: 7:48:40`, **the key `Q` did nothing** because
/// the terminal takes no key of a raw mode that no program holds, the audio
/// played on, and the place of the user stayed at 0 on the disk and on the
/// server for the whole book. A thread of this program that dies takes a part
/// of the work of the program with it, therefore the program stops and the user
/// starts it again.
pub fn install_panic_hook_with(restore: fn(), stop: fn()) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // The caller catches this panic, and the application continues. The
        // terminal must stay as it is, and the screen must stay as it is.
        if panic_is_expected() {
            log::error!("[panic] the caller expects this panic: {}", info);
            return;
        }

        // **The log holds the fault too.** The terminal of the user goes away
        // with the next command of the shell, and the maintainer reads the log.
        // See T-197.
        log::error!("[panic] {}", info);

        restore();
        previous(info);

        let mut stderr = io::stderr();
        let _ = writeln!(stderr, "{}", THE_SENTENCE_OF_A_PANIC);
        let _ = stderr.flush();

        stop();
    }));
}

/// The sentence of a panic, for the user. See T-197.
///
/// The lines of the panic name a file of the source, and those words belong to
/// a report. This sentence says what happened to the program, and what the user
/// does now.
pub const THE_SENTENCE_OF_A_PANIC: &str = "Toutui stopped: a part of the program \
     had an internal fault. The lines above name that fault, and the file of the \
     log holds them too. Start Toutui again to listen again.";

/// Gives the terminal back to the shell when the program panics, and it stops
/// the program.
pub fn install_panic_hook() {
    install_panic_hook_with(restore_terminal, || process::exit(1));
}

/// Gives the terminal back and stops the program.
pub fn clean_exit() {
    restore_terminal();
    process::exit(0);
}

/// Starts this program again, in the place of this process. See T-123.
///
/// **A login that comes again needs a terminal that no view holds.** The program
/// takes the terminal one time at its start, and a second start of the terminal
/// inside the same process gave a screen of no character: the login screen drew
/// its box, and the box then went away. A measurement of 2026-08-12 in tmux
/// showed it. The program that starts again meets the login screen of a first
/// start, and that screen works.
///
/// The new process also takes the tasks away: the task of the live messages, the
/// task of the probe, and the task of the positions that wait hold the token
/// that the server refused, and every one of them stops with this process.
///
/// The function gives an error, and it gives nothing when it succeeds: the new
/// program holds this process then. A system that has no `exec` gives the error
/// of that system, and the caller then makes the login screen inside this
/// process.
///
/// The new program takes the address of the server in a variable of the
/// environment, because the value of a variable of this process goes away with
/// it: the login screen of the new program then writes the address of the user
/// in its first field.
pub fn start_the_program_again(the_address_of_the_login: &str) -> io::Error {
    start_the_program_again_with(&[(
        crate::logic::auth::auth_input::THE_ADDRESS_OF_THE_LOGIN,
        the_address_of_the_login,
    )])
}

/// Starts this program again, with the variables of the environment that the
/// caller gives. See T-123 and T-124.
///
/// A token that the server refused gives the address of the login (T-123). A
/// user who adds an account gives that address **and** the request of the login
/// screen, and a user who takes a different account gives nothing at all: the
/// database then holds the account of the start.
pub fn start_the_program_again_with(variables: &[(&str, &str)]) -> io::Error {
    restore_terminal();

    let program = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => return error,
    };

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let mut command = std::process::Command::new(program);
        command.args(std::env::args_os().skip(1));

        for (name, value) in variables {
            command.env(name, value);
        }

        // `exec` gives an answer only when it fails.
        command.exec()
    }

    #[cfg(not(unix))]
    {
        let _ = (program, variables);
        io::Error::new(
            io::ErrorKind::Unsupported,
            "This system cannot start the program in the place of this process.",
        )
    }
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

    static STOPPED: AtomicUsize = AtomicUsize::new(0);

    /// The `stop` of the program, for a test. The `stop` of the program itself
    /// never comes back, therefore a test gives its own.
    fn count_one_stop() {
        STOPPED.fetch_add(1, Ordering::SeqCst);
    }

    /// The hook gives the terminal back for a panic that no caller expects, and
    /// it keeps the terminal for a panic that a caller expects.
    ///
    /// The two rules are in one test, because `set_hook` changes a value of the
    /// whole program. Two tests that install a hook run at the same time, and
    /// each hook then calls the hook before it.
    ///
    /// The test gives the hook a function of its own, therefore it does not
    /// touch the terminal of the test.
    #[test]
    fn the_hook_gives_the_terminal_back_for_a_panic_that_no_caller_expects() {
        install_panic_hook_with(count_one_restore, count_one_stop);

        let before = RESTORED.load(Ordering::SeqCst);
        let stops = STOPPED.load(Ordering::SeqCst);

        // A panic that a caller expects keeps the terminal.
        let result = {
            let _guard = ExpectedPanic::new();
            std::panic::catch_unwind(|| panic!("a panic that the caller expects"))
        };

        assert!(result.is_err(), "the panic must arrive");
        assert_eq!(
            RESTORED.load(Ordering::SeqCst),
            before,
            "the hook gave the terminal back for a panic that the caller expects"
        );

        // **A panic that a caller expects stops no program.** The Opus decoder
        // catches it, and the application continues. See T-17 and T-197.
        assert_eq!(
            STOPPED.load(Ordering::SeqCst),
            stops,
            "the hook stopped the program for a panic that the caller expects"
        );

        // The guard is gone. A panic gives the terminal back again.
        let result = std::panic::catch_unwind(|| panic!("a panic of the test"));

        assert!(result.is_err(), "the panic must arrive");
        assert_eq!(
            RESTORED.load(Ordering::SeqCst),
            before + 1,
            "the hook did not give the terminal back"
        );

        // **A panic that no caller expects stops the program.** The hook gave
        // the terminal back and it then came back to a program that lives: the
        // render wrote to a terminal of the shell, the words of the panic went
        // under those characters, and the key `Q` did nothing. See T-197.
        assert_eq!(
            STOPPED.load(Ordering::SeqCst),
            stops + 1,
            "the hook did not stop the program"
        );
    }

    /// The sentence of a panic says what happened and what the user does now.
    #[test]
    fn the_sentence_of_a_panic_names_the_program_and_the_log() {
        assert!(
            THE_SENTENCE_OF_A_PANIC.contains("Toutui stopped")
                && THE_SENTENCE_OF_A_PANIC.contains("log"),
            "the sentence says that the program stopped, and where the fault \
             stands: {}",
            THE_SENTENCE_OF_A_PANIC
        );
    }
}
