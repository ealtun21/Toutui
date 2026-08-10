//! The plan of a download, and the logic that continues an interrupted file.
//!
//! This module has no network code and no disk code, except the size of a
//! file. Therefore the tests do not need a server.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// One audio file of a book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioFilePlan {
    /// The sequence of the file in the book. The first file has the number 1.
    pub index: u32,
    /// The identity of the file on the server.
    pub ino: String,
    /// The name of the file on the server.
    pub filename: String,
    /// The number of bytes of the complete file.
    pub size: u64,
    /// The length of the file in seconds.
    pub duration: f64,
}

impl AudioFilePlan {
    /// Gives the name of the file on the disk.
    ///
    /// The number at the start keeps the sequence of the files. A file manager
    /// and the player then show the files in the correct sequence.
    pub fn disk_name(&self) -> String {
        format!("{:03} - {}", self.index, sanitise(&self.filename))
    }
}

/// All the work of one download.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadPlan {
    /// The identity of the library item. The address of each file holds this
    /// value.
    pub item_id: String,
    /// The identity of the download in the database and in the map of the
    /// progress.
    ///
    /// A book is one download, thus the key is the identity of the item. A
    /// podcast holds many episodes, and each episode is a separate download.
    /// Therefore the key of an episode is the identity of the episode.
    pub key: String,
    pub title: String,
    pub author: String,
    /// The files, in the correct sequence.
    pub files: Vec<AudioFilePlan>,
}

impl DownloadPlan {
    /// Gives the number of bytes of all the files together.
    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|file| file.size).sum()
    }

    /// Gives the length of the book in seconds.
    pub fn total_duration(&self) -> f64 {
        self.files.iter().map(|file| file.duration).sum()
    }

    /// Gives the start time of a file in the book.
    ///
    /// The server gave no value in `startOffset` in the test. Therefore this
    /// function adds the durations of the files before this file.
    pub fn start_offset(&self, index: u32) -> f64 {
        self.files
            .iter()
            .filter(|file| file.index < index)
            .map(|file| file.duration)
            .sum()
    }
}

/// Removes the characters that a file system does not accept.
fn sanitise(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            other => other,
        })
        .collect()
}

/// What the downloader must do with a file before it sends a request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resume {
    /// The file is complete. Change the name and do nothing more.
    Complete,
    /// Get the file from this byte. The value 0 means the full file.
    From(u64),
}

/// Examines the file on the disk and tells the downloader where to start.
///
/// The rules are:
///
/// - No file on the disk: start at byte 0.
/// - A part file that is shorter than the expected size: start at the end of
///   the part file.
/// - A part file that has the expected size: the file is complete.
/// - A part file that is longer than the expected size: the file on the server
///   changed. Delete the part file and start at byte 0.
pub fn resume_from(part_path: &Path, expected_size: u64) -> std::io::Result<Resume> {
    let have = match std::fs::metadata(part_path) {
        Ok(meta) => meta.len(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Resume::From(0)),
        Err(error) => return Err(error),
    };

    if have == expected_size && expected_size > 0 {
        return Ok(Resume::Complete);
    }

    if have > expected_size {
        std::fs::remove_file(part_path)?;
        return Ok(Resume::From(0));
    }

    Ok(Resume::From(have))
}

