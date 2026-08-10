//! The offline mode.
//!
//! The application plays a local copy when the server does not answer. This
//! module holds the three parts of that work:
//!
//! 1. The lists of the media that the disk holds.
//! 2. The track list of a local copy, for the engine.
//! 3. The positions that wait for the server, and the rule that decides
//!    between the local position and the position of the server.
//!
//! See T-25.

use crate::api::client::ApiClient;
use crate::api::me::get_media_progress::get_book_progress;
use crate::api::me::update_media_progress::*;
use crate::db::crud::*;
use crate::player::engine::track::{Track, TrackList};
use log::{info, warn};
use std::time::{SystemTime, UNIX_EPOCH};

/// Gives the time of the local computer in milliseconds.
///
/// A clock that is before 1970 gives 0. The value is only a comparison with
/// `lastUpdate` of the server, thus 0 means "older than every position of the
/// server".
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as i64)
        .unwrap_or(0)
}

/// Tells if the application must send the local position to the server.
///
/// The user listens offline, and a different client writes a position in the
/// same time. Then two positions exist. The newer position wins.
///
/// The application sends its position when the local position is not older
/// than the position of the server. A server that has no position for this
/// media gives `None`, and the application always sends.
pub fn should_send(local_updated_at: i64, server_last_update: Option<i64>) -> bool {
    match server_last_update {
        Some(server) => local_updated_at >= server,
        None => true,
    }
}

/// Makes the track list of a local copy.
///
/// The rows of `download_files` give the sequence, the path, and the length of
/// each file. The engine needs a file name for the hint of the format, and the
/// path holds that name.
///
/// Gives `None` when the disk holds no file of this media.
pub fn tracks_from_downloads(key: &str, username: &str) -> Option<TrackList> {
    let files = get_download_files(key, username);

    if files.is_empty() {
        return None;
    }

    let tracks: Vec<Track> = files
        .into_iter()
        .map(|(index, path, duration)| Track {
            index,
            // The engine reads the local file. Therefore it needs no identity
            // of a file on the server.
            ino: String::new(),
            filename: std::path::Path::new(&path)
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone()),
            mime_type: None,
            size: std::fs::metadata(&path).ok().map(|data| data.len()),
            duration,
            start_offset: 0.0,
        })
        .collect();

    Some(TrackList::new(tracks, Vec::new()))
}

/// Writes a position that waits for the server.
pub fn remember_progress(
    username: &str,
    server: &str,
    id_item: &str,
    id_pod: Option<&str>,
    current_time: f64,
    duration: f64,
    is_finished: bool,
) {
    let progress = PendingProgress {
        id_item: id_item.to_string(),
        id_pod: id_pod.unwrap_or_default().to_string(),
        current_time,
        duration,
        is_finished,
        updated_at: now_ms(),
    };

    if let Err(error) = insert_pending_progress(username, server, &progress) {
        warn!(
            "[offline] the application did not keep the position: {}",
            error
        );
    } else {
        info!(
            "[offline] the position {}s of {} waits for the server",
            current_time.round(),
            id_item
        );
    }
}

