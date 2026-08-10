//! The bookmarks of one media, between the task and the screen. See T-24.
//!
//! The render is not asynchronous. Therefore a task asks the server and it
//! puts the answer here, and the render takes it at the next frame. This is
//! the shape of the statistics and of the values of the filter.

use crate::api::me::bookmarks::Bookmark;
use std::sync::{Mutex, OnceLock};

/// What the view must draw.
#[derive(Debug, Clone, Default)]
pub enum State {
    /// The program did not ask the server.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered. The list holds the bookmarks of one media, the
    /// first place first.
    Ready(Vec<Bookmark>),
    /// The server gave no answer, and this text says why.
    Fault(String),
}

fn box_of_the_state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(State::Nothing))
}

/// Writes the state. The task of the request calls this.
pub fn keep(state: State) {
    if let Ok(mut place) = box_of_the_state().lock() {
        *place = state;
    }
}

/// Gives the state. The render calls this at each frame.
pub fn state() -> State {
    match box_of_the_state().lock() {
        Ok(place) => place.clone(),
        Err(_) => State::Nothing,
    }
}

/// Gives the bookmarks that the view holds now, and nothing while the program
/// waits.
pub fn bookmarks() -> Vec<Bookmark> {
    match state() {
        State::Ready(all) => all,
        _ => Vec::new(),
    }
}

/// Forgets the answer.
pub fn forget() {
    keep(State::Nothing);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The state belongs to the process, therefore the parts of this test
    /// must stay in one function.
    #[test]
    fn the_state_goes_from_the_task_to_the_screen() {
        forget();
        assert!(matches!(state(), State::Nothing));
        assert!(bookmarks().is_empty());

        keep(State::Waiting);
        assert!(bookmarks().is_empty());

        keep(State::Ready(vec![Bookmark {
            library_item_id: "book-1".to_string(),
            time: 42.0,
            title: "A place".to_string(),
        }]));

        assert_eq!(bookmarks().len(), 1);
        assert_eq!(bookmarks()[0].title, "A place");

        keep(State::Fault("no answer".to_string()));
        assert!(bookmarks().is_empty());

        forget();
    }
}
