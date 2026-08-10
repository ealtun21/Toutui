use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// Get a PersonalizedView's Personalized View  for podcast(allow to have continue linstening)
/// https://api.audiobookshelf.org/#get-a-library-39-s-personalized-view

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub id: Option<String>,
    pub label: String,
    pub label_string_key: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub entities: Option<Vec<Entity>>,
    pub total: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: Option<String>,
    pub ino: Option<String>,
    pub old_library_item_id: Option<Value>,
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
    pub media: Option<Media>,
    pub num_files: Option<i64>,
    pub size: Option<i64>,
    pub recent_episode: Option<RecentEpisode>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: Option<String>,
    pub metadata: Option<Metadata>,
    pub cover_path: Option<String>,
    pub tags: Option<Vec<Value>>,
    pub num_episodes: Option<i64>,
    pub auto_download_episodes: Option<bool>,
    pub auto_download_schedule: Option<String>,
    pub last_episode_check: Option<i64>,
    pub max_episodes_to_keep: Option<i64>,
    pub max_new_episodes_to_download: Option<i64>,
    pub size: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub genres: Option<Vec<String>>,
    pub feed_url: Option<String>,
    pub image_url: Option<String>,
    pub itunes_page_url: Option<String>,
    pub itunes_id: Option<Value>,
    pub itunes_artist_id: Option<String>,
    pub explicit: Option<bool>,
    pub language: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub title_ignore_prefix: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentEpisode {
    pub library_item_id: Option<String>,
    pub podcast_id: Option<String>,
    pub id: Option<String>,
    pub old_episode_id: Option<Value>,
    pub index: Option<Value>,
    pub season: Option<String>,
    pub episode: Option<String>,
    pub episode_type: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub enclosure: Option<Enclosure>,
    pub guid: Option<String>,
    pub pub_date: Option<String>,
    pub chapters: Option<Vec<Chapter>>,
    pub audio_file: Option<AudioFile>,
    pub published_at: Option<i64>,
    pub added_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enclosure {
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub length: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub title: Option<String>,
    pub id: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioFile {
    pub index: Option<i64>,
    pub ino: Option<String>,
    pub added_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub track_num_from_meta: Option<Value>,
    pub disc_num_from_meta: Option<Value>,
    pub track_num_from_filename: Option<Value>,
    pub disc_num_from_filename: Option<Value>,
    pub manually_verified: Option<bool>,
    pub exclude: Option<bool>,
    pub error: Option<Value>,
    pub format: Option<String>,
    pub duration: Option<f64>,
    pub bit_rate: Option<i64>,
    pub language: Option<Value>,
    pub codec: Option<String>,
    pub time_base: Option<String>,
    pub channels: Option<i64>,
    pub channel_layout: Option<String>,
    pub embedded_cover_art: Option<Value>,
    pub mime_type: Option<String>,
}

/// Gets the podcast episodes that the user continues to listen to.
///
/// The function keeps the shelf that has the label `Continue Listening` only.
pub async fn get_continue_listening_pod(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<Vec<Root>, ApiError> {
    let libraries: Vec<Root> = client
        .get_json(&format!("/api/libraries/{}/personalized", id_selected_lib))
        .await?;

    // The shelf of "Continue Listening" carries a name for the screen and an
    // identity. The old code compared the name, and a name is a text for a
    // person: a server that gives it in a different language would give this
    // program an empty Home view, with no error at all. The identity
    // `continue-listening` does not change. See T-24.
    //
    // The name stays as a second way, for a server that gives no identity.
    let continue_listening: Vec<Root> = libraries
        .into_iter()
        .filter(is_the_shelf_of_continue_listening)
        .collect();

    Ok(continue_listening)
}

/// Tells if a shelf of the personalized view is "Continue Listening".
///
/// The identity comes first, because it is the same on every server. See T-24.
pub fn is_the_shelf_of_continue_listening(shelf: &Root) -> bool {
    match shelf.id.as_deref() {
        Some(id) => id == "continue-listening",
        None => shelf.label == "Continue Listening",
    }
}
