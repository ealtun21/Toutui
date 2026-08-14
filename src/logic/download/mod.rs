pub mod fetch;
pub mod lock;
pub mod plan;
pub mod progress;

use crate::db::crud::{
    delete_download, get_download, get_download_files, insert_download, insert_download_file,
};
use fetch::{fetch_item, TheFaultOfTheDownload};
use log::{error, info};
use plan::{plan_from_episode, plan_from_item, the_words_of_a_plan_that_did_not_come};
use progress::{claim_the_download, release_the_download, ProgressMap, TheClaimOfTheDownload};
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

    /// Gives the identity of the episode. A book has no episode.
    ///
    /// The place of a media stands under the item and the episode, therefore a
    /// key that asks for the place of a media gives both. See T-156.
    pub fn episode_id(&self) -> Option<&str> {
        match self {
            DownloadTarget::Book { .. } => None,
            DownloadTarget::Episode { episode_id, .. } => Some(episode_id),
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
    // **This program holds the map of its own downloads**, and the key of that
    // map is the media. A second press of the key `D` on one media therefore
    // wrote over the row of the download that runs, and the bar of that
    // download went off the screen for the whole of it. The claim holds the
    // place of the media, and it names this program with the truth. See T-154.
    if claim_the_download(&progress, target.key(), &title)
        == TheClaimOfTheDownload::ThisProgramDownloadsIt
    {
        info!("[download_item] this program downloads \"{}\" now", title);
        crate::logic::message::say(&text_of_the_key_that_downloads(&title));
        return;
    }

    crate::logic::message::say(&format!(
        "Downloading \"{}\" for offline listening...",
        title
    ));

    let Some(token) = token else {
        error!(
            "[download_item] No token. The download of \"{}\" stopped.",
            title
        );
        release_the_download(&progress, target.key(), "no authentication token");
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
            release_the_download(&progress, target.key(), &message);
            crate::logic::message::say(&format!("Download failed for \"{}\": {}", title, message));
            return;
        }
    };

    let plan = match &target {
        DownloadTarget::Book { .. } => plan_from_item(&item),
        DownloadTarget::Episode { episode_id, .. } => plan_from_episode(&item, episode_id),
    };

    // **The words of the fault name what the server did not give** (T-181): a
    // file with no `ino` is not "no audio file", and the program must not take
    // the other files of that book alone.
    let plan = match plan {
        Ok(plan) => plan,
        Err(why) => {
            let message = the_words_of_a_plan_that_did_not_come(&why);
            error!("[download_item] Failed to plan \"{}\": {}", title, message);
            release_the_download(&progress, target.key(), &message);
            crate::logic::message::say(&format!("Download failed for \"{}\": {}", title, message));
            return;
        }
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

/// What the audio of a media gave to the key `X`. See T-150.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheAudioOfTheRemoval {
    /// The disk held the whole download, and the database held its row.
    TheWholeCopy,
    /// The disk held the bytes of a download that did not come to its end. The
    /// offline mode plays no such file, and no key removed it before T-150.
    ThePartOfADownload(u64),
    /// The disk held no audio of this media at all.
    Nothing,
}

/// Gives the sentence of the key `X` for the user. See T-65 and T-150.
///
/// The function is pure, therefore a test needs no file.
pub fn text_of_the_removal(
    title: &str,
    of_the_audio: &TheAudioOfTheRemoval,
    bytes_of_the_ebook: u64,
) -> String {
    let of_the_ebook = if bytes_of_the_ebook > 0 {
        format!(
            " Removed its ebook of {}.",
            text_of_the_size(bytes_of_the_ebook)
        )
    } else {
        String::new()
    };

    match of_the_audio {
        TheAudioOfTheRemoval::TheWholeCopy => {
            format!("Removed the local copy of \"{}\".{}", title, of_the_ebook)
        }
        // **The sentence must not say "the local copy"**: the offline mode
        // plays no part of a download, therefore the user had no copy at all.
        TheAudioOfTheRemoval::ThePartOfADownload(bytes) => format!(
            "Removed {} of a download of \"{}\" that did not come to its end.{}",
            text_of_the_size(*bytes),
            title,
            of_the_ebook
        ),
        TheAudioOfTheRemoval::Nothing if bytes_of_the_ebook > 0 => format!(
            "Removed the ebook of \"{}\" of {}. It held no local copy of the audio.",
            title,
            text_of_the_size(bytes_of_the_ebook)
        ),
        TheAudioOfTheRemoval::Nothing => {
            format!("\"{}\" holds no local copy and no ebook.", title)
        }
    }
}

/// Gives the sentence of the key `X` for a download that runs now. See T-150.
///
/// The function is pure, therefore a test needs no file. **The sentence must
/// not promise a key that the program does not hold** (T-143): no key of this
/// program stops a download.
pub fn text_of_the_download_that_runs(title: &str, of_this_program: bool) -> String {
    if of_this_program {
        return format!(
            "This program downloads \"{}\" now. The key X removes it when that download ends.",
            title
        );
    }

    format!(
        "A different program of this account downloads \"{}\" now. The key X removes it when that download ends.",
        title
    )
}

/// Gives the sentence of the key `D` for a download that this program runs
/// already. See T-154.
///
/// The key `X` holds two sentences since T-150 — the program of this window and
/// the program of the other window — and the key `D` held the sentence of the
/// other window alone. **A user who presses the key two times then reads that a
/// different program downloads their media**, and no different program exists.
///
/// The sentence promises no key that the program does not hold (T-118 and
/// T-143): no key of this program stops a download.
///
/// The function is pure, therefore a test needs no server.
pub fn text_of_the_key_that_downloads(title: &str) -> String {
    format!("This program downloads \"{}\" now.", title)
}

/// Gives the sentence of the key `X` for a media that plays from the disk. See
/// T-156.
///
/// **The sentence names no program.** The place of an offline playback stands in
/// one table for the whole account, and no column of it holds a process: a
/// sentence that named this program or a different program would say a thing
/// that the program does not know (T-91 and T-154). It promises no key, because
/// no key of this program stops the playback of a different window (T-118 and
/// T-143).
///
/// The function is pure, therefore a test needs no server.
pub fn text_of_the_media_that_plays_from_the_disk(title: &str) -> String {
    format!(
        "A program of this account plays \"{}\" from the disk now.",
        title
    )
}

/// What the key `X` must do with the files of one download. See T-150.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheWorkOfTheKeyThatRemoves {
    /// No program writes these files. The key takes the disk.
    TakeTheDisk,
    /// This program downloads the media now.
    ThisProgramDownloads,
    /// A different program of this account downloads the media now.
    ADifferentProgramDownloads,
    /// A program of this account plays the media from the disk now. See T-156.
    AProgramPlaysItFromTheDisk,
}

