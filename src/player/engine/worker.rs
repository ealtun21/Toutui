//! The thread that owns the audio.
//!
//! One thread owns the `rodio::Player`. It reads the commands, and it writes
//! the state. No other thread touches the player.
//!
//! The thread appends two tracks only. A book of the test library has 209
//! audio files, and 209 open connections are not acceptable.

use crate::player::engine::source::open_decoder;
use crate::player::engine::{
    media_position, PlaybackRequest, PlaybackState, PlaybackStatus, PlayerCommand,
};
use log::{error, info, warn};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// The time between two examinations of the state.
const TICK: Duration = Duration::from_millis(200);

/// The time that the message "Reconnected" stays on the screen.
const NOTICE_TIME: Duration = Duration::from_secs(4);

/// The number of tracks that the queue holds.
const QUEUE_DEPTH: usize = 2;

/// Starts the thread of the engine.
///
/// The function opens the sound card before it starts the thread. Therefore
/// the caller gets the error if the computer has no sound card.
pub fn spawn(
    receiver: Receiver<PlayerCommand>,
    state: Arc<RwLock<PlaybackState>>,
    token: String,
) -> Result<(), String> {
    let sink = DeviceSinkBuilder::open_default_sink()
        .map_err(|error| format!("The application cannot open the sound card: {}", error))?;

    std::thread::Builder::new()
        .name("toutui-audio".to_string())
        .spawn(move || run(receiver, state, token, sink))
        .map_err(|error| format!("The application cannot start the audio thread: {}", error))?;

    Ok(())
}

/// What the thread plays now.
struct Current {
    request: PlaybackRequest,
    /// The track that plays now.
    playing: usize,
    /// The number of tracks in the queue.
    queued: usize,
}

fn run(
    receiver: Receiver<PlayerCommand>,
    state: Arc<RwLock<PlaybackState>>,
    token: String,
    sink: MixerDeviceSink,
) {
    let mut player = Player::connect_new(sink.mixer());
    let mut current: Option<Current> = None;
    let mut notice_until: Option<Instant> = None;

    loop {
        match receiver.recv_timeout(TICK) {
            Ok(command) => handle(command, &mut player, &sink, &mut current, &token, &state),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }

        advance(&mut player, &mut current, &token);
        publish(&player, &current, &state, &mut notice_until);
    }
}

/// Runs one command.
fn handle(
    command: PlayerCommand,
    player: &mut Player,
    sink: &MixerDeviceSink,
    current: &mut Option<Current>,
    token: &str,
    state: &Arc<RwLock<PlaybackState>>,
) {
    match command {
        PlayerCommand::Start(request) => start(*request, player, sink, current, token, state),
        PlayerCommand::Pause => player.pause(),
        PlayerCommand::Resume => player.play(),
        PlayerCommand::SeekTo(position) => seek_to(player, current, token, position),
        PlayerCommand::SeekBy(change) => {
            let now = position_now(player, current);
            seek_to(player, current, token, now + change);
        }
        PlayerCommand::NextChapter => {
            let now = position_now(player, current);
            let target = current
                .as_ref()
                .and_then(|item| item.request.tracks.next_chapter_start(now));

            if let Some(target) = target {
                seek_to(player, current, token, target);
            }
        }
        PlayerCommand::PreviousChapter => {
            let now = position_now(player, current);
            let target = current
                .as_ref()
                .and_then(|item| item.request.tracks.previous_chapter_start(now));

            if let Some(target) = target {
                seek_to(player, current, token, target);
            }
        }
        PlayerCommand::SetSpeed(value) => player.set_speed(value.clamp(0.1, 5.0)),
        PlayerCommand::SetVolume(value) => player.set_volume(value.clamp(0.0, 2.0)),
        PlayerCommand::Stop => {
            player.stop();
            *current = None;
            set_status(state, PlaybackStatus::Stopped);
        }
    }
}

/// Starts a book.
fn start(
    request: PlaybackRequest,
    player: &mut Player,
    sink: &MixerDeviceSink,
    current: &mut Option<Current>,
    token: &str,
    state: &Arc<RwLock<PlaybackState>>,
) {
    player.stop();

    // A new player gives an empty queue and a speed that is known.
    *player = Player::connect_new(sink.mixer());
    player.set_speed(request.speed.clamp(0.1, 5.0));

    let start_position = request.start_position;

    let (track_index, offset) = match request.tracks.locate(start_position) {
        Some(value) => value,
        None => {
            error!("[worker] the book has no audio file");
            set_status(state, PlaybackStatus::Stopped);
            *current = None;
            return;
        }
    };

    let mut item = Current {
        request,
        playing: track_index,
        queued: 0,
    };

    if let Err(error) = fill_queue(player, &mut item, token) {
        error!("[worker] the engine cannot start the book: {}", error);
        set_status(state, PlaybackStatus::Stopped);
        *current = None;
        return;
    }

    if offset > 0.0 {
        if let Err(error) = player.try_seek(Duration::from_secs_f64(offset)) {
            warn!("[worker] the engine cannot move to the position: {}", error);
        }
    }

    player.play();
    info!("[worker] the playback starts at {} seconds", start_position);

    *current = Some(item);
}

