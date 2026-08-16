//! The position of every media of the account, in one answer. See T-127.
//!
//! **The start asked one request for each media of the Home view.**
//! `GET /api/me/progress/:id` gives the position of one media, and the shelves
//! of a user hold 29 of them: the measurement of 2026-08-12 with a server of
//! 500 milliseconds gave **2.1 seconds** of a start of 3.8 for those requests,
//! eight at a time.
//!
//! `GET /api/me` holds `mediaProgress` for every media of the account, and the
//! program asks that endpoint for the permissions of the account already
//! (T-110). One request therefore gives every position, and the start of a
//! library of every size costs the same.
//!
//! **A media that this answer does not name keeps its own request.** The rows
//! of a podcast hold the episode beside the media, and no measurement says that
//! the row of the media alone answers for every shelf: the program asks for
//! those media as it did before, and it asks for no other.

use crate::api::me::get_media_progress::Root;
use std::collections::BTreeMap;

/// Reads the row of one media out of the answer of `GET /api/me`.
///
/// The row of a book names the media and no episode, therefore a caller with no
/// episode takes the row that names none: **a row that names an episode belongs
/// to that episode**, and not to the media of the line.
///
/// **A line of the Home view of a library of podcasts is one episode** (T-226),
/// and the identity of the item names every episode of one podcast (T-223).
/// Such a caller gives the episode, and the answer is then the row of that
/// episode alone. A line that took the row of the item took the place of a
/// different episode, and a line that took no row at all held no place of the
/// user: the two episodes of `Arthur Gordon Pym` of the sandbox stood at 80
/// percent and at 10 percent, and the Home view said nothing of either of them
/// (T-228).
///
/// The row is the row of `GET /api/me/progress/:id`, therefore the collectors
/// of the lists of the screen read it as they read that answer: the values of
/// the screen are the same values.
///
/// The function is pure, therefore a test needs no server.
pub fn the_place_of_a_media<'a>(
    rows: &'a [Root],
    id: &str,
    episode_id: Option<&str>,
) -> Option<&'a Root> {
    rows.iter().find(|row| {
        row.library_item_id == id
            && match episode_id {
                Some(episode) => row.episode_id.as_str() == Some(episode),
                None => row.episode_id.is_null(),
            }
    })
}

/// The place of the user of every media of the account, keyed by
/// `crate::logic::live::the_key_of_the_media`. See T-241.
///
/// **A view that holds no list of the places of its own reads this box.** The
/// Library view and the view of the search each name a book of the library, and
/// neither of them asked the server for the place of that book: the panel of
/// such a line said the author and the year alone, while the panel of that same
/// book of the Home view of that same frame said `Progress: 38%, 5h left, Not
/// finished`.
///
/// The row holds the three values of `App::book_progress_cnt_list` and of
/// `the_places_of_the_queue`: the percent of the user, the mark of the end, and
/// the place of the user in seconds. **The day of the start of the media stands
/// after the three of them** (T-328): the panel 5 of the design names that day,
/// and the answer of the account is the one road to it. A reader of the box
/// takes the value of its own place, therefore a row of three values and a row
/// of four give the same three values to every reader that stood before T-328.
fn box_of_the_places() -> &'static std::sync::Mutex<BTreeMap<String, Vec<String>>> {
    static PLACES: std::sync::OnceLock<std::sync::Mutex<BTreeMap<String, Vec<String>>>> =
        std::sync::OnceLock::new();
    PLACES.get_or_init(|| std::sync::Mutex::new(BTreeMap::new()))
}

/// Writes the place of the user of every media of the account. See T-241.
///
/// **The list takes the place of the list that came before it**: `App::new`
/// writes it, and the key `R` makes a new application (T-185). A start that
/// read no place of the account therefore empties the box, and no value of a
/// program that stood before this one stays on the screen.
pub fn keep_the_places(places: BTreeMap<String, Vec<String>>) {
    if let Ok(mut slot) = box_of_the_places().lock() {
        *slot = places;
    }
}

/// Gives the place of the user of one media of the account. See T-241.
///
/// The render calls this at each frame for the line that the user selected,
/// therefore it clones one row and not the whole list. A media that the answer
/// of the account did not name gives nothing at all, and the panel of that line
/// then says the words of a media that never played.
pub fn the_place_of(key: &str) -> Option<Vec<String>> {
    match box_of_the_places().lock() {
        Ok(places) => places.get(key).cloned(),
        Err(_) => None,
    }
}

