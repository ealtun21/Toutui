use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// Get a PersonalizedView's Personalized View for book (allow to have continue linstening)
/// https://api.audiobookshelf.org/#get-a-library-39-s-personalized-view

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub id: Option<String>,
    /// The name of the shelf for the user.
    ///
    /// **The field takes a default**, because a shelf that holds no `label`
    /// took every shelf of the Home view away: serde gives the fault of one
    /// row to the whole answer, and the answer of this path is the list of
    /// the shelves itself. A shelf with no name keeps its media, and
    /// `crate::logic::home_view::the_name_of_the_shelf` gives the name of its
    /// line. See T-190.
    #[serde(default)]
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

    /// The kind of the file of the ebook of the media: `epub`, `pdf`, and so
    /// on. A media of no ebook holds no value.
    ///
    /// **The answer of this path holds this field and the struct had no field
    /// of it at all** (T-326), therefore the panel of a media of the Home view
    /// could say no word of the ebook of it. The name of the server is
    /// `ebookFormat`, and `rename_all = "camelCase"` reads this name as that
    /// one: see T-325 for the field of the same name of the answer of the
    /// items, which held the name `ebook_file_format` and gave `None` for
    /// every book of every server.
    pub ebook_format: Option<String>,
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

/// Gets the shelves of the Home view.
///
/// The program asked this endpoint before, and it kept the shelf
/// `continue-listening` only. The other five shelves came in the same answer,
/// and the program threw them away. The Home view shows every shelf now, and
/// the request does not change. See T-24.
///
/// A measurement against an Audiobookshelf 2.36.0 on 2026-08-11 gives six
/// shelves for a library of books: `continue-listening` (4 media),
/// `recently-added` (9 media), `recent-series` (2 series), `discover`
/// (2 media), `listen-again` (2 media), and `newest-authors` (4 authors).
///
/// The function gives every shelf. `crate::logic::home_view::group_home`
/// makes the lines, and it drops a shelf that gives no line: an author has no
/// media and no book, therefore the view can show nothing for it.
pub async fn get_the_shelves(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<Vec<Root>, ApiError> {
    let mut shelves: Vec<Root> = client
        .get_json(&format!("/api/libraries/{}/personalized", id_selected_lib))
        .await?;

    the_shelves_with_no_name(
        shelves
            .iter()
            .map(|shelf| (shelf.id.as_deref(), &shelf.label)),
    );

    // The shelf of Continue Listening comes first. The server puts it first
    // today, and this line makes it sure: that shelf holds the media that the
    // user started, and the user looks for it before every other shelf.
    let first = shelves.iter().position(is_the_shelf_of_continue_listening);

    if let Some(first) = first {
        let shelf = shelves.remove(first);
        shelves.insert(0, shelf);
    }

    Ok(shelves)
}

/// Writes one line of the log for each shelf that the server gave with no
/// name.
///
/// **The line of that shelf stays**, because the media of it reach every
/// request of the program: `crate::logic::home_view::the_name_of_the_shelf`
/// gives the name of the line. The log holds the one word of this fault of the
/// answer, and no view of the user says it — the rule of T-177. See T-190.
pub fn the_shelves_with_no_name<'a>(shelves: impl Iterator<Item = (Option<&'a str>, &'a String)>) {
    for (id, label) in shelves {
        if label.trim().is_empty() {
            warn!(
                "[home] The answer of the server holds a shelf with no name. \
                 The line of that shelf takes the name \"{}\".",
                crate::logic::home_view::the_name_of_the_shelf(id, label)
            );
        }
    }
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

    /// **One shelf with no `label` took every shelf away.** The answer of
    /// `GET /api/libraries/:id/personalized` is the list of the shelves
    /// itself, therefore serde gave the fault of that one row to the whole
    /// answer and the Home view held no media at all. See T-190.
    #[test]
    fn a_shelf_with_no_name_keeps_every_other_shelf() {
        let body = r#"[
            {"id": "continue-listening", "label": "Continue Listening",
             "entities": [{"id": "a", "media": {}}]},
            {"id": "recently-added", "entities": [{"id": "b", "media": {}}]},
            {"id": "discover", "label": "Discover",
             "entities": [{"id": "c", "media": {}}]}
        ]"#;

        let shelves: Vec<Root> = serde_json::from_str(body).unwrap();

        assert_eq!(shelves.len(), 3);
        assert_eq!(shelves[1].label, "");
        assert_eq!(shelves[2].label, "Discover");
    }

    /// The same answer of a library of podcasts. See T-190.
    #[test]
    fn a_shelf_of_podcasts_with_no_name_keeps_every_other_shelf() {
        use crate::api::libraries::get_library_perso_view_pod::Root as RootPod;

        let body = r#"[
            {"id": "newest-episodes", "entities": [{"id": "a"}]},
            {"id": "listen-again", "label": "Listen Again", "entities": []}
        ]"#;

        let shelves: Vec<RootPod> = serde_json::from_str(body).unwrap();

        assert_eq!(shelves.len(), 2);
        assert_eq!(shelves[0].label, "");
        assert_eq!(shelves[1].label, "Listen Again");
    }
}
