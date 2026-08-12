//! The episodes of one podcast, between the task and the screen. See T-126.
//!
//! **The start of the program read the episodes of every podcast of the
//! library, one request after the other.** A library of 520 podcasts therefore
//! made 500 requests before the first frame: the sweep of 2026-08-12 measured
//! **11.9 seconds** for that frame with a server of 20 milliseconds, and 409
//! milliseconds for a library of one podcast of the same server.
//!
//! Two faults came out of that shape:
//!
//! - a podcast of a page that the program did not read had **no row at all** in
//!   those lists, and the key `l` of that line stopped the program with an
//!   index of a vector that does not exist;
//! - the page that came after the first gave no episode to its podcasts,
//!   therefore the view said "This podcast has no episode" for a podcast of one
//!   episode.
//!
//! The program reads the episodes of **one** podcast now, when the user opens
//! that podcast. This is the shape of `logic::library_pages` (T-70): a task
//! reads, the box holds the answer, and the render takes it at the next frame.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

/// The episodes of one podcast, in the shape that the lists of the screen need.
///
/// The task makes these lists, because the functions that collect them are
/// asynchronous and the render is not.
#[derive(Debug, Clone, Default)]
pub struct Episodes {
    /// The place of the podcast in the lists of the library.
    pub place: usize,
    /// The identity of the podcast. The place alone is not enough: a new
    /// library or a new filter moves every line.
    pub id: String,
    pub titles: Vec<String>,
    pub ids: Vec<String>,
    pub subtitles: Vec<String>,
    pub seasons: Vec<String>,
    pub numbers: Vec<String>,
    pub authors: Vec<String>,
    pub descriptions: Vec<String>,
    pub titles_of_the_podcast: Vec<String>,
    pub durations: Vec<String>,
}

fn the_episodes_that_wait() -> &'static Mutex<Option<Episodes>> {
    static EPISODES: OnceLock<Mutex<Option<Episodes>>> = OnceLock::new();
    EPISODES.get_or_init(|| Mutex::new(None))
}

fn the_flag() -> &'static AtomicBool {
    static ASKING: OnceLock<AtomicBool> = OnceLock::new();
    ASKING.get_or_init(|| AtomicBool::new(false))
}

/// Tells if a task asks the server for the episodes of a podcast now.
pub fn asks() -> bool {
    the_flag().load(Ordering::SeqCst)
}

/// Writes that a task asks the server, or that it does not.
pub fn keep_the_flag(asking: bool) {
    the_flag().store(asking, Ordering::SeqCst);
}

/// Puts the episodes in the box that the render reads. The task calls this.
pub fn keep(episodes: Episodes) {
    if let Ok(mut place) = the_episodes_that_wait().lock() {
        *place = Some(episodes);
    }
}

/// Takes the episodes that wait, and it leaves the box empty.
pub fn take() -> Option<Episodes> {
    match the_episodes_that_wait().lock() {
        Ok(mut place) => place.take(),
        Err(_) => None,
    }
}

/// Empties the box and the flag.
///
/// A new library, a new filter, and the key `R` all make the lines of the
/// library before them wrong.
pub fn forget() {
    keep_the_flag(false);

    if let Ok(mut place) = the_episodes_that_wait().lock() {
        *place = None;
    }
}

/// The sentence of the view of the episodes, while no episode stands there.
///
/// **A view must not give a reason that the program does not have** (T-91). The
/// view said "This podcast has no episode" for every podcast whose episodes the
/// program did not read, and a podcast of one episode met that sentence.
///
/// The three conditions: the answer of the server came and it holds no episode;
/// the program asks the server now; or the server does not answer at all.
///
/// The function is pure, therefore a test needs no server.
pub fn the_reason_of_no_episode(the_episodes_came: bool, is_offline: bool) -> &'static str {
    if the_episodes_came {
        return "This podcast has no episode.";
    }

    if is_offline {
        return "The server does not answer, therefore this program does not have the \
                episodes of this podcast.";
    }

    "The program gets the episodes of this podcast…"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The box holds one answer, and `take` leaves it empty.
    #[test]
    fn the_box_gives_the_episodes_one_time() {
        forget();

        keep(Episodes {
            place: 3,
            id: "a-podcast".to_string(),
            titles: vec!["episode-1".to_string()],
            ..Episodes::default()
        });

        let came = take().expect("the box holds the episodes");
        assert_eq!(came.place, 3);
        assert_eq!(came.id, "a-podcast");
        assert!(take().is_none(), "the box is empty now");

        forget();
    }

    /// A view says why it holds no line, and it says a reason that the program
    /// has. See T-126 and T-91.
    #[test]
    fn the_view_says_why_it_holds_no_episode() {
        // The answer of the server came, and that podcast holds no episode.
        assert_eq!(
            the_reason_of_no_episode(true, false),
            "This podcast has no episode."
        );

        // The program did not read the episodes of that podcast yet.
        assert!(the_reason_of_no_episode(false, false).contains("gets the episodes"));

        // The server does not answer. The program knows nothing of that
        // podcast, therefore it says so (T-91).
        assert!(the_reason_of_no_episode(false, true).contains("does not answer"));
    }
}
