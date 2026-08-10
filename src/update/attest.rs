//! The proof of the origin of an archive.
//!
//! The workflow `release.yml` runs `actions/attest-build-provenance`.
//! Therefore each archive of a release has a proof, and that proof names the
//! repository and the workflow that made the archive.
//!
//! The sum SHA-256 is not enough. That sum comes from `SHA256SUMS` of the same
//! release. Therefore the sum finds a download that stops, and it does not
//! find a release that a different person made. The proof finds that release.
//! See T-29.
//!
//! The program does not read the proof itself. It gives the work to
//! `gh attestation verify`, because that command holds the keys and the log of
//! the transparency. The program tells the user what it tested when `gh` is
//! not on the system.

use std::path::Path;
use std::process::Output;

/// The repository that makes the releases of the fork.
pub const REPO: &str = "ealtun21/Toutui";

/// The workflow that has permission to make an archive of a release.
///
/// `--signer-workflow` refuses a proof that a different workflow made, and
/// `--repo` alone accepts every workflow of the repository.
pub const SIGNER_WORKFLOW: &str = "ealtun21/Toutui/.github/workflows/release.yml";

/// What the program knows about the origin of an archive.
#[derive(Debug, PartialEq, Eq)]
pub enum Attestation {
    /// `gh` confirmed the proof. The workflow of the repository made the
    /// archive.
    Confirmed,
    /// The program could not test the proof. The reason is for the user.
    /// The update goes on, because most users have no `gh`.
    NotTested(String),
    /// `gh` read the proof and refused the archive. The update must stop.
    Refused(String),
}

/// Gives the reason when the answer of `gh` means "the program cannot test",
/// and not "the archive is not correct".
///
/// A refusal names the proof: no attestation, or a verification that fails.
/// These conditions name the tool, the account, or the network instead. The
/// program must not stop an update for them, because the archive can be
/// correct.
fn reason_it_cannot_test(output: &str) -> Option<String> {
    let lower = output.to_lowercase();

    let tool = [
        "unknown flag",
        "unknown command",
        "unknown shorthand",
        "unknown subcommand",
        "accepts at most",
    ];
    let account = [
        "gh auth login",
        "authentication",
        "not logged in",
        "no such remote",
        "http 401",
        "http 403",
        "requires authentication",
        "bad credentials",
    ];
    let network = [
        "no such host",
        "dial tcp",
        "connection refused",
        "connection reset",
        "i/o timeout",
        "network is unreachable",
        "temporary failure in name resolution",
    ];

    if tool.iter().any(|word| lower.contains(word)) {
        return Some("the command `gh` on this system is too old for this test".to_string());
    }
    if account.iter().any(|word| lower.contains(word)) {
        return Some("the command `gh` has no account. Run `gh auth login`".to_string());
    }
    if network.iter().any(|word| lower.contains(word)) {
        return Some("the command `gh` did not reach GitHub".to_string());
    }

    None
}

/// Reads the answer of `gh` and gives what the program knows.
pub fn classify(succeeded: bool, output: &str) -> Attestation {
    if succeeded {
        return Attestation::Confirmed;
    }

    match reason_it_cannot_test(output) {
        Some(reason) => Attestation::NotTested(reason),
        None => Attestation::Refused(output.trim().to_string()),
    }
}

/// The message that tells the user what the program tested, and what it did
/// not test.
pub fn message_of(attestation: &Attestation) -> String {
    match attestation {
        Attestation::Confirmed => format!(
            "The proof of the origin is correct. The workflow of {} made this archive.",
            REPO
        ),
        Attestation::NotTested(reason) => format!(
            "The program tested the sum SHA-256 of the archive. It did not test the proof of \
             the origin, because {}. The sum comes from the same release. Therefore the sum \
             finds a download that stops, and it does not find a release that a different \
             person made. Install `gh` from https://cli.github.com to test the proof.",
            reason
        ),
        Attestation::Refused(output) => format!(
            "The proof of the origin of this archive is not correct. The program did not \
             change.\n{}",
            output
        ),
    }
}

/// The name of the command that reads a proof.
pub const GH: &str = "gh";