/// Sends every position that waits, and removes each row that the server took.
///
/// The function gives the number of positions that the server took. It stops
/// at the first address that does not answer, because every other request then
/// fails in the same way.
pub async fn flush_pending_progress(api: &ApiClient, username: &str, server: &str) -> usize {
    let waiting = get_pending_progress(username, server);

    if waiting.is_empty() {
        return 0;
    }

    info!(
        "[offline] {} position(s) wait for the server",
        waiting.len()
    );

    let mut sent = 0;

    for progress in waiting {
        // The server can hold a newer position from a different client.
        let server_last_update = match get_book_progress(api, &progress.id_item).await {
            Ok(root) => Some(root.last_update),
            Err(error) if error.is_offline() => {
                warn!("[offline] the server does not answer: {}", error);
                return sent;
            }
            // A media that the user did not start gives 404. The application
            // then sends its position.
            Err(_) => None,
        };

        if !should_send(progress.updated_at, server_last_update) {
            info!(
                "[offline] the server has a newer position of {}. The local position goes away.",
                progress.id_item
            );
            let _ = delete_pending_progress(username, &progress.id_item, &progress.id_pod);
            continue;
        }

        let duration = (progress.duration.round() as u32).to_string();
        let position = Some(progress.current_time.round() as u32);
        let episode = if progress.id_pod.is_empty() {
            None
        } else {
            Some(progress.id_pod.as_str())
        };

        let result = match (episode, progress.is_finished) {
            (Some(episode), true) => {
                update_media_progress2_pod(
                    api,
                    &progress.id_item,
                    position,
                    &duration,
                    true,
                    episode,
                )
                .await
            }
            (Some(episode), false) => {
                update_media_progress_pod(api, &progress.id_item, position, &duration, episode)
                    .await
            }
            (None, true) => {
                update_media_progress2_book(api, &progress.id_item, position, &duration, true).await
            }
            (None, false) => {
                update_media_progress_book(api, &progress.id_item, position, &duration).await
            }
        };

        match result {
            Ok(()) => {
                let _ = delete_pending_progress(username, &progress.id_item, &progress.id_pod);
                sent += 1;

                info!(
                    "[offline] the server took the position {}s of {}",
                    progress.current_time.round(),
                    progress.id_item
                );
            }
            Err(error) if error.is_offline() => {
                warn!("[offline] the server does not answer: {}", error);
                return sent;
            }
            Err(error) => {
                // The server answered with a fault of the request. A second
                // attempt gives the same answer, thus the row goes away.
                warn!(
                    "[offline] the server refused the position of {}: {}",
                    progress.id_item, error
                );
                let _ = delete_pending_progress(username, &progress.id_item, &progress.id_pod);
            }
        }
    }

    sent
}

/// The number of seconds between two attempts of the background task.
const FLUSH_PERIOD: u64 = 30;

/// Starts a task that sends the positions when the server answers again.
///
/// The user does not need to start the application again. The task tries every
/// 30 seconds, and it does nothing when no position waits. Therefore it costs
/// no request in the normal condition.
pub fn spawn_flush_task(api: std::sync::Arc<ApiClient>, username: String, server: String) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(FLUSH_PERIOD)).await;

            // The count comes from the database. A playback that ends offline
            // writes a row, thus the task finds it without a message.
            if count_pending_progress(&username, &server) == 0 {
                continue;
            }

            // Every address has the state `Down` after the offline mode, and
            // the client then sends no request at all. The probe task gives an
            // address the state `Up` again, but it waits 60 seconds. This task
            // examines the addresses itself, thus a position goes to the
            // server as soon as the server answers.
            let pool = api.pool();

            if pool.active().is_none() {
                crate::api::client::probe::probe_once(api.http(), &pool).await;
            }

            let sent = flush_pending_progress(&api, &username, &server).await;

            if sent > 0 {
                info!(
                    "[offline] the server answers again. {} position(s) went to it.",
                    sent
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The server has no position of this media. The application sends.
    #[test]
    fn a_server_with_no_position_takes_the_local_position() {
        assert!(should_send(1000, None));
        assert!(should_send(0, None));
    }

    /// The local position is newer. The application sends.
    #[test]
    fn a_newer_local_position_goes_to_the_server() {
        assert!(should_send(2000, Some(1000)));
    }

    /// A different client wrote a newer position. The local position goes
    /// away, in the same way as T-4.
    #[test]
    fn a_newer_position_of_the_server_stays() {
        assert!(!should_send(1000, Some(2000)));
    }

    /// The two positions have the same time. The application sends, because
    /// the local position is the position of this user.
    #[test]
    fn the_same_time_gives_the_local_position() {
        assert!(should_send(1500, Some(1500)));
    }

    #[test]
    fn the_clock_gives_a_value_of_this_time() {
        // The value must be after 2020 and before 2100.
        let now = now_ms();
        assert!(now > 1_577_836_800_000);
        assert!(now < 4_102_444_800_000);
    }
}
