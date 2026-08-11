//! The media that a collection or a playlist holds. See T-84.
//!
//! The program read the collections and the playlists, and it played them. It
//! **changed** none of them: a user who wanted a book in a playlist opened the
//! web page of the server.
//!
//! **The two lists take two different requests**, and the measurement of
//! 2026-08-11 against an Audiobookshelf 2.36.0 gave this:
//!
//! | Request | Answer |
//! |---|---|
//! | `POST /api/collections/:id/book` with `{"id":"<the item>"}` | `200`, and the whole collection |
//! | `POST /api/collections/:id/book` a second time | **`400`, "Book already in collection"** |
//! | `DELETE /api/collections/:id/book/:itemId` | `200` |
//! | `POST /api/playlists/:id/item` with `{"libraryItemId":"…"}` | `200`, and the whole playlist |
//! | `POST /api/playlists/:id/item` a second time | **`400`, "Item already in playlist"** |
//! | `DELETE /api/playlists/:id/item/:itemId` | `200` |
//!
//! A playlist takes an episode of a podcast too: the body then holds
//! `episodeId`, and the address of the removal holds that identity after the
//! identity of the item.
//!
//! **A collection holds books only.** The server refuses an episode, therefore
//! the program offers the collections of a library of books only.
//!
//! **The program makes a list too. See T-88.** The measurement of 2026-08-11
//! against the same server gave this:
//!
//! | Request | Answer |
//! |---|---|
//! | `POST /api/collections` with `books` of one item | `200`, and the whole collection |
//! | `POST /api/collections` with **no** `books` | **`400`, "Invalid collection data. No books"** |
//! | `POST /api/playlists` with **no** `items` | `200` |
//! | Either, with a name of no letter | **`400`, "Invalid … data"** |
//! | Either, with a name that a list holds already | `200`, **and a second list of that name** |
//!
//! **A new collection therefore needs a media**, and the key `m` of the program
//! holds one. The server takes two lists of one name, and a user cannot tell
//! them apart: the program refuses that name before the request.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use crate::api::utils::collect_lists::{ListKind, ListView};
use serde_json::Value;

/// Puts one medium in a collection or in a playlist.
///
/// The function gives `Ok(false)` when the list holds that medium already: the
/// server answers `400` for that condition, and it is not a fault of the
/// program. Every other fault comes back as an error.
pub async fn put_in_the_list(
    client: &ApiClient,
    kind: ListKind,
    list_id: &str,
    item_id: &str,
    episode_id: Option<&str>,
) -> Result<bool, ApiError> {
    let (path, body) = match kind {
        ListKind::Collection => (
            format!("/api/collections/{}/book", list_id),
            serde_json::json!({ "id": item_id }),
        ),
        ListKind::Playlist => (
            format!("/api/playlists/{}/item", list_id),
            match episode_id {
                Some(episode) => {
                    serde_json::json!({ "libraryItemId": item_id, "episodeId": episode })
                }
                None => serde_json::json!({ "libraryItemId": item_id }),
            },
        ),
    };

    match client.post_json::<Value, Value>(&path, &body).await {
        Ok(_) => Ok(true),
        // The server answers 400 for a medium that stands in the list already.
        Err(ApiError::Server(400)) => Ok(false),
        Err(error) => Err(error),
    }
}

/// Takes one medium out of a collection or of a playlist.
pub async fn take_out_of_the_list(
    client: &ApiClient,
    kind: ListKind,
    list_id: &str,
    item_id: &str,
    episode_id: Option<&str>,
) -> Result<(), ApiError> {
    let path = match kind {
        ListKind::Collection => format!("/api/collections/{}/book/{}", list_id, item_id),
        ListKind::Playlist => match episode_id {
            Some(episode) => format!("/api/playlists/{}/item/{}/{}", list_id, item_id, episode),
            None => format!("/api/playlists/{}/item/{}", list_id, item_id),
        },
    };

    client.delete_no_content(&path).await
}

/// Gives the body of the request that makes a list. See T-88.
///
/// The function is pure, therefore a test needs no server. **A collection
/// holds books, and the server refuses one with no book**: the body of a
/// collection therefore always names the media.
pub fn the_body_of_the_new_list(
    kind: ListKind,
    library_id: &str,
    name: &str,
    item_id: &str,
    episode_id: Option<&str>,
) -> Value {
    match kind {
        ListKind::Collection => serde_json::json!({
            "libraryId": library_id,
            "name": name,
            "books": [item_id],
        }),
        ListKind::Playlist => {
            let entry = match episode_id {
                Some(episode) => {
                    serde_json::json!({ "libraryItemId": item_id, "episodeId": episode })
                }
                None => serde_json::json!({ "libraryItemId": item_id }),
            };

            serde_json::json!({
                "libraryId": library_id,
                "name": name,
                "items": [entry],
            })
        }
    }
}

