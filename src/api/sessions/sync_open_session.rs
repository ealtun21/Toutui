//! The request that sends the position of an open listening session.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::json;

/// Gives the body of the sync of an open session.
///
/// **The two values are numbers, and this program sent them as a text.** The
/// server keeps the form that it takes: one sync of `"currentTime": "714"` made
/// `GET /api/me` answer `"currentTime": "714"` for that media, and the row of
/// that answer did not read at all — the Home view then lost the position of a
/// media that the server holds. A measurement of 2026-08-12 against
/// Audiobookshelf 2.36.0 gave `"currentTime": 714` for the same request with a
/// number. See T-130.
///
/// The function is pure, therefore a test needs no server.
pub fn the_body_of_a_sync(current_time: Option<u32>, time_listened: u32) -> serde_json::Value {
    json!({
        "currentTime": current_time.unwrap_or(0),
        "timeListened": time_listened,
    })
}

/// Sends the position of an open listening session to the server.
///
/// The client never sends this request a second time. A second request adds
/// the listened time two times on the server.
///
/// See <https://api.audiobookshelf.org/#sync-an-open-session>.
pub async fn sync_session(
    client: &ApiClient,
    session_id: &str,
    current_time: Option<u32>,
    time_listened: u32,
) -> Result<(), ApiError> {
    client
        .post_no_content(
            &format!("/api/session/{}/sync", session_id),
            &the_body_of_a_sync(current_time, time_listened),
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The position of a session is a number in the body of the request.** A
    /// text of that value stays in the database of the server, and the answer of
    /// `GET /api/me` then holds a row that this program cannot read. See T-130.
    #[test]
    fn the_body_of_a_sync_holds_two_numbers() {
        let body = the_body_of_a_sync(Some(714), 10);

        assert!(
            body["currentTime"].is_number(),
            "the position must be a number, and it is {}",
            body["currentTime"]
        );
        assert!(
            body["timeListened"].is_number(),
            "the time that the user listened must be a number, and it is {}",
            body["timeListened"]
        );
        assert_eq!(body["currentTime"], 714);
        assert_eq!(body["timeListened"], 10);
    }

    /// A session with no position gives the second 0, and that value is a
    /// number too.
    #[test]
    fn a_session_with_no_position_gives_the_number_zero() {
        let body = the_body_of_a_sync(None, 0);

        assert!(body["currentTime"].is_number());
        assert_eq!(body["currentTime"], 0);
    }
}
