//! The search of the server. See T-24.
//!
//! The program looked in the titles of the items that it holds, with
//! `contains`. A user who wrote the name of an author therefore found nothing.
//!
//! `GET /api/libraries/:id/search?q=` gives six groups: `book`, `authors`,
//! `series`, `narrators`, `tags`, and `genres`. A measurement against an
//! Audiobookshelf 2.36.0 on 2026-08-11:
//!
//! - `q=carroll` gives one author, "Lewis Carroll", and no book. The program
//!   found nothing for that word before.
//! - `q=chronicles` gives three books and one series, and the group of the
//!   series carries the books of that series.
//!
//! The program takes the identity of every book of the answer, from the group
//! of the books and from the books of every series. The screen then shows the
//! items of the library that the server found.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;

/// The largest number of answers of one group.
///
/// A user reads a screen, and not a thousand lines. The server takes this
/// number as `limit`.
const LIMIT: usize = 50;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchRoot {
    #[serde(default)]
    pub book: Vec<BookMatch>,
    #[serde(default)]
    pub series: Vec<SeriesMatch>,
    #[serde(default)]
    pub authors: Vec<NamedMatch>,
    #[serde(default)]
    pub narrators: Vec<NamedMatch>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookMatch {
    #[serde(rename = "libraryItem")]
    pub library_item: Option<Item>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeriesMatch {
    #[serde(default)]
    pub books: Vec<Item>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamedMatch {
    pub name: Option<String>,
    /// The identity of an author. A narrator has none: the server holds a
    /// narrator as a name of the metadata, and not as a row of its own.
    #[serde(default)]
    pub id: Option<String>,
}

/// One name that the server found, with the filter that gives its books.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    pub name: String,
    /// The query for `get_all_books`, for example
    /// `&filter=authors.MzEyYzQyZmY…`.
    pub query: String,
}

/// The largest number of names that the program asks the server about.
///
/// Each name costs one request of the library. A user reads a screen, and three
/// names hold every measurement of the sandbox. See T-70.
const NAMES_TO_ASK: usize = 3;

/// Gives the names of an answer that can give books, with their filter.
///
/// **The group of the books of the server does not hold the name of an author.**
/// A measurement of 2026-08-11: `q=carroll` gives one author, "Lewis Carroll",
/// and **no book**. The screen of the program then held one line of the header and
/// no line of a book, and the book of that author stands in the library.
///
/// The filter of the server gives those books:
/// `?filter=authors.<the identity in base64>` gave "Alice in Wonderland", and
/// `?filter=narrators.<the name in base64>` gave the two books of a narrator.
///
/// An author comes before a narrator, because a user who writes a name looks for
/// the writer of the book more often. **A tag and a genre do not come**: the view
/// of the filter of T-60 holds them, and a genre of a large library gives some
/// hundred books.
///
/// The function is pure, therefore a test needs no server. See T-70.
pub fn the_names_to_ask(answer: &SearchRoot) -> Vec<Named> {
    let of_the_authors = answer.authors.iter().filter_map(|author| {
        let name = author.name.clone()?;
        let id = author.id.clone()?;

        if name.trim().is_empty() || id.trim().is_empty() {
            return None;
        }

        Some(Named {
            name,
            query: format!(
                "&filter=authors.{}",
                crate::logic::sort_filter::encode_base64(&id)
            ),
        })
    });

    let of_the_narrators = answer.narrators.iter().filter_map(|narrator| {
        let name = narrator.name.clone()?;

        if name.trim().is_empty() {
            return None;
        }

        Some(Named {
            query: format!(
                "&filter=narrators.{}",
                crate::logic::sort_filter::encode_base64(&name)
            ),
            name,
        })
    });

    of_the_authors
        .chain(of_the_narrators)
        .take(NAMES_TO_ASK)
        .collect()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub id: Option<String>,
}

/// Asks the server for the media that agree with the words of the user.
pub async fn search_library(
    client: &ApiClient,
    id_selected_lib: &str,
    words: &str,
) -> Result<SearchRoot, ApiError> {
    // A space and every other character of a query must not break the path.
    let query = encode_the_query(words);

    client
        .get_json(&format!(
            "/api/libraries/{}/search?q={}&limit={}",
            id_selected_lib, query, LIMIT
        ))
        .await
}

/// Gives the identity of every book of an answer, with no repetition.
///
/// The group of the books gives the books that agree by themselves. The group
/// of the series gives the books of a series whose name agrees. A book can
/// stand in the two groups, therefore this function gives it one time.
pub fn items_of(answer: &SearchRoot) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();

    let from_the_books = answer
        .book
        .iter()
        .filter_map(|one| one.library_item.as_ref())
        .filter_map(|item| item.id.clone());

    let from_the_series = answer
        .series
        .iter()
        .flat_map(|one| one.books.iter())
        .filter_map(|item| item.id.clone());

    for id in from_the_books.chain(from_the_series) {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }

    ids
}

/// Gives the names that the server found, for the line of the screen.
///
/// A search for the name of an author gives no book when the library holds no
/// book of that author. The user then reads the name that the server found,
/// and they know that the program did its work.
pub fn names_of(answer: &SearchRoot) -> Vec<String> {
    answer
        .authors
        .iter()
        .chain(answer.narrators.iter())
        .filter_map(|one| one.name.clone())
        .collect()
}

/// Writes a query for the part of an address.
///
/// The program has no crate for this work, and the rule is short: a letter, a
/// number, and four marks stay. Every other byte goes as `%XX`.
pub fn encode_the_query(words: &str) -> String {
    let mut out = String::with_capacity(words.len());

    for byte in words.as_bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~');

        if keep {
            out.push(*byte as char);
        } else {
            out.push_str(&format!("%{:02X}", byte));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_with_a_space_is_safe() {
        assert_eq!(encode_the_query("two words"), "two%20words");
        assert_eq!(encode_the_query("a&b=c"), "a%26b%3Dc");
        assert_eq!(encode_the_query("a/../b"), "a%2F..%2Fb");
        assert_eq!(encode_the_query("plain"), "plain");
    }

    #[test]
    fn a_query_of_a_different_writing_stays_correct() {
        // Every byte of a character outside ASCII goes as %XX.
        assert_eq!(encode_the_query("é"), "%C3%A9");
        assert_eq!(encode_the_query("日本"), "%E6%97%A5%E6%9C%AC");
    }

    fn an_answer() -> SearchRoot {
        serde_json::from_value(serde_json::json!({
            "book": [
                { "libraryItem": { "id": "book-3" } },
                { "libraryItem": { "id": "book-2" } }
            ],
            "series": [
                { "series": { "name": "A Series" },
                  "books": [ { "id": "book-3" }, { "id": "book-1" } ] }
            ],
            "authors": [ { "name": "Lewis Carroll" } ],
            "narrators": [],
            "tags": [],
            "genres": []
        }))
        .expect("the answer of the server must read")
    }

    #[test]
    fn the_answer_gives_every_book_one_time() {
        // `book-3` stands in the two groups, and it must come one time.
        assert_eq!(items_of(&an_answer()), vec!["book-3", "book-2", "book-1"]);
    }

    #[test]
    fn the_answer_gives_the_names_that_the_server_found() {
        assert_eq!(names_of(&an_answer()), vec!["Lewis Carroll"]);
    }

    #[test]
    fn an_answer_with_no_group_gives_nothing_and_no_fault() {
        let empty: SearchRoot = serde_json::from_value(serde_json::json!({})).expect("an answer");
        assert!(items_of(&empty).is_empty());
        assert!(names_of(&empty).is_empty());
    }

    #[test]
    fn a_book_with_no_identity_gives_no_line() {
        let answer: SearchRoot = serde_json::from_value(serde_json::json!({
            "book": [ { "libraryItem": { } }, { } ]
        }))
        .expect("an answer");

        assert!(items_of(&answer).is_empty());
    }
}
