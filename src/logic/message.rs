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
//!
//! **A message belongs to the view of the user, and some messages belong to a
//! view of their own.** Three rules of the loop of `src/main.rs` write a
//! message with no key of the user — the shelf Continue Listening of the Home
//! view (T-160), the line of the view of the queue (T-161), and the media of
//! the view of the chapters (T-162) — and each of them wrote that text to the
//! one slot. The last writer won, whatever view the user was looking at: a
//! measurement of 2026-08-14 held a user in the view of the queue whose line
//! went to nobody, and the six seconds of their screen said "The media \"A Long
//! Test Book\" is not on the shelf Continue Listening now" — the sentence of a
//! view that they were not in. **The sentence of their own view never reached
//! the screen.** See T-164.
//!
//! A message of a view therefore waits for that view, and **its life starts
//! when the user first reads it**: the user of the queue reads the sentence of
//! the queue, and the sentence of the Home view stands on the screen when they
//! come back to that view. A message of no view — the answer of a key, and the
//! answer of a task — stands above them all: the user pressed that key, and the
//! answer of a key must come at once.

use crate::app::AppView;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// The time that a message stays on the screen.
///
/// A user reads a short sentence in about two seconds. This value gives room for
/// a user who looks away, and it stays short enough that an old message does not
/// answer for a new key.
pub const LIFE: Duration = Duration::from_secs(6);

/// One message, and the time when the user first read it.
///
/// `written` is `None` while a message of a view waits for that view: the life
/// of a message is the time of a person who reads it, therefore it starts at
/// the frame that shows it. See T-164.
#[derive(Debug, Clone)]
struct Message {
    text: String,
    written: Option<Instant>,
}

/// Every message that stands now.
#[derive(Debug, Default)]
struct TheMessages {
    /// The message that belongs to no view. A key of the user and a task of the
    /// program write this one, and every view shows it.
    of_no_view: Option<Message>,
    /// The messages that belong to a view. Each view holds one, and the user
    /// reads it when they look at that view. See T-164.
    of_the_views: Vec<(AppView, Message)>,
}

fn box_of_the_message() -> &'static Mutex<TheMessages> {
    static MESSAGE: OnceLock<Mutex<TheMessages>> = OnceLock::new();
    MESSAGE.get_or_init(|| Mutex::new(TheMessages::default()))
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
        place.of_no_view = Some(Message {
            text: text.to_string(),
            written: Some(Instant::now()),
        });
    }
}

/// Writes a message that belongs to one view.
///
/// **The user must read this message in that view, and in no other view**: a
/// rule of the loop of the program writes it with no key of the user, and the
/// user can stand anywhere. The message waits for that view, and its life
/// starts at the frame that shows it. A second message of the same view takes
/// the place of the first one. See T-164.
pub fn say_in(view: AppView, text: &str) {
    let text = text.trim();

    if text.is_empty() {
        return;
    }

    let Ok(mut place) = box_of_the_message().lock() else {
        return;
    };

    let message = Message {
        text: text.to_string(),
        written: None,
    };

    match place.of_the_views.iter_mut().find(|(one, _)| *one == view) {
        Some((_, older)) => *older = message,
        None => place.of_the_views.push((view, message)),
    }
}

