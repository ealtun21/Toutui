//! The collections and the playlists, between the task and the screen.
//! See T-84.
//!
//! The program changes a list of the server now: the key `m` puts a media in a
//! list, and the key `X` takes one out. **The screen must follow that change**,
//! and the request runs in a task that holds no `App`. Therefore the task puts
//! the new lists here, and the render takes them at the next frame.
//!
//! This is the shape of `logic::the_downloads` and of `logic::authors`. The
//! difference: the answer goes into `App` itself (`app.lists`), because every
//! view of the lists reads that field already.

use crate::api::client::ApiClient;
use crate::api::utils::collect_lists::{ListKind, ListView};
use std::sync::{Mutex, OnceLock};

/// What the line of the view of the lists holds after a new answer of the
/// server. See T-165.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheLineOfTheLists {
    /// The list of the line stands in the answer, at this place. The line goes
    /// to that place: the user chose that list, and not that number.
    TheSameList(usize),
    /// The line held no list.
    NoLine,
    /// The list of the line is not in the answer of the server any more.
    ThatListWentAway,
}

/// Tells what the line of the view of the lists must hold now.
///
/// **The line of the user holds a list, and not a number of a line.** The
/// answer of the server takes the place of the lists of the screen at a frame,
/// and a list that a second program removed moves the list below it under that
/// line with no word at all: a measurement of 2026-08-14 held a user in the
/// media of "A Test Playlist", and the key `X` of that user then took a media
/// out of "Z Second Playlist" — a list that they never opened. See T-165, and
/// the same rule of T-160, of T-161, of T-162, and of T-163.
///
/// The function is pure, therefore a test needs no server.
pub fn what_the_line_of_the_lists_holds(
    the_list_of_the_line: Option<&str>,
    lists: &[ListView],
) -> TheLineOfTheLists {
    let Some(id) = the_list_of_the_line else {
        return TheLineOfTheLists::NoLine;
    };

    match lists.iter().position(|one| one.id == id) {
        Some(place) => TheLineOfTheLists::TheSameList(place),
        None => TheLineOfTheLists::ThatListWentAway,
    }
}

/// The text for the user when the collection or the playlist of their line
/// goes away.
///
/// **The program cannot know which list the user wants now**, therefore it
/// takes the line away and it says what happened. A key of the selection then
/// reaches no list at all, and the user chooses the next one. The text names
/// the two keys that this view holds, and it promises no other key (T-118 and
/// T-143). See T-165.
pub fn the_text_of_the_list_that_went_away(kind: ListKind, name: &str) -> String {
    format!(
        "The {} \"{}\" is not on the server now. \
         No line is selected: the keys j and k select one.",
        kind.name().to_lowercase(),
        name
    )
}

/// The text for the user who stands in the media of a list that went away.
///
/// **That view holds nothing at all without its list**: no title, no line, and
/// a footer of five keys that do nothing. Therefore the program gives the user
/// the view of the lists again, and it says why. See T-165.
pub fn the_text_of_the_media_of_a_list_that_went_away(kind: ListKind, name: &str) -> String {
    format!(
        "The {} \"{}\" is not on the server now. \
         This view shows the collections and the playlists again.",
        kind.name().to_lowercase(),
        name
    )
}

fn box_of_the_lists() -> &'static Mutex<Option<Vec<ListView>>> {
    static LISTS: OnceLock<Mutex<Option<Vec<ListView>>>> = OnceLock::new();
    LISTS.get_or_init(|| Mutex::new(None))
}

/// Writes the lists that the server gave. The task of the request calls this.
pub fn keep(lists: Vec<ListView>) {
    if let Ok(mut place) = box_of_the_lists().lock() {
        *place = Some(lists);
    }
}

/// Gives the lists one time, and it forgets them.
///
/// The render calls this at each frame. A frame that gets `None` draws the
/// lists that the program holds already.
pub fn take() -> Option<Vec<ListView>> {
    box_of_the_lists().lock().ok()?.take()
}

