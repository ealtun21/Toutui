# Fork Release Process, Revised Implementation Plan (T-21)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the fork its own way to install, to update, and to remove, under the name it already has, so that `--update` stops being a command that installs the archived original project.

**Architecture:** The project keeps the name `toutui`. A workflow makes the releases from a tag on `main`, and it writes `SHA256SUMS` with the machine. The program updates itself: it receives the archive of its own target, it compares the sum, and it moves the new binary on to the old binary. No script runs, and no registry holds the program.

**Tech Stack:** Rust 1.97, `reqwest` with rustls, `sha2`, `flate2` with the Rust backend, `tar`, GitHub Actions, `cargo-zigbuild`.

**Spec:** `docs/superpowers/specs/2026-08-10-fork-release-process-design.md`, and above all its section **Revision, 2026-08-10: the project keeps its name**.

## Why this plan replaces an earlier one

`docs/superpowers/plans/2026-08-10-fork-release-process.md` changed the name of
the project to `abstui`, it made the program copy the old directory of
configuration, and it published the program to crates.io and to the AUR. Five
of its tasks are in the history of the branch `release-process`.

The maintainer then changed three decisions. The name stays `toutui`, the
program copies no configuration, and the work uses no registry. Tasks R1 and R2
of this plan remove the work that those decisions made unnecessary. The other
tasks continue the work that stays correct.

**Do not use the earlier plan.** This plan governs.

## Global Constraints

- Every document, doc comment, and user-facing string uses ASD-STE100 Simplified Technical English: short sentences, active voice, one instruction in one sentence.
- Before each commit, `cargo clippy --all-targets -- -D warnings` must give no output, and `cargo test` must pass. Give `ALSA_CONFIG_PATH=/dev/null` to `cargo test` so that no test opens a sound card.
- Add no dependency that needs a C toolchain or a program that the user installs. `cargo tree -i openssl-sys` must find nothing.
- The name of the project is `toutui`. The name of the binary is `toutui`. The directory of configuration is `~/.config/toutui/`. The name of the secret key is `TOUTUI_SECRET_KEY`.
- The version of the first release of the fork is `0.5.0-beta`.
- The work publishes nothing to crates.io and nothing to the AUR.
- The three ways to install are the script, `cargo install --git`, and the releases of GitHub. The README gives these three and no other.
- AlbanDAVID wrote the original program. The README, the LICENSE, and the screen of settings must name him.
- Commit changes that are separate in separate commits.
- No subagent runs `git push`, `git tag`, or any `gh release` command. Those steps belong to the maintainer.
- This work closes issue #21 (T-21). It does **not** close issue #14 (T-14): the earlier plan closed T-14 only because a copy of the directory made a change of the directory safe, and there is no copy now.

## File Structure

| File | Responsibility |
|---|---|
| `src/paths.rs` | Every path of the program. Stays, with `APP_DIR = "toutui"` |
| `src/utils/encrypt_token.rs` | Reads `TOUTUI_SECRET_KEY` |
| `src/utils/check_update.rs` | Asks the API of the fork |
| `src/utils/clap.rs` | `--update` runs the update. `--uninstall` writes the paths |
| `src/update/mod.rs` (create) | Declares the two modules below |
| `src/update/release.rs` (create) | Asks the API, and finds the archive of this target |
| `src/update/install.rs` (create) | Compares the sum, opens the archive, moves the binary |
| `tests/update.rs` (create) | The plan of the release and the installation |
| `.github/workflows/release.yml` | Makes the release from a tag |
| `install.sh` (create) | The short script of installation |
| `hello_toutui.sh` (delete) | The long script of the original project |
| `README.md` | The three ways to install |

---

### Task R1: Give the project its name again

Five commits on this branch changed the name to `abstui`. This task changes it
back. Four corrections that those commits also made are **not** about the name,
and they must stay.

**Files:**
- Modify: `Cargo.toml`, `Cargo.lock`
- Modify: `src/main.rs`, and every file under `src/` and `tests/` that holds the name
- Modify: `flake.nix`
- Create: `linux/toutui.desktop`. Delete: `linux/abstui.desktop`
- Modify: `macos/launch.command`, `macos/Info.plist`
- Modify: `README.md`, `LICENSE`
- Modify: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: nothing.
- Produces: the crate `toutui`, the binary `toutui`, and the assets `toutui-<target>.tar.gz`. Every later task uses these names.

