# Fork Release Process Implementation Plan (T-21)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the fork its own way to install, to update, and to remove, under the name `abstui`, so that `--update` stops being a command that installs the archived original project.

**Architecture:** One module holds every path that the program uses, and that module also copies the configuration of the original program one time. The name of the project then changes in one place. A workflow makes the releases from a tag on `main`, and it writes `SHA256SUMS` with the machine. The program updates itself: it receives the archive of its own target, it compares the sum, and it moves the new binary on to the old binary. No script runs.

**Tech Stack:** Rust 1.97, `reqwest` with rustls, `sha2`, `flate2` with the Rust backend, `tar`, GitHub Actions, `cargo-zigbuild`, crates.io, the AUR.

**Spec:** `docs/superpowers/specs/2026-08-10-fork-release-process-design.md`

## Global Constraints

- Every document, doc comment, and user-facing string uses ASD-STE100 Simplified Technical English: short sentences, active voice, one instruction in one sentence.
- Before each commit, `cargo clippy --all-targets -- -D warnings` must give no output, and `cargo test` must pass.
- Add no dependency that needs a C toolchain or a program that the user installs. `cargo tree -i openssl-sys` must find nothing.
- The version of the first release of the fork is `0.5.0-beta`.
- The name of the project is `abstui`. The name of the binary is `abstui`. The directory of configuration is `abstui`.
- The old directory of configuration is `toutui`. The program copies it. It must not move it and must not delete it.
- Tasks 1 to 3 must come before task 4. If the name changes first, a user who updates loses the configuration.
- Commit changes that are separate in separate commits.
- This work closes issue #21 (T-21) and issue #14 (T-14).

## File Structure

| File | Responsibility |
|---|---|
| `src/paths.rs` (create) | Every path of the program, and the copy of the old directory |
| `src/lib.rs` (modify) | Declares `paths` and `update` |
| `src/config.rs:50-67` (modify) | Uses `paths` in place of its own code |
| `src/utils/logs.rs:9-24` (modify) | Uses `paths` in place of its own code |
| `src/db/migrate.rs:19-33` (modify) | Uses `paths` in place of its own code |
| `src/main.rs:34-53` (modify) | Uses `paths`, and calls the copy at the start |
| `src/utils/encrypt_token.rs` (modify) | Reads the new name of the key, then the old name |
| `src/utils/clap.rs` (modify) | `--update` runs the update. `--uninstall` writes the paths |
| `src/utils/check_update.rs` (modify) | Asks the API of the fork |
| `src/update/mod.rs` (create) | Declares the two modules below |
| `src/update/release.rs` (create) | Asks the API, and finds the archive of this target |
| `src/update/install.rs` (create) | Compares the sum, opens the archive, moves the binary |
| `tests/paths.rs` (create) | The paths and the copy of the old directory |
| `tests/update.rs` (create) | The plan of the release and the installation |
| `.github/workflows/release.yml` (modify) | Makes the release from a tag |
| `install.sh` (create) | The short script of installation |
| `hello_toutui.sh` (delete) | The long script of the original project |
| `packaging/aur/PKGBUILD` (create) | The package of the AUR |
| `packaging/aur/.SRCINFO` (create) | The metadata of the AUR |
| `linux/abstui.desktop` (create) | The file of the launcher |
| `linux/toutui.desktop` (delete) | The old file of the launcher |

---

### Task 1: One place for every path

The code that finds the directory of configuration is in four files, and each
copy is the same. This task puts that code in one module. The module gives
pure functions that take the parent directory, and therefore the tests need no
variable of the environment.

**Files:**
- Create: `src/paths.rs`
- Create: `tests/paths.rs`
- Modify: `src/lib.rs`
- Modify: `src/config.rs:50-67`
- Modify: `src/utils/logs.rs:9-24`
- Modify: `src/db/migrate.rs:19-33`
- Modify: `src/main.rs:34-53`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub const APP_DIR: &str` — `"abstui"`
  - `pub const OLD_APP_DIR: &str` — `"toutui"`
  - `pub fn config_home() -> PathBuf`
  - `pub fn default_config_home(home: &Path) -> PathBuf`
  - `pub fn config_dir() -> PathBuf`
  - `pub fn config_dir_in(config_home: &Path) -> PathBuf`
  - `pub fn old_config_dir_in(config_home: &Path) -> PathBuf`
  - `pub fn config_file() -> PathBuf`
  - `pub fn env_file() -> PathBuf`
  - `pub fn db_file() -> PathBuf`
  - `pub fn log_file() -> PathBuf`

- [ ] **Step 1: Write the failing test**

Create `tests/paths.rs`:

```rust
//! Tests of the paths of the program.
//!
//! The tests use the pure functions that take the parent directory. A
//! variable of the environment is common to all the tests of one process,
//! and therefore a test that writes a variable is not safe.

use std::path::{Path, PathBuf};
use toutui::paths;

/// The directory of configuration is the parent directory and the name of the
/// program.
#[test]
fn the_directory_holds_the_name_of_the_program() {
    let home = Path::new("/tmp/example");
    assert_eq!(
        paths::config_dir_in(home),
        PathBuf::from("/tmp/example/abstui")
    );
}

/// The old directory holds the name of the program before the fork.
#[test]
fn the_old_directory_holds_the_old_name() {
    let home = Path::new("/tmp/example");
    assert_eq!(
        paths::old_config_dir_in(home),
        PathBuf::from("/tmp/example/toutui")
    );
}

/// On a system that is not macOS, the parent directory is `.config`.
#[test]
fn the_parent_directory_is_config_on_linux() {
    let home = Path::new("/home/example");
    let expected = if cfg!(target_os = "macos") {
        PathBuf::from("/home/example/Library/Preferences")
    } else {
        PathBuf::from("/home/example/.config")
    };
    assert_eq!(paths::default_config_home(home), expected);
}
```

- [ ] **Step 2: Run the test to see that it fails**

Run: `cargo test --test paths`
Expected: FAIL. The compiler says that `paths` is not a module of `toutui`.

- [ ] **Step 3: Write the module**

Create `src/paths.rs`:

```rust
//! The directories and the files that the program uses.
//!
//! Every module takes its paths from here. Therefore a change of the name of
//! the directory happens in one place only.

use std::path::{Path, PathBuf};

/// The name of the directory of this program.
pub const APP_DIR: &str = "abstui";

/// The name of the directory of the program before the fork.
pub const OLD_APP_DIR: &str = "toutui";

/// Gives the parent directory of the directory of configuration.
///
/// `XDG_CONFIG_HOME` has the highest importance. If that variable is absent
/// or empty, the function uses the home directory of the user.
pub fn config_home() -> PathBuf {
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(value) if !value.is_empty() => PathBuf::from(value),
        _ => {
            let home = dirs::home_dir().expect("Unable to find the user's home directory");
            default_config_home(&home)
        }
    }
}

/// Gives the parent directory for a home directory.
///
/// The parent directory is `Library/Preferences` on macOS and `.config` on
/// every other system.
pub fn default_config_home(home: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        home.join("Library").join("Preferences")
    } else {
        home.join(".config")
    }
}

/// Gives the directory of configuration of this program.
pub fn config_dir() -> PathBuf {
    config_dir_in(&config_home())
}

/// Gives the directory of configuration in a parent directory.
pub fn config_dir_in(config_home: &Path) -> PathBuf {
    config_home.join(APP_DIR)
}

