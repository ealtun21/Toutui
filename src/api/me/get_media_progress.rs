use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// The position of one media of the account.
///
/// **The program reads the media of the position, and every other field takes a
/// default.** This is the rule of T-176 for this answer. `Root` asked for every
/// field of Audiobookshelf 2.36.0, and `mediaItemId` and `mediaItemType` came
/// to `mediaProgress` with the version 2.5.0 alone: a server before that one
/// gave the log 20 lines of "missing field `mediaItemId`", and **the Home view
/// then held the position of no media at all** — no percent, no mark of a media
/// that is finished, and `Progress:  N/A%,   N/A` on the line of the media. The
/// program reads neither field. See T-177.
///
/// **`libraryItemId` is the one field that stays**: a row that names no media
/// belongs to no line of any view, therefore the program can say nothing of it
/// and it keeps no such row (`the_positions_of_the_answer` of
/// `src/api/me/permissions.rs`).
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub user_id: String,
    pub library_item_id: String,
    #[serde(default)]
    pub episode_id: Value,
    #[serde(default)]
    pub media_item_id: String,
    #[serde(default)]
    pub media_item_type: String,
    /// **A number of this answer can come as a text.** See `a_number` and
    /// T-130.
    #[serde(deserialize_with = "a_number", default)]
    pub duration: f64,
    #[serde(deserialize_with = "a_number", default)]
    pub progress: f64,
    #[serde(deserialize_with = "a_number", default)]
    pub current_time: f64,
    #[serde(default)]
    pub is_finished: bool,
    #[serde(default)]
    pub hide_from_continue_listening: bool,
    #[serde(default)]
    pub ebook_location: Value,
    /// **The place of the reader is a fraction of the book, and not a whole
    /// number.** The field held `i64`, therefore the answer of a media whose
    /// value is 0.35 did not read at all and the program lost the position of
    /// that media. See T-127.
    #[serde(deserialize_with = "a_number", default)]
    pub ebook_progress: f64,
    #[serde(default)]
    pub last_update: i64,
    #[serde(default)]
    pub started_at: i64,
    #[serde(default)]
    pub finished_at: Value,
}

/// Reads a number that the server gives as a number or as a text.
///
/// **The server keeps the form that a client gave it.** This program sent
/// `POST /api/session/:id/sync` with `"currentTime": "714"`, and `GET /api/me`
/// then answered `"currentTime": "714"` for that media: the row did not read at
/// all, and the Home view lost the position of a media that the server holds.
/// The program sends a number now (T-130), and this function reads the rows that
/// stand in the database of a server already. A client that is not this program
/// can write such a value too.
fn a_number<'de, D>(of: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(of)? {
        Value::Number(number) => Ok(number.as_f64().unwrap_or_default()),
        // A text of no number gives 0. The row of the media then reads, and the
        // program says "N/A" for that one value only.
        Value::String(text) => Ok(text.trim().parse::<f64>().unwrap_or_default()),
        // `null` of a value that the server did not write gives 0, as
        // `#[serde(default)]` does for a field that the answer does not hold.
        _ => Ok(0.0),
    }
}

/// Gets the listening progress of one book.
///
/// The server gives `404` if the user did not start the book. The caller then
/// shows an empty progress.
///
/// See <https://api.audiobookshelf.org/#get-a-media-progress>.
pub async fn get_book_progress(client: &ApiClient, book_id: &str) -> Result<Root, ApiError> {
    client
        .get_json(&format!("/api/me/progress/{}", book_id))
        .await
}

/// The path of the position of one media of the account.
///
/// An episode of a podcast holds its own position, and the path of that
/// position names the episode after the item. See T-182.
///
/// The function is pure, therefore a test needs no server.
pub fn the_path_of_the_place(item_id: &str, episode_id: Option<&str>) -> String {
    match episode_id {
        Some(episode) => format!("/api/me/progress/{}/{}", item_id, episode),
        None => format!("/api/me/progress/{}", item_id),
    }
}