/// Gives the position in the book.
fn position_now(player: &Player, current: &Option<Current>) -> f64 {
    let item = match current {
        Some(item) => item,
        None => return 0.0,
    };

    let inside = media_position(player.get_pos(), player.speed());

    item.request.tracks.position_of(item.playing, inside)
}

/// Moves to a position in the book.
fn seek_to(player: &mut Player, current: &mut Option<Current>, token: &str, position: f64) {
    let item = match current.as_mut() {
        Some(item) => item,
        None => return,
    };

    let (track_index, offset) = match item.request.tracks.locate(position) {
        Some(value) => value,
        None => return,
    };

    if track_index == item.playing {
        if let Err(error) = player.try_seek(Duration::from_secs_f64(offset)) {
            warn!("[worker] the engine cannot move inside the track: {}", error);
        }
        return;
    }

    // The target is in a different track. Make the queue again.
    player.clear();
    item.playing = track_index;
    item.queued = 0;

    if let Err(error) = fill_queue(player, item, token) {
        error!("[worker] the engine cannot make the queue again: {}", error);
        return;
    }

    if offset > 0.0 {
        if let Err(error) = player.try_seek(Duration::from_secs_f64(offset)) {
            warn!("[worker] the engine cannot move inside the track: {}", error);
        }
    }

    player.play();
}

/// Appends tracks until the queue holds `QUEUE_DEPTH` tracks.
fn fill_queue(player: &mut Player, item: &mut Current, token: &str) -> Result<(), String> {
    while item.queued < QUEUE_DEPTH {
        let track_index = item.playing + item.queued;

        let track = match item.request.tracks.get(track_index) {
            Some(track) => track.clone(),
            None => break,
        };

        let source = match item.request.sources.get(track_index) {
            Some(source) => source.clone(),
            None => break,
        };

        let decoder = open_decoder(&source, token, &track.filename)?;
        player.append(decoder);
        item.queued += 1;
    }

    Ok(())
}

/// Follows the queue, and appends the next track.
///
/// The player removes a track from the queue when that track comes to the end.
/// The difference between the number of tracks that the engine appended and
/// the number that the queue holds gives the number of tracks that played.
fn advance(player: &mut Player, current: &mut Option<Current>, token: &str) {
    let item = match current.as_mut() {
        Some(item) => item,
        None => return,
    };

    let in_queue = player.len();

    if in_queue < item.queued {
        let played = item.queued - in_queue;
        item.playing += played;
        item.queued = in_queue;
    }

    if item.queued >= QUEUE_DEPTH {
        return;
    }

    if let Err(error) = fill_queue(player, item, token) {
        warn!("[worker] the engine cannot append the next track: {}", error);
    }
}

/// Writes the state that the user interface reads.
fn publish(
    player: &Player,
    current: &Option<Current>,
    state: &Arc<RwLock<PlaybackState>>,
    notice_until: &mut Option<Instant>,
) {
    let mut value = match state.write() {
        Ok(value) => value,
        Err(_) => return,
    };

    let item = match current {
        Some(item) => item,
        None => {
            value.status = PlaybackStatus::Stopped;
            return;
        }
    };

    let position = position_now(player, current);

    value.item_id = item.request.item_id.clone();
    value.title = item.request.title.clone();
    value.author = item.request.author.clone();
    value.position = position;
    value.duration = item.request.tracks.total_duration();
    value.chapter_title = item
        .request
        .tracks
        .chapter_at(position)
        .map(|chapter| chapter.title.clone());
    value.speed = player.speed();
    value.volume = player.volume();

    let was_stalled = value.status == PlaybackStatus::Stalled;

    value.status = if player.is_paused() {
        PlaybackStatus::Paused
    } else if player.empty() {
        PlaybackStatus::Stopped
    } else {
        PlaybackStatus::Playing
    };

    if was_stalled && value.status == PlaybackStatus::Playing {
        value.notice = Some("Reconnected".to_string());
        *notice_until = Some(Instant::now() + NOTICE_TIME);
    }

    if let Some(limit) = *notice_until {
        if Instant::now() > limit {
            value.notice = None;
            *notice_until = None;
        }
    }
}

/// Writes the status only.
fn set_status(state: &Arc<RwLock<PlaybackState>>, status: PlaybackStatus) {
    if let Ok(mut value) = state.write() {
        value.status = status;
    }
}
