pub mod fetch;
pub mod lock;
pub mod plan;
pub mod progress;

use crate::db::crud::{
    delete_download, get_download, get_download_files, insert_download, insert_download_file,
};
use fetch::{fetch_item, TheFaultOfTheDownload};
use log::{error, info};
use plan::{plan_from_episode, plan_from_item};
use progress::ProgressMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

/// Gives the directory that holds the downloads of one user, for offline
/// listening.
///
/// The path comes from `paths::data_dir`, so that this rule and the rule of
/// `--uninstall` agree. See T-21.
pub fn downloads_base_dir(username: &str) -> PathBuf {
    crate::paths::data_dir().join("downloads").join(username)
}

/// Gives the address of a download.
///
/// **The pool decides**, as it decides for every other request of the program
/// (T-105 and T-128). The address of the login is the answer when the pool holds
/// no address at all, and the request then says what that address says.
///
/// The function is pure, therefore a test needs no server. See T-149.
pub fn the_address_of_the_download(of_the_pool: Option<String>, of_the_login: &str) -> String {
    of_the_pool.unwrap_or_else(|| of_the_login.to_string())
}

/// Makes the client of a download.
///
/// **A download holds no limit of its whole time.** The send of a book of 479
/// megabytes took 36 seconds in the measurement of T-119, and a book of some
/// gigabytes takes much more. The two limits are therefore a limit of the
/// connection and a limit of a wait with no byte at all: a user who presses the
/// key `D` toward an address that no machine answers reads a sentence, and they
/// do not wait for ever. See T-149.
fn the_client_of_a_download() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(crate::api::client::CONNECT_TIMEOUT)
        .read_timeout(THE_TIME_WITH_NO_BYTE)
        .build()
        .unwrap_or_default()
}

/// The longest wait for a block of the answer of a download.
const THE_TIME_WITH_NO_BYTE: std::time::Duration = std::time::Duration::from_secs(30);

