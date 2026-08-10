//! Tests of the update in the program.
//!
//! The tests give the answers of the API from a mock server. No test uses the
//! network.

use toutui::update::release::{parse_release, target, Release};

/// Gives an answer of the API with the assets of a release.
fn answer(tag: &str) -> String {
    format!(
        r#"{{
            "tag_name": "{tag}",
            "assets": [
                {{"name": "SHA256SUMS",
                  "browser_download_url": "https://example.test/{tag}/SHA256SUMS"}},
                {{"name": "toutui-x86_64-unknown-linux-gnu.tar.gz",
                  "browser_download_url": "https://example.test/{tag}/toutui-x86_64-unknown-linux-gnu.tar.gz"}},
                {{"name": "toutui-universal-apple-darwin.tar.gz",
                  "browser_download_url": "https://example.test/{tag}/toutui-universal-apple-darwin.tar.gz"}}
            ]
        }}"#
    )
}

/// The program finds the archive of its target and the file of the sums.
#[test]
fn the_program_finds_the_archive_of_the_target() {
    let release = parse_release(&answer("v0.6.0-beta"), "x86_64-unknown-linux-gnu").unwrap();

    assert_eq!(
        release,
        Release {
            version: "0.6.0-beta".to_string(),
            archive_name: "toutui-x86_64-unknown-linux-gnu.tar.gz".to_string(),
            archive_url: "https://example.test/v0.6.0-beta/toutui-x86_64-unknown-linux-gnu.tar.gz"
                .to_string(),
            sums_url: "https://example.test/v0.6.0-beta/SHA256SUMS".to_string(),
        }
    );
}

/// A release that has no archive for the target gives a clear error.
#[test]
fn a_target_without_an_archive_gives_an_error() {
    let error = parse_release(&answer("v0.6.0-beta"), "aarch64-unknown-linux-gnu").unwrap_err();

    assert!(error.contains("aarch64-unknown-linux-gnu"));
}

/// An answer that has no tag gives an error.
#[test]
fn an_answer_without_a_tag_gives_an_error() {
    assert!(parse_release("{}", "x86_64-unknown-linux-gnu").is_err());
}

/// The program knows the target that it runs on.
#[test]
fn the_program_knows_its_target() {
    if cfg!(any(target_os = "linux", target_os = "macos")) {
        assert!(target().is_some());
    }
}
