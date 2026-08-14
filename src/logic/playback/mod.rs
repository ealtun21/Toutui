//! The one playback loop of the application.
//!
//! The engine plays the audio. This module starts a session on the server,
//! gives the work to the engine, and writes the progress.
//!
//! Before this module, four files held a loop that was almost the same. The
//! engine reads a local file and a file on the server with one trait.
//! Therefore one loop is enough now.

pub mod the_place_of_the_disk;

use crate::api::client::ApiClient;
use crate::api::library_items::play_lib_item_or_pod::*;
use crate::api::me::get_media_progress::get_the_place_of_a_media;
use crate::api::me::update_media_progress::*;
use crate::api::sessions::close_open_session::*;
use crate::api::sessions::sync_open_session::*;
use crate::db::crud::*;
use crate::logic::offline::{remember_progress, tracks_from_downloads};
use crate::logic::queue::{self, the_media_goes_back_to_the_queue, the_queue_goes_on, Outcome};
use crate::logic::sync_session::force_sync;
use crate::logic::sync_session::sync_session_from_database::*;
use crate::logic::sync_session::the_rows_that_the_disk_kept::{
    the_row_of_a_closed_session_goes_away, ThePlaceOfTheSession,
};
use crate::logic::sync_session::wait_prev_session_finished::*;
use crate::logic::the_files_of_a_media::the_numbers_of_the_files;
use crate::logic::the_playback::{
    the_place_of_a_media_that_never_played, the_start_of_a_playback,
    the_words_of_a_playback_that_did_not_start, TheStartOfAPlayback, WhyNot,
};
use crate::player::engine::source::{select_sources, TrackSource};
use crate::player::engine::track::{Chapter, Track, TrackList};
use crate::player::engine::{
    next_playback_id, PlaybackRequest, PlaybackStatus, PlayerCommand, PlayerHandle,
};
use log::{error, info, warn};
use std::time::Duration;

/// The number of seconds between two sync requests to the server.
const SYNC_PERIOD: u64 = 10;

/// The number of seconds that a loop waits for the engine.
///
/// The engine opens the first audio file before it plays, and that file can
/// come from the server. Therefore the engine does not always start
/// immediately. If the engine does not start the playback in this time, the
/// loop closes the session that it opened. A session that stays open is the
/// report `dd9a649`.
const START_TIME_LIMIT: u64 = 30;

/// The sentence of a playback of the disk that the database of the program
/// stopped. See T-203.
///
/// **The words name the thing that failed** (T-91 and T-199). The three faults of
/// the offline playback said "the disk has no copy of this media" and "the disk
/// does not hold every file of this media" for a media of the disk that the program
/// did not read, and a second Toutui of this account makes that condition (T-140).
const THE_DATABASE_OF_THE_PROGRAM_SAID_NOTHING: &str =
    "The program did not read the copy of the disk in its database. \
     Stop a second Toutui, and press the key again.";

/// What the program says for a playback of the disk whose last place reached no
/// machine. See T-212.
///
/// **The place of an offline playback stands in the table of the places that
/// wait, and in no other place** (T-152): a write of it that failed takes that
/// place away for ever, therefore the user reads one word of it. The sentence
/// names the thing that failed, the work that the program did not do, and the key
/// of that work (T-91 and T-170).
pub const THE_DISK_KEPT_NO_PLACE: &str =
    "The program did not write the place of this media: the database did not answer. \
     The server gets that place never. Play the media again to set your place.";

/// The speed of the playback of an account, and 1.00x for a disk that did not
/// answer. See T-209.
///
/// **A read of the disk that failed is not a speed that the user chose.** The
/// two starts of a playback wrote `get_speed_rate(&username).parse::<f32>()
/// .unwrap_or(1.0)`, therefore every fault of the disk gave the media the speed
/// 1.00x with no word of the screen and no line of the log: the measurement of
/// 2026-08-14 held **1.5** on the disk of the account and `Speed: 1.00x` in the
/// row of the player.
///
/// The playback goes on, because the user pressed a key to hear a media and a
/// speed is not that media. **The program says which speed it plays and why**,
/// and the key of the user waits for this answer (T-199).
fn the_speed_of_this_playback(username: &str) -> f32 {
    match crate::db::crud::get_speed_rate(username) {
        Ok(speed) => speed,
        Err(error) => {
            error!(
                "[play] the program did not read the speed of {}: {}",
                username, error
            );
            crate::logic::message::say(THE_SPEED_OF_THE_DISK_DID_NOT_COME);
            1.0
        }
    }
}

/// What the program says for a media that plays at a speed of no account. See
/// T-209.
///
/// The sentence names the work that the program did (the media plays), the
/// value that it used, and the key of that work (T-79 and T-170). It says no
/// reason that the program does not have (T-91): the disk did not answer, and
/// the program does not know whether the user chose another speed.
pub const THE_SPEED_OF_THE_DISK_DID_NOT_COME: &str =
    "The program did not read the speed of this account: the database did not answer. \
     This media plays at 1.00x. Press O or I to set the speed again.";

