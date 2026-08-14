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

/// The text of the view of the lists, while that view holds no line.
///
/// **A view must not give a reason that the program does not have** (T-91). The
/// three conditions of that item were the answer that holds no list and the
/// server that did not answer at the start, and **a fourth exists**: the server
/// answers, and the request of the collections and of the playlists came back
/// with a fault.
///
/// A measurement of 2026-08-14 gave the program a server that answers the
/// libraries, the shelves, and the items, and that fails
/// `GET /api/libraries/:id/collections` and `GET /api/libraries/:id/playlists`
/// with the status 500. The library `Books` of the sandbox holds one collection
/// and one playlist, and the view said
/// "This library has no collection and no playlist." **`is_offline` of `App`
/// holds the offline mode of the start** (T-25), therefore the words of the
/// offline mode never came. See T-169, and the same shape of T-168.
///
/// The sentence names what the server said, and it promises no key that this
/// view does not hold (T-118 and T-143).
///
/// The function is pure, therefore a test needs no server.
pub fn the_reason_of_no_list(is_offline: bool, what_the_server_said: Option<&str>) -> String {
    if let Some(fault) = what_the_server_said {
        return format!(
            "The server did not give the collections and the playlists: {}\n\
             Press h to go back.",
            fault
        );
    }

    if is_offline {
        return "The server gave no collection and no playlist: the server does not answer.\n\
                Press h to go back."
            .to_string();
    }

    "This library has no collection and no playlist.\nPress h to go back.".to_string()
}

/// The title of the view of the key `m`, while the library holds no list.
///
/// That view puts one media in a collection or in a playlist. **It must not ask
/// the user to make a list when the program does not know the lists of the
/// server**: the two keys `c` and `p` belong to a library that holds no list,
/// and not to a request that came back with a fault. See T-169 and T-91.
///
/// The function is pure, therefore a test needs no server.
pub fn the_title_of_no_list(is_offline: bool, what_the_server_said: Option<&str>) -> String {
    if let Some(fault) = what_the_server_said {
        return format!(
            "The server did not give the collections and the playlists: {}",
            fault
        );
    }

    if is_offline {
        return "The server does not answer. A collection and a playlist stand on the server."
            .to_string();
    }

    "This library holds no collection and no playlist. Press c or p to make one.".to_string()
}

/// The box of the request of the lists that came back with a fault. See T-169.
///
/// The box holds the library of that request and what the server said. **The
/// lists belong to one library**: a user who changes the library with the key
/// `S` must not read the fault of the library before it.
fn the_fault_that_waits() -> &'static Mutex<Option<(String, String)>> {
    static FAULT: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
    FAULT.get_or_init(|| Mutex::new(None))
}

/// Writes that the server did not give the lists of one library. The task of
/// the start and the task of a key both call this. See T-169.
pub fn keep_the_fault(library: &str, what_the_server_said: &str) {
    if let Ok(mut slot) = the_fault_that_waits().lock() {
        *slot = Some((library.to_string(), what_the_server_said.to_string()));
    }
}

/// Gives what the server said of the lists of this library, and `None` for a
/// library whose request holds no fault. See T-169.
pub fn the_fault_of(library: &str) -> Option<String> {
    match the_fault_that_waits().lock() {
        Ok(slot) => slot
            .as_ref()
            .filter(|(of_the_library, _)| of_the_library == library)
            .map(|(_, text)| text.clone()),
        Err(_) => None,
    }
}

/// Takes the fault of one library away. An answer that comes calls this. See
/// T-169.
pub fn forget_the_fault_of(library: &str) {
    if let Ok(mut slot) = the_fault_that_waits().lock() {
        if slot
            .as_ref()
            .is_some_and(|(of_the_library, _)| of_the_library == library)
        {
            *slot = None;
        }
    }
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
            // The answer of that library came. A fault of a request before it
            // is not the truth of this moment. See T-169.
            forget_the_fault_of(library);

            keep(crate::api::utils::collect_lists::collect_lists(
                &collections,
                &playlists,
            ));
        }
        (collections, playlists) => {
            // **The view must say why it holds no line** (T-91 and T-169). The
            // program said this fault to the log alone before, and the view
            // then said "This library has no collection and no playlist."
            //
            // The message row says nothing here: every caller of this function
            // comes after a write of the user, and that write says its own
            // sentence already (T-164).
            let what_the_server_said = match (&collections, &playlists) {
                (Err(error), _) => error.to_string(),
                (_, Err(error)) => error.to_string(),
                _ => String::new(),
            };

            log::warn!(
                "[lists] the server did not give the lists again: {:?} {:?}",
                collections.err(),
                playlists.err()
            );

            keep_the_fault(library, &what_the_server_said);
        }
    }
}

