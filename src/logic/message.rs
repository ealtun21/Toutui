//! The message for the user, between the work and the screen. See T-59.
//!
//! The program wrote a message with `pop_message`, and that function writes to
//! the terminal **outside the buffer of ratatui**. Two faults come of that:
//!
//! 1. **A message goes away before the user reads it.** ratatui writes the cells
//!    that changed at each frame. A view that draws the row of the message takes
//!    it away. The key `C` with no media met that fault: the user pressed a key
//!    and read nothing. See T-59.
//! 2. **A message stays after the work of it ended.** ratatui writes no cell that
//!    did not change, therefore the bytes of the message stay on the screen. The
//!    key `R` met that fault, and the loop of the program now clears the whole
//!    terminal after a refresh. See T-42.
//!
//! This module is the slot between the work and the screen, and it has the shape
//! of `crate::logic::live` and of `crate::logic::stats`: a task or a key writes
//! the message, and the render takes it at the next frame. **The message stands
//! inside the frame**, therefore no cell of it stays and no view takes it away.
//!
//! The screen of the login kept `pop_message` until T-134, and it draws its
//! message inside its own frame now: the cursor of the terminal stood at the end
//! of that message while the user wrote their password. **No function of the
//! program writes outside a frame**, therefore the module `pop_up_message` went
//! away.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// The time that a message stays on the screen.
///
/// A user reads a short sentence in about two seconds. This value gives room for
/// a user who looks away, and it stays short enough that an old message does not
/// answer for a new key.
pub const LIFE: Duration = Duration::from_secs(6);

/// One message, and the time when the program wrote it.
#[derive(Debug, Clone)]
struct Message {
    text: String,
    written: Instant,
}

fn box_of_the_message() -> &'static Mutex<Option<Message>> {
    static MESSAGE: OnceLock<Mutex<Option<Message>>> = OnceLock::new();
    MESSAGE.get_or_init(|| Mutex::new(None))
}

/// Tells if a message of an age is still for the screen.
///
/// The function is pure, therefore a test needs no clock of the machine.
pub fn is_for_the_screen(age: Duration, life: Duration) -> bool {
    age < life
}

/// Writes the message that the screen shows.
///
/// Every work of the program calls this: a key, and a task. The function needs no
/// `&mut App`, therefore a task that holds no application writes a message as
/// well.
pub fn say(text: &str) {
    let text = text.trim();

    if text.is_empty() {
        forget();
        return;
    }

    if let Ok(mut place) = box_of_the_message().lock() {
        *place = Some(Message {
            text: text.to_string(),
            written: Instant::now(),
        });
    }
}

/// Gives the message that the screen must draw, if one is fresh.
///
/// The render calls this at each frame. A message that is older than [`LIFE`]
/// gives nothing, and the next frame then draws the view alone.
pub fn for_the_screen() -> Option<String> {
    let mut place = box_of_the_message().lock().ok()?;
    let message = place.clone()?;

    if is_for_the_screen(message.written.elapsed(), LIFE) {
        return Some(message.text);
    }

    *place = None;
    None
}

/// Makes one line of a message for a screen of a width.
///
/// A message of more letters than the width would go to a second row, and the
/// row of the message holds one row. Therefore the function cuts a long message
/// and it names the cut with three points. The whole message stands in the log.
///
/// The function is pure, therefore a test needs no screen.
pub fn one_line(text: &str, width: u16) -> String {
    let width = usize::from(width);
    let letters = text.chars().count();

    if width == 0 {
        return String::new();
    }

    if letters <= width {
        return text.to_string();
    }

    // Two spaces stand beside the message, therefore the cut leaves room for
    // them and for the three points.
    let room = width.saturating_sub(4);

    if room == 0 {
        return text.chars().take(width).collect();
    }

    let kept: String = text.chars().take(room).collect();

    format!("{}…", kept.trim_end())
}

/// Takes the message away at once.
///
/// A work that ended calls this, and the screen then shows the view alone. An
/// example is the message "The program gets the book…" of a book that came.
pub fn forget() {
    if let Ok(mut place) = box_of_the_message().lock() {
        *place = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slot belongs to the process, therefore the parts of this test stay in
    /// one function. Two test functions would fight for it.
    #[test]
    fn the_message_goes_from_the_work_to_the_screen() {
        forget();
        assert_eq!(for_the_screen(), None);

        say("The server does not answer.");
        assert_eq!(
            for_the_screen().as_deref(),
            Some("The server does not answer.")
        );

        // The render reads the message at every frame, therefore a read takes
        // nothing away.
        assert!(for_the_screen().is_some());

        // A new message takes the place of the older one. The user reads the
        // answer of their newest key.
        say("The mark of the media goes to the server…");
        assert_eq!(
            for_the_screen().as_deref(),
            Some("The mark of the media goes to the server…")
        );

        // A message of no letters takes the message away. A work that gives an
        // empty text says nothing.
        say("   ");
        assert_eq!(for_the_screen(), None);

        say("A message");
        forget();
        assert_eq!(for_the_screen(), None);

        // The space at the two ends of a message says nothing, and it moves the
        // text that the screen puts in the middle.
        say("  The server does not answer.  ");
        assert_eq!(
            for_the_screen().as_deref(),
            Some("The server does not answer.")
        );

        forget();
    }

    /// The row of the message holds one row. A message of more letters than the
    /// width of the screen must not go to a second row.
    #[test]
    fn a_long_message_gives_one_line() {
        assert_eq!(one_line("A short message", 40), "A short message");
        // The message holds the width exactly.
        assert_eq!(one_line("12345", 5), "12345");

        let long = one_line("The server does not answer, and the program waits.", 20);
        assert!(
            long.chars().count() <= 20,
            "{} letters: {}",
            long.chars().count(),
            long
        );
        assert!(long.ends_with('…'), "{}", long);
        assert!(long.starts_with("The server"), "{}", long);

        // A screen of no width draws nothing.
        assert_eq!(one_line("A message", 0), "");
        // A screen of very little width gives the letters that it holds.
        assert_eq!(one_line("A message", 3).chars().count(), 3);
    }

    /// A message goes away after its time. The rule is pure, therefore this test
    /// needs no wait.
    #[test]
    fn a_message_goes_away_after_its_time() {
        assert!(is_for_the_screen(Duration::from_secs(0), LIFE));
        assert!(is_for_the_screen(Duration::from_secs(5), LIFE));
        assert!(!is_for_the_screen(LIFE, LIFE));
        assert!(!is_for_the_screen(Duration::from_secs(60), LIFE));

        // A life of no time gives no message at all.
        assert!(!is_for_the_screen(
            Duration::from_secs(0),
            Duration::from_secs(0)
        ));
    }
}
