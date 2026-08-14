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
use crate::api::me::get_media_progress::{get_the_place_of_a_media, Root};
use crate::api::me::update_media_progress::*;
use crate::db::crud::*;
use crate::player::engine::track::{Track, TrackList};
use log::{error, info, warn};
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
/// Gives `Ok(None)` when the disk holds no file of this media, and a fault when
/// the program did not read its database: **a read that failed is not a media with
/// no file of the disk** (T-203).
pub fn tracks_from_downloads(key: &str, username: &str) -> rusqlite::Result<Option<TrackList>> {
    let files = get_download_files(key, username)?;

    if files.is_empty() {
        return Ok(None);
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

    Ok(Some(TrackList::new(tracks, Vec::new())))
}

/// Writes a position that waits for the server, and says one line of the log.
///
/// This is for the end of a playback. **The loop of an offline playback calls
/// `keep_progress` at each second**, and that function says nothing: one line
/// for each second gives 28800 lines of the log for a book of eight hours.
pub fn remember_progress(
    username: &str,
    server: &str,
    id_item: &str,
    id_pod: Option<&str>,
    current_time: f64,
    duration: f64,
    is_finished: bool,
) {
    if keep_progress(
        username,
        server,
        id_item,
        id_pod,
        current_time,
        duration,
        is_finished,
    ) {
        info!(
            "[offline] the position {}s of {} waits for the server",
            current_time.round(),
            id_item
        );
    }
}

/// Writes a position that waits for the server, and says nothing.
///
/// Gives `true` when the row is on the disk.
///
/// **An offline playback reaches no server at all**, therefore the row of this
/// table is the one copy of that playback: no row of `listening_session`
/// stands beside it, and the next program of the account has nothing else to
/// send. A program that dies writes nothing more, and a position that the loop
/// keeps at its end only is a position that such a program never kept. The
/// loop therefore calls this function at each second, in the same way that it
/// writes the place of the user to the row of the download at each second.
///
/// A newer position replaces the older one, thus the table holds one row of a
/// media whatever the number of the calls. See T-152.
pub fn keep_progress(
    username: &str,
    server: &str,
    id_item: &str,
    id_pod: Option<&str>,
    current_time: f64,
    duration: f64,
    is_finished: bool,
) -> bool {
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
        return false;
    }

    true
}

/// What the read of the position of the server tells the flush. See T-188.
#[derive(Debug, Clone, PartialEq)]
pub enum TheRead {
    /// The server holds a position of this media, and this is the moment of it.
    TheMoment(i64),
    /// The server holds no position of this media. The program sends its own.
    NoPosition,
    /// The server does not answer. Every request after it fails in the same
    /// way, therefore the flush stops here.
    TheServerIsAway(String),
    /// The program did not read the position of the server. It writes nothing,
    /// and the row of the disk waits for the next attempt.
    NoRead(String),
    /// The server holds a position, and it gave no moment of it. The program
    /// cannot say which position is the newer one.
    NoMoment,
}

/// Tells what the flush must do with the answer of the read of the position.
///
/// **The flush reads a state of the server, and it then writes it.** That is
/// the shape of T-175, and the rule of T-175 and of T-178 holds here too: **the
/// status 404 is the answer of a media that never played, and every other fault
/// stops the write.**
///
/// The old code read every fault as "the server holds no position of this
/// media", therefore it sent its own position. A measurement of 2026-08-14 with
/// `docs/harness/one_method_fails.py`, which answered `500` to
/// `GET /api/me/progress/:id` and which forwarded the `PATCH` of the same path:
/// the server held the place 5000 seconds of a book of eight hours with the
/// moment of now, the disk held the place 100 seconds with a moment of one hour
/// before it, and the flush of the start wrote 100 seconds to the server. The
/// place of the user went away, and the log said "the server took the position
/// 100s". **A fault of the read must therefore keep the row**: the row is the
/// one copy of an offline playback (T-152), and the task of the flush tries
/// again every 30 seconds.
///
/// **A moment of 0 is a moment that the server did not give.** `lastUpdate`
/// takes the default 0 (the rule of T-177), therefore an answer of a server
/// that does not hold that field made `should_send` compare the moment of the
/// disk with the moment of 1970: the program then wrote its old position over
/// every newer position of the server. A measurement of 2026-08-14 with
/// `docs/harness/a_field_of_the_answer_goes_away.py` of the field `lastUpdate`
/// gave the same 5000 seconds to 100 seconds. This is the rule of T-180 for
/// this field: a program that cannot compare the two positions writes neither
/// of them, and it keeps the row. See T-188.
pub fn the_read_of_the_position(
    answer: Result<Root, crate::api::client::error::ApiError>,
) -> TheRead {
    match answer {
        Ok(root) if root.last_update > 0 => TheRead::TheMoment(root.last_update),
        Ok(_) => TheRead::NoMoment,
        Err(error) if error.is_offline() => TheRead::TheServerIsAway(error.to_string()),
        // A media that the user did not start gives 404. The program then sends
        // its position.
        Err(crate::api::client::error::ApiError::NotFound) => TheRead::NoPosition,
        Err(error) => TheRead::NoRead(error.to_string()),
    }
}

