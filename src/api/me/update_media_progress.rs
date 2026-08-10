//! The requests that write the listening position on the server.
//!
//! The endpoint `PATCH /api/me/progress/:id` sets an absolute position.
//! Therefore the client can send the request a second time without a risk.
//!
//! **A property of the server.** A body that holds `isFinished` together with
//! `progress` and `currentTime` does not always mark the media as finished. A
//! measurement with Audiobookshelf 2.36.0 on 2026-08-10 shows that the
//! sequence of the keys changes the result:
//!
//! ```text
//! {"progress":1.0,"isFinished":true,"currentTime":60}  ->  isFinished true
//! {"currentTime":60,"isFinished":true,"progress":1.0}  ->  isFinished false
//! ```
//!
//! `serde_json` writes the keys in the sequence of the alphabet, thus the
//! application always made the second body. Therefore the application sends
//! the position first, and it sends `{"isFinished": true}` in a second
//! request. That body has one key, and the sequence then has no meaning.

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
///
/// The function sends two requests. See the note at the start of this module.
pub async fn update_media_progress2_book(
    client: &ApiClient,
    id_library_item: &str,
    current_time: Option<u32>,
    duration: &str,
    is_finished: bool,
) -> Result<(), ApiError> {
    update_media_progress_book(client, id_library_item, current_time, duration).await?;

    if !is_finished {
        return Ok(());
    }

    client
        .patch_json(
            &format!("/api/me/progress/{}", id_library_item),
            &finished_body(),
        )
        .await
}

/// Gives the body that marks a media as finished.
///
/// The body holds one key. Therefore the sequence of the keys has no meaning,
/// and the server always marks the media.
fn finished_body() -> serde_json::Value {
    json!({ "isFinished": true })
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
///
/// The function sends two requests. See the note at the start of this module.
pub async fn update_media_progress2_pod(
    client: &ApiClient,
    id_library_item: &str,
    current_time: Option<u32>,
    duration: &str,
    is_finished: bool,
    ep_id: &str,
) -> Result<(), ApiError> {
    update_media_progress_pod(client, id_library_item, current_time, duration, ep_id).await?;

    if !is_finished {
        return Ok(());
    }

    client
        .patch_json(
            &format!("/api/me/progress/{}/{}", id_library_item, ep_id),
            &finished_body(),
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// The body that marks a media must hold one key only. A body with more
    /// keys does not always mark the media, because the sequence of the keys
    /// changes the result on the server.
    #[test]
    fn the_body_of_the_mark_holds_one_key() {
        let body = finished_body();
        let object = body.as_object().expect("the body must be an object");

        assert_eq!(object.len(), 1);
        assert_eq!(object.get("isFinished"), Some(&serde_json::json!(true)));
    }
}