/// Asks `gh` for the proof of one file on the disk.
fn ask_gh(program: &str, archive: &Path) -> Result<Output, std::io::Error> {
    std::process::Command::new(program)
        .arg("attestation")
        .arg("verify")
        .arg(archive)
        .arg("--repo")
        .arg(REPO)
        .arg("--signer-workflow")
        .arg(SIGNER_WORKFLOW)
        .output()
}

/// Tests the proof of the origin of one file on the disk.
///
/// `program` is the name of the command. A test gives the path of a command
/// that it made itself, therefore no test asks GitHub.
pub fn verify_file_with(program: &str, archive: &Path) -> Attestation {
    match ask_gh(program, archive) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Attestation::NotTested("the command `gh` is not on this system".to_string())
        }
        Err(error) => Attestation::NotTested(format!("the command `gh` did not run: {}", error)),
        Ok(output) => {
            let mut text = String::from_utf8_lossy(&output.stderr).to_string();
            text.push_str(&String::from_utf8_lossy(&output.stdout));
            classify(output.status.success(), &text)
        }
    }
}

/// Tests the proof of the origin of an archive that is in the memory.
///
/// `gh` reads a file, therefore the program writes the bytes to a temporary
/// file first. That file is not the binary that the program installs: the
/// program still takes the binary out of the bytes in the memory.
pub fn verify_bytes_with(program: &str, archive: &[u8], name: &str) -> Attestation {
    let dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(error) => {
            return Attestation::NotTested(format!(
                "the program could not make a temporary directory: {}",
                error
            ))
        }
    };

    let path = dir.path().join(name);
    if let Err(error) = std::fs::write(&path, archive) {
        return Attestation::NotTested(format!(
            "the program could not write the temporary file: {}",
            error
        ));
    }

    verify_file_with(program, &path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_that_succeeds_gives_confirmed() {
        assert_eq!(classify(true, ""), Attestation::Confirmed);
    }

    #[test]
    fn an_archive_with_no_proof_gives_refused() {
        let output = "✗ No attestations found for subject";
        assert!(matches!(classify(false, output), Attestation::Refused(_)));
    }

    #[test]
    fn a_proof_of_a_different_repository_gives_refused() {
        let output = "✗ Verification failed: the certificate names a different repository";
        assert!(matches!(classify(false, output), Attestation::Refused(_)));
    }

    /// This is the answer of gh 2.96.0 for an archive that no workflow
    /// attested. A measurement on 2026-08-10 gives this text. The status 404
    /// means "no proof for this sum", and that is a refusal.
    #[test]
    fn the_real_answer_for_an_archive_with_no_proof_gives_refused() {
        let output = "Error: HTTP 404: Not Found (https://api.github.com/repos/ealtun21/\
                      Toutui/attestations/sha256:68ff09fd105afb850000b6db0b2b121a9649f5a131a\
                      5d1c81aa95e920603e1c7?per_page=30&predicate_type=https%3A%2F%2Fslsa.dev\
                      %2Fprovenance%2Fv1)";

        assert!(matches!(classify(false, output), Attestation::Refused(_)));
    }

    #[test]
    fn a_gh_that_is_too_old_gives_not_tested() {
        let output = "unknown flag: --signer-workflow";
        assert!(matches!(classify(false, output), Attestation::NotTested(_)));
    }

    #[test]
    fn a_gh_with_no_account_gives_not_tested() {
        let output = "gh: To use GitHub CLI in a GitHub Actions workflow, run: gh auth login";
        assert!(matches!(classify(false, output), Attestation::NotTested(_)));
    }

    #[test]
    fn a_network_that_fails_gives_not_tested() {
        let output = "dial tcp: lookup api.github.com: no such host";
        assert!(matches!(classify(false, output), Attestation::NotTested(_)));
    }

    #[test]
    fn the_message_of_not_tested_names_the_sum_and_the_proof() {
        let message = message_of(&Attestation::NotTested("gh is absent".to_string()));

        assert!(message.contains("tested the sum SHA-256"));
        assert!(message.contains("did not test the proof"));
        assert!(message.contains("gh is absent"));
    }

    #[test]
    fn the_message_of_confirmed_names_the_repository() {
        assert!(message_of(&Attestation::Confirmed).contains(REPO));
    }
}
