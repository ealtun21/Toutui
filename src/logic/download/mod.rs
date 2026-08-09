pub mod plan;

use crate::api::library_items::download_item::download_library_item_file;
use crate::db::crud::insert_download;
use crate::utils::pop_up_message::*;
use std::env;
use std::io::stdout;
use std::path::PathBuf;
use log::{error, info};

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

/// Download a library item for offline listening and register it in the local database
pub async fn download_item(
    token: Option<String>,
    id_library_item: String,
    server_address: String,
    username: String,
    title: String,
    author: String,
    duration: f64,
) {
    let mut stdout = stdout();
    let _ = pop_message(&mut stdout, 3, &format!("Downloading \"{}\" for offline listening...", title));

    let dest_dir = downloads_base_dir(&username).join(&id_library_item);
    let fallback_filename = format!("{}.m4b", id_library_item);

    match download_library_item_file(token.as_ref(), &id_library_item, server_address, &dest_dir, &fallback_filename).await {
        Ok(path) => {
            let path_str = path.to_string_lossy().to_string();
            let _ = insert_download(&id_library_item, &username, &title, &author, &path_str, duration);
            info!("[download_item] Downloaded \"{}\" to {}", title, path_str);
            let _ = pop_message(&mut stdout, 3, &format!("\"{}\" is now available offline.", title));
        }
        Err(e) => {
            error!("[download_item] Failed to download \"{}\": {}", title, e);
            let _ = pop_message(&mut stdout, 3, &format!("Download failed for \"{}\": {}", title, e));
        }
    }
}
