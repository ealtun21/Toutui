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
    pub duration: f64,
    pub progress: f64,
    pub current_time: f64,
    pub is_finished: bool,
    pub hide_from_continue_listening: bool,
    pub ebook_location: Value,
    /// **The place of the reader is a fraction of the book, and not a whole
    /// number.** The field held `i64`, therefore the answer of a media whose
    /// value is 0.35 did not read at all and the program lost the position of
    /// that media. See T-127.
    pub ebook_progress: f64,
    pub last_update: i64,
    pub started_at: i64,
    pub finished_at: Value,
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
