use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// Get a Library Item, used for collect podact info (allow in particular to retrieve all podcast episode id)
/// This endpoint retrieves a library item, allow in particular to retrieve all podcast episode id.
/// https://api.audiobookshelf.org/#get-a-library-item

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
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
    pub scan_version: Option<Value>,
    pub is_missing: Option<bool>,
    pub is_invalid: Option<bool>,
    pub media_type: Option<String>,
    pub media: Option<Media>,
    pub library_files: Option<Vec<LibraryFile>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: Option<String>,
    pub library_item_id: Option<String>,
    pub metadata: Option<Metadata>,
    pub cover_path: Option<String>,
    pub tags: Option<Vec<Value>>,
    pub episodes: Option<Vec<Episode>>,
    pub auto_download_episodes: Option<bool>,
    pub auto_download_schedule: Option<String>,
    pub last_episode_check: Option<i64>,
    pub max_episodes_to_keep: Option<i64>,
    pub max_new_episodes_to_download: Option<i64>,
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
    pub itunes_artist_id: Option<Value>,
    pub explicit: Option<bool>,
    pub language: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
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
    pub chapters: Option<Vec<Value>>,
    pub audio_file: Option<AudioFile>,
    pub published_at: Option<i64>,
    pub added_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enclosure {
    pub url: Option<String>,
    pub length: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioFile {
    pub path: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFile {
    pub file_name: Option<String>,
    pub file_path: Option<String>,
}

/// Gets one library item with its podcast episodes.
///
/// See <https://api.audiobookshelf.org/#get-a-library-item>.
pub async fn get_pod_ep(client: &ApiClient, id: &str) -> Result<Root, ApiError> {
    let mut root: Root = client.get_json(&format!("/api/items/{}", id)).await?;

    keep_the_episodes_with_an_identity(&mut root);

    Ok(root)
}

/// Keeps the episodes that hold an identity.
///
/// The identity of an episode is the road to every function of its line: the
/// place of the user, the queue, and the playback each name the episode in a
/// path of the server. A row of the answer with no `id`, or with an `id` of no
/// character, gives the program no such road: `collect_ids_pod_ep` wrote the
/// text "N/A" for it, the key `l` then sent `POST /api/items/:id/play/N/A`,
/// and the program said that the server does not have an item that the server
/// has (T-388). A line that promises a function that the program does not
/// have belongs to no view (the rule of T-183 and of T-386), therefore such a
/// row takes a WARN of the log and no line.
///
/// The lists of the view stand one against the other by the number of the
/// line (T-24), therefore the removal stands here, before every collector,
/// and each collector then reads the same rows.
fn keep_the_episodes_with_an_identity(root: &mut Root) {
    let Some(episodes) = root
        .media
        .as_mut()
        .and_then(|media| media.episodes.as_mut())
    else {
        return;
    };

    episodes.retain(|episode| {
        let has_an_identity = episode
            .id
            .as_deref()
            .is_some_and(|id| !id.trim().is_empty());

        if !has_an_identity {
            log::warn!(
                "The answer of the server holds the episode \"{}\" with no identity. The program cannot play it, therefore the line goes away.",
                episode.title.as_deref().unwrap_or("")
            );
        }

        has_an_identity
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_podcast_of(episodes: Vec<Episode>) -> Root {
        Root {
            media: Some(Media {
                episodes: Some(episodes),
                ..Media::default()
            }),
            ..Root::default()
        }
    }

    fn an_episode_of(id: Option<&str>, title: &str) -> Episode {
        Episode {
            id: id.map(str::to_string),
            title: Some(title.to_string()),
            ..Episode::default()
        }
    }

    /// An episode of the server with no identity takes no line. See T-388.
    #[test]
    fn an_episode_with_no_identity_takes_no_line() {
        let mut root = a_podcast_of(vec![
            an_episode_of(None, "Chapter 00"),
            an_episode_of(Some("482f0136"), "Chapter 01"),
        ]);

        keep_the_episodes_with_an_identity(&mut root);

        let episodes = root.media.unwrap().episodes.unwrap();
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].title.as_deref(), Some("Chapter 01"));
    }

    /// An identity of no character is no identity. See T-388: such an id gave
    /// the path `/api/items/:id/play/` with an empty last segment.
    #[test]
    fn an_episode_of_an_identity_of_no_character_takes_no_line() {
        let mut root = a_podcast_of(vec![
            an_episode_of(Some(""), "Chapter 00"),
            an_episode_of(Some("   "), "Chapter 01"),
            an_episode_of(Some("ff28a3b0"), "Chapter 02"),
        ]);

        keep_the_episodes_with_an_identity(&mut root);

        let episodes = root.media.unwrap().episodes.unwrap();
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].title.as_deref(), Some("Chapter 02"));
    }

    /// The rows with an identity keep their lines and their sequence.
    #[test]
    fn the_episodes_with_an_identity_keep_their_lines() {
        let mut root = a_podcast_of(vec![
            an_episode_of(Some("one"), "Chapter 00"),
            an_episode_of(Some("two"), "Chapter 01"),
        ]);

        keep_the_episodes_with_an_identity(&mut root);

        let episodes = root.media.unwrap().episodes.unwrap();
        assert_eq!(episodes.len(), 2);
        assert_eq!(episodes[0].id.as_deref(), Some("one"));
        assert_eq!(episodes[1].id.as_deref(), Some("two"));
    }
}