/// Forgets the answer and the fault. A test starts from a known condition.
pub fn forget() {
    if let Ok(mut place) = box_of_the_lists().lock() {
        *place = None;
    }

    if let Ok(mut slot) = the_fault_that_waits().lock() {
        *slot = None;
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
    /// same lists again. The box of the fault holds the request that came back
    /// with a fault, and it holds it for one library alone (T-169).
    ///
    /// **The parts of this test stay in one function**: two test functions of
    /// one module fight for the boxes of the process (T-144 and T-157).
    #[test]
    fn the_answer_comes_one_time() {
        let _guard = guard();
        forget();

        assert!(take().is_none());

        keep(vec![one("A Test Playlist")]);

        let lists = take().expect("the answer must come");
        assert_eq!(lists.len(), 1);

        assert!(take().is_none());

        // The request of the lists of the library `Books` came back with a
        // fault. See T-169.
        forget();
        keep_the_fault("books", "The server reported a fault. Status 500.");

        assert_eq!(
            the_fault_of("books").as_deref(),
            Some("The server reported a fault. Status 500."),
            "the view of that library must read the fault of its own request"
        );

        // **A user who takes the key `S` to another library must not read the
        // fault of the library before it.**
        assert_eq!(the_fault_of("empty"), None);

        // A new request of that library takes the fault away.
        forget_the_fault_of("empty");
        assert!(
            the_fault_of("books").is_some(),
            "a request of another library keeps this fault"
        );
        forget_the_fault_of("books");
        assert_eq!(the_fault_of("books"), None);

        keep_the_fault("books", "a fault");
        forget();
        assert_eq!(the_fault_of("books"), None);
    }

    /// The view says why it holds no line, and it says a reason that the
    /// program has. See T-169 and T-91.
    #[test]
    fn the_view_says_why_it_holds_no_list() {
        // The answer of the server came, and that library holds no list.
        assert_eq!(
            the_reason_of_no_list(false, None),
            "This library has no collection and no playlist.\nPress h to go back."
        );

        // The server did not answer at the start of the program (T-25 and
        // T-91).
        assert!(the_reason_of_no_list(true, None).contains("does not answer"));

        // **The request came back with a fault** (T-169). The server answers,
        // therefore the words of the offline mode never come, and the view said
        // that a library of one collection and of one playlist holds none.
        let text = the_reason_of_no_list(false, Some("The server reported a fault. Status 500."));

        assert!(
            text.starts_with("The server did not give the collections and the playlists:"),
            "{}",
            text
        );
        assert!(text.contains("Status 500."), "{}", text);
        assert!(
            !text.contains("has no collection"),
            "the view must not say a reason that the program does not have: {}",
            text
        );

        // The fault of the request stands above the words of the offline mode
        // of the start: the program made that request, therefore it knows more
        // than the state of its start.
        assert!(the_reason_of_no_list(true, Some("a fault")).contains("a fault"));

        // The title of the view of the key `m` holds the same three
        // conditions, and it names the two keys that make a list for the
        // library that holds none.
        assert!(the_title_of_no_list(false, None).contains("Press c or p"));
        assert!(the_title_of_no_list(true, None).contains("does not answer"));

        let title = the_title_of_no_list(false, Some("The server reported a fault. Status 500."));

        assert!(
            title.starts_with("The server did not give the collections and the playlists:"),
            "{}",
            title
        );
        assert!(
            !title.contains("Press c or p"),
            "the title must not ask the user to make a list while the program \
             does not know the lists of the server: {}",
            title
        );
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