/// Says what the key `X` must do with a download. See T-150.
///
/// **A removal that takes the files of a writer gives that writer a fault**, and
/// it gives the user "Download failed" for a download that works: that is the
/// shape of T-148 from the other side. The key therefore removes nothing while a
/// program writes those files, and it says which program that is.
///
/// The function is pure, therefore a test needs no file and no server.
pub fn the_work_of_the_key_that_removes(
    this_program_downloads: bool,
    a_program_writes_the_files: bool,
    a_program_plays_it_from_the_disk: bool,
) -> TheWorkOfTheKeyThatRemoves {
    // **The media that a program of this account plays from the disk keeps its
    // files**, and that rule stands before every other rule of this key: the
    // user hears that media at this second, and the server can be away. It is
    // the rule of the cache of the ebooks — "the book that the user reads now
    // never goes away" (T-65 and T-153) — for the audio. See T-156.
    if a_program_plays_it_from_the_disk {
        return TheWorkOfTheKeyThatRemoves::AProgramPlaysItFromTheDisk;
    }

    if this_program_downloads {
        return TheWorkOfTheKeyThatRemoves::ThisProgramDownloads;
    }

    if a_program_writes_the_files {
        return TheWorkOfTheKeyThatRemoves::ADifferentProgramDownloads;
    }

    TheWorkOfTheKeyThatRemoves::TakeTheDisk
}

/// Says that this program gets the files of this download now.
///
/// The map of the progress is a map of the process (T-148), therefore this
/// answer holds for this program alone.
pub fn this_program_downloads(key: &str) -> bool {
    downloads()
        .read()
        .map(|all| {
            all.get(key)
                .is_some_and(|one| one.state == progress::DownloadState::Running)
        })
        .unwrap_or(false)
}

