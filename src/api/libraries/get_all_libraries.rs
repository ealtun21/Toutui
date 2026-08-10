use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// Get All Libraries (can be a podcast or book library (shelf))
/// https://api.audiobookshelf.org/#get-all-libraries

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub libraries: Vec<Library>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub id: String,
    pub name: String,
    pub folders: Vec<Folder>,
    pub display_order: i64,
    pub icon: String,
    pub media_type: String,
    pub provider: String,
    pub settings: Settings,
    pub last_scan: Option<i64>,
    pub last_scan_version: Option<String>,
    pub created_at: i64,
    pub last_update: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub full_path: String,
    pub library_id: String,
    pub added_at: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub cover_aspect_ratio: i64,
    pub disable_watcher: bool,
    pub auto_scan_cron_expression: Value,
    pub skip_matching_media_with_asin: Option<bool>,
    pub skip_matching_media_with_isbn: Option<bool>,
    pub audiobooks_only: Option<bool>,
    pub epubs_allow_scripted_content: Option<bool>,
    pub hide_single_book_series: Option<bool>,
    pub only_show_later_books_in_continue_series: Option<bool>,
    pub metadata_precedence: Option<Vec<String>>,
    #[serde(default)]
    pub mark_as_finished_percent_complete: Value,
    #[serde(default)]
    pub mark_as_finished_time_remaining: i64,
    pub podcast_search_region: Option<String>,
}

/// Gets all libraries of the server. A library holds books or podcasts.
///
/// See <https://api.audiobookshelf.org/#get-all-libraries>.
pub async fn get_all_libraries(client: &ApiClient) -> Result<Root, ApiError> {
    client.get_json("/api/libraries").await
}
