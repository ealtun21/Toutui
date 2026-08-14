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

/// What the line of the view of the downloads holds now. See T-166.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheLineOfTheDownloads {
    /// The episode of the line stands in the queue of the server, at this
    /// place. The line goes to that place: the user chose that episode, and not
    /// that number of a line.
    ItStandsAt(usize),
    /// The episode of the line is not in the queue of the server now.
    ItWentAway,
    /// The user moved the cursor after the frame that the program holds. The
    /// line of the user is the truth of the choice, therefore the program reads
    /// the episode of that line again.
    TheUserChoseAnother,
}

/// Tells what the line of the view of the downloads must hold now.
///
/// **The line of the user holds an episode, and not a number of a line.** This
/// view is the second list of the program that moves with no key of any user
/// at all (the queue of the media of T-161 is the first): the server takes an
/// episode out of the queue when it downloaded it, it sends a message of that
/// change, and the view then asks the server again and draws the new list at
/// that frame.
///
/// A measurement of 2026-08-14 held a user on the line 5, `Chapter 10` of
/// `Narrative of Arthur Gordon Pym`. Two episodes came to their end, and the
/// line 5 then held `Letter 12` of `Letters of Two Brides` with no word at all:
/// the two presses of the key `X` emptied the queue of a podcast that the user
/// never chose, and eight episodes went away. **The queue of the downloads
/// belongs to the library**, therefore the cost of that key is larger than a
/// view of one user. See T-166, and the same rule of T-160, of T-161, of T-162,
/// of T-163, and of T-165.
///
/// The function is pure, therefore a test needs no server.
pub fn what_the_line_of_the_downloads_holds(
    all: &[OneDownload],
    of_the_program: Option<(usize, &str)>,
    of_the_user: Option<usize>,
) -> TheLineOfTheDownloads {
    let Some((line, key)) = of_the_program else {
        return TheLineOfTheDownloads::TheUserChoseAnother;
    };

    if of_the_user != Some(line) {
        return TheLineOfTheDownloads::TheUserChoseAnother;
    }

    match all.iter().position(|one| one.key() == key) {
        Some(place) => TheLineOfTheDownloads::ItStandsAt(place),
        None => TheLineOfTheDownloads::ItWentAway,
    }
}

/// The text for the user when the episode of their line leaves the queue of the
/// server.
///
/// **The program cannot say why that episode left**: the server downloaded it,
/// or a second program of the library emptied that queue. Therefore the text
/// says what the program knows (T-91). It names the two keys that give a line
/// again, and it promises no other key (T-118 and T-143). See T-166.
pub fn the_text_of_the_episode_that_went_away(title: &str, podcast: &str) -> String {
    format!(
        "The episode \"{}\" of \"{}\" is not in the queue of the server now. \
         No line is selected: the keys j and k select one.",
        title, podcast
    )
}