/// Gives the directory of the program before the fork, in a parent directory.
pub fn old_config_dir_in(config_home: &Path) -> PathBuf {
    config_home.join(OLD_APP_DIR)
}

/// Gives the path of `config.toml`.
pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

/// Gives the path of `.env`.
pub fn env_file() -> PathBuf {
    config_dir().join(".env")
}

/// Gives the path of the database.
pub fn db_file() -> PathBuf {
    config_dir().join("db.sqlite3")
}

/// Gives the path of the file of the log.
pub fn log_file() -> PathBuf {
    config_dir().join("abstui.log")
}
```

- [ ] **Step 4: Declare the module**

In `src/lib.rs`, add `pub mod paths;` after `pub mod logic;`, so that the list
stays in the sequence of the alphabet:

```rust
pub mod logic;
pub mod paths;
pub mod player;
```

- [ ] **Step 5: Run the test to see that it passes**

Run: `cargo test --test paths`
Expected: PASS, three tests.

- [ ] **Step 6: Make the four files use the module**

In `src/config.rs`, remove the block at lines 50 to 67 that makes
`config_home_path` and `config_path`, and remove `use std::env;` and
`use std::path::PathBuf;` if no other code in the file needs them. Put this in
place of the removed block, inside `load_config`:

```rust
    let config_path = crate::paths::config_file();
    let config_path_str = config_path.to_str().unwrap().to_string();
```

In `src/utils/logs.rs`, remove the block that makes `config_home_path` and
`log_path`, and remove `use std::env;` and `use std::path::PathBuf;`. Put this
in place:

```rust
    let log_path = crate::paths::log_file();
```

In `src/db/migrate.rs`, replace the whole body of `db_path` and remove
`use std::env;` and `use std::path::PathBuf;` if nothing else needs them. Keep
`use std::path::PathBuf;` because the signature needs it:

```rust
/// Gives the full path of the database file.
pub fn db_path() -> PathBuf {
    crate::paths::db_file()
}
```

In `src/main.rs`, remove the block from `let home_dir = ...` to
`let env_path = ...` at lines 37 to 52, and put this in place:

```rust
    // The program reads the secret key from `.env`. See `encrypt_token.rs`.
    let env_path = toutui::paths::env_file();
```

Then remove `use std::env;` and `use std::path::PathBuf;` from `src/main.rs`
if no other code needs them.

- [ ] **Step 7: Run the whole gate**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clippy gives no output. Every test passes.

- [ ] **Step 8: Commit**

```bash
git add src/paths.rs src/lib.rs src/config.rs src/utils/logs.rs src/db/migrate.rs src/main.rs tests/paths.rs
git commit -m "refactor: put every path of the program in one module

The code that finds the directory of configuration was in four files.
One module holds it now, thus a change of the name of the directory
happens in one place.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 2: The copy of the old configuration

The program copies `config.toml`, `.env`, and `db.sqlite3` from the directory
`toutui` to the directory `abstui` one time. The copy keeps the old directory
complete. The copy writes `ABSTUI_SECRET_KEY` in place of `TOUTUI_SECRET_KEY`
in the new `.env`, because that key decrypts the token. This task closes T-14.

**Files:**
- Modify: `src/paths.rs`
- Modify: `tests/paths.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `paths::config_dir_in`, `paths::old_config_dir_in`, `paths::config_home` from Task 1.
- Produces: `pub fn migrate_old_config(config_home: &Path) -> std::io::Result<bool>` — gives `true` if it copied the files. `pub fn migrate_old_config_here() -> std::io::Result<bool>` — the same function with the real parent directory.

- [ ] **Step 1: Write the failing tests**

Add to `tests/paths.rs`:

```rust
use std::fs;

/// Makes a parent directory that holds an old directory with the three files.
fn old_installation(config_home: &Path) {
    let old = paths::old_config_dir_in(config_home);
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("config.toml"), "[colors]\n").unwrap();
    fs::write(old.join(".env"), "TOUTUI_SECRET_KEY=abc123\n").unwrap();
    fs::write(old.join("db.sqlite3"), b"not a real database").unwrap();
}

/// The program copies the three files when only the old directory is present.
#[test]
fn the_program_copies_the_old_files() {
    let home = tempfile::tempdir().unwrap();
    old_installation(home.path());

    let copied = paths::migrate_old_config(home.path()).unwrap();

    let new = paths::config_dir_in(home.path());
    assert!(copied);
    assert_eq!(fs::read_to_string(new.join("config.toml")).unwrap(), "[colors]\n");
    assert_eq!(fs::read(new.join("db.sqlite3")).unwrap(), b"not a real database");
}

/// The copy writes the new name of the key, and it keeps the value.
#[test]
fn the_copy_changes_the_name_of_the_key() {
    let home = tempfile::tempdir().unwrap();
    old_installation(home.path());

    paths::migrate_old_config(home.path()).unwrap();

    let env = fs::read_to_string(paths::config_dir_in(home.path()).join(".env")).unwrap();
    assert_eq!(env, "ABSTUI_SECRET_KEY=abc123\n");
}

/// The old directory does not change.
#[test]
fn the_old_directory_stays_complete() {
    let home = tempfile::tempdir().unwrap();
    old_installation(home.path());

    paths::migrate_old_config(home.path()).unwrap();

    let old = paths::old_config_dir_in(home.path());
    assert_eq!(
        fs::read_to_string(old.join(".env")).unwrap(),
        "TOUTUI_SECRET_KEY=abc123\n"
    );
    assert!(old.join("db.sqlite3").exists());
}

/// The program does not write when the new directory is present.
#[test]
fn the_program_does_not_write_on_a_new_installation() {
    let home = tempfile::tempdir().unwrap();
    old_installation(home.path());
    let new = paths::config_dir_in(home.path());
    fs::create_dir_all(&new).unwrap();
    fs::write(new.join("config.toml"), "the file of the user\n").unwrap();

    let copied = paths::migrate_old_config(home.path()).unwrap();

    assert!(!copied);
    assert_eq!(
        fs::read_to_string(new.join("config.toml")).unwrap(),
        "the file of the user\n"
    );
}

/// The program makes the new directory when no directory is present.
#[test]
fn the_program_makes_the_directory_for_a_first_installation() {
    let home = tempfile::tempdir().unwrap();

    let copied = paths::migrate_old_config(home.path()).unwrap();

    assert!(!copied);
    assert!(paths::config_dir_in(home.path()).is_dir());
}

/// A file that the old directory does not have gives no error.
#[test]
fn a_file_that_is_absent_gives_no_error() {
    let home = tempfile::tempdir().unwrap();
    let old = paths::old_config_dir_in(home.path());
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("config.toml"), "[colors]\n").unwrap();

    let copied = paths::migrate_old_config(home.path()).unwrap();

    assert!(copied);
    assert!(!paths::config_dir_in(home.path()).join(".env").exists());
}
```

Add `tempfile` to the test file with `use tempfile;` only if the compiler asks
for it. `tempfile` is already in `[dev-dependencies]`.

- [ ] **Step 2: Run the tests to see that they fail**

Run: `cargo test --test paths`
Expected: FAIL. The compiler says that `migrate_old_config` does not exist.

- [ ] **Step 3: Write the function**

Add to `src/paths.rs`:

```rust
/// The files that the program copies from the old directory.
const MIGRATED_FILES: [&str; 3] = ["config.toml", ".env", "db.sqlite3"];

