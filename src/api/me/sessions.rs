//! The sessions of the user, with pages. See T-24.
//!
//! The view of the key `T` shows the five last sessions. This endpoint gives
//! every session that the server holds, and it gives them in pages.
//!
//! `GET /api/me/listening-sessions?itemsPerPage=N&page=P`. A measurement
//! against an Audiobookshelf 2.36.0 on 2026-08-11 gives this shape:
//!
//! ```json
//! {
//!   "total": 6, "numPages": 3, "page": 1, "itemsPerPage": 2,
//!   "sessions": [ { "id": "…", "displayTitle": "A Long Test Book",
//!                   "displayAuthor": "Long Author", "date": "2026-08-11",
//!                   "dayOfWeek": "Tuesday", "timeListening": 120,
//!                   "currentTime": 90, "duration": 1800,
//!                   "mediaType": "book", "mediaPlayer": "test" } ]
//! }
//! ```
//!
//! **The first page is the page 0.** A measurement of `page=0` and `page=1`
//! with `itemsPerPage=2` gives two different pairs of sessions, therefore the
//! server counts the pages from 0.
//!
//! **A page after the last page gives `200` and an empty list**, and not an
//! error. `page=99` of a server with 3 pages gives `sessions: []` and it keeps
//! `total` and `numPages`. Therefore the caller must look at the list, and not
//! at the status.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;

/// The number of sessions of one request.
///
/// A page of 25 fills more than one screen, therefore the user scrolls before
/// the program asks the server again. A larger page would carry sessions that
/// the user never reads.
pub const PER_PAGE: usize = 25;

/// One session of the user.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlaySession {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub display_title: Option<String>,
    #[serde(default)]
    pub display_author: Option<String>,
    /// The date of the session, of the form `2026-08-11`.
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub day_of_week: Option<String>,
    /// The time that the user listened in this session, in seconds.
    #[serde(default)]
    pub time_listening: f64,
    /// The place in the media at the end of the session, in seconds.
    #[serde(default)]
    pub current_time: f64,
    /// The time of the whole media, in seconds.
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub media_player: Option<String>,
}

impl PlaySession {
    /// The title, or a short message for a session with no title.
    pub fn title(&self) -> String {
        match &self.display_title {
            Some(title) if !title.trim().is_empty() => title.clone(),
            _ => "A media with no title".to_string(),
        }
    }

    /// The author, or an empty text.
    pub fn author(&self) -> String {
        self.display_author.clone().unwrap_or_default()
    }

    /// The date, or an empty text.
    pub fn day(&self) -> String {
        self.date.clone().unwrap_or_default()
    }

    /// The part of the media that the user reached, from 0 to 1.
    ///
    /// A media with no time gives 0. A place after the end gives 1.
    pub fn fraction(&self) -> f64 {
        if !self.duration.is_finite() || self.duration <= 0.0 || !self.current_time.is_finite() {
            return 0.0;
        }
        (self.current_time / self.duration).clamp(0.0, 1.0)
    }
}

/// One page of the sessions.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionPage {
    /// The number of sessions of the account, over every page.
    #[serde(default)]
    pub total: usize,
    #[serde(default)]
    pub num_pages: usize,
    /// The page that this answer holds. The first page is 0.
    #[serde(default)]
    pub page: usize,
    #[serde(default)]
    pub items_per_page: usize,
    #[serde(default)]
    pub sessions: Vec<PlaySession>,
}

/// Asks the server for one page of the sessions.
///
/// The first page is the page 0.
pub async fn get_sessions(
    client: &ApiClient,
    page: usize,
    per_page: usize,
) -> Result<SessionPage, ApiError> {
    client
        .get_json(&format!(
            "/api/me/listening-sessions?itemsPerPage={}&page={}",
            per_page, page
        ))
        .await
}

