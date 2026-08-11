//! The live messages, between the task and the screen. See T-47.
//!
//! The render is not asynchronous. Therefore the task of the live messages puts
//! every change here, and the render takes it at the next frame. This is the
//! shape of `src/logic/stats.rs` and of `src/logic/bookmarks.rs`.
//!
//! The box holds three things, and they answer three different needs:
//!
//! - **The position of a media.** A different client of the same account moved
//!   in a book. The mark of the line shows the new position at the next frame,
//!   and the program asks the server for nothing. See T-44.
//! - **The media away from Continue Listening.** A different client finished a
//!   media, or hid it. That media must leave the shelf of Continue Listening,
//!   and the program holds every line of that shelf already. Therefore the
//!   render makes the lines again, and it asks the server for nothing. See
//!   T-66.
//! - **The lists are old.** A different client changed the metadata of an item.
//!   That value stands in many lists, therefore the program cannot correct one
//!   line. The header asks the user for the key `R`.

use crate::api::live::Progress;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Mutex, OnceLock};

/// The state of the connection of the live messages.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum State {
    /// The task did not start.
    #[default]
    Nothing,
    /// The task asks the server for the handshake.
    Waiting,
    /// The connection is open, and the messages come.
    Ready,
    /// The connection ended, and this text says why. The task starts a new
    /// connection after a short time.
    Fault(String),
}

/// Everything that the live messages gave.
#[derive(Debug, Clone, Default)]
struct Live {
    state: State,
    /// The newest position of each media. The key is the identity of the item.
    progress: HashMap<String, Progress>,
    /// A different client changed a list of the screen.
    the_lists_are_old: bool,
    /// The media that must not stand on the shelf of Continue Listening. The
    /// user finished them, or the user hid them. See T-66.
    away_from_continue_listening: BTreeSet<String>,
}

fn box_of_the_live() -> &'static Mutex<Live> {
    static LIVE: OnceLock<Mutex<Live>> = OnceLock::new();
    LIVE.get_or_init(|| Mutex::new(Live::default()))
}

/// Writes the state of the connection. The task calls this.
pub fn keep(state: State) {
    if let Ok(mut place) = box_of_the_live().lock() {
        place.state = state;
    }
}

/// Gives the state of the connection.
///
/// A lock that fails gives `Nothing`. The screen then says that no message
/// came, and it does not stop.
pub fn state() -> State {
    match box_of_the_live().lock() {
        Ok(place) => place.state.clone(),
        Err(_) => State::Nothing,
    }
}

/// Writes the position of every media of one message.
///
/// A message carries the position of every media of the account, therefore this
/// function writes them all. A position of a message is always newer than a
/// position of a request, because the server sends the message after it wrote
/// the value.
pub fn note_the_progress(rows: Vec<(String, Progress)>) {
    if let Ok(mut place) = box_of_the_live().lock() {
        for (id, progress) in rows {
            place.progress.insert(id, progress);
        }
    }
}

/// Gives the newest position of one media, if a message gave one.
///
/// The render calls this for each line of the Home view. A media with no
/// message gives nothing, and the line then shows the value of the request of
/// the start.
pub fn progress_of(item_id: &str) -> Option<Progress> {
    match box_of_the_live().lock() {
        Ok(place) => place.progress.get(item_id).cloned(),
        Err(_) => None,
    }
}

/// Writes the media that must not stand on the shelf of Continue Listening.
///
/// This list **takes the place** of the list that came before it, because a
/// message carries the whole account. A media that a different client made
/// unfinished comes back on the shelf in that way. See T-66.
pub fn note_the_media_away_from_continue_listening(ids: Vec<String>) {
    if let Ok(mut place) = box_of_the_live().lock() {
        place.away_from_continue_listening = ids.into_iter().collect();
    }
}

/// Gives the media that must not stand on the shelf of Continue Listening.
///
/// The Home view reads this at the frame, and it asks the server for nothing.
/// A message that never came gives an empty list, and every line then stays.
pub fn the_media_away_from_continue_listening() -> BTreeSet<String> {
    match box_of_the_live().lock() {
        Ok(place) => place.away_from_continue_listening.clone(),
        Err(_) => BTreeSet::new(),
    }
}

