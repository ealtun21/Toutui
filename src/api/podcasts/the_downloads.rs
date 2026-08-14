//! The queue of the episodes that the server downloads. See T-81.
//!
//! The key `E` tells the server to get the episodes of a feed that it does not
//! hold. **The server does that work alone**, and the program showed nothing of
//! it: a user who pressed `E` on a feed of 57 episodes read one message, and no
//! view of the program said what happened after it.
//!
//! `GET /api/libraries/:id/episode-downloads` gives the work of that library:
//!
//! ```json
//! { "currentDownload": { "episodeDisplayTitle": "Letter 4", … },
//!   "queue": [ { "episodeDisplayTitle": "Letter 5", … }, … ] }
//! ```
//!
//! **The episode that downloads now is not in `queue`.** A measurement of
//! 2026-08-11 against an Audiobookshelf 2.36.0 gave "Letter 4" as
//! `currentDownload` and eight episodes in `queue`.
//!
//! `GET /api/podcasts/:id/clear-queue` empties the queue of one podcast and it
//! gives `200`. **It does not stop the episode that downloads now**: the log of
//! the server of the same measurement wrote "Successfully downloaded podcast
//! episode \"Letter 12\"" after the clear.
//!
//! **The queue does not fill at once.** `POST /api/podcasts/:id/download-episodes`
//! answers `200` and the server takes some seconds to put the episodes in the
//! queue: a read two seconds after that request gave an empty queue, and the
//! clear three seconds later removed nine episodes.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::Value;

/// One episode that the server downloads, or that waits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneDownload {
    /// The title of the episode, for the user.
    pub title: String,
    /// The podcast of the episode. The key that empties the queue needs it.
    pub item_id: String,
    /// The name of the podcast.
    pub podcast: String,
    /// The server downloads this episode now.
    pub now: bool,
}

impl OneDownload {
    /// The line of this episode, in the list of the screen.
    pub fn line(&self) -> String {
        let mark = if self.now { "▼" } else { " " };

        format!("{} {} — {}", mark, self.title, self.podcast)
    }

    /// The name of this episode for the line of the user. See T-166.
    ///
    /// **The field `now` stands outside this name.** An episode that becomes
    /// the download of this moment is the same episode: it moves from `queue`
    /// to `currentDownload` of the answer of the server, and the line of the
    /// user must follow it there.
    ///
    /// The podcast alone is not enough: a queue holds many episodes of one
    /// podcast.
    pub fn key(&self) -> String {
        format!("{}\u{1f}{}", self.item_id, self.title)
    }
}

/// Gives one download of the answer of the server.
fn one(value: &Value, now: bool) -> Option<OneDownload> {
    let title = value
        .get("episodeDisplayTitle")
        .and_then(Value::as_str)
        .unwrap_or("An episode")
        .to_string();

    let item_id = value
        .get("libraryItemId")
        .and_then(Value::as_str)?
        .to_string();

    let podcast = value
        .get("podcastTitle")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    Some(OneDownload {
        title,
        item_id,
        podcast,
        now,
    })
}

/// Gives every download of the answer of the server, the one of now first.
///
/// The function is pure, therefore a test needs no server.
pub fn the_downloads_of_the_answer(answer: &Value) -> Vec<OneDownload> {
    let mut all: Vec<OneDownload> = Vec::new();

    if let Some(value) = answer.get("currentDownload") {
        if let Some(download) = one(value, true) {
            all.push(download);
        }
    }

    if let Some(queue) = answer.get("queue").and_then(Value::as_array) {
        all.extend(queue.iter().filter_map(|value| one(value, false)));
    }

    all
}

/// Asks the server for the work of the downloads of one library.
pub async fn the_downloads_of_the_library(
    client: &ApiClient,
    library_id: &str,
) -> Result<Vec<OneDownload>, ApiError> {
    let answer: Value = client
        .get_json(&format!("/api/libraries/{}/episode-downloads", library_id))
        .await?;

    Ok(the_downloads_of_the_answer(&answer))
}

/// Tells the server to empty the queue of one podcast.
///
/// The episode that downloads now goes on: the server holds it outside the
/// queue.
pub async fn empty_the_queue(client: &ApiClient, item_id: &str) -> Result<(), ApiError> {
    let _: Value = client
        .get_json(&format!("/api/podcasts/{}/clear-queue", item_id))
        .await
        .or_else(|error| match error {
            // The endpoint answers `200` with no body. A body that is not JSON
            // is not a fault of this request.
            ApiError::Decode(_) => Ok(Value::Null),
            other => Err(other),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of the measurement of 2026-08-11, with the fields that this
    /// module reads.
    fn the_answer_of_the_server() -> Value {
        serde_json::json!({
            "currentDownload": {
                "episodeDisplayTitle": "Letter 4",
                "libraryItemId": "a-podcast",
                "podcastTitle": "Letters of Two Brides"
            },
            "queue": [
                { "episodeDisplayTitle": "Letter 5", "libraryItemId": "a-podcast",
                  "podcastTitle": "Letters of Two Brides" },
                { "episodeDisplayTitle": "Letter 6", "libraryItemId": "a-podcast",
                  "podcastTitle": "Letters of Two Brides" }
            ]
        })
    }

    #[test]
    fn the_episode_of_now_stands_first_and_it_carries_its_mark() {
        let all = the_downloads_of_the_answer(&the_answer_of_the_server());

        assert_eq!(all.len(), 3, "the episode of now is not in the queue");
        assert_eq!(all[0].title, "Letter 4");
        assert!(all[0].now);
        assert!(!all[1].now);

        assert_eq!(all[0].line(), "▼ Letter 4 — Letters of Two Brides");
        assert_eq!(all[1].line(), "  Letter 5 — Letters of Two Brides");
    }

    /// A library that downloads nothing gives `currentDownload` of null and an
    /// empty queue.
    #[test]
    fn a_library_with_no_work_gives_no_line() {
        let answer = serde_json::json!({ "currentDownload": null, "queue": [] });

        assert!(the_downloads_of_the_answer(&answer).is_empty());
    }

    /// A server that gives no field must not stop the program.
    #[test]
    fn an_answer_with_no_field_gives_no_line() {
        assert!(the_downloads_of_the_answer(&serde_json::json!({})).is_empty());
    }

    /// An entry with no podcast still gives its line.
    #[test]
    fn an_entry_with_no_name_of_the_podcast_gives_a_line() {
        let answer = serde_json::json!({
            "queue": [ { "libraryItemId": "a-podcast" } ]
        });

        let all = the_downloads_of_the_answer(&answer);

        assert_eq!(all[0].title, "An episode");
        assert_eq!(all[0].line(), "  An episode — ");
    }
}
