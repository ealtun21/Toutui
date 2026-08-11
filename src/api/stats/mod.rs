//! The statistics of a library and of a year. See T-24.
//!
//! The view of the key `T` showed the time of the user only. These two
//! endpoints give the size of the library and the work of the year, and the
//! view shows them as two more groups.
//!
//! # `GET /api/libraries/:id/stats`
//!
//! A measurement against an Audiobookshelf 2.36.0 on 2026-08-11 gives this
//! shape:
//!
//! ```json
//! {
//!   "totalItems": 9, "totalSize": 7987553, "totalDuration": 1883,
//!   "numAudioTracks": 11, "totalAuthors": 4, "totalGenres": 2,
//!   "largestItems": [ { "id": "…", "title": "A Long Test Book",
//!                       "size": 7200565 } ],
//!   "longestItems": [ { "id": "…", "title": "A Long Test Book",
//!                       "duration": 1800 } ],
//!   "authorsWithCount": [ { "id": "…", "name": "…", "count": 6 } ],
//!   "genresWithCount": [ { "genre": "Fiction", "count": 1 } ]
//! }
//! ```
//!
//! # `GET /api/stats/year/:year`
//!
//! ```json
//! {
//!   "numListeningSessions": 6, "totalListeningTime": 401,
//!   "numBooksAdded": 9, "numAuthorsAdded": 4, "numBooks": 9,
//!   "totalBooksSize": 7987553, "totalBooksDuration": 1883,
//!   "topAuthors":   [ { "name": "Long Author",     "time": 396 } ],
//!   "topNarrators": [ { "name": "A Test Narrator", "time": 120 } ],
//!   "topGenres":    [ { "genre": "Fiction",        "time": 120 } ]
//! }
//! ```
//!
//! **`topGenres` names its value `genre`, and the two other lists name it
//! `name`.** A measurement on 2026-08-11 shows this. The three lists carry the
//! same shape for the program, therefore [`TopName`] takes both keys.
//!
//! The two lists were empty in the first measurement, because the sandbox held
//! no genre and no narrator. The measurement gave the shape only after a book
//! took a genre and a narrator **and** the user played it again: the server
//! keeps a copy of the metadata inside each session, therefore an older session
//! holds no genre.
//!
//! Every function that makes a text here is pure. A test needs no server.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;

/// One item of the list of the largest items, or of the longest items.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BigItem {
    #[serde(default)]
    pub title: Option<String>,
    /// The size in bytes. The list of the longest items gives 0 here.
    #[serde(default)]
    pub size: f64,
    /// The time in seconds. The list of the largest items gives 0 here.
    #[serde(default)]
    pub duration: f64,
}

impl BigItem {
    /// The title, or a short message for an item with no title.
    pub fn name(&self) -> String {
        match &self.title {
            Some(title) if !title.trim().is_empty() => title.clone(),
            _ => "An item with no title".to_string(),
        }
    }
}

/// One name of the lists `topAuthors`, `topNarrators`, and `topGenres`.
///
/// The server names the value `name` in the first two lists and `genre` in the
/// third. The alias takes both.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct TopName {
    #[serde(default, alias = "genre")]
    pub name: Option<String>,
    /// The time in seconds.
    #[serde(default)]
    pub time: f64,
}

impl TopName {
    pub fn label(&self) -> String {
        match &self.name {
            Some(name) if !name.trim().is_empty() => name.clone(),
            _ => "No name".to_string(),
        }
    }
}

/// The statistics of one library.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStats {
    #[serde(default)]
    pub total_items: u64,
    /// The size of every file of the library, in bytes.
    #[serde(default)]
    pub total_size: f64,
    /// The time of every media of the library, in seconds.
    #[serde(default)]
    pub total_duration: f64,
    #[serde(default)]
    pub num_audio_tracks: u64,
    #[serde(default)]
    pub total_authors: u64,
    #[serde(default)]
    pub total_genres: u64,
    #[serde(default)]
    pub largest_items: Vec<BigItem>,
    #[serde(default)]
    pub longest_items: Vec<BigItem>,
}

/// The statistics of one year.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct YearStats {
    #[serde(default)]
    pub num_listening_sessions: u64,
    /// The time of the year, in seconds.
    #[serde(default)]
    pub total_listening_time: f64,
    #[serde(default)]
    pub num_books_added: u64,
    #[serde(default)]
    pub num_authors_added: u64,
    #[serde(default)]
    pub num_books: u64,
    /// The size of the books that the year added, in bytes.
    #[serde(default)]
    pub total_books_added_size: f64,
    #[serde(default)]
    pub top_authors: Vec<TopName>,
    #[serde(default)]
    pub top_narrators: Vec<TopName>,
    #[serde(default)]
    pub top_genres: Vec<TopName>,
}

/// Asks the server for the statistics of one library.
pub async fn get_library_stats(
    client: &ApiClient,
    library_id: &str,
) -> Result<LibraryStats, ApiError> {
    client
        .get_json(&format!("/api/libraries/{}/stats", library_id))
        .await
}

/// Asks the server for the statistics of one year.
///
/// The year is a number of four digits, for example 2026.
pub async fn get_year_stats(client: &ApiClient, year: i32) -> Result<YearStats, ApiError> {
    client.get_json(&format!("/api/stats/year/{}", year)).await
}

/// The year of today.
///
/// The endpoint of the year needs a number, and the user wants the year that
/// runs now.
pub fn this_year() -> i32 {
    use chrono::Datelike;
    chrono::Local::now().year()
}

