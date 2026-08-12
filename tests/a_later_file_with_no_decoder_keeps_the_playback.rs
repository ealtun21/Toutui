//! A book that holds one file that plays and one file that no decoder reads.
//! See T-120.
//!
//! **The book of the measurement is a real book of the user.** It holds the same
//! 26 hours two times: `02_..._AAC-LC.m4b` of 93285 seconds, and
//! `02_..._xHE-AAC.m4b` of 93278 seconds after it. The server therefore gives
//! two tracks and a media of 51 hours, and the place of the user (2 percent,
//! 3731 seconds) stands inside the **first** track.
//!
//! The measurement of 2026-08-12 against the server of the user, with the old
//! code:
//!
//! ```text
//! [play]   the item ... starts at 3731 seconds with 2 tracks
//! [worker] the engine cannot open the track 2 of 2: ... xHE-AAC ...
//!          The tracks before it play.
//! [worker] the playback starts at 3731 seconds          <- the book plays
//! [play]   no decoder of the program reads ... xHE-AAC.m4b. The program asks
//!          the server for a stream of the whole media.  <- and this ends it
//! [worker] the playback starts at 0 seconds
//! ```
//!
//! The user then heard the book again from the start, and the screen said "One
//! file needs the server" for a file that stands 26 hours after their place.
//! With the correction the log ends at "the playback starts at 3731 seconds",
//! and no stream comes.

use toutui::logic::playback::{the_stream_must_take_the_playback, TheStart};
use toutui::player::engine::{PlaybackState, PlaybackStatus};

/// The state of the engine for a playback that plays, and that met a fault of a
/// **later** file. The engine writes `playback_id` in the loop that follows a
/// playback that plays.
fn the_playback_plays_and_a_later_file_has_no_decoder(playback_id: u64) -> PlaybackState {
    PlaybackState {
        playback_id,
        status: PlaybackStatus::Playing,
        position: 3731.0,
        duration: 186564.03,
        file_with_no_decoder: Some("02_Depthless Hunger 2_[B0GGDKX4GP]_xHE-AAC.m4b".to_string()),
        playback_of_the_fault: playback_id,
        ..PlaybackState::default()
    }
}

/// The state of the engine for a playback that did **not** start. `fill_queue`
/// failed, the engine stopped the player, and it never wrote `playback_id` for
/// this playback: the state holds the identity of the playback before it.
fn the_playback_did_not_start(playback_id: u64, the_one_before: u64) -> PlaybackState {
    PlaybackState {
        playback_id: the_one_before,
        status: PlaybackStatus::Stopped,
        file_with_no_decoder: Some("01 - a file of wma.wma".to_string()),
        playback_of_the_fault: playback_id,
        ..PlaybackState::default()
    }
}

/// **This is the fault of the book of the user.** The old code read the flag of
/// the fault before it read "the engine plays this playback", therefore it gave
/// the name of the later file and the caller went to a stream of the server.
#[test]
fn a_later_file_with_no_decoder_does_not_end_a_playback_that_plays() {
    let state = the_playback_plays_and_a_later_file_has_no_decoder(7);

    assert_eq!(
        the_stream_must_take_the_playback(&state, 7),
        TheStart::ThePlaybackPlays,
        "the engine plays the track of the place of the user, and the book ends at the \
         track before the file of xHE-AAC"
    );
}

/// The condition that the stream of the server answers: the track that the
/// playback needs **now** does not open. See T-53.
#[test]
fn a_file_that_stops_the_start_asks_the_server_for_a_stream() {
    let state = the_playback_did_not_start(7, 6);

    assert_eq!(
        the_stream_must_take_the_playback(&state, 7),
        TheStart::TheFileNeedsTheServer("01 - a file of wma.wma".to_string()),
        "the playback is dead, therefore the stream of the server is the answer"
    );
}

/// The fault of a media that the user left must not touch this playback.
/// See T-53.
#[test]
fn the_fault_of_the_playback_before_it_says_nothing() {
    let state = PlaybackState {
        playback_id: 6,
        status: PlaybackStatus::Stopped,
        file_with_no_decoder: Some("a file of the media before it.wma".to_string()),
        playback_of_the_fault: 6,
        ..PlaybackState::default()
    };

    assert_eq!(
        the_stream_must_take_the_playback(&state, 7),
        TheStart::NoAnswerYet
    );
}

/// The engine did not answer yet. The loop must wait, and it must not go to a
/// stream.
#[test]
fn a_state_with_no_answer_makes_the_loop_wait() {
    assert_eq!(
        the_stream_must_take_the_playback(&PlaybackState::default(), 7),
        TheStart::NoAnswerYet
    );
}

/// A playback that plays and that met no fault at all.
#[test]
fn a_playback_with_no_fault_plays() {
    let state = PlaybackState {
        playback_id: 7,
        status: PlaybackStatus::Playing,
        ..PlaybackState::default()
    };

    assert_eq!(
        the_stream_must_take_the_playback(&state, 7),
        TheStart::ThePlaybackPlays
    );
}

/// A playback that the user paused still plays: the engine holds it, and a
/// fault of a later file must not end it.
#[test]
fn a_playback_that_the_user_paused_keeps_its_tracks() {
    let state = PlaybackState {
        status: PlaybackStatus::Paused,
        ..the_playback_plays_and_a_later_file_has_no_decoder(7)
    };

    assert_eq!(
        the_stream_must_take_the_playback(&state, 7),
        TheStart::ThePlaybackPlays
    );
}
