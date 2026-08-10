//! The installation of a new binary.
//!
//! The program compares the sum before it moves the binary. Therefore a
//! download that stops leaves the binary that operates.

use crate::update::attest::{self, Attestation};
use crate::update::release::{is_newer, latest_release, target, Release};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;

/// The version of this build.
const LOCAL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The limit on the size of one file that the program receives.
///
/// A host that sends without end must not fill the memory of the computer.
const MAX_DOWNLOAD_BYTES: u64 = 200 * 1024 * 1024;

/// The limit on the time that the program waits for one address.
///
/// A host that does not answer must not stop the update forever with no
/// message.
const DOWNLOAD_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Gives the sum SHA-256 of the bytes, in hexadecimal.
pub fn sum_of(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

/// Finds the sum of one name in the file of the sums.
///
/// Each line of the file has the form `<sum>  <name>`, or `<sum> *<name>`
/// when a tool such as `sha256sum -b` makes the file.
pub fn expected_sum(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let sum = parts.next()?;
        let file = parts.next()?.trim_start_matches('*');
        if file == name {
            Some(sum.to_string())
        } else {
            None
        }
    })
}

/// Takes the binary out of a `tar.gz`.
///
/// The decoder reads every member of the file, and not the first member
/// only, because a `tar.gz` can hold more than one member.
pub fn binary_from_archive(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let gz = flate2::read::MultiGzDecoder::new(bytes);
    let mut archive = tar::Archive::new(gz);

    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        let is_named_toutui = path.file_name().and_then(|name| name.to_str()) == Some("toutui");

        // A symlink, a hard link, or a directory can have the name `toutui`
        // and the size 0. Only a plain file holds the bytes of the binary.
        if entry.header().entry_type().is_file() && is_named_toutui {
            let mut contents = Vec::new();
            entry
                .read_to_end(&mut contents)
                .map_err(|e| e.to_string())?;
            if contents.is_empty() {
                return Err("The file toutui in the archive holds no bytes.".to_string());
            }
            return Ok(contents);
        }
    }

    Err("The archive holds no file with the name toutui.".to_string())
}

/// Gives `true` if the program can write in the directory of the binary.
///
/// A move needs permission on the directory and not on the file. Therefore
/// the test makes a file in that directory.
pub fn can_replace(binary: &Path) -> bool {
    let Some(dir) = binary.parent() else {
        return false;
    };
    tempfile::Builder::new()
        .prefix(".toutui-")
        .tempfile_in(dir)
        .is_ok()
}

/// Gives the command that runs the update with more rights, with the path in
/// quotation marks, so that a path with a space stays one argument.
fn sudo_command(binary: &Path) -> String {
    format!("sudo '{}' --update", binary.display())
}

/// Gives the mode that the new binary must have.
///
/// The new binary keeps the mode of the binary that is present, so that a
/// binary that was private to one user stays private. The program uses
/// 0o755 only when no binary is present yet.
#[cfg(unix)]
fn mode_for(binary: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(binary)
        .map(|meta| meta.permissions().mode())
        .unwrap_or(0o755)
}

/// Moves the new binary on to the old binary with one operation.
///
/// The temporary file is in the directory of the binary, because a move
/// between two file systems is not one operation. The program sends the
/// bytes and the mode to the disk before it moves the file, so that a loss
/// of power right after the move cannot give a name with no data.
pub fn replace_binary(binary: &Path, contents: &[u8]) -> std::io::Result<()> {
    let dir = binary.parent().unwrap_or(Path::new("."));
    let mut temp = tempfile::Builder::new()
        .prefix(".toutui-new-")
        .tempfile_in(dir)?;

    temp.write_all(contents)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temp.as_file()
            .set_permissions(std::fs::Permissions::from_mode(mode_for(binary)))?;
    }

    // `flush` on this file does nothing, because the write goes straight to
    // the file with no buffer in the program. `sync_all` is what sends the
    // bytes and the mode to the disk.
    temp.as_file().sync_all()?;

    temp.persist(binary).map_err(|e| e.error)?;
    Ok(())
}

