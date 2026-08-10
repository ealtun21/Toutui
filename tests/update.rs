//! Tests of the update in the program.
//!
//! The tests give the answers of the API from a mock server. No test uses the
//! network.

use std::io::Write;
use toutui::update::install::{
    binary_from_archive, expected_sum, receive_at_most, replace_binary, sum_of,
};
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

/// A tag with a capital letter gives the version with no letter. See T-28.
///
/// The program removed the letter with a comparison that looks at the case.
/// Therefore a tag `V0.6.0` kept its letter, and every comparison with the
/// version of the build disagreed.
#[test]
fn a_tag_with_a_capital_letter_gives_the_version() {
    let release = parse_release(&answer("V0.6.0"), "x86_64-unknown-linux-gnu").unwrap();

    assert_eq!(release.version, "0.6.0");
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
    assert_eq!(
        expected_sum(sums, "toutui-aarch64-unknown-linux-gnu.tar.gz"),
        None
    );
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

    Mock::given(method("GET"))
        .and(path("/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/archive"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(archive_of(b"new")))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/SHA256SUMS"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            "{}  {}\n",
            "0".repeat(64),
            name
        )))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();

    let error =
        toutui::update::install::run_update_at(&format!("{}/latest", server.uri()), &binary)
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

    Mock::given(method("GET"))
        .and(path("/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();

    let message =
        toutui::update::install::run_update_at(&format!("{}/latest", server.uri()), &binary)
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

    Mock::given(method("GET"))
        .and(path("/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/archive"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(archive_of(b"new")))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/SHA256SUMS"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            "{}  {}\n",
            "0".repeat(64),
            name
        )))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();

    // The version test does not stop the update; the sum test does. The
    // error therefore comes from the sum, and not from the version.
    let error =
        toutui::update::install::run_update_at(&format!("{}/latest", server.uri()), &binary)
            .await
            .unwrap_err();

    assert!(error.contains("not correct"));
}

/// Starts a host that answers with `Transfer-Encoding: chunked` and never
/// stops.
///
/// The answer has no header `Content-Length`. Therefore the test of that
/// header cannot stop this host, and only a count of the bytes can. The test
/// needs a raw socket, because a mock server always gives a length. See T-30.
async fn a_host_that_sends_without_end() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                // Read the request, and stop at the empty line.
                let mut request = Vec::new();
                let mut byte = [0u8; 1];
                while socket.read(&mut byte).await.unwrap_or(0) == 1 {
                    request.push(byte[0]);
                    if request.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }

                let head = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n";
                if socket.write_all(head).await.is_err() {
                    return;
                }

                // One chunk of 4096 bytes, again and again, and no last
                // chunk. A program with no limit reads until the memory or
                // the time runs out.
                let chunk = [b'x'; 4096];
                loop {
                    if socket.write_all(b"1000\r\n").await.is_err()
                        || socket.write_all(&chunk).await.is_err()
                        || socket.write_all(b"\r\n").await.is_err()
                    {
                        return;
                    }
                }
            });
        }
    });

    address
}

/// An answer with no length stops at the limit. This is T-30.
///
/// The program read the whole body into the memory when the answer had no
/// header `Content-Length`. Only the limit of 120 seconds stopped it.
#[tokio::test]
async fn an_answer_with_no_length_stops_at_the_limit() {
    let address = a_host_that_sends_without_end().await;

    let error = receive_at_most(&address, 64 * 1024).await.unwrap_err();

    assert!(error.contains("more than the limit"), "{}", error);
}

/// A header that gives a size above the limit stops the download before the
/// bytes arrive.
#[tokio::test]
async fn a_length_above_the_limit_stops_the_download() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/big"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'x'; 4096]))
        .mount(&server)
        .await;

    let error = receive_at_most(&format!("{}/big", server.uri()), 1024)
        .await
        .unwrap_err();

    assert!(error.contains("The limit is 1024 bytes"), "{}", error);
}

/// A file that is inside the limit arrives whole.
#[tokio::test]
async fn a_file_inside_the_limit_arrives_whole() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/small"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'x'; 4096]))
        .mount(&server)
        .await;

    let body = receive_at_most(&format!("{}/small", server.uri()), 64 * 1024)
        .await
        .unwrap();

    assert_eq!(body, vec![b'x'; 4096]);
}

/// Makes a command that answers in place of `gh`.
///
/// The command writes its arguments to `args.txt`, it writes `output` to the
/// error stream, and it ends with `code`. Therefore a test can read what the
/// program asked, and it can choose the answer. See T-29.
fn fake_gh(dir: &std::path::Path, code: i32, output: &str) -> std::path::PathBuf {
    let path = dir.join("gh");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}/args.txt'\nprintf '%s' '{}' >&2\nexit {}\n",
        dir.display(),
        output,
        code
    );
    std::fs::write(&path, script).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    path
}