/// Tells if the server holds a page after the page that came.
///
/// The function takes the number of sessions that the program holds and the
/// number of sessions of the account. It gives `false` for an answer with no
/// session, therefore a server that gives an empty page stops the reads.
pub fn there_is_more(held: usize, page: &SessionPage) -> bool {
    if page.sessions.is_empty() {
        return false;
    }
    held < page.total
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of the sandbox on 2026-08-11.
    const PAGE: &str = r#"{
        "total": 6, "numPages": 3, "page": 1, "itemsPerPage": 2,
        "sessions": [
            { "id": "a", "displayTitle": "A Long Test Book",
              "displayAuthor": "Long Author", "date": "2026-08-11",
              "dayOfWeek": "Tuesday", "timeListening": 120,
              "currentTime": 90, "duration": 1800,
              "mediaType": "book", "mediaPlayer": "test" },
            { "id": "b", "displayTitle": "Multi File Test Book",
              "displayAuthor": "Test Author", "date": "2026-08-10",
              "dayOfWeek": "Monday", "timeListening": 5,
              "currentTime": 5, "duration": 60, "mediaType": "book" }
        ]
    }"#;

    #[test]
    fn it_reads_a_page_of_the_server() {
        let page: SessionPage = serde_json::from_str(PAGE).expect("the answer must parse");
        assert_eq!(6, page.total);
        assert_eq!(3, page.num_pages);
        assert_eq!(1, page.page);
        assert_eq!(2, page.items_per_page);
        assert_eq!(2, page.sessions.len());

        let first = &page.sessions[0];
        assert_eq!("A Long Test Book", first.title());
        assert_eq!("Long Author", first.author());
        assert_eq!("2026-08-11", first.day());
        assert_eq!(120.0, first.time_listening);
        assert_eq!(Some("Tuesday".to_string()), first.day_of_week);
    }

    #[test]
    fn the_part_of_the_media_comes_from_the_place_and_the_time() {
        let page: SessionPage = serde_json::from_str(PAGE).expect("the answer must parse");
        assert!((page.sessions[0].fraction() - 0.05).abs() < 1e-9);
        assert!((page.sessions[1].fraction() - 5.0 / 60.0).abs() < 1e-9);
    }

    #[test]
    fn a_session_with_no_time_gives_no_part_and_no_fault() {
        for body in [
            r#"{"duration":0,"currentTime":5}"#,
            r#"{"duration":-1,"currentTime":5}"#,
            r#"{"currentTime":5}"#,
            "{}",
        ] {
            let session: PlaySession = serde_json::from_str(body).expect("it must parse");
            assert_eq!(0.0, session.fraction(), "{body}");
        }
        // A place after the end gives the whole media, and not a value above 1.
        let past: PlaySession =
            serde_json::from_str(r#"{"duration":10,"currentTime":900}"#).expect("it must parse");
        assert_eq!(1.0, past.fraction());
    }

    #[test]
    fn a_session_with_no_title_gives_a_short_message() {
        let session: PlaySession = serde_json::from_str("{}").expect("it must parse");
        assert_eq!("A media with no title", session.title());
        assert_eq!("", session.author());
        assert_eq!("", session.day());
        let empty: PlaySession =
            serde_json::from_str(r#"{"displayTitle":"  "}"#).expect("it must parse");
        assert_eq!("A media with no title", empty.title());
    }

    #[test]
    fn an_answer_with_no_field_gives_no_error() {
        let page: SessionPage = serde_json::from_str("{}").expect("an empty answer must parse");
        assert_eq!(0, page.total);
        assert!(page.sessions.is_empty());
    }

    #[test]
    fn the_program_asks_for_a_page_while_it_holds_less_than_the_whole() {
        let page: SessionPage = serde_json::from_str(PAGE).expect("the answer must parse");
        assert!(there_is_more(2, &page));
        assert!(there_is_more(5, &page));
        assert!(!there_is_more(6, &page));
        assert!(!there_is_more(99, &page));
    }

    /// A page after the last page gives `200` and an empty list. The program
    /// must stop there, and it must not ask for ever.
    #[test]
    fn a_page_with_no_session_stops_the_reads() {
        let empty: SessionPage =
            serde_json::from_str(r#"{"total":6,"numPages":3,"page":99,"sessions":[]}"#)
                .expect("the answer must parse");
        assert!(!there_is_more(2, &empty));
    }
}