/// Gives the message that the screen must draw, if one is fresh.
///
/// The render calls this at each frame, and it names the view of the user. The
/// message of no view comes first: the user pressed a key, and the answer of a
/// key must come at once. The message of the view of the user comes after it,
/// and its life starts here. A message that is older than [`LIFE`] gives
/// nothing, and the next frame then draws the view alone. See T-164.
pub fn for_the_screen(view: AppView) -> Option<String> {
    let mut place = box_of_the_message().lock().ok()?;

    if let Some(message) = place.of_no_view.clone() {
        let age = message.written.map(|when| when.elapsed());

        if age.is_none_or(|age| is_for_the_screen(age, LIFE)) {
            return Some(message.text);
        }

        place.of_no_view = None;
    }

    let at = place
        .of_the_views
        .iter()
        .position(|(one, _)| *one == view)?;

    // The life of a message is the time of a person who reads it, therefore it
    // starts at the frame that shows it. See T-164.
    let when = *place.of_the_views[at]
        .1
        .written
        .get_or_insert_with(Instant::now);

    if is_for_the_screen(when.elapsed(), LIFE) {
        return Some(place.of_the_views[at].1.text.clone());
    }

    place.of_the_views.remove(at);
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

/// Takes every message away at once.
///
/// A work that ended calls this, and the screen then shows the view alone. An
/// example is the message "The program gets the book…" of a book that came.
pub fn forget() {
    if let Ok(mut place) = box_of_the_message().lock() {
        place.of_no_view = None;
        place.of_the_views.clear();
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
        assert_eq!(for_the_screen(AppView::Home), None);

        say("The server does not answer.");
        assert_eq!(
            for_the_screen(AppView::Home).as_deref(),
            Some("The server does not answer.")
        );

        // The render reads the message at every frame, therefore a read takes
        // nothing away.
        assert!(for_the_screen(AppView::Home).is_some());

        // A message of no view stands in every view: the user pressed the key
        // that it answers.
        assert!(for_the_screen(AppView::Queue).is_some());

        // A new message takes the place of the older one. The user reads the
        // answer of their newest key.
        say("The mark of the media goes to the server…");
        assert_eq!(
            for_the_screen(AppView::Home).as_deref(),
            Some("The mark of the media goes to the server…")
        );

        // A message of no letters takes the message away. A work that gives an
        // empty text says nothing.
        say("   ");
        assert_eq!(for_the_screen(AppView::Home), None);

        say("A message");
        forget();
        assert_eq!(for_the_screen(AppView::Home), None);

        // The space at the two ends of a message says nothing, and it moves the
        // text that the screen puts in the middle.
        say("  The server does not answer.  ");
        assert_eq!(
            for_the_screen(AppView::Home).as_deref(),
            Some("The server does not answer.")
        );

        forget();

        // **A message of a view belongs to that view, and to no other view.** A
        // rule of the loop writes it with no key of the user, and the user can
        // stand anywhere. See T-164.
        say_in(AppView::Home, "The media is not on the shelf now.");

        // The user stands in the view of the queue, therefore they read
        // nothing of the Home view.
        assert_eq!(for_the_screen(AppView::Queue), None);
        assert_eq!(for_the_screen(AppView::Chapters), None);

        // The user comes to the Home view, and they read it there.
        assert_eq!(
            for_the_screen(AppView::Home).as_deref(),
            Some("The media is not on the shelf now.")
        );

        // **The two rules of one frame do not take each other away**: the fault
        // of T-164 is one slot for every view.
        forget();
        say_in(AppView::Queue, "The media is not in the queue now.");
        say_in(AppView::Home, "The media is not on the shelf now.");

        assert_eq!(
            for_the_screen(AppView::Queue).as_deref(),
            Some("The media is not in the queue now.")
        );
        assert_eq!(
            for_the_screen(AppView::Home).as_deref(),
            Some("The media is not on the shelf now.")
        );

        // A second message of one view takes the place of the first one.
        say_in(AppView::Queue, "A different media is not in the queue now.");
        assert_eq!(
            for_the_screen(AppView::Queue).as_deref(),
            Some("A different media is not in the queue now.")
        );

        // The answer of a key stands above them all.
        say("The media goes out of the queue…");
        assert_eq!(
            for_the_screen(AppView::Queue).as_deref(),
            Some("The media goes out of the queue…")
        );

        // A message of no letters says nothing at all.
        say_in(AppView::Stats, "   ");
        forget();
        assert_eq!(for_the_screen(AppView::Stats), None);
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
