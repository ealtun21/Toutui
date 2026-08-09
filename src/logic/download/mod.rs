pub mod fetch;
pub mod plan;
pub mod progress;

use crate::db::crud::{insert_download, insert_download_file};
use crate::utils::pop_up_message::*;
use fetch::fetch_item;
use log::{error, info};
use plan::plan_from_item;
use progress::ProgressMap;
use std::collections::HashMap;
use std::env;
use std::io::stdout;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

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

/// Downloads a library item for offline listening and writes it to the local
/// database.
///
/// The function makes its own map of the progress. The user interface cannot
/// read that map. Use `download_item_with_progress` to give a map that the
/// user interface reads.
#[allow(clippy::too_many_arguments)]
pub async fn download_item(
    token: Option<String>,
    id_library_item: String,
    server_address: String,
    username: String,
    title: String,
    author: String,
    duration: f64,
) {
    download_item_with_progress(
        token,
        id_library_item,
        server_address,
        username,
        title,
        author,
        duration,
        new_progress_map(),
    )
    .await;
}

/// Downloads a library item and writes its progress to the given map.
///
/// The function gets the audio files one at a time. A file that is not
/// complete stays on the disk with the name `.part`. The next call continues
/// that file.
#[allow(clippy::too_many_arguments)]
pub async fn download_item_with_progress(
    token: Option<String>,
    id_library_item: String,
    server_address: String,
    username: String,
    title: String,
    author: String,
    _duration: f64,
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

    let Some(plan) = plan_from_item(&item) else {
        let message = "the server gave no audio file";
        error!("[download_item] Failed to plan \"{}\": {}", title, message);
        let _ = pop_message(
            &mut stdout,
            3,
            &format!("Download failed for \"{}\": {}", title, message),
        );
        return;
    };

    let dest_dir = downloads_base_dir(&username).join(&id_library_item);

    match fetch_item(&client, &base_url, &token, &plan, &dest_dir, progress).await {
        Ok(paths) => {
            // The `downloads` table holds one path only. It holds the path of
            // the first audio file. Therefore the player does not change.
            let first = paths
                .first()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default();

            let _ = insert_download(
                &id_library_item,
                &username,
                &title,
                &author,
                &first,
                plan.total_duration(),
            );

            for (file, path) in plan.files.iter().zip(paths.iter()) {
                let _ = insert_download_file(
                    &id_library_item,
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
