use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_DISPOSITION};
use color_eyre::eyre::{eyre, Result};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Download a Library Item for offline listening
/// GET /api/items/:id/download
/// Requires the user to have the "download" permission on the server.
/// Returns the raw audio file for single-file items, or a zip archive for
/// multi-file items.
pub async fn download_library_item_file(
    token: Option<&String>,
    id_library_item: &str,
    server_address: String,
    dest_dir: &Path,
    fallback_filename: &str,
) -> Result<PathBuf> {
    let token = token.ok_or_else(|| eyre!("Missing auth token"))?;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/items/{}/download", server_address, id_library_item))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(eyre!("Download request failed with status {}", response.status()));
    }

    let filename = response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .split("filename=")
                .nth(1)
                .map(|f| f.trim_matches('"').to_string())
        })
        .unwrap_or_else(|| fallback_filename.to_string());

    tokio::fs::create_dir_all(dest_dir).await?;
    let dest_path = dest_dir.join(&filename);

    let bytes = response.bytes().await?;
    let mut file = tokio::fs::File::create(&dest_path).await?;
    file.write_all(&bytes).await?;

    Ok(dest_path)
}