/// Says that a program of this account writes the files of this download now.
pub fn a_program_downloads(key: &str, username: &str) -> bool {
    lock::a_program_writes_the_files(&downloads_base_dir(username).join(key))
}

/// Removes the directory of one download from the disk, and gives its bytes.
///
/// **The disk is the truth** (T-142, T-147, and T-148): the database holds a row
/// after the **last** byte of the last file, therefore the bytes of a download
/// that stopped stand in no row at all. This function removes every file of the
/// directory — the audio, the `.part` file of a download that did not end, and
/// the lock of T-148 — and then the directory. See T-150.
pub fn remove_the_directory_of_the_download(key: &str, username: &str) -> u64 {
    let directory = downloads_base_dir(username).join(key);

    let Ok(rows) = std::fs::read_dir(&directory) else {
        return 0;
    };

    let mut bytes = 0u64;

    for row in rows.flatten() {
        let size = match row.metadata() {
            Ok(data) if data.is_file() => data.len(),
            _ => continue,
        };

        match std::fs::remove_file(row.path()) {
            Ok(()) => bytes += size,
            Err(error) => error!(
                "[remove_the_directory] the file {} stays: {}",
                row.path().display(),
                error
            ),
        }
    }

    // A directory that is not empty stays. The error is not important.
    let _ = std::fs::remove_dir(&directory);

    bytes
}

/// Gives a number of bytes in the form that a person reads.
pub fn text_of_the_size(bytes: u64) -> String {
    const MEGABYTE: u64 = 1024 * 1024;

    if bytes >= MEGABYTE {
        return format!("{} MB", bytes / MEGABYTE);
    }

    format!("{} kB", (bytes / 1024).max(1))
}