/// Writes a size for a person.
///
/// The value is a number of bytes. The function gives one number after the
/// point, because a person reads "7.6 GB" faster than "7987553 bytes".
///
/// The unit goes up at 1024, and not at 1000, because a file system counts in
/// that way.
pub fn human_size(bytes: f64) -> String {
    if !bytes.is_finite() || bytes < 1.0 {
        return "0 B".to_string();
    }

    const UNITS: [&str; 6] = ["B", "kB", "MB", "GB", "TB", "PB"];
    let mut value = bytes;
    let mut unit = 0;

    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{} B", value.round() as i64)
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of the sandbox on 2026-08-11.
    const LIBRARY: &str = r#"{
        "totalItems": 9, "totalSize": 7987553, "totalDuration": 1883,
        "numAudioTracks": 11, "totalAuthors": 4, "totalGenres": 2,
        "largestItems": [{"id":"a","title":"A Long Test Book","size":7200565}],
        "longestItems": [{"id":"a","title":"A Long Test Book","duration":1800}],
        "authorsWithCount": [{"id":"b","name":"Series Author","count":6}],
        "genresWithCount": []
    }"#;

    /// The answer of the sandbox on 2026-08-11, after a book took a genre and a
    /// narrator and the user played it again.
    const YEAR: &str = r#"{
        "numListeningSessions": 6, "totalListeningTime": 401,
        "numBooksAdded": 9, "numAuthorsAdded": 4, "numBooks": 9,
        "totalBooksAddedSize": 7987553, "totalBooksAddedDuration": 1883,
        "totalBooksSize": 7987553, "totalBooksDuration": 1883,
        "booksAddedWithCovers": ["a","b"],
        "topAuthors":   [{"name":"Long Author","time":396},{"name":"Test Author","time":5}],
        "topNarrators": [{"name":"A Test Narrator","time":120}],
        "topGenres":    [{"genre":"Fiction","time":120},{"genre":"Adventure","time":120}]
    }"#;

    #[test]
    fn it_reads_the_answer_of_a_library() {
        let stats: LibraryStats = serde_json::from_str(LIBRARY).expect("the answer must parse");
        assert_eq!(9, stats.total_items);
        assert_eq!(7_987_553.0, stats.total_size);
        assert_eq!(1883.0, stats.total_duration);
        assert_eq!(11, stats.num_audio_tracks);
        assert_eq!(4, stats.total_authors);
        assert_eq!(2, stats.total_genres);
        assert_eq!(1, stats.largest_items.len());
        assert_eq!("A Long Test Book", stats.largest_items[0].name());
        assert_eq!(7_200_565.0, stats.largest_items[0].size);
        assert_eq!(1800.0, stats.longest_items[0].duration);
    }

    #[test]
    fn it_reads_the_answer_of_a_year() {
        let stats: YearStats = serde_json::from_str(YEAR).expect("the answer must parse");
        assert_eq!(6, stats.num_listening_sessions);
        assert_eq!(401.0, stats.total_listening_time);
        assert_eq!(9, stats.num_books_added);
        assert_eq!(4, stats.num_authors_added);
        assert_eq!(7_987_553.0, stats.total_books_added_size);
        assert_eq!("Long Author", stats.top_authors[0].label());
        assert_eq!(396.0, stats.top_authors[0].time);
        assert_eq!("A Test Narrator", stats.top_narrators[0].label());
    }

    /// The trap of this endpoint: the list of the genres names its value
    /// `genre`, and the two other lists name it `name`.
    #[test]
    fn the_list_of_the_genres_uses_a_different_key() {
        let stats: YearStats = serde_json::from_str(YEAR).expect("the answer must parse");
        assert_eq!(2, stats.top_genres.len());
        assert_eq!("Fiction", stats.top_genres[0].label());
        assert_eq!(120.0, stats.top_genres[0].time);
        assert_eq!("Adventure", stats.top_genres[1].label());
    }

    #[test]
    fn an_answer_with_no_field_gives_no_error() {
        // A server of a different version can give less. The screen must show
        // the fields that came, and it must not fail.
        let library: LibraryStats = serde_json::from_str("{}").expect("an empty answer must parse");
        assert_eq!(0, library.total_items);
        assert!(library.largest_items.is_empty());
        let year: YearStats = serde_json::from_str("{}").expect("an empty answer must parse");
        assert!(year.top_genres.is_empty());
    }

    #[test]
    fn an_item_with_no_title_gives_a_short_message() {
        let item: BigItem = serde_json::from_str(r#"{"size":1}"#).expect("it must parse");
        assert_eq!("An item with no title", item.name());
        let empty: BigItem = serde_json::from_str(r#"{"title":"  "}"#).expect("it must parse");
        assert_eq!("An item with no title", empty.name());
        let name: TopName = serde_json::from_str("{}").expect("it must parse");
        assert_eq!("No name", name.label());
    }

    #[test]
    fn a_size_goes_to_the_unit_that_a_person_reads() {
        assert_eq!("0 B", human_size(0.0));
        assert_eq!("0 B", human_size(-5.0));
        assert_eq!("0 B", human_size(f64::NAN));
        assert_eq!("512 B", human_size(512.0));
        assert_eq!("1.0 kB", human_size(1024.0));
        assert_eq!("1.5 kB", human_size(1536.0));
        assert_eq!("7.6 MB", human_size(7_987_553.0));
        assert_eq!("1.0 GB", human_size(1024.0 * 1024.0 * 1024.0));
        // A number that is very large must give a unit, and never a panic.
        assert!(human_size(f64::MAX).ends_with(" PB"));
        assert_eq!("1.0 PB", human_size(1024f64.powi(5)));
    }

    #[test]
    fn the_year_of_today_has_four_digits() {
        let year = this_year();
        assert!((2020..=2100).contains(&year), "the year is {year}");
    }
}
