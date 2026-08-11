//! The statistics of the user, between the task and the screen. See T-24.
//!
//! The render is not asynchronous. Therefore a task asks the server and it
//! puts the answer here, and the render takes it at the next frame. This is
//! the shape of the search of the server and of the cover art of T-23.

use crate::api::me::listening_stats::ListeningStats;
use crate::api::stats::{LibraryStats, YearStats};
use std::sync::{Mutex, OnceLock};

/// Everything that the view of the key `T` shows.
///
/// The view sends three requests. The time of the user is the important one,
/// therefore a fault of that request gives `State::Fault` and the screen shows
/// nothing else. The two other requests give a group each, and a fault of one
/// of them takes that group away only. A user with no permission for the
/// statistics of the year then keeps the rest of the view.
#[derive(Debug, Clone, Default)]
pub struct Statistics {
    /// The time of the user. `GET /api/me/listening-stats`.
    pub listening: ListeningStats,
    /// The size of the library. `GET /api/libraries/:id/stats`.
    pub library: Option<LibraryStats>,
    /// The name of the library that `library` counts.
    pub library_name: String,
    /// The work of the year. `GET /api/stats/year/:year`.
    pub year: Option<YearStats>,
    /// The year that `year` counts.
    pub year_number: i32,
}

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
    Ready(Box<Statistics>),
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

        let all = Statistics {
            listening: ListeningStats {
                total_time: 281.0,
                ..Default::default()
            },
            year_number: 2026,
            ..Default::default()
        };
        keep(State::Ready(Box::new(all)));

        match state() {
            State::Ready(all) => {
                assert_eq!(all.listening.total_time, 281.0);
                assert_eq!(all.year_number, 2026);
                // A fault of one of the two other requests takes its group
                // away, and it keeps the rest of the view.
                assert!(all.library.is_none());
                assert!(all.year.is_none());
            }
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
