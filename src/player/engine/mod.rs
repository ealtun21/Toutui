//! The audio engine of the application.
//!
//! The engine decodes the audio in the process. The application does not start
//! a different program. Therefore the token stays in the memory of the
//! process.

pub mod http_file;
pub mod pos_probe;
pub mod source;
pub mod track;
pub mod worker;

use crate::player::engine::source::TrackSource;
use crate::player::engine::track::TrackList;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Changes the position that `rodio` reports to the position in the media.
///
/// `rodio::Player::get_pos` gives the position of the sound that plays now.
/// The test module `pos_probe` measures the behaviour of that function when
/// the speed is not 1.0. The engine must report the position in the media to
/// the server, because the server counts the seconds of the recording and not
/// the seconds of the listener.
pub fn media_position(reported: Duration, speed: f32) -> f64 {
    let seconds = reported.as_secs_f64();

    if speed <= 0.0 || !speed.is_finite() {
        return seconds;
    }

    // A measurement on 2026-08-10 shows that `get_pos` gives the time of the
    // listener. A sound of 1.0 second at the speed 2.0 gives 0.5 seconds.
    // The server counts the seconds of the recording. Therefore the function
    // multiplies by the speed. The test module `pos_probe` holds the
    // measurement.
    seconds * speed as f64
}

/// Changes a position in the media to the value that `try_seek` needs.
///
/// This function is the opposite of `media_position`. `rodio` multiplies the
/// value of `try_seek` by the speed, and it divides the value of `get_pos` by
/// the speed. Therefore the engine must divide before it moves.
///
/// A test against a real book found this fault. The engine moved to the
/// position 7640 seconds at the speed 1.1, and the reader went to the byte of
/// the second 8404. The book then came to the end, and the playback stopped.
pub fn seek_target(media_seconds: f64, speed: f32) -> Duration {
    let seconds = media_seconds.max(0.0);

    if speed <= 0.0 || !speed.is_finite() {
        return Duration::from_secs_f64(seconds);
    }

    Duration::from_secs_f64(seconds / speed as f64)
}

/// What the engine does now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    /// The engine plays no media.
    Stopped,
    /// The engine plays the media.
    Playing,
    /// The user stopped the playback.
    Paused,
    /// The buffer is empty, and the engine waits for data. The user did not
    /// stop the playback. The engine continues without an action of the user.
    Stalled,
}

/// What the user interface shows.
#[derive(Debug, Clone)]
pub struct PlaybackState {
    /// The identity of the book.
    pub item_id: String,
    pub title: String,
    pub author: String,
    /// The position in the whole book, in seconds.
    pub position: f64,
    /// The length of the whole book, in seconds.
    pub duration: f64,
    /// The name of the chapter, if the book has chapters.
    pub chapter_title: Option<String>,
    pub speed: f32,
    pub volume: f32,
    pub status: PlaybackStatus,
    /// A message for the user. An example is "Reconnected".
    pub notice: Option<String>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState {
            item_id: String::new(),
            title: String::new(),
            author: String::new(),
            position: 0.0,
            duration: 0.0,
            chapter_title: None,
            speed: 1.0,
            volume: 1.0,
            status: PlaybackStatus::Stopped,
            notice: None,
        }
    }
}

/// All the data that the engine needs to play a book.
#[derive(Debug, Clone)]
pub struct PlaybackRequest {
    pub item_id: String,
    pub title: String,
    pub author: String,
    pub username: String,
    /// The tracks and the chapters of the book.
    pub tracks: TrackList,
    /// The source of each track. The sequence agrees with the tracks.
    pub sources: Vec<TrackSource>,
    /// Where the playback starts, in seconds from the start of the book.
    pub start_position: f64,
    pub speed: f32,
}

/// A command for the engine.
#[derive(Debug, Clone)]
pub enum PlayerCommand {
    /// Starts a book. The engine stops the book that plays now.
    Start(Box<PlaybackRequest>),
    Pause,
    Resume,
    /// Moves to a position in the book, in seconds.
    SeekTo(f64),
    /// Moves forward or backward, in seconds.
    SeekBy(f64),
    NextChapter,
    PreviousChapter,
    SetSpeed(f32),
    SetVolume(f32),
    /// Stops the playback and empties the queue.
    Stop,
}

