//! The byte sources of the engine.
//!
//! The engine reads a local file or a file on the server. The two sources obey
//! one trait. Therefore the engine has one decode path only, and the offline
//! mode needs no separate code.

use crate::db::crud::get_download_files;
use crate::player::engine::http_file::HttpFile;
use crate::player::engine::opus::OpusSource;
use crate::player::engine::track::Track;
use log::{info, warn};
use rodio::source::SeekError;
use rodio::{ChannelCount, Decoder, SampleRate, Source};
use std::io::{Read, Seek};
use std::path::PathBuf;
use std::time::Duration;

/// A source of bytes that `rodio::Decoder` accepts.
///
/// `rodio::Decoder<R>` needs all of these bounds.
pub trait MediaRead: Read + Seek + Send + Sync {}

impl<T: Read + Seek + Send + Sync> MediaRead for T {}

/// Where the engine reads one track.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackSource {
    /// A file on the disk. The user downloaded the book.
    Local(PathBuf),
    /// A file on the server.
    Remote {
        /// The base address of the server.
        base_url: String,
        /// The identity of the book.
        item_id: String,
        /// The identity of the file.
        ino: String,
    },
}

/// Gives the file name extension in lower case.
///
/// The decoder uses this value as a hint. The hint stops the examination of
/// the format. That examination costs range requests on a file that comes from
/// the server.
pub fn hint_for(filename: &str) -> Option<String> {
    let extension = std::path::Path::new(filename).extension()?;
    Some(extension.to_string_lossy().to_lowercase())
}

/// Selects the source of each track.
///
/// A copy on the disk always has more importance than the server. The engine
/// uses the copy on the disk only if the disk has every file of the book. The
/// engine does not mix the two sources in one book.
///
/// The parameter `download_key` is the identity of the download. It is the
/// identity of the item for a book, and the identity of the episode for one
/// episode of a podcast. The parameter `item_id` stays the identity of the
/// item, because the address of a file on the server holds that value.
pub fn select_sources(
    download_key: &str,
    item_id: &str,
    username: &str,
    base_url: &str,
    tracks: &[Track],
) -> Vec<TrackSource> {
    let on_disk: Vec<(u32, String)> = get_download_files(download_key, username)
        .into_iter()
        .map(|(index, path, _duration)| (index, path))
        .collect();

    sources_from(&on_disk, base_url, item_id, tracks)
}

/// The pure part of `select_sources`. The tests use this function, because it
/// needs no database.
fn sources_from(
    on_disk: &[(u32, String)],
    base_url: &str,
    item_id: &str,
    tracks: &[Track],
) -> Vec<TrackSource> {
    let complete = !tracks.is_empty()
        && tracks.iter().all(|track| {
            on_disk
                .iter()
                .any(|(index, path)| *index == track.index && !path.is_empty())
        });

    tracks
        .iter()
        .map(|track| {
            if complete {
                let path = on_disk
                    .iter()
                    .find(|(index, _)| *index == track.index)
                    .map(|(_, path)| PathBuf::from(path))
                    .unwrap_or_default();

                TrackSource::Local(path)
            } else {
                TrackSource::Remote {
                    base_url: base_url.trim_end_matches('/').to_string(),
                    item_id: item_id.to_string(),
                    ino: track.ino.clone(),
                }
            }
        })
        .collect()
}

/// The formats that the application plays.
///
/// A measurement on 2026-08-10 confirms each one of these formats with a real
/// file. The message of a failure shows this list to the user.
pub const SUPPORTED_FORMATS: &str =
    "mp3, m4b, m4a, mp4, aac, flac, wav, aiff, ogg, oga, opus, mka, webm, and caf";

/// The formats that the application does not play.
///
/// Audiobookshelf accepts 19 audio formats. The engine plays 17 of them.
/// `symphonia` has no decoder for AMR-WB, and it has no reader for the ASF
/// container of WMA. No pure Rust crate gives either one, and a decoder that
/// needs a C library is not permitted. See T-18.
///
/// Opus left this list on 2026-08-10. The engine reads the packets with
/// symphonia, and `opus-decoder` decodes them. See T-17 and
/// `crate::player::engine::opus`.
pub const UNSUPPORTED_FORMATS: &str = "wma and awb";

/// What the engine reads to make the audio.
///
/// `rodio::Decoder` plays 16 formats. It cannot play Opus, because it uses the
/// fixed codec registry of symphonia. Therefore Opus has its own source. See
/// T-17 and `crate::player::engine::opus`.
pub enum EngineSource {
    /// The decoder of rodio.
    ///
    /// The value is in a box, because the two decoders have a large difference
    /// of size. Every value of the enumeration would take the size of the
    /// largest one without the box.
    Rodio(Box<Decoder<Box<dyn MediaRead>>>),
    /// The Opus source of this project.
    Opus(Box<OpusSource>),
}