/// Copies the configuration of the program before the fork.
///
/// The function makes the new directory. If the new directory was present
/// before, the function copies nothing, because the user has a configuration
/// already. If the new directory was absent and the old directory is present,
/// the function copies the files and gives `true`.
///
/// The function copies the files. It does not move them, thus the old
/// directory stays complete and the user can use the old program again.
pub fn migrate_old_config(config_home: &Path) -> std::io::Result<bool> {
    let new = config_dir_in(config_home);
    if new.is_dir() {
        return Ok(false);
    }

    std::fs::create_dir_all(&new)?;

    let old = old_config_dir_in(config_home);
    if !old.is_dir() {
        return Ok(false);
    }

    let mut copied = false;
    for name in MIGRATED_FILES {
        let source = old.join(name);
        if !source.is_file() {
            continue;
        }
        if name == ".env" {
            let text = std::fs::read_to_string(&source)?;
            std::fs::write(new.join(name), text.replace("TOUTUI_SECRET_KEY", "ABSTUI_SECRET_KEY"))?;
        } else {
            std::fs::copy(&source, new.join(name))?;
        }
        copied = true;
    }

    Ok(copied)
}

/// Copies the old configuration in the real parent directory.
pub fn migrate_old_config_here() -> std::io::Result<bool> {
    migrate_old_config(&config_home())
}
```

- [ ] **Step 4: Run the tests to see that they pass**

Run: `cargo test --test paths`
Expected: PASS, nine tests.

- [ ] **Step 5: Call the function at the start of the program**

In `src/main.rs`, the call must come before `setup_logs()`, because
`setup_logs` opens a file in the directory and fails if the directory is
absent. Put this immediately after `clap();`:

```rust
    // The program before the fork used a different directory. The program
    // copies that directory one time. The old directory does not change,
    // therefore the user can use the old program again. See T-14 and T-21.
    match toutui::paths::migrate_old_config_here() {
        Ok(true) => println!("The configuration of the old directory is now in the new directory."),
        Ok(false) => {}
        Err(e) => eprintln!("The program cannot read the old directory: {}", e),
    }
```

The message uses `println!` and not `log::info!`, because `setup_logs` did not
run yet.

- [ ] **Step 6: Run the whole gate**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clippy gives no output. Every test passes.

- [ ] **Step 7: Commit**

```bash
git add src/paths.rs src/main.rs tests/paths.rs
git commit -m "feat: copy the configuration of the program before the fork

The program copies config.toml, .env, and db.sqlite3 to the new
directory one time. The old directory does not change. The copy writes
the new name of the secret key, thus the user keeps the token.

Closes #14

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 3: The program accepts the two names of the secret key

The copy in Task 2 writes `ABSTUI_SECRET_KEY`. A user who wrote `.env` by hand
has `TOUTUI_SECRET_KEY`. The program must accept both, and the new name has
the higher importance.

**Files:**
- Modify: `src/utils/encrypt_token.rs`
- Create: `tests/secret_key.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: `pub fn secret_key() -> Result<String, String>` in `src/utils/encrypt_token.rs`.

- [ ] **Step 1: Write the failing test**

Create `tests/secret_key.rs`:

```rust
//! Tests of the name of the secret key.
//!
//! The program before the fork used `TOUTUI_SECRET_KEY`. This program uses
//! `ABSTUI_SECRET_KEY`, and it accepts the old name.
//!
//! The tests write variables of the environment, and variables are common to
//! the process. Therefore this file holds one test only.

use toutui::utils::encrypt_token::secret_key;

#[test]
fn the_program_accepts_the_two_names() {
    // No name is present.
    std::env::remove_var("ABSTUI_SECRET_KEY");
    std::env::remove_var("TOUTUI_SECRET_KEY");
    assert!(secret_key().is_err());

    // The old name only.
    std::env::set_var("TOUTUI_SECRET_KEY", "old");
    assert_eq!(secret_key().unwrap(), "old");

    // The two names. The new name has the higher importance.
    std::env::set_var("ABSTUI_SECRET_KEY", "new");
    assert_eq!(secret_key().unwrap(), "new");
}
```

`set_var` and `remove_var` are safe on edition 2021, and the package uses that
edition. Do not put the calls in an `unsafe` block. An `unsafe` block that is
not necessary is a warning, and clippy stops on a warning.

- [ ] **Step 2: Run the test to see that it fails**

Run: `cargo test --test secret_key`
Expected: FAIL. The compiler says that `secret_key` does not exist.

- [ ] **Step 3: Write the function and use it**

In `src/utils/encrypt_token.rs`, add this function at the top of the file,
after the `use` lines:

```rust
/// The message that tells the user how to make the secret key.
const NO_KEY: &str = "No secret key is present. Do this:\n\
    mkdir -p ~/.config/abstui\n\
    echo 'ABSTUI_SECRET_KEY=secret' >> ~/.config/abstui/.env";

/// Gives the secret key that encrypts the token.
///
/// The name `ABSTUI_SECRET_KEY` has the higher importance. The program also
/// accepts `TOUTUI_SECRET_KEY`, because a user who wrote `.env` by hand
/// before the fork has that name.
pub fn secret_key() -> Result<String, String> {
    env::var("ABSTUI_SECRET_KEY")
        .or_else(|_| env::var("TOUTUI_SECRET_KEY"))
        .map_err(|_| {
            error!("{}", NO_KEY);
            NO_KEY.to_string()
        })
}
```

Then replace the body of `encrypt_token`:

```rust
pub fn encrypt_token(token_to_encrypt: &str) -> Result<String, String> {
    let key = secret_key()?;
    let mc = new_magic_crypt!(key, 256);
    Ok(mc.encrypt_str_to_base64(token_to_encrypt))
}
```

And the body of `decrypt_token`:

```rust
pub fn decrypt_token(encrypted_token: &str) -> Result<String, String> {
    let key = secret_key()?;
    let mc = new_magic_crypt!(key, 256);
    mc.decrypt_base64_to_string(encrypted_token).map_err(|_| {
        error!("Failed to decrypt the token.");
        "Failed to decrypt the token.".to_string()
    })
}
```

- [ ] **Step 4: Run the test to see that it passes**

Run: `cargo test --test secret_key`
Expected: PASS.

- [ ] **Step 5: Run the whole gate**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clippy gives no output. Every test passes.

- [ ] **Step 6: Commit**

```bash
git add src/utils/encrypt_token.rs tests/secret_key.rs
git commit -m "feat: accept the two names of the secret key

The program reads ABSTUI_SECRET_KEY first, and then
TOUTUI_SECRET_KEY. A user who wrote .env by hand before the fork keeps
a program that operates.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 4: The change of the name

