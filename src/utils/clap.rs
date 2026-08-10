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
use std::path::{Path, PathBuf};

/// Gives the paths that a user can delete to remove the program.
///
/// The list holds the directory of configuration, the binary, the entry of
/// the launcher, and the directory of the data. `binary` is absent when the
/// program cannot find its own path, and `launcher_entry` is absent on a
/// system that has no such entry.
///
/// The fork gives no bundle of an application for macOS, therefore this list
/// is complete on macOS as well. `macos/Info.plist` and `macos/launch.command`
/// described a bundle, and no part of the project made that bundle or
/// installed it: `install.sh` did not write it, `release.yml` did not put it
/// in an archive, and this command could not name it. The two files also
/// disagreed with the installation, because `launch.command` opened
/// `$HOME/.cargo/bin/toutui` and `install.sh` writes `/usr/local/bin/toutui`.
/// Therefore the fork removed the two files. See T-31.
pub fn uninstall_paths(
    config_dir: &Path,
    binary: Option<&Path>,
    launcher_entry: Option<&Path>,
    data_dir: &Path,
) -> Vec<String> {
    let mut list = vec![config_dir.display().to_string()];

    match binary {
        Some(path) => list.push(path.display().to_string()),
        None => list.push("the binary toutui in your PATH".to_string()),
    }

    if let Some(entry) = launcher_entry {
        list.push(entry.display().to_string());
    }

    list.push(data_dir.display().to_string());
    list
}

/// Gives the entry of the launcher that `install.sh` writes.
///
/// The script writes that file on Linux only. Therefore no other system has
/// this path.
fn launcher_entry() -> Option<PathBuf> {
    if cfg!(target_os = "linux") {
        dirs::home_dir().map(|home| home.join(".local/share/applications/toutui.desktop"))
    } else {
        None
    }
}

/// Writes the paths that a user can delete to remove the program.
///
/// The command deletes nothing. The command before this one used `sudo rm -r`
/// on paths that came from variables of the environment, and that is a
/// danger.
fn write_uninstall_message() {
    let binary = std::env::current_exe().ok();
    let entry = launcher_entry();

    println!("The command deletes nothing. Delete these paths to remove toutui:");
    println!();
    for path in uninstall_paths(
        &paths::config_dir(),
        binary.as_deref(),
        entry.as_deref(),
        &paths::data_dir(),
    ) {
        println!("    {}", path);
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

#[cfg(test)]
mod tests {
    use super::uninstall_paths;
    use std::path::Path;

    /// The list on Linux names the four paths that an installation makes.
    #[test]
    fn the_list_on_linux_names_every_path() {
        let list = uninstall_paths(
            Path::new("/home/a/.config/toutui"),
            Some(Path::new("/usr/local/bin/toutui")),
            Some(Path::new(
                "/home/a/.local/share/applications/toutui.desktop",
            )),
            Path::new("/home/a/.local/share/toutui"),
        );

        assert_eq!(
            list,
            vec![
                "/home/a/.config/toutui",
                "/usr/local/bin/toutui",
                "/home/a/.local/share/applications/toutui.desktop",
                "/home/a/.local/share/toutui",
            ]
        );
    }

    /// The list on macOS names every path of that system.
    ///
    /// macOS has no entry of a launcher, because `install.sh` writes that file
    /// on Linux only. The fork gives no bundle of an application, therefore
    /// this list has no gap. This test cannot run on macOS on this machine,
    /// thus it gives the paths of macOS itself. See T-31.
    #[test]
    fn the_list_on_macos_names_every_path_and_no_bundle() {
        let list = uninstall_paths(
            Path::new("/Users/a/Library/Preferences/toutui"),
            Some(Path::new("/usr/local/bin/toutui")),
            None,
            Path::new("/Users/a/.local/share/toutui"),
        );

        assert_eq!(
            list,
            vec![
                "/Users/a/Library/Preferences/toutui",
                "/usr/local/bin/toutui",
                "/Users/a/.local/share/toutui",
            ]
        );
        assert!(!list.iter().any(|path| path.contains(".app")));
    }

    /// A program that cannot find its own path tells the user where to look.
    #[test]
    fn a_binary_that_has_no_path_gives_a_message() {
        let list = uninstall_paths(
            Path::new("/home/a/.config/toutui"),
            None,
            None,
            Path::new("/home/a/.local/share/toutui"),
        );

        assert_eq!(list[1], "the binary toutui in your PATH");
    }
}
