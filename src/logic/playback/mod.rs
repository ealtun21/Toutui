//! The one playback loop of the application.
//!
//! The engine plays the audio. This module starts a session on the server,
//! gives the work to the engine, and writes the progress.
//!
//! Before this module, four files held a loop that was almost the same. The
//! engine reads a local file and a file on the server with one trait.
//! Therefore one loop is enough now.

use crate::api::client::ApiClient;
use crate::api::library_items::play_lib_item_or_pod::*;
use crate::api::me::update_media_progress::*;
use crate::api::sessions::close_open_session::*;
use crate::api::sessions::sync_open_session::*;
use crate::db::crud::*;
use crate::logic::offline::{remember_progress, tracks_from_downloads};
use crate::logic::sync_session::sync_session_from_database::*;
use crate::logic::sync_session::wait_prev_session_finished::*;
use crate::player::engine::source::{select_sources, TrackSource};
use crate::player::engine::track::{Chapter, Track, TrackList};
use crate::player::engine::{PlaybackRequest, PlaybackStatus, PlayerCommand, PlayerHandle};
use crate::utils::pop_up_message::*;
use log::{error, info, warn};
use std::io::stdout;

/// The number of seconds between two sync requests to the server.
const SYNC_PERIOD: u64 = 10;

/// What the user selected.
#[derive(Debug, Clone)]
pub enum PlaybackTarget {
    /// A book of the library.
    Book {
        item_id: String,
        /// The length of the whole book, from the field `media.duration`.
        whole_book_duration: Option<f64>,
    },
    /// One episode of a podcast.
    Episode { item_id: String, episode_id: String },
}

impl PlaybackTarget {
    /// Gives the identity of the library item.
    pub fn item_id(&self) -> &str {
        match self {
            PlaybackTarget::Book { item_id, .. } => item_id,
            PlaybackTarget::Episode { item_id, .. } => item_id,
        }
    }

    /// Gives the identity of the episode. A book has no episode.
    pub fn episode_id(&self) -> Option<&str> {
        match self {
            PlaybackTarget::Book { .. } => None,
            PlaybackTarget::Episode { episode_id, .. } => Some(episode_id),
        }
    }
}

