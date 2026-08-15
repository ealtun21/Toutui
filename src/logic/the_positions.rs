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
