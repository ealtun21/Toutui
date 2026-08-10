//! Tests of the Opus source. See T-17.
//!
//! `rodio::Decoder` cannot play Opus, because it uses the fixed codec registry
//! of symphonia. The project reads the packets with symphonia and decodes them
//! with `opuscule`.
//!
//! A measurement on 2026-08-10 compared the samples of that path with libopus
//! over 50 files. The largest difference of one sample is 0.00002 of a full
//! scale of 1.0. These tests hold the numbers of that measurement. A change of
//! the decoder that changes the audio therefore gives a failure here.
//!
//! `ffmpeg` made the test files:
//!
//! ```text
//! ffmpeg -f lavfi -i "sine=frequency=440:duration=0.5:sample_rate=48000" \
//!     -ac 1 -c:a libopus -b:a 32k -f FORMAT -strict -2 FILE
//! ```
//!
//! `FORMAT` is `webm`, `matroska`, or `mp4`. The files `tone.opus`,
//! `tone_stereo.opus`, and `tone_opus.ogg` come from the tests before this
//! measurement.

use rodio::Source;
use std::path::{Path, PathBuf};
use toutui::player::engine::opus::OpusSource;
use toutui::player::engine::source::{open_decoder, EngineSource, TrackSource};
use toutui::player::engine::track::Track;

/// The number of samples that a file of 0.5 seconds gives.
///
/// A tone of 0.5 seconds at 48000 samples each second holds 24000 samples. The
/// encoder adds padding to make the last frame complete, and symphonia 0.5.5
/// does not remove that padding. Therefore the source gives 648 samples more.
/// See the head of `src/player/engine/opus.rs`.
const SAMPLES_OF_HALF_A_SECOND: usize = 24648;

/// The highest value of the files that `ffmpeg` made on 2026-08-10. libopus
/// gives 0.12692 for the same files.
const PEAK_OF_THE_CONTAINERS: f32 = 0.12692;

fn fixture(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/audio")
        .join(name);

    assert!(path.exists(), "the test file {} is absent", name);
    path
}

fn track_for(path: &Path) -> Track {
    Track {
        index: 1,
        ino: "test".to_string(),
        filename: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        mime_type: None,
        size: None,
        duration: 0.5,
        start_offset: 0.0,
    }
}

/// Opens a file with the source that the engine chooses, and gives the samples.
fn samples_of(name: &str) -> (EngineSourceKind, Vec<f32>) {
    let path = fixture(name);
    let track = track_for(&path);

    let source = open_decoder(&TrackSource::Local(path), "no-token", &track)
        .unwrap_or_else(|error| panic!("the file {} did not open: {}", name, error));

    let kind = match &source {
        EngineSource::Rodio(_) => EngineSourceKind::Rodio,
        EngineSource::Opus(_) => EngineSourceKind::Opus,
    };

    (kind, source.take(4_000_000).collect())
}

#[derive(Debug, PartialEq)]
enum EngineSourceKind {
    Rodio,
    Opus,
}

fn peak_of(pcm: &[f32]) -> f32 {
    pcm.iter()
        .fold(0.0f32, |highest, value| highest.max(value.abs()))
}

/// Every container that holds Opus must give the Opus source.
#[test]
fn every_container_that_holds_opus_gives_the_opus_source() {
    for name in [
        "tone.opus",
        "tone_stereo.opus",
        "tone_opus.ogg",
        "tone_opus.mka",
        "tone_opus.webm",
        "tone_opus.mp4",
    ] {
        let (kind, pcm) = samples_of(name);

        assert_eq!(
            kind,
            EngineSourceKind::Opus,
            "the file {} must play with the Opus source",
            name
        );
        assert!(!pcm.is_empty(), "the file {} gave no sample", name);
    }
}