/// Reads the chapters of a book from the answer of the server.
fn chapters_from(item: &serde_json::Value) -> Vec<Chapter> {
    item["media"]["chapters"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|chapter| Chapter {
                    start: chapter["start"].as_f64().unwrap_or(0.0),
                    end: chapter["end"].as_f64().unwrap_or(0.0),
                    title: chapter["title"].as_str().unwrap_or_default().to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Makes a track from one element of `media.audioFiles`.
fn track_from(file: &serde_json::Value, index: u32) -> Track {
    Track {
        index,
        ino: file["ino"].as_str().unwrap_or_default().to_string(),
        filename: file["metadata"]["filename"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        mime_type: file["mimeType"].as_str().map(|value| value.to_string()),
        size: file["metadata"]["size"].as_u64(),
        duration: file["duration"].as_f64().unwrap_or(0.0),
        start_offset: 0.0,
    }
}

/// Makes the track list of a book from the answer of `GET /api/items/:id`.
///
/// The function puts the files in the sequence of the field `index`. A book
/// with many audio files then plays in the correct sequence. See T-2.
///
/// Gives `None` if the book has no audio file.
pub fn tracks_from_item(item: &serde_json::Value) -> Option<TrackList> {
    let files = item["media"]["audioFiles"].as_array()?;

    if files.is_empty() {
        return None;
    }

    let mut tracks: Vec<Track> = files
        .iter()
        .map(|file| {
            let index = file["index"].as_u64().unwrap_or(1) as u32;
            track_from(file, index)
        })
        .collect();

    tracks.sort_by_key(|track| track.index);

    Some(TrackList::new(tracks, chapters_from(item)))
}

/// Makes the track list of one podcast episode.
///
/// An episode has one audio file. Therefore the list holds one track.
pub fn tracks_from_episode(item: &serde_json::Value, episode_id: &str) -> Option<TrackList> {
    let episodes = item["media"]["episodes"].as_array()?;

    let episode = episodes
        .iter()
        .find(|episode| episode["id"].as_str() == Some(episode_id))?;

    let file = &episode["audioFile"];

    if file.is_null() {
        return None;
    }

    let mut track = track_from(file, 1);

    // The episode holds the length. The audio file does not always hold it.
    if track.duration <= 0.0 {
        track.duration = episode["duration"].as_f64().unwrap_or(0.0);
    }

    Some(TrackList::new(vec![track], Vec::new()))
}

/// Starts a book or an episode, and follows the playback to the end.
///
/// The function does this sequence:
///
/// 1. It stops the media that plays now.
/// 2. It closes the session before this session.
/// 3. It opens a session on the server, and it reads the position.
/// 4. It reads the audio files and the chapters.
/// 5. It gives the work to the engine.
/// 6. It follows the playback, and it writes the progress.
pub async fn play(
    api: &ApiClient,
    player: &PlayerHandle,
    target: PlaybackTarget,
    username: String,
    server_address: String,
    server_key: String,
) {
    // The engine stops the media that plays now. There is no separate
    // program, thus the application does not stop a process.
    player.send(PlayerCommand::Stop);

    wait_prev_session_finished(username.clone());

    let mut stdout = stdout();
    let _ = pop_message(&mut stdout, 3, "Loading the media...");

    // If the application stopped without a correct exit, close the last
    // session now.
    sync_session_from_database(api, username.clone(), server_key.clone(), false, "l").await;

    let item_id = target.item_id().to_string();

    // Open the session. The answer gives the position that the server holds.
    let info_item = match target.episode_id() {
        Some(episode_id) => post_start_playback_session_pod(api, &item_id, episode_id).await,
        None => post_start_playback_session_book(api, &item_id).await,
    };

    let info_item = match info_item {
        Ok(value) => value,
        // The server does not answer. A copy on the disk still plays. See
        // T-25.
        Err(error) if error.is_offline() => {
            warn!(
                "[play] the server does not answer: {}. The offline mode starts.",
                error
            );
            play_offline(player, &target, username, server_key, &mut stdout).await;
            return;
        }
        Err(error) => {
            error!("[play] the server did not start the session: {}", error);
            let _ = clear_message(&mut stdout, 3);
            return;
        }
    };

    let start_position = info_item[0].parse::<f64>().unwrap_or(0.0);
    let session_id = info_item[3].clone();

    // Read the audio files and the chapters.
    let item: serde_json::Value = match api.get_json(&format!("/api/items/{}", item_id)).await {
        Ok(value) => value,
        Err(error) => {
            error!("[play] the server did not give the item: {}", error);
            let _ = clear_message(&mut stdout, 3);
            return;
        }
    };

    let tracks = match target.episode_id() {
        Some(episode_id) => tracks_from_episode(&item, episode_id),
        None => tracks_from_item(&item),
    };

    let tracks = match tracks {
        Some(tracks) => tracks,
        None => {
            error!("[play] the item has no audio file");
            let _ = clear_message(&mut stdout, 3);
            return;
        }
    };

    // The playback session gives the length of the first audio file only. Use
    // the length of the whole book. See T-2 and upstream issue 33.
    let total_duration = total_duration_of(&target, &tracks, &info_item[2]);

    let track_list: Vec<Track> = (0..tracks.len())
        .filter_map(|index| tracks.get(index).cloned())
        .collect();

    // The download of an episode has the identity of the episode. The download
    // of a book has the identity of the item.
    let download_key = target.episode_id().unwrap_or(&item_id);

    let sources = select_sources(
        download_key,
        &item_id,
        &username,
        &server_address,
        &track_list,
    );

    let local = sources
        .iter()
        .filter(|source| matches!(source, TrackSource::Local(_)))
        .count();

    info!(
        "[play] the download {} gives {} of {} track(s) from the disk",
        download_key,
        local,
        sources.len()
    );
    let speed = get_speed_rate(&username).parse::<f32>().unwrap_or(1.0);

    let request = PlaybackRequest {
        item_id: item_id.clone(),
        title: info_item[4].clone(),
        author: info_item[6].clone(),
        username: username.clone(),
        tracks,
        sources,
        start_position,
        speed,
    };

    let _ = insert_listening_session(
        session_id.clone(),
        item_id.clone(),
        start_position.round() as u32,
        total_duration.clone(),
        target.episode_id().unwrap_or_default().to_string(),
        0,
        request.title.clone(),
        request.author.clone(),
        true,
        String::new(),
    );

    info!(
        "[play] the item {} starts at {} seconds with {} tracks",
        item_id,
        start_position,
        request.tracks.len()
    );

    player.send(PlayerCommand::Start(Box::new(request)));

    let _ = clear_message(&mut stdout, 3);

    follow_playback(
        api,
        player,
        session_id,
        item_id,
        target.episode_id().map(|value| value.to_string()),
        username,
        total_duration,
    )
    .await;
}

/// Plays a local copy when the server does not answer.
///
/// The function needs no session on the server. It reads the files, the
/// length, and the position from the database. It writes the position to the
/// database, and it keeps that position for the server. See T-25.
async fn play_offline(
    player: &PlayerHandle,
    target: &PlaybackTarget,
    username: String,
    server: String,
    stdout: &mut std::io::Stdout,
) {
    let selected = target.item_id().to_string();

    // The download of an episode has the identity of the episode.
    let key = target.episode_id().unwrap_or(&selected).to_string();

    let Some(row) = get_download_row(&key, &username) else {
        error!("[play] the disk has no copy of {}", key);
        let _ = pop_message(
            stdout,
            3,
            "The server does not answer, and the disk has no copy of this media.",
        );
        return;
    };

    let Some(tracks) = tracks_from_downloads(&key, &username) else {
        error!("[play] the disk has no audio file of {}", key);
        let _ = pop_message(
            stdout,
            3,
            "The server does not answer, and the disk has no audio file of this media.",
        );
        return;
    };

    let track_list: Vec<Track> = (0..tracks.len())
        .filter_map(|index| tracks.get(index).cloned())
        .collect();

    let sources: Vec<TrackSource> = track_list
        .iter()
        .filter_map(|track| {
            get_download_files(&key, &username)
                .into_iter()
                .find(|(index, _, _)| *index == track.index)
                .map(|(_, path, _)| TrackSource::Local(std::path::PathBuf::from(path)))
        })
        .collect();

    if sources.len() != track_list.len() {
        error!("[play] the disk does not hold every file of {}", key);
        let _ = pop_message(
            stdout,
            3,
            "The disk does not hold every file of this media.",
        );
        return;
    }

    // The row holds the identity of the library item. The key of an episode is
    // the identity of the episode, and the server needs both values. Therefore
    // the offline mode reads them from the row, and not from the view.
    let item_id = if row.item_id.is_empty() {
        key.clone()
    } else {
        row.item_id.clone()
    };

    let episode_id = if item_id == key {
        None
    } else {
        Some(key.clone())
    };

    // The length of the download has more importance, because the tracks of a
    // book with one file give the same value.
    let total_duration = if row.duration > 0.0 {
        row.duration
    } else {
        tracks.total_duration()
    };

    let speed = get_speed_rate(&username).parse::<f32>().unwrap_or(1.0);

    let request = PlaybackRequest {
        item_id: item_id.clone(),
        title: row.title.clone(),
        author: row.author.clone(),
        username: username.clone(),
        tracks,
        sources,
        start_position: row.current_time as f64,
        speed,
    };

    info!(
        "[play] the offline mode plays {} at {} seconds with {} track(s)",
        row.title,
        row.current_time,
        request.tracks.len()
    );

    player.send(PlayerCommand::Start(Box::new(request)));

    let _ = clear_message(stdout, 3);
    let _ = pop_message(
        stdout,
        3,
        &format!("Offline: \"{}\" plays from the disk.", row.title),
    );

    follow_playback_offline(
        player,
        key,
        item_id,
        episode_id,
        username,
        server,
        total_duration,
    )
    .await;
}

/// Follows a playback that has no server.
///
/// The loop writes the position in the database for each second. When the
/// playback stops, the loop keeps the position for the server. The application
/// sends it when the server answers again.
#[allow(clippy::too_many_arguments)]
async fn follow_playback_offline(
    player: &PlayerHandle,
    key: String,
    item_id: String,
    episode_id: Option<String>,
    username: String,
    server: String,
    total_duration: f64,
) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let state = player.state();
        let position = state.position.max(0.0) as u32;

        let _ = update_download_current_time(key.as_str(), username.as_str(), position);

        if state.status == PlaybackStatus::Stopped {
            let finished = state.finished;

            info!(
                "[follow_playback_offline] the playback stopped at {} seconds, finished={}",
                position, finished
            );

            remember_progress(
                &username,
                &server,
                &item_id,
                episode_id.as_deref(),
                position as f64,
                total_duration,
                finished,
            );

            let _ = update_is_loop_break("1", username.as_str());
            return;
        }
    }
}

/// Gives the length that the application reports to the server.
///
/// A book uses the length of the whole book. The playback session gives the
/// length of the first audio file only, and a book with many audio files then
/// gets a value that is far too small. See upstream issue 33.
fn total_duration_of(
    target: &PlaybackTarget,
    tracks: &TrackList,
    session_duration: &str,
) -> String {
    if let PlaybackTarget::Book {
        whole_book_duration: Some(duration),
        ..
    } = target
    {
        if duration.is_finite() && *duration > 0.0 {
            return (duration.round() as u32).to_string();
        }
    }

    let total = tracks.total_duration();

    if total > 0.0 {
        return (total.round() as u32).to_string();
    }

    session_duration.to_string()
}

/// Follows the playback, and writes the progress.
///
/// The loop reads the state of the engine one time each second. It writes the
/// position in the database for each read operation, because a crash must not
/// lose the position. It sends the progress to the server every ten seconds.
///
/// The loop sends `/sync` only during the playback. It does not send
/// `/progress`. Two requests at the same time can make a race condition, and
/// then the item stays in "continue listening". See upstream issue 35.
#[allow(clippy::too_many_arguments)]
pub async fn follow_playback(
    api: &ApiClient,
    player: &PlayerHandle,
    session_id: String,
    item_id: String,
    episode_id: Option<String>,
    username: String,
    total_duration: String,
) {
    let mut since_sync: u64 = 0;
    let mut last_position: u32 = 0;
    let mut was_stalled = false;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let state = player.state();
        let position = state.position.max(0.0) as u32;

        // Write the position for each second. A crash must not lose it.
        let _ = update_current_time(position, session_id.as_str());
        let _ = update_download_current_time(item_id.as_str(), username.as_str(), position);

        if let Some(title) = state.chapter_title.as_ref() {
            let _ = update_chapter(title, session_id.as_str());
        }

        match state.status {
            PlaybackStatus::Playing => {
                if was_stalled {
                    info!("[follow_playback] the data comes again");
                    was_stalled = false;
                }

                let moved = position.saturating_sub(last_position);
                since_sync += 1;

                if since_sync >= SYNC_PERIOD {
                    if let Err(error) = sync_session(api, &session_id, Some(position), moved).await
                    {
                        warn!(
                            "[follow_playback] the server did not accept the sync: {}",
                            error
                        );
                    }

                    let _ = update_elapsed_time(moved, session_id.as_str());
                    since_sync = 0;
                }

                last_position = position;
            }

            // The engine waits for data. The playback continues without an
            // action of the user. Therefore the loop does not stop.
            PlaybackStatus::Stalled => {
                if !was_stalled {
                    warn!("[follow_playback] the data stopped. The engine waits.");
                    was_stalled = true;
                }
            }

            PlaybackStatus::Paused => {
                since_sync = 0;
                last_position = position;
            }

            PlaybackStatus::Stopped => {
                let finished = state.finished;

                info!(
                    "[follow_playback] the playback stopped at {} seconds, finished={}",
                    position, finished
                );

                let _ = update_is_finished(if finished { "1" } else { "0" }, session_id.as_str());

                if let Err(error) = close_session_without_send_prg_data(api, &session_id).await {
                    warn!(
                        "[follow_playback] the server did not close the session: {}",
                        error
                    );
                }

                // The media came to the end, or the user stopped it. This is a
                // command of the user, and it is not a report during the
                // playback. Therefore `/progress` is correct here. See
                // upstream issue 35.
                //
                // If the media came to its end, the request must also mark
                // the item as finished. See T-16.
                let result = match (episode_id.as_deref(), finished) {
                    (Some(episode_id), true) => {
                        update_media_progress2_pod(
                            api,
                            &item_id,
                            Some(position),
                            &total_duration,
                            true,
                            episode_id,
                        )
                        .await
                    }
                    (Some(episode_id), false) => {
                        update_media_progress_pod(
                            api,
                            &item_id,
                            Some(position),
                            &total_duration,
                            episode_id,
                        )
                        .await
                    }
                    (None, true) => {
                        update_media_progress2_book(
                            api,
                            &item_id,
                            Some(position),
                            &total_duration,
                            true,
                        )
                        .await
                    }
                    (None, false) => {
                        update_media_progress_book(api, &item_id, Some(position), &total_duration)
                            .await
                    }
                };

                if let Err(error) = result {
                    warn!(
                        "[follow_playback] the server did not accept the position: {}",
                        error
                    );
                }

                let _ = update_is_loop_break("1", username.as_str());
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> serde_json::Value {
        serde_json::json!({
            "media": {
                "duration": 60.0,
                "audioFiles": [
                    {
                        "index": 2, "ino": "222", "duration": 50.0,
                        "mimeType": "audio/mpeg",
                        "metadata": { "filename": "part2.mp3", "size": 5000 }
                    },
                    {
                        "index": 1, "ino": "111", "duration": 10.0,
                        "mimeType": "audio/mpeg",
                        "metadata": { "filename": "part1.mp3", "size": 1000 }
                    }
                ],
                "chapters": [
                    { "start": 0.0, "end": 25.0, "title": "One" },
                    { "start": 25.0, "end": 60.0, "title": "Two" }
                ]
            }
        })
    }

    /// The files can come in any sequence in the answer. The function must
    /// sort them by the field `index`. This is the correction of T-2.
    #[test]
    fn the_function_sorts_the_files_by_index() {
        let tracks = tracks_from_item(&book()).unwrap();

        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks.get(0).unwrap().ino, "111");
        assert_eq!(tracks.get(1).unwrap().ino, "222");
        assert_eq!(tracks.get(1).unwrap().start_offset, 10.0);
    }

    #[test]
    fn the_total_duration_is_the_length_of_the_whole_book() {
        assert_eq!(tracks_from_item(&book()).unwrap().total_duration(), 60.0);
    }

    #[test]
    fn the_track_holds_the_type_and_the_size() {
        let tracks = tracks_from_item(&book()).unwrap();
        let first = tracks.get(0).unwrap();

        assert_eq!(first.mime_type.as_deref(), Some("audio/mpeg"));
        assert_eq!(first.size, Some(1000));
    }

    #[test]
    fn the_book_holds_the_chapters() {
        let tracks = tracks_from_item(&book()).unwrap();
        assert_eq!(tracks.chapter_at(30.0).unwrap().title, "Two");
    }

    #[test]
    fn a_book_with_no_audio_file_gives_no_track_list() {
        let item = serde_json::json!({ "media": { "audioFiles": [] } });
        assert!(tracks_from_item(&item).is_none());
    }

    /// 118 books of the test library have no chapter.
    #[test]
    fn a_book_with_no_chapter_gives_a_track_list() {
        let item = serde_json::json!({
            "media": { "audioFiles": [
                { "index": 1, "ino": "111", "duration": 10.0,
                  "metadata": { "filename": "a.mp3" } }
            ]}
        });

        let tracks = tracks_from_item(&item).unwrap();
        assert!(tracks.chapter_at(5.0).is_none());
    }

    fn podcast() -> serde_json::Value {
        serde_json::json!({
            "media": {
                "episodes": [
                    {
                        "id": "ep1", "duration": 120.0,
                        "audioFile": {
                            "ino": "900", "duration": 120.0,
                            "mimeType": "audio/mpeg",
                            "metadata": { "filename": "ep1.mp3", "size": 900 }
                        }
                    },
                    {
                        "id": "ep2", "duration": 240.0,
                        "audioFile": {
                            "ino": "901", "duration": 0.0,
                            "metadata": { "filename": "ep2.mp3" }
                        }
                    }
                ]
            }
        })
    }

    #[test]
    fn an_episode_gives_one_track() {
        let tracks = tracks_from_episode(&podcast(), "ep1").unwrap();

        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks.get(0).unwrap().ino, "900");
        assert_eq!(tracks.total_duration(), 120.0);
    }

    /// The audio file does not always hold the length. The episode holds it.
    #[test]
    fn an_episode_takes_the_length_from_the_episode() {
        let tracks = tracks_from_episode(&podcast(), "ep2").unwrap();
        assert_eq!(tracks.total_duration(), 240.0);
    }

    #[test]
    fn an_episode_that_is_absent_gives_no_track_list() {
        assert!(tracks_from_episode(&podcast(), "ep9").is_none());
    }

    #[test]
    fn the_whole_book_duration_replaces_the_session_duration() {
        let target = PlaybackTarget::Book {
            item_id: "a".to_string(),
            whole_book_duration: Some(53764.0),
        };
        let tracks = tracks_from_item(&book()).unwrap();

        assert_eq!(total_duration_of(&target, &tracks, "841"), "53764");
    }

    /// The server gave no length for the whole book. The sum of the tracks is
    /// still better than the length of the first file.
    #[test]
    fn the_sum_of_the_tracks_replaces_the_session_duration() {
        let target = PlaybackTarget::Book {
            item_id: "a".to_string(),
            whole_book_duration: None,
        };
        let tracks = tracks_from_item(&book()).unwrap();

        assert_eq!(total_duration_of(&target, &tracks, "841"), "60");
    }

    #[test]
    fn a_length_that_is_not_valid_falls_back_to_the_session() {
        let target = PlaybackTarget::Book {
            item_id: "a".to_string(),
            whole_book_duration: Some(f64::NAN),
        };
        let empty = TrackList::new(Vec::new(), Vec::new());

        assert_eq!(total_duration_of(&target, &empty, "841"), "841");
    }
}