/// The connection to the engine.
///
/// The handle sends commands. It also gives the state that the user interface
/// reads.
#[derive(Debug, Clone)]
pub struct PlayerHandle {
    sender: Sender<PlayerCommand>,
    state: Arc<RwLock<PlaybackState>>,
}

impl PlayerHandle {
    /// Starts the engine.
    ///
    /// The function opens the sound card. It gives an error if the computer
    /// has no sound card.
    pub fn start(token: String) -> Result<PlayerHandle, String> {
        let (sender, receiver) = channel();
        let state = Arc::new(RwLock::new(PlaybackState::default()));

        worker::spawn(receiver, Arc::clone(&state), token)?;

        Ok(PlayerHandle { sender, state })
    }

    /// Sends a command. The function does not wait for the engine.
    pub fn send(&self, command: PlayerCommand) {
        if self.sender.send(command).is_err() {
            log::error!("[PlayerHandle] the engine stopped");
        }
    }

    /// Gives a copy of the state.
    pub fn state(&self) -> PlaybackState {
        match self.state.read() {
            Ok(state) => state.clone(),
            Err(_) => PlaybackState::default(),
        }
    }

    /// Gives the state that the user interface reads for each frame.
    pub fn shared_state(&self) -> Arc<RwLock<PlaybackState>> {
        Arc::clone(&self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::{media_position, seek_target};
    use std::time::Duration;

    #[test]
    fn a_normal_speed_does_not_change_the_position() {
        let position = media_position(Duration::from_secs(30), 1.0);
        assert!((position - 30.0).abs() < 0.001);
    }

    /// `get_pos` gives the time of the listener. A listener that uses the
    /// speed 2.0 for 30 seconds hears 60 seconds of the recording. The server
    /// must get the value 60.
    #[test]
    fn a_double_speed_gives_the_position_in_the_media() {
        let position = media_position(Duration::from_secs(30), 2.0);
        assert!((position - 60.0).abs() < 0.001);
    }

    /// A speed that is less than 1.0 gives a position that is smaller than
    /// the time of the listener.
    #[test]
    fn a_slow_speed_gives_a_smaller_position() {
        let position = media_position(Duration::from_secs(30), 0.5);
        assert!((position - 15.0).abs() < 0.001);
    }

    /// A speed of zero, or a speed that is not a number, must not give a
    /// position of zero and must not give an infinite value.
    #[test]
    fn a_speed_that_is_not_valid_gives_the_reported_position() {
        assert!((media_position(Duration::from_secs(30), 0.0) - 30.0).abs() < 0.001);
        assert!((media_position(Duration::from_secs(30), -1.0) - 30.0).abs() < 0.001);
        assert!((media_position(Duration::from_secs(30), f32::NAN) - 30.0).abs() < 0.001);
    }
    /// `try_seek` and `get_pos` are opposites. `rodio` multiplies the value of
    /// `try_seek` by the speed, and it divides the value of `get_pos`.
    #[test]
    fn the_seek_target_is_the_opposite_of_the_position() {
        let media = 7640.0;
        let speed = 1.1;

        let target = seek_target(media, speed);
        let back = media_position(target, speed);

        assert!(
            (back - media).abs() < 0.001,
            "the two functions must give the same media position, but the \
             result is {}",
            back
        );
    }

    /// This is the fault that a test against a real book found. The engine
    /// moved to 7640 seconds at the speed 1.1, and `rodio` went to 8404
    /// seconds. The book then came to the end.
    #[test]
    fn the_seek_target_does_not_go_past_the_end() {
        let target = seek_target(7640.0, 1.1).as_secs_f64();

        assert!(
            (target - 6945.45).abs() < 0.1,
            "the target must be smaller than the media position, but it is {}",
            target
        );
    }

    #[test]
    fn a_normal_speed_does_not_change_the_seek_target() {
        assert!((seek_target(30.0, 1.0).as_secs_f64() - 30.0).abs() < 0.001);
    }

    #[test]
    fn a_seek_target_with_a_speed_that_is_not_valid_uses_the_media_position() {
        assert!((seek_target(30.0, 0.0).as_secs_f64() - 30.0).abs() < 0.001);
        assert!((seek_target(30.0, f32::NAN).as_secs_f64() - 30.0).abs() < 0.001);
    }

    /// A position before the start must not make a negative duration.
    #[test]
    fn a_negative_position_gives_zero() {
        assert_eq!(seek_target(-5.0, 1.0).as_secs_f64(), 0.0);
    }
}
