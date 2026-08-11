//! Add a podcast, between the task and the screen. See T-24.
//!
//! The render is not asynchronous. Therefore a task asks the server and it
//! puts the answer here, and the render takes it at the next frame.

use crate::api::podcasts::Found;
use std::sync::{Mutex, OnceLock};

/// What the view must draw.
#[derive(Debug, Clone, Default)]
pub enum State {
    /// The user did not search.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered.
    Ready(Vec<Found>),
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

/// Gives the answers that the view holds now.
pub fn found() -> Vec<Found> {
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
        assert!(found().is_empty());

        keep(State::Waiting);
        assert!(found().is_empty());

        keep(State::Ready(vec![Found {
            title: "A Podcast".to_string(),
            artist_name: "A Name".to_string(),
            description_plain: String::new(),
            feed_url: "https://example.test/feed".to_string(),
            track_count: 3,
        }]));

        assert_eq!(found().len(), 1);
        assert_eq!(found()[0].title, "A Podcast");

        keep(State::Fault("no answer".to_string()));
        assert!(found().is_empty());

        forget();
    }
}
