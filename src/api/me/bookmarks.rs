//! The bookmarks of the user. See T-24.
//!
//! A user of a long book needs a place to come back to. Audiobookshelf holds
//! that place for each user, and every client of that user reads it.
//!
//! Measurements against an Audiobookshelf 2.36.0 on 2026-08-11:
//!
//! | Request | Answer |
//! |---|---|
//! | `POST /api/me/item/:id/bookmark` with `{"time":42,"title":"..."}` | `200`, and `{libraryItemId,time,title,createdAt}` |
//! | The same request again, with the same time | `200`, and the title changes. `createdAt` does not change |
//! | `PATCH /api/me/item/:id/bookmark` with the same body | `200`, and the title changes |
//! | `DELETE /api/me/item/:id/bookmark/:time` | `200` |
//! | The same delete again | `404` |
//! | `GET /api/me` | the field `bookmarks`, a list of every media |
//!
//! **The time is the key of a bookmark.** Two bookmarks of one media cannot
//! hold the same second, and the delete names the second. Therefore the
//! program does not need an identity of its own.
//!
//! The reference of <https://api.audiobookshelf.org> gives
//! `POST /api/me/bookmarks`. That path gives `404` on 2.36.0. See section 1 of
//! `docs/T-24-coverage.md`.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use crate::utils::convert_seconds::clock;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    #[serde(default)]
    pub library_item_id: String,
    /// The place in the media, in seconds.
    #[serde(default)]
    pub time: f64,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct Me {
    #[serde(default)]
    bookmarks: Vec<Bookmark>,
}

/// Asks the server for every bookmark of this account.
///
/// `GET /api/me` gives them together, therefore the program sends one request
/// for every media.
pub async fn get_bookmarks(client: &ApiClient) -> Result<Vec<Bookmark>, ApiError> {
    let me: Me = client.get_json("/api/me").await?;

    Ok(me.bookmarks)
}

/// Writes a bookmark on the server.
///
/// A bookmark that holds the same second changes its name, and it does not
/// make a second line. The server does that work.
pub async fn add_bookmark(
    client: &ApiClient,
    item_id: &str,
    time: f64,
    title: &str,
) -> Result<(), ApiError> {
    let body = serde_json::json!({ "time": whole_seconds(time), "title": title });

    client
        .post_no_content(&format!("/api/me/item/{}/bookmark", item_id), &body)
        .await
}

/// Removes a bookmark from the server.
pub async fn remove_bookmark(client: &ApiClient, item_id: &str, time: f64) -> Result<(), ApiError> {
    client
        .delete_no_content(&format!(
            "/api/me/item/{}/bookmark/{}",
            item_id,
            whole_seconds(time)
        ))
        .await
}

/// Gives the second of a place.
///
/// The address of the delete holds the time, therefore the value must be a
/// whole number and it must be the same number that the write sent. A value
/// that is not a number, or a value below zero, gives zero.
pub fn whole_seconds(time: f64) -> i64 {
    if time.is_finite() && time > 0.0 {
        time.round() as i64
    } else {
        0
    }
}

/// Gives the bookmarks of one media, the first place first.
///
/// `GET /api/me` gives the bookmarks of every media together. A view of one
/// book must show the bookmarks of that book only.
pub fn of_item(all: &[Bookmark], item_id: &str) -> Vec<Bookmark> {
    let mut mine: Vec<Bookmark> = all
        .iter()
        .filter(|one| one.library_item_id == item_id)
        .cloned()
        .collect();

    mine.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    mine
}

/// Gives the name that a bookmark takes when the user writes none.
///
/// The name holds the place, therefore a list of names says something even
/// when the user writes nothing.
pub fn default_title(time: f64) -> String {
    format!("The place at {}", clock(time))
}

/// Makes the text of each line of the list of the bookmarks.
pub fn lines(bookmarks: &[Bookmark]) -> Vec<String> {
    bookmarks
        .iter()
        .map(|one| format!("{}  ({})", one.title, clock(one.time)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of the sandbox, measured on 2026-08-11.
    fn the_answer_of_the_server() -> Me {
        serde_json::from_value(serde_json::json!({
            "id": "a-user",
            "username": "toutuitest",
            "bookmarks": [
                { "libraryItemId": "book-1", "time": 42, "title": "A test mark",
                  "createdAt": 1786404361214i64 },
                { "libraryItemId": "book-1", "time": 10, "title": "The start",
                  "createdAt": 1786404361215i64 },
                { "libraryItemId": "book-2", "time": 5, "title": "Of a different book",
                  "createdAt": 1786404361216i64 }
            ]
        }))
        .expect("the answer of the server must read")
    }

    #[test]
    fn the_answer_of_a_real_server_reads() {
        let me = the_answer_of_the_server();

        assert_eq!(me.bookmarks.len(), 3);
        assert_eq!(me.bookmarks[0].library_item_id, "book-1");
        assert_eq!(me.bookmarks[0].time, 42.0);
        assert_eq!(me.bookmarks[0].title, "A test mark");
    }

    #[test]
    fn an_answer_with_no_bookmark_gives_no_fault() {
        let me: Me = serde_json::from_value(serde_json::json!({ "username": "a" }))
            .expect("the answer must read");

        assert!(me.bookmarks.is_empty());
        assert!(of_item(&me.bookmarks, "book-1").is_empty());
    }

    /// The list of the server holds every media. A view of one book must show
    /// the bookmarks of that book only.
    #[test]
    fn the_view_of_a_book_holds_the_bookmarks_of_that_book() {
        let all = the_answer_of_the_server().bookmarks;
        let mine = of_item(&all, "book-1");

        assert_eq!(mine.len(), 2);
        assert!(mine.iter().all(|one| one.library_item_id == "book-1"));
    }

    /// The server gives no sequence. The first place must come first, or the
    /// user cannot find a place in a long book.
    #[test]
    fn the_first_place_comes_first() {
        let all = the_answer_of_the_server().bookmarks;
        let mine = of_item(&all, "book-1");

        assert_eq!(mine[0].time, 10.0);
        assert_eq!(mine[1].time, 42.0);
    }

    /// The address of the delete holds the time. A number with a part after
    /// the full stop would name a bookmark that does not exist.
    #[test]
    fn the_time_of_the_address_is_a_whole_number() {
        assert_eq!(whole_seconds(42.0), 42);
        assert_eq!(whole_seconds(42.4), 42);
        assert_eq!(whole_seconds(42.6), 43);
        assert_eq!(whole_seconds(0.0), 0);
        assert_eq!(whole_seconds(-5.0), 0);
        assert_eq!(whole_seconds(f64::NAN), 0);
        assert_eq!(whole_seconds(f64::INFINITY), 0);
    }

    #[test]
    fn a_bookmark_with_no_name_takes_the_name_of_its_place() {
        let name = default_title(750.0);

        assert!(name.starts_with("The place at"));
        assert!(name.contains("12:30"));
    }

    #[test]
    fn every_bookmark_gives_one_line() {
        let all = of_item(&the_answer_of_the_server().bookmarks, "book-1");
        let text = lines(&all);

        assert_eq!(text.len(), 2);
        assert!(text[0].contains("The start"));
        assert!(text[1].contains("A test mark"));
    }

    #[test]
    fn no_bookmark_gives_no_line() {
        assert!(lines(&[]).is_empty());
    }
}