- [ ] **Step 1: Read what the branch changed**

```bash
git log --oneline 4616f71..HEAD
git show 538a27f --stat
git show 6c84bf4 --stat
```

Commit `538a27f` made the change of the name. Commit `6c84bf4` corrected five
things that `538a27f` missed. Read both before you touch a file.

- [ ] **Step 2: Change the metadata of the package**

In `Cargo.toml`, the block `[package]` becomes:

```toml
[package]
name = "toutui"
version = "0.5.0-beta"
edition = "2021"
description = "A TUI client of Audiobookshelf for Linux and macOS."
license = "GPL-3.0-or-later"
repository = "https://github.com/ealtun21/Toutui"
readme = "README.md"
authors = [
    "AlbanDAVID <https://github.com/AlbanDAVID>",
    "ealtun21 <https://github.com/ealtun21>",
]
```

The name and the address change. The version `0.5.0-beta` stays, because the
fork has more corrections than `0.4.2-beta`. The fields `keywords` and
`categories` go away, because only crates.io reads them and this work uses no
registry.

- [ ] **Step 3: Change the name in the code**

```bash
grep -rl "abstui\|AbsTui\|ABSTUI" src/ tests/ | xargs sed -i 's/abstui/toutui/g; s/AbsTui/Toutui/g; s/ABSTUI/TOUTUI/g'
```

Then run `cargo build` and confirm that the binary is `target/debug/toutui`.

- [ ] **Step 4: Keep the four corrections that are not about the name**

These four came from commit `6c84bf4`, and each one corrects a real fault.
Confirm after step 3 that each is still correct:

1. `src/utils/changelog.rs` — the entry of 15/05/2025 must hold the literal
   text `v0.4.2-beta`. It must **not** use `format!` with `CARGO_PKG_VERSION`,
   because the version of the build then relabels an entry of the past. If the
   constant `VERSION` has no use in the file, it must not be there.
2. `src/ui/tui.rs` — the line of the screen of settings must give
   `https://github.com/ealtun21/Toutui/issues` as the contact and
   `https://github.com/ealtun21/Toutui` as the source. It must not give the
   personal address of mail of AlbanDAVID, and it must not name the archived
   repository as the source of this program.
3. `macos/Info.plist` — the identity of the bundle must be
   `com.github.ealtun21.toutui`, and not `com.example.toutui`.
   `macos/launch.command` must name `$HOME/.cargo/bin/toutui`.
4. `.gitignore` — the line `/src/toutui.log` must not come back. The log is
   in the directory of configuration.

- [ ] **Step 5: Change the file of the launcher**

```bash
git mv linux/abstui.desktop linux/toutui.desktop
```

`linux/toutui.desktop` becomes:

```ini
[Desktop Entry]
Name=Toutui
GenericName=Audiobookshelf client
Exec=toutui
Icon=utilities-terminal
Type=Application
Categories=Utility;
Terminal=true
```

- [ ] **Step 6: Change the workflow of release**

In `.github/workflows/release.yml`, change every `abstui` to `toutui`. The
assets then have the names `toutui-<target>.tar.gz` and the file of the
launcher is `linux/toutui.desktop`.

Verify:

```bash
grep -n "abstui" .github/workflows/release.yml
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"
ls config.example.toml linux/toutui.desktop
```

Expected: the first command gives no result, the second gives no output, and
the third finds both files.

- [ ] **Step 7: Change the flake**

In `flake.nix`, change `pname`, `mainProgram`, the name of the binding, the
attribute of the package, the description, and the message of the shell to
`toutui`. The homepage becomes `https://github.com/ealtun21/Toutui`.

Verify: `grep -in "abstui" flake.nix` gives no result.

The command `nix flake check` does not run on this machine, because the store
of Nix is absent. Read the file instead. The set `packages` must not have the
word `rec`, because `toutui = toutui;` in a set with `rec` is an endless
recursion.

- [ ] **Step 8: Change the README and the LICENSE**

In `README.md`, change `abstui` to `toutui` and `AbsTui` to `Toutui`. The
first block then says that this is the maintained fork of the archived
project, and not that the project has a new name. Write it so:

````markdown
## 🍴 This is a maintained fork

