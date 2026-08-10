//! The series of a library.
//!
//! Audiobookshelf puts a book in a series, and it gives the number of the book
//! in the series. This module gets that list.
//!
//! The endpoint is `GET /api/libraries/:id/series`. A measurement on
//! 2026-08-10 shows an important difference from the endpoint of the items:
//! `limit=0` gives an empty list, and not every series. Therefore this module
//! always asks for a page of a known size.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use crate::api::libraries::get_all_books::{wants_more_pages, LibraryItem, PAGE_SIZE};
use serde::{Deserialize, Serialize};

/// The answer of `GET /api/libraries/:id/series`.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesRoot {
    pub results: Option<Vec<Series>>,
    pub total: Option<i64>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

/// One series of a library.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    /// The books of the series. The server gives them in the sequence of the
    /// series.
    pub books: Option<Vec<LibraryItem>>,
}

/// The largest number of requests for one library.
///
/// The value stops an endless loop if a server always gives a full page.
const MAX_PAGES: i64 = 500;

/// Gets all the series of one library.
///
/// The function asks for one page at a time, in the same way as
/// [`crate::api::libraries::get_all_books::get_all_books`]. The parameter
/// `sort=name` gives the same sequence at each start.
pub async fn get_all_series(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<SeriesRoot, ApiError> {
    let mut all: Vec<Series> = Vec::new();
    let mut root = SeriesRoot::default();

    for page in 0..MAX_PAGES {
        let answer: SeriesRoot = client
            .get_json(&format!(
                "/api/libraries/{}/series?limit={}&page={}&sort=name",
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
    root.limit = Some(PAGE_SIZE);
    root.page = None;

    Ok(root)
}
