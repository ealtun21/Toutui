//! What the program does while it starts. See T-40.
//!
//! `App::new` asks the server many times before the first screen: the
//! libraries, the list "Continue Listening", the position of each book of that
//! list, the series, the items, and the lists. A slow server therefore gave a
//! black screen for a long time, and a server that answers nothing gave a
//! black screen for the whole time of the timeout.
//!
//! The program draws a screen at once now, and that screen names the step. The
//! step lives here, because `App::new` writes it and the loop of the loading
//! screen reads it.

use std::sync::{Mutex, OnceLock};

fn box_of_the_step() -> &'static Mutex<String> {
    static STEP: OnceLock<Mutex<String>> = OnceLock::new();
    STEP.get_or_init(|| Mutex::new(String::from("the program starts")))
}

/// Names the step that runs now. The loading screen shows this text.
pub fn set(step: impl Into<String>) {
    if let Ok(mut value) = box_of_the_step().lock() {
        *value = step.into();
    }
}

/// Gives the name of the step that runs now.
pub fn step() -> String {
    box_of_the_step()
        .lock()
        .map(|value| value.clone())
        .unwrap_or_else(|_| String::from("the program starts"))
}

/// Names a step that has a number of parts, for example the position of each
/// book of the list "Continue Listening".
pub fn set_part(step: &str, done: usize, total: usize) {
    if total <= 1 {
        set(step);
    } else {
        set(format!("{} ({} of {})", step, done, total));
    }
}

/// Clears the screen of the shell before the first frame, and it gives those
/// bytes to the terminal at once. See T-265.
///
/// **A write of `stdout` that holds no line waits in the buffer.** The clear
/// held no line, and the program flushed it with the first write of the screen
/// of ratatui. A program that stops between the clear and that screen therefore
/// wrote its words to `stderr` first, and the buffer of `stdout` then gave the
/// clear at the exit: the terminal of the user kept **no word at all**. A
/// configuration file that the program cannot read is that condition, and the
/// measurement gave an empty screen and the status 1.
pub fn clear_the_screen_of_the_shell<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    write!(out, "\x1B[2J\x1B[1;1H")?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A writer that counts the bytes and the flushes of the caller.
    #[derive(Default)]
    struct AWriterThatCounts {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl std::io::Write for AWriterThatCounts {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    /// The clear of the start reaches the terminal at the moment of the write.
    /// A clear that waits in the buffer takes the words of a program that stops
    /// away with it. See T-265.
    #[test]
    fn the_clear_of_the_start_goes_to_the_terminal_at_once() {
        let mut out = AWriterThatCounts::default();

        clear_the_screen_of_the_shell(&mut out).expect("the writer takes every byte");

        assert_eq!(
            String::from_utf8_lossy(&out.bytes),
            "\x1B[2J\x1B[1;1H",
            "the bytes of the clear must not change"
        );
        assert_eq!(
            out.flushes, 1,
            "the clear must reach the terminal before the program can stop"
        );
    }

    #[test]
    fn the_step_of_many_parts_gives_the_count() {
        set_part("the position of each book", 3, 12);
        assert_eq!(step(), "the position of each book (3 of 12)");
    }

    #[test]
    fn one_part_gives_no_count() {
        set_part("the libraries", 1, 1);
        assert_eq!(step(), "the libraries");
    }
}
