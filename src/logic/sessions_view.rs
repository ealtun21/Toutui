//! The sessions of the user, between the task and the screen. See T-24.
//!
//! The render is not asynchronous. Therefore a task asks the server and it puts
//! the answer here, and the render takes it at the next frame. This is the
//! shape of `logic::stats`, of `logic::bookmarks`, and of `logic::authors`.
//!
//! This view holds one thing more than those: **the pages**. The server gives
//! 25 sessions of the whole list, and the view asks for the next page when the
//! user reaches the end of what it holds. [`Loaded`] therefore keeps the
//! sessions of every page that came, and it keeps the page that came last.

use crate::api::me::sessions::{PlaySession, SessionPage};
use std::sync::{Mutex, OnceLock};

/// The sessions that the program holds now.
#[derive(Debug, Clone, Default)]
pub struct Loaded {
    /// The sessions of every page that came, the newest first.
    pub sessions: Vec<PlaySession>,
    /// The number of sessions of the account, over every page.
    pub total: usize,
    /// The page that came last. The first page is 0.
    pub page: usize,
    /// True while a task asks the server for the next page.
    pub asking: bool,
    /// True when the server holds a page after the page that came.
    pub more: bool,
}

impl Loaded {
    /// Puts the first page in a new value.
    pub fn first(page: SessionPage) -> Loaded {
        let mut loaded = Loaded {
            total: page.total,
            page: page.page,
            sessions: page.sessions,
            asking: false,
            more: false,
        };
        loaded.more = loaded.sessions.len() < loaded.total && !loaded.sessions.is_empty();
        loaded
    }

    /// Adds a page after the pages that the value holds.
    ///
    /// A page with no session stops the reads, because the server gives `200`
    /// and an empty list for a page after the last page.
    pub fn add(&mut self, page: SessionPage) {
        self.asking = false;
        if page.sessions.is_empty() {
            self.more = false;
            return;
        }
        self.page = page.page;
        self.total = page.total;
        self.sessions.extend(page.sessions);
        self.more = self.sessions.len() < self.total;
    }

    /// Tells if the program must ask for the next page now.
    ///
    /// The program asks when the server holds more, when no task asks already,
    /// and when the user is near the end of the lines that the view holds.
    pub fn wants_the_next_page(&self, first_line_of_the_screen: usize, lines: usize) -> bool {
        if !self.more || self.asking {
            return false;
        }
        // "Near the end" is the last screen of the lines. A user who scrolls
        // fast then never waits at the end of the list.
        lines == 0 || first_line_of_the_screen + 1 >= lines.saturating_sub(LINES_BEFORE_THE_END)
    }
}

/// The number of lines before the end where the program asks for a page.
const LINES_BEFORE_THE_END: usize = 20;

