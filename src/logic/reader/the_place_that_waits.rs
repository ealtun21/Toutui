//! The place of the reader that no machine holds. See T-292.
//!
//! **The reader has no table of the disk.** The audio playback keeps the place
//! of the user in the table `pending_progress` (T-212) and in the row of the
//! listening session (T-201), therefore a program that stops gives that place
//! to the server and a program that dies leaves it for the program after it.
//! The reader keeps its place in the `App` alone: the view of the reader goes
//! away with the process, and the place of the user goes away with the view.
//!
//! The key `Q` stops the program from the view of the reader too — the footer
//! of that view names it — and the terminal that went away takes that same
//! road (T-271). Neither of the two roads holds the `App` of the user: the road
//! of the key `Q` stops the process with `clean_exit` before a `tokio::spawn`
//! of the send can run, and the watch of the terminal stands in a task that
//! never saw an `App` at all.
//!
//! Therefore the loop of the application writes the place of the reader in the
//! box of this module at each turn, and the two roads of the end read that box
//! and send it before the process goes away.

use crate::api::client::ApiClient;
use log::{info, warn};

/// The place of the reader for the request of the server.
///
/// The value holds the identity of the media, because the road of the end
/// reads this box with no `App` and no reader at all.
#[derive(Clone, Debug, PartialEq)]
pub struct ThePlaceOfTheReader {
    /// The identity of the media of the book.
    pub item_id: String,
    /// The place of the user, in the form of the field `ebookLocation`.
    pub location: String,
    /// The part of the book that the user read, of 0 to 1.
    pub fraction: f64,
}

/// The box of the process that holds the place of each book. See T-293.
///
/// **The box holds one place for each media, and not one place for the whole
/// program.** The user can read a second book after a send that the server
/// refused, and the place of one book says nothing of another one.
type TheBoxOfThePlaces = std::sync::Arc<std::sync::Mutex<Vec<ThePlaceOfTheReader>>>;

/// Gives the box that the loop of the application writes.
fn the_box() -> TheBoxOfThePlaces {
    static PLACES: std::sync::OnceLock<TheBoxOfThePlaces> = std::sync::OnceLock::new();
    std::sync::Arc::clone(
        PLACES.get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(Vec::new()))),
    )
}

/// Says which place of a book no machine holds.
///
/// The place takes the place of the book of the same media, and it leaves the
/// place of every other book where it stands. See T-293.
pub fn say_the_place_that_waits(place: ThePlaceOfTheReader) {
    let the_box = the_box();

    let Ok(mut the_places) = the_box.lock() else {
        return;
    };

    match the_places
        .iter_mut()
        .find(|that_place| that_place.item_id == place.item_id)
    {
        Some(that_place) => *that_place = place,
        None => the_places.push(place),
    }
}

/// Takes the place of one book out of the box.
///
/// **A place that no machine holds must not go away with a reader that goes
/// away** (T-293), therefore the box loses a place on two roads alone: the
/// server takes that place, or the reader of that same book says that the
/// server holds it already.
pub fn the_place_of_this_book_waits_no_more(item_id: &str) {
    if let Ok(mut the_places) = the_box().lock() {
        the_places.retain(|place| place.item_id != item_id);
    }
}

/// Gives the places that wait, and it leaves the box as it stands.
pub fn the_places_that_wait() -> Vec<ThePlaceOfTheReader> {
    the_box()
        .lock()
        .map(|places| places.clone())
        .unwrap_or_default()
}

/// The loop of the application says the place of the reader of this turn.
/// See T-293.
///
/// `place` is the place of the reader that no machine holds, and
/// `the_book_of_the_server` names the media of a reader whose place the server
/// holds already.
///
/// **A reader that went away is neither of the two, and the box then keeps
/// every place that it holds.** The key `h` gives the view before the reader
/// back, and the key `e` of a second book writes `self.reader = None` at once:
/// a loop that emptied the box for a reader that went away lost the place of
/// the book before it, and the user read that book for nothing.
pub fn the_loop_says_the_place_of_the_reader(
    place: Option<ThePlaceOfTheReader>,
    the_book_of_the_server: Option<&str>,
) {
    match place {
        Some(place) => say_the_place_that_waits(place),

        None => {
            if let Some(item_id) = the_book_of_the_server {
                the_place_of_this_book_waits_no_more(item_id);
            }
        }
    }
}

/// Gives the place of one book that waits, and it leaves the box as it stands.
pub fn the_place_of_this_book_that_waits(item_id: &str) -> Option<ThePlaceOfTheReader> {
    the_places_that_wait()
        .into_iter()
        .find(|place| place.item_id == item_id)
}

/// Sends the place of each book that no machine holds, for a program that
/// stops. See T-292 and T-293.
///
/// **The caller awaits this function.** The two roads of the end give the
/// process to the machine at the line after it, therefore a `tokio::spawn` of
/// this request would never run.
///
/// **A book whose place the server refuses stops no other book** (T-293): the
/// user can read more than one book in one program, and each place of them
/// takes a request of its own.
///
/// **The program says no word of this send** (T-177): the screen of the user
/// goes away with the process on both roads, therefore the log holds the
/// answer of the server.
pub async fn the_place_of_the_reader_goes_to_the_server(api: &ApiClient, handle_key: &str) {
    for place in the_places_that_wait() {
        let body = serde_json::json!({
            "ebookLocation": place.location,
            "ebookProgress": place.fraction,
        });

        match api
            .patch_json(&format!("/api/me/progress/{}", place.item_id), &body)
            .await
        {
            Ok(()) => {
                info!(
                    "[{}] the place of the book of the media {} went to the server before the \
                     program stopped ({}).",
                    handle_key, place.item_id, place.location
                );

                // A second road of the end must not send the same place again.
                the_place_of_this_book_waits_no_more(&place.item_id);
            }

            Err(error) => {
                warn!(
                    "[{}] the server did not take the place of the book of the media {} ({}): {}. \
                     The reader holds no table of the disk, therefore that place goes away with \
                     this program.",
                    handle_key, place.item_id, place.location, error
                );
            }
        }
    }
}