/// Tells if a place that the server did not take can reach that server later.
///
/// **The place of the user is the value to keep.** A write of a place that came
/// back with a fault leaves the program with one question: does this place wait,
/// or does it go away? The old code of the two places of this work asked
/// `is_offline`, therefore **every** answer of the server threw the place away.
///
/// A measurement of 2026-08-14 with `docs/harness/one_path_fails.py` of the path
/// `/api/me/progress`, and a row of a listening session of a program that died at
/// 1234 seconds: `close_one_session` of the key `Q` read the status 500, it wrote
/// no row of `pending_progress`, and it then removed the row of that session. The
/// place of the server stayed 0, and the log said
/// "Item 6ba57b9a-… closed at 1234s". **The place of the user went away for
/// ever, and the words of the program said the words of a success.** See T-189.
///
/// **Two faults say that this place reaches this server never**:
///
/// - the status 404: the server does not hold this media, therefore the place of
///   it belongs to nothing (the rule of T-187), and
/// - the status 400: the server refused the request itself, and a second attempt
///   of the same request gives the same answer (the rule of T-87).
///
/// **Every other fault can pass**: a status of 500 or more is the fault of one
/// machine and a second address of the same server can answer it (T-128), a
/// token that is not valid holds until the user logs in again, a permission of
/// an account can come back (T-136), and a body that the program did not read is
/// the fault of one answer.
pub fn the_place_can_wait(fault: &crate::api::client::error::ApiError) -> bool {
    !matches!(
        fault,
        crate::api::client::error::ApiError::NotFound
            | crate::api::client::error::ApiError::Server(400)
    )
}

