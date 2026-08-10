//! The request that sends the position of an open listening session.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::json;

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
    let body = json!({
        "currentTime": format!("{}", current_time.unwrap_or(0)),
        "timeListened": format!("{}", time_listened),
    });

    client
        .post_no_content(&format!("/api/session/{}/sync", session_id), &body)
        .await
}
