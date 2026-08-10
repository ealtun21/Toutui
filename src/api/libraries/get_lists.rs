//! The collections and the playlists of a library.
//!
//! A collection holds books, and every user of the server sees it. A playlist
//! belongs to one user, and it can hold books or episodes of a podcast.
//!
//! The two endpoints are `GET /api/libraries/:id/collections` and
//! `GET /api/libraries/:id/playlists`. Both accept a limit and a page, in the
//! same way as the endpoint of the series.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use crate::api::libraries::get_all_books::{wants_more_pages, LibraryItem, PAGE_SIZE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The answer of `GET /api/libraries/:id/collections`.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRoot {
    pub results: Option<Vec<Collection>>,
    pub total: Option<i64>,
}

/// One collection of a library.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub books: Option<Vec<LibraryItem>>,
}

/// The answer of `GET /api/libraries/:id/playlists`.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistRoot {
    pub results: Option<Vec<Playlist>>,
    pub total: Option<i64>,
}

/// One playlist of a user.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub items: Option<Vec<PlaylistItem>>,
}

/// One entry of a playlist.
///
/// The entry names a library item. It also names an episode when the item is a
/// podcast.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub library_item_id: Option<String>,
    pub library_item: Option<LibraryItem>,
    pub episode_id: Option<String>,
    /// The episode, when the item is a podcast. The application reads the
    /// title and the length from it.
    pub episode: Option<Value>,
}

/// The largest number of requests for one library.
const MAX_PAGES: i64 = 500;

/// Gets all the collections of one library.
pub async fn get_all_collections(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<CollectionRoot, ApiError> {
    let mut all: Vec<Collection> = Vec::new();
    let mut root = CollectionRoot::default();

    for page in 0..MAX_PAGES {
        let answer: CollectionRoot = client
            .get_json(&format!(
                "/api/libraries/{}/collections?limit={}&page={}",
                id_selected_lib, PAGE_SIZE, page
            ))
            .await?;

        let items = answer.results.clone().unwrap_or_default();
        let count = items.len();

        all.extend(items);
        root = answer;

        if !wants_more_pages(all.len(), root.total, count) {
            break;
        }
    }

    root.results = Some(all);

    Ok(root)
}

/// Gets all the playlists of one library.
pub async fn get_all_playlists(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<PlaylistRoot, ApiError> {
    let mut all: Vec<Playlist> = Vec::new();
    let mut root = PlaylistRoot::default();

    for page in 0..MAX_PAGES {
        let answer: PlaylistRoot = client
            .get_json(&format!(
                "/api/libraries/{}/playlists?limit={}&page={}",
                id_selected_lib, PAGE_SIZE, page
            ))
            .await?;

        let items = answer.results.clone().unwrap_or_default();
        let count = items.len();

        all.extend(items);
        root = answer;

        if !wants_more_pages(all.len(), root.total, count) {
            break;
        }
    }

    root.results = Some(all);

    Ok(root)
}
