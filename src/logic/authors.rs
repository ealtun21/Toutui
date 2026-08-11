//! The authors of a library, between the task and the screen. See T-24.

use crate::api::libraries::get_authors::Author;
use std::sync::{Mutex, OnceLock};

/// What the view must draw.
#[derive(Debug, Clone, Default)]
pub enum State {
    /// The program did not ask the server.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered, in the sequence of the alphabet.
    Ready(Vec<Author>),
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

/// Gives the authors that the view holds now.
pub fn authors() -> Vec<Author> {
    match state() {
        State::Ready(all) => all,
        _ => Vec::new(),
    }
}

/// Forgets the answer. A refresh of the program asks the server again.
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
        assert!(authors().is_empty());

        keep(State::Waiting);
        assert!(authors().is_empty());

        keep(State::Ready(vec![Author {
            id: "a".to_string(),
            name: "A Name".to_string(),
            description: None,
            num_books: 3,
        }]));

        assert_eq!(authors().len(), 1);
        assert_eq!(authors()[0].num_books, 3);

        keep(State::Fault("no answer".to_string()));
        assert!(authors().is_empty());

        forget();
    }
}
