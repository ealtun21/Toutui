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

/// Removes the letter `v` from the front of a tag, without regard to the case.
///
/// A person can write the tag as `v0.5.1` or as `V0.5.1`. The program used
/// `trim_start_matches('v')`, and that function looks at the case. Therefore a
/// tag `V0.5.1` kept its letter, the comparison with the version of the build
/// never agreed, and the message "a version is available" stayed on the screen
/// after the user updated. See T-28.
pub fn version_of_tag(tag: &str) -> &str {
    tag.strip_prefix('v')
        .or_else(|| tag.strip_prefix('V'))
        .unwrap_or(tag)
}

/// Gives `true` when the release is newer than the version of this build.
///
/// The comparison uses semver. If either version does not parse, the function
/// gives `true` only when the two strings disagree, which is the behaviour
/// that the program had before semver.
pub fn is_newer(remote: &str, local: &str) -> bool {
    match (
        semver::Version::parse(remote),
        semver::Version::parse(local),
    ) {
        (Ok(remote), Ok(local)) => remote > local,
        _ => remote != local,
    }
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
        version: version_of_tag(tag).to_string(),
        archive_name,
        archive_url,
        sums_url,
    })
}

/// Asks the API for the last release.
///
/// The request has a limit on time, so that a host that accepts the
/// connection and never answers cannot stop the update forever with no
/// message.
pub async fn latest_release(api: &str, target: &str) -> Result<Release, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(api)
        .header(reqwest::header::USER_AGENT, "Toutui-Updater")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        return Err(
            "GitHub reached the limit of requests with no account. Wait, and try again."
                .to_string(),
        );
    }
    if !status.is_success() {
        return Err(format!("The API gives {}.", status));
    }

    let body = response.text().await.map_err(|e| e.to_string())?;

    parse_release(&body, target)
}

#[cfg(test)]
mod tests {
    use super::{is_newer, version_of_tag};

    #[test]
    fn a_tag_with_a_small_letter_loses_it() {
        assert_eq!(version_of_tag("v0.5.1"), "0.5.1");
    }

    #[test]
    fn a_tag_with_a_capital_letter_loses_it() {
        assert_eq!(version_of_tag("V0.5.1"), "0.5.1");
    }

    #[test]
    fn a_tag_with_no_letter_does_not_change() {
        assert_eq!(version_of_tag("0.5.1"), "0.5.1");
    }

    #[test]
    fn the_function_removes_one_letter_only() {
        assert_eq!(version_of_tag("vv0.5.1"), "v0.5.1");
    }

    #[test]
    fn a_tag_with_a_capital_letter_agrees_with_the_build() {
        // This is T-28. The program compared `V0.5.1` with `0.5.1`, no
        // comparison agreed, and the message stayed on the screen for ever.
        assert!(!is_newer(version_of_tag("V0.5.1"), "0.5.1"));
    }

    /// A release that this repository writes as a candidate must never take
    /// the place of the release itself.
    ///
    /// Semver puts a version with a pre-release below the same version with
    /// none. Therefore `0.6.0-rc.1` is older than `0.6.0`, and a user of the
    /// candidate gets the message when `0.6.0` comes.
    ///
    /// A name that semver cannot read has no such rule. `0.6.0beta1` is not a
    /// version at all, and the function then compares the two texts only.
    #[test]
    fn a_candidate_stands_below_its_release() {
        assert!(is_newer("0.6.0", "0.6.0-rc.1"));
        assert!(!is_newer("0.6.0-rc.1", "0.6.0"));
        assert!(!is_newer("0.6.0-rc.1", "0.6.0-rc.2"));
        assert!(is_newer("0.6.0-rc.2", "0.6.0-rc.1"));

        // The release before it must not give a message to a candidate.
        assert!(!is_newer("0.5.0", "0.6.0-rc.1"));

        // A tag with a capital letter still gives the same answer. See T-28.
        assert!(!is_newer(version_of_tag("V0.6.0-rc.1"), "0.6.0-rc.1"));

        // A name that is not a version compares as a text. The two disagree,
        // therefore the program would offer an update. This is the reason for
        // the rule: write a candidate as `0.6.0-rc.N`, and nothing else.
        assert!(is_newer("0.6.0beta1", "0.6.0"));
    }

    #[test]
    fn newer_release_gives_true() {
        assert!(is_newer("0.6.0", "0.5.0"));
    }

    #[test]
    fn equal_release_gives_false() {
        assert!(!is_newer("0.5.0", "0.5.0"));
    }

    #[test]
    fn older_release_gives_false() {
        assert!(!is_newer("0.4.0", "0.5.0"));
    }

    #[test]
    fn a_version_that_does_not_parse_falls_back_to_text() {
        // Neither string is semver here, so the function keeps the old
        // rule: not equal means newer.
        assert!(is_newer("not-a-version", "0.5.0"));
        assert!(!is_newer("0.5.0", "0.5.0"));
    }
}