/// Makes an empty map of the progress of the downloads.
pub fn new_progress_map() -> ProgressMap {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Gives the map of the progress of all the downloads.
///
/// The map is global. The program makes a new `App` when the user refreshes
/// with the key `R`. A map in `App` loses the progress of a download that
/// runs. This map does not.
pub fn downloads() -> ProgressMap {
    static DOWNLOADS: OnceLock<ProgressMap> = OnceLock::new();
    Arc::clone(DOWNLOADS.get_or_init(new_progress_map))
}

/// What the user asked the application to download.
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadTarget {
    /// A book of the library. The application gets every audio file.
    Book { item_id: String },
    /// One episode of a podcast. The application gets one audio file.
    Episode {
        /// The identity of the podcast.
        item_id: String,
        /// The identity of the episode.
        episode_id: String,
    },
}

impl DownloadTarget {
    /// Gives the identity of the library item.
    pub fn item_id(&self) -> &str {
        match self {
            DownloadTarget::Book { item_id } => item_id,
            DownloadTarget::Episode { item_id, .. } => item_id,
        }
    }

    /// Gives the identity of the download.
    ///
    /// A book is one download. A podcast holds many episodes, and each episode
    /// is a separate download. See [`plan::DownloadPlan::key`].
    pub fn key(&self) -> &str {
        match self {
            DownloadTarget::Book { item_id } => item_id,
            DownloadTarget::Episode { episode_id, .. } => episode_id,
        }
    }
}

/// Downloads a book or one episode of a podcast, and writes its progress to
/// the given map.
///
/// The function gets the audio files one at a time. A file that is not
/// complete stays on the disk with the name `.part`. The next call continues
/// that file.
#[allow(clippy::too_many_arguments)]
pub async fn download_with_progress(
    token: Option<String>,
    target: DownloadTarget,
    server_address: String,
    username: String,
    title: String,
    author: String,
    server_key: String,
    progress: ProgressMap,
) {
    crate::logic::message::say(&format!(
        "Downloading \"{}\" for offline listening...",
        title
    ));

    let Some(token) = token else {
        error!(
            "[download_item] No token. The download of \"{}\" stopped.",
            title
        );
        crate::logic::message::say("Download failed: no authentication token.");
        return;
    };

    let id_library_item = target.item_id().to_string();

    let client = the_client_of_a_download();
    let base_url = server_address.trim_end_matches('/').to_string();

    // The list of the audio files comes from the item. The archive endpoint
    // gives a ZIP file, and the player cannot play a ZIP file.
    let item = match get_item(&client, &base_url, &token, &id_library_item).await {
        Ok(item) => item,
        Err(message) => {
            error!("[download_item] Failed to read \"{}\": {}", title, message);
            crate::logic::message::say(&format!("Download failed for \"{}\": {}", title, message));
            return;
        }
    };

    let plan = match &target {
        DownloadTarget::Book { .. } => plan_from_item(&item),
        DownloadTarget::Episode { episode_id, .. } => plan_from_episode(&item, episode_id),
    };

    let Some(plan) = plan else {
        let message = "the server gave no audio file";
        error!("[download_item] Failed to plan \"{}\": {}", title, message);
        crate::logic::message::say(&format!("Download failed for \"{}\": {}", title, message));
        return;
    };

    // Each download has its own directory. An episode of a podcast then does
    // not mix with a different episode of the same podcast.
    let dest_dir = downloads_base_dir(&username).join(&plan.key);

    match fetch_item(&client, &base_url, &token, &plan, &dest_dir, progress).await {
        Ok(paths) => {
            // The `downloads` table holds one path only. It holds the path of
            // the first audio file. Therefore the player does not change.
            let first = paths
                .first()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default();

            let _ = insert_download(
                &plan.key,
                &username,
                &title,
                &author,
                &first,
                plan.total_duration(),
                // The offline mode needs the identity of the item, because the
                // key of an episode is the identity of the episode. See T-25.
                &plan.item_id,
                // A user can have an account on more than one server. The
                // offline list shows the media of one server only.
                &server_key,
            );

            for (file, path) in plan.files.iter().zip(paths.iter()) {
                let _ = insert_download_file(
                    &plan.key,
                    &username,
                    file.index,
                    &file.ino,
                    &path.to_string_lossy(),
                    file.size,
                    file.duration,
                );
            }

            info!(
                "[download_item] Downloaded \"{}\": {} file(s) in {}",
                title,
                paths.len(),
                dest_dir.display()
            );
            crate::logic::message::say(&format!("\"{}\" is now available offline.", title));
        }
        // A second program of this account writes these files now, therefore
        // this program writes nothing and it says why. The download of the user
        // is on its way already: this is no fault of the user, and the sentence
        // must not say "failed". See T-148.
        Err(TheFaultOfTheDownload::ADifferentProgramWritesTheFiles) => {
            info!(
                "[download_item] a different program downloads \"{}\" now",
                title
            );
            crate::logic::message::say(&format!(
                "A different program of this account downloads \"{}\" now.",
                title
            ));
        }
        Err(message) => {
            error!(
                "[download_item] Failed to download \"{}\": {}",
                title, message
            );
            crate::logic::message::say(&format!("Download failed for \"{}\": {}", title, message));
        }
    }
}

/// Gives the files that the application must remove, with no file two times.
///
/// The table `downloads` holds the path of the first file, and the table
/// `download_files` holds the path of each file. The two tables therefore give
/// the first file two times.
fn paths_to_remove(first: &str, files: &[String]) -> Vec<String> {
    let mut paths: Vec<String> = Vec::with_capacity(files.len() + 1);

    for path in files.iter().chain(std::iter::once(&first.to_string())) {
        if !path.is_empty() && !paths.contains(path) {
            paths.push(path.clone());
        }
    }

    paths
}

/// Removes the local copy of a book or of an episode.
///
/// The function removes every audio file, and it removes the directory of the
/// download when that directory is empty. Then it removes the two rows of the
/// database.
///
/// The function gives the title of the download. It gives `None` when the
/// database holds no download with this key.
/// Removes the ebook of one item that the reader keeps on the disk. See T-65.
///
/// The reader of T-10 writes the file of an ebook in the directory of the
/// downloads, and it keeps that file for ever: a second visit of the book then
/// needs no request, and the reader works with no server. **Nothing removed such a
/// file before this work**, and a PDF of a scan holds some hundred megabytes.
///
/// The function gives the number of bytes that it removed. An item with no ebook on
/// the disk gives 0, and that is not a fault.
pub fn remove_the_ebook_of_the_item(item_id: &str, username: &str) -> u64 {
    // An item can hold more than one ebook, and each of them holds the name of
    // the item and the identity of its file. The key `X` removes every one of
    // them. See T-76.
    let directory = downloads_base_dir(username);

    let mut every_book: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(lines) = std::fs::read_dir(&directory) {
        for line in lines.flatten() {
            let name = line.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };

            if crate::logic::reader::session::the_file_is_an_ebook_of_the_item(name, item_id) {
                every_book.push(line.path());
            }
        }
    }

    let mut bytes = 0u64;

    for path in every_book {
        let size = match std::fs::metadata(&path) {
            Ok(data) if data.is_file() => data.len(),
            _ => continue,
        };

        match std::fs::remove_file(&path) {
            Ok(()) => {
                bytes += size;
                info!(
                    "[remove_the_ebook] the program removed the ebook of {} of {} bytes",
                    item_id, size
                );
            }
            Err(error) => error!(
                "[remove_the_ebook] the ebook {} stays: {}",
                path.display(),
                error
            ),
        }
    }

    bytes
}