/// Sends every position that waits, and removes each row that the server took.
///
/// The function gives the number of positions that the server took. It stops
/// at the first address that does not answer, because every other request then
/// fails in the same way.
pub async fn flush_pending_progress(api: &ApiClient, username: &str, server: &str) -> usize {
    // **A read of the disk that failed is not a disk with no place that waits**
    // (T-203). Every place of that disk waits for the next attempt (T-189), and
    // this function runs before the first frame and in a task of every 30 seconds:
    // it holds no key of the user, therefore the fault takes a line of the log
    // (T-177 and T-188).
    let waiting = match get_pending_progress(username, server) {
        Ok(waiting) => waiting,
        Err(error) => {
            error!(
                "[offline] the program did not read the positions that wait: {}. \
                 Each of them waits for the next attempt.",
                error
            );

            return 0;
        }
    };

    if waiting.is_empty() {
        return 0;
    }

    info!(
        "[offline] {} position(s) wait for the server",
        waiting.len()
    );

    let mut sent = 0;

    for progress in waiting {
        let episode = if progress.id_pod.is_empty() {
            None
        } else {
            Some(progress.id_pod.as_str())
        };

        // The server can hold a newer position from a different client.
        //
        // **The position of an episode of a podcast stands at the path of that
        // episode** (T-182). The old code asked for the path of the item alone,
        // and Audiobookshelf answers that path with the position of **one**
        // episode of the podcast: the moment of another episode then decided
        // for this one. See T-188.
        let answer = get_the_place_of_a_media(api, &progress.id_item, episode).await;

        let server_last_update = match the_read_of_the_position(answer) {
            TheRead::TheMoment(moment) => Some(moment),
            TheRead::NoPosition => None,
            TheRead::TheServerIsAway(fault) => {
                warn!("[offline] the server does not answer: {}", fault);
                return sent;
            }
            TheRead::NoRead(fault) => {
                warn!(
                    "[offline] the program did not read the position of {} of the server: {} \
                     The position of the disk waits.",
                    progress.id_item, fault
                );
                continue;
            }
            TheRead::NoMoment => {
                warn!(
                    "[offline] the server gave no moment of its position of {}. \
                     The position of the disk waits.",
                    progress.id_item
                );
                continue;
            }
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
            Err(error) if the_place_can_wait(&error) => {
                // The fault belongs to this attempt, and not to this position:
                // the task tries again every 30 seconds. See T-189.
                warn!(
                    "[offline] the server did not take the position of {}: {} \
                     The position of the disk waits.",
                    progress.id_item, error
                );
            }
            Err(error) => {
                // The server refused the request itself, or it does not hold
                // this media. A second attempt gives the same answer, thus the
                // row goes away. See T-189.
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
///
/// The caller keeps the handle, because a login that comes again holds a new
/// token: the task of the token before it must stop. See T-123.
pub fn spawn_flush_task(
    api: std::sync::Arc<ApiClient>,
    username: String,
    server: String,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(FLUSH_PERIOD)).await;

            // The count comes from the database. A playback that ends offline
            // writes a row, thus the task finds it without a message.
            //
            // **A read that failed is not a disk with no place that waits**
            // (T-203): the flush of this attempt reads the rows again, and it names
            // the fault in the log.
            match count_pending_progress(&username, &server) {
                Ok(0) => continue,
                Ok(_) => {}
                Err(error) => {
                    warn!(
                        "[offline] the program did not count the positions that wait: {}. \
                         The task asks the disk again.",
                        error
                    );

                    continue;
                }
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A media that never played gives 404, and the program sends its
    /// position. See T-188.
    #[test]
    fn a_media_that_never_played_takes_the_local_position() {
        assert_eq!(
            the_read_of_the_position(Err(crate::api::client::error::ApiError::NotFound)),
            TheRead::NoPosition
        );
    }

    /// **A fault of the read must stop the write.** The old code read every
    /// fault as "the server holds no position", therefore the program wrote its
    /// old position over the newer position of the server: the measurement of
    /// T-188 lost 5000 seconds of a book of eight hours, and the log said "the
    /// server took the position 100s".
    #[test]
    fn a_fault_of_the_read_keeps_the_position_of_the_disk() {
        for fault in [
            crate::api::client::error::ApiError::Server(500),
            crate::api::client::error::ApiError::Forbidden,
            crate::api::client::error::ApiError::Unauthorized,
            crate::api::client::error::ApiError::Decode("no field".to_string()),
        ] {
            let text = fault.to_string();
            assert_eq!(
                the_read_of_the_position(Err(fault)),
                TheRead::NoRead(text),
                "a fault of the read must keep the row of the disk"
            );
        }
    }

    /// A server that does not answer stops the whole flush, because every
    /// request after it fails in the same way.
    #[test]
    fn a_server_that_does_not_answer_stops_the_flush() {
        assert!(matches!(
            the_read_of_the_position(Err(crate::api::client::error::ApiError::Unreachable)),
            TheRead::TheServerIsAway(_)
        ));
        assert!(matches!(
            the_read_of_the_position(Err(crate::api::client::error::ApiError::Timeout)),
            TheRead::TheServerIsAway(_)
        ));
    }

    /// **A moment of 0 is a moment that the server did not give.** `lastUpdate`
    /// takes the default 0, therefore an answer that holds no such field made
    /// the program compare the moment of the disk with the moment of 1970 and
    /// write its old position over the newer position of the server. See T-188
    /// and T-180.
    #[test]
    fn a_position_with_no_moment_keeps_the_position_of_the_disk() {
        let of_the_server = Root {
            last_update: 0,
            ..Default::default()
        };

        assert_eq!(
            the_read_of_the_position(Ok(of_the_server)),
            TheRead::NoMoment
        );
    }

    /// The server gave the moment of its position, and the flush compares it.
    #[test]
    fn a_position_of_the_server_gives_its_moment() {
        let of_the_server = Root {
            last_update: 1700,
            ..Default::default()
        };

        assert_eq!(
            the_read_of_the_position(Ok(of_the_server)),
            TheRead::TheMoment(1700)
        );
    }

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
