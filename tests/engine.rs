//! Tests of the queue and the position.
//!
//! The tests use `Player::new`, thus they need no sound card. The continuous
//! integration machine has no sound card.
//!
//! Two properties of `rodio` are important here:
//!
//! - `Player::new` makes a queue that stays alive when it is empty. Therefore
//!   the output gives silence for ever, and a test must never read to the end.
//! - The test sound is not silent. Therefore a test finds the end of the
//!   sound at the first silent sample.

use rodio::buffer::SamplesBuffer;
use rodio::source::Source;
use rodio::Player;
use std::num::NonZero;
use toutui::player::engine::track::{Chapter, TrackList};

/// The value of each sample of the test sound.
const LEVEL: f32 = 0.5;

/// The sample rate of the test sound. 8000 samples give 1 second.
const RATE: u32 = 8000;

/// The largest number of samples that a test reads.
const LIMIT: usize = 80_000;

/// Makes a sound of a number of seconds. The sound has one channel.
fn sound(seconds: usize) -> SamplesBuffer {
    SamplesBuffer::new(
        NonZero::new(1).unwrap(),
        NonZero::new(RATE).unwrap(),
        vec![LEVEL; RATE as usize * seconds],
    )
}

/// Gives the number of samples before the first silence.
fn count_sound(output: &mut dyn Iterator<Item = f32>) -> usize {
    let mut count = 0;

    for sample in output.take(LIMIT) {
        if sample == 0.0 {
            break;
        }
        count += 1;
    }

    count
}

/// This is the behaviour that T-2 needs. A book with more than one audio file
/// must play all the files, and not the first file only.
#[test]
fn the_queue_plays_two_tracks_one_after_the_other() {
    let (player, mut output) = Player::new();
    player.append(sound(1));
    player.append(sound(1));
    player.play();

    assert_eq!(player.len(), 2);

    let total = count_sound(&mut output);

    assert!(
        (15_900..=16_100).contains(&total),
        "two sounds of 1 second give approximately 16000 samples, but they \
         gave {}",
        total
    );
}

/// The queue becomes shorter when a track comes to the end. The engine uses
/// this value to know the track that plays now.
#[test]
fn the_queue_becomes_shorter_after_a_track_ends() {
    let (player, mut output) = Player::new();
    player.append(sound(1));
    player.append(sound(1));
    player.play();

    assert_eq!(player.len(), 2);

    let _ = count_sound(&mut output);

    assert_eq!(player.len(), 0);
}

#[test]
fn the_position_of_the_book_uses_the_start_offset_of_the_track() {
    let list = TrackList::new(TrackList::from_durations(&[10.0, 20.0, 30.0]), Vec::new());

    // The engine plays the second track, at 5 seconds inside that track.
    assert_eq!(list.position_of(1, 5.0), 15.0);
    assert_eq!(list.locate(15.0).unwrap(), (1, 5.0));
}

#[test]
fn a_chapter_movement_gives_a_position_in_the_book() {
    let chapters = vec![
        Chapter {
            start: 0.0,
            end: 25.0,
            title: "One".to_string(),
        },
        Chapter {
            start: 25.0,
            end: 60.0,
            title: "Two".to_string(),
        },
    ];
    let list = TrackList::new(TrackList::from_durations(&[10.0, 20.0, 30.0]), chapters);

    assert_eq!(list.next_chapter_start(10.0).unwrap(), 25.0);
    assert_eq!(list.chapter_at(30.0).unwrap().title, "Two");
}

/// The speed must change during the playback. This is the correction of T-8.
/// The user must not start the playback again.
#[test]
fn the_speed_changes_during_the_playback() {
    let (player, mut output) = Player::new();
    player.append(sound(2));
    player.play();

    let first_rate = output.sample_rate();

    // Take some samples at the normal speed.
    for _ in 0..1000 {
        let _ = output.next();
    }

    player.set_speed(2.0);
    assert_eq!(player.speed(), 2.0);

    // The queue reads the values of the source again when it gives the next
    // sample. Therefore the test takes one sample before it reads the rate.
    let _ = output.next();

    // `rodio` does not remove samples for a higher speed. It increases the
    // sample rate that the source reports. The sound card then plays the same
    // samples in half the time.
    let second_rate = output.sample_rate();

    assert!(
        second_rate > first_rate,
        "the speed 2.0 must increase the sample rate from {} but it gave {}",
        first_rate,
        second_rate
    );
}

/// The pause command stops the sound, and the resume command starts it again.
#[test]
fn the_pause_command_and_the_resume_command_operate() {
    let (player, _output) = Player::new();
    player.append(sound(1));
    player.play();

    assert!(!player.is_paused());

    player.pause();
    assert!(player.is_paused());

    player.play();
    assert!(!player.is_paused());
}