/// Gives the sentence of the key `X` for the user. See T-65.
///
/// The function is pure, therefore a test needs no file.
pub fn text_of_the_removal(title: &str, the_audio_came: bool, bytes_of_the_ebook: u64) -> String {
    match (the_audio_came, bytes_of_the_ebook > 0) {
        (true, true) => format!(
            "Removed the local copy of \"{}\", and its ebook of {}.",
            title,
            text_of_the_size(bytes_of_the_ebook)
        ),
        (true, false) => format!("Removed the local copy of \"{}\".", title),
        (false, true) => format!(
            "Removed the ebook of \"{}\" of {}. It held no local copy of the audio.",
            title,
            text_of_the_size(bytes_of_the_ebook)
        ),
        (false, false) => format!("\"{}\" holds no local copy and no ebook.", title),
    }
}

/// Gives a number of bytes in the form that a person reads.
pub fn text_of_the_size(bytes: u64) -> String {
    const MEGABYTE: u64 = 1024 * 1024;

    if bytes >= MEGABYTE {
        return format!("{} MB", bytes / MEGABYTE);
    }

    format!("{} kB", (bytes / 1024).max(1))
}

pub fn remove_download(key: &str, username: &str) -> Option<String> {
    let (first, _current_time, _duration, title, _author) = get_download(key, username)?;

    let files: Vec<String> = get_download_files(key, username)
        .into_iter()
        .map(|(_index, path, _duration)| path)
        .collect();

    let paths = paths_to_remove(&first, &files);

    // The directory of the download holds the files only. Therefore the
    // application removes it after the last file.
    let directory = paths
        .first()
        .and_then(|path| std::path::Path::new(path).parent().map(PathBuf::from));

    for path in &paths {
        if let Err(error) = std::fs::remove_file(path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                error!("[remove_download] the file {} stays: {}", path, error);
            }
        }
    }

    if let Some(directory) = directory {
        // A directory that is not empty stays. The error is not important.
        let _ = std::fs::remove_dir(&directory);
    }

    let _ = delete_download(key, username);

    info!(
        "[remove_download] the application removed {} file(s) of \"{}\"",
        paths.len(),
        title
    );

    Some(title)
}

