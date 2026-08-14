//! The thread that owns the audio.
//!
//! One thread owns the `rodio::Player`. It reads the commands, and it writes
//! the state. No other thread touches the player.
//!
//! The thread appends two tracks only. A book of the test library has 209
//! audio files, and 209 open connections are not acceptable.

use crate::player::engine::source::open_decoder;
use crate::player::engine::speed::{SharedSpeed, SpeedSource};
use crate::player::engine::{
    is_complete, media_position, reached_the_end, seek_target, PlaybackRequest, PlaybackState,
    PlaybackStatus, PlayerCommand,
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
    let sink = open_sink()?;

    std::thread::Builder::new()
        .name("toutui-audio".to_string())
        .spawn(move || run(receiver, state, token, sink))
        .map_err(|error| format!("The application cannot start the audio thread: {}", error))?;

    Ok(())
}

/// The name of the variable that selects the sound device.
pub const DEVICE_VARIABLE: &str = "TOUTUI_AUDIO_DEVICE";

/// Opens the sound device.
///
/// The application uses the default device of the operating system. If the
/// variable `TOUTUI_AUDIO_DEVICE` holds a name, the application uses the
/// device that has that name. A computer with more than one sound card needs
/// this variable.
///
/// The value `null` on Linux gives a device that discards the sound. A test
/// uses that value, thus the test makes no sound.
fn open_sink() -> Result<MixerDeviceSink, String> {
    let wanted = match std::env::var(DEVICE_VARIABLE) {
        Ok(name) if !name.trim().is_empty() => name.trim().to_string(),
        _ => {
            return DeviceSinkBuilder::open_default_sink()
                .map_err(|error| format!("The application cannot open the sound card: {}", error))
        }
    };

    let host = rodio::cpal::default_host();

    let devices = rodio::cpal::traits::HostTrait::output_devices(&host)
        .map_err(|error| format!("The application cannot read the sound cards: {}", error))?;

    let mut known: Vec<String> = Vec::new();

    for device in devices {
        // `id()` gives a stable identifier, and it has the form
        // `alsa:null`. `name()` is deprecated. Therefore the comparison uses
        // the identifier, and it also accepts the part after the colon.
        let id = match rodio::DeviceTrait::id(&device) {
            Ok(id) => id.to_string(),
            Err(_) => continue,
        };

        let short = id.rsplit(':').next().unwrap_or(&id).to_string();
        known.push(id.clone());

        if id == wanted || short == wanted {
            info!("[worker] the application uses the sound device {}", id);

            return DeviceSinkBuilder::from_device(device)
                .and_then(|builder| builder.open_stream())
                .map_err(|error| {
                    format!(
                        "The application cannot open the device {}: {}",
                        wanted, error
                    )
                });
        }
    }

    Err(format!(
        "The application cannot find the sound device {}. Remove the variable \
         {} to use the default device. These devices are available: {}.",
        wanted,
        DEVICE_VARIABLE,
        known.join(", ")
    ))
}

/// What the thread plays now.
struct Current {
    request: PlaybackRequest,
    /// The track that plays now.
    playing: usize,
    /// The number of tracks in the queue.
    queued: usize,
    /// The number of tracks that the engine can play, from the first one.
    ///
    /// This value is the number of tracks of the book, and it becomes smaller
    /// when the decoder does not read a track. The playback then stops at the
    /// track before it, and the engine asks the server for that file no more.
    /// See T-48.
    tracks_that_play: usize,
    /// The name of the file that the decoder does not read, if one exists. The
    /// screen shows it, therefore the user knows why the playback stops early.
    the_file_that_no_decoder_reads: Option<String>,
    /// The playback reads the stream of the server, and not the file. See T-53.
    plays_the_stream_of_the_server: bool,
    /// The place of the media where the bytes of the track that plays start, in
    /// seconds.
    ///
    /// A file starts at the start of its track and gives 0. **The stream of the
    /// server starts at a part of the playlist**, therefore the position of the
    /// decoder is the position inside that part and not the position of the media.
    /// See T-63.
    offset_of_the_bytes: f64,
    /// The box where the reader of the stream of the server says what it
    /// reached.
    ///
    /// **A stream that stopped before its last part is not the end of the
    /// media.** The engine reads this box at the end of the tracks, therefore
    /// it never gives the end of the whole media for a stream that stopped. See
    /// T-194.
    the_stream: Option<std::sync::Arc<crate::player::engine::hls_file::StreamReport>>,
    /// The speed that every track of this book reads. WSOLA stretches the
    /// time, thus the pitch does not change.
    speed: SharedSpeed,
}