/// Receives one file from an address, with a limit on the size.
///
/// The header `Content-Length` is the first test, because a host that gives a
/// correct header saves the whole download. That header is not enough: a host
/// can send no header, or a header that is not true. Therefore the program
/// counts the bytes as they arrive and stops at the limit. See T-30.
///
/// The limit is an argument, so that a test can use a small number.
pub async fn receive_at_most(url: &str, limit: u64) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())?;

    let mut response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "Toutui-Updater")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("The address {} gives {}.", url, response.status()));
    }

    if let Some(length) = response.content_length() {
        if length > limit {
            return Err(format!(
                "The address {} gives a file of {} bytes. The limit is {} bytes.",
                url, length, limit
            ));
        }
    }

    let mut body: Vec<u8> = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if body.len() as u64 + chunk.len() as u64 > limit {
            return Err(format!(
                "The address {} sends more than the limit of {} bytes. \
                 The program stopped the download.",
                url, limit
            ));
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

/// Receives one file from an address, with the limit of the program.
///
/// The request has a limit on time and a limit on size, so that a host that
/// does not answer, or a host that sends without end, cannot stop the
/// update with no message or fill the memory of the computer.
async fn receive(url: &str) -> Result<Vec<u8>, String> {
    receive_at_most(url, MAX_DOWNLOAD_BYTES).await
}

/// Does the full update on the binary at the given path, and gives the
/// message that the user reads.
///
/// A caller that runs as the program itself gives its own path, from
/// `std::env::current_exe`. A test gives a path in a directory that the
/// test made, so that the test cannot write over the binary of the test.
pub async fn run_update_at(api: &str, binary: &Path) -> Result<String, String> {
    run_update_at_with(api, binary, attest::GH).await
}

/// Does the full update, and asks the given command for the proof of the
/// origin.
///
/// `gh` is the name of that command. A test gives the path of a command that
/// it made itself, therefore no test asks GitHub.
pub async fn run_update_at_with(api: &str, binary: &Path, gh: &str) -> Result<String, String> {
    let target = target().ok_or_else(|| {
        "This system has no archive. Use `cargo install --git https://github.com/ealtun21/Toutui`."
            .to_string()
    })?;

    let release: Release = latest_release(api, target).await?;

    // The same rule as `check_update`, in one place: `is_newer` in
    // `update::release`. A build newer than the last release, for example
    // one from `cargo install --git`, must read the same answer here that it
    // read before it ran this command.
    if !is_newer(&release.version, LOCAL_VERSION) {
        return Ok(format!(
            "Version {} is installed. The release gives {}, and that version is not newer. The program did not change.",
            LOCAL_VERSION, release.version
        ));
    }

    if !can_replace(binary) {
        return Err(format!(
            "The program cannot write in {}. Run this command:\n    {}",
            binary.parent().unwrap_or(Path::new(".")).display(),
            sudo_command(binary)
        ));
    }

    let archive = receive(&release.archive_url).await?;
    let sums = receive(&release.sums_url).await?;
    let sums = String::from_utf8_lossy(&sums);

    let expected = expected_sum(&sums, &release.archive_name)
        .ok_or_else(|| format!("SHA256SUMS has no sum for {}.", release.archive_name))?;

    let actual = sum_of(&archive);
    if actual != expected {
        return Err(format!(
            "The sum of the archive is not correct. The program did not change.\n\
             expected: {}\n\
             actual:   {}",
            expected, actual
        ));
    }

    // The sum agrees. The sum comes from the same release, therefore it does
    // not tell who made the release. The proof of the origin tells that. See
    // T-29.
    let attestation = attest::verify_bytes_with(gh, &archive, &release.archive_name);
    if let Attestation::Refused(_) = attestation {
        return Err(attest::message_of(&attestation));
    }

    let new_binary = binary_from_archive(&archive)?;

    if let Err(e) = replace_binary(binary, &new_binary) {
        // The permission to write can go away between the probe and the
        // move. Give the command with `sudo` again when that happens.
        if !can_replace(binary) {
            return Err(format!(
                "The program cannot write in {}. Run this command:\n    {}",
                binary.parent().unwrap_or(Path::new(".")).display(),
                sudo_command(binary)
            ));
        }
        return Err(e.to_string());
    }

    Ok(format!(
        "{}\nVersion {} is now installed. The version before it was {}.",
        attest::message_of(&attestation),
        release.version,
        LOCAL_VERSION
    ))
}

/// Does the full update on this program's own binary, and gives the message
/// that the user reads.
pub async fn run_update(api: &str) -> Result<String, String> {
    let binary = std::env::current_exe().map_err(|e| e.to_string())?;
    run_update_at(api, &binary).await
}
