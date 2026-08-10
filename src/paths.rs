//! The directories and the files that the program uses.
//!
//! Every module takes its paths from here. Therefore a change of the name of
//! the directory happens in one place only.

use std::path::{Path, PathBuf};

/// The name of the directory of this program.
pub const APP_DIR: &str = "toutui";

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
    config_dir().join("toutui.log")
}

/// The files that the program copies from the old directory.
const MIGRATED_FILES: [&str; 3] = ["config.toml", ".env", "db.sqlite3"];

/// Changes the name of the secret key on one line of `.env`.
///
/// The function changes the line only when the line starts with
/// `TOUTUI_SECRET_KEY=` or with `export TOUTUI_SECRET_KEY=`, after the
/// leading space of the line. The function keeps every other line exactly as
/// it was. A file that holds a comment, a longer name, or a value with the
/// old name inside it must not change on those lines.
fn rename_secret_key_line(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    if let Some(rest) = trimmed.strip_prefix("TOUTUI_SECRET_KEY=") {
        format!("{indent}TOUTUI_SECRET_KEY={rest}")
    } else if let Some(rest) = trimmed.strip_prefix("export TOUTUI_SECRET_KEY=") {
        format!("{indent}export TOUTUI_SECRET_KEY={rest}")
    } else {
        line.to_string()
    }
}

/// Copies `.env` from the old directory to the new directory.
///
/// The function changes the name of the secret key on the line that holds
/// the key. It keeps every other line, and the final newline of the file,
/// exactly as they were.
///
/// The function gives the new file the same permissions as the old file. A
/// user can set the mode of `.env` to keep other users of the computer from
/// reading the secret key. The copy must keep that protection, because the
/// key decrypts the token of the server.
fn copy_env_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    let text = std::fs::read_to_string(source)?;
    let mut new_text: String = text
        .lines()
        .map(rename_secret_key_line)
        .collect::<Vec<_>>()
        .join("\n");
    if text.ends_with('\n') {
        new_text.push('\n');
    }
    std::fs::write(destination, new_text)?;
    #[cfg(unix)]
    std::fs::set_permissions(destination, std::fs::metadata(source)?.permissions())?;
    Ok(())
}

/// The name of the temporary directory that holds the files during the copy.
const STAGING_DIR: &str = "toutui.partial";

/// Copies the configuration of the program before the fork.
///
/// The function makes the new directory. If the new directory was present
/// before, the function copies nothing, because the user has a configuration
/// already. If the new directory was absent and the old directory is present,
/// the function copies the files and gives `true`.
///
/// The function copies the files. It does not move them, thus the old
/// directory stays complete and the user can use the old program again.
///
/// The function copies the files into a temporary directory first, and it
/// moves that directory into place with one operation at the end. A copy
/// that stops in the middle, for example because the disk becomes full,
/// must not leave a new directory that holds some files and not others. A
/// move inside one file system is one operation, therefore the new
/// directory holds every file or it does not exist. A new directory that
/// does not exist lets the program try the copy again at the next start.
pub fn migrate_old_config(config_home: &Path) -> std::io::Result<bool> {
    let new = config_dir_in(config_home);
    if new.is_dir() {
        return Ok(false);
    }

    let old = old_config_dir_in(config_home);
    if !old.is_dir() {
        std::fs::create_dir_all(&new)?;
        return Ok(false);
    }

    // A start that failed before can leave this directory behind. The
    // directory belongs to the program, and it never holds data of the
    // user, therefore the function deletes it and starts again.
    let staging = config_home.join(STAGING_DIR);
    if staging.is_dir() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;

    let mut copied = false;
    for name in MIGRATED_FILES {
        let source = old.join(name);
        if !source.is_file() {
            continue;
        }
        let destination = staging.join(name);
        if name == ".env" {
            copy_env_file(&source, &destination)?;
        } else {
            std::fs::copy(&source, &destination)?;
        }
        copied = true;
    }

    if copied {
        std::fs::rename(&staging, &new)?;
    } else {
        std::fs::remove_dir_all(&staging)?;
        std::fs::create_dir_all(&new)?;
    }

    Ok(copied)
}

/// Copies the old configuration in the real parent directory.
pub fn migrate_old_config_here() -> std::io::Result<bool> {
    migrate_old_config(&config_home())
}