impl Current {
    /// Gives the report of a stream of the server that did not reach its last
    /// part. See T-194.
    fn the_stream_stopped(&self) -> Option<crate::player::engine::hls_file::TheStreamStopped> {
        self.the_stream
            .as_ref()
            .and_then(|report| report.the_stream_stopped())
    }
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
            Ok(command) => {
                // The engine holds this command now, therefore no wait of the
                // engine must stop for it. See T-68.
                crate::player::engine::the_engine_took_the_command();
                handle(command, &mut player, &sink, &mut current, &token, &state)
            }
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
        PlayerCommand::SetSpeed(value) => {
            if let Some(item) = current.as_ref() {
                // WSOLA reads this value on the next sample. The playback
                // does not start again. See T-8 and T-19.
                item.speed.set(value);
            }
        }
        PlayerCommand::SetVolume(value) => player.set_volume(value.clamp(0.0, 2.0)),
        PlayerCommand::Stop => {
            player.stop();
            *current = None;

            // The user stopped the media. It is not finished.
            if let Ok(mut value) = state.write() {
                value.status = PlaybackStatus::Stopped;
                value.finished = false;
            }
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

    // A new player gives an empty queue. The speed of the player stays 1.0,
    // because `Player::set_speed` changes the pitch. WSOLA in `SpeedSource`
    // changes the speed and keeps the pitch. See T-19.
    *player = Player::connect_new(sink.mixer());

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

    let speed = SharedSpeed::new(request.speed);

    let tracks_that_play = item_count_of(&request);

    // A new playback starts with no fault of a decoder. See T-53.
    if let Ok(mut value) = state.write() {
        value.file_with_no_decoder = None;
        value.why_the_start_did_not_work = None;
        value.playback_of_the_fault = 0;
    }

    let plays_the_stream_of_the_server = request.sources.iter().any(|source| {
        matches!(
            source,
            crate::player::engine::source::TrackSource::Stream { .. }
        )
    });

    let mut item = Current {
        request,
        playing: track_index,
        queued: 0,
        speed,
        tracks_that_play,
        the_file_that_no_decoder_reads: None,
        plays_the_stream_of_the_server,
        offset_of_the_bytes: 0.0,
        the_stream: None,
    };

    if let Err(error) = fill_queue(player, &mut item, token) {
        error!("[worker] the engine cannot start the book: {}", error);

        // The loop of the playback reads this name, and it then asks the server
        // for a stream of the whole media. See T-53.
        let name = item
            .request
            .tracks
            .get(track_index)
            .map(|track| track.filename.clone())
            .unwrap_or_default();

        if let Ok(mut value) = state.write() {
            value.file_with_no_decoder = Some(name);
            // The place that met the fault wrote this sentence. The loop of the
            // playback gives it to the user, therefore the user reads the true
            // cause and not a guess. See T-68.
            value.why_the_start_did_not_work = Some(error.clone());
            value.playback_of_the_fault = item.request.playback_id;
        }

        // The queue of the player plays a track as soon as the engine appends
        // it. A start that ends here must therefore stop the player, or the
        // sound goes on while the screen shows no player and no position goes
        // to the server. See T-48.
        player.stop();
        set_status(state, PlaybackStatus::Stopped);
        *current = None;
        return;
    }

    if offset > 0.0 {
        if let Err(error) = player.try_seek(seek_target(offset, item.speed.get())) {
            warn!("[worker] the engine cannot move to the position: {}", error);
        }
    }

    player.play();
    info!("[worker] the playback starts at {} seconds", start_position);

    *current = Some(item);
}

/// Gives the number of tracks of a request.
fn item_count_of(request: &PlaybackRequest) -> usize {
    request.tracks.len()
}

/// Gives the position in the book.
fn position_now(player: &Player, current: &Option<Current>) -> f64 {
    let item = match current {
        Some(item) => item,
        None => return 0.0,
    };

    // Every track that plays played. The position is then the end of those
    // tracks. Two rules stand behind this one:
    //
    // 1. Without it, `position_of` gets an index that does not exist and it
    //    gives the offset in the last track only. A book of many files then
    //    never reaches its end, and the application does not mark it as
    //    finished. See T-2 and T-16.
    // 2. The queue of the player goes on counting when it is empty. A book that
    //    ends at a file that no decoder reads would then send a position that
    //    the user never heard, and the position would reach the end of the
    //    **whole** book. See T-48 and T-55.
    if item.playing >= item.tracks_that_play {
        return crate::player::engine::the_place_of_the_end(
            item.the_stream_stopped().map(|stop| stop.seconds),
            item.request.tracks.end_of_the_first(item.tracks_that_play),
        );
    }

    let inside = media_position(player.get_pos(), item.speed.get());

    // The stream of the server starts inside the media, therefore the position of
    // the decoder is not the position of the media. A file gives the offset 0 and
    // this sum does not change it. See T-63.
    item.request.tracks.position_of(item.playing, inside) + item.offset_of_the_bytes
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

    // **The stream of the server moves forward only.** A movement therefore asks
    // the server for the stream again, at the new place: the reader takes the part
    // of the playlist that holds it. A `try_seek` of such a source gives a fault,
    // and the playback then stayed where it was. See T-63.
    if let Some(crate::player::engine::source::TrackSource::Stream { seconds, .. }) =
        item.request.sources.get_mut(track_index)
    {
        *seconds = position.max(0.0);

        player.clear();
        item.playing = track_index;
        item.queued = 0;

        if let Err(error) = fill_queue(player, item, token) {
            error!(
                "[worker] the engine cannot ask for the stream again: {}",
                error
            );
            return;
        }

        player.play();

        info!(
            "[worker] the stream of the server starts again at {} seconds",
            position.round()
        );

        return;
    }

    if track_index == item.playing {
        if let Err(error) = player.try_seek(seek_target(offset, item.speed.get())) {
            warn!(
                "[worker] the engine cannot move inside the track: {}",
                error
            );
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
        if let Err(error) = player.try_seek(seek_target(offset, item.speed.get())) {
            warn!(
                "[worker] the engine cannot move inside the track: {}",
                error
            );
        }
    }

    player.play();
}

/// Tells if the fault of one track stops the whole playback. See T-48.
///
/// The queue holds `QUEUE_DEPTH` tracks, therefore the engine opens the track
/// that plays now **and the track after it**. A book of two files can hold one
/// file that the decoder reads and one file that it does not read: a book of a
/// user on 2026-08-11 held the same audio two times, as AAC-LC and as xHE-AAC,
/// and symphonia reads AAC-LC only.
///
/// The fault of the track that plays now stops the playback, because no sound
/// can come. The fault of a track after it must not: the user hears the first
/// 26 hours, and the engine tries that track again at each tick.
pub fn the_fault_stops_the_playback(queued_before_the_fault: usize) -> bool {
    queued_before_the_fault == 0
}

/// Appends tracks until the queue holds `QUEUE_DEPTH` tracks.
///
/// A track that the decoder does not read stops the filling. The function then
/// gives `Ok`, because the tracks before it play. See `the_fault_stops_the_playback`.
fn fill_queue(player: &mut Player, item: &mut Current, token: &str) -> Result<(), String> {
    while item.queued < QUEUE_DEPTH {
        let track_index = item.playing + item.queued;

        // A track that no decoder reads ends the book. The engine must not ask
        // the server for that file again at each tick. See T-48.
        if track_index >= item.tracks_that_play {
            break;
        }

        let track = match item.request.tracks.get(track_index) {
            Some(track) => track.clone(),
            None => break,
        };

        let source = match item.request.sources.get(track_index) {
            Some(source) => source.clone(),
            None => break,
        };

        let opened = match open_decoder(&source, token, &track) {
            Ok(opened) => opened,
            Err(error) => {
                if the_fault_stops_the_playback(item.queued) {
                    return Err(error);
                }

                warn!(
                    "[worker] the engine cannot open the track {} of {}: {}. \
                     The tracks before it play.",
                    track_index + 1,
                    item.request.tracks.len(),
                    error
                );

                item.tracks_that_play = track_index;
                item.the_file_that_no_decoder_reads = Some(track.filename.clone());

                return Ok(());
            }
        };

        // The bytes of the track that plays give the place of the media. See
        // T-63.
        if item.queued == 0 {
            item.offset_of_the_bytes = opened.offset;
            item.the_stream = opened.the_stream.clone();
        }

        player.append(SpeedSource::new(opened.source, item.speed.clone()));
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
        warn!(
            "[worker] the engine cannot append the next track: {}",
            error
        );
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

    // The chapters do not change while one media plays, therefore the state
    // takes them one time. A copy at each tick would copy the name of every
    // chapter twenty times each second. See T-24.
    if value.playback_id != item.request.playback_id {
        value.chapters = item.request.tracks.chapters().to_vec();

        // A new playback starts with no fault of a decoder and with no message
        // of the playback before it. See T-53.
        value.file_with_no_decoder = None;
        value.why_the_start_did_not_work = None;
        value.why_the_stream_stopped = None;
        value.notice = None;
    }

    value.playback_id = item.request.playback_id;
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
    value.speed = item.speed.get();
    value.volume = player.volume();

    let was_stalled = value.status == PlaybackStatus::Stalled;

    // The queue becomes empty for a short time between two tracks. Therefore
    // an empty queue alone does not mean that the playback is complete. See
    // T-2.
    // A track that no decoder reads ends the book at the track before it.
    // Therefore the count of this rule is the count of the tracks that play,
    // and not the count of the tracks of the book. See T-48.
    let complete = is_complete(player.empty(), item.playing, item.tracks_that_play);

    value.status = if player.is_paused() {
        PlaybackStatus::Paused
    } else if complete {
        PlaybackStatus::Stopped
    } else {
        PlaybackStatus::Playing
    };

    // The media came to its end only if no track stays and the position is at
    // the end. See T-16.
    value.finished = reached_the_end(position, value.duration, complete);

    // The user must know why a book stops before its end. See T-48. The loop of
    // the playback reads the name, and it asks the server for a stream of the
    // whole media. See T-53.
    if let Some(name) = &item.the_file_that_no_decoder_reads {
        value.notice = Some(format!("The program cannot read {}", name));
        value.file_with_no_decoder = Some(name.clone());
        value.playback_of_the_fault = item.request.playback_id;
    }

    // A playback of the stream of the server waits for ffmpeg of that server.
    // The user must know why, therefore the panel says it. See T-53.
    if item.plays_the_stream_of_the_server {
        value.notice = Some("The server makes the stream of this media".to_string());
    }

    // **A stream that stopped before its last part is not the end of the
    // media.** The old program said nothing at all: the book of the user
    // stopped in the middle, the program wrote the whole place of the media,
    // and it told the server that the user finished the book. See T-194.
    if let Some(stop) = item.the_stream_stopped() {
        value.notice = Some(stop.why.clone());
        value.why_the_stream_stopped = Some(stop.why);
    }

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

#[cfg(test)]
mod tests {
    use super::the_fault_stops_the_playback;
    use crate::player::engine::is_complete;

    /// A book of a user on 2026-08-11 held the same audio two times: one file
    /// of AAC-LC, and one file of xHE-AAC. symphonia reads AAC-LC only.
    ///
    /// The queue holds two tracks, therefore the engine opened the second file
    /// at the start and the whole start then failed. The first file was in the
    /// queue of the player already, and the queue of the player plays a track
    /// as soon as the engine appends it. Therefore the sound came, the state
    /// said `Stopped`, the screen showed no player, and no position went to the
    /// server. See T-48.
    #[test]
    fn the_fault_of_a_later_track_does_not_stop_the_playback() {
        // No track plays yet. No sound can come, therefore the playback stops.
        assert!(the_fault_stops_the_playback(0));

        // One track plays already. The user hears that track.
        assert!(!the_fault_stops_the_playback(1));
        assert!(!the_fault_stops_the_playback(2));
    }

    /// The book ends at the track before the track that no decoder reads.
    /// Therefore the rule of the end counts the tracks that play, and not the
    /// tracks of the book.
    #[test]
    fn the_book_ends_at_the_track_that_no_decoder_reads() {
        // Two tracks, and the second one has no decoder. The first track
        // played, and the queue is empty.
        let tracks_that_play = 1;
        assert!(is_complete(true, 1, tracks_that_play));

        // The old rule counted the two tracks of the book. The state then said
        // `Playing` for ever with an empty queue.
        assert!(!is_complete(true, 1, 2));

        // A book with no such fault does not change.
        assert!(!is_complete(true, 1, 3));
        assert!(is_complete(true, 3, 3));
    }
}
