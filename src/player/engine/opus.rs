//! The Opus source of the engine.
//!
//! Audiobookshelf accepts Opus, and `rodio::Decoder` cannot play it.
//! `rodio::Decoder` uses the fixed codec registry of symphonia, and that
//! registry has no Opus decoder. A program cannot add a codec to that registry.
//!
//! Therefore this module makes its own `rodio::Source`. Symphonia reads the
//! container and gives the packets. `opuscule` decodes each packet. Both crates
//! are pure Rust, thus the build needs no C toolchain. See T-17.
//!
//! # The measurement of 2026-08-10
//!
//! A test examined four pure Rust decoders with 47 files: the 40 files of a
//! matrix (one channel and two channels, 6 to 128 kilobits each second, and
//! frames of 2.5, 10, 20, and 60 milliseconds), the three files of
//! `tests/fixtures/audio`, and Opus in a Matroska container, in a WebM
//! container, and in an MP4 container.
//!
//! The test ran a debug build. Rust examines every arithmetic operation in such
//! a build, therefore an operation that goes past its limit stops the program.
//! A release build gives no such examination, and a first measurement in a
//! release build did not find these faults.
//!
//! | Crate | Result |
//! |---|---|
//! | `opus-decoder` 0.1.1 | stops the program on 13 of 47 files |
//! | `mousiki` 0.2.1 | stops the program on 15 of 47 files |
//! | `moosicbox_opus_native` 0.4.0 | `decode_float` holds `todo!()` |
//! | `opuscule` 0.2.0 | plays all 47 files |
//!
//! The samples of `opuscule` agree with libopus. The largest difference of one
//! sample is 0.00003 of a full scale of 1.0, and the highest value of each file
//! agrees to five places after the point.
//!
//! # The risk of the crate, and the answer to that risk
//!
//! `opuscule` 0.2.0 is young, and its own document says that an agent of
//! artificial intelligence made most of the code from the reference in C. The
//! crate rests its correctness on the test vectors of RFC 8251.
//!
//! Two properties make the risk acceptable. The crate holds `forbid(unsafe)`,
//! therefore a fault gives a wrong sample or a panic, and never damage of the
//! memory. And this module catches a panic of the decoder: the one track stops,
//! and the application continues. See `ExpectedPanic`.
//!
//! # One difference that stays
//!
//! An Opus stream holds padding at its end. The encoder makes the last frame
//! complete, and the container gives the true end. Symphonia 0.5.5 does not
//! remove that padding. Therefore this source gives up to one frame of padding
//! after the audio: 20 milliseconds for the usual file, and 60 milliseconds for
//! the largest frame. The padding is the tail of the encoder, and it is not a
//! click.

use crate::player::engine::source::MediaRead;
use crate::utils::exit_app::ExpectedPanic;
use log::{info, warn};
use opuscule::{Channels as OpusChannels, Decoder as OpusDecoder, SampleRate as OpusRate};
use rodio::source::SeekError;
use rodio::{ChannelCount, Source};
use std::io::{Read, Seek, SeekFrom};
use std::num::NonZero;
use std::time::Duration;
use symphonia::core::codecs::CODEC_TYPE_OPUS;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Opus always gives 48000 samples each second, whatever the rate of the
/// recording is.
const OPUS_RATE: u32 = 48_000;

/// The largest number of samples of one channel in one frame.
///
/// A frame of Opus holds 120 milliseconds at the most, and 48000 samples each
/// second give 5760 samples.
const MAX_FRAME: usize = 5760;

/// The number of packets that may fail before the source stops.
///
/// One packet that fails is a fault of the data, and the decoder continues. A
/// stream where every packet fails is not Opus, and the source must stop.
const MAX_ERRORS: usize = 16;

/// Gives symphonia a source of bytes.
///
/// The engine reads a local file or a file on the server. Both obey
/// `MediaRead`. Symphonia needs `MediaSource`, and that trait also gives the
/// length and tells if the source can move.
struct ByteSource {
    inner: Box<dyn MediaRead>,
    length: Option<u64>,
}

