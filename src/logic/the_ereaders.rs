//! The devices of an e-reader, between the task and the screen. See T-119.
//!
//! The render is not asynchronous. Therefore the task of the request puts the
//! answer here, and the render takes it at the next frame. This is the shape of
//! `logic::the_ebooks`, of `logic::the_downloads`, and of `logic::authors`.
//!
//! **The program asks the server at the key**, and it holds no list from the
//! start: an administrator of the server adds a device while this program runs,
//! and one request of `POST /api/authorize` costs a few milliseconds.

use crate::api::ereaders::Device;
use std::sync::{Mutex, OnceLock};

/// What the view must draw.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum State {
    /// The program did not ask the server.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered. An empty list is an answer: the server holds no
    /// device.
    Ready(Vec<Device>),
    /// The server gave no answer, and this text says why.
    Fault(String),
}

#[derive(Debug, Default)]
struct TheDevices {
    state: State,
}

fn box_of_the_devices() -> &'static Mutex<TheDevices> {
    static DEVICES: OnceLock<Mutex<TheDevices>> = OnceLock::new();
    DEVICES.get_or_init(|| Mutex::new(TheDevices::default()))
}

/// Writes the state. The task of the request calls this.
pub fn keep(state: State) {
    if let Ok(mut place) = box_of_the_devices().lock() {
        place.state = state;
    }
}

/// The user opened the view. The box forgets the answer that came before it.
pub fn ask() {
    keep(State::Waiting);
}

/// Gives the state. The render calls this at each frame.
pub fn state() -> State {
    match box_of_the_devices().lock() {
        Ok(place) => place.state.clone(),
        Err(_) => State::Nothing,
    }
}

/// Gives the devices that the view holds now.
pub fn devices() -> Vec<Device> {
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

    /// The tests of one file run at the same time on many threads, and this
    /// module holds one value for the whole process.
    fn guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn one(name: &str) -> Device {
        Device {
            name: name.to_string(),
            email: format!("{}@example.invalid", name),
        }
    }

    #[test]
    fn the_key_gives_the_state_of_a_wait() {
        let _guard = guard();
        forget();

        ask();

        assert_eq!(state(), State::Waiting);
        assert!(devices().is_empty());
    }

    #[test]
    fn the_answer_of_the_server_comes_to_the_render() {
        let _guard = guard();
        forget();

        ask();
        keep(State::Ready(vec![one("Kobo")]));

        assert_eq!(devices().len(), 1);
        assert_eq!(devices()[0].name, "Kobo");
    }

    #[test]
    fn a_server_that_holds_no_device_is_an_answer_and_not_a_fault() {
        let _guard = guard();
        forget();

        keep(State::Ready(Vec::new()));

        assert_eq!(state(), State::Ready(Vec::new()));
        assert!(devices().is_empty());
    }

    #[test]
    fn a_fault_holds_the_reason() {
        let _guard = guard();
        forget();

        keep(State::Fault("no answer".to_string()));

        assert_eq!(state(), State::Fault("no answer".to_string()));
        assert!(devices().is_empty());
    }
}
