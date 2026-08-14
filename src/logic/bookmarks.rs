//! The bookmarks of one media, between the task and the screen. See T-24.
//!
//! The render is not asynchronous. Therefore a task asks the server and it
//! puts the answer here, and the render takes it at the next frame. This is
//! the shape of the statistics and of the values of the filter.

use crate::api::me::bookmarks::Bookmark;
use std::sync::{Mutex, OnceLock};

/// What the view must draw.
#[derive(Debug, Clone, Default)]
pub enum State {
    /// The program did not ask the server.
    #[default]
    Nothing,
    /// The program asked the server, and no answer came.
    Waiting,
    /// The server answered. The list holds the bookmarks of one media, the
    /// first place first.
    Ready(Vec<Bookmark>),
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

/// Gives the bookmarks that the view holds now, and nothing while the program
/// waits.
pub fn bookmarks() -> Vec<Bookmark> {
    match state() {
        State::Ready(all) => all,
        _ => Vec::new(),
    }
}

/// Forgets the answer.
pub fn forget() {
    keep(State::Nothing);
}

/// What the media of the view of the bookmarks is now. See T-163.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheMediaOfTheBookmarks {
    /// The media whose bookmarks the user opened plays now.
    ItPlays,
    /// The media whose bookmarks the user opened does not play now.
    ItDoesNotPlay,
}

/// Tells if the media of the view of the bookmarks plays now.
///
/// **The media that plays changes while this view stands open, and no key of
/// the user does it**: the media comes to its end, and the queue starts the
/// media of its front (T-24). The list of this view holds the bookmarks of the
/// media that the user opened, therefore the key `b` of this view must write a
/// place of **that** media. The measurement of 2026-08-14: the user read the
/// bookmarks of a book of 30 minutes, the queue started a book of eight hours,
/// and the key `b` wrote a bookmark of that book at 19530 seconds. The view
/// showed the same one line before the key and after it. See T-163, and T-160,
/// T-161, and T-162 for the same rule of three other views.
///
/// `of_the_view` is the media that the user opened, and `of_the_player` is the
/// media of the engine now. A playback that stopped gives nothing.
///
/// The function is pure, therefore a test needs no engine and no screen.
pub fn what_the_media_of_the_bookmarks_is(
    of_the_view: &str,
    of_the_player: Option<&str>,
) -> TheMediaOfTheBookmarks {
    match of_the_player {
        Some(media) if media == of_the_view => TheMediaOfTheBookmarks::ItPlays,
        _ => TheMediaOfTheBookmarks::ItDoesNotPlay,
    }
}

/// The text for the user when the media of the view of the bookmarks does not
/// play.
///
/// **A bookmark holds a place, and a media that does not play has no place**,
/// therefore the key `b` of this view writes nothing. The sentence names the
/// media of the view, and it names the key `V` — that key shows the bookmarks
/// of the media that plays (T-118 and T-143). It names no cause: this program
/// cannot tell a media that came to its end from a media that a key of the
/// player stopped (T-91). See T-163.
pub fn the_text_of_the_media_that_does_not_play(name: &str) -> String {
    format!(
        "The media \"{}\" does not play now, and this key writes a place of it. \
         The key V shows the bookmarks of the media that plays.",
        name
    )
}

/// The title of the view of the bookmarks.
///
/// **The title names the media**, because the media that plays changes with no
/// key of the user: a title of "The bookmarks" alone leaves the user with no
/// way to tell whose places they read. The view of the chapters names its media
/// in the same way. See T-163.
pub fn the_title(name: &str, items: &str) -> String {
    format!("The bookmarks of \"{}\" [{}]", name, items)
}

/// The title of the view of the bookmarks when the media holds no bookmark.
///
/// The text names the key `b`, and the key `b` of this view writes a place of
/// this media alone. See T-118 and T-163.
pub fn the_title_of_no_bookmark(name: &str) -> String {
    format!("\"{}\" has no bookmark. Press b while it plays.", name)
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
        assert!(bookmarks().is_empty());

        keep(State::Waiting);
        assert!(bookmarks().is_empty());

        keep(State::Ready(vec![Bookmark {
            library_item_id: "book-1".to_string(),
            time: 42.0,
            title: "A place".to_string(),
        }]));

        assert_eq!(bookmarks().len(), 1);
        assert_eq!(bookmarks()[0].title, "A place");

        keep(State::Fault("no answer".to_string()));
        assert!(bookmarks().is_empty());

        forget();
    }

    /// The media of the user plays, therefore the key `b` writes a place of it.
    /// See T-163.
    #[test]
    fn the_media_that_the_user_opened_plays() {
        assert_eq!(
            what_the_media_of_the_bookmarks_is("book-1", Some("book-1")),
            TheMediaOfTheBookmarks::ItPlays
        );
    }

    /// **The queue starts the media of its front with no key of the user**, and
    /// the key `b` would then write a place of another media. See T-163.
    #[test]
    fn a_media_that_does_not_play_now_takes_no_place() {
        assert_eq!(
            what_the_media_of_the_bookmarks_is("book-1", Some("book-2")),
            TheMediaOfTheBookmarks::ItDoesNotPlay
        );

        // The media came to its end, and no media plays now.
        assert_eq!(
            what_the_media_of_the_bookmarks_is("book-1", None),
            TheMediaOfTheBookmarks::ItDoesNotPlay
        );
    }

    /// The text names the media of the view, and it promises the key `V` only.
    /// See T-118, T-143, and T-163.
    #[test]
    fn the_text_names_the_media_and_one_key() {
        let text = the_text_of_the_media_that_does_not_play("A Long Test Book");

        assert!(text.contains("A Long Test Book"), "{}", text);
        assert!(text.contains("key V"), "{}", text);
        assert!(!text.contains("press h"), "{}", text);
    }

    /// The title names the media, therefore the user knows whose places they
    /// read. See T-163.
    #[test]
    fn the_title_names_the_media() {
        let title = the_title("A Long Test Book", "1 item");

        assert!(title.contains("A Long Test Book"), "{}", title);
        assert!(title.contains("1 item"), "{}", title);

        let empty = the_title_of_no_bookmark("A Long Test Book");

        assert!(empty.contains("A Long Test Book"), "{}", empty);
        assert!(empty.contains("Press b"), "{}", empty);
    }
}