impl Iterator for EngineSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        match self {
            EngineSource::Rodio(inner) => inner.next(),
            EngineSource::Opus(inner) => inner.next(),
        }
    }
}

impl Source for EngineSource {
    fn current_span_len(&self) -> Option<usize> {
        match self {
            EngineSource::Rodio(inner) => inner.current_span_len(),
            EngineSource::Opus(inner) => inner.current_span_len(),
        }
    }

    fn channels(&self) -> ChannelCount {
        match self {
            EngineSource::Rodio(inner) => inner.channels(),
            EngineSource::Opus(inner) => inner.channels(),
        }
    }

    fn sample_rate(&self) -> SampleRate {
        match self {
            EngineSource::Rodio(inner) => inner.sample_rate(),
            EngineSource::Opus(inner) => inner.sample_rate(),
        }
    }

    fn total_duration(&self) -> Option<Duration> {
        match self {
            EngineSource::Rodio(inner) => inner.total_duration(),
            EngineSource::Opus(inner) => inner.total_duration(),
        }
    }

    fn try_seek(&mut self, position: Duration) -> Result<(), SeekError> {
        match self {
            EngineSource::Rodio(inner) => inner.try_seek(position),
            EngineSource::Opus(inner) => inner.try_seek(position),
        }
    }
}

/// The file name extensions that hold Opus in an OGG container.
///
/// A file that has one of these extensions gets the Opus source first. The
/// other containers get `rodio::Decoder` first, and the Opus source only when
/// that decoder gives an error. Therefore a format that operates now cannot
/// change. See T-17.
const OGG_EXTENSIONS: [&str; 3] = ["opus", "oga", "ogg"];

/// Opens a decoder for one track.
///
/// The function gives symphonia three values that make the decoder more
/// robust:
///
/// - The file name extension, as a hint.
/// - The type of the content, as a second hint. A file with no extension then
///   still gets a hint.
/// - The number of bytes. Symphonia then knows the length of the stream.
///
/// The function also sets the gapless mode, so that a book with many files has
/// no silence between the files.
pub fn open_decoder(
    source: &TrackSource,
    token: &str,
    track: &Track,
) -> Result<EngineSource, String> {
    let hint = hint_for(&track.filename);
    let opus_first = hint
        .as_deref()
        .map(|value| OGG_EXTENSIONS.contains(&value))
        .unwrap_or(false)
        || track
            .mime_type
            .as_deref()
            .map(|value| value.contains("opus"))
            .unwrap_or(false);

    if opus_first {
        match open_opus(source, token, track) {
            Ok(opus) => {
                info!("[open_decoder] the file {} plays as Opus", track.filename);
                return Ok(EngineSource::Opus(Box::new(opus)));
            }
            Err(error) => {
                // An OGG file also holds Vorbis or FLAC. The decoder of rodio
                // plays those.
                info!(
                    "[open_decoder] the file {} is not Opus: {}",
                    track.filename, error
                );
            }
        }
    }

    match open_rodio(source, token, track) {
        Ok(decoder) => Ok(EngineSource::Rodio(Box::new(decoder))),
        Err(error) if opus_first => Err(error),
        Err(error) => {
            // The container is not one of the OGG containers, and the decoder of
            // rodio gave an error. Opus in a Matroska container, in a WebM
            // container, or in an MP4 container comes here.
            match open_opus(source, token, track) {
                Ok(opus) => {
                    info!("[open_decoder] the file {} plays as Opus", track.filename);
                    Ok(EngineSource::Opus(Box::new(opus)))
                }
                Err(opus_error) => {
                    warn!(
                        "[open_decoder] the file {} is not Opus either: {}",
                        track.filename, opus_error
                    );

                    // The message of rodio names the formats. Therefore the
                    // user reads that message, and not the message of Opus.
                    Err(error)
                }
            }
        }
    }
}

/// Opens the bytes of a track.
///
/// The function gives the length of the stream, if it knows the length.
/// Symphonia then knows where the stream ends.
fn open_bytes(
    source: &TrackSource,
    token: &str,
) -> Result<(Box<dyn MediaRead>, Option<u64>), String> {
    match source {
        TrackSource::Local(path) => {
            let file = std::fs::File::open(path)
                .map_err(|error| format!("The application cannot open the file: {}", error))?;

            let size = file.metadata().ok().map(|meta| meta.len());
            Ok((Box::new(file), size))
        }
        TrackSource::Remote {
            base_url,
            item_id,
            ino,
        } => {
            let file = HttpFile::open(base_url, token, item_id, ino)
                .map_err(|error| format!("The server did not give the file: {}", error))?;

            let size = Some(file.len());
            Ok((Box::new(file), size))
        }
    }
}

