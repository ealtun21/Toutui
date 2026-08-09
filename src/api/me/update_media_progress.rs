//! The requests that write the listening position on the server.
//!
//! The endpoint `PATCH /api/me/progress/:id` sets an absolute position.
//! Therefore the client can send the request a second time without a risk.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::json;

/// Calculates the part of the item that the user listened to.
///
/// The function gives `0.0` if the duration is not a number, or if the
/// duration is not more than zero. A division by zero gives a value that JSON
/// does not accept.
fn progress_ratio(current_time: u32, duration: &str) -> f32 {
    let duration_f32 = duration.parse::<f32>().unwrap_or(0.0);

    if duration_f32 > 0.0 {
        current_time as f32 / duration_f32
    } else {
        0.0
    }
}

/// Sends the listening position of a book to the server.
///
/// See <https://api.audiobookshelf.org/#create-update-media-progress>.
pub async fn update_media_progress_book(
    client: &ApiClient,
    id_library_item: &str,
    current_time: Option<u32>,
    duration: &str,
) -> Result<(), ApiError> {
    let body = json!({
        "progress": progress_ratio(current_time.unwrap_or(0), duration),
        "currentTime": current_time,
    });

    client
        .patch_json(&format!("/api/me/progress/{}", id_library_item), &body)
        .await
}

/// Sends the listening position of a book, and marks the book as finished.
pub async fn update_media_progress2_book(
    client: &ApiClient,
    id_library_item: &str,
    current_time: Option<u32>,
    duration: &str,
    is_finished: bool,
) -> Result<(), ApiError> {
    let body = json!({
        "progress": progress_ratio(current_time.unwrap_or(0), duration),
        "isFinished": is_finished,
        "currentTime": current_time,
    });

    client
        .patch_json(&format!("/api/me/progress/{}", id_library_item), &body)
        .await
}

/// Sends the listening position of a podcast episode to the server.
pub async fn update_media_progress_pod(
    client: &ApiClient,
    id_library_item: &str,
    current_time: Option<u32>,
    duration: &str,
    ep_id: &str,
) -> Result<(), ApiError> {
    let body = json!({
        "progress": progress_ratio(current_time.unwrap_or(0), duration),
        "currentTime": current_time,
    });

    client
        .patch_json(
            &format!("/api/me/progress/{}/{}", id_library_item, ep_id),
            &body,
        )
        .await
}

/// Sends the listening position of a podcast episode, and marks the episode
/// as finished.
pub async fn update_media_progress2_pod(
    client: &ApiClient,
    id_library_item: &str,
    current_time: Option<u32>,
    duration: &str,
    is_finished: bool,
    ep_id: &str,
) -> Result<(), ApiError> {
    let body = json!({
        "progress": progress_ratio(current_time.unwrap_or(0), duration),
        "isFinished": is_finished,
        "currentTime": current_time,
    });

    client
        .patch_json(
            &format!("/api/me/progress/{}/{}", id_library_item, ep_id),
            &body,
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::progress_ratio;

    #[test]
    fn the_ratio_is_the_position_divided_by_the_duration() {
        assert_eq!(progress_ratio(50, "100"), 0.5);
    }

    /// A duration of zero, or a duration that is not a number, must not make
    /// an infinite value. JSON does not accept an infinite value.
    #[test]
    fn a_bad_duration_gives_zero() {
        assert_eq!(progress_ratio(50, "0"), 0.0);
        assert_eq!(progress_ratio(50, "N/A"), 0.0);
    }
}
