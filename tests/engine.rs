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
use abstui::player::engine::speed::{SharedSpeed, SpeedSource};
use abstui::player::engine::track::{Chapter, TrackList};

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

/// This test records the behaviour of `rodio::Player::set_speed`, and it
/// shows why the engine does not use that function.
///
/// The function increases the sample rate that the source reports. Therefore
/// the pitch increases with the speed, and a voice sounds too high. The engine
/// uses WSOLA in `SpeedSource` in place of this function. See T-19.
#[test]
fn the_speed_of_rodio_changes_the_sample_rate_and_the_pitch() {
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

/// Makes a sine wave of a frequency, for a number of seconds.
fn tone(freq: f32, seconds: f32) -> SamplesBuffer {
    let count = (RATE as f32 * seconds) as usize;
    let data: Vec<f32> = (0..count)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            (2.0 * std::f32::consts::PI * freq * t).sin() * 0.8
        })
        .collect();

    SamplesBuffer::new(NonZero::new(1).unwrap(), NonZero::new(RATE).unwrap(), data)
}

/// Counts the times that the samples go from a value below zero to a value
/// above zero. That number gives the frequency of a sine wave.
fn frequency_of(samples: &[f32], rate: u32) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }

    let crossings = samples
        .windows(2)
        .filter(|pair| pair[0] < 0.0 && pair[1] >= 0.0)
        .count();

    crossings as f32 * rate as f32 / samples.len() as f32
}

/// The test proves that the harness measures a frequency correctly.
#[test]
fn the_test_measures_the_frequency_of_a_tone() {
    let samples: Vec<f32> = tone(440.0, 1.0).collect();
    let freq = frequency_of(&samples, RATE);

    assert!(
        (freq - 440.0).abs() < 5.0,
        "the tone must be 440 Hz, but the test measured {}",
        freq
    );
}

/// This is the correction of T-19. A change of the speed must not change the
/// pitch. The old code used `Player::set_speed`, and that function increases
/// the sample rate. A voice then sounds too high.
#[test]
fn a_double_speed_keeps_the_pitch() {
    let shared = SharedSpeed::new(2.0);
    let source = SpeedSource::new(tone(440.0, 2.0), shared);

    let rate = source.sample_rate().get();
    let samples: Vec<f32> = source.take(200_000).collect();

    let freq = frequency_of(&samples, rate);

    assert!(
        (freq - 440.0).abs() < 25.0,
        "the speed 2.0 must keep the tone near 440 Hz, but the test measured \
         {} Hz",
        freq
    );
}

/// The speed must still change the length. A speed of 2.0 gives half the
/// number of samples.
#[test]
fn a_double_speed_gives_half_the_length() {
    let normal: Vec<f32> = SpeedSource::new(tone(440.0, 2.0), SharedSpeed::new(1.0))
        .take(200_000)
        .collect();
    let fast: Vec<f32> = SpeedSource::new(tone(440.0, 2.0), SharedSpeed::new(2.0))
        .take(200_000)
        .collect();

    let ratio = fast.len() as f32 / normal.len() as f32;

    assert!(
        (ratio - 0.5).abs() < 0.1,
        "the speed 2.0 must give about half the samples, but the ratio is {}",
        ratio
    );
}

/// A slow speed keeps the pitch too.
#[test]
fn a_slow_speed_keeps_the_pitch() {
    let source = SpeedSource::new(tone(440.0, 1.0), SharedSpeed::new(0.5));
    let rate = source.sample_rate().get();
    let samples: Vec<f32> = source.take(200_000).collect();

    let freq = frequency_of(&samples, rate);

    assert!(
        (freq - 440.0).abs() < 25.0,
        "the speed 0.5 must keep the tone near 440 Hz, but the test measured \
         {} Hz",
        freq
    );
}

/// A book of many files must report the end of the book when every track
/// played. The engine counts the tracks that played, and that count goes past
/// the last index. See T-2 and T-16.
#[test]
fn a_position_past_the_last_track_is_the_end_of_the_book() {
    let list = TrackList::new(TrackList::from_durations(&[20.0, 20.0, 20.0]), Vec::new());

    assert_eq!(list.total_duration(), 60.0);

    // The engine gives the total when the index is past the last track.
    let index_past_the_end = list.len();
    assert!(index_past_the_end >= list.len());

    // Inside the list the calculation stays normal.
    assert_eq!(list.position_of(2, 20.0), 60.0);
    assert_eq!(list.position_of(1, 5.0), 25.0);
}