/// Tells if a list of that kind holds that name already. See T-88.
///
/// The server takes two lists of one name, and it gives each of them its own
/// identity. The user then sees two lines that say the same words, and no key
/// tells them apart. Therefore the program refuses the name.
///
/// The comparison ignores the case and the spaces at the two ends, because a
/// name that differs in those two ways only is the same name for a user.
pub fn a_list_holds_that_name(lists: &[ListView], kind: ListKind, name: &str) -> bool {
    let name = name.trim().to_lowercase();

    lists
        .iter()
        .any(|list| list.kind == kind && list.name.trim().to_lowercase() == name)
}

/// Makes a collection or a playlist, and it puts one medium in it. See T-88.
///
/// The function gives the identity of the new list.
pub async fn make_the_list(
    client: &ApiClient,
    kind: ListKind,
    library_id: &str,
    name: &str,
    item_id: &str,
    episode_id: Option<&str>,
) -> Result<String, ApiError> {
    let path = match kind {
        ListKind::Collection => "/api/collections",
        ListKind::Playlist => "/api/playlists",
    };

    let body = the_body_of_the_new_list(kind, library_id, name, item_id, episode_id);
    let answer: Value = client.post_json(path, &body).await?;

    Ok(answer
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string())
}

/// Gives the sentence of the new list for the user. See T-88.
pub fn the_sentence_of_the_new_list(kind: ListKind, name: &str, title: &str) -> String {
    format!(
        "The {} \"{}\" exists now, and it holds \"{}\".",
        kind.name().to_lowercase(),
        name,
        title
    )
}

/// Gives the sentence of a name that a list holds already. See T-88.
pub fn the_sentence_of_the_name_that_exists(kind: ListKind, name: &str) -> String {
    format!(
        "A {} of the name \"{}\" exists already. Give a different name.",
        kind.name().to_lowercase(),
        name
    )
}

/// Gives the sentence of the work for the user.
///
/// The function is pure, therefore a test needs no server.
pub fn the_sentence_of_the_work(kind: ListKind, name: &str, title: &str, came: bool) -> String {
    if came {
        return format!(
            "\"{}\" is in the {} \"{}\" now.",
            title,
            kind.name().to_lowercase(),
            name
        );
    }

    format!(
        "\"{}\" stands in the {} \"{}\" already.",
        title,
        kind.name().to_lowercase(),
        name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sentence_says_the_media_and_the_list() {
        assert_eq!(
            the_sentence_of_the_work(ListKind::Collection, "A Test Collection", "Alice", true),
            "\"Alice\" is in the collection \"A Test Collection\" now."
        );

        assert_eq!(
            the_sentence_of_the_work(ListKind::Playlist, "A Test Playlist", "Alice", false),
            "\"Alice\" stands in the playlist \"A Test Playlist\" already."
        );
    }

    /// The server refuses a collection with no book, therefore the body of a
    /// new collection names the media. See T-88.
    #[test]
    fn the_body_of_a_new_collection_holds_the_book() {
        let body = the_body_of_the_new_list(
            ListKind::Collection,
            "a-library",
            "A New Collection",
            "an-item",
            None,
        );

        assert_eq!(body["libraryId"], "a-library");
        assert_eq!(body["name"], "A New Collection");
        assert_eq!(body["books"], serde_json::json!(["an-item"]));
    }

    /// A playlist takes an episode of a podcast, and the entry then names that
    /// episode. See T-88.
    #[test]
    fn the_body_of_a_new_playlist_holds_the_episode() {
        let body = the_body_of_the_new_list(
            ListKind::Playlist,
            "a-library",
            "A New Playlist",
            "a-podcast",
            Some("an-episode"),
        );

        assert_eq!(
            body["items"],
            serde_json::json!([{ "libraryItemId": "a-podcast", "episodeId": "an-episode" }])
        );

        let of_a_book =
            the_body_of_the_new_list(ListKind::Playlist, "a-library", "A Name", "a-book", None);

        assert_eq!(
            of_a_book["items"],
            serde_json::json!([{ "libraryItemId": "a-book" }])
        );
    }

    /// The name of a new list must differ from the name of every list of that
    /// kind, and the case and the spaces of the two ends say nothing. A
    /// collection and a playlist of one name stay apart: the line of the screen
    /// names the kind. See T-88.
    #[test]
    fn a_name_that_a_list_holds_already() {
        let lists = vec![
            ListView {
                id: "one".to_string(),
                kind: ListKind::Collection,
                name: "A Test Collection".to_string(),
                description: String::new(),
                entries: Vec::new(),
            },
            ListView {
                id: "two".to_string(),
                kind: ListKind::Playlist,
                name: "A Test Playlist".to_string(),
                description: String::new(),
                entries: Vec::new(),
            },
        ];

        assert!(a_list_holds_that_name(
            &lists,
            ListKind::Collection,
            "A Test Collection"
        ));
        assert!(a_list_holds_that_name(
            &lists,
            ListKind::Collection,
            "  a test collection "
        ));

        // The kind is a part of the comparison.
        assert!(!a_list_holds_that_name(
            &lists,
            ListKind::Playlist,
            "A Test Collection"
        ));
        assert!(!a_list_holds_that_name(
            &lists,
            ListKind::Collection,
            "A Different Name"
        ));
    }
}
