//! A screen that asks the user for a text takes no lock of the standard
//! output. See T-174.
//!
//! **A measurement of 2026-08-14 with the sandbox, tmux, and
//! `docs/harness/no_library.py`.** The thread of the login of T-173 panicked,
//! and the program then stood for ever with a screen of no character: no word
//! came to the terminal, no word came to the standard error, and the log
//! stopped at the line before the panic. `strace` says the last thing that the
//! thread of the login did: it gave the raw mode of the terminal back, and it
//! then waited for a mutex of the program.
//!
//! **The cause is one lock.** `AppLogin::auth` made the backend of its screen
//! of `io::stdout().lock()`, and it held that lock while it waited for the
//! thread of the login (T-133). `ratatui::init` of `main` puts a hook on a
//! panic, that hook gives the terminal back on `io::stdout()`, and the lock of
//! the standard output of Rust makes a second thread wait. The screen of the
//! search and the screen of a prompt of a text held the same lock.
//!
//! The three screens take the backend of `the_backend_of_a_field` now, and that
//! backend holds no lock. **Without the correction this test never comes back**,
//! therefore the write of the second thread runs on a thread of its own and
//! this test reads the end of that thread with a limit of time.

use std::io::Write;
use std::sync::mpsc;
use std::time::Duration;
use toutui::ui::text_field::the_backend_of_a_field;

/// A second thread writes to the standard output while a screen of a field
/// stands.
///
/// **The parts of this test stay in one function**: the lock of the standard
/// output belongs to the process, and two test functions of one binary fight
/// for it (T-144 and T-157).
#[test]
fn a_second_thread_writes_while_a_screen_of_a_field_stands() {
    let the_screen = the_backend_of_a_field();

    let (say, hear) = mpsc::channel();

    std::thread::spawn(move || {
        // The hook of a panic writes to the standard output in this way.
        let mut the_standard_output = std::io::stdout();
        let _ = write!(the_standard_output, "");
        let _ = the_standard_output.flush();

        let _ = say.send(());
    });

    let the_end = hear.recv_timeout(Duration::from_secs(10));

    // The screen stands for the whole wait, as the screen of the login stands
    // while it waits for the thread of the login.
    let _ = &the_screen;

    assert!(
        the_end.is_ok(),
        "the screen of a field holds the lock of the standard output"
    );
}
