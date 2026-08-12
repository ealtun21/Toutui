use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub id: String,
    pub user_id: String,
    pub library_item_id: String,
    pub episode_id: Value,
    pub media_item_id: String,
    pub media_item_type: String,
    /// **A number of this answer can come as a text.** See `a_number` and
    /// T-130.
    #[serde(deserialize_with = "a_number", default)]
    pub duration: f64,
    #[serde(deserialize_with = "a_number", default)]
    pub progress: f64,
    #[serde(deserialize_with = "a_number", default)]
    pub current_time: f64,
    pub is_finished: bool,
    pub hide_from_continue_listening: bool,
    pub ebook_location: Value,
    /// **The place of the reader is a fraction of the book, and not a whole
    /// number.** The field held `i64`, therefore the answer of a media whose
    /// value is 0.35 did not read at all and the program lost the position of
    /// that media. See T-127.
    #[serde(deserialize_with = "a_number", default)]
    pub ebook_progress: f64,
    pub last_update: i64,
    pub started_at: i64,
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
