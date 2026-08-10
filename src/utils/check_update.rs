use serde_json::Value;
use reqwest::header::USER_AGENT;
use reqwest::Client;
use crate::update::release::{is_newer, version_of_tag};

/// The address that gives the last release of the fork.
///
/// The program before the fork asked `AlbanDAVID/Toutui`, and that repository
/// is archived. Therefore the program never saw a release of the fork. See
/// T-21.
pub const RELEASES_API: &str = "https://api.github.com/repos/ealtun21/Toutui/releases/latest";

const LOCAL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn check_update() -> Option<String> {
    match get_latest_release_gh().await {
        Ok(latest_version_gh) => {
            if is_newer(&latest_version_gh, LOCAL_VERSION) {
                log::warn!(
                    "You are not up-to-date. Current: {} / Available: {}",
                    LOCAL_VERSION,
                    latest_version_gh
                );
                Some(format!(
                    "🔄 Version {} is available. Run `toutui --update`.",
                    latest_version_gh
                ))
            } else {
                None
            }
        }
        Err(e) => {
            log::error!("{}", e);
            None
        }
    }
}

pub async fn get_latest_release_gh() -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client
        .get(RELEASES_API)
        .header(USER_AGENT, "Toutui-Updater")
        .send()
        .await?;
    let text = response.text().await?;

    let v: Value = serde_json::from_str(&text)?;

    if let Some(tag_name) = v["tag_name"].as_str() {
        Ok(version_of_tag(tag_name).to_string())
    } else {
        Err("[get_latest_release_gh] couldn't find last release".into())
    }
}

