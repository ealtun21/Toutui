//! Tests of the update in the program.
//!
//! The tests give the answers of the API from a mock server. No test uses the
//! network.

use std::io::Write;
use toutui::update::install::{binary_from_archive, expected_sum, replace_binary, sum_of};
use toutui::update::release::{parse_release, target, Release};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

/// Makes a `tar.gz` that holds one file with the name `toutui`.
fn archive_of(contents: &[u8]) -> Vec<u8> {
    let mut tar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    tar.append_data(&mut header, "toutui", contents).unwrap();
    let tar = tar.into_inner().unwrap();

    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(&tar).unwrap();
    gz.finish().unwrap()
}

/// The sum of an empty input is the known sum of SHA-256.
#[test]
fn the_program_calculates_the_sum() {
    assert_eq!(
        sum_of(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

/// The program finds the sum of one name in the file of the sums.
#[test]
fn the_program_finds_the_sum_of_a_name() {
    let sums = "aaa  toutui-x86_64-unknown-linux-gnu.tar.gz\n\
                bbb  toutui-universal-apple-darwin.tar.gz\n";

    assert_eq!(
        expected_sum(sums, "toutui-universal-apple-darwin.tar.gz"),
        Some("bbb".to_string())
    );
    assert_eq!(expected_sum(sums, "toutui-aarch64-unknown-linux-gnu.tar.gz"), None);
}

/// The program takes the binary out of the archive.
#[test]
fn the_program_opens_the_archive() {
    let archive = archive_of(b"the new binary");

    assert_eq!(binary_from_archive(&archive).unwrap(), b"the new binary");
}

/// An archive that holds no file with the name of the program gives an error.
#[test]
fn an_archive_without_the_binary_gives_an_error() {
    let mut tar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(3);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, "README", &b"abc"[..]).unwrap();
    let tar = tar.into_inner().unwrap();
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(&tar).unwrap();

    assert!(binary_from_archive(&gz.finish().unwrap()).is_err());
}

/// A directory with the name of the program gives an error, and not an empty
/// binary. A symlink or a directory has the size 0 in a tar file, so a check
/// of the name only would give an empty file with no error.
#[test]
fn a_directory_with_the_name_of_the_binary_gives_an_error() {
    let mut tar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Directory);
    header.set_size(0);
    header.set_mode(0o755);
    header.set_cksum();
    tar.append_data(&mut header, "toutui", &b""[..]).unwrap();
    let tar = tar.into_inner().unwrap();
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(&tar).unwrap();

    assert!(binary_from_archive(&gz.finish().unwrap()).is_err());
}

/// The program moves the new binary on to the old binary, and the new binary
/// can run.
#[test]
fn the_program_replaces_the_binary() {
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    replace_binary(&binary, b"the new binary").unwrap();

    assert_eq!(std::fs::read(&binary).unwrap(), b"the new binary");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&binary).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
    }
}

/// The program keeps the mode of the binary that was present, so that a
/// binary that was private to its owner does not become readable by every
/// user.
#[cfg(unix)]
#[test]
fn the_program_keeps_the_mode_of_the_old_binary() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();

    replace_binary(&binary, b"the new binary").unwrap();

    let mode = std::fs::metadata(&binary).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
}

/// A directory that the user cannot write gives `false`.
///
/// The test skips when the user is root, because root can write in a
/// directory of the mode 0o555 as well, and the assertion would then fail
/// with no fault in the program.
#[cfg(unix)]
#[test]
fn a_directory_that_is_read_only_gives_false() {
    use std::os::unix::fs::PermissionsExt;
    use toutui::update::install::can_replace;

    if std::env::var("USER").as_deref() == Ok("root") {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();

    let result = can_replace(&binary);

    // The permissions come back, so that the temporary directory goes away.
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();

    assert!(!result);
}

/// The program stops and does not touch the binary when the sum disagrees.
///
/// The test gives `run_update_at` a path inside a directory that the test
/// made, and not the binary of the test itself. Therefore, if the order of
/// the test of the sum and the move of the file ever changes, this test
/// fails an assertion instead of writing over the test harness.
#[tokio::test]
async fn a_sum_that_disagrees_stops_the_update() {
    let server = MockServer::start().await;
    let target = toutui::update::release::target().unwrap();
    let name = format!("toutui-{}.tar.gz", target);

    let body = serde_json::json!({
        "tag_name": "v99.0.0",
        "assets": [
            {"name": "SHA256SUMS",
             "browser_download_url": format!("{}/SHA256SUMS", server.uri())},
            {"name": name,
             "browser_download_url": format!("{}/archive", server.uri())}
        ]
    });

    Mock::given(method("GET")).and(path("/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/archive"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(archive_of(b"new")))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/SHA256SUMS"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(format!("{}  {}\n", "0".repeat(64), name)))
        .mount(&server).await;

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();

    let error = toutui::update::install::run_update_at(&format!("{}/latest", server.uri()), &binary)
        .await
        .unwrap_err();

    assert!(error.contains("not correct"));
    assert_eq!(std::fs::read(&binary).unwrap(), b"the old binary");
}

/// A release that is older than the build does not replace the binary.
#[tokio::test]
async fn an_older_release_does_not_replace_the_binary() {
    let server = MockServer::start().await;
    let target = toutui::update::release::target().unwrap();
    let name = format!("toutui-{}.tar.gz", target);

    let body = serde_json::json!({
        "tag_name": "v0.0.1",
        "assets": [
            {"name": "SHA256SUMS",
             "browser_download_url": format!("{}/SHA256SUMS", server.uri())},
            {"name": name,
             "browser_download_url": format!("{}/archive", server.uri())}
        ]
    });

    Mock::given(method("GET")).and(path("/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server).await;

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();

    let message = toutui::update::install::run_update_at(&format!("{}/latest", server.uri()), &binary)
        .await
        .unwrap();

    assert!(message.contains("not newer"));
    assert_eq!(std::fs::read(&binary).unwrap(), b"the old binary");
}

/// A release that is newer than the build passes the test of the version,
/// and the program goes on to the test of the sum.
#[tokio::test]
async fn a_newer_release_passes_the_version_test() {
    let server = MockServer::start().await;
    let target = toutui::update::release::target().unwrap();
    let name = format!("toutui-{}.tar.gz", target);

    let body = serde_json::json!({
        "tag_name": "v99.0.0",
        "assets": [
            {"name": "SHA256SUMS",
             "browser_download_url": format!("{}/SHA256SUMS", server.uri())},
            {"name": name,
             "browser_download_url": format!("{}/archive", server.uri())}
        ]
    });

    Mock::given(method("GET")).and(path("/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/archive"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(archive_of(b"new")))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/SHA256SUMS"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(format!("{}  {}\n", "0".repeat(64), name)))
        .mount(&server).await;

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();

    // The version test does not stop the update; the sum test does. The
    // error therefore comes from the sum, and not from the version.
    let error = toutui::update::install::run_update_at(&format!("{}/latest", server.uri()), &binary)
        .await
        .unwrap_err();

    assert!(error.contains("not correct"));
}
