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
//!
//! **The reader has a table of the disk from T-294**,
//! `pending_ebook_progress`. The box of the process above holds the two roads
//! where the program stops of its own will, and it holds no road at all for a
//! program that takes `SIGKILL` and for a machine that stops: that box goes
//! away with the process. A place that the server refused therefore takes a row
//! of the disk, and the start of the program after this one sends every row of
//! it. It is the rule of `pending_progress` of the audio playback (T-152 and
//! T-212), and the row holds the two fields of the request of a book alone.

use crate::api::client::ApiClient;
use crate::db::crud::{
    delete_pending_ebook_progress, get_pending_ebook_progress, insert_pending_ebook_progress,
    PendingEbookProgress,
};
use log::{error, info, warn};

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
pub async fn the_place_of_the_reader_goes_to_the_server(
    api: &ApiClient,
    username: &str,
    server: &str,
    handle_key: &str,
) {
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

                // **A place that the server took must leave the disk** (T-211).
                the_place_of_this_book_waits_no_more_on_the_disk(
                    username,
                    server,
                    &place.item_id,
                    handle_key,
                )
                .await;
            }

            Err(error) => {
                warn!(
                    "[{}] the server did not take the place of the book of the media {} ({}): {}. \
                     The place waits on the disk for the program after this one.",
                    handle_key, place.item_id, place.location, error
                );

                // **The box of the process goes away with the process** (T-294).
                the_place_of_this_book_waits_on_the_disk(username, server, &place, handle_key)
                    .await;
            }
        }
    }
}

/// Gives the moment of now, in milliseconds.
fn the_moment_of_now_in_milliseconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() as i64)
        .unwrap_or(0)
}

/// Writes the place of one book on the disk, for the program after this one.
/// See T-294.
///
/// **A place that reached no machine keeps no row of a box of the process
/// alone** (T-292 and T-293): a `SIGKILL` and a machine that stops each take
/// that box away. The audio playback holds the same rule in the table
/// `pending_progress` (T-212).
///
/// **The caller reads the answer of the write** (T-207): a disk that took no
/// row wrote nothing at all, and the words of that fault name the disk and the
/// media (T-211). The user reads no line of this fault: the road of the end
/// takes the screen away, and the road of the key `s` and of the key `h` says
/// the fault of the server already.
pub async fn the_place_of_this_book_waits_on_the_disk(
    username: &str,
    server: &str,
    place: &ThePlaceOfTheReader,
    handle_key: &str,
) {
    let row = PendingEbookProgress {
        id_item: place.item_id.clone(),
        location: place.location.clone(),
        fraction: place.fraction,
        updated_at: the_moment_of_now_in_milliseconds(),
    };

    let of_the_account = username.to_string();
    let of_the_server = server.to_string();

    let answer = crate::db::the_work_of_the_disk(move || {
        insert_pending_ebook_progress(&of_the_account, &of_the_server, &row)
    })
    .await;

    match answer {
        Some(Ok(())) => info!(
            "[{}] the place of the book of the media {} waits on the disk ({}).",
            handle_key, place.item_id, place.location
        ),

        Some(Err(error)) => error!(
            "[{}] the disk did not take the place of the book of the media {} ({}): {}. That \
             place goes away with this program.",
            handle_key, place.item_id, place.location, error
        ),

        None => error!(
            "[{}] the thread of the disk did not come back with the place of the book of the \
             media {}. That place goes away with this program.",
            handle_key, place.item_id
        ),
    }
}

/// Takes the place of one book off the disk. See T-294.
///
/// **A place that the server took must leave the disk** (T-211): a row that
/// stays gives the server the same place at the start of the program after this
/// one, and that place can then stand behind the place of a second client.
pub async fn the_place_of_this_book_waits_no_more_on_the_disk(
    username: &str,
    server: &str,
    item_id: &str,
    handle_key: &str,
) {
    let of_the_account = username.to_string();
    let of_the_server = server.to_string();
    let of_the_media = item_id.to_string();

    let answer = crate::db::the_work_of_the_disk(move || {
        delete_pending_ebook_progress(&of_the_account, &of_the_server, &of_the_media)
    })
    .await;

    match answer {
        Some(Ok(())) => {}

        Some(Err(error)) => error!(
            "[{}] the disk keeps the place of the book of the media {}, and the server holds that \
             place already: {}. The program after this one sends it again.",
            handle_key, item_id, error
        ),

        None => error!(
            "[{}] the thread of the disk did not come back with the removal of the place of the \
             book of the media {}. The program after this one sends it again.",
            handle_key, item_id
        ),
    }
}

/// Sends every place of a book that the disk kept, at the start of the program.
/// See T-294.
///
/// It gives the number of the places that the server took.
///
/// **A read of the disk that failed is not a disk with no place that waits**
/// (T-203), therefore that fault takes a line of the log and the places wait for
/// the program after this one.
///
/// **A place that the server refuses stops no other place** (T-293): each book
/// takes a request of its own, and a row of a book that failed stays on the disk.
///
/// **The program says no word of this send** (T-177): it runs before the first
/// frame, and it holds no key of the user.
pub async fn the_places_of_the_disk_go_to_the_server(
    api: &ApiClient,
    username: &str,
    server: &str,
) -> usize {
    let of_the_account = username.to_string();
    let of_the_server = server.to_string();

    let waiting = match crate::db::the_work_of_the_disk(move || {
        get_pending_ebook_progress(&of_the_account, &of_the_server)
    })
    .await
    {
        Some(Ok(waiting)) => waiting,

        Some(Err(error)) => {
            error!(
                "[reader] the program did not read the places of the books that wait: {}. Each of \
                 them waits for the next attempt.",
                error
            );

            return 0;
        }

        None => {
            error!(
                "[reader] the thread of the disk did not come back with the places of the books \
                 that wait. Each of them waits for the next attempt."
            );

            return 0;
        }
    };

    if waiting.is_empty() {
        return 0;
    }

    info!(
        "[reader] {} place(s) of a book wait for the server",
        waiting.len()
    );

    let mut sent = 0;

    for place in waiting {
        let body = serde_json::json!({
            "ebookLocation": place.location,
            "ebookProgress": place.fraction,
        });

        match api
            .patch_json(&format!("/api/me/progress/{}", place.id_item), &body)
            .await
        {
            Ok(()) => {
                info!(
                    "[reader] the place of the book of the media {} that waited on the disk went \
                     to the server ({}).",
                    place.id_item, place.location
                );

                sent += 1;

                the_place_of_this_book_waits_no_more_on_the_disk(
                    username,
                    server,
                    &place.id_item,
                    "reader",
                )
                .await;
            }

            Err(error) => warn!(
                "[reader] the server did not take the place of the book of the media {} ({}): {}. \
                 That place waits on the disk.",
                place.id_item, place.location, error
            ),
        }
    }

    sent
}
