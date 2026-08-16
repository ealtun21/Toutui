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

/// Gives the number of rows that a message of a width needs.
///
/// The count follows the rule of `Wrap { trim: true }` of ratatui: a break comes
/// at a space, and a word that is longer than the width takes rows of its own.
/// The function is pure, therefore a test needs no screen. See T-297 and T-299.
///
/// **The spaces between two words keep their width** (T-302). The count read
/// one space between two words, and every footer of this program holds **two**
/// of them: the count then said that the footer of the Home view needs three
/// rows of a terminal of 40 columns, and the render of ratatui took four. The
/// key `Q: quit` stood outside the screen at the end of the third row. A wrap
/// of `trim: true` takes the spaces of the start of a new row away, and it
/// keeps every space that stands inside a row.
pub fn the_rows_of_a_message(text: &str, width: u16) -> u16 {
    if text.trim().is_empty() || width == 0 {
        return 0;
    }

    let width = usize::from(width);
    let mut rows = 1u16;
    let mut column = 0usize;
    let mut spaces = 0usize;

    for part in text.split_inclusive(char::is_whitespace) {
        let word = part.trim_end();
        let after = part.chars().count() - word.chars().count();

        if word.is_empty() {
            spaces += after;
            continue;
        }

        let length = word.chars().count();

        if column == 0 {
            column = length;
        } else if column + spaces + length <= width {
            column += spaces + length;
        } else {
            rows = rows.saturating_add(1);
            column = length;
        }

        spaces = after;

        while column > width {
            rows = rows.saturating_add(1);
            column -= width;
        }
    }

    rows
}

/// Gives the first row and the number of rows of a message of a view.
///
/// The last row of the message stays above the footer, where one row of a
/// message stood, and the rows before it grow upward over the view (the trap
/// 39). The header of the screen keeps its rows, therefore a message that needs
/// more rows than that room takes the room and it loses its end.
///
/// A screen that holds no such row gives `None`, and the view then draws no
/// message at all. The function is pure, therefore a test needs no screen. See
/// T-299.
pub fn the_place_of_a_message(
    top: u16,
    height: u16,
    header: u16,
    footer: u16,
    rows_that_it_needs: u16,
) -> Option<(u16, u16)> {
    if height < footer + 1 {
        return None;
    }

    let last = top + height - footer - 1;
    let room = last.saturating_sub(top + header) + 1;
    let rows = rows_that_it_needs.clamp(1, room.max(1));

    Some((last.saturating_sub(rows.saturating_sub(1)), rows))
}

/// Makes the text of a message that must stand in a number of rows.
///
/// **A message that the screen cuts is a message that says nothing** (T-278,
/// T-297, and T-299). The row of the message of a view held one row and no
/// wrap, therefore the road back of a long sentence stood outside the screen.
/// The message now takes the rows that it needs, and this function is the last
/// limit of that growth: a message that needs more rows than the screen holds
/// loses its end, and the three points then say that the screen cut it. The
/// whole message stands in the log.
///
/// The function is pure, therefore a test needs no screen.
pub fn in_the_rows(text: &str, width: u16, rows: u16) -> String {
    if width == 0 || rows == 0 {
        return String::new();
    }

    if the_rows_of_a_message(text, width) <= rows {
        return text.to_string();
    }

    // The room of the rows is the first cut, and the wrap of a word then takes
    // one character at a time away.
    let room = usize::from(width) * usize::from(rows);
    let mut kept: Vec<char> = text.chars().take(room).collect();

    while !kept.is_empty() {
        let cut: String = kept.iter().collect();
        let cut = format!("{}…", cut.trim_end());

        if the_rows_of_a_message(&cut, width) <= rows {
            return cut;
        }

        kept.pop();
    }

    String::new()
}

/// The name of the variable of the environment that carries a sentence over a
/// start of the program again. See T-298.
///
/// **A user does not write this variable.** The program writes it for itself,
/// like [`crate::logic::auth::auth_input::THE_ADDRESS_OF_THE_LOGIN`].
///
/// **The box of the message belongs to no process after this one.** A log out of
/// the account that starts the program gives the start to another account, and
/// the program starts again with `exec`: every message of this process goes away
/// with it. The login screen takes the disk for that work (T-270), and the disk
/// says nothing to a program that holds an account, because that program draws
/// no login screen. Therefore the words of such a start ride on the environment
/// of the new process.
pub const THE_WORDS_OF_THE_START: &str = "TOUTUI_THE_WORDS_OF_THE_START";

/// Gives the sentence that the program before this one left, if it left one.
///
/// The function is pure, therefore a test needs no variable of the environment.
/// A value of no letter is no sentence: every start of the program again writes
/// this variable, and a start that carries no words writes it empty. See T-298.
pub fn the_words_of_the_start(of_the_environment: Option<&str>) -> Option<&str> {
    let words = of_the_environment?.trim();

    if words.is_empty() {
        None
    } else {
        Some(words)
    }
}

/// Says the sentence that the program before this one left. See T-298.
///
/// The loop of `src/main.rs` calls this before the first frame: the box of the
/// message belongs to no `App`, therefore the sentence waits there for the frame
/// that draws it.
pub fn say_the_words_of_the_start() {
    let of_the_environment = std::env::var(THE_WORDS_OF_THE_START).ok();

    if let Some(words) = the_words_of_the_start(of_the_environment.as_deref()) {
        say(words);
    }
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

    /// A message that the rows of the screen hold stays whole, and a message
    /// that is longer than them loses its end. See T-299.
    #[test]
    fn a_long_message_stands_in_the_rows_that_it_has() {
        assert_eq!(in_the_rows("A short message", 40, 1), "A short message");
        // The message holds the width exactly.
        assert_eq!(in_the_rows("12345", 5, 1), "12345");

        // Two rows hold a message that one row cuts.
        let sentence = "The server does not answer, and the program waits.";
        assert_eq!(the_rows_of_a_message(sentence, 20), 3);
        assert_eq!(in_the_rows(sentence, 20, 3), sentence);

        let long = in_the_rows(sentence, 20, 1);
        assert!(
            long.chars().count() <= 20,
            "{} letters: {}",
            long.chars().count(),
            long
        );
        assert!(long.ends_with('…'), "{}", long);
        assert!(long.starts_with("The server"), "{}", long);

        // A screen of no width and a screen of no row draw nothing.
        assert_eq!(in_the_rows("A message", 0, 1), "");
        assert_eq!(in_the_rows("A message", 20, 0), "");
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
