//! The keys of the player.
//!
//! The application sends a command to the engine. There is no separate
//! program, thus there is no remote control interface and no TCP connection.

use crate::db::crud::*;
use crate::player::engine::{PlaybackStatus, PlayerCommand, PlayerHandle};

/// The number of seconds of a jump.
const JUMP: f64 = 10.0;

/// The change of the volume for one key.
const VOLUME_STEP: f32 = 0.1;

/// Sends the command of a key to the engine.
pub fn handle_key_player(key: &str, player: &PlayerHandle, username: &str) {
    let state = player.state();

    match key {
        // Change between the playback and the pause.
        " " => {
            if let Ok(Some(session)) = get_listening_session() {
                let value = if session.is_playback { "0" } else { "1" };
                let _ = update_is_playback(value, session.id_session.as_str());
            }

            if state.status == PlaybackStatus::Paused {
                player.send(PlayerCommand::Resume);
            } else {
                player.send(PlayerCommand::Pause);
            }
        }

        // Jump forward.
        "p" => player.send(PlayerCommand::SeekBy(JUMP)),

        // Jump backward.
        "u" => player.send(PlayerCommand::SeekBy(-JUMP)),

        // The next chapter.
        "P" => player.send(PlayerCommand::NextChapter),

        // The chapter before this chapter.
        "U" => player.send(PlayerCommand::PreviousChapter),

        // More volume.
        "o" => player.send(PlayerCommand::SetVolume(state.volume + VOLUME_STEP)),

        // Less volume.
        "i" => player.send(PlayerCommand::SetVolume(state.volume - VOLUME_STEP)),

        // More speed. The engine changes the speed during the playback, thus
        // the user does not start the playback again. See T-8.
        "O" => {
            let _ = update_speed_rate(username, true);
            let speed = get_speed_rate(username).parse::<f32>().unwrap_or(1.0);
            player.send(PlayerCommand::SetSpeed(speed));
        }

        // Less speed.
        "I" => {
            let _ = update_speed_rate(username, false);
            let speed = get_speed_rate(username).parse::<f32>().unwrap_or(1.0);
            player.send(PlayerCommand::SetSpeed(speed));
        }

        // Stop the playback.
        "Y" => player.send(PlayerCommand::Stop),

        _ => {}
    }
}
