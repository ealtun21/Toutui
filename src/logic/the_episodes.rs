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
    /// The length of each episode, in seconds. The text of `durations` gives
    /// no number, and the key `n` of this view needs the length of the media
    /// for the line of the view of the queue. See T-236.
    pub lengths: Vec<Option<f64>>,
    /// The place of the user of each episode: the percent and the mark of the
    /// end, in the form of `App::book_progress_cnt_list`. See T-229.
    pub places: Vec<Vec<String>>,
}

/// Gives the text of each line of the view of the episodes of a podcast.
/// See T-229.
///
/// **A line of this view is one episode**, and the identity of the item names
/// every episode of one podcast (T-223). The key of a line therefore names the
/// episode after the item, and it is the key of
/// `crate::logic::live::the_key_of_the_media`: the mark of the media that plays
/// stands on the line of the episode that plays, and on no other line of that
/// podcast.
///
/// `places` holds one row for each episode, in the form of
/// `App::book_progress_cnt_list`: the percent of the user and the mark of the
/// end. A line of no row takes no mark, as a media that never played takes
/// none.
///
/// **The view said nothing at all before this function** (T-229): the user
/// played `Chapter 02` of `Arthur Gordon Pym` of the sandbox and the eleven
/// lines of that view each held the title alone, while the Library view of that
/// same second gave `▶ Arthur Gordon Pym` and the Home view gave
/// `▶ Chapter 02`.
///
/// The function is pure, therefore a test needs no server and no screen.
pub fn the_lines_of_the_episodes(
    podcast_id: &str,
    titles: &[String],
    ids: &[String],
    places: &[Vec<String>],
    playing: Option<&str>,
) -> Vec<String> {
    titles
        .iter()
        .enumerate()
        .map(|(line, title)| {
            // An episode whose identity the server did not give holds no key,
            // therefore no media that plays can stand on its line. See T-226.
            let key = ids
                .get(line)
                .filter(|one| !one.is_empty())
                .map(|episode| crate::logic::live::the_key_of_the_media(podcast_id, Some(episode)));

            let plays_now = key
                .as_ref()
                .zip(playing)
                .is_some_and(|(key, playing)| key == playing);

            let row = places.get(line);
            let percent = row
                .and_then(|row| row.first())
                .map(String::as_str)
                .unwrap_or("");
            let finished = row
                .and_then(|row| row.get(1))
                .map(String::as_str)
                .unwrap_or("");

            crate::ui::marks::line(
                &crate::ui::marks::of_progress(percent, finished, plays_now),
                title,
            )
        })
        .collect()
}

fn the_episodes_that_wait() -> &'static Mutex<Option<Episodes>> {
    static EPISODES: OnceLock<Mutex<Option<Episodes>>> = OnceLock::new();
    EPISODES.get_or_init(|| Mutex::new(None))
}

fn the_flag() -> &'static AtomicBool {
    static ASKING: OnceLock<AtomicBool> = OnceLock::new();
    ASKING.get_or_init(|| AtomicBool::new(false))
}

/// The box of the request that did not come back. See T-168.
///
/// The box holds the place of the podcast and what the server said. **A request
/// of a key belongs to one podcast**: a user who opens a second podcast must
/// not read the fault of the first one.
fn the_fault_that_waits() -> &'static Mutex<Option<(usize, String)>> {
    static FAULT: OnceLock<Mutex<Option<(usize, String)>>> = OnceLock::new();
    FAULT.get_or_init(|| Mutex::new(None))
}

/// Writes that the server did not give the episodes of one podcast. The task
/// calls this. See T-168.
pub fn keep_the_fault(place: usize, what_the_server_said: &str) {
    if let Ok(mut slot) = the_fault_that_waits().lock() {
        *slot = Some((place, what_the_server_said.to_string()));
    }
}

/// Gives what the server said of the podcast of this place, and `None` for a
/// podcast whose request holds no fault. See T-168.
pub fn the_fault_of(place: usize) -> Option<String> {
    match the_fault_that_waits().lock() {
        Ok(slot) => slot
            .as_ref()
            .filter(|(of_the_podcast, _)| *of_the_podcast == place)
            .map(|(_, text)| text.clone()),
        Err(_) => None,
    }
}

/// Takes the fault of one podcast away. A new request of that podcast calls
/// this, and the answer that comes calls it too. See T-168.
pub fn forget_the_fault_of(place: usize) {
    if let Ok(mut slot) = the_fault_that_waits().lock() {
        if slot
            .as_ref()
            .is_some_and(|(of_the_podcast, _)| *of_the_podcast == place)
        {
            *slot = None;
        }
    }
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
    // The answer of that podcast came. A fault of a request before it is not
    // the truth of this podcast now. See T-168.
    forget_the_fault_of(episodes.place);

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

    if let Ok(mut slot) = the_fault_that_waits().lock() {
        *slot = None;
    }
}

