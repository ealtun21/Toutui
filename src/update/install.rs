//! The installation of a new binary.
//!
//! The program compares the sum before it moves the binary. Therefore a
//! download that stops leaves the binary that operates.

use crate::update::release::{latest_release, target, Release};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;

/// The version of this build.
const LOCAL_VERSION: &str = env!("CARGO_PKG_VERSION");

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
/// Each line of the file has the form `<sum>  <name>`.
pub fn expected_sum(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let sum = parts.next()?;
        let file = parts.next()?;
        if file == name {
            Some(sum.to_string())
        } else {
            None
        }
    })
}

/// Takes the binary out of a `tar.gz`.
pub fn binary_from_archive(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let gz = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(gz);

    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        if path.file_name().and_then(|name| name.to_str()) == Some("toutui") {
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents).map_err(|e| e.to_string())?;
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

/// Moves the new binary on to the old binary with one operation.
///
/// The temporary file is in the directory of the binary, because a move
/// between two file systems is not one operation.
pub fn replace_binary(binary: &Path, contents: &[u8]) -> std::io::Result<()> {
    let dir = binary.parent().unwrap_or(Path::new("."));
    let mut temp = tempfile::Builder::new()
        .prefix(".toutui-new-")
        .tempfile_in(dir)?;

    temp.write_all(contents)?;
    temp.flush()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temp.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o755))?;
    }

    temp.persist(binary).map_err(|e| e.error)?;
    Ok(())
}

/// Receives one file from an address.
async fn receive(url: &str) -> Result<Vec<u8>, String> {
    let response = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, "Toutui-Updater")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("The address {} gives {}.", url, response.status()));
    }

    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|e| e.to_string())
}

/// Does the full update, and gives the message that the user reads.
pub async fn run_update(api: &str) -> Result<String, String> {
    let target = target().ok_or_else(|| {
        "This system has no archive. Use `cargo install --git https://github.com/ealtun21/Toutui`."
            .to_string()
    })?;

    let release: Release = latest_release(api, target).await?;

    if release.version == LOCAL_VERSION {
        return Ok(format!("Version {} is the last version.", LOCAL_VERSION));
    }

    let binary = std::env::current_exe().map_err(|e| e.to_string())?;

    if !can_replace(&binary) {
        return Err(format!(
            "The program cannot write in {}. Run this command:\n    sudo {} --update",
            binary.parent().unwrap_or(Path::new(".")).display(),
            binary.display()
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

    let new_binary = binary_from_archive(&archive)?;
    replace_binary(&binary, &new_binary).map_err(|e| e.to_string())?;

    Ok(format!(
        "Version {} is now installed. The version before it was {}.",
        release.version, LOCAL_VERSION
    ))
}
