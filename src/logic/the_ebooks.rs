//! The list of the ebooks of one media, between the task and the screen.
//! See T-76.
//!
//! The render is not asynchronous. Therefore the task of the request puts the
//! answer here, and the render takes it at the next frame. This is the shape of
//! `src/logic/authors.rs` and of `src/logic/stats.rs`.
//!
//! The list belongs to one media. A user who opens the list of a second media
//! must not read the answer of the first one, therefore the box holds the
//! identity of the media beside the answer.

use crate::api::library_items::the_ebooks::Ebook;
use std::sync::{Mutex, OnceLock};

/// What the view must draw.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum State {
    /// The program did not ask the server.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered. The book of the server stands first.
    Ready(Vec<Ebook>),
    /// The server gave no answer, and this text says why.
    Fault(String),
}

#[derive(Debug, Clone, Default)]
struct TheList {
    state: State,
    /// The media of this answer.
    item_id: String,
}

fn box_of_the_list() -> &'static Mutex<TheList> {
    static LIST: OnceLock<Mutex<TheList>> = OnceLock::new();
    LIST.get_or_init(|| Mutex::new(TheList::default()))
}

/// Writes the state of one media. The task of the request calls this.
///
/// The write gives nothing when a different media holds the box: the user left
/// that list, and the answer of the media that came before has no value.
pub fn keep(item_id: &str, state: State) {
    if let Ok(mut place) = box_of_the_list().lock() {
        if place.item_id != item_id {
            return;
        }

        place.state = state;
    }
}

/// The user opens the list of this media. The box forgets the media that came
/// before it.
pub fn ask_for(item_id: &str) {
    if let Ok(mut place) = box_of_the_list().lock() {
        place.item_id = item_id.to_string();
        place.state = State::Waiting;
    }
}

/// Gives the state. The render calls this at each frame.
pub fn state() -> State {
    match box_of_the_list().lock() {
        Ok(place) => place.state.clone(),
        Err(_) => State::Nothing,
    }
}

/// Gives the media of the list that the box holds.
pub fn item_id() -> String {
    match box_of_the_list().lock() {
        Ok(place) => place.item_id.clone(),
        Err(_) => String::new(),
    }
}

/// Gives the ebooks that the view holds now.
pub fn ebooks() -> Vec<Ebook> {
    match state() {
        State::Ready(all) => all,
        _ => Vec::new(),
    }
}

/// Forgets the answer.
pub fn forget() {
    if let Ok(mut place) = box_of_the_list().lock() {
        *place = TheList::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tests of one file run at the same time on many threads, and this
    /// module holds one value for the whole process.
    fn guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn one(name: &str) -> Ebook {
        Ebook {
            ino: name.to_string(),
            name: name.to_string(),
            size: 0,
            is_the_book_of_the_server: false,
        }
    }

    #[test]
    fn the_answer_of_the_media_of_the_user_comes_to_the_screen() {
        let _guard = guard();
        forget();

        ask_for("item-X");
        assert_eq!(state(), State::Waiting);

        keep("item-X", State::Ready(vec![one("a.epub")]));
        assert_eq!(ebooks().len(), 1);
    }

    /// The user opened the list of a second media before the first answer came.
    /// That answer must not stand on the screen of the second media.
    #[test]
    fn the_answer_of_a_different_media_gives_nothing() {
        let _guard = guard();
        forget();

        ask_for("item-X");
        ask_for("item-Y");

        keep("item-X", State::Ready(vec![one("a.epub")]));

        assert_eq!(state(), State::Waiting, "the list of item-Y still waits");
        assert_eq!(item_id(), "item-Y");
    }

    #[test]
    fn a_fault_of_the_server_comes_to_the_screen() {
        let _guard = guard();
        forget();

        ask_for("item-X");
        keep("item-X", State::Fault("404".to_string()));

        assert_eq!(state(), State::Fault("404".to_string()));
        assert!(ebooks().is_empty());
    }
}