/// Asks the server for the collections and the playlists of one library.
///
/// **The caller must wait for the write of the list before it calls this.** A
/// question that goes with the write gives the lists of the moment before it,
/// and the screen then shows the state that came before the key of the user.
/// A measurement of 2026-08-11 showed that fault: the view held "[2 items]"
/// after the key `X` took one media out.
pub async fn ask(api: &ApiClient, library: &str) {
    let collections = crate::api::libraries::get_lists::get_all_collections(api, library).await;
    let playlists = crate::api::libraries::get_lists::get_all_playlists(api, library).await;

    match (collections, playlists) {
        (Ok(collections), Ok(playlists)) => {
            keep(crate::api::utils::collect_lists::collect_lists(
                &collections,
                &playlists,
            ));
        }
        (collections, playlists) => log::warn!(
            "[lists] the server did not give the lists again: {:?} {:?}",
            collections.err(),
            playlists.err()
        ),
    }
}

/// Forgets the answer. A test starts from a known condition.
pub fn forget() {
    if let Ok(mut place) = box_of_the_lists().lock() {
        *place = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::utils::collect_lists::ListKind;

    fn one(name: &str) -> ListView {
        ListView {
            id: "a-list".to_string(),
            kind: ListKind::Playlist,
            name: name.to_string(),
            description: String::new(),
            entries: Vec::new(),
        }
    }

    fn guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }

    /// The render takes the answer one time. A second frame must not write the
    /// same lists again.
    #[test]
    fn the_answer_comes_one_time() {
        let _guard = guard();
        forget();

        assert!(take().is_none());

        keep(vec![one("A Test Playlist")]);

        let lists = take().expect("the answer must come");
        assert_eq!(lists.len(), 1);

        assert!(take().is_none());
    }

    fn with_the_identity(id: &str, name: &str) -> ListView {
        ListView {
            id: id.to_string(),
            ..one(name)
        }
    }

    /// **The line of the user holds a list, and not a number of a line.** A
    /// second program of the account removed the collection of the line 0, and
    /// the user of the line 1 then stood in a playlist that they never opened:
    /// the key `X` of that user took a media out of it. See T-165.
    #[test]
    fn the_line_of_the_lists_holds_a_list_and_not_a_number() {
        let before = vec![
            with_the_identity("a-collection", "A Test Collection"),
            with_the_identity("a-playlist", "A Test Playlist"),
            with_the_identity("z-playlist", "Z Second Playlist"),
        ];

        // The list of the line stays: the line follows that list to its new
        // place.
        let after = vec![before[1].clone(), before[2].clone()];

        assert_eq!(
            what_the_line_of_the_lists_holds(Some("a-playlist"), &after),
            TheLineOfTheLists::TheSameList(0)
        );
        assert_eq!(
            what_the_line_of_the_lists_holds(Some("z-playlist"), &after),
            TheLineOfTheLists::TheSameList(1)
        );

        // Nothing changed: the line stays where it stands.
        assert_eq!(
            what_the_line_of_the_lists_holds(Some("a-playlist"), &before),
            TheLineOfTheLists::TheSameList(1)
        );

        // The list of the line went away.
        assert_eq!(
            what_the_line_of_the_lists_holds(Some("a-collection"), &after),
            TheLineOfTheLists::ThatListWentAway
        );

        // Every list went away.
        assert_eq!(
            what_the_line_of_the_lists_holds(Some("a-playlist"), &[]),
            TheLineOfTheLists::ThatListWentAway
        );

        // The line held no list at all.
        assert_eq!(
            what_the_line_of_the_lists_holds(None, &after),
            TheLineOfTheLists::NoLine
        );
    }

    /// The text names the list that went away, and it promises no key that the
    /// view does not hold (T-118 and T-143). See T-165.
    #[test]
    fn the_text_names_the_list_that_went_away() {
        let text = the_text_of_the_list_that_went_away(ListKind::Collection, "A Test Collection");

        assert!(text.contains("A Test Collection"), "{}", text);
        assert!(text.contains("collection"), "{}", text);
        assert!(text.contains("j and k"), "{}", text);

        let text =
            the_text_of_the_media_of_a_list_that_went_away(ListKind::Playlist, "A Test Playlist");

        assert!(text.contains("A Test Playlist"), "{}", text);
        assert!(text.contains("playlist"), "{}", text);
        assert!(text.contains("collections and the playlists"), "{}", text);
    }
}
