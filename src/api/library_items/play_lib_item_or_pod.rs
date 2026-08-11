//! The request that starts a playback session.
//!
//! A `POST` request makes a new session on the server. The client never sends
//! this request a second time, because a second request makes a duplicate
//! session.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::json;
use serde_json::Value;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Makes the body of a playback session request.
///
/// The body tells the server which player the user has. The server shows this
/// data in the panel of user activity.
///
/// The application decodes the audio itself. Therefore the name of the player
/// is the name of the application. The old code started `vlc --version` here,
/// and that started a process for each session.
fn session_body() -> Value {
    json!({
        // The server must give the original file, and not a stream that it
        // makes again. The engine reads the original file.
        "forceDirectPlay": true,
        "mediaPlayer": format!("Toutui v{}", VERSION),
        "deviceInfo": {
            "clientName": "Toutui",
            "clientVersion": format!("v{}", VERSION),
            // to have OS displayed in user activity pannel (audiobookshelf/config/users/)
            "manufacturer": format!("{}", std::env::consts::OS),
            "model": format!("{}", std::env::consts::ARCH),
        }
    })
}

/// Makes the body that asks for a stream of the server. See T-53.
///
/// `forceTranscode` gives `playMethod: 2` and one address of HLS for the whole
/// media. ffmpeg of the server makes that stream, therefore every codec that
/// ffmpeg reads becomes a codec that this program plays.
///
/// The list of the forms that the client takes holds MP3 and ADTS only. The
/// server reads that list before it copies the codec of the file, therefore a
/// file of a form that no decoder of this program reads becomes AAC.
fn stream_body() -> Value {
    let mut body = session_body();

    if let Some(object) = body.as_object_mut() {
        object.insert("forceDirectPlay".to_string(), json!(false));
        object.insert("forceTranscode".to_string(), json!(true));
        object.insert(
            "supportedMimeTypes".to_string(),
            json!(["audio/mpeg", "audio/aac"]),
        );
    }

    body
}

/// The stream of one media of the server. See T-53.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamSession {
    /// The identity of the session. The program closes it at the end.
    pub session_id: String,
    /// The address of the playlist of HLS.
    pub playlist: String,
    /// The place that the server holds, in seconds.
    pub current_time: f64,
    /// The length of the whole media, in seconds.
    pub duration: f64,
}

/// Reads the answer of a session of a stream.
///
/// The function is pure, therefore a test examines it with the answer of the
/// measurement and with no server.
pub fn stream_session_of(v: &Value) -> Result<StreamSession, String> {
    let playlist = v["audioTracks"][0]["contentUrl"]
        .as_str()
        .unwrap_or("")
        .to_string();

    if playlist.is_empty() {
        return Err("The session of the server names no stream.".to_string());
    }

    let session_id = v["id"].as_str().unwrap_or("").to_string();

    if session_id.is_empty() {
        return Err("The session of the server has no identity.".to_string());
    }

    // The whole media stands in one stream, therefore the length of the media is
    // the length of the session and not the length of one file.
    let duration = v["duration"]
        .as_f64()
        .or_else(|| v["audioTracks"][0]["duration"].as_f64())
        .unwrap_or(0.0);

    Ok(StreamSession {
        session_id,
        playlist,
        current_time: v["currentTime"].as_f64().unwrap_or(0.0),
        duration,
    })
}

/// Asks the server for a stream of the whole media. See T-53.
pub async fn post_a_stream_session(
    client: &ApiClient,
    id_library_item: &str,
    episode_id: Option<&str>,
) -> Result<StreamSession, ApiError> {
    let body = stream_body();

    let path = match episode_id {
        Some(episode) => format!("/api/items/{}/play/{}", id_library_item, episode),
        None => format!("/api/items/{}/play", id_library_item),
    };

    let v: Value = client.post_json(&path, &body).await?;

    stream_session_of(&v).map_err(ApiError::Decode)
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
    let body = session_body();

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
    let body = session_body();

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

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of an Audiobookshelf 2.36.0 for a session of a stream, of the
    /// measurement of 2026-08-11. The answer holds one track for the whole
    /// media, and the address of a playlist of HLS.
    #[test]
    fn the_answer_of_a_stream_gives_the_playlist_and_the_length() {
        let answer = json!({
            "id": "d16e658d-75a0-4393-84ca-78b06c708d64",
            "playMethod": 2,
            "currentTime": 12.5,
            "duration": 1829.999,
            "audioTracks": [
                {
                    "index": 1,
                    "duration": 1829.999,
                    "contentUrl": "/hls/d16e658d-75a0-4393-84ca-78b06c708d64/output.m3u8",
                    "mimeType": "application/vnd.apple.mpegurl"
                }
            ]
        });

        let session = stream_session_of(&answer).expect("the answer must give a stream");

        assert_eq!(session.session_id, "d16e658d-75a0-4393-84ca-78b06c708d64");
        assert_eq!(
            session.playlist,
            "/hls/d16e658d-75a0-4393-84ca-78b06c708d64/output.m3u8"
        );
        assert_eq!(session.current_time, 12.5);
        assert!((session.duration - 1829.999).abs() < 0.001);
    }

    /// A session of a direct playback names no stream. The program must then
    /// keep the file, and it must not stop.
    #[test]
    fn an_answer_with_no_stream_gives_a_fault() {
        assert!(stream_session_of(&json!({})).is_err());
        assert!(stream_session_of(&json!({"id": "a", "audioTracks": []})).is_err());
        // A stream with no identity cannot be closed.
        assert!(stream_session_of(&json!({
            "audioTracks": [{"contentUrl": "/hls/a/output.m3u8"}]
        }))
        .is_err());
    }

    /// The body of a stream must ask for the stream, and it must not ask for the
    /// file. See T-53.
    #[test]
    fn the_body_of_a_stream_asks_for_the_stream() {
        let body = stream_body();

        assert_eq!(body["forceDirectPlay"], json!(false));
        assert_eq!(body["forceTranscode"], json!(true));
        assert_eq!(
            body["supportedMimeTypes"],
            json!(["audio/mpeg", "audio/aac"])
        );

        // The body of a direct playback does not change.
        assert_eq!(session_body()["forceDirectPlay"], json!(true));
        assert!(session_body().get("forceTranscode").is_none());
    }
}