/// Starts a host that gives a release whose sum agrees with its archive.
///
/// The tests of the proof of the origin need an archive that passes the test
/// of the sum, because the test of the proof comes after it.
async fn a_release_that_has_a_correct_sum() -> (MockServer, String) {
    let server = MockServer::start().await;
    let target = toutui::update::release::target().unwrap();
    let name = format!("toutui-{}.tar.gz", target);
    let archive = archive_of(b"the new binary");

    let body = serde_json::json!({
        "tag_name": "v99.0.0",
        "assets": [
            {"name": "SHA256SUMS",
             "browser_download_url": format!("{}/SHA256SUMS", server.uri())},
            {"name": name.clone(),
             "browser_download_url": format!("{}/archive", server.uri())}
        ]
    });

    Mock::given(method("GET"))
        .and(path("/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/archive"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(archive.clone()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/SHA256SUMS"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            "{}  {}\n",
            sum_of(&archive),
            name
        )))
        .mount(&server)
        .await;

    let api = format!("{}/latest", server.uri());
    (server, api)
}

/// A proof that `gh` confirms lets the update finish, and the program asks
/// for the repository and for the workflow of the release.
#[tokio::test]
async fn a_proof_that_is_correct_lets_the_update_finish() {
    let (_server, api) = a_release_that_has_a_correct_sum().await;
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();
    let gh = fake_gh(dir.path(), 0, "");

    let message = toutui::update::install::run_update_at_with(&api, &binary, gh.to_str().unwrap())
        .await
        .unwrap();

    assert!(
        message.contains("The proof of the origin is correct"),
        "{}",
        message
    );
    assert_eq!(std::fs::read(&binary).unwrap(), b"the new binary");

    let args = std::fs::read_to_string(dir.path().join("args.txt")).unwrap();
    assert!(args.contains("attestation"), "{}", args);
    assert!(args.contains("verify"), "{}", args);
    assert!(args.contains("ealtun21/Toutui"), "{}", args);
    assert!(args.contains("--signer-workflow"), "{}", args);
    assert!(
        args.contains("ealtun21/Toutui/.github/workflows/release.yml"),
        "{}",
        args
    );
}

/// An archive that has no proof stops the update, and the binary does not
/// change. This is the fault of T-29: the sum agrees here, and the sum alone
/// let this archive pass.
#[tokio::test]
async fn an_archive_with_no_proof_stops_the_update() {
    let (_server, api) = a_release_that_has_a_correct_sum().await;
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();
    let gh = fake_gh(dir.path(), 1, "No attestations found for subject");

    let error = toutui::update::install::run_update_at_with(&api, &binary, gh.to_str().unwrap())
        .await
        .unwrap_err();

    assert!(error.contains("proof of the origin"), "{}", error);
    assert_eq!(std::fs::read(&binary).unwrap(), b"the old binary");
}

/// A system that has no `gh` gets the update, and the message tells the user
/// what the program tested and what it did not test.
#[tokio::test]
async fn a_system_with_no_gh_gets_the_update_and_a_clear_message() {
    let (_server, api) = a_release_that_has_a_correct_sum().await;
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();
    let absent = dir.path().join("no-gh-is-here");

    let message =
        toutui::update::install::run_update_at_with(&api, &binary, absent.to_str().unwrap())
            .await
            .unwrap();

    assert!(message.contains("did not test the proof"), "{}", message);
    assert!(message.contains("tested the sum SHA-256"), "{}", message);
    assert_eq!(std::fs::read(&binary).unwrap(), b"the new binary");
}

/// A `gh` that has no account does not stop the update, because the archive
/// can be correct. The message tells the user to run `gh auth login`.
#[tokio::test]
async fn a_gh_with_no_account_does_not_stop_the_update() {
    let (_server, api) = a_release_that_has_a_correct_sum().await;
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();
    let gh = fake_gh(
        dir.path(),
        4,
        "gh: you are not logged in. Run gh auth login",
    );

    let message = toutui::update::install::run_update_at_with(&api, &binary, gh.to_str().unwrap())
        .await
        .unwrap();

    assert!(message.contains("gh auth login"), "{}", message);
    assert_eq!(std::fs::read(&binary).unwrap(), b"the new binary");
}

/// The update changes the binary, and it changes nothing else.
///
/// This is the guard of T-14. The script of the original project merged
/// `config.example.toml` into the configuration of the user at every
/// installation, and that merge lost the options of the user. The update of
/// the fork moves one file: the binary.
#[tokio::test]
async fn the_update_does_not_touch_the_configuration() {
    let (_server, api) = a_release_that_has_a_correct_sum().await;
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();

    // The configuration of a user: a colour that the user chose, and an
    // option that `config.example.toml` does not name.
    let config_dir = dir.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config = config_dir.join("config.toml");
    let contents = "[colors]\naccent = \"magenta\"\nmy_own_option = \"keep me\"\n";
    std::fs::write(&config, contents).unwrap();
    let env = config_dir.join(".env");
    let key = "TOUTUI_SECRET_KEY=0123456789abcdef\n";
    std::fs::write(&env, key).unwrap();

    let gh = fake_gh(dir.path(), 0, "");
    toutui::update::install::run_update_at_with(&api, &binary, gh.to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(std::fs::read(&binary).unwrap(), b"the new binary");
    assert_eq!(std::fs::read_to_string(&config).unwrap(), contents);
    assert_eq!(std::fs::read_to_string(&env).unwrap(), key);
}