impl Read for ByteSource {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buffer)
    }
}

impl Seek for ByteSource {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(position)
    }
}

impl MediaSource for ByteSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.length
    }
}

/// Reads the number of samples that a stream skips at its start.
///
/// Every Opus stream holds that number in its head. The head has two forms, and
/// symphonia gives both forms with the eight characters `OpusHead` first:
///
/// | Container | Version | The two bytes of the number |
/// |---|---|---|
/// | OGG, Matroska, WebM | 1 | the byte of the smallest value first |
/// | MP4, in the box `dOps` | 0 | the byte of the largest value first |
///
/// The version byte comes after the eight characters. The number of channels
/// comes after the version, and the number of samples comes after that.
///
/// A measurement on 2026-08-10 found this difference. The old code always read
/// the two bytes with the byte of the smallest value first. The value 312 of an
/// MP4 file then became 14337, and the source removed 0.3 seconds of the audio.
///
/// Gives `None` if the data is not such a head.
fn pre_skip_of(extra_data: Option<&[u8]>) -> Option<usize> {
    let data = extra_data?;

    if data.len() < 12 || &data[..8] != b"OpusHead" {
        return None;
    }

    let bytes = [data[10], data[11]];

    // The version byte tells which form the head has.
    let value = match data[8] {
        0 => u16::from_be_bytes(bytes),
        _ => u16::from_le_bytes(bytes),
    };

    Some(value as usize)
}

/// A `rodio::Source` that plays Opus.
pub struct OpusSource {
    reader: Box<dyn FormatReader>,
    decoder: OpusDecoder,
    /// The track of the container that holds the Opus audio.
    track_id: u32,
    channels: ChannelCount,
    /// The samples of the packet that the source gives now, interleaved.
    frame: Vec<f32>,
    /// The next sample of `frame` that the source gives.
    next: usize,
    /// The scratch buffer of the decoder.
    scratch: Vec<f32>,
    /// The number of samples that the source must not give, interleaved. The
    /// `OpusHead` header of the stream gives this number as `pre-skip`.
    skip: usize,
    /// The value of `skip` for a new position. `try_seek` uses it.
    pre_skip: usize,
    total: Option<Duration>,
    errors: usize,
    /// The source gives no more audio.
    complete: bool,
}

