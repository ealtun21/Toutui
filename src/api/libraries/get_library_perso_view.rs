use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::Value;
use serde::Deserialize;
use serde::Serialize;

/// Get a PersonalizedView's Personalized View for book (allow to have continue linstening)
/// https://api.audiobookshelf.org/#get-a-library-39-s-personalized-view

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub id: Option<String>,
    pub label: String,
    pub entities: Option<Vec<Entity>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: Option<String>,
    pub library_id: Option<String>,
    pub folder_id: Option<String>,
    pub path: Option<String>,
    pub media: Option<Media>,
    pub name: Option<String>,
    #[serde(default)]
    pub books: Option<Vec<Book>>,
    pub in_progress: Option<bool>,
    pub has_active_book: Option<bool>,
    pub hide_from_continue_listening: Option<bool>,
    pub book_in_progress_last_update: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub metadata: Option<Metadata>,
    pub cover_path: Option<String>,
    pub tags: Option<Vec<Value>>,
    pub num_tracks: Option<i64>,
    pub num_audio_files: Option<i64>,
    pub num_chapters: Option<i64>,
    pub duration: Option<f64>,
    pub size: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: Option<String>,
    pub title_ignore_prefix: Option<String>,
    pub author_name: Option<String>,
    pub narrator_name: Option<String>,
    pub series_name: Option<String>,
    pub genres: Option<Vec<String>>,
    pub published_year: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub asin: Option<String>,
    pub explicit: Option<bool>,
    pub series: Option<Series>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub id: Option<String>,
    pub name: Option<String>,
    pub sequence: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: Option<String>,
    pub ino: Option<String>,
    pub library_id: Option<String>,
    pub folder_id: Option<String>,
    pub path: Option<String>,
    pub rel_path: Option<String>,
    pub is_file: Option<bool>,
    pub mtime_ms: Option<i64>,
    pub ctime_ms: Option<i64>,
    pub birthtime_ms: Option<i64>,
    pub added_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub is_missing: Option<bool>,
    pub is_invalid: Option<bool>,
    pub media_type: Option<String>,
    pub num_files: Option<i64>,
    pub size: Option<i64>,
    pub series_sequence: Option<String>,
}

/// Gets the books that the user continues to listen to.
///
/// The function keeps the shelf that has the label `Continue Listening` only.
pub async fn get_continue_listening(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<Vec<Root>, ApiError> {
    let libraries: Vec<Root> = client
        .get_json(&format!("/api/libraries/{}/personalized", id_selected_lib))
        .await?;

    // Filter libraries to keep only those with label "Continue Listening"
    let continue_listening: Vec<Root> = libraries
        .into_iter()
        .filter(|lib| lib.label == "Continue Listening")
        .collect();

    Ok(continue_listening)
}