The crate, the binary, the display name, and the desktop file become `abstui`.
Task 1 put the directory of configuration in one place already, therefore this
task changes no path.

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs:1`
- Modify: every file in `tests/` that has `use toutui::`
- Create: `linux/abstui.desktop`
- Delete: `linux/toutui.desktop`
- Modify: `README.md`
- Modify: `LICENSE`

**Interfaces:**
- Consumes: nothing.
- Produces: the crate `abstui`. Every later task writes `use abstui::` and not `use toutui::`.

- [ ] **Step 1: Change the metadata of the package**

In `Cargo.toml`, replace the block `[package]`:

```toml
[package]
name = "abstui"
version = "0.5.0-beta"
edition = "2021"
description = "A TUI client of Audiobookshelf for Linux and macOS."
license = "GPL-3.0-or-later"
repository = "https://github.com/ealtun21/abstui"
readme = "README.md"
keywords = ["audiobookshelf", "audiobook", "tui", "podcast", "terminal"]
categories = ["command-line-utilities", "multimedia::audio"]
authors = [
    "AlbanDAVID <https://github.com/AlbanDAVID>",
    "ealtun21 <https://github.com/ealtun21>",
]
```

crates.io accepts five keywords, and each keyword must have 20 characters or
less.

- [ ] **Step 2: Change the name of the crate in the code**

Run this command. It changes `use toutui::` and `toutui::` in `src/main.rs`
and in every file of `tests/`:

```bash
grep -rl "toutui::" src/main.rs tests/ | xargs sed -i 's/\btoutui::/abstui::/g'
```

- [ ] **Step 3: Run the build to see that it passes**

Run: `cargo build`
Expected: the build passes. The binary is `target/debug/abstui`.

- [ ] **Step 4: Change the file of the launcher**

Create `linux/abstui.desktop`:

```ini
[Desktop Entry]
Name=AbsTui
GenericName=Audiobookshelf client
Exec=abstui
Icon=utilities-terminal
Type=Application
Categories=Utility;
Terminal=true
```

Then delete the old file:

```bash
git rm linux/toutui.desktop
```

- [ ] **Step 5: Change the name in the strings that the user reads**

Find every remaining string:

```bash
grep -rn -i "toutui" src/ --include="*.rs" | grep -v "TOUTUI_SECRET_KEY" | grep -v "OLD_APP_DIR"
```

Change each name that the user reads to `AbsTui`, and each command that the
user types to `abstui`. Do not change:
- `TOUTUI_SECRET_KEY` in `src/utils/encrypt_token.rs`, because Task 3 needs it.
- `OLD_APP_DIR` in `src/paths.rs`, because Task 2 needs it.
- The names of the old files in `src/utils/changelog.rs`, because the
  changelog is a record of the past.

- [ ] **Step 6: Write the attribution in the README**

Replace the first block of `README.md`, from the line `## 🍴 This is a
maintained fork` to the line before the badge of CI:

```markdown
## 🍴 AbsTui is a fork of Toutui