/// Makes a plan from the answer of `GET /api/items/:id`.
///
/// The function reads `media.audioFiles`. It puts the files in the sequence of
/// the field `index`.
pub fn plan_from_item(item: &serde_json::Value) -> Option<DownloadPlan> {
    let media = item.get("media")?;
    let metadata = media.get("metadata");

    let title = metadata
        .and_then(|m| m.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown title")
        .to_string();

    let author = metadata
        .and_then(|m| m.get("authorName"))
        .and_then(|a| a.as_str())
        .unwrap_or("Unknown author")
        .to_string();

    let mut files: Vec<AudioFilePlan> = media
        .get("audioFiles")?
        .as_array()?
        .iter()
        .filter_map(|file| {
            let file_metadata = file.get("metadata");

            Some(AudioFilePlan {
                index: file.get("index").and_then(|i| i.as_u64()).unwrap_or(1) as u32,
                ino: file
                    .get("ino")?
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| {
                        file.get("ino")
                            .and_then(|i| i.as_u64())
                            .map(|i| i.to_string())
                    })?,
                filename: file_metadata
                    .and_then(|m| m.get("filename"))
                    .and_then(|f| f.as_str())
                    .unwrap_or("audio")
                    .to_string(),
                size: file_metadata
                    .and_then(|m| m.get("size"))
                    .and_then(|s| s.as_u64())
                    .unwrap_or(0),
                duration: file.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0),
            })
        })
        .collect();

    if files.is_empty() {
        return None;
    }

    files.sort_by_key(|file| file.index);

    let item_id = item.get("id")?.as_str()?.to_string();

    Some(DownloadPlan {
        key: item_id.clone(),
        item_id,
        title,
        author,
        files,
    })
}