/// Removes the audio of one download of the disk, and the row of the database.
///
/// **The caller must first ask [`the_work_of_the_key_that_removes`]**: a
/// removal that takes the files of a program that writes them gives that
/// program a fault (T-150).
///
/// The function gives the title of the row of the database, and what the disk
/// held. **A download that did not come to its end holds no row at all**, and
/// its bytes stand on the disk: the disk is the truth here, and not the
/// database.
pub fn remove_download(key: &str, username: &str) -> (Option<String>, TheAudioOfTheRemoval) {
    let row = get_download(key, username);

    // The row of an older version of the program holds a path of a directory
    // that this program does not make. Those files go away first, and the
    // directory of the download goes away after them.
    if let Some((first, _current_time, _duration, _title, _author)) = &row {
        let files: Vec<String> = get_download_files(key, username)
            .into_iter()
            .map(|(_index, path, _duration)| path)
            .collect();

        for path in paths_to_remove(first, &files) {
            let of_this_download = std::path::Path::new(&path)
                .parent()
                .map(PathBuf::from)
                .is_some_and(|directory| directory == downloads_base_dir(username).join(key));

            if of_this_download {
                continue;
            }

            if let Err(error) = std::fs::remove_file(&path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    error!("[remove_download] the file {} stays: {}", path, error);
                }
            }
        }
    }

    let bytes = remove_the_directory_of_the_download(key, username);

    let title = row.map(|(_first, _current_time, _duration, title, _author)| title);

    let of_the_audio = match (&title, bytes) {
        (Some(_), _) => TheAudioOfTheRemoval::TheWholeCopy,
        (None, 0) => TheAudioOfTheRemoval::Nothing,
        (None, bytes) => TheAudioOfTheRemoval::ThePartOfADownload(bytes),
    };

    if of_the_audio != TheAudioOfTheRemoval::Nothing {
        let _ = delete_download(key, username);

        info!(
            "[remove_download] the application removed {} bytes of the download {}",
            bytes, key
        );
    }

    (title, of_the_audio)
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
    use super::{
        text_of_the_download_that_runs, text_of_the_removal, text_of_the_size,
        the_work_of_the_key_that_removes, TheAudioOfTheRemoval, TheWorkOfTheKeyThatRemoves,
    };

    /// The key `X` removes the audio of the download and the ebook of the reader.
    /// The sentence must name what went away. See T-65.
    #[test]
    fn the_sentence_of_the_key_names_what_went_away() {
        // The title stands between the words and the size, therefore the test
        // reads the two parts and not one text of the two.
        let both = text_of_the_removal(
            "A Book",
            &TheAudioOfTheRemoval::TheWholeCopy,
            5 * 1024 * 1024,
        );
        assert!(both.contains("A Book"), "{}", both);
        assert!(both.contains("local copy"), "{}", both);
        assert!(both.contains("ebook"), "{}", both);
        assert!(both.contains("5 MB"), "{}", both);

        let audio = text_of_the_removal("A Book", &TheAudioOfTheRemoval::TheWholeCopy, 0);
        assert!(audio.contains("local copy"), "{}", audio);
        assert!(!audio.contains("ebook"), "{}", audio);

        // A user who read an ebook and downloaded no audio must still remove that
        // ebook, and the sentence must say what happened.
        let ebook =
            text_of_the_removal("A Book", &TheAudioOfTheRemoval::Nothing, 137 * 1024 * 1024);
        assert!(ebook.contains("ebook"), "{}", ebook);
        assert!(ebook.contains("137 MB"), "{}", ebook);
        assert!(ebook.contains("no local copy of the audio"), "{}", ebook);

        let nothing = text_of_the_removal("A Book", &TheAudioOfTheRemoval::Nothing, 0);
        assert!(
            nothing.contains("no local copy and no ebook"),
            "{}",
            nothing
        );
    }

    /// The bytes of a download that did not come to its end are no local copy:
    /// the offline mode plays no such file. See T-150.
    #[test]
    fn the_sentence_of_a_download_that_did_not_end_says_no_local_copy() {
        let part = text_of_the_removal(
            "A Book",
            &TheAudioOfTheRemoval::ThePartOfADownload(7 * 1024 * 1024),
            0,
        );

        assert!(part.contains("A Book"), "{}", part);
        assert!(part.contains("7 MB"), "{}", part);
        assert!(part.contains("did not come to its end"), "{}", part);
        assert!(
            !part.contains("the local copy"),
            "a part of a download is no local copy: {}",
            part
        );
    }

    /// A download that runs holds its files, and the key `X` says which program
    /// writes them. See T-150.
    #[test]
    fn the_key_takes_no_file_of_a_download_that_runs() {
        assert_eq!(
            the_work_of_the_key_that_removes(false, false, false),
            TheWorkOfTheKeyThatRemoves::TakeTheDisk
        );
        assert_eq!(
            the_work_of_the_key_that_removes(true, true, false),
            TheWorkOfTheKeyThatRemoves::ThisProgramDownloads
        );
        // The lock of this program stands on the disk, therefore the two
        // answers agree. A lock that this program did not make gives the other
        // program.
        assert_eq!(
            the_work_of_the_key_that_removes(false, true, false),
            TheWorkOfTheKeyThatRemoves::ADifferentProgramDownloads
        );

        let of_this = text_of_the_download_that_runs("A Book", true);
        assert!(of_this.contains("This program downloads"), "{}", of_this);
        assert!(of_this.contains("A Book"), "{}", of_this);

        let of_the_other = text_of_the_download_that_runs("A Book", false);
        assert!(
            of_the_other.contains("A different program"),
            "{}",
            of_the_other
        );
    }

    /// **The media that a program of this account plays from the disk keeps its
    /// files**, and that rule stands before the rule of a download. See T-156.
    #[test]
    fn the_key_keeps_a_media_that_plays_from_the_disk() {
        assert_eq!(
            the_work_of_the_key_that_removes(false, false, true),
            TheWorkOfTheKeyThatRemoves::AProgramPlaysItFromTheDisk
        );

        // A download of this program and a playback of the disk can stand at
        // one moment. The playback decides: its files go away under the ear of
        // the user, and the server can be away.
        assert_eq!(
            the_work_of_the_key_that_removes(true, true, true),
            TheWorkOfTheKeyThatRemoves::AProgramPlaysItFromTheDisk
        );

        let text = text_of_the_media_that_plays_from_the_disk("A Book Of Many Hours");

        assert!(text.contains("A Book Of Many Hours"), "{}", text);
        assert!(text.contains("from the disk"), "{}", text);
        // The table of the place holds no program, therefore the sentence names
        // none: it must not say "a different program" for the window of the
        // user (T-154), and it must promise no key (T-118).
        assert!(!text.contains("different program"), "{}", text);
        assert!(!text.contains("Press"), "{}", text);
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