AlbanDAVID wrote [Toutui](https://github.com/AlbanDAVID/Toutui) and archived
it. This repository continues that work with the same name.

The fork corrects the faults of the original project, and it adds functions.
`docs/TAKEOVER-BACKLOG.md` holds the full list. Report a fault in the
[issues of this repository](https://github.com/ealtun21/Toutui/issues), and
not in the archived repository.
````

Keep every address that names `AlbanDAVID/Toutui`, because those addresses
name the archived original project.

In `LICENSE`, the two lines of copyright become:

```
    Toutui, Copyright (C) 2025 AlbanDAVID
    Toutui, Copyright (C) 2026 ealtun21
```

- [ ] **Step 9: Confirm that no name of the fork survives**

```bash
grep -rn -i "abstui" . --exclude-dir=target --exclude-dir=.git --exclude-dir=.superpowers --exclude-dir=docs
```

Expected: no result. A result in `docs/` is correct, because the documents of
the design record the decision that the maintainer made and then changed.

- [ ] **Step 10: Run the whole gate**

```bash
cargo clippy --all-targets -- -D warnings
ALSA_CONFIG_PATH=/dev/null cargo test
```

Expected: clippy gives no output. Every test passes.

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "revert: give the project the name toutui again

The name toutui is free on crates.io, and this work uses no registry.
Therefore no registry made a new name necessary.

The commit keeps the four corrections of 6c84bf4 that are not about the
name: the entry of the changelog of 15/05/2025 holds its own version,
the screen of settings gives the addresses of the fork, the bundle of
macOS has a correct identity, and .gitignore has no dead line.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task R2: Remove the copy of the configuration

The directory of configuration does not move now, therefore the program has
nothing to copy. A function that copies a directory to itself makes every
reader of the code unsure.

**Files:**
- Modify: `src/paths.rs`
- Modify: `tests/paths.rs`
- Modify: `src/main.rs`
- Modify: `src/utils/encrypt_token.rs`
- Delete: `tests/secret_key.rs`

**Interfaces:**
- Consumes: `src/paths.rs` from Task R1.
- Produces: `src/paths.rs` with `APP_DIR` and no `OLD_APP_DIR`. `secret_key()` reads one name.

- [ ] **Step 1: Remove the copy from `src/paths.rs`**

Delete `OLD_APP_DIR`, `MIGRATED_FILES`, `old_config_dir_in`,
`migrate_old_config`, `migrate_old_config_here`, and every private function
that only those used, such as the function that rewrites a line of `.env`.

Keep `APP_DIR`, `config_home`, `default_config_home`, `config_dir`,
`config_dir_in`, `config_file`, `env_file`, `db_file`, and `log_file`.

`APP_DIR` must be `"toutui"`.

- [ ] **Step 2: Remove the call from `src/main.rs`**

Delete the block that calls `migrate_old_config_here` and writes its message.
`setup_logs()` then becomes the first call after `clap()`.

`setup_logs` opens a file in the directory of configuration and panics if that
directory is absent. The copy made that directory before. Therefore
`setup_logs` must make the directory itself. In `src/utils/logs.rs`, add this
before the file opens:

```rust
    // The directory must be present before the file opens. The program made
    // this directory in another place before, and that place is gone.
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
```

`setup_logs` gives `Result<(), fern::InitError>`. `fern::InitError` holds a
variant for `std::io::Error`, therefore the operator `?` works. Confirm this
with the compiler. If it does not work, map the error.

- [ ] **Step 3: Remove the tests of the copy**

In `tests/paths.rs`, delete `old_installation` and every test that calls it.
Keep the tests of the paths, and keep the test of `XDG_CONFIG_HOME` that has
three states. Change the name `abstui` to `toutui` in what stays if Task R1
did not change it already.

- [ ] **Step 4: Remove the second name of the secret key**

In `src/utils/encrypt_token.rs`, `secret_key` reads one name:

```rust
/// Gives the secret key that encrypts the token.
pub fn secret_key() -> Result<String, String> {
    env::var("TOUTUI_SECRET_KEY").map_err(|_| {
        error!("{}", NO_KEY);
        NO_KEY.to_string()
    })
}
```

`NO_KEY` names `TOUTUI_SECRET_KEY` and `~/.config/toutui`.

Keep the shape of `encrypt_token` and `decrypt_token` that calls `secret_key`.
That shape is shorter than the two blocks that were there before, and it has
one place that reads the variable.

Then delete `tests/secret_key.rs`:

```bash
git rm tests/secret_key.rs
```

- [ ] **Step 5: Run the whole gate**

```bash
cargo clippy --all-targets -- -D warnings
ALSA_CONFIG_PATH=/dev/null cargo test
```

Expected: clippy gives no output. Every test passes.

Then start the program one time and confirm that it makes the directory of
configuration when that directory is absent:

```bash
HOME=$(mktemp -d) XDG_CONFIG_HOME= timeout 5 cargo run 2>&1 | head -5
```

Expected: the program does not panic on the file of the log.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "revert: remove the copy of the directory of configuration

The directory does not move now, therefore the program has nothing to
copy. setup_logs makes the directory, because the copy made it before.

The program reads TOUTUI_SECRET_KEY only.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task R3: The program asks the API of the fork

**Files:**
- Modify: `src/utils/check_update.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `pub const RELEASES_API: &str`. Task R6 uses it.

- [ ] **Step 1: Change the address**

In `src/utils/check_update.rs`, add this after the line `use reqwest::Client;`:

```rust
/// The address that gives the last release of the fork.
///
/// The program before the fork asked `AlbanDAVID/Toutui`, and that repository
/// is archived. Therefore the program never saw a release of the fork. See
/// T-21.
pub const RELEASES_API: &str = "https://api.github.com/repos/ealtun21/Toutui/releases/latest";
```

Then in `get_latest_release_gh`, the request uses it:

```rust
        .get(RELEASES_API)
        .header(USER_AGENT, "Toutui-Updater")
```

- [ ] **Step 2: Change the message that the user reads**

```rust
                Some(format!(
                    "🔄 Version {} is available. Run `toutui --update`.",
                    latest_version_gh
                ))
```

- [ ] **Step 3: Run the gate and commit**

```bash
cargo clippy --all-targets -- -D warnings
ALSA_CONFIG_PATH=/dev/null cargo test
git add src/utils/check_update.rs
git commit -m "fix: ask the API of the fork for the last release

The program asked the archived repository, therefore it never saw a
release of the fork.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task R4: The program finds the archive of its own target

The program reads the answer of the API and finds the archive for the system
that it runs on. This task holds no download.

**Files:**
- Create: `src/update/mod.rs`, `src/update/release.rs`, `src/update/install.rs`
- Create: `tests/update.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `check_update::RELEASES_API` from Task R3.
- Produces:
  - `pub fn target() -> Option<&'static str>`
  - `pub struct Release { pub version: String, pub archive_name: String, pub archive_url: String, pub sums_url: String }`
  - `pub fn parse_release(body: &str, target: &str) -> Result<Release, String>`
  - `pub async fn latest_release(api: &str, target: &str) -> Result<Release, String>`

- [ ] **Step 1: Write the failing tests**

Create `tests/update.rs`:

```rust
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
```

- [ ] **Step 2: Run the tests to see that they fail**

Run: `cargo test --test update`
Expected: FAIL. The compiler says that `toutui::update` does not exist.

- [ ] **Step 3: Write the modules**

Create `src/update/mod.rs`:

```rust
//! The update of the program.
//!
//! The program receives the archive of its own target, it compares the sum,
//! and it moves the new binary on to the old binary. The program runs no file
//! that it receives. See T-21.

pub mod install;
pub mod release;
```

Create `src/update/release.rs`:

```rust
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
```

Create `src/update/install.rs` with the doc comment only, so that
`src/update/mod.rs` compiles. Task R5 writes the code:

```rust
//! The installation of a new binary.
```

In `src/lib.rs`, add `pub mod update;` after `pub mod ui;`.

- [ ] **Step 4: Run the gate and commit**

```bash
cargo test --test update
cargo clippy --all-targets -- -D warnings
ALSA_CONFIG_PATH=/dev/null cargo test
git add src/update/ src/lib.rs tests/update.rs
git commit -m "feat: find the archive of the target in a release

The program reads the answer of the API of GitHub and finds the
archive for the system that it runs on.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task R5: The program installs the new binary

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/update/install.rs`
- Modify: `tests/update.rs`

**Interfaces:**
- Consumes: `Release`, `target`, `latest_release` from Task R4.
- Produces:
  - `pub fn sum_of(bytes: &[u8]) -> String`
  - `pub fn expected_sum(sums: &str, name: &str) -> Option<String>`
  - `pub fn binary_from_archive(bytes: &[u8]) -> Result<Vec<u8>, String>`
  - `pub fn can_replace(binary: &Path) -> bool`
  - `pub fn replace_binary(binary: &Path, contents: &[u8]) -> std::io::Result<()>`
  - `pub async fn run_update(api: &str) -> Result<String, String>`

- [ ] **Step 1: Add the three dependencies**

In `Cargo.toml`, add to `[dependencies]`:

```toml
# The update compares the sum of the archive and opens the archive. The three
# crates below are pure Rust, therefore the build needs no C toolchain.
sha2 = "0.10"
flate2 = { version = "1", default-features = false, features = ["rust_backend"] }
tar = "0.4"
```

`can_replace` and `replace_binary` need `tempfile`. Move `tempfile = "3"` from
`[dev-dependencies]` to `[dependencies]`, and remove the line from
`[dev-dependencies]`.

Then confirm that no new dependency compiles C:

```bash
cargo tree -i cc | grep -E "sha2|flate2|tar" ; echo "exit=$?"
cargo tree -i openssl-sys
```

Expected: the first command finds nothing. The second says that the package is
not in the tree.

- [ ] **Step 2: Write the failing tests**

Add to `tests/update.rs`:

```rust
use std::io::Write;
use toutui::update::install::{binary_from_archive, expected_sum, replace_binary, sum_of};

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

/// The program moves the new binary on to the old binary, and the new binary
/// can run.
#[test]
fn the_program_replaces_the_binary() {
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();

    replace_binary(&binary, b"the new binary").unwrap();

    assert_eq!(std::fs::read(&binary).unwrap(), b"the new binary");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&binary).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
    }
}

/// A directory that the user cannot write gives `false`.
#[cfg(unix)]
#[test]
fn a_directory_that_is_read_only_gives_false() {
    use std::os::unix::fs::PermissionsExt;
    use toutui::update::install::can_replace;

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("toutui");
    std::fs::write(&binary, b"the old binary").unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();

    let result = can_replace(&binary);

    // The permissions come back, so that the temporary directory goes away.
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();

    assert!(!result);
}
```

- [ ] **Step 3: Run the tests to see that they fail**

Run: `cargo test --test update`
Expected: FAIL. The compiler says that `sum_of` does not exist.

- [ ] **Step 4: Write the module**

Replace the whole of `src/update/install.rs`:

```rust
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
```

- [ ] **Step 5: Add the test of the full update**

Add to `tests/update.rs`:

```rust
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The program stops and does not touch the binary when the sum disagrees.
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

    let error = toutui::update::install::run_update(&format!("{}/latest", server.uri()))
        .await
        .unwrap_err();

    assert!(error.contains("not correct"));
}
```

The test uses the binary of the test as `current_exe`, and it stops before
`replace_binary`, therefore it changes no file.

- [ ] **Step 6: Run the gate and commit**

```bash
cargo test --test update
cargo clippy --all-targets -- -D warnings
ALSA_CONFIG_PATH=/dev/null cargo test
cargo tree -i openssl-sys
git add Cargo.toml Cargo.lock src/update/install.rs tests/update.rs
git commit -m "feat: install a new binary in the program

The program compares the sum of the archive before it moves the new
binary. Therefore a download that stops leaves the binary that
operates. The program runs no file that it receives.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task R6: The commands `--update` and `--uninstall`

**Files:**
- Modify: `src/utils/clap.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `update::install::run_update` from Task R5, `check_update::RELEASES_API` from Task R3, `paths::config_dir` from Task R1.
- Produces: `pub async fn clap()`. `src/main.rs` writes `clap().await;`.

- [ ] **Step 1: Write the module**

Replace the whole of `src/utils/clap.rs`:

```rust
//! The commands of the line of command.
//!
//! `--update` installs the last release of the fork. `--uninstall` writes the
//! list of the paths, and it deletes nothing.
//!
//! The command before this one ran a script of the original project, and
//! every address in that script names the archived repository. Therefore the
//! command removed the fork and installed the original program. See T-21.

use crate::paths;
use crate::update::install::run_update;
use crate::utils::check_update::RELEASES_API;
use clap::{Arg, Command};

/// Writes the paths that a user can delete to remove the program.
///
/// The command deletes nothing. The command before this one used `sudo rm -r`
/// on paths that came from variables of the environment, and that is a
/// danger.
fn write_uninstall_message() {
    println!("The command deletes nothing. Delete these paths to remove toutui:");
    println!();
    println!("    {}", paths::config_dir().display());
    match std::env::current_exe() {
        Ok(binary) => println!("    {}", binary.display()),
        Err(_) => println!("    the binary toutui in your PATH"),
    }
    if cfg!(target_os = "linux") {
        if let Some(home) = dirs::home_dir() {
            println!(
                "    {}",
                home.join(".local/share/applications/toutui.desktop").display()
            );
            println!("    {}", home.join(".local/share/toutui").display());
        }
    }
}

pub async fn clap() {
    let matches = Command::new("toutui")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A TUI client of Audiobookshelf.")
        .arg(
            Arg::new("update")
                .long("update")
                .help("Install the last release.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .help("Write the paths that you can delete.")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("uninstall") {
        write_uninstall_message();
        std::process::exit(0);
    }

    if matches.get_flag("update") {
        match run_update(RELEASES_API).await {
            Ok(message) => {
                println!("{}", message);
                std::process::exit(0);
            }
            Err(message) => {
                eprintln!("{}", message);
                std::process::exit(1);
            }
        }
    }
}
```

The path `~/.local/share/toutui` holds the copies that the program plays with
no server. The message names it, because a user who removes the program wants
to remove those files as well.

- [ ] **Step 2: Change the call in `src/main.rs`**

Replace `clap();` with `clap().await;`.

- [ ] **Step 3: Verify the two commands by hand**

```bash
cargo run -- --uninstall
cargo run -- --update
```

Expected: the first gives the list of the paths and changes no file. The
second gives a message. The fork has no release before the maintainer makes
one, therefore a message that says that the API gives nothing is correct now.

- [ ] **Step 4: Run the gate and commit**

```bash
cargo clippy --all-targets -- -D warnings
ALSA_CONFIG_PATH=/dev/null cargo test
git add src/utils/clap.rs src/main.rs
git commit -m "feat: --update installs the last release of the fork

--update receives the archive of its target, it compares the sum, and
it moves the new binary. --uninstall writes the paths and deletes
nothing, because the command before it used sudo rm -r on paths that
came from variables of the environment.

Closes #21

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task R7: The short script of installation

`hello_toutui.sh` has 1080 lines. It installs VLC and netcat that the player
in the program does not need. Its array of sums needs a person to write each
sum. Its verification of itself at line 15 gives two variables that no code
sets, therefore that verification never operated.

**Files:**
- Create: `install.sh`
- Delete: `hello_toutui.sh`

**Interfaces:**
- Consumes: the assets of the workflow of release.
- Produces: `install.sh` at the root of the repository.

- [ ] **Step 1: Write the script**

Create `install.sh`:

```bash
#!/usr/bin/env bash
# Install toutui.
#
# Use:
#   curl -LsSf https://raw.githubusercontent.com/ealtun21/Toutui/main/install.sh | bash
#
# The script receives the archive of the last release, it compares the sum
# with SHA256SUMS of that release, and it installs the binary. The script
# installs no other program, because toutui plays the audio itself.

set -euo pipefail

REPO="ealtun21/Toutui"
API="https://api.github.com/repos/${REPO}/releases/latest"
BIN_DIR="${TOUTUI_BIN_DIR:-/usr/local/bin}"

fail() {
    echo "[ERROR] $1" >&2
    exit 1
}

identify_target() {
    local os arch
    os=$(uname -s)
    arch=$(uname -m)

    case "$os" in
        Darwin) echo "universal-apple-darwin" ;;
        Linux)
            case "$arch" in
                x86_64)  echo "x86_64-unknown-linux-gnu" ;;
                aarch64) echo "aarch64-unknown-linux-gnu" ;;
                *) fail "Linux $arch has no archive. Use: cargo install --git https://github.com/${REPO}" ;;
            esac
            ;;
        *) fail "$os has no archive. Use: cargo install --git https://github.com/${REPO}" ;;
    esac
}