/// Makes a plan for one episode of a podcast.
///
/// The function reads `media.episodes`. An episode holds one audio file in the
/// field `audioFile`. Therefore the plan holds one file.
///
/// The title of the plan is the title of the episode, and the author is the
/// title of the podcast. The user then reads the name of the episode in the
/// bar of the progress and in the list of the downloads.
pub fn plan_from_episode(item: &serde_json::Value, episode_id: &str) -> Option<DownloadPlan> {
    let media = item.get("media")?;

    let episode = media
        .get("episodes")?
        .as_array()?
        .iter()
        .find(|episode| episode.get("id").and_then(|id| id.as_str()) == Some(episode_id))?;

    let file = episode.get("audioFile")?;

    if file.is_null() {
        return None;
    }

    let file_metadata = file.get("metadata");

    // The episode gives the length when the audio file does not give it.
    let duration = match file.get("duration").and_then(|d| d.as_f64()) {
        Some(duration) if duration > 0.0 => duration,
        _ => episode
            .get("duration")
            .and_then(|d| d.as_f64())
            .unwrap_or(0.0),
    };

    let plan_file = AudioFilePlan {
        // The episode is one file, thus the file is always the first file.
        index: 1,
        ino: file
            .get("ino")?
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| {
                file.get("ino")
                    .and_then(|i| i.as_u64())
                    .map(|i| i.to_string())
            })?,
        filename: file_metadata
            .and_then(|m| m.get("filename"))
            .and_then(|f| f.as_str())
            .unwrap_or("audio")
            .to_string(),
        size: file_metadata
            .and_then(|m| m.get("size"))
            .and_then(|s| s.as_u64())
            .unwrap_or(0),
        duration,
    };

    let title = episode
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown episode")
        .to_string();

    let author = media
        .get("metadata")
        .and_then(|m| m.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown podcast")
        .to_string();

    Some(DownloadPlan {
        item_id: item.get("id")?.as_str()?.to_string(),
        key: episode_id.to_string(),
        title,
        author,
        files: vec![plan_file],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item() -> serde_json::Value {
        // The sequence in the answer is not the sequence of the book. The
        // planner must correct it.
        json!({
            "id": "item-1",
            "media": {
                "metadata": { "title": "A Book", "authorName": "An Author" },
                "audioFiles": [
                    { "index": 2, "ino": "507", "duration": 41639.5,
                      "metadata": { "filename": "part2.m4b", "size": 673479824u64 } },
                    { "index": 1, "ino": "4434", "duration": 24017.5,
                      "metadata": { "filename": "part1.m4b", "size": 381251868u64 } }
                ]
            }
        })
    }

    #[test]
    fn the_planner_puts_the_files_in_sequence() {
        let plan = plan_from_item(&item()).unwrap();
        let order: Vec<u32> = plan.files.iter().map(|f| f.index).collect();
        assert_eq!(order, vec![1, 2]);
        assert_eq!(plan.files[0].ino, "4434");
    }

    #[test]
    fn the_planner_reads_the_title_and_the_author() {
        let plan = plan_from_item(&item()).unwrap();
        assert_eq!(plan.title, "A Book");
        assert_eq!(plan.author, "An Author");
    }

    #[test]
    fn the_planner_adds_the_sizes() {
        let plan = plan_from_item(&item()).unwrap();
        assert_eq!(plan.total_bytes(), 381251868 + 673479824);
    }

    /// The server gave no startOffset. Therefore the plan calculates it.
    #[test]
    fn the_plan_calculates_the_start_of_each_file() {
        let plan = plan_from_item(&item()).unwrap();
        assert_eq!(plan.start_offset(1), 0.0);
        assert_eq!(plan.start_offset(2), 24017.5);
    }

    #[test]
    fn a_book_without_audio_files_has_no_plan() {
        let empty = json!({ "id": "x", "media": { "audioFiles": [] } });
        assert!(plan_from_item(&empty).is_none());
    }

    /// The server sends `ino` as a string in some answers and as a number in
    /// others. The planner must accept both.
    #[test]
    fn the_planner_accepts_a_number_for_ino() {
        let value = json!({
            "id": "item-2",
            "media": { "audioFiles": [
                { "index": 1, "ino": 36495, "duration": 1.0,
                  "metadata": { "filename": "a.mp3", "size": 10u64 } }
            ]}
        });
        let plan = plan_from_item(&value).unwrap();
        assert_eq!(plan.files[0].ino, "36495");
    }

    #[test]
    fn the_disk_name_keeps_the_sequence() {
        let plan = plan_from_item(&item()).unwrap();
        assert_eq!(plan.files[0].disk_name(), "001 - part1.m4b");
        assert_eq!(plan.files[1].disk_name(), "002 - part2.m4b");
    }

    #[test]
    fn the_disk_name_removes_a_slash() {
        let file = AudioFilePlan {
            index: 1,
            ino: "1".to_string(),
            filename: "a/b:c.mp3".to_string(),
            size: 1,
            duration: 1.0,
        };
        assert_eq!(file.disk_name(), "001 - a_b_c.mp3");
    }

    #[test]
    fn a_file_that_does_not_exist_starts_at_zero() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.part");
        assert_eq!(resume_from(&path, 100).unwrap(), Resume::From(0));
    }

    #[test]
    fn a_part_file_continues_from_its_end() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.part");
        std::fs::write(&path, vec![0u8; 40]).unwrap();
        assert_eq!(resume_from(&path, 100).unwrap(), Resume::From(40));
    }

    #[test]
    fn a_part_file_with_the_full_size_is_complete() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.part");
        std::fs::write(&path, vec![0u8; 100]).unwrap();
        assert_eq!(resume_from(&path, 100).unwrap(), Resume::Complete);
    }

    /// The file on the server changed. The part file is not usable.
    // These two tests use answers from a real Audiobookshelf 2.36.0 server.
    // They fail if the server changes the shape of the answer.

    #[test]
    fn the_planner_reads_a_real_book_that_has_one_file() {
        let raw = include_str!("../../../tests/fixtures/item_single_file.json");
        let value: serde_json::Value = serde_json::from_str(raw).unwrap();
        let plan = plan_from_item(&value).unwrap();

        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.total_bytes(), 2_797_969);
        assert_eq!(plan.files[0].ino, "36495");
        assert!(plan.files[0].filename.ends_with(".mp3"));
    }

    #[test]
    fn the_planner_reads_a_real_book_that_has_many_files() {
        let raw = include_str!("../../../tests/fixtures/item_multi_file.json");
        let value: serde_json::Value = serde_json::from_str(raw).unwrap();
        let plan = plan_from_item(&value).unwrap();

        assert_eq!(plan.files.len(), 79);

        // The sequence must be 1, 2, 3 ... with no gap.
        let order: Vec<u32> = plan.files.iter().map(|f| f.index).collect();
        let expected: Vec<u32> = (1..=79).collect();
        assert_eq!(order, expected);

        assert!(plan.total_bytes() > 0);

        // The start of the second file is the length of the first file.
        assert_eq!(plan.start_offset(2), plan.files[0].duration);
    }

    /// The key of a book is the identity of the item.
    #[test]
    fn the_key_of_a_book_is_the_item() {
        let plan = plan_from_item(&item()).unwrap();
        assert_eq!(plan.key, "item-1");
        assert_eq!(plan.item_id, "item-1");
    }

    fn podcast() -> serde_json::Value {
        json!({
            "id": "pod-1",
            "media": {
                "metadata": { "title": "A Podcast" },
                "episodes": [
                    { "id": "ep1", "title": "The first letter",
                      "audioFile": { "ino": "700", "duration": 60.5,
                        "metadata": { "filename": "one.mp3", "size": 1000u64 } } },
                    { "id": "ep2", "title": "The second letter", "duration": 30.0,
                      "audioFile": { "ino": "701",
                        "metadata": { "filename": "two.mp3", "size": 2000u64 } } }
                ]
            }
        })
    }

    /// An episode is one file, and its key is the identity of the episode. The
    /// address of the file still holds the identity of the podcast.
    #[test]
    fn the_planner_reads_one_episode() {
        let plan = plan_from_episode(&podcast(), "ep1").unwrap();

        assert_eq!(plan.item_id, "pod-1");
        assert_eq!(plan.key, "ep1");
        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].index, 1);
        assert_eq!(plan.files[0].ino, "700");
        assert_eq!(plan.total_bytes(), 1000);
        assert_eq!(plan.total_duration(), 60.5);
    }

    /// The title is the title of the episode. The user reads that title in the
    /// bar of the progress.
    #[test]
    fn an_episode_gives_its_own_title() {
        let plan = plan_from_episode(&podcast(), "ep1").unwrap();
        assert_eq!(plan.title, "The first letter");
        assert_eq!(plan.author, "A Podcast");
    }

    /// Two episodes of the same podcast are two separate downloads.
    #[test]
    fn two_episodes_give_two_different_keys() {
        let first = plan_from_episode(&podcast(), "ep1").unwrap();
        let second = plan_from_episode(&podcast(), "ep2").unwrap();

        assert_ne!(first.key, second.key);
        assert_eq!(first.item_id, second.item_id);
        assert_eq!(second.files[0].disk_name(), "001 - two.mp3");
    }

    /// The audio file does not always give the length. The episode gives it.
    #[test]
    fn an_episode_takes_the_length_from_the_episode() {
        let plan = plan_from_episode(&podcast(), "ep2").unwrap();
        assert_eq!(plan.total_duration(), 30.0);
    }

    #[test]
    fn an_episode_that_is_absent_has_no_plan() {
        assert!(plan_from_episode(&podcast(), "ep9").is_none());
    }

    /// The server gives no audio file before the download of the episode.
    #[test]
    fn an_episode_without_an_audio_file_has_no_plan() {
        let value = json!({
            "id": "pod-1",
            "media": { "episodes": [ { "id": "ep1", "title": "New", "audioFile": null } ] }
        });
        assert!(plan_from_episode(&value, "ep1").is_none());
    }

    /// A book has no episode, and a podcast has no `audioFiles`.
    #[test]
    fn a_podcast_has_no_plan_for_a_book() {
        assert!(plan_from_item(&podcast()).is_none());
    }

    /// This answer comes from a real Audiobookshelf 2.36.0 server.
    #[test]
    fn the_planner_reads_a_real_episode() {
        let raw = include_str!("../../../tests/fixtures/item_podcast.json");
        let value: serde_json::Value = serde_json::from_str(raw).unwrap();

        let episode_id = value["media"]["episodes"][0]["id"].as_str().unwrap();
        let plan = plan_from_episode(&value, episode_id).unwrap();

        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].ino, "30861080");
        assert_eq!(plan.total_bytes(), 14_115_980);
        assert!(plan.total_duration() > 1700.0);
        assert_eq!(plan.files[0].disk_name(), "001 - Letter 1.mp3");
        assert_eq!(plan.title, "Letter 1");
    }

    #[test]
    fn a_part_file_that_is_too_long_is_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.part");
        std::fs::write(&path, vec![0u8; 150]).unwrap();

        assert_eq!(resume_from(&path, 100).unwrap(), Resume::From(0));
        assert!(!path.exists(), "the part file must be deleted");
    }
}
