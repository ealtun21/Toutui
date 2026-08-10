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

/// Gets the shelves of the Home view of a library of podcasts.
///
/// **A library of podcasts can give no shelf of Continue Listening.** A
/// measurement against an Audiobookshelf 2.36.0 on 2026-08-11 gives three
/// shelves for the library of podcasts of the sandbox: `newest-episodes`
/// (3 episodes), `recently-added` (1 podcast), and `listen-again`
/// (2 episodes). The program kept the shelf `continue-listening` only,
/// therefore the Home view of that library was empty and it said nothing.
/// See T-24.
pub async fn get_the_shelves_pod(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<Vec<Root>, ApiError> {
    let mut shelves: Vec<Root> = client
        .get_json(&format!("/api/libraries/{}/personalized", id_selected_lib))
        .await?;

    // The shelf of Continue Listening comes first, if the server gives one.
    let first = shelves.iter().position(is_the_shelf_of_continue_listening);

    if let Some(first) = first {
        let shelf = shelves.remove(first);
        shelves.insert(0, shelf);
    }

    Ok(shelves)
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
