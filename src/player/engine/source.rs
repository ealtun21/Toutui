//! The byte sources of the engine.
//!
//! The engine reads a local file or a file on the server. The two sources obey
//! one trait. Therefore the engine has one decode path only, and the offline
//! mode needs no separate code.

use crate::db::crud::get_download_files;
use crate::player::engine::http_file::HttpFile;
use crate::player::engine::track::Track;
use rodio::Decoder;
use std::io::{Read, Seek};
use std::path::PathBuf;

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
pub fn select_sources(
    item_id: &str,
    username: &str,
    base_url: &str,
    tracks: &[Track],
) -> Vec<TrackSource> {
    let on_disk: Vec<(u32, String)> = get_download_files(item_id, username)
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

/// Opens a decoder for one track.
///
/// The function gives the file name extension as a hint. It sets the gapless
/// mode, so that a book with many files has no silence between the files.
pub fn open_decoder(
    source: &TrackSource,
    token: &str,
    filename: &str,
) -> Result<Decoder<Box<dyn MediaRead>>, String> {
    let data: Box<dyn MediaRead> = match source {
        TrackSource::Local(path) => {
            let file = std::fs::File::open(path)
                .map_err(|error| format!("The application cannot open the file: {}", error))?;
            Box::new(file)
        }
        TrackSource::Remote {
            base_url,
            item_id,
            ino,
        } => {
            let file = HttpFile::open(base_url, token, item_id, ino)
                .map_err(|error| format!("The server did not give the file: {}", error))?;
            Box::new(file)
        }
    };

    let mut builder = Decoder::builder().with_data(data).with_gapless(true);

    if let Some(hint) = hint_for(filename) {
        builder = builder.with_hint(&hint);
    }

    builder
        .build()
        .map_err(|error| format!("The application cannot read this audio format: {}", error))
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