/// Says that a different client changed a list of the screen.
pub fn note_that_the_lists_are_old() {
    if let Ok(mut place) = box_of_the_live().lock() {
        place.the_lists_are_old = true;
    }
}

/// Tells if a list of the screen is old. The header reads this.
pub fn the_lists_are_old() -> bool {
    match box_of_the_live().lock() {
        Ok(place) => place.the_lists_are_old,
        Err(_) => false,
    }
}

/// Forgets that the lists are old. The key `R` calls this, because that key
/// asks the server for every list again.
///
/// The list of the media away from Continue Listening goes away too: the shelf
/// of the new request holds none of them already, and a media that came back on
/// that shelf must not go away a second time. The next message gives the list
/// again. See T-66.
pub fn the_lists_are_new_again() {
    if let Ok(mut place) = box_of_the_live().lock() {
        place.the_lists_are_old = false;
        place.away_from_continue_listening.clear();
    }
}

/// Forgets everything. A test calls this, and the program calls it when the
/// user signs in with a different account.
pub fn forget() {
    if let Ok(mut place) = box_of_the_live().lock() {
        *place = Live::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_progress(percent: &str) -> Progress {
        Progress {
            percent: percent.to_string(),
            finished: "Not finished".to_string(),
        }
    }

    /// The box belongs to the process, therefore the parts of this test must
    /// stay in one function. Two test functions would fight for it.
    #[test]
    fn the_messages_go_from_the_task_to_the_screen() {
        forget();

        assert_eq!(state(), State::Nothing);
        assert!(!the_lists_are_old());
        assert!(progress_of("a book").is_none());

        keep(State::Waiting);
        assert_eq!(state(), State::Waiting);

        keep(State::Ready);
        assert_eq!(state(), State::Ready);

        // The position of two media comes in one message.
        note_the_progress(vec![
            ("a book".to_string(), a_progress("43")),
            ("a second book".to_string(), a_progress("7")),
        ]);

        assert_eq!(
            progress_of("a book").map(|row| row.percent),
            Some("43".to_string())
        );
        assert_eq!(
            progress_of("a second book").map(|row| row.percent),
            Some("7".to_string())
        );
        assert!(progress_of("a book that never played").is_none());

        // A newer message takes the place of the older one.
        note_the_progress(vec![("a book".to_string(), a_progress("44"))]);
        assert_eq!(
            progress_of("a book").map(|row| row.percent),
            Some("44".to_string())
        );
        // The message named one media only, therefore the other one stays.
        assert_eq!(
            progress_of("a second book").map(|row| row.percent),
            Some("7".to_string())
        );

        // The media away from Continue Listening. See T-66.
        assert!(the_media_away_from_continue_listening().is_empty());

        note_the_media_away_from_continue_listening(vec![
            "a book that ended".to_string(),
            "a book that the user hid".to_string(),
        ]);
        assert!(the_media_away_from_continue_listening().contains("a book that ended"));
        assert_eq!(the_media_away_from_continue_listening().len(), 2);

        // A message carries the whole account, therefore the new list takes the
        // place of the old one: the book that the user hid comes back.
        note_the_media_away_from_continue_listening(vec!["a book that ended".to_string()]);
        assert_eq!(
            the_media_away_from_continue_listening()
                .into_iter()
                .collect::<Vec<_>>(),
            vec!["a book that ended".to_string()]
        );

        // The lists become old, and the key `R` makes them new again.
        note_that_the_lists_are_old();
        assert!(the_lists_are_old());
        the_lists_are_new_again();
        assert!(!the_lists_are_old());

        // That key asks the server for the shelves again, therefore the list of
        // the media away from Continue Listening goes away too. See T-66.
        assert!(the_media_away_from_continue_listening().is_empty());

        // A fault of the connection keeps the positions that came before it.
        keep(State::Fault(
            "the server did not answer in time".to_string(),
        ));
        assert_eq!(
            state(),
            State::Fault("the server did not answer in time".to_string())
        );
        assert!(progress_of("a book").is_some());

        forget();
        assert_eq!(state(), State::Nothing);
        assert!(progress_of("a book").is_none());
    }
}