impl OpusSource {
    /// Opens an Opus stream.
    ///
    /// The function gives an error if the container holds no Opus track. The
    /// caller then uses `rodio::Decoder`.
    ///
    /// `hint` is the file name extension, and `mime_type` is the type of the
    /// content. Both make the examination of the container quicker.
    pub fn open(
        data: Box<dyn MediaRead>,
        length: Option<u64>,
        hint: Option<&str>,
        mime_type: Option<&str>,
    ) -> Result<OpusSource, String> {
        let stream = MediaSourceStream::new(
            Box::new(ByteSource {
                inner: data,
                length,
            }),
            Default::default(),
        );

        let mut probe_hint = Hint::new();

        if let Some(hint) = hint {
            probe_hint.with_extension(hint);
        }

        if let Some(mime_type) = mime_type {
            if !mime_type.is_empty() {
                probe_hint.mime_type(mime_type);
            }
        }

        let probed = symphonia::default::get_probe()
            .format(
                &probe_hint,
                stream,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|error| format!("symphonia did not read the container: {}", error))?;

        let reader = probed.format;

        let track = reader
            .tracks()
            .iter()
            .find(|track| track.codec_params.codec == CODEC_TYPE_OPUS)
            .ok_or_else(|| "the container holds no Opus track".to_string())?;

        let track_id = track.id;
        let count = track.codec_params.channels.map(|value| value.count());

        let count = match count {
            Some(count @ 1..=2) => count,
            Some(count) => {
                return Err(format!(
                    "the file holds {} channels of Opus. The application plays \
                     one channel or two channels.",
                    count
                ))
            }
            None => 1,
        };

        let channels =
            NonZero::new(count as u16).ok_or_else(|| "the file holds no channel".to_string())?;

        // The head of the stream gives the number of samples that the stream
        // skips at its start. That head is the only value that agrees with
        // libopus.
        //
        // `codec_params.delay` is not that number. A measurement on 2026-08-10
        // shows that symphonia 0.5.5 gives `delay` 648 for a file whose head
        // says 312, and 648 is the padding at the end of that file. The old code
        // used `delay`, and the audio then began 336 samples too late. A
        // Matroska container, a WebM container, and an MP4 container give no
        // `delay` at all.
        //
        // Therefore the head has the highest importance, and `delay` is the
        // answer only for a stream that gives no head.
        let delay = pre_skip_of(track.codec_params.extra_data.as_deref())
            .or(track.codec_params.delay.map(|value| value as usize))
            .unwrap_or(0);

        let pre_skip = delay * count;

        // The length comes from the number of samples of the track. The time
        // base of the track changes that number to a time. A Matroska container
        // counts milliseconds, and an OGG container counts samples of 48000.
        let total = match (track.codec_params.n_frames, track.codec_params.time_base) {
            (Some(frames), Some(time_base)) => {
                let time = time_base.calc_time(frames);
                Some(Duration::from_secs_f64(time.seconds as f64 + time.frac))
            }
            (Some(frames), None) => Some(Duration::from_secs_f64(frames as f64 / OPUS_RATE as f64)),
            (None, _) => None,
        };

        let decoder = OpusDecoder::new(
            OpusRate::Hz48000,
            if count == 2 {
                OpusChannels::Stereo
            } else {
                OpusChannels::Mono
            },
        );

        info!(
            "[opus] the stream holds {} channel(s), and it skips {} sample(s)",
            count, pre_skip
        );

        Ok(OpusSource {
            reader,
            decoder,
            track_id,
            channels,
            frame: Vec::new(),
            next: 0,
            scratch: vec![0.0; MAX_FRAME * count],
            skip: pre_skip,
            pre_skip,
            total,
            errors: 0,
            complete: false,
        })
    }

    /// Reads the next packet, and puts its samples in `frame`.
    ///
    /// Gives `false` when the stream comes to its end.
    fn read_packet(&mut self) -> bool {
        let count = self.channels.get() as usize;

        loop {
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(error) => {
                    info!("[opus] the stream comes to its end: {}", error);
                    self.complete = true;
                    return false;
                }
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            // The decoder is a young crate, and it can stop with an
            // arithmetic fault on data that it does not expect. Therefore the
            // source catches such a panic. The guard tells the hook of the
            // panic that the caller expects this panic, thus the hook keeps the
            // terminal and the screen of the application. See T-17.
            let decoded = {
                let _guard = ExpectedPanic::new();
                let decoder = &mut self.decoder;
                let scratch = &mut self.scratch;
                let data = &packet.data;

                std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                    decoder.decode(Some(data), scratch, false)
                }))
            };

            let samples = match decoded {
                Ok(Ok(per_channel)) => per_channel * count,
                Ok(Err(error)) => {
                    self.errors += 1;
                    warn!(
                        "[opus] a packet did not decode ({} of {}): {:?}",
                        self.errors, MAX_ERRORS, error
                    );

                    if self.errors >= MAX_ERRORS {
                        self.complete = true;
                        return false;
                    }

                    continue;
                }
                // The decoder stopped. Its state is not known now, therefore
                // the source must not use it again.
                Err(_) => {
                    warn!(
                        "[opus] the decoder stopped on a packet. The track \
                         stops, and the application continues."
                    );
                    self.complete = true;
                    return false;
                }
            };

            // The number of samples of the frame is the truth. The length that
            // the container gives is in the time base of the track, and that
            // time base is 1/48000 for an OGG container only. A Matroska
            // container counts milliseconds, therefore a calculation with that
            // value gave 7 samples for a file of one second. A measurement on
            // 2026-08-10 found that fault.
            let keep = samples;

            // Remove the samples of `pre-skip`. Those samples are the warm-up
            // of the decoder, and they are not audio.
            let start = self.skip.min(keep);
            self.skip -= start;

            if start == keep {
                continue;
            }

            self.frame.clear();
            self.frame.extend_from_slice(&self.scratch[start..keep]);
            self.next = 0;

            return true;
        }
    }
}

