use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

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

#[cfg(test)]
mod tests_of_the_shelf {
    use super::{is_the_shelf_of_continue_listening, Root};

    fn shelf(id: Option<&str>, label: &str) -> Root {
        Root {
            id: id.map(|value| value.to_string()),
            label: label.to_string(),
            entities: None,
        }
    }

    #[test]
    fn the_identity_names_the_shelf() {
        assert!(is_the_shelf_of_continue_listening(&shelf(
            Some("continue-listening"),
            "Continue Listening"
        )));
        assert!(!is_the_shelf_of_continue_listening(&shelf(
            Some("recently-added"),
            "Recently Added"
        )));
    }

    /// A server in a different language gives a different name. The identity
    /// stays the same, therefore the Home view still holds its media. The old
    /// code gave an empty view here, and it gave no error. See T-24.
    #[test]
    fn a_name_in_a_different_language_changes_nothing() {
        assert!(is_the_shelf_of_continue_listening(&shelf(
            Some("continue-listening"),
            "Continuer l'écoute"
        )));
    }

    #[test]
    fn a_shelf_with_no_identity_uses_the_name() {
        assert!(is_the_shelf_of_continue_listening(&shelf(
            None,
            "Continue Listening"
        )));
        assert!(!is_the_shelf_of_continue_listening(&shelf(
            None,
            "Recently Added"
        )));
    }
}