/// What the user selected.
#[derive(Debug, Clone, PartialEq)]
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
/// **A book of a server that gave no `index` of every file takes the sequence of
/// the answer** (T-181). The old rule gave such a file the number 1, and two
/// files of one book then held the same number: the sequence of the book
/// changed, and the sources of the disk of `sources_from` took one file for two
/// tracks.
///
/// Gives `None` if the book has no audio file.
pub fn tracks_from_item(item: &serde_json::Value) -> Option<TrackList> {
    let files = item["media"]["audioFiles"].as_array()?;

    if files.is_empty() {
        return None;
    }

    let mut tracks: Vec<Track> = files
        .iter()
        .zip(the_numbers_of_the_files(files))
        .map(|(file, index)| track_from(file, index))
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

/// Makes the tracks of a playback from the answer of `GET /api/items/:id`.
///
/// **A length of 0 of an audio file is a length that the server did not give.**
/// The session of the playback holds the length of the media, and the target of
/// the library holds it too. A book of one file holds the whole media in that
/// file, therefore that length is the length of the file. See T-180.
///
/// The function is pure, therefore a test examines it with the answer of the
/// measurement and with no server.
pub fn the_tracks_of_the_playback(
    item: &serde_json::Value,
    target: &PlaybackTarget,
    the_length_of_the_session: &str,
) -> Option<TrackList> {
    let tracks = match target.episode_id() {
        Some(episode_id) => tracks_from_episode(item, episode_id),
        None => tracks_from_item(item),
    }?;

    let of_the_session = the_length_of_the_session.parse::<f64>().unwrap_or(0.0);

    let of_the_media = if of_the_session > 0.0 {
        of_the_session
    } else {
        match target {
            PlaybackTarget::Book {
                whole_book_duration: Some(duration),
                ..
            } => *duration,
            _ => 0.0,
        }
    };

    Some(tracks.the_length_of_the_media(of_the_media))
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
///
/// The function always releases the wait of the next playback.
/// `wait_prev_session_finished` waits while `is_loop_break` is not `1`, and it
/// gives that value `0` before the playback begins.
///
/// `play_media` comes back in five places without a playback: a server that
/// gives an error, an item that the server does not give, an item with no audio
/// file, and two conditions of the offline mode. The old code gave the value
/// `1` in the two loops only. Therefore the next playback waited for ever, and
/// the screen held the message "Syncing your last listening session. Please
/// wait...". A measurement on 2026-08-10 ran `play` against a server that gave
/// the answer 500, and the value stayed `0`.
///
/// One place owns this value now. `tests/playback_wait_flag.rs` holds the rule.
///
/// # The queue
///
/// A media that comes to its end gives the engine to the next media of the
/// queue. The function therefore does not come back after one media: it takes
/// the next media of the queue and it plays that media in the same task. See
/// `crate::logic::queue`.
///
/// A media that the user stopped, and a media that a different playback took
/// away, leave the queue where it is. `the_queue_goes_on` holds that rule.
///
/// **A playback that did not start gives its media back to the queue** when that
/// media came of the queue. `the_media_goes_back_to_the_queue` holds that rule.
/// See T-146.
pub async fn play(
    api: &ApiClient,
    player: &PlayerHandle,
    target: PlaybackTarget,
    username: String,
    server_address: String,
    server_key: String,
) {
    the_loop_of_the_playback(
        api,
        player,
        target,
        None,
        username,
        server_address,
        server_key,
    )
    .await
}

/// Plays a media that the queue holds, and it follows the queue after it.
///
/// The key `l` of the view of the queue takes the media out of the queue before
/// the playback, therefore a playback that does not start must give it back.
/// This function takes the whole entry for that reason, and `play` takes the
/// target alone. See T-146.
pub async fn play_the_media_of_the_queue(
    api: &ApiClient,
    player: &PlayerHandle,
    entry: queue::Entry,
    username: String,
    server_address: String,
    server_key: String,
) {
    let target = entry.target.clone();

    the_loop_of_the_playback(
        api,
        player,
        target,
        Some(entry),
        username,
        server_address,
        server_key,
    )
    .await
}

/// The loop that plays one media and then every media of the queue after it.
///
/// `the_media_of_the_queue` holds the entry of the queue that the playback of
/// now plays, and it holds nothing for a media that a key of a view started.
/// **The entry goes back to the queue when the playback does not start.**
#[allow(clippy::too_many_arguments)]
async fn the_loop_of_the_playback(
    api: &ApiClient,
    player: &PlayerHandle,
    target: PlaybackTarget,
    the_media_of_the_queue: Option<queue::Entry>,
    username: String,
    server_address: String,
    server_key: String,
) {
    let mut target = target;
    let mut the_media_of_the_queue = the_media_of_the_queue;

    loop {
        let outcome = play_media(
            api,
            player,
            target,
            username.clone(),
            server_address.clone(),
            server_key.clone(),
        )
        .await;

        // The next media of the queue opens its own session. Therefore this
        // playback must release the wait before that media starts.
        //
        // **The mark of this program comes first** (T-207): the row of the disk
        // was the one answer of that wait, and a disk that takes no write held
        // the user for the whole 30 seconds of `THE_LONGEST_WAIT` at the key `l`
        // after this loop.
        the_loop_of_this_program_ended();

        if let Err(error) = update_is_loop_break("1", username.as_str()) {
            error!(
                "[play] the disk did not take the end of the loop of the playback of {}: {}. The \
                 playback after this one reads the mark of this program.",
                username, error
            );
        }

        // The playback did not start, and the media came of the queue: the
        // media goes back to the front of the queue. A media of the user must
        // not go away with a server that does not answer. See T-146.
        if the_media_goes_back_to_the_queue(outcome) {
            if let Some(entry) = the_media_of_the_queue.take() {
                let of_the_media = entry.title.clone();

                warn!(
                    "[play] the playback of \"{}\" did not start. The media goes \
                     back to the front of the queue, and the queue stops.",
                    of_the_media
                );

                // **The disk is the truth of the queue** (T-147 and T-202). A
                // disk that says nothing takes this media of the queue away for
                // ever, therefore the log names it: the queue of the disk holds
                // every media of the account, and no view holds this fault.
                //
                // **A read and a write are two conditions** (T-206), and the
                // line names the one that happened.
                if let Err(fault) = queue::put_at_the_front(entry) {
                    error!(
                        "[play] the program did not {} the queue of its disk, therefore \"{}\" \
                         went out of the queue.",
                        queue::the_word_of_the_work_of_the_disk(fault),
                        of_the_media
                    );
                }
            }
        }

        if !the_queue_goes_on(outcome) {
            return;
        }

        // **A disk that did not answer stops the queue** (T-202 and T-206): the
        // media of the user stays on the disk, and no media goes away. The line
        // of the log names the road, because no view of the user holds this
        // fault (T-177).
        let entry = match queue::take_next() {
            Ok(Some(entry)) => entry,
            Ok(None) => return,
            Err(fault) => {
                error!(
                    "[play] the program did not {} the queue of its disk, therefore the queue \
                     stops. Every media of the queue stays on the disk.",
                    queue::the_word_of_the_work_of_the_disk(fault)
                );

                return;
            }
        };

        info!(
            "[play] the media came to its end. The queue starts \"{}\", and {} \
             media wait.",
            entry.title,
            queue::len()
        );

        crate::logic::message::say(&format!("The queue starts \"{}\".", entry.title));

        target = entry.target.clone();
        the_media_of_the_queue = Some(entry);
    }
}

/// Writes one line of the log while a write of a loop of a playback fails. See
/// T-207.
///
/// **A loop of a playback writes the disk each second**, therefore a line of the
/// log of each fault gives one line each second while the disk says nothing —
/// that is the shape of T-203 for a read of the render. The line comes one time,
/// and the next write that the disk takes gives the line back for the fault after
/// it.
///
/// The function gives `true` when the disk did not take this write. **The row of
/// the player holds the word of that condition** (T-210): a line of the log
/// reaches no user of a screen, and a message of the program lives six seconds
/// while this condition stands for the whole playback.
fn the_line_of_a_write_of_the_loop(
    said_already: &mut bool,
    what: &str,
    answer: Option<rusqlite::Result<()>>,
) -> bool {
    match answer {
        Some(Err(error)) => {
            if !*said_already {
                *said_already = true;

                error!(
                    "[follow_playback] the disk did not take {}: {}. The loop of the playback \
                     goes on, and it writes this line one time.",
                    what, error
                );
            }

            true
        }

        _ => {
            *said_already = false;
            false
        }
    }
}

/// Says why a playback did not start. See T-167.
///
/// **The answer of a key belongs to no view** (T-164): the user pressed `l` in
/// the Home view, in the view of the episodes, in the view of the queue, or in
/// a different view, and they must read the answer at once. The media of the
/// queue that a rule of the loop starts writes `The queue starts "…"` to the
/// same slot, therefore this text keeps that shape.
fn say_why_the_playback_did_not_start(why: WhyNot<'_>) {
    crate::logic::message::say(the_words_of_a_playback_that_did_not_start(why).as_str());
}

/// Starts a media, and follows the playback to the end. `play` calls this
/// function.
async fn play_media(
    api: &ApiClient,
    player: &PlayerHandle,
    target: PlaybackTarget,
    username: String,
    server_address: String,
    server_key: String,
) -> Outcome {
    // The engine stops the media that plays now. There is no separate
    // program, thus the application does not stop a process.
    player.send(PlayerCommand::Stop);

    wait_prev_session_finished(username.clone());

    crate::logic::message::say("Loading the media...");

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
            return play_offline(player, &target, username, server_key).await;
        }
        Err(error) => {
            error!("[play] the server did not start the session: {}", error);
            // **The user pressed a key, and the program must answer it.** The
            // message "Loading the media..." above went away after its six
            // seconds, and no media played: the user read nothing at all. See
            // T-167.
            say_why_the_playback_did_not_start(WhyNot::TheSessionDidNotOpen(
                error.to_string().as_str(),
            ));
            return Outcome::Fault;
        }
    };

    let session_id = info_item[3].clone();

    // **The answer of the session holds the identity of that session and the
    // place of the user, and a server can give neither of them** (T-182).
    // `stream_session_of` of `src/api/library_items/play_lib_item_or_pod.rs`
    // holds the first rule for the stream of T-53 already.
    let start_position = match the_start_of_a_playback(&info_item) {
        TheStartOfAPlayback::ItStartsAt(place) => place,

        TheStartOfAPlayback::TheSessionHasNoIdentity => {
            error!("[play] the answer of the session names no session");
            say_why_the_playback_did_not_start(WhyNot::TheSessionHasNoIdentity);
            return Outcome::Fault;
        }

        // **A place that the server did not give is not the place 0.** The
        // program asks the server for the place of this media, and the status
        // 404 is the answer of a media that never played.
        TheStartOfAPlayback::TheProgramAsksForThePlace => {
            warn!(
                "[play] the answer of the session gave no place of {}. \
                 The program asks the server for it.",
                item_id
            );

            match get_the_place_of_a_media(api, &item_id, target.episode_id()).await {
                Ok(row) => row.current_time,
                Err(error) => match the_place_of_a_media_that_never_played(&error) {
                    Some(place) => place,
                    None => {
                        error!("[play] the server did not give the place: {}", error);
                        say_why_the_playback_did_not_start(WhyNot::ThePlaceDidNotCome(
                            error.to_string().as_str(),
                        ));
                        return Outcome::Fault;
                    }
                },
            }
        }
    };

    // Read the audio files and the chapters.
    let item: serde_json::Value = match api.get_json(&format!("/api/items/{}", item_id)).await {
        Ok(value) => value,
        Err(error) => {
            error!("[play] the server did not give the item: {}", error);
            say_why_the_playback_did_not_start(WhyNot::TheMediaDidNotCome(
                error.to_string().as_str(),
            ));
            return Outcome::Fault;
        }
    };

    let tracks = the_tracks_of_the_playback(&item, &target, &info_item[2]);

    let tracks = match tracks {
        Some(tracks) => tracks,
        None => {
            error!("[play] the item has no audio file");
            say_why_the_playback_did_not_start(WhyNot::NoAudioFile);
            return Outcome::Fault;
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
    let speed = the_speed_of_this_playback(&username);

    // Every playback has its own identity. The loop below reads the state of
    // the engine only while the engine plays this playback. See `9bacac`.
    let playback_id = next_playback_id();

    let request = PlaybackRequest {
        playback_id,
        item_id: item_id.clone(),
        title: info_item[4].clone(),
        author: info_item[6].clone(),
        username: username.clone(),
        tracks,
        sources,
        start_position,
        speed,
    };

    // **A playback that keeps no place does not start** (T-201). The old line
    // read the answer of this write with `let _ =`, and `insert_listening_session`
    // gave `Ok(())` for a connection that it did not get (T-200): the engine then
    // played the audio with no row of `listening_session` at all. The row of the
    // player of the screen reads that row, therefore it said `N/A` and no title;
    // the place of the user reached no disk, therefore a program that dies lost
    // the whole playback (T-145 and T-152); and every write of that place after it
    // changed 0 rows.
    //
    // **The rule of T-182 stands here**: a playback that the program cannot
    // follow does not start, and the program says why. The session of the server
    // is open already, therefore this program closes it.
    if let Err(error) = insert_listening_session(
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
        username.as_str(),
        server_key.as_str(),
    ) {
        error!(
            "[play] the disk did not take the session {} of the item {}: {}",
            session_id, item_id, error
        );

        if let Err(why) =
            crate::api::sessions::close_open_session::close_session_without_send_prg_data(
                api,
                session_id.as_str(),
            )
            .await
        {
            error!(
                "[play] the session {} of the server stays open: {}",
                session_id, why
            );
        }

        say_why_the_playback_did_not_start(WhyNot::TheDiskDidNotTakeTheSession(
            error.to_string().as_str(),
        ));

        return Outcome::Fault;
    }

    info!(
        "[play] the item {} starts at {} seconds with {} tracks",
        item_id,
        start_position,
        request.tracks.len()
    );

    player.send(PlayerCommand::Start(Box::new(request)));

    // The engine opens the file of the playback and the file after it. A file of
    // a codec that no decoder of the program reads gives a fault at once, and
    // the program then asks the server for a stream of the whole media. See
    // T-53.
    if let Some(name) = the_file_that_no_decoder_reads(player, playback_id, WAIT_FOR_A_FAULT).await
    {
        info!(
            "[play] no decoder of the program reads {}. The program asks the \
             server for a stream of the whole media.",
            name
        );

        return play_the_stream_of_the_server(
            api,
            player,
            &target,
            username,
            server_key,
            session_id,
            start_position,
            speed,
            &info_item,
        )
        .await;
    }

    follow_playback(
        api,
        player,
        session_id,
        item_id,
        target.episode_id().map(|value| value.to_string()),
        username,
        total_duration,
        playback_id,
        start_position,
    )
    .await
}

/// The time that the program waits for the fault of a decoder of a **file**.
/// See T-53.
///
/// The engine opens the decoders inside the command `Start`, therefore the fault
/// comes in some milliseconds. This value gives room for a server that answers
/// slowly, and it stays short: a playback that starts must not wait.
const WAIT_FOR_A_FAULT: Duration = Duration::from_millis(2500);

/// The time that the program waits for the fault of a **stream** of the server.
///
/// **A stream does not open in some milliseconds.** The open asks the server for
/// the first part, and ffmpeg of the server writes that part when it made it. A
/// measurement of 2026-08-11 with a file of xHE-AAC: the server needed 10.5
/// seconds for the first part of its second try, and the open of the program
/// waits about 25 seconds.
///
/// The old value of 2500 milliseconds therefore lost every message of such a
/// playback: the engine wrote the fault at the second 11.6, and no loop read it
/// any more. **The user pressed a key and read nothing at all.** A playback that
/// starts costs nothing here, because the loop stops at the first frame of the
/// engine. See T-68.
const WAIT_FOR_THE_STREAM: Duration = Duration::from_secs(35);

/// The time between two looks at the state of the engine.
const LOOK_AGAIN: Duration = Duration::from_millis(100);

/// Gives the name of the file that no decoder of the program reads, **and only
/// when that file stops this playback**. See T-53 and T-120.
///
/// **The engine sets one flag for two conditions**, and they need two answers:
///
/// 1. The track that the playback needs **now** does not open. `fill_queue`
///    fails, the engine stops the player, and it never writes `playback_id` for
///    this playback. **The playback is dead**, therefore the stream of the
///    server is the answer.
/// 2. A **later** track does not open, and the engine plays the tracks before
///    it. The engine says "The tracks before it play", and the book ends at that
///    track (T-48 and T-55). **The playback works**, therefore the stream of the
///    server is the wrong answer.
///
/// **The old code read the fault first**, therefore the condition 2 killed a
/// playback that played. The measurement of 2026-08-12, with a book of the user
/// that holds one file of AAC-LC and one file of xHE-AAC of the same 26 hours:
///
/// ```text
/// [play]   the item ... starts at 5031 seconds with 2 tracks
/// [worker] the engine cannot open the track 2 of 2: ... xHE-AAC ...
///          The tracks before it play.
/// [worker] the playback starts at 5031 seconds          <- the book plays
/// [play]   no decoder of the program reads ... xHE-AAC.m4b. The program asks
///          the server for a stream of the whole media.  <- and this ends it
/// ```
///
/// The user then heard the book from the start of the stream and not from their
/// place, and the screen said "One file needs the server" for a file that stands
/// 26 hours after the place of the user.
///
/// **The state tells the two conditions apart.** The engine writes
/// `playback_id` in the loop that follows a playback that plays, therefore a
/// playback that never started holds the identity of the playback before it.
async fn the_file_that_no_decoder_reads(
    player: &PlayerHandle,
    playback_id: u64,
    how_long: Duration,
) -> Option<String> {
    let end = std::time::Instant::now() + how_long;

    loop {
        let state = player.state();

        match the_stream_must_take_the_playback(&state, playback_id) {
            TheStart::TheFileNeedsTheServer(name) => return Some(name),
            TheStart::ThePlaybackPlays => return None,
            TheStart::NoAnswerYet => {}
        }

        if std::time::Instant::now() >= end {
            return None;
        }

        tokio::time::sleep(LOOK_AGAIN).await;
    }
}

/// What the state of the engine says about the start of one playback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheStart {
    /// The engine did not answer for this playback yet.
    NoAnswerYet,
    /// The engine plays this playback.
    ThePlaybackPlays,
    /// The playback did not start, and this file is the reason.
    TheFileNeedsTheServer(String),
}

/// Reads the state of the engine for one playback. See T-120.
///
/// **The rule of the loop stands here**, therefore a test holds it with no
/// engine and no server. The engine sets one flag (`file_with_no_decoder`) for
/// two conditions, and this function tells them apart: it reads "the engine
/// plays this playback" **before** the flag.
///
/// The engine writes `playback_id` in the loop that follows a playback that
/// plays. A start that failed never reaches that loop, therefore the state then
/// holds the identity of the playback before it and the status `Stopped`.
pub fn the_stream_must_take_the_playback(
    state: &crate::player::engine::PlaybackState,
    playback_id: u64,
) -> TheStart {
    // The engine plays this playback. A fault of a **later** file belongs to the
    // loop of the playback, and the book ends at the track before it (T-48 and
    // T-55). A stream of the server would end a playback that works.
    if state.playback_id == playback_id && state.status != PlaybackStatus::Stopped {
        return TheStart::ThePlaybackPlays;
    }

    // The fault must belong to this playback. The fault of the playback before
    // it belongs to a media that the user left. See T-53.
    if state.playback_of_the_fault == playback_id {
        if let Some(name) = state.file_with_no_decoder.clone() {
            return TheStart::TheFileNeedsTheServer(name);
        }
    }

    TheStart::NoAnswerYet
}

/// Plays the stream of the server, for a media with a file that no decoder of
/// the program reads. See T-53.
///
/// ffmpeg of the server makes the stream, therefore every codec of ffmpeg
/// becomes a codec of this program. The stream holds the **whole** media in one
/// track, therefore a book of many files needs no queue here.
#[allow(clippy::too_many_arguments)]
async fn play_the_stream_of_the_server(
    api: &ApiClient,
    player: &PlayerHandle,
    target: &PlaybackTarget,
    username: String,
    server_key: String,
    session_of_the_file: String,
    start_position: f64,
    speed: f32,
    info_item: &[String],
) -> Outcome {
    let item_id = target.item_id().to_string();

    crate::logic::message::say("One file needs the server. The stream starts, please wait…");

    // The session of the file stays open, and the server would then hold two
    // sessions of one media. The program closes it before the new one.
    player.send(PlayerCommand::Stop);

    if let Err(error) = close_session_without_send_prg_data(api, &session_of_the_file).await {
        warn!(
            "[play] the server did not close the session of the file: {}",
            error
        );
    }

    // **The program tries more than one place of the media.** The place decides
    // where ffmpeg of the server starts, and one part of a file can stop that
    // program. See T-69.
    let places = the_places_to_try(start_position);
    let mut the_last_fault: Option<String> = None;

    // The ratio of the progress needs the length of the media, and the program
    // holds it before the first session.
    let total_duration_of_the_item = info_item[2].clone();

    for (attempt, place) in places.iter().enumerate() {
        let place = *place;

        if attempt > 0 {
            info!(
                "[play] the place {} s gave no stream. The program tries {} s.",
                start_position, place
            );
            crate::logic::message::say(&the_message_of_a_second_place(place));

            // **The place of the server comes from the position of the user, and
            // not from the part that the client asks for.** Its log says
            // "Starting Stream at startTime 4:52 (User startTime 5:22)", and the
            // argument of ffmpeg is then `-ss 292s` for every part that the
            // client asks for. Therefore the program writes the position first.
            //
            // The program plays that place at once, therefore the position is
            // true. A run that gives no stream at all writes the position of the
            // user again. See T-69.
            if let Err(error) =
                write_the_place(api, target, place, &total_duration_of_the_item).await
            {
                warn!(
                    "[play] the server did not take the place {}: {}",
                    place, error
                );
            }
        }

        let stream = match post_a_stream_session(api, &item_id, target.episode_id()).await {
            Ok(stream) => stream,
            Err(error) => {
                error!("[play] the server gave no stream: {}", error);
                crate::logic::message::say(
                    "The server cannot make a stream of this media. See the log.",
                );
                return Outcome::Fault;
            }
        };

        let total_duration = if stream.duration > 0.0 {
            format!("{}", stream.duration.round() as u64)
        } else {
            info_item[2].clone()
        };

        // The stream holds the whole media, therefore the list holds one track and
        // that track starts at the second 0 of the media.
        let tracks = TrackList::new(
            vec![Track {
                index: 1,
                filename: format!("the stream of {}", item_id),
                ino: String::new(),
                size: None,
                mime_type: None,
                duration: stream.duration,
                start_offset: 0.0,
            }],
            chapters_of_the_media(api, &item_id).await,
        );

        let sources = vec![TrackSource::Stream {
            // The stream comes from the address that answered the request of
            // the stream. `an_address` gives an address in every condition,
            // therefore this value is never an empty text. See T-128.
            base_url: api.pool().an_address().unwrap_or_default(),
            playlist: stream.playlist.clone(),
            seconds: place,
        }];

        let playback_id = next_playback_id();

        let request = PlaybackRequest {
            playback_id,
            item_id: item_id.clone(),
            title: info_item[4].clone(),
            author: info_item[6].clone(),
            username: username.clone(),
            tracks,
            sources,
            // The stream itself starts at the place of the user, therefore the engine
            // starts at the second 0 of the stream and it adds that place to every
            // position that it reports. See T-53 and T-63.
            start_position: 0.0,
            speed,
        };

        // **A playback that keeps no place does not start** (T-201). The rule
        // of the file of the playback stands here too, and the stream of the
        // server is a session of the server like every other one.
        if let Err(error) = insert_listening_session(
            stream.session_id.clone(),
            item_id.clone(),
            place.round() as u32,
            total_duration.clone(),
            target.episode_id().unwrap_or_default().to_string(),
            0,
            request.title.clone(),
            request.author.clone(),
            true,
            String::new(),
            username.as_str(),
            server_key.as_str(),
        ) {
            error!(
                "[play] the disk did not take the session {} of the stream of the item {}: {}",
                stream.session_id, item_id, error
            );

            if let Err(why) =
                close_session_without_send_prg_data(api, stream.session_id.as_str()).await
            {
                error!(
                    "[play] the session {} of the stream stays open: {}",
                    stream.session_id, why
                );
            }

            say_why_the_playback_did_not_start(WhyNot::TheDiskDidNotTakeTheSession(
                error.to_string().as_str(),
            ));

            return Outcome::Fault;
        }

        info!(
            "[play] the stream of the item {} starts at {} seconds",
            item_id, place
        );

        player.send(PlayerCommand::Start(Box::new(request)));

        // The stream of the server can hold the audio in a form that no decoder of
        // the program reads. ffmpeg of the server copies the codec of the file when
        // that codec fits a transport stream, and AAC of the newest form fits it as
        // LATM only. The program must then say so, and it must not give silence.
        // See T-53.
        if let Some(name) =
            the_file_that_no_decoder_reads(player, playback_id, WAIT_FOR_THE_STREAM).await
        {
            error!(
                "[play] the stream of the place {} s did not play: {}",
                place, name
            );

            the_last_fault = player.state().why_the_start_did_not_work.clone();

            // **The session of this attempt must not stay open.** The server would
            // hold one session for each place that the program tried, and a session
            // that stays open is the report `dd9a649`.
            player.send(PlayerCommand::Stop);

            if let Err(error) = close_session_without_send_prg_data(api, &stream.session_id).await {
                warn!(
                    "[play] the server did not close the session of the stream: {}",
                    error
                );
            }

            continue;
        }

        return follow_playback(
            api,
            player,
            stream.session_id,
            item_id,
            target.episode_id().map(|value| value.to_string()),
            username,
            total_duration,
            playback_id,
            // The engine gives the position inside the stream, and the stream starts
            // at the place that the server gave. The loop adds that place. See T-53
            // and T-69.
            place,
        )
        .await;
    }

    // No place of the media gave a stream that plays. The program wrote a place
    // of its own for each attempt, therefore the position of the user comes
    // back. See T-69.
    if places.len() > 1 {
        if let Err(error) =
            write_the_place(api, target, start_position, &total_duration_of_the_item).await
        {
            warn!(
                "[play] the position {} of the user did not come back: {}",
                start_position, error
            );
        }
    }

    crate::logic::message::say(&the_message_of_a_stream_that_did_not_play(
        the_last_fault.as_deref(),
    ));

    Outcome::Fault
}

/// Writes one place of a media as the position of the user.
///
/// The stream of the server starts at the position of the user, therefore the
/// program must write a place before it asks for a stream of that place. See T-69.
async fn write_the_place(
    api: &ApiClient,
    target: &PlaybackTarget,
    place: f64,
    duration: &str,
) -> Result<(), crate::api::client::error::ApiError> {
    let seconds = Some(place.max(0.0).round() as u32);

    match target.episode_id() {
        Some(episode_id) => {
            update_media_progress_pod(api, target.item_id(), seconds, duration, episode_id).await
        }
        None => update_media_progress_book(api, target.item_id(), seconds, duration).await,
    }
}

/// Gives the sentence for a second place of a stream. See T-69.
///
/// The function is pure, therefore a test needs no server.
pub fn the_message_of_a_second_place(place: f64) -> String {
    format!(
        "The server gave no stream of that place. The program tries {} now.",
        crate::utils::convert_seconds::clock(place)
    )
}

/// The length of one part of the stream of the server, in seconds.
///
/// Audiobookshelf gives ffmpeg the argument `-hls_time 6`, therefore every part
/// of every playlist of that server holds six seconds. See T-69.
const SECONDS_OF_A_PART: f64 = 6.0;

/// The number of places that the program tries for one stream.
///
/// Each place costs the ten seconds of the server and the open, therefore a large
/// number makes the user wait. Three places gave the playback in one step in every
/// measurement of 2026-08-11. See T-69.
const PLACES_TO_TRY: usize = 3;

/// Gives the places of the media to try for a stream, in sequence.
///
/// **The place of the user decides where ffmpeg of the server starts.** The client
/// asks for one part of the playlist, and the server then starts its transcode at
/// that part: its log says "Segment #N Request is before starting segment number
/// #M - Reset Transcode". One part of a file of xHE-AAC holds a frame that gives
/// NaN to the encoder of ffmpeg, and ffmpeg then stops with the code 234 and the
/// server deletes the whole session. **A part beside it plays.**
///
/// A measurement of 2026-08-11 with the book of the user, at the parts of the
/// second 310 to 334:
///
/// | The place | The part | The answer of the server |
/// |---|---|---|
/// | 310 s | 51 | it ended the stream |
/// | 316 s | 52 | 74448 bytes |
/// | 322 s | 53 | it ended the stream |
/// | 328 s | 54 | 79148 bytes |
///
/// The program tries the place of the user first. **The part before it comes
/// next**, because a user hears a few seconds again more easily than they lose a
/// few seconds. A place before the start of the media does not come.
///
/// The function is pure, therefore a test needs no server. See T-69.
pub fn the_places_to_try(seconds: f64) -> Vec<f64> {
    let wanted = seconds.max(0.0);
    let mut places = vec![wanted];

    let mut step = SECONDS_OF_A_PART;

    while places.len() < PLACES_TO_TRY {
        // The part before the place of the user, and then the part after it.
        for place in [wanted - step, wanted + step] {
            if places.len() >= PLACES_TO_TRY {
                break;
            }

            if place >= 0.0 && !places.contains(&place) {
                places.push(place);
            }
        }

        step += SECONDS_OF_A_PART;

        // A media that starts at its beginning has no place before it. The step
        // must not grow for ever.
        if step > SECONDS_OF_A_PART * PLACES_TO_TRY as f64 * 2.0 {
            break;
        }
    }

    places
}

/// Gives the sentence for a stream of the server that did not play.
///
/// **The old sentence said that the program cannot read the form of the stream.**
/// A measurement of 2026-08-11 with a file of xHE-AAC showed that this is often
/// false: the server made **no part at all**, because its own ffmpeg cannot read
/// the form of the file. The user then read a sentence that names the wrong
/// program, and the log of the server holds the true cause.
///
/// The engine writes the sentence of the fault that it met, therefore this
/// function gives that sentence and it adds what the user can do. A start with no
/// such sentence keeps the old text.
///
/// The function is pure, therefore a test needs no server and no engine.
/// See T-68.
pub fn the_message_of_a_stream_that_did_not_play(why: Option<&str>) -> String {
    let Some(why) = why.map(str::trim).filter(|text| !text.is_empty()) else {
        return "The stream of the server did not play. Read the log, and see \
                T-53."
            .to_string();
    };

    // **The message stands in one row of the screen.** A measurement of
    // 2026-08-11 lost the end of a message of 200 letters in a terminal of 160
    // columns. Therefore the two parts together must stay short. See T-68.
    format!("{} A file of a different form is the answer.", why)
}

/// Gives the chapters of one media, for a playback of the stream of the server.
///
/// The stream holds no chapter, therefore the program asks the server for them.
/// A media with no chapter gives an empty list, and that is not a fault.
async fn chapters_of_the_media(api: &ApiClient, item_id: &str) -> Vec<Chapter> {
    let item: serde_json::Value = match api.get_json(&format!("/api/items/{}", item_id)).await {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    chapters_from(&item)
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
) -> Outcome {
    let selected = target.item_id().to_string();

    // The download of an episode has the identity of the episode.
    let key = target.episode_id().unwrap_or(&selected).to_string();

    // **A read of the disk that failed is not a media with no copy on the disk**
    // (T-203). The words of a program that did not read its own database name the
    // database, and not the disk of the user (T-91 and T-199).
    let row = match get_download_row(&key, &username) {
        Ok(row) => row,
        Err(error) => {
            error!(
                "[play] the program did not read the row of the download {}: {}",
                key, error
            );
            crate::logic::message::say(THE_DATABASE_OF_THE_PROGRAM_SAID_NOTHING);
            return Outcome::Fault;
        }
    };

    let Some(row) = row else {
        error!("[play] the disk has no copy of {}", key);
        crate::logic::message::say(
            "The server does not answer, and the disk has no copy of this media.",
        );
        return Outcome::Fault;
    };

    let tracks = match tracks_from_downloads(&key, &username) {
        Ok(tracks) => tracks,
        Err(error) => {
            error!(
                "[play] the program did not read the files of the download {}: {}",
                key, error
            );
            crate::logic::message::say(THE_DATABASE_OF_THE_PROGRAM_SAID_NOTHING);
            return Outcome::Fault;
        }
    };

    let Some(tracks) = tracks else {
        error!("[play] the disk has no audio file of {}", key);
        crate::logic::message::say(
            "The server does not answer, and the disk has no audio file of this media.",
        );
        return Outcome::Fault;
    };

    let track_list: Vec<Track> = (0..tracks.len())
        .filter_map(|index| tracks.get(index).cloned())
        .collect();

    // **The program reads the files of the download one time.** The old shape read
    // the disk for each track, and a read that failed then said that the disk does
    // not hold every file of the media (T-203).
    let files = match get_download_files(&key, &username) {
        Ok(files) => files,
        Err(error) => {
            error!(
                "[play] the program did not read the files of the download {}: {}",
                key, error
            );
            crate::logic::message::say(THE_DATABASE_OF_THE_PROGRAM_SAID_NOTHING);
            return Outcome::Fault;
        }
    };

    let sources: Vec<TrackSource> = track_list
        .iter()
        .filter_map(|track| {
            files
                .iter()
                .find(|(index, _, _)| *index == track.index)
                .map(|(_, path, _)| TrackSource::Local(std::path::PathBuf::from(path)))
        })
        .collect();

    if sources.len() != track_list.len() {
        error!("[play] the disk does not hold every file of {}", key);
        crate::logic::message::say("The disk does not hold every file of this media.");
        return Outcome::Fault;
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

    let speed = the_speed_of_this_playback(&username);

    let playback_id = next_playback_id();

    let request = PlaybackRequest {
        playback_id,
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

    crate::logic::message::say(&format!("Offline: \"{}\" plays from the disk.", row.title));

    follow_playback_offline(
        player,
        key,
        item_id,
        episode_id,
        username,
        server,
        total_duration,
        playback_id,
        row.current_time as f64,
    )
    .await
}

/// Follows a playback that has no server.
///
/// The loop writes the position in the database for each second. When the
/// playback stops, the loop keeps the position for the server. The application
/// sends it when the server answers again.
///
/// The loop reads the state of the engine only while the engine plays this
/// playback. See the head of `follow_playback`.
#[allow(clippy::too_many_arguments)]
pub async fn follow_playback_offline(
    player: &PlayerHandle,
    key: String,
    item_id: String,
    episode_id: Option<String>,
    username: String,
    server: String,
    total_duration: f64,
    playback_id: u64,
    start_position: f64,
) -> Outcome {
    let mut own_position = start_position.max(0.0) as u32;
    let mut engine_started = false;
    let mut waited: u64 = 0;

    // The engine reported a position at the place where this playback starts.
    // Before that, every value that the engine gives belongs to the time
    // before the seek. See T-38.
    let mut reached_the_start = false;

    // **One line of the log while the disk says nothing** (T-207). The loop
    // writes the disk each second, and it writes two rows of it: the row of the
    // download, and the row that waits for the server (T-210).
    let mut the_disk_said_nothing_of_the_place = false;
    let mut the_disk_said_nothing_of_the_place_that_waits = false;

    // The playback of before this one is not the playback of the user now.
    the_place_of_the_disk::the_disk_says(false);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let state = player.state();

        if state.playback_id != playback_id {
            waited += 1;

            if !gave_up(&state, playback_id, engine_started, waited) {
                continue;
            }

            info!(
                "[follow_playback_offline] the engine does not play the item {} \
                 now. The loop keeps the position {} seconds.",
                item_id, own_position
            );

            // **The place of an offline playback reaches the server through this
            // row alone** (T-152), therefore the caller reads the answer of it
            // and the user reads one word of that fault (T-212). The playback
            // belongs to no view, therefore the message stands above them all
            // (T-164).
            if !remember_progress(
                &username,
                &server,
                &item_id,
                episode_id.as_deref(),
                own_position as f64,
                total_duration,
                false,
            ) {
                crate::logic::message::say(THE_DISK_KEPT_NO_PLACE);
            }

            return Outcome::Stopped;
        }

        engine_started = true;

        let reported = state.position.max(0.0);

        // The same rule as in `follow_playback`: the engine gives 0 until the
        // seek finishes, and that 0 must not go to the database. See T-38.
        if !reached_the_start && !position_is_at_the_start(reported, start_position) {
            continue;
        }

        reached_the_start = true;

        let position = reported as u32;
        own_position = position;

        // **The work of the disk stands on a thread of its own** (T-204): a
        // write that meets the lock of a second program of the account holds
        // the thread that calls it for five seconds, and a thread of the
        // runtime is the driver of the loop of the screen.
        let of_the_download = key.clone();
        let of_the_account = username.clone();

        let the_answer = crate::db::the_work_of_the_disk(move || {
            update_download_current_time(
                of_the_download.as_str(),
                of_the_account.as_str(),
                position,
            )
        })
        .await;

        // **A caller that reads no answer of its write says nothing at all**
        // (T-207). The place of an offline playback reaches the disk here alone
        // (T-152), therefore a write that failed takes the place of the user of
        // this second away with no word.
        let the_disk_takes_no_place = the_line_of_a_write_of_the_loop(
            &mut the_disk_said_nothing_of_the_place,
            "the place of the offline playback",
            the_answer,
        );

        // **The position of an offline playback reaches the server at no other
        // moment.** `play_offline` opens no session on the server, therefore
        // no row of `listening_session` stands for this playback and the rule
        // of T-145 gives the next program nothing to send. A program that dies
        // here — the terminal that goes away, the kill of the machine — writes
        // nothing more.
        //
        // The loop keeps the place of the user for the server at each second,
        // in the same way that it writes that place to the row of the download
        // at each second. A newer position replaces the older one, and the
        // function says no line of the log. See T-152.
        let of_the_account = username.clone();
        let of_the_server = server.clone();
        let of_the_item = item_id.clone();
        let of_the_episode = episode_id.clone();

        let the_answer = crate::db::the_work_of_the_disk(move || {
            crate::logic::offline::keep_progress(
                of_the_account.as_str(),
                of_the_server.as_str(),
                of_the_item.as_str(),
                of_the_episode.as_deref(),
                position as f64,
                total_duration,
                false,
            )
        })
        .await;

        // **A caller that reads no answer of its write says nothing at all**
        // (T-206 and T-207). This write is the one copy of the place of an
        // offline playback for the server (T-152), and the old code threw its
        // answer away: `keep_progress` wrote a line of the log for each second
        // — 28800 lines for a book of eight hours — and no word came to the
        // user. See T-210.
        let the_place_that_waits_did_not_come = the_line_of_a_write_of_the_loop(
            &mut the_disk_said_nothing_of_the_place_that_waits,
            "the place that waits for the server",
            the_answer,
        );

        // **The row of the player holds the word of a condition that stands**
        // (T-210): a message of the program lives six seconds, and a disk that
        // takes no write takes every second of this playback away.
        the_place_of_the_disk::the_disk_says(
            the_disk_takes_no_place || the_place_that_waits_did_not_come,
        );

        if state.status == PlaybackStatus::Stopped {
            let finished = state.finished;

            info!(
                "[follow_playback_offline] the playback stopped at {} seconds, finished={}",
                position, finished
            );

            // The last place of this playback, and the mark of a media that came
            // to its end: the loop wrote no such mark at any second before this
            // one. See T-212.
            if !remember_progress(
                &username,
                &server,
                &item_id,
                episode_id.as_deref(),
                position as f64,
                total_duration,
                finished,
            ) {
                crate::logic::message::say(THE_DISK_KEPT_NO_PLACE);
            }

            return outcome_of(finished);
        }
    }
}

/// Tells if a loop must stop, because the engine does not play its playback.
///
/// The engine writes the identity of the playback that it plays. A loop whose
/// identity is not that value is in one of two conditions:
///
/// - A different playback took the engine. The state then holds the position
///   of that other media, and the loop must stop immediately.
/// - The engine did not start this playback yet. The loop waits, because the
///   engine opens the first audio file before it plays.
///
/// A larger identity in the state means that a later playback took the engine.
/// A loop that saw its own playback one time and does not see it now is also
/// in the first condition.
/// Gives the outcome of a playback that stopped.
///
/// The engine writes `finished` only when no track stays and the position is
/// at the end. `PlayerCommand::Stop` writes `false` in the same field.
/// Therefore this one value separates an end from a stop of the user, and the
/// queue reads it. See T-16.
pub fn outcome_of(finished: bool) -> Outcome {
    if finished {
        Outcome::Finished
    } else {
        Outcome::Stopped
    }
}

fn gave_up(
    state: &crate::player::engine::PlaybackState,
    playback_id: u64,
    engine_started: bool,
    waited: u64,
) -> bool {
    engine_started || state.playback_id > playback_id || waited >= START_TIME_LIMIT
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
///
/// Tells if the engine reached the place where a playback starts.
///
/// `rodio` gives the position inside the source, and it gives 0 until the seek
/// finishes. A book that starts at 1227 seconds therefore reports 0 for a
/// short time, and a playback that never starts reports 0 for ever.
///
/// The tolerance of two seconds is for a decoder that gives a position a
/// little before the target of the seek.
///
/// A book that starts at 0 gives `true` at once, therefore this rule changes
/// nothing for a book that the user never opened. See T-38.
pub fn position_is_at_the_start(reported: f64, start: f64) -> bool {
    const TOLERANCE: f64 = 2.0;

    reported + TOLERANCE >= start
}

/// # The identity of the playback
///
/// The state of the engine is one value for the whole application. The loop
/// therefore reads that state only while `state.playback_id` is the identity
/// of its own playback.
///
/// The old code read the state always. Two playbacks can run at the same time,
/// because the key that starts a media gives its work to a new task. The loop
/// of the book X then read the position of the book Y, and it reported that
/// position for X. The loop also never saw the status `Stopped`, because the
/// engine played Y. Therefore the session of X stayed open.
///
/// A measurement on 2026-08-10 showed this fault. The loop of X sent
/// `{"currentTime":"4","timeListened":"0"}` to the session of X while the
/// engine played Y at 4 seconds, and X was at 100 seconds. The loop did not
/// stop. See `9bacac`, `86384e`, and `dd9a649` in `known_bugs.md`.
#[allow(clippy::too_many_arguments)]
pub async fn follow_playback(
    api: &ApiClient,
    player: &PlayerHandle,
    session_id: String,
    item_id: String,
    episode_id: Option<String>,
    username: String,
    total_duration: String,
    playback_id: u64,
    start_position: f64,
) -> Outcome {
    let mut since_sync: u64 = 0;
    let mut last_position: u32 = 0;
    let mut was_stalled = false;

    // The last position that the engine reported for this playback. The loop
    // reports this value, and it never reports the position of a different
    // media.
    let mut own_position = start_position.max(0.0) as u32;
    let mut engine_started = false;
    let mut waited: u64 = 0;

    // The engine reported a position at the place where this playback starts.
    // Before that, every value that the engine gives belongs to the time
    // before the seek. See T-38.
    let mut reached_the_start = false;

    // **One line of the log while the disk says nothing** (T-207). The loop
    // writes the disk each second.
    let mut the_disk_said_nothing_of_the_place = false;

    // The playback of before this one is not the playback of the user now.
    the_place_of_the_disk::the_disk_says(false);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let state = player.state();

        // The engine does not play this playback. The state holds the position
        // of a different media, therefore the loop must not read it.
        if state.playback_id != playback_id {
            waited += 1;

            if !gave_up(&state, playback_id, engine_started, waited) {
                continue;
            }

            if engine_started || state.playback_id > playback_id {
                info!(
                    "[follow_playback] a different playback took the engine. The \
                     loop of the item {} stops at {} seconds.",
                    item_id, own_position
                );
            } else {
                warn!(
                    "[follow_playback] the engine did not start the item {} in {} \
                     seconds. The loop closes the session.",
                    item_id, START_TIME_LIMIT
                );
            }

            // The media is not finished: this playback never came to an end.
            let the_server_holds_it = close_and_report(
                api,
                &session_id,
                &item_id,
                episode_id.as_deref(),
                own_position,
                &total_duration,
                false,
            )
            .await;

            if the_server_holds_it {
                // **A removal that the disk refused is no removal** (T-207), and
                // the box of it names the machine that holds that place (T-212).
                the_row_of_a_closed_session_goes_away(
                    session_id.as_str(),
                    ThePlaceOfTheSession::TheServerHoldsIt,
                );
            }

            return Outcome::Stopped;
        }

        engine_started = true;

        let reported = state.position.max(0.0);

        // The engine did not reach the place where this playback starts.
        //
        // `rodio` gives the position inside the source, and `get_pos` gives 0
        // until the seek finishes. A book of one file that starts at 1227
        // seconds therefore reports 0 for a short time. A playback that never
        // starts reports 0 for the whole wait.
        //
        // The old code wrote that 0 in the database every second, and it gave
        // that 0 to the server when the session closed. The user then lost
        // their place, on the disk and on the server, and the book started at
        // the beginning. See T-38.
        if !reached_the_start && !position_is_at_the_start(reported, start_position) {
            continue;
        }

        reached_the_start = true;

        let position = reported as u32;
        own_position = position;

        // Write the position for each second. A crash must not lose it.
        //
        // **The work of the disk stands on a thread of its own** (T-204): these
        // three writes took a thread of the runtime for 15 seconds while a
        // second program of the account held the database, and the loop of the
        // screen waited on that thread — the row of the player, the timer for
        // sleep, and every key of the user stopped.
        let of_the_session = session_id.clone();
        let of_the_item = item_id.clone();
        let of_the_account = username.clone();
        let of_the_chapter = state.chapter_title.clone();

        let the_answer = crate::db::the_work_of_the_disk(move || {
            // **The row of the session holds the place of the user for a program
            // that dies, and the row of the player of the screen reads it**
            // (T-201). A write that failed took both of them away with no word.
            let the_place = update_current_time(position, of_the_session.as_str());

            let of_the_disk = update_download_current_time(
                of_the_item.as_str(),
                of_the_account.as_str(),
                position,
            );

            let of_the_chapter = match of_the_chapter.as_ref() {
                Some(title) => update_chapter(title, of_the_session.as_str()),
                None => Ok(()),
            };

            the_place.and(of_the_disk).and(of_the_chapter)
        })
        .await;

        // **The row of the player holds the word of a condition that stands**
        // (T-210). The row of `listening_session` is the place of the user for a
        // program that dies, and the row of the player of the screen reads it
        // (T-201): a disk that takes no write takes the two of them away.
        the_place_of_the_disk::the_disk_says(the_line_of_a_write_of_the_loop(
            &mut the_disk_said_nothing_of_the_place,
            "the place of this playback",
            the_answer,
        ));

        match state.status {
            PlaybackStatus::Playing => {
                if was_stalled {
                    info!("[follow_playback] the data comes again");
                    was_stalled = false;
                }

                let moved = position.saturating_sub(last_position);
                since_sync += 1;

                // The user asked for the sync now. The loop does the work,
                // because the loop holds the listened time. Two senders would
                // give that time to the server two times. See T-32.
                let forced = force_sync::take_request(playback_id);

                if since_sync >= SYNC_PERIOD || forced {
                    let outcome = sync_session(api, &session_id, Some(position), moved).await;

                    if let Err(error) = &outcome {
                        warn!(
                            "[follow_playback] the server did not accept the sync: {}",
                            error
                        );
                    }

                    if forced {
                        force_sync::report(force_sync::message(&outcome, position));
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
                // A playback that waits gives no new listened time. The sync
                // therefore sends the position and the value 0. See T-32.
                if force_sync::take_request(playback_id) {
                    let outcome = sync_session(api, &session_id, Some(position), 0).await;

                    if let Err(error) = &outcome {
                        warn!(
                            "[follow_playback] the server did not accept the sync: {}",
                            error
                        );
                    }

                    force_sync::report(force_sync::message(&outcome, position));
                }

                since_sync = 0;
                last_position = position;
            }

            PlaybackStatus::Stopped => {
                let finished = state.finished;

                info!(
                    "[follow_playback] the playback stopped at {} seconds, finished={}",
                    position, finished
                );

                // **A stream of the server that stopped before its last part is
                // not the end of the media**, and the user read nothing at all:
                // the book stopped in the middle, and the program told the
                // server that the user finished it. The playback belongs to no
                // view, therefore the message of it stands above them all
                // (T-164). See T-194.
                if let Some(why) = &state.why_the_stream_stopped {
                    warn!("[follow_playback] {}", why);
                    crate::logic::message::say(why);
                }

                let _ = update_is_finished(if finished { "1" } else { "0" }, session_id.as_str());

                let the_server_holds_it = close_and_report(
                    api,
                    &session_id,
                    &item_id,
                    episode_id.as_deref(),
                    position,
                    &total_duration,
                    finished,
                )
                .await;

                if the_server_holds_it {
                    // **A removal that the disk refused is no removal** (T-207),
                    // and the box of it names the machine that holds that place
                    // (T-212).
                    the_row_of_a_closed_session_goes_away(
                        session_id.as_str(),
                        ThePlaceOfTheSession::TheServerHoldsIt,
                    );
                }

                return outcome_of(finished);
            }
        }
    }
}

/// Closes the session on the server, and sends the position of the media.
///
/// The function sends `/progress` and not `/sync`. The media came to its end,
/// or the user stopped it. This is a command of the user, and it is not a
/// report during the playback. See upstream issue 35.
///
/// If the media came to its end, the request also marks the item as finished.
/// See T-16.
///
/// **The answer says if the server holds the position now**, and the caller
/// removes the row of that playback when it does: a row that stays sends this
/// position again at the next start, and it then destroys a place that a
/// different client wrote. A server that refused it keeps the row, because the
/// position of the user lives in that row only. See T-141, T-4, and T-25.
#[allow(clippy::too_many_arguments)]
async fn close_and_report(
    api: &ApiClient,
    session_id: &str,
    item_id: &str,
    episode_id: Option<&str>,
    position: u32,
    total_duration: &str,
    finished: bool,
) -> bool {
    if let Err(error) = close_session_without_send_prg_data(api, session_id).await {
        warn!(
            "[follow_playback] the server did not close the session: {}",
            error
        );
    }

    let result = match (episode_id, finished) {
        (Some(episode_id), true) => {
            update_media_progress2_pod(
                api,
                item_id,
                Some(position),
                total_duration,
                true,
                episode_id,
            )
            .await
        }
        (Some(episode_id), false) => {
            update_media_progress_pod(api, item_id, Some(position), total_duration, episode_id)
                .await
        }
        (None, true) => {
            update_media_progress2_book(api, item_id, Some(position), total_duration, true).await
        }
        (None, false) => {
            update_media_progress_book(api, item_id, Some(position), total_duration).await
        }
    };

    if let Err(error) = result {
        warn!(
            "[follow_playback] the server did not accept the position: {}",
            error
        );

        return false;
    }

    true
}

#[cfg(test)]
mod tests_of_the_start {
    use super::position_is_at_the_start;

    /// The report of the user of 2026-08-10: the book started at the
    /// beginning, and the position of the disk and of the server went to 0.
    ///
    /// `rodio` gives 0 until the seek finishes. The old rule took every value
    /// that the engine gave, therefore it wrote that 0.
    #[test]
    fn a_position_of_zero_is_not_the_place_of_a_book_that_starts_late() {
        // The book starts at 1227 seconds. The engine says 0 while it seeks.
        assert!(!position_is_at_the_start(0.0, 1227.0));
        assert!(!position_is_at_the_start(3.0, 1227.0));
        assert!(!position_is_at_the_start(1200.0, 1227.0));
    }

    #[test]
    fn the_place_of_the_seek_is_the_start() {
        assert!(position_is_at_the_start(1227.0, 1227.0));
        assert!(position_is_at_the_start(1300.0, 1227.0));
    }

    #[test]
    fn a_decoder_that_is_a_little_early_still_counts() {
        // A decoder can give a position a little before the target of the
        // seek. Two seconds are inside the rule, and three are not.
        assert!(position_is_at_the_start(1225.5, 1227.0));
        assert!(!position_is_at_the_start(1224.0, 1227.0));
    }

    #[test]
    fn a_book_that_starts_at_the_beginning_changes_nothing() {
        // Every value passes for a book that the user never opened. The rule
        // must not hold the position of such a book.
        assert!(position_is_at_the_start(0.0, 0.0));
        assert!(position_is_at_the_start(0.0, 1.0));
        assert!(position_is_at_the_start(5.0, 0.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The program tries the place of the user first, and then the part before
    /// it: a user hears a few seconds again more easily than they lose a few
    /// seconds. See T-69.
    #[test]
    fn the_places_of_a_stream_hold_the_place_of_the_user_first() {
        // The measurement of 2026-08-11: the part of 322 s stops ffmpeg of the
        // server, and the part of 316 s plays.
        assert_eq!(the_places_to_try(322.0), vec![322.0, 316.0, 328.0]);

        assert_eq!(the_places_to_try(322.0).len(), PLACES_TO_TRY);
        assert_eq!(
            the_places_to_try(322.0).first(),
            Some(&322.0),
            "the place of the user comes first"
        );
    }

    /// A media at its beginning has no place before it. The places must stay
    /// inside the media, and the list must hold no value two times.
    #[test]
    fn the_places_of_a_stream_stay_inside_the_media() {
        let of_the_start = the_places_to_try(0.0);

        assert_eq!(of_the_start.first(), Some(&0.0));
        assert!(
            of_the_start.iter().all(|place| *place >= 0.0),
            "no place before the start of the media: {:?}",
            of_the_start
        );
        assert_eq!(of_the_start, vec![0.0, 6.0, 12.0]);

        // A negative place of the caller becomes the start of the media.
        assert_eq!(the_places_to_try(-30.0).first(), Some(&0.0));

        // A place of the first part has one place before it only.
        let of_the_first_part = the_places_to_try(4.0);
        assert!(of_the_first_part.iter().all(|place| *place >= 0.0));
        assert_eq!(of_the_first_part.len(), PLACES_TO_TRY);

        for places in [the_places_to_try(0.0), the_places_to_try(322.0)] {
            let mut once = places.clone();
            once.dedup();
            assert_eq!(once.len(), places.len(), "no place two times: {:?}", places);
        }
    }

    /// The message of a second place must name the place in the form of a clock,
    /// and not a number of seconds. See T-69.
    #[test]
    fn the_message_of_a_second_place_names_the_place() {
        let message = the_message_of_a_second_place(316.0);

        assert!(message.contains("5:16"), "{}", message);
        assert!(message.chars().count() <= 150, "{}", message);
    }

    /// The message of a stream that did not play must give the sentence of the
    /// place that met the fault, and it must not name the program of the user as
    /// the cause. See T-68.
    #[test]
    fn the_message_names_the_true_cause_of_a_stream_that_did_not_play() {
        let of_the_server = crate::player::engine::hls_file::the_sentence_of_no_part(14);
        let message = the_message_of_a_stream_that_did_not_play(Some(&of_the_server));

        assert!(
            message.contains("made no part"),
            "the message must hold the sentence of the engine: {}",
            message
        );
        assert!(
            message.contains("A file of a different form"),
            "the message must say what the user can do: {}",
            message
        );

        // The message stands in one row of the screen. A terminal of 160 columns
        // loses the end of a longer message. See T-68.
        assert!(
            message.chars().count() <= 150,
            "the message holds {} letters, and the row of the screen is shorter",
            message.chars().count()
        );
        assert!(
            !message.contains("the program cannot read"),
            "the message must not name the program of the user: {}",
            message
        );

        // A fault of the form of the stream keeps its own sentence.
        let of_the_form = "The stream of the server holds the audio in the form \
                           Latm, and no decoder of the program reads it.";
        assert!(the_message_of_a_stream_that_did_not_play(Some(of_the_form)).contains("Latm"));

        // A start with no sentence gives the old text, and it names the log.
        for nothing in [None, Some(""), Some("   ")] {
            let message = the_message_of_a_stream_that_did_not_play(nothing);
            assert!(message.contains("Read the log"), "{}", message);
        }
    }

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

    /// Makes a state that reports a playback.
    fn state_of(playback_id: u64) -> crate::player::engine::PlaybackState {
        crate::player::engine::PlaybackState {
            playback_id,
            ..Default::default()
        }
    }

    /// A later playback took the engine. The loop must stop immediately, and it
    /// must not wait. This is the report `9bacac`.
    #[test]
    fn a_later_playback_stops_the_loop_immediately() {
        assert!(gave_up(&state_of(8), 7, false, 1));
    }

    /// The loop saw its own playback one time, and it does not see it now.
    /// Therefore a different playback took the engine.
    #[test]
    fn a_loop_that_lost_the_engine_stops_immediately() {
        assert!(gave_up(&state_of(0), 7, true, 1));
    }

    /// The engine opens the first audio file before it plays. The loop waits
    /// for that operation, and it must not close the session.
    #[test]
    fn a_loop_waits_for_the_start_of_the_engine() {
        assert!(!gave_up(&state_of(0), 7, false, 1));
        assert!(!gave_up(&state_of(6), 7, false, START_TIME_LIMIT - 1));
    }

    /// The engine did not start this playback at all. The loop closes its
    /// session, because a session that stays open is the report `dd9a649`.
    #[test]
    fn a_loop_stops_when_the_engine_does_not_start() {
        assert!(gave_up(&state_of(0), 7, false, START_TIME_LIMIT));
    }

    /// The queue reads this value. A media that came to its end gives the
    /// engine to the next media, and a media that the user stopped does not.
    #[test]
    fn the_end_of_a_media_gives_the_outcome_of_an_end() {
        assert_eq!(outcome_of(true), Outcome::Finished);
        assert_eq!(outcome_of(false), Outcome::Stopped);

        assert!(the_queue_goes_on(outcome_of(true)));
        assert!(!the_queue_goes_on(outcome_of(false)));
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
