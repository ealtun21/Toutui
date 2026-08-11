//! The queue of the downloads of the server, between the task and the screen.
//! See T-81.
//!
//! The render is not asynchronous. Therefore the task of the request puts the
//! answer here, and the render takes it at the next frame. This is the shape of
//! `logic::authors`, of `logic::the_ebooks`, and of `logic::stats`.
//!
//! **The server sends a message at each change of that queue.** The task of the
//! live messages writes a mark here, and the view then asks the server again:
//! the queue moves alone while the user looks at it, and the user presses no
//! key. See `note_that_the_queue_changed`.

use crate::api::podcasts::the_downloads::OneDownload;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// The longest time between two questions of the view.
///
/// The message of the server makes the view ask at each change. A connection of
/// the live messages that is not open gives no such message, therefore the view
/// asks again after this time. One request of three seconds is a small cost, and
/// the view stands open for some seconds only.
pub const THE_TIME_BETWEEN_TWO_QUESTIONS: Duration = Duration::from_secs(3);

/// What the view must draw.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum State {
    /// The program did not ask the server.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered. The episode of now stands first.
    Ready(Vec<OneDownload>),
    /// The server gave no answer, and this text says why.
    Fault(String),
}

#[derive(Debug, Default)]
struct TheQueue {
    state: State,
    /// A message of the server said that the queue changed.
    changed: bool,
    /// The time of the last question of the view.
    asked_at: Option<Instant>,
}

fn box_of_the_queue() -> &'static Mutex<TheQueue> {
    static QUEUE: OnceLock<Mutex<TheQueue>> = OnceLock::new();
    QUEUE.get_or_init(|| Mutex::new(TheQueue::default()))
}

/// Writes the state. The task of the request calls this.
pub fn keep(state: State) {
    if let Ok(mut place) = box_of_the_queue().lock() {
        place.state = state;
    }
}

/// Gives the state. The render calls this at each frame.
pub fn state() -> State {
    match box_of_the_queue().lock() {
        Ok(place) => place.state.clone(),
        Err(_) => State::Nothing,
    }
}

/// Gives the downloads that the view holds now.
pub fn downloads() -> Vec<OneDownload> {
    match state() {
        State::Ready(all) => all,
        _ => Vec::new(),
    }
}

/// The task of the live messages says that the queue of the server changed.
pub fn note_that_the_queue_changed() {
    if let Ok(mut place) = box_of_the_queue().lock() {
        place.changed = true;
    }
}

/// Says that the view must ask the server, and it holds the time of that
/// question.
///
/// The render calls this at each frame. It gives `true` at the first frame of
/// the view, at each message of the server, and after
/// `THE_TIME_BETWEEN_TWO_QUESTIONS`.
pub fn the_view_must_ask() -> bool {
    match box_of_the_queue().lock() {
        Ok(mut place) => {
            let must = the_view_must_ask_now(
                place.changed,
                place.asked_at.map(|time| time.elapsed()),
                THE_TIME_BETWEEN_TWO_QUESTIONS,
            );

            if must {
                place.changed = false;
                place.asked_at = Some(Instant::now());
            }

            must
        }
        Err(_) => false,
    }
}

/// The rule of the question of the view.
///
/// The function is pure, therefore a test needs no time and no server.
pub fn the_view_must_ask_now(
    changed: bool,
    since_the_last_question: Option<Duration>,
    period: Duration,
) -> bool {
    match since_the_last_question {
        // The view did not ask one time. This is its first frame.
        None => true,
        Some(time) => changed || time >= period,
    }
}

/// Forgets the answer and the mark of the change.
pub fn forget() {
    if let Ok(mut place) = box_of_the_queue().lock() {
        *place = TheQueue::default();
    }
}

/// Tells if a message of the server changes the queue of the downloads.
///
/// The four names come from the server of the measurement of 2026-08-11:
/// `grep -rho "episode_download[a-z_]*"` of an Audiobookshelf 2.36.0 gives
/// `episode_download_queued`, `episode_download_started`,
/// `episode_download_finished`, and `episode_download_queue_cleared`.
///
/// The function is pure, therefore a test needs no server.
pub fn the_queue_of_the_downloads_changed(name: &str) -> bool {
    name.starts_with("episode_download")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn one(title: &str) -> OneDownload {
        OneDownload {
            title: title.to_string(),
            item_id: "a-podcast".to_string(),
            podcast: "A Podcast".to_string(),
            now: false,
        }
    }

    #[test]
    fn every_message_of_a_download_makes_the_view_ask_again() {
        assert!(the_queue_of_the_downloads_changed(
            "episode_download_queued"
        ));
        assert!(the_queue_of_the_downloads_changed(
            "episode_download_started"
        ));
        assert!(the_queue_of_the_downloads_changed(
            "episode_download_finished"
        ));
        assert!(the_queue_of_the_downloads_changed(
            "episode_download_queue_cleared"
        ));

        assert!(!the_queue_of_the_downloads_changed("user_updated"));
        assert!(!the_queue_of_the_downloads_changed("item_updated"));
    }

    /// The mark comes one time for each change. A view that asks at each frame
    /// would make some hundred requests each minute.
    #[test]
    fn the_view_asks_at_its_first_frame_and_at_each_change() {
        let period = Duration::from_secs(3);

        // The first frame of the view.
        assert!(the_view_must_ask_now(false, None, period));

        // A frame that comes after it, with no message of the server.
        assert!(!the_view_must_ask_now(
            false,
            Some(Duration::from_millis(50)),
            period
        ));

        // A message of the server came.
        assert!(the_view_must_ask_now(
            true,
            Some(Duration::from_millis(50)),
            period
        ));

        // No message came, and the time went by: a connection of the live
        // messages that is not open must not hold the view at an old list.
        assert!(the_view_must_ask_now(
            false,
            Some(Duration::from_secs(3)),
            period
        ));
    }

    #[test]
    fn the_mark_of_the_change_comes_one_time() {
        let _guard = guard();
        forget();

        // The first question of the view.
        assert!(the_view_must_ask());
        assert!(!the_view_must_ask());

        note_that_the_queue_changed();
        assert!(the_view_must_ask());
        assert!(!the_view_must_ask());
    }

    #[test]
    fn the_answer_of_the_server_comes_to_the_screen() {
        let _guard = guard();
        forget();

        assert_eq!(state(), State::Nothing);

        keep(State::Waiting);
        assert!(downloads().is_empty());

        keep(State::Ready(vec![one("Letter 4"), one("Letter 5")]));
        assert_eq!(downloads().len(), 2);

        forget();
    }
}