config_dir() {
    if [ -n "${XDG_CONFIG_HOME:-}" ]; then
        echo "${XDG_CONFIG_HOME}/toutui"
    elif [ "$(uname -s)" = "Darwin" ]; then
        echo "${HOME}/Library/Preferences/toutui"
    else
        echo "${HOME}/.config/toutui"
    fi
}

sum_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

main() {
    [ "$(id -u)" -ne 0 ] || fail "Do not run this script as root."

    command -v curl >/dev/null 2>&1 || fail "Install curl first."
    command -v tar  >/dev/null 2>&1 || fail "Install tar first."

    local target archive tag tmp
    target=$(identify_target)
    archive="toutui-${target}.tar.gz"

    tag=$(curl -sSfL "$API" | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
    [ -n "$tag" ] || fail "The repository has no release."

    echo "[INFO] The last release is ${tag}."

    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT

    local base="https://github.com/${REPO}/releases/download/${tag}"
    curl -sSfL "${base}/${archive}" -o "${tmp}/${archive}"
    curl -sSfL "${base}/SHA256SUMS" -o "${tmp}/SHA256SUMS"

    local expected actual
    expected=$(grep " ${archive}\$" "${tmp}/SHA256SUMS" | awk '{print $1}')
    [ -n "$expected" ] || fail "SHA256SUMS has no sum for ${archive}."
    actual=$(sum_of "${tmp}/${archive}")
    [ "$expected" = "$actual" ] || fail "The sum of ${archive} is not correct."

    echo "[INFO] The sum is correct."

    tar -xzf "${tmp}/${archive}" -C "$tmp"
    sudo install -m 755 "${tmp}/toutui" "${BIN_DIR}/toutui"
    echo "[INFO] The binary is in ${BIN_DIR}/toutui."

    local config
    config=$(config_dir)
    mkdir -p "$config"
    if [ ! -f "${config}/config.toml" ]; then
        curl -sSfL "${base}/config.example.toml" -o "${config}/config.toml"
        echo "[INFO] The configuration is in ${config}/config.toml."
    fi

    if [ ! -f "${config}/.env" ]; then
        local key
        key=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')
        ( umask 077; echo "TOUTUI_SECRET_KEY=${key}" > "${config}/.env" )
        echo "[INFO] The secret key is in ${config}/.env."
    fi

    if [ "$(uname -s)" = "Linux" ]; then
        mkdir -p "${HOME}/.local/share/applications"
        curl -sSfL "${base}/toutui.desktop" \
            -o "${HOME}/.local/share/applications/toutui.desktop"
    fi

    echo "[DONE] Type toutui to start the program."
}

main "$@"
```

The script makes the secret key with `/dev/urandom`, and `umask 077` gives the
file its permissions at the time it comes into existence. The script before
this one asked the user for the key, and a user who typed a short key had weak
encryption.

- [ ] **Step 2: Verify the script**

```bash
bash -n install.sh
shellcheck install.sh || true
chmod +x install.sh
```

Expected: `bash -n` gives no output. Correct every error that `shellcheck`
gives, if `shellcheck` is present.

- [ ] **Step 3: Delete the old script and commit**

```bash
git rm hello_toutui.sh
git add install.sh
git commit -m "feat: give the fork a short script of installation

The script receives the archive of the last release, it compares the
sum with SHA256SUMS, and it installs the binary.

The script before this one had 1080 lines. It installed VLC and netcat
that the player in the program does not need, it held an array of sums
that a person wrote, and its verification of itself gave two variables
that no code set.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task R8: The documents

**Files:**
- Modify: `README.md`
- Modify: `docs/TAKEOVER-BACKLOG.md`
- Modify: `known_bugs.md`
- Modify: `src/utils/changelog.rs`
- Modify: `CONTRIBUTING.md`
- Modify: `.github/ISSUE_TEMPLATE/*.yml`, `.github/pull_request_template.md`

**Interfaces:**
- Consumes: every task before it.
- Produces: a README that gives three ways to install and no other.

- [ ] **Step 1: Write the part of the README that tells how to install**

Remove every instruction that names the AUR, crates.io, `hello_toutui.sh`, the
wiki of the archived project, or a release of the archived project. Remove the
block `> [!WARNING]` that says that the fork has no release. Put this in place:

````markdown
## Installation

### The script

```bash
curl -LsSf https://raw.githubusercontent.com/ealtun21/Toutui/main/install.sh | bash
```

The script receives the archive of the last release, it compares the sum with
`SHA256SUMS`, and it installs the binary in `/usr/local/bin`.

### From the source

```bash
cargo install --git https://github.com/ealtun21/Toutui
```

Alpine and every other system without glibc must use this method. The build
needs the headers of ALSA: `libasound2-dev` on Debian, `alsa-lib` on Arch.

### The archives

The [releases](https://github.com/ealtun21/Toutui/releases) hold one archive
for each system, and `SHA256SUMS`. Compare the sum before you use an archive.

### The update

```bash
toutui --update
```

The program receives the archive of its target, it compares the sum, and it
moves the new binary. The program runs no file that it receives.

### The removal

```bash
toutui --uninstall
```

The command writes the paths. It deletes nothing.
````

- [ ] **Step 2: Remove the rest of the junk from the README**

Read the whole file. Remove every part that the fork does not do:

- Instructions that name `yay -S toutui` or `toutui-bin`, because the packages
  of the AUR build the archived code and this fork publishes no package.
- Instructions that tell the user to install VLC or netcat. The player in the
  program needs neither.
- Links to the wiki of the archived repository that give instructions of
  installation. A link that names the archived project as the author stays.

Keep every link that gives credit to AlbanDAVID.

- [ ] **Step 3: Change the addresses in the other documents**

```bash
grep -rn "hello_toutui\|yay -S toutui\|toutui-bin\|crates.io" \
    README.md CONTRIBUTING.md known_bugs.md docs/ .github/
```

Correct each result. A result in `docs/superpowers/` is a record of the design
and does not change.

- [ ] **Step 4: Mark T-21 as done in the backlog**

In `docs/TAKEOVER-BACKLOG.md`, the section `### T-21` becomes:

```markdown
### T-21: `--update` installed the archived original project

`src/utils/clap.rs` ran the install script of the original repository, and
every address in that script names `AlbanDAVID/Toutui`. Therefore the command
built and installed the original program, and the user lost every correction
of the fork.

**The correction.** The fork makes its own releases from a tag on `main`.
`--update` receives the archive of its target, it compares the sum with
`SHA256SUMS`, and it moves the new binary. The program runs no file that it
receives. `--uninstall` writes the paths and deletes nothing.

The fork publishes no package to a registry. The three ways to install are
the script, `cargo install --git`, and the archives of the releases.
```

T-14 stays open. Write one line in its entry that says that the fork examined
T-14 with T-21 and did not correct it.

- [ ] **Step 5: Add the entry of the changelog**

In `src/utils/changelog.rs`, add this above `changelog_14`, and push it first:

```rust
let changelog_15 = format!(
    "Changelog Toutui v{} (08/10/2026) \n\
     \n\
     Added:\n\
     - The program plays a local copy when the server does not answer, and\n\
       it sends the positions when the server answers again.\n\
     - The program updates itself with `toutui --update`. The program\n\
       compares the sum of the archive before it moves the new binary.\n\
     - The releases come from this repository, and each archive has a proof\n\
       of its origin.\n\
     \n\
     Fixed:\n\
     - `--update` installed the archived original project.\n\
     - `Mark as finished` did not always operate.\n\
     \n\
     Changed:\n\
     - The script of installation has 100 lines and not 1080. It installs\n\
       no VLC and no netcat, because the player in the program needs\n\
       neither.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy and be toutui!\n\
     ####\n",
     VERSION
);
```

`VERSION` must come back to this file for this entry only, as
`const VERSION: &str = env!("CARGO_PKG_VERSION");`. The entry of 15/05/2025
keeps its literal `v0.4.2-beta`.

Every line ends with `\n\`. A line that ends with `\` only joins to the next
line with no break, and a line that ends with neither puts its indentation in
the text.

- [ ] **Step 6: Run the gate and commit**

```bash
cargo clippy --all-targets -- -D warnings
ALSA_CONFIG_PATH=/dev/null cargo test
git add -A
git commit -m "docs: tell how to install, to update, and to remove toutui

The README gives three ways: the script, cargo install --git, and the
archives of the releases. It names no registry, because the fork
publishes to none.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

## The work of the maintainer

No subagent does these. They change a public repository.

1. Push the branch and merge it to `main`.
2. Make the first release:

```bash
git tag v0.5.0-beta
git push origin v0.5.0-beta
gh run watch
```

3. Confirm that the release holds three archives, `SHA256SUMS`,
   `config.example.toml`, and `toutui.desktop`, and that it is not a draft.
4. Confirm that the binary of Linux runs on an old system:

```bash
gh release download v0.5.0-beta -p 'toutui-x86_64-unknown-linux-gnu.tar.gz' -D /tmp/rel
tar -xzf /tmp/rel/toutui-x86_64-unknown-linux-gnu.tar.gz -C /tmp/rel
podman run --rm -v /tmp/rel:/rel:ro debian:bullseye /rel/toutui --version
```

5. Delete the branch `stable` if no address names it.
6. Run `nix flake check` on a machine that has a store of Nix.
