//! The request that starts a playback session.
//!
//! A `POST` request makes a new session on the server. The client never sends
//! this request a second time, because a second request makes a duplicate
//! session.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use crate::player::vlc::fetch_vlc_data::get_vlc_version;
use serde_json::json;
use serde_json::Value;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Makes the body of a playback session request.
///
/// The body tells the server which player the user has. The server shows this
/// data in the panel of user activity.
async fn session_body() -> Value {
    let mut vlc_version = String::new();
    match get_vlc_version().await {
        Ok(version) => {
            vlc_version = version;
        }
        Err(e) => {
            log::error!("[get_vlc_version] {}", e);
        }
    }

    json!({
        // avoid latency load, allow view chapter, cover etc.(the .m3u8 stream
        // the original format, ex: .m4b) when playing with vlc
        "forceDirectPlay": true,
        "mediaPlayer": format!("VLC v{}", vlc_version),
        "deviceInfo": {
            "clientName": "Toutui",
            "clientVersion": format!("v{}", VERSION),
            // to have OS displayed in user activity pannel (audiobookshelf/config/users/)
            "manufacturer": format!("{}", std::env::consts::OS),
            "model": format!("{}", std::env::consts::ARCH),
        }
    })
}

/// Takes the values that the player needs from the answer of the server.
///
/// The sequence of the values is important. The callers read the list by
/// position.
fn collect_info_item(v: &Value, subtitle: &Value) -> Vec<String> {
    let current_time = v["currentTime"].as_f64().unwrap_or(0.0);
    let content_url = v["audioTracks"][0]["contentUrl"].as_str().unwrap_or("");
    let duration = v["audioTracks"][0]["duration"].as_f64().unwrap_or(0.0);
    let duration: u32 = duration as u32;
    let id_session = v["id"].as_str().unwrap_or("");
    let title = v["mediaMetadata"]["title"].as_str().unwrap_or("N/A");
    let subtitle = subtitle.as_str().unwrap_or("N/A");
    let author = v["displayAuthor"].as_str().unwrap_or("N/A");

    vec![
        current_time.to_string(),
        content_url.to_string(),
        duration.to_string(),
        id_session.to_string(),
        title.to_string(),
        subtitle.to_string(),
        author.to_string(),
    ]
}

/// Starts a playback session for a book.
///
/// See <https://api.audiobookshelf.org/#play-a-library-item-or-podcast-episode>.
pub async fn post_start_playback_session_book(
    client: &ApiClient,
    id_library_item: &str,
) -> Result<Vec<String>, ApiError> {
    let body = session_body().await;

    let v: Value = client
        .post_json(&format!("/api/items/{}/play", id_library_item), &body)
        .await?;

    // A book gives the subtitle from the same field as the title.
    let subtitle = v["mediaMetadata"]["title"].clone();

    Ok(collect_info_item(&v, &subtitle))
}

/// Starts a playback session for a podcast episode.
pub async fn post_start_playback_session_pod(
    client: &ApiClient,
    id_library_item: &str,
    pod_ep_id: &str,
) -> Result<Vec<String>, ApiError> {
    let body = session_body().await;

    let v: Value = client
        .post_json(
            &format!("/api/items/{}/play/{}", id_library_item, pod_ep_id),
            &body,
        )
        .await?;

    // A podcast episode gives the subtitle from the display title.
    let subtitle = v["displayTitle"].clone();

    Ok(collect_info_item(&v, &subtitle))
}
