//! The directories and the files that the program uses.
//!
//! Every module takes its paths from here. Therefore a change of the name of
//! the directory happens in one place only.

use std::path::{Path, PathBuf};

/// The name of the directory of this program.
pub const APP_DIR: &str = "toutui";

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

/// Gives the parent directory of the data of the program.
///
/// `XDG_DATA_HOME` has the highest importance. If that variable is absent or
/// empty, the path is `~/.local/share`.
pub fn data_home() -> PathBuf {
    match std::env::var("XDG_DATA_HOME") {
        Ok(value) if !value.is_empty() => PathBuf::from(value),
        _ => {
            let home = dirs::home_dir().expect("Unable to find the user's home directory");
            home.join(".local").join("share")
        }
    }
}

/// Gives the directory that holds the data of this program.
pub fn data_dir() -> PathBuf {
    data_home().join(APP_DIR)
}

/// Gives the directory that holds the data of this program, in a parent
/// directory. The tests use this function.
pub fn data_dir_in(data_home: &Path) -> PathBuf {
    data_home.join(APP_DIR)
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