impl Iterator for OpusSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.next >= self.frame.len() && (self.complete || !self.read_packet()) {
            return None;
        }

        let sample = self.frame.get(self.next).copied();
        self.next += 1;

        sample
    }
}

impl Source for OpusSource {
    /// The stream never changes its number of channels and never changes its
    /// rate. Therefore the source has one span, and that span ends with the
    /// audio.
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        NonZero::new(OPUS_RATE).expect("48000 is not zero")
    }

    fn total_duration(&self) -> Option<Duration> {
        self.total
    }

    /// Moves to a position in the stream.
    ///
    /// The function moves the container and then makes the decoder new. The
    /// decoder holds the state of the frames before this position, and that
    /// state does not agree with the new position.
    fn try_seek(&mut self, position: Duration) -> Result<(), SeekError> {
        let target = SeekTo::Time {
            time: position.into(),
            track_id: Some(self.track_id),
        };

        self.reader
            .seek(SeekMode::Coarse, target)
            .map_err(|error| {
                SeekError::Other(std::sync::Arc::new(std::io::Error::other(
                    error.to_string(),
                )))
            })?;

        // `opuscule` gives no function that makes the state new. A new decoder
        // gives that state, and it allocates nothing on the path of the decode.
        self.decoder = OpusDecoder::new(
            OpusRate::Hz48000,
            if self.channels.get() == 2 {
                OpusChannels::Stereo
            } else {
                OpusChannels::Mono
            },
        );

        self.frame.clear();
        self.next = 0;
        self.errors = 0;
        self.complete = false;

        // The decoder has no state of the frames before this position.
        // Therefore the first samples are a warm-up, as at the start of the
        // stream.
        self.skip = self.pre_skip;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rate of Opus is always 48000, and `NonZero` accepts that value.
    #[test]
    fn the_rate_is_48000() {
        assert_eq!(NonZero::new(OPUS_RATE).unwrap().get(), 48_000);
    }

    /// A header of one channel with the value 312.
    fn header() -> Vec<u8> {
        let mut data = b"OpusHead".to_vec();
        data.push(1); // the version
        data.push(1); // the number of channels
        data.extend_from_slice(&312u16.to_le_bytes());
        data.extend_from_slice(&48_000u32.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.push(0); // the mapping of the channels
        data
    }

    /// The same head in the form of the box `dOps` of MP4: the version is 0,
    /// and the byte of the largest value comes first.
    fn header_of_mp4() -> Vec<u8> {
        let mut data = b"OpusHead".to_vec();
        data.push(0); // the version of the box `dOps`
        data.push(1); // the number of channels
        data.extend_from_slice(&312u16.to_be_bytes());
        data.extend_from_slice(&48_000u32.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.push(0); // the mapping of the channels
        data
    }

    #[test]
    fn the_head_of_ogg_gives_the_number_of_samples_to_skip() {
        assert_eq!(pre_skip_of(Some(&header())), Some(312));
    }

    /// The old code read the two bytes of the MP4 head in the wrong sequence.
    /// The value 312 became 14337, and the source then removed 0.3 seconds of
    /// the audio. A measurement on 2026-08-10 found that fault.
    #[test]
    fn the_head_of_mp4_gives_the_number_of_samples_to_skip() {
        assert_eq!(pre_skip_of(Some(&header_of_mp4())), Some(312));
        assert_ne!(pre_skip_of(Some(&header_of_mp4())), Some(14337));
    }

    /// A Matroska container gives no header of Opus in some conditions. The
    /// function must give no value, and it must not stop the program.
    #[test]
    fn data_that_is_not_the_header_gives_no_value() {
        assert_eq!(pre_skip_of(None), None);
        assert_eq!(pre_skip_of(Some(&[])), None);
        assert_eq!(pre_skip_of(Some(b"OpusHead")), None);
        assert_eq!(pre_skip_of(Some(b"something else at all")), None);
    }
}
