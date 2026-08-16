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

/// The box of the process that holds the place of the reader.
type TheBoxOfThePlace = std::sync::Arc<std::sync::Mutex<Option<ThePlaceOfTheReader>>>;

/// Gives the box that the loop of the application writes.
fn the_box() -> TheBoxOfThePlace {
    static PLACE: std::sync::OnceLock<TheBoxOfThePlace> = std::sync::OnceLock::new();
    std::sync::Arc::clone(PLACE.get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(None))))
}

/// Says which place of the reader no machine holds.
///
/// `None` empties the box: a reader whose place the server took already, and a
/// program with no reader at all, leave no place behind them.
pub fn say_the_place_that_waits(place: Option<ThePlaceOfTheReader>) {
    if let Ok(mut the_place) = the_box().lock() {
        *the_place = place;
    }
}

/// Gives the place that waits, and it leaves the box as it stands.
pub fn the_place_that_waits() -> Option<ThePlaceOfTheReader> {
    the_box().lock().ok()?.clone()
}

/// Sends the place of the reader that no machine holds, for a program that
/// stops. See T-292.
///
/// **The caller awaits this function.** The two roads of the end give the
/// process to the machine at the line after it, therefore a `tokio::spawn` of
/// this request would never run.
///
/// **The program says no word of this send** (T-177): the screen of the user
/// goes away with the process on both roads, therefore the log holds the
/// answer of the server.
pub async fn the_place_of_the_reader_goes_to_the_server(api: &ApiClient, handle_key: &str) {
    let Some(place) = the_place_that_waits() else {
        return;
    };

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
                "[{}] the place of the book of the media {} went to the server before the program \
                 stopped ({}).",
                handle_key, place.item_id, place.location
            );

            // A second road of the end must not send the same place again.
            say_the_place_that_waits(None);
        }

        Err(error) => {
            warn!(
                "[{}] the server did not take the place of the book of the media {} ({}): {}. \
                 The reader holds no table of the disk, therefore that place goes away with this \
                 program.",
                handle_key, place.item_id, place.location, error
            );
        }
    }
}