/// A file with the extension `ogg` that holds Vorbis must still use
/// `rodio::Decoder`. The Opus source must not take a file that is not Opus.
#[test]
fn a_vorbis_file_does_not_use_the_opus_source() {
    let (kind, pcm) = samples_of("tone.ogg");

    assert_eq!(kind, EngineSourceKind::Rodio);
    assert!(pcm.len() > 500);
}

/// The three containers hold the same audio, therefore they must give the same
/// number of samples and the same highest value.
///
/// A measurement on 2026-08-10 found a fault of the MP4 container. The head of
/// the stream holds the number of samples that the stream skips at its start,
/// and the box `dOps` of MP4 writes the two bytes of that number with the byte
/// of the largest value first. The old code always read the byte of the smallest
/// value first, thus 312 became 14337 and the source removed 0.3 seconds. The
/// MP4 file then gave 10623 samples, and the other two gave 24648.
#[test]
fn the_containers_give_the_same_audio() {
    for name in ["tone_opus.mka", "tone_opus.webm", "tone_opus.mp4"] {
        let (_, pcm) = samples_of(name);

        assert_eq!(
            pcm.len(),
            SAMPLES_OF_HALF_A_SECOND,
            "the file {} gave {} samples",
            name,
            pcm.len()
        );

        assert!(
            (peak_of(&pcm) - PEAK_OF_THE_CONTAINERS).abs() < 0.0002,
            "the file {} gave the highest value {}, and the measurement gives {}",
            name,
            peak_of(&pcm),
            PEAK_OF_THE_CONTAINERS
        );
    }
}

/// A file of two channels gives two samples for each sample of one channel.
#[test]
fn a_file_of_two_channels_gives_two_samples_for_each_sample() {
    let (_, mono) = samples_of("tone.opus");
    let (_, stereo) = samples_of("tone_stereo.opus");

    assert_eq!(mono.len(), SAMPLES_OF_HALF_A_SECOND);
    assert_eq!(stereo.len(), SAMPLES_OF_HALF_A_SECOND * 2);
}

/// The source reports 48000 samples each second, whatever the rate of the
/// recording is. Opus always decodes at that rate.
#[test]
fn the_source_reports_the_rate_of_opus() {
    let path = fixture("tone.opus");
    let file = std::fs::File::open(&path).unwrap();
    let length = file.metadata().unwrap().len();

    let source = OpusSource::open(Box::new(file), Some(length), Some("opus"), None).unwrap();

    assert_eq!(source.sample_rate().get(), 48_000);
    assert_eq!(source.channels().get(), 1);
}

/// The source moves to a position. The engine needs this for the key that moves
/// forward and for the position that the server gives.
#[test]
fn the_source_moves_to_a_position() {
    let path = fixture("tone.opus");
    let file = std::fs::File::open(&path).unwrap();
    let length = file.metadata().unwrap().len();

    let mut source = OpusSource::open(Box::new(file), Some(length), Some("opus"), None).unwrap();

    source
        .try_seek(std::time::Duration::from_millis(250))
        .expect("the source must move to a position");

    let after: Vec<f32> = source.take(4_000_000).collect();

    // The move goes to the page of the container, therefore the number of
    // samples is not exact. The source must give audio, and it must give less
    // audio than the whole file.
    assert!(
        !after.is_empty(),
        "the source gave no sample after the move"
    );
    assert!(
        after.len() < SAMPLES_OF_HALF_A_SECOND,
        "the source gave {} samples after a move to the middle",
        after.len()
    );
    assert!(peak_of(&after) > 0.05, "the audio after the move is silent");
}

/// A file that holds no Opus must give an error, and the error must not stop
/// the program.
#[test]
fn a_file_that_is_not_opus_gives_an_error() {
    let path = fixture("tone.mp3");
    let file = std::fs::File::open(&path).unwrap();
    let length = file.metadata().unwrap().len();

    let result = OpusSource::open(Box::new(file), Some(length), Some("mp3"), None);

    assert!(result.is_err(), "an MP3 file must not open as Opus");
}