/// Gets one library item from the server.
///
/// The answer gives `media.audioFiles`. The planner reads that list.
async fn get_item(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    id_library_item: &str,
) -> Result<serde_json::Value, String> {
    let response = client
        .get(format!("{}/api/items/{}", base_url, id_library_item))
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        // This answer holds the list of the audio files, and it is a small
        // answer: it takes the limit of every request of the program. The
        // transfer of the files takes no such limit. See T-149.
        .timeout(crate::api::client::REQUEST_TIMEOUT)
        .send()
        .await
        .map_err(|error| format!("the request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("the server answered {}", response.status()));
    }

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|error| format!("the answer is not correct: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{text_of_the_removal, text_of_the_size};

    /// The key `X` removes the audio of the download and the ebook of the reader.
    /// The sentence must name what went away. See T-65.
    #[test]
    fn the_sentence_of_the_key_names_what_went_away() {
        // The title stands between the words and the size, therefore the test
        // reads the two parts and not one text of the two.
        let both = text_of_the_removal("A Book", true, 5 * 1024 * 1024);
        assert!(both.contains("A Book"), "{}", both);
        assert!(both.contains("local copy"), "{}", both);
        assert!(both.contains("ebook"), "{}", both);
        assert!(both.contains("5 MB"), "{}", both);

        let audio = text_of_the_removal("A Book", true, 0);
        assert!(audio.contains("local copy"), "{}", audio);
        assert!(!audio.contains("ebook"), "{}", audio);

        // A user who read an ebook and downloaded no audio must still remove that
        // ebook, and the sentence must say what happened.
        let ebook = text_of_the_removal("A Book", false, 137 * 1024 * 1024);
        assert!(ebook.contains("ebook"), "{}", ebook);
        assert!(ebook.contains("137 MB"), "{}", ebook);
        assert!(ebook.contains("no local copy of the audio"), "{}", ebook);

        let nothing = text_of_the_removal("A Book", false, 0);
        assert!(
            nothing.contains("no local copy and no ebook"),
            "{}",
            nothing
        );
    }

    #[test]
    fn the_size_comes_in_the_form_that_a_person_reads() {
        assert_eq!(text_of_the_size(137 * 1024 * 1024), "137 MB");
        assert_eq!(text_of_the_size(1024 * 1024), "1 MB");
        assert_eq!(text_of_the_size(500 * 1024), "500 kB");
        // A file of some bytes gives one kilobyte, and not zero.
        assert_eq!(text_of_the_size(10), "1 kB");
    }

    use super::*;

    /// A book with many files gives every file, and the first file one time
    /// only.
    #[test]
    fn the_list_holds_every_file_one_time() {
        let files = vec![
            "/d/001 - a.mp3".to_string(),
            "/d/002 - b.mp3".to_string(),
            "/d/003 - c.mp3".to_string(),
        ];

        let paths = paths_to_remove("/d/001 - a.mp3", &files);

        assert_eq!(paths, files);
    }

    /// The table `download_files` is empty in a database from an older
    /// version. The path of the first file is then the only path.
    #[test]
    fn the_list_holds_the_first_file_with_no_other_file() {
        assert_eq!(
            paths_to_remove("/d/001 - a.mp3", &[]),
            vec!["/d/001 - a.mp3".to_string()]
        );
    }

    #[test]
    fn the_list_holds_no_empty_path() {
        assert!(paths_to_remove("", &[String::new()]).is_empty());
    }

    /// The pool decides the address of a download, as it decides the address of
    /// every other request. See T-149.
    #[test]
    fn the_download_takes_the_address_that_answers() {
        assert_eq!(
            the_address_of_the_download(
                Some("https://abs.example.com".to_string()),
                "http://192.168.1.10:13378"
            ),
            "https://abs.example.com"
        );

        // A program with no block `[[servers]]` holds one address, and the pool
        // gives it. A pool with no address at all gives the address of the
        // login, and the request then says what that address says.
        assert_eq!(
            the_address_of_the_download(None, "http://192.168.1.10:13378"),
            "http://192.168.1.10:13378"
        );
    }

    /// A key of a book and a key of an episode are two different downloads.
    #[test]
    fn the_target_gives_the_key() {
        let book = DownloadTarget::Book {
            item_id: "item-1".to_string(),
        };
        let episode = DownloadTarget::Episode {
            item_id: "pod-1".to_string(),
            episode_id: "ep-1".to_string(),
        };

        assert_eq!(book.key(), "item-1");
        assert_eq!(book.item_id(), "item-1");
        assert_eq!(episode.key(), "ep-1");
        assert_eq!(episode.item_id(), "pod-1");
    }
}
