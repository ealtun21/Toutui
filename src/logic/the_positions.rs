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
/// The row of a book names the media and no episode. **A row that names an
/// episode belongs to that episode**, and not to the media of the line:
/// `episodeId` of such a row holds a text, and the answer here is `None`.
///
/// The row is the row of `GET /api/me/progress/:id`, therefore the collectors
/// of the lists of the screen read it as they read that answer: the values of
/// the screen are the same values.
///
/// The function is pure, therefore a test needs no server.
pub fn the_position_of_a_media<'a>(rows: &'a [Root], id: &str) -> Option<&'a Root> {
    rows.iter()
        .find(|row| row.library_item_id == id && row.episode_id.is_null())
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
        let row = the_position_of_a_media(&rows, "a-book").expect("the row of that book");

        assert_eq!(row.progress, 0.5);
        assert!(!row.is_finished);
        assert_eq!(row.current_time, 900.0);
    }

    /// A media that no row names has no position, and the caller then asks the
    /// server for it.
    #[test]
    fn a_media_of_no_row_gives_nothing() {
        assert!(the_position_of_a_media(&rows(), "a-book-that-never-played").is_none());
    }

    /// **A row of an episode belongs to that episode.** The line of the Home
    /// view of a podcast names the media, and the position of one episode is
    /// not the position of that line.
    #[test]
    fn a_row_of_an_episode_is_not_the_position_of_the_media() {
        assert!(the_position_of_a_media(&rows(), "a-podcast").is_none());
    }
}
