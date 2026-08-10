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
