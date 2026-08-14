use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

// Get All Libraries (can be a podcast or book library (shelf))
// https://api.audiobookshelf.org/#get-all-libraries

/// The libraries of the server.
///
/// **The program reads each library of the answer apart** (T-191). The field
/// `libraries` is a list, therefore serde gave the fault of one row to the
/// whole answer: a measurement of 2026-08-14 with
/// `docs/harness/a_field_of_one_row_goes_away.py` took the `name` of the row 1
/// of five libraries away, and **the program did not start at all**. The four
/// other libraries of that server held every field.
///
/// `the_libraries_of_the_rows` reads the rows one at a time now, and a row
/// that this program cannot use takes a line of the log and no line of the
/// view.
#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Root {
    pub libraries: Vec<Library>,
}

impl<'de> Deserialize<'de> for Root {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        /// The rows of the answer. **The `Option` is the answer of a field of
        /// the value `null`**: `#[serde(default)]` holds for a field that is
        /// absent alone (T-183).
        #[derive(Deserialize)]
        struct TheRowsOfTheAnswer {
            #[serde(default)]
            libraries: Option<Vec<Value>>,
        }

        let rows = TheRowsOfTheAnswer::deserialize(deserializer)?;

        Ok(Root {
            libraries: the_libraries_of_the_rows(rows.libraries.unwrap_or_default()),
        })
    }
}

/// Reads each row of the list of the libraries apart.
///
/// **A library needs two values of the server, and it needs them both**: the
/// id is the address of every request of that library, and the media type
/// decides the views of it. A row with no id or with no media type is a
/// library that this program cannot use, therefore it takes a line of the log
/// and no line of the view — the rule of T-177 and of T-183.
///
/// **The name is a word for the user, and it is no address.** A library with
/// no name keeps its line, and that line holds the id of the library: the
/// program has that value, and a line of no character promises nothing to the
/// user (T-118). This is the road of `the_name_of_the_shelf` (T-190), and it
/// closes the words that T-176 left open.
pub fn the_libraries_of_the_rows(rows: Vec<Value>) -> Vec<Library> {
    let mut libraries = Vec::new();

    for row in rows {
        let mut library = match serde_json::from_value::<Library>(row) {
            Ok(library) => library,
            Err(error) => {
                warn!(
                    "[libraries] The program cannot read a library of the answer of \
                     the server: {}. The other libraries stay.",
                    error
                );
                continue;
            }
        };

        if library.id.trim().is_empty() {
            warn!(
                "[libraries] The answer of the server holds a library with no \
                 identity. That library has no address, therefore it belongs to no \
                 line of the view."
            );
            continue;
        }

        if library.media_type.trim().is_empty() {
            warn!(
                "[libraries] The answer of the server gives no media type to the \
                 library {}. The program does not know the views of that library, \
                 therefore it belongs to no line of the view.",
                library.id
            );
            continue;
        }

        // A folder with no address is no folder. The program reads the first
        // folder for a new podcast, and a library of no folder holds no new
        // podcast already. See T-191.
        library
            .folders
            .retain(|folder| !folder.id.trim().is_empty() && !folder.full_path.trim().is_empty());

        if library.name.trim().is_empty() {
            warn!(
                "[libraries] The answer of the server gives no name to the library \
                 {}. The line of that library holds its identity.",
                library.id
            );
            library.name = library.id.clone();
        }

        libraries.push(library);
    }

    libraries
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
/// type decides the views of a library. **A row that holds no one of the three
/// takes no line of the view**, and the log names it: see
/// `the_libraries_of_the_rows` and T-191. T-176 gave the fault of one row to
/// the whole answer, and that fault took every library of the server away.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    /// The address of every request of this library. A row with no id belongs
    /// to no line of the view. See T-191.
    #[serde(default)]
    pub id: String,
    /// The name of the library for the user. A library with no name keeps its
    /// line, and that line holds the id. See T-191.
    #[serde(default)]
    pub name: String,
    /// `book` or `podcast`. A row with no media type belongs to no line of the
    /// view. See T-191.
    #[serde(default)]
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
    /// **The field takes a default** (T-191): a folder of one library must not
    /// take the line of that library away. `the_folders_of_the_rows` gives no
    /// folder for a row with no id and no path, and the program then says that
    /// the library holds no folder.
    #[serde(default)]
    pub id: String,
    #[serde(default)]
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

#[cfg(test)]
mod tests_of_the_rows {
    use super::the_libraries_of_the_rows;
    use serde_json::json;

    /// **One row of the list must not take the other rows away.** See T-191.
    #[test]
    fn a_row_that_the_program_cannot_read_keeps_the_other_libraries() {
        let rows = vec![
            json!({"id": "a", "name": "Books", "mediaType": "book"}),
            json!("a row of no library"),
            json!({"id": "b", "name": "Podcasts", "mediaType": "podcast"}),
        ];

        let libraries = the_libraries_of_the_rows(rows);

        assert_eq!(libraries.len(), 2);
        assert_eq!(libraries[0].id, "a");
        assert_eq!(libraries[1].id, "b");
    }

    /// A library with no name keeps its line, and the line holds the id.
    /// See T-190 and T-191.
    #[test]
    fn the_line_of_a_library_with_no_name_holds_the_id() {
        let libraries = the_libraries_of_the_rows(vec![
            json!({"id": "a", "mediaType": "book"}),
            json!({"id": "b", "name": "   ", "mediaType": "podcast"}),
        ]);

        assert_eq!(libraries.len(), 2);
        assert_eq!(libraries[0].name, "a");
        assert_eq!(libraries[1].name, "b");
    }

    /// A library with no id has no address, and a library with no media type
    /// gives no view. Neither belongs to a line. See T-191.
    #[test]
    fn a_library_that_the_program_cannot_use_belongs_to_no_line() {
        let libraries = the_libraries_of_the_rows(vec![
            json!({"name": "No Address", "mediaType": "book"}),
            json!({"id": "  ", "name": "No Address", "mediaType": "book"}),
            json!({"id": "b", "name": "No View"}),
            json!({"id": "c", "name": "No View", "mediaType": "  "}),
            json!({"id": "d", "name": "A Library", "mediaType": "book"}),
        ]);

        assert_eq!(libraries.len(), 1);
        assert_eq!(libraries[0].id, "d");
    }

    /// A folder with no address is no folder, and the library keeps its line.
    /// See T-191.
    #[test]
    fn a_folder_with_no_address_takes_no_library_away() {
        let libraries = the_libraries_of_the_rows(vec![json!({
            "id": "a", "name": "Books", "mediaType": "book",
            "folders": [{"fullPath": "/a/path"}, {"id": "f", "fullPath": "/a/second/path"}]
        })]);

        assert_eq!(libraries.len(), 1);
        assert_eq!(libraries[0].folders.len(), 1);
        assert_eq!(libraries[0].folders[0].id, "f");
    }

    /// A body with no list of the libraries gives no library, and it gives no
    /// fault of a decode: the program says that the server gave no library
    /// (T-173). See T-191.
    #[test]
    fn a_body_with_no_list_gives_no_library() {
        use super::Root;

        let root: Root = serde_json::from_str(r#"{"libraries": null}"#).unwrap();
        assert!(root.libraries.is_empty());

        let root: Root = serde_json::from_str("{}").unwrap();
        assert!(root.libraries.is_empty());
    }
}