/// Opens the Opus source of this project.
fn open_opus(source: &TrackSource, token: &str, track: &Track) -> Result<OpusSource, String> {
    let (data, size) = open_bytes(source, token)?;

    OpusSource::open(
        data,
        size.or(track.size),
        hint_for(&track.filename).as_deref(),
        track.mime_type.as_deref(),
    )
}

/// Opens the decoder of rodio. That decoder plays 16 formats.
fn open_rodio(
    source: &TrackSource,
    token: &str,
    track: &Track,
) -> Result<Decoder<Box<dyn MediaRead>>, String> {
    let (data, size) = open_bytes(source, token)?;

    // The two sources obey `Seek`. Therefore symphonia can move in the file.
    // An M4B file needs this, because the decoder reads the `moov` atom
    // before it reads the audio.
    let mut builder = Decoder::builder()
        .with_data(data)
        .with_gapless(true)
        .with_seekable(true);

    if let Some(size) = size.or(track.size) {
        builder = builder.with_byte_len(size);
    }

    if let Some(hint) = hint_for(&track.filename) {
        builder = builder.with_hint(&hint);
    }

    if let Some(mime_type) = track.mime_type.as_deref() {
        if !mime_type.is_empty() {
            builder = builder.with_mime_type(mime_type);
        }
    }

    builder.build().map_err(|error| {
        format!(
            "The application cannot read the file {}: {}. \
             The application plays {}. It does not play {}.",
            track.filename, error, SUPPORTED_FORMATS, UNSUPPORTED_FORMATS
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::engine::track::TrackList;

    fn tracks() -> Vec<Track> {
        TrackList::from_durations(&[10.0, 20.0])
    }

    /// The disk has every file. The engine must use the copy on the disk.
    #[test]
    fn a_complete_download_gives_local_sources() {
        let on_disk = vec![
            (1u32, "/tmp/a/001.mp3".to_string()),
            (2u32, "/tmp/a/002.mp3".to_string()),
        ];

        let sources = sources_from(&on_disk, "http://server", "item1", &tracks());

        assert_eq!(sources.len(), 2);
        assert!(matches!(sources[0], TrackSource::Local(_)));
        assert!(matches!(sources[1], TrackSource::Local(_)));
    }

    /// One file is absent. The engine must use the server for all tracks. It
    /// must not mix the two sources in one book.
    #[test]
    fn an_incomplete_download_gives_remote_sources() {
        let on_disk = vec![(1u32, "/tmp/a/001.mp3".to_string())];

        let sources = sources_from(&on_disk, "http://server", "item1", &tracks());

        assert_eq!(sources.len(), 2);
        assert!(matches!(sources[0], TrackSource::Remote { .. }));
        assert!(matches!(sources[1], TrackSource::Remote { .. }));
    }

    #[test]
    fn no_download_gives_remote_sources() {
        let sources = sources_from(&[], "http://server", "item1", &tracks());

        assert_eq!(sources.len(), 2);
        assert!(matches!(sources[0], TrackSource::Remote { .. }));
    }

    #[test]
    fn a_remote_source_holds_the_identity_of_the_file() {
        let sources = sources_from(&[], "http://server/", "item1", &tracks());

        match &sources[0] {
            TrackSource::Remote {
                base_url,
                item_id,
                ino,
            } => {
                assert_eq!(base_url, "http://server");
                assert_eq!(item_id, "item1");
                assert_eq!(ino, "ino-0");
            }
            other => panic!("the source must be remote, but it is {:?}", other),
        }
    }

    /// A row with an empty path is not a complete file.
    #[test]
    fn a_row_with_an_empty_path_gives_remote_sources() {
        let on_disk = vec![(1u32, "/tmp/a/001.mp3".to_string()), (2u32, String::new())];

        let sources = sources_from(&on_disk, "http://server", "item1", &tracks());

        assert!(matches!(sources[0], TrackSource::Remote { .. }));
    }

    #[test]
    fn the_hint_is_the_extension_of_the_file() {
        assert_eq!(hint_for("001 - part.m4b"), Some("m4b".to_string()));
        assert_eq!(hint_for("001 - part.MP3"), Some("mp3".to_string()));
        assert_eq!(hint_for("no-extension"), None);
    }
}
