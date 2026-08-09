//! Tests of the audio formats that the application plays.
//!
//! Each test file in `tests/fixtures/audio` holds a tone of 0.25 seconds. The
//! files are small, thus the repository stays small.
//!
//! These tests protect the user against a change of a dependency. If a new
//! version of `rodio` or of `symphonia` stops a format, one of these tests
//! fails. A measurement alone does not give this protection, because a
//! measurement is true only on the day of the measurement.

use std::path::{Path, PathBuf};
use toutui::player::engine::source::{open_decoder, TrackSource, SUPPORTED_FORMATS};
use toutui::player::engine::track::Track;

/// Gives the directory of the test files.
fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/audio")
}

/// Makes a track for one test file.
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
        duration: 0.25,
        start_offset: 0.0,
    }
}

/// Decodes one test file and gives the number of samples.
///
/// The function counts the samples. A format that gives no sample is a
/// failure, and not a success. A measurement on 2026-08-10 found this
/// condition with an MKV file: the decoder gave no error, and it gave no
/// audio.
fn decode(name: &str) -> Result<usize, String> {
    let path = fixture_dir().join(name);
    assert!(path.exists(), "the test file {} is absent", name);

    let track = track_for(&path);
    let source = TrackSource::Local(path);
    let decoder = open_decoder(&source, "no-token", &track)?;

    Ok(decoder.take(1_000_000).count())
}

/// Examines one format. The file must decode, and it must give audio.
fn assert_plays(name: &str) {
    match decode(name) {
        Ok(0) => panic!(
            "the file {} decoded, but it gave no audio sample. The container \
             is correct and the codec is absent.",
            name
        ),
        Ok(samples) => {
            assert!(
                samples > 500,
                "the file {} gave only {} samples. A tone of 0.25 seconds at \
                 8000 samples each second gives approximately 2000 samples.",
                name,
                samples
            );
        }
        Err(error) => panic!("the file {} did not decode: {}", name, error),
    }
}

#[test]
fn mp3_plays() {
    assert_plays("tone.mp3");
}

/// An M4B file is the usual format of an audiobook. It holds AAC audio in an
/// MP4 container.
#[test]
fn m4b_plays() {
    assert_plays("tone.m4b");
}

#[test]
fn m4a_plays() {
    assert_plays("tone.m4a");
}

/// A raw AAC file has the ADTS container.
#[test]
fn aac_plays() {
    assert_plays("tone.aac");
}

#[test]
fn flac_plays() {
    assert_plays("tone.flac");
}

#[test]
fn wav_plays() {
    assert_plays("tone.wav");
}

#[test]
fn ogg_vorbis_plays() {
    assert_plays("tone.ogg");
}

/// ALAC is the lossless codec of Apple. The feature `symphonia-all` gives it.
#[test]
fn alac_in_mp4_plays() {
    assert_plays("tone_alac.m4a");
}

/// AIFF is the audio format of Apple. The feature `symphonia-all` gives it.
#[test]
fn aiff_plays() {
    assert_plays("tone.aiff");
}

/// CAF is a container of Apple. The feature `symphonia-all` gives it.
#[test]
fn caf_plays() {
    assert_plays("tone.caf");
}

/// The Matroska container. Audiobookshelf accepts a file with the extension
/// `mka`.
#[test]
fn matroska_plays() {
    assert_plays("tone.mka");
}

/// WebM is a Matroska container with a different name.
#[test]
fn webm_plays() {
    assert_plays("tone.webm");
}

/// ADPCM is an old codec of the WAV container.
#[test]
fn adpcm_plays() {
    assert_plays("tone_adpcm.wav");
}

/// Tells if a test file holds Opus audio.
///
/// The application cannot play Opus today. See T-17 and the test below.
fn is_opus(name: &str) -> bool {
    name.contains("opus")
}

/// Every file in the directory must have a test. This test finds a file that
/// a person adds and forgets.
#[test]
fn every_test_file_decodes() {
    let mut count = 0;

    for entry in std::fs::read_dir(fixture_dir()).unwrap() {
        let path = entry.unwrap().path();

        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy().to_string();

            if is_opus(&name) {
                continue;
            }

            assert_plays(&name);
            count += 1;
        }
    }

    assert!(count >= 13, "the directory must hold the test files");
}

/// Audiobookshelf accepts Opus, and the application cannot play it. The
/// application must give a clear message, and it must not stop.
///
/// A measurement on 2026-08-10 examined two pure Rust decoders. The crate
/// `opus-decoder` 0.1.1 stops the program with an arithmetic fault on a file
/// of 24 kilobits each second. The crate `moosicbox_opus_native` 0.4.0 gives
/// no sample for a file of one channel. Therefore no decoder is ready.
///
/// This test becomes a test of a working format when a decoder is ready. See
/// T-17.
#[test]
fn an_opus_file_gives_a_clear_message() {
    for name in ["tone.opus", "tone_stereo.opus", "tone_opus.ogg"] {
        let path = fixture_dir().join(name);
        assert!(path.exists(), "the test file {} is absent", name);

        match decode(name) {
            Ok(samples) => panic!(
                "the application played the Opus file {} and gave {} samples.                  If a decoder is ready now, move this file to the tests of                  the formats that play.",
                name, samples
            ),
            Err(error) => {
                assert!(
                    error.contains(name),
                    "the message must name the file, but it is: {}",
                    error
                );
                assert!(
                    error.contains(SUPPORTED_FORMATS),
                    "the message must show the formats, but it is: {}",
                    error
                );
            }
        }
    }
}

/// A file that is not audio must give a clear message. The message must name
/// the file, and it must show the formats that the application plays.
#[test]
fn a_file_that_is_not_audio_gives_a_clear_message() {
    let path = std::env::temp_dir().join("toutui-not-audio.mp3");
    std::fs::write(&path, b"this is not audio").unwrap();

    let track = track_for(&path);
    let source = TrackSource::Local(path);

    // `Decoder` does not obey `Debug`. Therefore the test cannot use
    // `unwrap_err`.
    let error = match open_decoder(&source, "no-token", &track) {
        Ok(_) => panic!("a file that is not audio must give an error"),
        Err(error) => error,
    };

    assert!(
        error.contains("toutui-not-audio.mp3"),
        "the message must name the file, but it is: {}",
        error
    );
    assert!(
        error.contains(SUPPORTED_FORMATS),
        "the message must show the formats, but it is: {}",
        error
    );
}

/// A file that is absent must give a message, and it must not stop the
/// application.
#[test]
fn a_file_that_is_absent_gives_a_message() {
    let track = track_for(Path::new("absent.mp3"));
    let source = TrackSource::Local(PathBuf::from("/does/not/exist/absent.mp3"));

    let error = match open_decoder(&source, "no-token", &track) {
        Ok(_) => panic!("a file that is absent must give an error"),
        Err(error) => error,
    };

    assert!(
        error.contains("cannot open the file"),
        "the message is: {}",
        error
    );
}