/// What the screen must draw.
#[derive(Debug, Clone, Default)]
pub enum State {
    /// The program did not ask the server.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered.
    Ready(Box<Loaded>),
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

/// Adds a page to the state, if the state holds the pages before it.
///
/// A page that comes after the user left the view finds a different state, and
/// it changes nothing.
pub fn add_a_page(page: SessionPage) {
    if let Ok(mut place) = box_of_the_state().lock() {
        if let State::Ready(loaded) = &mut *place {
            loaded.add(page);
        }
    }
}

/// Says that a task asks the server for the next page now.
///
/// The mark stops a second task for the same page.
pub fn a_task_asks() -> bool {
    let Ok(mut place) = box_of_the_state().lock() else {
        return false;
    };
    match &mut *place {
        State::Ready(loaded) if loaded.more && !loaded.asking => {
            loaded.asking = true;
            true
        }
        _ => false,
    }
}

/// Says that the request of the next page failed.
///
/// The view keeps the sessions that it holds. The user then reads them, and the
/// program asks again at the next move.
pub fn the_page_did_not_come() {
    if let Ok(mut place) = box_of_the_state().lock() {
        if let State::Ready(loaded) = &mut *place {
            loaded.asking = false;
        }
    }
}

/// Forgets the answer. The next request starts from nothing.
pub fn forget() {
    keep(State::Nothing);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page_of(page: usize, count: usize, total: usize) -> SessionPage {
        SessionPage {
            total,
            num_pages: total.div_ceil(count.max(1)),
            page,
            items_per_page: count,
            sessions: (0..count).map(|_| PlaySession::default()).collect(),
        }
    }

    #[test]
    fn the_first_page_says_that_the_server_holds_more() {
        let loaded = Loaded::first(page_of(0, 25, 60));
        assert_eq!(25, loaded.sessions.len());
        assert_eq!(60, loaded.total);
        assert!(loaded.more);
    }

    #[test]
    fn a_first_page_that_holds_everything_says_that_there_is_no_more() {
        let loaded = Loaded::first(page_of(0, 6, 6));
        assert!(!loaded.more);
    }

    #[test]
    fn an_account_with_no_session_asks_for_no_page() {
        let loaded = Loaded::first(page_of(0, 0, 0));
        assert!(loaded.sessions.is_empty());
        assert!(!loaded.more);
        assert!(!loaded.wants_the_next_page(0, 0));
    }

    #[test]
    fn the_pages_come_one_after_the_other_and_the_reads_stop_at_the_end() {
        let mut loaded = Loaded::first(page_of(0, 25, 60));
        loaded.add(page_of(1, 25, 60));
        assert_eq!(50, loaded.sessions.len());
        assert_eq!(1, loaded.page);
        assert!(loaded.more);

        loaded.add(page_of(2, 10, 60));
        assert_eq!(60, loaded.sessions.len());
        assert!(!loaded.more, "the program holds every session");
    }

    /// A page after the last page gives `200` and an empty list. The program
    /// must stop, and it must not ask for ever.
    #[test]
    fn a_page_with_no_session_stops_the_reads() {
        let mut loaded = Loaded::first(page_of(0, 25, 999));
        assert!(loaded.more);
        loaded.add(page_of(1, 0, 999));
        assert!(!loaded.more);
        assert_eq!(25, loaded.sessions.len());
    }

    #[test]
    fn the_program_asks_for_a_page_near_the_end_and_not_before_it() {
        let loaded = Loaded::first(page_of(0, 25, 60));
        // 100 lines: the program asks after the line 79.
        assert!(!loaded.wants_the_next_page(0, 100));
        assert!(!loaded.wants_the_next_page(50, 100));
        assert!(loaded.wants_the_next_page(79, 100));
        assert!(loaded.wants_the_next_page(99, 100));
        // A list that is shorter than the window asks at once.
        assert!(loaded.wants_the_next_page(0, 5));
    }

    #[test]
    fn a_task_that_asks_already_stops_a_second_task() {
        let mut loaded = Loaded::first(page_of(0, 25, 60));
        loaded.asking = true;
        assert!(!loaded.wants_the_next_page(99, 100));
    }

    #[test]
    fn a_list_that_holds_everything_asks_for_no_page() {
        let loaded = Loaded::first(page_of(0, 6, 6));
        assert!(!loaded.wants_the_next_page(99, 100));
    }

    /// The state belongs to the process, therefore the parts of this test must
    /// stay in one function. Two test functions would fight for it.
    #[test]
    fn the_state_goes_from_the_task_to_the_screen() {
        forget();
        assert!(matches!(state(), State::Nothing));

        keep(State::Waiting);
        assert!(matches!(state(), State::Waiting));
        // A page that comes for a state that is not ready changes nothing.
        add_a_page(page_of(1, 25, 60));
        assert!(matches!(state(), State::Waiting));
        assert!(!a_task_asks(), "a state that is not ready starts no task");

        keep(State::Ready(Box::new(Loaded::first(page_of(0, 25, 60)))));

        assert!(a_task_asks(), "the first task must start");
        assert!(!a_task_asks(), "a second task must not start");

        add_a_page(page_of(1, 25, 60));
        match state() {
            State::Ready(loaded) => {
                assert_eq!(50, loaded.sessions.len());
                assert!(!loaded.asking, "the page came, therefore no task asks");
            }
            other => panic!("the state must hold the sessions: {:?}", other),
        }

        // A request that fails keeps the sessions, and it lets the program ask
        // again.
        assert!(a_task_asks());
        the_page_did_not_come();
        match state() {
            State::Ready(loaded) => {
                assert_eq!(50, loaded.sessions.len());
                assert!(!loaded.asking);
            }
            other => panic!("the state must keep the sessions: {:?}", other),
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
