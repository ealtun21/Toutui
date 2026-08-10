use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// Get all books or podcasts from a library
/// https://api.audiobookshelf.org/#get-a-library-39-s-items

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub results: Option<Vec<LibraryItem>>,
    pub total: Option<i64>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub filter_by: Option<String>,
    pub media_type: Option<String>,
    pub minified: Option<bool>,
    pub collapseseries: Option<bool>,
    pub include: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
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
    pub media: Option<Media>,
    pub num_files: Option<i64>,
    pub size: Option<i64>,
    pub collapsed_series: Option<CollapsedSeries>,
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
    pub ebook_file_format: Option<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: Option<String>,
    pub title_ignore_prefix: Option<String>,
    pub subtitle: Option<Value>,
    pub author_name: Option<String>,
    pub author: Option<String>,
    pub narrator_name: Option<String>,
    pub series_name: Option<String>,
    pub genres: Option<Vec<String>>,
    pub published_year: Option<String>,
    pub published_date: Option<Value>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub isbn: Option<Value>,
    pub asin: Option<String>,
    pub language: Option<Value>,
    pub explicit: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollapsedSeries {
    pub id: Option<String>,
    pub name: Option<String>,
    pub name_ignore_prefix: Option<String>,
    pub num_books: Option<i64>,
}

/// The number of items in one request.
///
/// The old code used `limit=0`, and that value tells the server to send every
/// item in one answer. A library with 10000 books then makes a very large
/// answer. An Audiobookshelf contributor gives this advice in upstream issue
/// 35.
pub const PAGE_SIZE: i64 = 500;

/// The largest number of requests for one library.
///
/// The value stops an endless loop if a server always gives a full page. With
/// 500 items in a page, this value permits 250000 items.
const MAX_PAGES: i64 = 500;

/// Tells if the application must ask for one more page.
///
/// The function stops when the last page was not full, because a page that is
/// not full is the last page. It also stops when the application has all the
/// items that the server reports.
pub fn wants_more_pages(collected: usize, total: Option<i64>, last_page: usize) -> bool {
    if last_page < PAGE_SIZE as usize {
        return false;
    }

    match total {
        Some(total) if total >= 0 => collected < total as usize,
        _ => true,
    }
}

/// Gets all books or all podcasts of one library.
///
/// The function asks for one page at a time. Therefore no answer of the server
/// is very large. The function gives all the items together, thus the code
/// that calls it does not change.
pub async fn get_all_books(client: &ApiClient, id_selected_lib: &str) -> Result<Root, ApiError> {
    let mut all: Vec<LibraryItem> = Vec::new();
    let mut root = Root::default();

    for page in 0..MAX_PAGES {
        let answer: Root = client
            .get_json(&format!(
                "/api/libraries/{}/items?limit={}&page={}",
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A page that is full, and the server has more items.
    #[test]
    fn a_full_page_with_more_items_wants_another_page() {
        assert!(wants_more_pages(500, Some(2056), 500));
    }

    /// A page that is not full is the last page.
    #[test]
    fn a_page_that_is_not_full_is_the_last_page() {
        assert!(!wants_more_pages(556, Some(2056), 56));
    }

    /// The application has every item that the server reports.
    #[test]
    fn the_last_item_stops_the_loop() {
        assert!(!wants_more_pages(2056, Some(2056), 500));
    }

    /// A library that has fewer items than one page needs one request.
    #[test]
    fn a_small_library_needs_one_request() {
        assert!(!wants_more_pages(12, Some(12), 12));
    }

    /// A library with no item needs one request.
    #[test]
    fn an_empty_library_needs_one_request() {
        assert!(!wants_more_pages(0, Some(0), 0));
    }

    /// The server gave no total. The loop continues while the pages are full.
    #[test]
    fn no_total_continues_while_the_pages_are_full() {
        assert!(wants_more_pages(500, None, 500));
        assert!(!wants_more_pages(700, None, 200));
    }

    /// A total that is not valid must not stop the loop early.
    #[test]
    fn a_total_that_is_not_valid_continues() {
        assert!(wants_more_pages(500, Some(-1), 500));
    }
}