AlbanDAVID wrote [Toutui](https://github.com/AlbanDAVID/Toutui) and archived
it. AbsTui continues that work under a new name. The new name keeps the two
projects separate, because the two projects have different maintainers.

AbsTui corrects the faults of the original project, and it adds functions.
`docs/TAKEOVER-BACKLOG.md` holds the full list. Report a fault in the
[issues of this repository](https://github.com/ealtun21/abstui/issues), and
not in the archived repository.

The first time that AbsTui starts, it copies `~/.config/toutui/` to
`~/.config/abstui/`. The old directory does not change, therefore Toutui
continues to operate.
```

Then replace the title `# 🦜 Toutui: A TUI Audiobookshelf client for Linux and
macOS` with `# 🦜 AbsTui: A TUI Audiobookshelf client for Linux and macOS`.

Then change every remaining `ealtun21/Toutui` to `ealtun21/abstui`:

```bash
sed -i 's|ealtun21/Toutui|ealtun21/abstui|g' README.md
```

Keep every `AlbanDAVID/Toutui` without a change, because those addresses name
the archived repository.

- [ ] **Step 7: Write the attribution in the LICENSE**

Find the block of copyright at the end of `LICENSE`, in the part
`How to Apply These Terms to Your New Programs`. Add these two lines below the
existing line of copyright:

```
    Toutui, Copyright (C) 2025 AlbanDAVID
    AbsTui, Copyright (C) 2026 ealtun21
```

- [ ] **Step 8: Write the credit on the screen that the user reads**

`src/app.rs:203` calls `changelog()`, and the screen of settings shows the
result. Therefore the credit goes at the top of that text.

In `src/utils/changelog.rs`, add these lines at the start of the function
`changelog`, before `let mut changelog = String::new();` becomes the first
statement. Put the credit in the string that the function gives:

```rust
pub fn changelog() -> String {
    let mut changelog = String::new();

    // The screen of settings shows this text. The credit is at the top,
    // because AlbanDAVID wrote the original program.
    changelog.push_str(
        "AbsTui is a fork of Toutui. AlbanDAVID wrote Toutui and archived it.\n\
         https://github.com/AlbanDAVID/Toutui\n\
         \n\
         ####\n",
    );
```

The rest of the function does not change.

- [ ] **Step 9: Run the whole gate**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clippy gives no output. Every test passes.

Then start the program and look at the screen of settings:

Run: `cargo run`
Expected: the screen of settings shows the credit at the top.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat!: change the name of the project to abstui

The crate, the binary, the directory of configuration, and the file of
the launcher become abstui. The two projects then stay separate,
because the two projects have different maintainers.

The README and the LICENSE name AlbanDAVID as the author of the
original program.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 5: The workflow that makes the release

A tag makes the release. The workflow builds three archives, it writes
`SHA256SUMS` with the machine, and it publishes the release. The workflow now
makes a draft, and therefore the address `/releases/latest` gives nothing.

**Files:**
- Modify: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: the binary `abstui` from Task 4.
- Produces: the assets `abstui-x86_64-unknown-linux-gnu.tar.gz`, `abstui-aarch64-unknown-linux-gnu.tar.gz`, `abstui-universal-apple-darwin.tar.gz`, `SHA256SUMS`, `config.example.toml`, and `abstui.desktop`. Task 7 and Task 10 read these names.

- [ ] **Step 1: Write the workflow**

Replace the whole of `.github/workflows/release.yml`:

```yaml
name: Release

# The workflow writes the release, and it makes a proof of the origin of
# each asset.
permissions:
  contents: write
  id-token: write
  attestations: write

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          # cargo-zigbuild gives a floor of glibc 2.17. Therefore one binary
          # operates from CentOS 7 and later.
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            build: zig
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-24.04-arm
            build: zig
          - target: universal-apple-darwin
            os: macos-latest
            build: mac
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2

      # The audio system of the machine gives the headers. The binary
      # connects to the libasound of the user at the time it runs.
      - name: Install the audio system headers
        if: matrix.build == 'zig'
        run: sudo apt-get update && sudo apt-get install -y libasound2-dev

      - name: Install cargo-zigbuild
        if: matrix.build == 'zig'
        run: |
          pip install ziglang
          cargo install cargo-zigbuild --locked

      - name: Build for Linux
        if: matrix.build == 'zig'
        run: cargo zigbuild --release --target ${{ matrix.target }}.2.17

      - name: Build for macOS
        if: matrix.build == 'mac'
        run: |
          rustup target add aarch64-apple-darwin x86_64-apple-darwin
          cargo build --release --target aarch64-apple-darwin
          cargo build --release --target x86_64-apple-darwin
          mkdir -p target/${{ matrix.target }}/release
          lipo -create -output target/${{ matrix.target }}/release/abstui \
            target/aarch64-apple-darwin/release/abstui \
            target/x86_64-apple-darwin/release/abstui

      - name: Make the archive
        run: |
          tar -czf abstui-${{ matrix.target }}.tar.gz \
            -C target/${{ matrix.target }}/release abstui

      - uses: actions/upload-artifact@v4
        with:
          name: abstui-${{ matrix.target }}
          path: abstui-${{ matrix.target }}.tar.gz

  publish:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/download-artifact@v4
        with:
          path: dist
          merge-multiple: true

      - name: Add the other files
        run: |
          cp config.example.toml dist/
          cp linux/abstui.desktop dist/

      # The machine writes the sums. No person writes a sum in a file.
      - name: Write SHA256SUMS
        working-directory: dist
        run: sha256sum abstui-*.tar.gz > SHA256SUMS

      - uses: actions/attest-build-provenance@v2
        with:
          subject-path: dist/abstui-*.tar.gz

      - name: Publish the release
        uses: softprops/action-gh-release@v2
        with:
          draft: false
          files: dist/*
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

- [ ] **Step 2: Verify the syntax of the workflow**

Run: `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml'))"`
Expected: no output.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: make the release from a tag on main

A tag with the form v* makes the release. The workflow builds three
archives, it writes SHA256SUMS with the machine, and it publishes the
release. The workflow before this one made a draft, and thus the
address /releases/latest gave nothing.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 4: Test the workflow with a tag that you remove after**

The workflow cannot be tested without a tag. Push a tag for the test, look at
the result, and then remove the tag and the release:

```bash
git tag v0.5.0-beta-rc1
git push origin v0.5.0-beta-rc1
gh run watch
gh release view v0.5.0-beta-rc1
```

Expected: three archives, `SHA256SUMS`, `config.example.toml`, and
`abstui.desktop`. The release is not a draft.

Then verify that the binary of Linux runs on an old system:

```bash
gh release download v0.5.0-beta-rc1 -p 'abstui-x86_64-unknown-linux-gnu.tar.gz' -D /tmp/rc
tar -xzf /tmp/rc/abstui-x86_64-unknown-linux-gnu.tar.gz -C /tmp/rc
podman run --rm -v /tmp/rc:/rc:ro debian:bullseye /rc/abstui --version
```

Expected: the version. Debian bullseye has glibc 2.31, and therefore this test
proves that the floor operates.

Then remove the tag and the release:

```bash
gh release delete v0.5.0-beta-rc1 --yes
git push --delete origin v0.5.0-beta-rc1
git tag -d v0.5.0-beta-rc1
```

---

### Task 6: The program asks the API of the fork

**Files:**
- Modify: `src/utils/check_update.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `pub const RELEASES_API: &str` — the address of the API of the releases of the fork. Task 7 uses it.

- [ ] **Step 1: Change the address**

In `src/utils/check_update.rs`, add this after the line `use reqwest::Client;`:

```rust
/// The address that gives the last release of the fork.
///
/// The program before the fork asked `AlbanDAVID/Toutui`, and that repository
/// is archived. See T-21.
pub const RELEASES_API: &str = "https://api.github.com/repos/ealtun21/abstui/releases/latest";
```

Then replace the address in `get_latest_release_gh`:

```rust
        .get(RELEASES_API)
        .header(USER_AGENT, "abstui-updater")
```

- [ ] **Step 2: Change the message that the user reads**

In the same file, replace the text of the message:

```rust
                Some(format!(
                    "🔄 Version {} is available. Run `abstui --update`.",
                    latest_version_gh
                ))
```

- [ ] **Step 3: Run the whole gate**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clippy gives no output. Every test passes.

- [ ] **Step 4: Commit**

```bash
git add src/utils/check_update.rs
git commit -m "fix: ask the API of the fork for the last release

The program asked the archived repository. Therefore the program never
saw a release of the fork.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 7: The program finds the archive of its own target

The program reads the answer of the API and finds the archive for the system
that it runs on. This task holds no download.

**Files:**
- Create: `src/update/mod.rs`
- Create: `src/update/release.rs`
- Create: `tests/update.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `check_update::RELEASES_API` from Task 6.
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

use abstui::update::release::{parse_release, target, Release};

/// Gives an answer of the API with the assets of a release.
fn answer(tag: &str) -> String {
    format!(
        r#"{{
            "tag_name": "{tag}",
            "assets": [
                {{"name": "SHA256SUMS",
                  "browser_download_url": "https://example.test/{tag}/SHA256SUMS"}},
                {{"name": "abstui-x86_64-unknown-linux-gnu.tar.gz",
                  "browser_download_url": "https://example.test/{tag}/abstui-x86_64-unknown-linux-gnu.tar.gz"}},
                {{"name": "abstui-universal-apple-darwin.tar.gz",
                  "browser_download_url": "https://example.test/{tag}/abstui-universal-apple-darwin.tar.gz"}}
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
            archive_name: "abstui-x86_64-unknown-linux-gnu.tar.gz".to_string(),
            archive_url: "https://example.test/v0.6.0-beta/abstui-x86_64-unknown-linux-gnu.tar.gz"
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
Expected: FAIL. The compiler says that `abstui::update` does not exist.

- [ ] **Step 3: Write the module**

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

    let archive_name = format!("abstui-{}.tar.gz", target);

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
        .header(reqwest::header::USER_AGENT, "abstui-updater")
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
`src/update/mod.rs` compiles. Task 8 writes the code:

```rust
//! The installation of a new binary.
```

In `src/lib.rs`, add `pub mod update;` after `pub mod ui;`.

- [ ] **Step 4: Run the tests to see that they pass**

Run: `cargo test --test update`
Expected: PASS, four tests.

- [ ] **Step 5: Run the whole gate**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clippy gives no output. Every test passes.

- [ ] **Step 6: Commit**

```bash
git add src/update/ src/lib.rs tests/update.rs
git commit -m "feat: find the archive of the target in a release

The program reads the answer of the API of GitHub and finds the
archive for the system that it runs on.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 8: The program installs the new binary

The program compares the sum, opens the archive, and moves the new binary on
to the old binary with one operation.

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/update/install.rs`
- Modify: `tests/update.rs`

**Interfaces:**
- Consumes: `Release` from Task 7.
- Produces:
  - `pub fn sum_of(bytes: &[u8]) -> String`
  - `pub fn expected_sum(sums: &str, name: &str) -> Option<String>`
  - `pub fn binary_from_archive(bytes: &[u8]) -> Result<Vec<u8>, String>`
  - `pub fn can_replace(binary: &Path) -> bool`
  - `pub fn replace_binary(binary: &Path, contents: &[u8]) -> std::io::Result<()>`
  - `pub async fn run_update(api: &str) -> Result<String, String>`

- [ ] **Step 1: Add the three dependencies**

In `Cargo.toml`, add these lines to `[dependencies]`:

```toml
# The update compares the sum of the archive and opens the archive. The three
# crates below are pure Rust, therefore the build needs no C toolchain.
sha2 = "0.10"
flate2 = { version = "1", default-features = false, features = ["rust_backend"] }
tar = "0.4"
```

Run: `cargo tree -i cc` and confirm that `sha2`, `flate2`, and `tar` are not in
the result.

- [ ] **Step 2: Write the failing tests**

Add to `tests/update.rs`:

```rust
use abstui::update::install::{binary_from_archive, expected_sum, replace_binary, sum_of};
use std::io::Write;

/// Makes a `tar.gz` that holds one file with the name `abstui`.
fn archive_of(contents: &[u8]) -> Vec<u8> {
    let mut tar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    tar.append_data(&mut header, "abstui", contents).unwrap();
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
    let sums = "aaa  abstui-x86_64-unknown-linux-gnu.tar.gz\n\
                bbb  abstui-universal-apple-darwin.tar.gz\n";

    assert_eq!(
        expected_sum(sums, "abstui-universal-apple-darwin.tar.gz"),
        Some("bbb".to_string())
    );
    assert_eq!(expected_sum(sums, "abstui-aarch64-unknown-linux-gnu.tar.gz"), None);
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
    let binary = dir.path().join("abstui");
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
    use abstui::update::install::can_replace;
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("abstui");
    std::fs::write(&binary, b"the old binary").unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();

    let result = can_replace(&binary);

    // The permissions come back, so that the temporary directory goes away.
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();

    assert!(!result);
}
```

Add `flate2`, `tar`, and `sha2` to `[dev-dependencies]` only if the test file
cannot see them. A crate in `[dependencies]` is available to the integration
tests already.

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
        if path.file_name().and_then(|name| name.to_str()) == Some("abstui") {
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents).map_err(|e| e.to_string())?;
            return Ok(contents);
        }
    }

    Err("The archive holds no file with the name abstui.".to_string())
}

/// Gives `true` if the program can write in the directory of the binary.
///
/// A move needs permission on the directory and not on the file. Therefore
/// the test makes a file in that directory.
pub fn can_replace(binary: &Path) -> bool {
    let Some(dir) = binary.parent() else {
        return false;
    };
    // `is_ok` and not a `match`. Clippy stops on `match ... Ok => true`.
    tempfile::Builder::new()
        .prefix(".abstui-")
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
        .prefix(".abstui-new-")
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
        .header(reqwest::header::USER_AGENT, "abstui-updater")
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
        "This system has no archive. Use `cargo install abstui`.".to_string()
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

`can_replace` and `replace_binary` use `tempfile`. Move `tempfile` from
`[dev-dependencies]` to `[dependencies]` in `Cargo.toml`, and remove the line
from `[dev-dependencies]`.

- [ ] **Step 5: Run the tests to see that they pass**

Run: `cargo test --test update`
Expected: PASS, ten tests.

- [ ] **Step 6: Write the test of the full update**

Add to `tests/update.rs`:

```rust
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The program stops and does not touch the binary when the sum disagrees.
#[tokio::test]
async fn a_sum_that_disagrees_stops_the_update() {
    let server = MockServer::start().await;
    let target = abstui::update::release::target().unwrap();
    let name = format!("abstui-{}.tar.gz", target);

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

    let error = abstui::update::install::run_update(&format!("{}/latest", server.uri()))
        .await
        .unwrap_err();

    assert!(error.contains("not correct"));
}
```

The test uses the real binary of the test as `current_exe`, and it stops
before `replace_binary`, therefore it changes no file.

- [ ] **Step 7: Run the tests to see that they pass**

Run: `cargo test --test update`
Expected: PASS, eleven tests.

- [ ] **Step 8: Run the whole gate**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clippy gives no output. Every test passes.

Then confirm that no new dependency compiles C:

Run: `cargo tree -i openssl-sys`
Expected: an error that says that the package is not in the tree.

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock src/update/install.rs tests/update.rs
git commit -m "feat: install a new binary in the program

The program compares the sum of the archive before it moves the new
binary. Therefore a download that stops leaves the binary that
operates. The program runs no file that it receives.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 9: The commands `--update` and `--uninstall`

`--update` runs the update. `--uninstall` writes the list of the paths, and it
deletes nothing.

**Files:**
- Modify: `src/utils/clap.rs`

**Interfaces:**
- Consumes: `update::install::run_update` from Task 8, `check_update::RELEASES_API` from Task 6, `paths::config_dir` from Task 1.
- Produces: `pub async fn clap()`. `src/main.rs` must write `clap().await;`.

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
    let config = paths::config_dir();
    println!("The command deletes nothing. Delete these paths to remove abstui:");
    println!();
    println!("    {}", config.display());
    match std::env::current_exe() {
        Ok(binary) => println!("    {}", binary.display()),
        Err(_) => println!("    the binary abstui in your PATH"),
    }
    if cfg!(target_os = "linux") {
        if let Some(home) = dirs::home_dir() {
            println!(
                "    {}",
                home.join(".local/share/applications/abstui.desktop").display()
            );
        }
    }
    println!();
    println!("The directory of the program before the fork does not change.");
}

pub async fn clap() {
    let matches = Command::new("abstui")
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

- [ ] **Step 2: Change the call in `main.rs`**

In `src/main.rs`, replace `clap();` with:

```rust
    clap().await;
```

- [ ] **Step 3: Verify the two commands by hand**

Run: `cargo run -- --uninstall`
Expected: the list of the paths. No file changes.

Run: `cargo run -- --update`
Expected: a message. The fork has no release before Task 12, and therefore the
message says that the release has no archive for this target, or that the API
gives nothing. This is correct at this point.

- [ ] **Step 4: Run the whole gate**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clippy gives no output. Every test passes.

- [ ] **Step 5: Commit**

```bash
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

### Task 10: The short script of installation

`hello_toutui.sh` has 1080 lines, and it installs VLC and netcat that the
player in the program does not need. Its array of sums needs a person to write
each sum. Its verification of itself at line 15 gives two variables that no
code sets, and therefore that verification never operated.

**Files:**
- Create: `install.sh`
- Delete: `hello_toutui.sh`
- Modify: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: the assets of Task 5.
- Produces: `install.sh` at the root of the repository.

- [ ] **Step 1: Write the script**

Create `install.sh`:

```bash
#!/usr/bin/env bash
# Install abstui.
#
# Use:
#   curl -LsSf https://raw.githubusercontent.com/ealtun21/abstui/main/install.sh | bash
#
# The script receives the archive of the last release, it compares the sum
# with SHA256SUMS of that release, and it installs the binary. The script
# installs no other program, because abstui plays the audio itself.

set -euo pipefail

REPO="ealtun21/abstui"
API="https://api.github.com/repos/${REPO}/releases/latest"
BIN_DIR="${ABSTUI_BIN_DIR:-/usr/local/bin}"

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
                *) fail "Linux $arch has no archive. Use: cargo install abstui" ;;
            esac
            ;;
        *) fail "$os has no archive. Use: cargo install abstui" ;;
    esac
}

config_dir() {
    if [ -n "${XDG_CONFIG_HOME:-}" ]; then
        echo "${XDG_CONFIG_HOME}/abstui"
    elif [ "$(uname -s)" = "Darwin" ]; then
        echo "${HOME}/Library/Preferences/abstui"
    else
        echo "${HOME}/.config/abstui"
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
    archive="abstui-${target}.tar.gz"

    tag=$(curl -sSfL "$API" | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
    [ -n "$tag" ] || fail "The repository has no release."

    echo "[INFO] The last release is ${tag}."

    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT

    local base="https://github.com/${REPO}/releases/download/${tag}"
    curl -sSfL "${base}/${archive}"   -o "${tmp}/${archive}"
    curl -sSfL "${base}/SHA256SUMS"   -o "${tmp}/SHA256SUMS"

    local expected actual
    expected=$(grep " ${archive}\$" "${tmp}/SHA256SUMS" | awk '{print $1}')
    [ -n "$expected" ] || fail "SHA256SUMS has no sum for ${archive}."
    actual=$(sum_of "${tmp}/${archive}")
    [ "$expected" = "$actual" ] || fail "The sum of ${archive} is not correct."

    echo "[INFO] The sum is correct."

    tar -xzf "${tmp}/${archive}" -C "$tmp"
    sudo install -m 755 "${tmp}/abstui" "${BIN_DIR}/abstui"
    echo "[INFO] The binary is in ${BIN_DIR}/abstui."

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
        echo "ABSTUI_SECRET_KEY=${key}" > "${config}/.env"
        chmod 600 "${config}/.env"
        echo "[INFO] The secret key is in ${config}/.env."
    fi

    if [ "$(uname -s)" = "Linux" ]; then
        mkdir -p "${HOME}/.local/share/applications"
        curl -sSfL "${base}/abstui.desktop" \
            -o "${HOME}/.local/share/applications/abstui.desktop"
    fi

    echo "[DONE] Type abstui to start the program."
}

main "$@"
```

The script makes the secret key with `/dev/urandom`. The script before this
one asked the user for the key, and a user who typed a short key had weak
encryption.

- [ ] **Step 2: Verify the script**

Run: `bash -n install.sh`
Expected: no output.

Run: `shellcheck install.sh` if `shellcheck` is present.
Expected: no error. A warning about `sudo` in a pipe is acceptable.

- [ ] **Step 3: Delete the old script**

```bash
git rm hello_toutui.sh
```

Then remove the line `cp ./hello_toutui.sh dist/` from
`.github/workflows/release.yml` if that line is still present after Task 5.
Task 5 removed it already, therefore verify with:

```bash
grep -n "hello_toutui" .github/workflows/release.yml
```

Expected: no output.

- [ ] **Step 4: Commit**

```bash
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

### Task 11: The files of the registries

**Files:**
- Create: `packaging/aur/PKGBUILD`
- Create: `packaging/aur/.SRCINFO`
- Create: `packaging/README.md`
- Modify: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: the assets of Task 5.
- Produces: the package `abstui-bin` of the AUR, and the step that publishes the crate.

- [ ] **Step 1: Write the PKGBUILD**

Create `packaging/aur/PKGBUILD`:

```bash
# Maintainer: ealtun21 <https://github.com/ealtun21>
pkgname=abstui-bin
pkgver=0.5.0.beta
pkgrel=1
pkgdesc="A TUI client of Audiobookshelf for Linux and macOS"
arch=('x86_64' 'aarch64')
url="https://github.com/ealtun21/abstui"
license=('GPL-3.0-or-later')
depends=('alsa-lib' 'gcc-libs')
provides=('abstui')
conflicts=('abstui')
options=('!debug')

_tag="v0.5.0-beta"
source_x86_64=("abstui-${_tag}-x86_64.tar.gz::${url}/releases/download/${_tag}/abstui-x86_64-unknown-linux-gnu.tar.gz")
source_aarch64=("abstui-${_tag}-aarch64.tar.gz::${url}/releases/download/${_tag}/abstui-aarch64-unknown-linux-gnu.tar.gz")
source=("LICENSE-${_tag}::${url}/raw/${_tag}/LICENSE"
        "abstui.desktop::${url}/releases/download/${_tag}/abstui.desktop")

# The sums come from SHA256SUMS of the release. Copy them at each new version.
sha256sums_x86_64=('SKIP')
sha256sums_aarch64=('SKIP')
sha256sums=('SKIP' 'SKIP')

package() {
    install -Dm755 "${srcdir}/abstui" "${pkgdir}/usr/bin/abstui"
    install -Dm644 "${srcdir}/LICENSE-${_tag}" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
    install -Dm644 "${srcdir}/abstui.desktop" "${pkgdir}/usr/share/applications/abstui.desktop"
}
```

The AUR does not accept `SKIP` for a source that comes from `https`. Replace
each `SKIP` with the sum from `SHA256SUMS` of the release before you send the
package. `packaging/README.md` in step 3 holds this instruction.

- [ ] **Step 2: Make the `.SRCINFO`**

The file `.SRCINFO` comes from the `PKGBUILD`. Make it on a machine that has
`pacman`:

```bash
cd packaging/aur && makepkg --printsrcinfo > .SRCINFO
```

If the machine has no `pacman`, leave `.SRCINFO` absent and write that in
`packaging/README.md`. The AUR needs the file, and `makepkg` is the only
correct way to make it.

- [ ] **Step 3: Write the instructions of the packaging**

Create `packaging/README.md`:

````markdown
# The packages of abstui

## The AUR

The package is `abstui-bin`. `AlbDav` keeps `toutui-bin` and `toutui-git`,
and those packages name the archived project.

Do this at each new version:

1. Change `pkgver` and `_tag` in `packaging/aur/PKGBUILD`.
2. Copy the sums from `SHA256SUMS` of the release into `sha256sums_x86_64`,
   `sha256sums_aarch64`, and `sha256sums`. The AUR does not accept `SKIP`.
3. Run `makepkg --printsrcinfo > .SRCINFO`.
4. Run `makepkg -si` and confirm that the package installs.
5. Send the package:

```bash
git clone ssh://aur@aur.archlinux.org/abstui-bin.git
cp PKGBUILD .SRCINFO abstui-bin/
cd abstui-bin && git commit -am "abstui-bin 0.5.0-beta" && git push
```

This needs an account of the AUR and a key SSH.

## crates.io

The workflow publishes the crate at each tag. The workflow needs the secret
`CARGO_REGISTRY_TOKEN` in the settings of the repository.

Make the token at https://crates.io/settings/tokens with the scope
`publish-update` only, and add it at
`https://github.com/ealtun21/abstui/settings/secrets/actions`.

crates.io does not delete a version. It only marks a version with `yank`.
````

- [ ] **Step 4: Add the step that publishes the crate**

In `.github/workflows/release.yml`, add this job at the end:

```yaml
  crate:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - name: Install the audio system headers
        run: sudo apt-get update && sudo apt-get install -y libasound2-dev

      # The job does nothing if the secret is absent. Therefore the release
      # does not fail before the maintainer makes the token.
      - name: Publish the crate
        env:
          TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          if [ -z "$TOKEN" ]; then
            echo "CARGO_REGISTRY_TOKEN is absent. The job publishes nothing."
            exit 0
          fi
          cargo publish --token "$TOKEN"
```

- [ ] **Step 5: Verify that the crate can be published**

Run: `cargo publish --dry-run`
Expected: the command passes. If the command says that a field is absent, add
that field to `[package]` in `Cargo.toml`.

- [ ] **Step 6: Verify the syntax of the workflow**

Run: `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml'))"`
Expected: no output.

- [ ] **Step 7: Commit**

```bash
git add packaging/ .github/workflows/release.yml
git commit -m "feat: add the files of the AUR and the publication of the crate

The package of the AUR is abstui-bin, because AlbDav keeps toutui-bin
and toutui-git. The workflow publishes the crate at each tag, and it
does nothing if the secret is absent.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 12: The documents and the first release

**Files:**
- Modify: `README.md`
- Modify: `docs/TAKEOVER-BACKLOG.md`
- Modify: `known_bugs.md`
- Modify: `src/utils/changelog.rs`
- Modify: `CONTRIBUTING.md`
- Modify: `.github/ISSUE_TEMPLATE/bug.yml`
- Modify: `.github/ISSUE_TEMPLATE/install_issue.yml`
- Modify: `.github/pull_request_template.md`

**Interfaces:**
- Consumes: every task before it.
- Produces: the release `v0.5.0-beta`.

- [ ] **Step 1: Change the instructions of installation in the README**

Remove the block `> [!WARNING]` that says that the fork has no release.
Replace the part that tells how to install with:

````markdown
## Installation

### The script

```bash
curl -LsSf https://raw.githubusercontent.com/ealtun21/abstui/main/install.sh | bash
```

The script receives the archive of the last release, it compares the sum with
`SHA256SUMS`, and it installs the binary in `/usr/local/bin`.

### Arch Linux

```bash
yay -S abstui-bin
```

### From the source

```bash
cargo install abstui
```

Alpine and every other system without glibc must use this method.

### The update

```bash
abstui --update
```

The program receives the archive of its target, it compares the sum, and it
moves the new binary. The program runs no file that it receives.

### The removal

```bash
abstui --uninstall
```

The command writes the paths. It deletes nothing.
````

- [ ] **Step 2: Change the addresses in the other documents**

```bash
sed -i 's|ealtun21/Toutui|ealtun21/abstui|g' \
    CONTRIBUTING.md known_bugs.md docs/TAKEOVER-BACKLOG.md \
    .github/ISSUE_TEMPLATE/bug.yml .github/ISSUE_TEMPLATE/install_issue.yml \
    .github/pull_request_template.md
```

Then look at each file and change `Toutui` to `AbsTui` where the name is the
name of this program. Keep `Toutui` where the text names the original project.

- [ ] **Step 3: Mark T-21 and T-14 as done in the backlog**

In `docs/TAKEOVER-BACKLOG.md`, change the section `### T-21` to say what the
fork does now:

```markdown
### T-21: `--update` installed the archived original project

`src/utils/clap.rs` ran the install script of the original repository, and
every address in that script names `AlbanDAVID/Toutui`. Therefore the command
built and installed the original program, and the user lost every correction
of the fork.

**The correction.** The project changed its name to `abstui`, and it makes its
own releases from a tag on `main`. `--update` receives the archive of its
target, it compares the sum with `SHA256SUMS`, and it moves the new binary.
The program runs no file that it receives. `--uninstall` writes the paths and
deletes nothing.

T-14 closes with this work, because the program copies the old directory of
configuration one time.
```

Then move T-14 from the list of open faults to the list of closed faults.

- [ ] **Step 4: Add the entry of the changelog**

In `src/utils/changelog.rs`, add a new entry above `changelog_14` and push it
first in the list:

```rust
let changelog_15 = format!(
    "Changelog AbsTui v{} (08/10/2026) \n\
     \n\
     Warning:\n\
     - The project has a new name. The program copies ~/.config/toutui/ to\n\
       ~/.config/abstui/ one time. The old directory does not change.\n\
     \n\
     Added:\n\
     - The program plays a local copy when the server does not answer, and\n\
       it sends the positions when the server answers again.\n\
     - The program updates itself with `abstui --update`. The program\n\
       compares the sum of the archive before it moves the new binary.\n\
     - The releases come from this repository, and each archive has a proof\n\
       of its origin.\n\
     \n\
     Fixed:\n\
     - `--update` installed the archived original project.\n\
     - The program lost the configuration after an update.\n\
     - `Mark as finished` did not always operate.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy!\n\
     ####\n",
     VERSION
);
```

Every line ends with `\n\`. A line that ends with `\` only joins to the next
line without a break, and a line that ends with no `\` puts the indentation in
the text. The entries before this one have that fault, and the screen shows
their indentation.

Then add `changelog.push_str(&changelog_15);` above the line
`changelog.push_str(&changelog_14);`.

- [ ] **Step 5: Run the whole gate**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clippy gives no output. Every test passes.

- [ ] **Step 6: Confirm that no address names the archived repository**

Run:

```bash
grep -rn "AlbanDAVID/Toutui" --include="*.rs" --include="*.sh" --include="*.yml" --include="*.toml" . | grep -v "^./target"
```

Expected: no result. Every remaining name of `AlbanDAVID/Toutui` must be in a
document, and it must name the original project.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "docs: tell how to install, to update, and to remove abstui

The README gives the script, the AUR, and cargo install. The backlog
marks T-21 and T-14 as done, and the changelog holds the entry of
v0.5.0-beta.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 8: Make the first release**

The maintainer must do these three things before this step:

1. Change the name of the repository on GitHub to `abstui`.
2. Make the token of crates.io and add `CARGO_REGISTRY_TOKEN` to the secrets.
3. Make an account of the AUR and add a key SSH.

Then:

```bash
git push
git tag v0.5.0-beta
git push origin v0.5.0-beta
gh run watch
```

Expected: the release holds three archives, `SHA256SUMS`,
`config.example.toml`, and `abstui.desktop`. The release is not a draft.

- [ ] **Step 9: Write the sums in the PKGBUILD**

Task 11 wrote `SKIP` in each field of sums, because the sums come from the
release and the release did not exist. The AUR does not accept `SKIP` for a
source that comes from `https`.

```bash
gh release download v0.5.0-beta -p 'SHA256SUMS' -D /tmp/rel
cat /tmp/rel/SHA256SUMS
sha256sum <(gh release download v0.5.0-beta -p 'abstui.desktop' -O -)
```

Put the sum of `abstui-x86_64-unknown-linux-gnu.tar.gz` in
`sha256sums_x86_64`, the sum of `abstui-aarch64-unknown-linux-gnu.tar.gz` in
`sha256sums_aarch64`, and the sums of `LICENSE` and `abstui.desktop` in
`sha256sums`, in the sequence of the array `source`.

Then make the `.SRCINFO` and commit:

```bash
cd packaging/aur && makepkg --printsrcinfo > .SRCINFO && cd ../..
git add packaging/aur
git commit -m "chore: write the sums of v0.5.0-beta in the PKGBUILD

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 10: Stop the branch `stable`**

The releases come from the tags on `main`. The branch `stable` came from the
original project, and no address names it now.

First confirm that no address names it:

```bash
grep -rn "branch stable\|/stable/\|--branch stable" --include="*.sh" --include="*.md" --include="*.yml" --include="*.rs" . | grep -v "^./target"
```

Expected: no result.

Then change the default branch on GitHub to `main` if it is not `main`, and
delete the branch:

```bash
gh repo edit ealtun21/abstui --default-branch main
git push origin --delete stable
```

- [ ] **Step 11: Verify the update from end to end**

Install version `0.5.0-beta` from the script, then make a tag `v0.5.1-beta`
and confirm that `--update` moves to it:

```bash
bash install.sh
abstui --version
abstui --update
abstui --version
```

Expected: the second `--version` gives the new version.

---

## Notes on the sequence

Tasks 1, 2, and 3 give the copy of the configuration. They must come before
Task 4. If the name changes first, a user who updates loses the token and the
database.

Tasks 5 to 9 need Task 4, because the names of the assets hold the name
`abstui`.

Task 12 step 8 needs the maintainer. Every step before it needs no person with
special permissions.
