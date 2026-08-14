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
pub fn handle_key_player(key: &str, player: &PlayerHandle, username: &str, server: &str) {
    let state = player.state();

    match key {
        // Change between the playback and the pause.
        " " => {
            if let Ok(Some(session)) = get_listening_session(username, server) {
                let value = if session.is_playback { "0" } else { "1" };

                // **A caller that reads no answer of its write says nothing at
                // all** (T-207). The row of `listening_session` is the one copy
                // of the state of this playback for a program that dies (T-201),
                // and the engine takes the pause below with no disk at all:
                // therefore a fault of this write takes a line of the log and no
                // word for the user (T-177).
                if let Err(error) = update_is_playback(value, session.id_session.as_str()) {
                    log::error!(
                        "[the pause of the player] the disk did not take the pause of the session                          {}: {}. The engine holds the media.",
                        session.id_session,
                        error
                    );
                }
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

        // More volume. The message says the new value: the screen held no
        // volume before T-80, therefore these two keys answered with nothing.
        "o" => {
            let value = player.change_the_volume(VOLUME_STEP);
            crate::logic::message::say(&crate::player::engine::the_sentence_of_the_volume(value));
        }

        // Less volume.
        "i" => {
            let value = player.change_the_volume(-VOLUME_STEP);
            crate::logic::message::say(&crate::player::engine::the_sentence_of_the_volume(value));
        }

        // More speed. The engine changes the speed during the playback, thus
        // the user does not start the playback again. See T-8.
        "O" => the_speed_changes(player, username, true, "O"),

        // Less speed.
        "I" => the_speed_changes(player, username, false, "I"),

        // Stop the playback.
        "Y" => player.send(PlayerCommand::Stop),

        _ => {}
    }
}

/// Changes the speed of the playback. The keys are `O` and `I`. See T-8 and
/// T-206.
///
/// **The row of the account is the truth of the speed.** The engine takes the
/// speed of that row, therefore a write that failed changes no speed at all: the
/// old shape gave the answer of `update_speed_rate` to nobody and it read the row
/// after it, and the two lines then gave the engine the speed of before. **The
/// key said nothing at all**, and it wrote no line of the log: a disk that the
/// program reads and cannot write held the speed of the user at its value, and
/// the user pressed the key again and again.
///
/// **A key of the user that writes the disk takes a sentence** (T-199), and that
/// sentence names the key of the view that the user sees at that moment (T-183).
fn the_speed_changes(player: &PlayerHandle, username: &str, faster: bool, key: &str) {
    if let Err(error) = update_speed_rate(username, faster) {
        log::warn!(
            "[the speed of the player] the program did not write the speed of {}: {}",
            username,
            error
        );

        crate::logic::message::say(&the_words_of_a_speed_that_the_disk_did_not_hold(key));

        return;
    }

    // **A read of the row that failed is not the speed 1.00x** (T-209). The old
    // line wrote `.parse::<f32>().unwrap_or(1.0)`, therefore a write that
    // reached the disk and a read that did not gave the engine a speed of no
    // account: the user pressed `O` for a faster media, and the media then
    // played more slowly than before. The disk holds the new speed in that
    // condition, and the engine does not: the program says so, and it changes
    // the engine no more.
    match get_speed_rate(username) {
        Ok(speed) => player.send(PlayerCommand::SetSpeed(speed)),
        Err(error) => {
            log::warn!(
                "[the speed of the player] the program did not read the speed of {}: {}",
                username,
                error
            );

            crate::logic::message::say(&the_words_of_a_speed_that_the_disk_did_not_give(key));
        }
    }
}

/// What the keys `O` and `I` say when the disk did not take the speed. See
/// T-206.
///
/// The function is pure, therefore a test needs no player and no database.
pub fn the_words_of_a_speed_that_the_disk_did_not_hold(key: &str) -> String {
    format!(
        "The program did not write the speed of this account: the database did not answer. The \
         speed does not change. Press {} again.",
        key
    )
}

/// What the keys `O` and `I` say when the disk took the speed and did not give
/// it back. See T-209.
///
/// **A write that the disk took and a read that failed are two conditions**
/// (T-91): the row of the account holds the new speed, and the engine holds the
/// speed of before. The sentence therefore names the media of now and not the
/// row of the disk, and it names the key of the view that the user sees (T-183).
///
/// The function is pure, therefore a test needs no player and no database.
pub fn the_words_of_a_speed_that_the_disk_did_not_give(key: &str) -> String {
    format!(
        "The program did not read the speed of this account: the database did not answer. This \
         media keeps its speed. Press {} again.",
        key
    )
}
