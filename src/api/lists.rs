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

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use crate::api::utils::collect_lists::ListKind;
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
}
