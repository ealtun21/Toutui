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

/// One library of the server.
///
/// **A field that the program does not read must not stop the program.** The
/// old code asked for every field of the answer of Audiobookshelf 2.36.0, and
/// one field fewer stopped the whole program: a measurement of 2026-08-14 with
/// `docs/harness/another_body_of_the_libraries.py` took `icon` out of the first
/// library, and the program said `Toutui stops: it cannot read the lists of the
/// server.` The same measurement of `settings.autoScanCronExpression` gave the
/// same answer. **Neither field reaches one line of this program**, and a
/// server of another version can hold neither.
///
/// **Three fields stay**: the id, the name, and the media type. The row of the
/// account of the database holds the name and the id (T-173), and the media
/// type decides the views of a library. A body that holds no one of the three
/// is not the answer of this endpoint, and the program says so. See T-176.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub id: String,
    pub name: String,
    pub media_type: String,
    /// The program reads the first folder for a new podcast. A library of no
    /// folder holds no new podcast, and `src/app.rs` says that sentence.
    #[serde(default)]
    pub folders: Vec<Folder>,
    #[serde(default)]
    pub display_order: i64,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub last_scan: Option<i64>,
    #[serde(default)]
    pub last_scan_version: Option<String>,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub last_update: i64,
}

/// One folder of a library. The program reads the id and the path of the first
/// folder for a new podcast, and it reads no other field. See T-176.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub full_path: String,
    #[serde(default)]
    pub library_id: String,
    #[serde(default)]
    pub added_at: i64,
}

/// The settings of a library.
///
/// **No line of this program reads one field of this structure.** It stays for
/// the shape of the answer, and every field of it takes a default: a server
/// that holds one field fewer, or one field more, changes nothing here.
/// See T-176.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
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
    pub mark_as_finished_percent_complete: Value,
    pub mark_as_finished_time_remaining: i64,
    pub podcast_search_region: Option<String>,
}

/// Gets all libraries of the server. A library holds books or podcasts.
///
/// See <https://api.audiobookshelf.org/#get-all-libraries>.
pub async fn get_all_libraries(client: &ApiClient) -> Result<Root, ApiError> {
    client.get_json("/api/libraries").await
}