/// The sentence of the view of the episodes, while no episode stands there.
///
/// **A view must not give a reason that the program does not have** (T-91). The
/// view said "This podcast has no episode" for every podcast whose episodes the
/// program did not read, and a podcast of one episode met that sentence.
///
/// The four conditions: the answer of the server came and it holds no episode;
/// **the request of the program did not come back**; the program asks the
/// server now; or the server did not answer at the start of the program.
///
/// **The request that did not come back said nothing at all** (T-168). The
/// server went away while the program stood, therefore `is_offline` of the
/// start holds `false`: the view said "The program gets the episodes of this
/// podcast…" for ever, and the program had stopped that work 28 seconds before.
/// A text must not promise a function that the program does not have (T-118),
/// and a view must not give a reason that the program does not have (T-91).
///
/// The sentence names what the server said, and it names no media: the title of
/// the podcast stands in the header of the view already. It promises no key
/// (T-118 and T-143).
///
/// The function is pure, therefore a test needs no server.
pub fn the_reason_of_no_episode(
    the_episodes_came: bool,
    is_offline: bool,
    what_the_server_said: Option<&str>,
) -> String {
    if the_episodes_came {
        return "This podcast has no episode.".to_string();
    }

    if let Some(fault) = what_the_server_said {
        return format!(
            "The server did not give the episodes of this podcast: {}",
            fault
        );
    }

    if is_offline {
        return "The server does not answer, therefore this program does not have the \
                episodes of this podcast."
            .to_string();
    }

    "The program gets the episodes of this podcast…".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The box holds one answer, and `take` leaves it empty. The box of the
    /// fault holds the request that did not come back, and it holds it for one
    /// podcast alone (T-168).
    ///
    /// **The parts of this test stay in one function**: two test functions of
    /// one module fight for the boxes of the process (T-144 and T-157).
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

        // The request of the podcast of the place 3 did not come back. See
        // T-168.
        keep_the_fault(3, "No server address answered.");

        assert_eq!(
            the_fault_of(3).as_deref(),
            Some("No server address answered."),
            "the view of that podcast must read the fault of its own request"
        );

        // **A user who opens a second podcast must not read the fault of the
        // first one.**
        assert_eq!(the_fault_of(4), None);

        // The answer of that podcast came. The fault of the request before it
        // is not the truth of this moment.
        keep(Episodes {
            place: 3,
            id: "a-podcast".to_string(),
            ..Episodes::default()
        });

        assert_eq!(the_fault_of(3), None, "the answer takes the fault away");

        let _ = take();

        // A new request of that podcast takes the fault away too.
        keep_the_fault(3, "No server address answered.");
        forget_the_fault_of(4);
        assert!(
            the_fault_of(3).is_some(),
            "a request of another podcast keeps this fault"
        );
        forget_the_fault_of(3);
        assert_eq!(the_fault_of(3), None);

        // A new library and the key `R` take every box away.
        keep_the_fault(7, "No server address answered.");
        forget();
        assert_eq!(the_fault_of(7), None);
    }

    /// A view says why it holds no line, and it says a reason that the program
    /// has. See T-126 and T-91.
    #[test]
    fn the_view_says_why_it_holds_no_episode() {
        // The answer of the server came, and that podcast holds no episode.
        assert_eq!(
            the_reason_of_no_episode(true, false, None),
            "This podcast has no episode."
        );

        // The program did not read the episodes of that podcast yet.
        assert!(the_reason_of_no_episode(false, false, None).contains("gets the episodes"));

        // The server does not answer. The program knows nothing of that
        // podcast, therefore it says so (T-91).
        assert!(the_reason_of_no_episode(false, true, None).contains("does not answer"));

        // **The request of the program did not come back** (T-168). The server
        // went away while the program stood, therefore `is_offline` of the
        // start holds `false`: the view said "The program gets the episodes of
        // this podcast…" for ever, and no episode ever came.
        let text = the_reason_of_no_episode(false, false, Some("No server address answered."));

        assert_eq!(
            text,
            "The server did not give the episodes of this podcast: No server address answered."
        );

        assert!(
            !text.contains("gets the episodes"),
            "the view must not promise a work that the program stopped: {:?}",
            text
        );

        // The fault of the request stands above the words of the offline mode
        // of the start: the program made that request, therefore it knows more
        // than the state of its start.
        assert!(the_reason_of_no_episode(false, true, Some("a fault")).contains("a fault"));

        // The answer that came stands above them all: a podcast of no episode
        // is not a podcast of a fault.
        assert_eq!(
            the_reason_of_no_episode(true, false, Some("a fault")),
            "This podcast has no episode."
        );
    }
}
