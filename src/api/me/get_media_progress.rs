use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::Value;
use serde::Deserialize;
use serde::Serialize;


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
    pub ebook_progress: i64,
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