/// Gets the position of one media of the account, and of one episode of a
/// podcast.
///
/// **The playback asks for this position when the answer of the session gave
/// no place** (T-182). The server gives `404` for a media that never played,
/// and the caller reads that status as the place 0.
pub async fn get_the_place_of_a_media(
    client: &ApiClient,
    item_id: &str,
    episode_id: Option<&str>,
) -> Result<Root, ApiError> {
    client
        .get_json(&the_path_of_the_place(item_id, episode_id))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The place of the reader inside a book is a fraction.**
    ///
    /// The field held `i64`. A measurement against Audiobookshelf 2.36.0 of
    /// 2026-08-12 gave `"ebookProgress": 0.8277488992014371` for a book that
    /// the user read, therefore the answer of that book did not read at all and
    /// the line of the Home view lost its position: the program showed "N/A"
    /// for a book of 92 percent. See T-127.
    /// **A number of the answer can be a text, and the row must read.** This
    /// program sent the position of an open session as a text, therefore
    /// `GET /api/me` answered `"currentTime": "714"` for a media that played:
    /// the row did not read, and the Home view said "N/A" for a book at the
    /// minute 11 of 30. See T-130.
    #[test]
    fn a_number_that_the_server_gives_as_a_text_reads() {
        let answer = serde_json::json!({
            "id": "a-row",
            "userId": "a-user",
            "libraryItemId": "a-book",
            "episodeId": null,
            "mediaItemId": "a-media",
            "mediaItemType": "book",
            "duration": 1800.0,
            "progress": "0.39666666666666667",
            "currentTime": "714",
            "isFinished": false,
            "hideFromContinueListening": false,
            "ebookLocation": null,
            "ebookProgress": "0.5",
            "lastUpdate": 1786317827954i64,
            "startedAt": 1786317827954i64,
            "finishedAt": null
        });

        let row: Root = serde_json::from_value(answer).expect("the answer of the server reads");

        assert_eq!(row.current_time, 714.0);
        assert_eq!(row.ebook_progress, 0.5);
        assert!((row.progress - 0.396_666_666_666_666_7).abs() < 0.000_001);
    }

    /// A text that holds no number gives 0, and the row still reads: one value
    /// of a row must not take the whole row away. See T-130 and T-41.
    #[test]
    fn a_text_of_no_number_keeps_the_row() {
        let answer = serde_json::json!({
            "id": "a-row",
            "userId": "a-user",
            "libraryItemId": "a-book",
            "episodeId": null,
            "mediaItemId": "a-media",
            "mediaItemType": "book",
            "duration": "",
            "progress": "not a number",
            "currentTime": null,
            "isFinished": true,
            "hideFromContinueListening": false,
            "ebookLocation": null,
            "ebookProgress": null,
            "lastUpdate": 1786317827954i64,
            "startedAt": 1786317827954i64,
            "finishedAt": null
        });

        let row: Root = serde_json::from_value(answer).expect("the answer of the server reads");

        assert_eq!(row.current_time, 0.0);
        assert_eq!(row.progress, 0.0);
        assert!(row.is_finished);
    }

    /// **A server of another version holds fewer fields, and the position of
    /// the media must read.** `mediaItemId` and `mediaItemType` came to
    /// `mediaProgress` with the version 2.5.0 of Audiobookshelf, and this
    /// program reads neither of them. A measurement of 2026-08-14 with
    /// `docs/harness/a_field_of_the_answer_goes_away.py` took those two fields
    /// out of `GET /api/me`: the 20 rows of the account each said
    /// "missing field `mediaItemId`" in the log, and **every position of every
    /// media went away** — the Home view showed no percent and no mark of a
    /// media that is finished, and the line of the media said
    /// `Progress:  N/A%,   N/A`. No word said why. See T-177 and T-176.
    #[test]
    fn the_position_of_a_server_of_another_version_reads() {
        let answer = serde_json::json!({
            "id": "a-row",
            "userId": "a-user",
            "libraryItemId": "a-book",
            "episodeId": null,
            "duration": 1800.0,
            "progress": 0.5,
            "currentTime": 900.0,
            "isFinished": false,
            "hideFromContinueListening": false,
            "ebookLocation": null,
            "ebookProgress": 0,
            "lastUpdate": 1786317827954i64,
            "startedAt": 1786317827954i64,
            "finishedAt": null
        });

        let row: Root = serde_json::from_value(answer).expect("the answer of the server reads");

        assert_eq!(row.library_item_id, "a-book");
        assert_eq!(row.current_time, 900.0);
        assert_eq!(row.media_item_type, "");
    }

    /// **The media of the position is the one field that must stand.** A row
    /// that names no media belongs to no line of any view, therefore the
    /// program can say nothing of it and it keeps no such row. This is the rule
    /// of T-176 for this answer: the program reads what it needs, and every
    /// other field takes a default.
    #[test]
    fn a_row_that_names_no_media_does_not_read() {
        let answer = serde_json::json!({
            "id": "a-row",
            "userId": "a-user",
            "progress": 0.5
        });

        let fault = serde_json::from_value::<Root>(answer)
            .expect_err("a row that names no media does not read");

        assert!(
            fault.to_string().contains("libraryItemId"),
            "the words of the fault name the field: {}",
            fault
        );
    }

    /// A row of a podcast of a server of another version keeps its episode,
    /// therefore `the_position_of_a_media` still leaves it to the episode. See
    /// T-177.
    #[test]
    fn the_episode_of_a_position_of_another_version_stays() {
        let answer = serde_json::json!({
            "id": "a-row",
            "userId": "a-user",
            "libraryItemId": "a-podcast",
            "episodeId": "an-episode",
            "progress": 1.0,
            "isFinished": true,
            "lastUpdate": 1i64
        });

        let row: Root = serde_json::from_value(answer).expect("the answer of the server reads");

        assert_eq!(row.episode_id, serde_json::json!("an-episode"));
        assert!(row.is_finished);
        assert_eq!(row.started_at, 0);
    }

    #[test]
    fn the_answer_of_a_book_that_the_user_read_reads() {
        let answer = serde_json::json!({
            "id": "a-row",
            "userId": "a-user",
            "libraryItemId": "a-book",
            "episodeId": null,
            "mediaItemId": "a-media",
            "mediaItemType": "book",
            "duration": 1800.0,
            "progress": 0.92,
            "currentTime": 1656.0,
            "isFinished": false,
            "hideFromContinueListening": false,
            "ebookLocation": "epubcfi(/6/6!/4/2/14/1:698)",
            "ebookProgress": 0.8277488992014371,
            "lastUpdate": 1786317827954i64,
            "startedAt": 1786317827954i64,
            "finishedAt": null
        });

        let row: Root = serde_json::from_value(answer).expect("the answer of the server reads");

        assert_eq!(row.progress, 0.92);
        assert_eq!(row.current_time, 1656.0);
        assert!(!row.is_finished);
    }
}