/// Makes the box of the places out of the answer of `GET /api/me`. See T-241.
///
/// **The answer of the account holds every media that this account played**
/// (T-127), therefore one request gives the place of every line of every view,
/// and a library of 2056 items costs no request at all.
///
/// The key names the episode after the item, as every key of a place does: two
/// episodes of one podcast hold the identity of that podcast (T-223).
pub async fn the_places_of_the_account(rows: &[Root]) -> BTreeMap<String, Vec<String>> {
    use crate::api::utils::collect_get_media_progress::{
        collect_current_time_prg, collect_is_finished_book, collect_progress_percentage_book,
        the_day_of_the_start,
    };

    let mut places = BTreeMap::new();

    for row in rows {
        let key = crate::logic::live::the_key_of_the_media(
            row.library_item_id.as_str(),
            row.episode_id.as_str(),
        );

        places.insert(
            key,
            vec![
                collect_progress_percentage_book(row).await,
                collect_is_finished_book(row).await,
                collect_current_time_prg(row).await.to_string(),
                the_day_of_the_start(row),
            ],
        );
    }

    places
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rows() -> Vec<Root> {
        serde_json::from_value(json!([
            {
                "id": "one",
                "userId": "a-user",
                "libraryItemId": "a-book",
                "episodeId": null,
                "mediaItemId": "a-media",
                "mediaItemType": "book",
                "duration": 1800.0,
                "progress": 0.5,
                "currentTime": 900.0,
                "isFinished": false,
                "hideFromContinueListening": false,
                "ebookLocation": null,
                "ebookProgress": 0,
                "lastUpdate": 1,
                "startedAt": 1,
                "finishedAt": null
            },
            {
                "id": "two",
                "userId": "a-user",
                "libraryItemId": "a-podcast",
                "episodeId": "an-episode",
                "mediaItemId": "an-episode",
                "mediaItemType": "podcastEpisode",
                "duration": 1764.0,
                "progress": 1.0,
                "currentTime": 1764.0,
                "isFinished": true,
                "hideFromContinueListening": false,
                "ebookLocation": null,
                "ebookProgress": 0,
                "lastUpdate": 1,
                "startedAt": 1,
                "finishedAt": null
            }
        ]))
        .expect("the answer of the server reads")
    }

    /// The row of a book gives the three values of the line of the Home view.
    #[test]
    fn the_answer_holds_the_position_of_a_book() {
        let rows = rows();
        let row = the_place_of_a_media(&rows, "a-book", None).expect("the row of that book");

        assert_eq!(row.progress, 0.5);
        assert!(!row.is_finished);
        assert_eq!(row.current_time, 900.0);
    }

    /// A media that no row names has no position, and the caller then asks the
    /// server for it.
    #[test]
    fn a_media_of_no_row_gives_nothing() {
        assert!(the_place_of_a_media(&rows(), "a-book-that-never-played", None).is_none());
    }

    /// **A row of an episode belongs to that episode.** A caller with no
    /// episode asks for the media, and the position of one episode is not the
    /// position of that media.
    #[test]
    fn a_row_of_an_episode_is_not_the_position_of_the_media() {
        assert!(the_place_of_a_media(&rows(), "a-podcast", None).is_none());
    }

    /// **The line of the Home view of a library of podcasts is one episode**
    /// (T-226 and T-228). That line gives the episode, and the answer is the row
    /// of that episode.
    #[test]
    fn the_answer_holds_the_place_of_an_episode() {
        let rows = rows();
        let row = the_place_of_a_media(&rows, "a-podcast", Some("an-episode"))
            .expect("the row of that episode");

        assert_eq!(row.current_time, 1764.0);
        assert!(row.is_finished);
    }

    /// An episode of no row of the answer takes no row of another episode of
    /// the same podcast. See T-228.
    #[test]
    fn an_episode_of_no_row_takes_no_row_of_its_neighbour() {
        assert!(the_place_of_a_media(&rows(), "a-podcast", Some("a-second-episode")).is_none());
    }
}