/// The line of the key `j` and of the key `k` in the view of the downloads.
///
/// **A view that holds no line gives its first line.** The text of the episode
/// that went away names these two keys, therefore these two keys must give a
/// line again: `ListState::select_previous` of ratatui gives `usize::MAX` to a
/// line of nobody, and the rule of the line then takes that line to nobody one
/// more time. See T-166.
///
/// The key `j` goes in a ring, and the key `k` stops at the first line. This is
/// the shape of the two keys of this view before T-166.
///
/// The function is pure, therefore a test needs no server.
pub fn the_line_of_the_move(of_the_user: Option<usize>, count: usize, down: bool) -> Option<usize> {
    if count == 0 {
        return None;
    }

    let Some(line) = of_the_user.filter(|line| *line < count) else {
        return Some(0);
    };

    Some(if down {
        if line + 1 < count {
            line + 1
        } else {
            0
        }
    } else {
        line.saturating_sub(1)
    })
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

    fn of(podcast: &str, title: &str) -> OneDownload {
        OneDownload {
            title: title.to_string(),
            item_id: podcast.to_string(),
            podcast: podcast.to_string(),
            now: false,
        }
    }

    /// **The line of the user holds an episode, and not a number of a line.**
    /// The server took two episodes out of the queue when it downloaded them,
    /// and the user of the line 5 then stood on an episode of a podcast that
    /// they never chose: the key `X` named that podcast, and the two presses
    /// emptied its queue. See T-166.
    #[test]
    fn the_line_of_the_downloads_holds_an_episode_and_not_a_number() {
        let before = vec![
            of("pym", "Chapter 8"),
            of("pym", "Chapter 9"),
            of("pym", "Chapter 10"),
            of("brides", "Letter 12"),
        ];

        // The server downloaded the two episodes of the front.
        let after = vec![before[2].clone(), before[3].clone()];

        // The episode of the line stays: the line follows it to its new place.
        assert_eq!(
            what_the_line_of_the_downloads_holds(&after, Some((2, &before[2].key())), Some(2)),
            TheLineOfTheDownloads::ItStandsAt(0),
            "the episode of the line stands two lines higher now"
        );

        // Nothing changed: the line stays where it stands.
        assert_eq!(
            what_the_line_of_the_downloads_holds(&before, Some((3, &before[3].key())), Some(3)),
            TheLineOfTheDownloads::ItStandsAt(3),
            "a queue that did not move keeps the line of the user"
        );

        // The episode of the line went away, and the line of that number holds
        // an episode of another podcast now. **This is the fault of T-166.**
        assert_eq!(
            what_the_line_of_the_downloads_holds(&after, Some((0, &before[0].key())), Some(0)),
            TheLineOfTheDownloads::ItWentAway
        );

        // Every episode went away.
        assert_eq!(
            what_the_line_of_the_downloads_holds(&[], Some((0, &before[0].key())), Some(0)),
            TheLineOfTheDownloads::ItWentAway
        );

        // The user moved the cursor after that frame.
        assert_eq!(
            what_the_line_of_the_downloads_holds(&before, Some((3, &before[3].key())), Some(1)),
            TheLineOfTheDownloads::TheUserChoseAnother
        );
        assert_eq!(
            what_the_line_of_the_downloads_holds(&before, None, Some(1)),
            TheLineOfTheDownloads::TheUserChoseAnother
        );

        // **The episode that the server downloads now is the same episode.**
        // It moves from `queue` to `currentDownload` of the answer, and the
        // field `now` therefore stands outside the name of the line.
        let now = OneDownload {
            now: true,
            ..before[2].clone()
        };
        assert_eq!(now.key(), before[2].key());
        assert_eq!(
            what_the_line_of_the_downloads_holds(&[now], Some((2, &before[2].key())), Some(2)),
            TheLineOfTheDownloads::ItStandsAt(0)
        );

        // Two episodes of one podcast are two episodes.
        assert_ne!(before[0].key(), before[1].key());
    }

    /// **The two keys that the text of the episode names must give a line
    /// again.** The key `k` of ratatui gives `usize::MAX` to a line of nobody,
    /// and the rule of the line then takes that line to nobody one more time:
    /// the view would hold no line for ever. See T-166.
    #[test]
    fn the_keys_j_and_k_give_a_line_to_a_view_that_holds_none() {
        assert_eq!(the_line_of_the_move(None, 4, true), Some(0));
        assert_eq!(the_line_of_the_move(None, 4, false), Some(0));

        // The line of ratatui after the key `k` of a line of nobody.
        assert_eq!(the_line_of_the_move(Some(usize::MAX), 4, false), Some(0));

        // The key `j` goes in a ring, and the key `k` stops at the first line.
        assert_eq!(the_line_of_the_move(Some(2), 4, true), Some(3));
        assert_eq!(the_line_of_the_move(Some(3), 4, true), Some(0));
        assert_eq!(the_line_of_the_move(Some(2), 4, false), Some(1));
        assert_eq!(the_line_of_the_move(Some(0), 4, false), Some(0));

        // The queue of the server can be empty, and no key gives a line then.
        assert_eq!(the_line_of_the_move(None, 0, true), None);
        assert_eq!(the_line_of_the_move(Some(0), 0, false), None);
    }

    /// The text names the episode and its podcast, and it promises no key that
    /// the view does not hold (T-118 and T-143). See T-166.
    #[test]
    fn the_text_names_the_episode_that_went_away() {
        let text =
            the_text_of_the_episode_that_went_away("Chapter 10", "Narrative of Arthur Gordon Pym");

        assert!(text.contains("Chapter 10"), "{}", text);
        assert!(text.contains("Narrative of Arthur Gordon Pym"), "{}", text);
        assert!(text.contains("j and k"), "{}", text);
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
