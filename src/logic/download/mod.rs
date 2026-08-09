pub mod fetch;
pub mod plan;
pub mod progress;

use crate::db::crud::{insert_download, insert_download_file};
use crate::utils::pop_up_message::*;
use fetch::fetch_item;
use log::{error, info};
use plan::{plan_from_episode, plan_from_item};
use progress::ProgressMap;
use std::collections::HashMap;
use std::env;
use std::io::stdout;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

/// Base directory where downloaded audiobooks are stored for offline listening
pub fn downloads_base_dir(username: &str) -> PathBuf {
    let base = env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");
            path.push(".local/share");
            path
        });

    base.join("toutui/downloads").join(username)
}

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
pub async fn download_with_progress(
    token: Option<String>,
    target: DownloadTarget,
    server_address: String,
    username: String,
    title: String,
    author: String,
    progress: ProgressMap,
) {
    let mut stdout = stdout();
    let _ = pop_message(
        &mut stdout,
        3,
        &format!("Downloading \"{}\" for offline listening...", title),
    );

    let Some(token) = token else {
        error!("[download_item] No token. The download of \"{}\" stopped.", title);
        let _ = pop_message(&mut stdout, 3, "Download failed: no authentication token.");
        return;
    };

    let id_library_item = target.item_id().to_string();

    let client = reqwest::Client::new();
    let base_url = server_address.trim_end_matches('/').to_string();

    // The list of the audio files comes from the item. The archive endpoint
    // gives a ZIP file, and the player cannot play a ZIP file.
    let item = match get_item(&client, &base_url, &token, &id_library_item).await {
        Ok(item) => item,
        Err(message) => {
            error!("[download_item] Failed to read \"{}\": {}", title, message);
            let _ = pop_message(
                &mut stdout,
                3,
                &format!("Download failed for \"{}\": {}", title, message),
            );
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
        let _ = pop_message(
            &mut stdout,
            3,
            &format!("Download failed for \"{}\": {}", title, message),
        );
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
            let _ = pop_message(
                &mut stdout,
                3,
                &format!("\"{}\" is now available offline.", title),
            );
        }
        Err(message) => {
            error!("[download_item] Failed to download \"{}\": {}", title, message);
            let _ = pop_message(
                &mut stdout,
                3,
                &format!("Download failed for \"{}\": {}", title, message),
            );
        }
    }
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
