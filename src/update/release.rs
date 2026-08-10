//! The release that the API of GitHub gives.

use serde_json::Value;

/// One release, with the addresses that the update needs.
#[derive(Debug, PartialEq, Eq)]
pub struct Release {
    /// The version without the letter `v`.
    pub version: String,
    /// The name of the archive of this target.
    pub archive_name: String,
    /// The address of the archive.
    pub archive_url: String,
    /// The address of the file of the sums.
    pub sums_url: String,
}

/// Gives the target of this build.
///
/// The value must agree with the names of the assets that the workflow
/// `release.yml` makes.
pub fn target() -> Option<&'static str> {
    if cfg!(target_os = "macos") {
        Some("universal-apple-darwin")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("x86_64-unknown-linux-gnu")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        Some("aarch64-unknown-linux-gnu")
    } else {
        None
    }
}

/// Finds the address of one asset in the answer of the API.
fn asset_url(assets: &[Value], name: &str) -> Option<String> {
    assets
        .iter()
        .find(|asset| asset["name"].as_str() == Some(name))
        .and_then(|asset| asset["browser_download_url"].as_str())
        .map(|url| url.to_string())
}

/// Reads the answer of the API and finds the archive of this target.
pub fn parse_release(body: &str, target: &str) -> Result<Release, String> {
    let value: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;

    let tag = value["tag_name"]
        .as_str()
        .ok_or_else(|| "The answer of the API has no tag.".to_string())?;

    let assets = value["assets"]
        .as_array()
        .ok_or_else(|| "The answer of the API has no assets.".to_string())?;

    let archive_name = format!("toutui-{}.tar.gz", target);

    let archive_url = asset_url(assets, &archive_name)
        .ok_or_else(|| format!("The release {} has no archive for {}.", tag, target))?;

    let sums_url = asset_url(assets, "SHA256SUMS")
        .ok_or_else(|| format!("The release {} has no SHA256SUMS.", tag))?;

    Ok(Release {
        version: tag.trim_start_matches('v').to_string(),
        archive_name,
        archive_url,
        sums_url,
    })
}

/// Asks the API for the last release.
pub async fn latest_release(api: &str, target: &str) -> Result<Release, String> {
    let body = reqwest::Client::new()
        .get(api)
        .header(reqwest::header::USER_AGENT, "Toutui-Updater")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    parse_release(&body, target)
}
