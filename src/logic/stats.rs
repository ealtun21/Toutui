//! The statistics of the user, between the task and the screen. See T-24.
//!
//! The render is not asynchronous. Therefore a task asks the server and it
//! puts the answer here, and the render takes it at the next frame. This is
//! the shape of the search of the server and of the cover art of T-23.

use crate::api::me::listening_stats::ListeningStats;
use std::sync::{Mutex, OnceLock};

/// What the screen must draw.
#[derive(Debug, Clone, Default)]
pub enum State {
    /// The program did not ask the server.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered.
    ///
    /// The answer is large, therefore it stands behind a `Box`. A large value
    /// inside a small one makes every copy of the small one expensive.
    Ready(Box<ListeningStats>),
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
///
/// A lock that fails gives `Nothing`. The screen then says that no answer
/// came, and it does not stop.
pub fn state() -> State {
    match box_of_the_state().lock() {
        Ok(place) => place.clone(),
        Err(_) => State::Nothing,
    }
}

/// Forgets the answer. The next request starts from nothing.
pub fn forget() {
    keep(State::Nothing);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The state belongs to the process, therefore the parts of this test
    /// must stay in one function. Two test functions would fight for it.
    #[test]
    fn the_state_goes_from_the_task_to_the_screen() {
        forget();
        assert!(matches!(state(), State::Nothing));

        keep(State::Waiting);
        assert!(matches!(state(), State::Waiting));

        let stats = ListeningStats {
            total_time: 281.0,
            ..Default::default()
        };
        keep(State::Ready(Box::new(stats)));

        match state() {
            State::Ready(stats) => assert_eq!(stats.total_time, 281.0),
            other => panic!("the state must hold the answer: {:?}", other),
        }

        keep(State::Fault("the server does not answer".to_string()));

        match state() {
            State::Fault(text) => assert_eq!(text, "the server does not answer"),
            other => panic!("the state must hold the fault: {:?}", other),
        }

        forget();
        assert!(matches!(state(), State::Nothing));
    }
}
