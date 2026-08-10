//! The commands of the line of command.
//!
//! `--update` installs the last release of the fork. `--uninstall` writes the
//! list of the paths and the commands, and it deletes nothing.
//!
//! The command before this one ran a script of the original project, and
//! every address in that script names the archived repository. Therefore the
//! command removed the fork and installed the original program. See T-21.

use crate::paths;
use crate::update::install::run_update;
use crate::utils::check_update::RELEASES_API;
use clap::{Arg, Command};
use std::path::{Path, PathBuf};

/// One thing that an installation put on the disk.
///
/// The user reads `what` and `path`, and the user runs `command`. The program
/// runs no command itself.
#[derive(Debug, PartialEq, Eq)]
pub struct Removal {
    /// The name of the thing, in the words of the user.
    pub what: String,
    /// The path on the disk, or a short instruction when the program does not
    /// know the path.
    pub path: String,
    /// The files that the path holds. A directory hides its contents,
    /// therefore the message names them. It is `None` for a file.
    pub holds: Option<String>,
    /// The command that deletes the path. It is `None` when the program does
    /// not know the path, because a command that holds a wrong path is a
    /// danger.
    pub command: Option<String>,
}

/// Puts a path between single quotes, for a shell.
///
/// Single quotes stop the shell from reading a space or a special character.
/// A path can also hold a single quote. The shell has no escape inside single
/// quotes, therefore each quote of the path closes the text, adds one quote,
/// and opens the text again.
fn quote(path: &str) -> String {
    format!("'{}'", path.replace('\'', "'\\''"))
}

/// Tells if a path needs the command `sudo`.
///
/// A path inside the home directory belongs to the user, therefore the user
/// can delete it. Every other path, for example `/usr/local/bin/toutui`,
/// belongs to the system. If the program does not know the home directory, it
/// gives no `sudo`, because a command with `sudo` that the user does not need
/// is a larger danger than a command that the system refuses.
fn needs_root(path: &Path, home: Option<&Path>) -> bool {
    match home {
        Some(home) => !path.starts_with(home),
        None => false,
    }
}

/// Makes the command that deletes one path.
///
/// `-r` is necessary for a directory. `-f` stops a question about a path that
/// is already absent.
fn removal_command(path: &Path, home: Option<&Path>, is_directory: bool) -> String {
    let flags = if is_directory { "-rf" } else { "-f" };
    let root = if needs_root(path, home) { "sudo " } else { "" };
    format!(
        "{}rm {} {}",
        root,
        flags,
        quote(&path.display().to_string())
    )
}

/// Gives every thing that a user can delete to remove the program.
///
/// The list holds the directory of configuration, the binary, the entry of
/// the launcher, and the directory of the data. The directory of
/// configuration holds `config.toml`, `.env`, the database `db.sqlite3`, and
/// the log `toutui.log`. The directory of the data holds the downloads.
/// Therefore two directories name six things, and the message says so.
///
/// `binary` is absent when the program cannot find its own path.
/// `launcher_entry` is absent on a system that has no such entry, and macOS
/// is such a system.
///
/// The fork gives no bundle of an application for macOS, therefore this list
/// is complete on macOS as well. `macos/Info.plist` and `macos/launch.command`
/// described a bundle, and no part of the project made that bundle or
/// installed it: `install.sh` did not write it, `release.yml` did not put it
/// in an archive, and this command could not name it. The two files also
/// disagreed with the installation, because `launch.command` opened
/// `$HOME/.cargo/bin/toutui` and `install.sh` writes `/usr/local/bin/toutui`.
/// Therefore the fork removed the two files. See T-31.
pub fn uninstall_plan(
    config_dir: &Path,
    binary: Option<&Path>,
    launcher_entry: Option<&Path>,
    data_dir: &Path,
    home: Option<&Path>,
) -> Vec<Removal> {
    let mut list = vec![Removal {
        what: "the configuration, the secret key, the database, and the log".to_string(),
        path: config_dir.display().to_string(),
        holds: Some("it holds config.toml, .env, db.sqlite3, and toutui.log".to_string()),
        command: Some(removal_command(config_dir, home, true)),
    }];

    match binary {
        Some(path) => list.push(Removal {
            what: "the binary".to_string(),
            path: path.display().to_string(),
            holds: None,
            command: Some(removal_command(path, home, false)),
        }),
        None => list.push(Removal {
            what: "the binary".to_string(),
            path: "the binary toutui in your PATH. Run: command -v toutui".to_string(),
            holds: None,
            command: None,
        }),
    }

    if let Some(entry) = launcher_entry {
        list.push(Removal {
            what: "the entry of the launcher".to_string(),
            path: entry.display().to_string(),
            holds: None,
            command: Some(removal_command(entry, home, false)),
        });
    }

    list.push(Removal {
        what: "the downloads for the offline mode".to_string(),
        path: data_dir.display().to_string(),
        holds: Some("it holds downloads/<user>".to_string()),
        command: Some(removal_command(data_dir, home, true)),
    });

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

/// Makes the text that `--uninstall` writes.
///
/// The function is pure, therefore a test can read the whole text. The text
/// gives the paths first and the commands after them. The user must see what
/// each path holds before the user deletes it.
fn uninstall_message(plan: &[Removal]) -> String {
    let mut text = String::new();
    text.push_str("The command deletes nothing. It writes the paths and the commands.\n\n");
    text.push_str("Your installation put these paths on the disk:\n\n");

    for item in plan {
        text.push_str(&format!("    {}\n        {}\n", item.what, item.path));
        if let Some(holds) = &item.holds {
            text.push_str(&format!("        {}\n", holds));
        }
    }

    text.push_str("\nRun these commands to delete the paths. Read each command first:\n\n");
    for item in plan {
        match &item.command {
            Some(command) => text.push_str(&format!("    {}\n", command)),
            None => text.push_str("    Find the binary first, then delete it.\n"),
        }
    }

    text.push_str("\nKeep the configuration if you want to install the program again.\n");
    text
}

/// Writes the paths and the commands that remove the program.
///
/// The command deletes nothing. The command before this one used `sudo rm -r`
/// on paths that came from variables of the environment, and that is a
/// danger.
fn write_uninstall_message() {
    let binary = std::env::current_exe().ok();
    let entry = launcher_entry();
    let home = dirs::home_dir();

    let plan = uninstall_plan(
        &paths::config_dir(),
        binary.as_deref(),
        entry.as_deref(),
        &paths::data_dir(),
        home.as_deref(),
    );

    print!("{}", uninstall_message(&plan));
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
                .help("Write the paths and the commands that remove the program.")
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
    use super::{quote, removal_command, uninstall_message, uninstall_plan};
    use std::path::Path;

    /// Gives the paths of the plan, in the sequence of the plan.
    fn paths_of(plan: &[super::Removal]) -> Vec<String> {
        plan.iter().map(|item| item.path.clone()).collect()
    }

    /// Gives the commands of the plan, in the sequence of the plan.
    fn commands_of(plan: &[super::Removal]) -> Vec<String> {
        plan.iter()
            .filter_map(|item| item.command.clone())
            .collect()
    }

    /// The plan on Linux names the four paths that an installation makes.
    #[test]
    fn the_plan_on_linux_names_every_path() {
        let plan = uninstall_plan(
            Path::new("/home/a/.config/toutui"),
            Some(Path::new("/usr/local/bin/toutui")),
            Some(Path::new(
                "/home/a/.local/share/applications/toutui.desktop",
            )),
            Path::new("/home/a/.local/share/toutui"),
            Some(Path::new("/home/a")),
        );

        assert_eq!(
            paths_of(&plan),
            vec![
                "/home/a/.config/toutui",
                "/usr/local/bin/toutui",
                "/home/a/.local/share/applications/toutui.desktop",
                "/home/a/.local/share/toutui",
            ]
        );
    }

    /// The plan on macOS names every path of that system.
    ///
    /// macOS has no entry of a launcher, because `install.sh` writes that file
    /// on Linux only. The fork gives no bundle of an application, therefore
    /// this plan has no gap. This test cannot run on macOS on this machine,
    /// thus it gives the paths of macOS itself. See T-31.
    #[test]
    fn the_plan_on_macos_names_every_path_and_no_bundle() {
        let plan = uninstall_plan(
            Path::new("/Users/a/Library/Preferences/toutui"),
            Some(Path::new("/usr/local/bin/toutui")),
            None,
            Path::new("/Users/a/.local/share/toutui"),
            Some(Path::new("/Users/a")),
        );

        assert_eq!(
            paths_of(&plan),
            vec![
                "/Users/a/Library/Preferences/toutui",
                "/usr/local/bin/toutui",
                "/Users/a/.local/share/toutui",
            ]
        );
        assert!(!plan.iter().any(|item| item.path.contains(".app")));
    }

    /// The commands of macOS delete the same paths, and only the binary needs
    /// the command `sudo`.
    ///
    /// `/usr/local/bin` belongs to the system on macOS as well as on Linux.
    /// Every other path is inside `/Users/a`, and the user owns it.
    #[test]
    fn the_commands_on_macos_use_sudo_for_the_binary_only() {
        let plan = uninstall_plan(
            Path::new("/Users/a/Library/Preferences/toutui"),
            Some(Path::new("/usr/local/bin/toutui")),
            None,
            Path::new("/Users/a/.local/share/toutui"),
            Some(Path::new("/Users/a")),
        );

        assert_eq!(
            commands_of(&plan),
            vec![
                "rm -rf '/Users/a/Library/Preferences/toutui'",
                "sudo rm -f '/usr/local/bin/toutui'",
                "rm -rf '/Users/a/.local/share/toutui'",
            ]
        );
    }

    /// A binary that `cargo install` wrote is inside the home directory, thus
    /// it needs no `sudo`.
    #[test]
    fn a_binary_in_the_home_directory_needs_no_sudo() {
        let command = removal_command(
            Path::new("/Users/a/.cargo/bin/toutui"),
            Some(Path::new("/Users/a")),
            false,
        );

        assert_eq!(command, "rm -f '/Users/a/.cargo/bin/toutui'");
    }

    /// A home directory that the program does not know gives no `sudo`.
    ///
    /// A command with `sudo` that the user does not need is the larger danger.
    #[test]
    fn an_unknown_home_directory_gives_no_sudo() {
        let command = removal_command(Path::new("/usr/local/bin/toutui"), None, false);

        assert_eq!(command, "rm -f '/usr/local/bin/toutui'");
    }

    /// A path with a space stays one word for the shell.
    #[test]
    fn a_path_with_a_space_keeps_its_quotes() {
        assert_eq!(
            quote("/Users/a b/Library/Preferences/toutui"),
            "'/Users/a b/Library/Preferences/toutui'"
        );
    }

    /// A path with a single quote closes the text and opens it again.
    #[test]
    fn a_path_with_a_quote_is_safe() {
        assert_eq!(
            quote("/Users/o'brien/toutui"),
            "'/Users/o'\\''brien/toutui'"
        );
    }

    /// A program that cannot find its own path tells the user where to look,
    /// and it gives no command.
    #[test]
    fn a_binary_that_has_no_path_gives_no_command() {
        let plan = uninstall_plan(
            Path::new("/home/a/.config/toutui"),
            None,
            None,
            Path::new("/home/a/.local/share/toutui"),
            Some(Path::new("/home/a")),
        );

        assert_eq!(
            plan[1].path,
            "the binary toutui in your PATH. Run: command -v toutui"
        );
        assert_eq!(plan[1].command, None);
    }

    /// The message says that the command deletes nothing, and it holds every
    /// path and every command.
    #[test]
    fn the_message_names_every_path_and_every_command() {
        let plan = uninstall_plan(
            Path::new("/Users/a/Library/Preferences/toutui"),
            Some(Path::new("/usr/local/bin/toutui")),
            None,
            Path::new("/Users/a/.local/share/toutui"),
            Some(Path::new("/Users/a")),
        );
        let text = uninstall_message(&plan);

        assert!(text.starts_with("The command deletes nothing."));
        for item in &plan {
            assert!(text.contains(&item.path));
            assert!(text.contains(item.command.as_deref().unwrap()));
        }

        // A directory hides its contents. The user must see that the message
        // names the database, the log, and the downloads as well.
        for name in [
            "config.toml",
            ".env",
            "db.sqlite3",
            "toutui.log",
            "downloads",
        ] {
            assert!(text.contains(name), "the message must name {}", name);
        }
    }

    /// The script of macOS names the same paths as this command.
    ///
    /// A user of macOS cannot always run `toutui --uninstall`: the binary can
    /// be absent, or Gatekeeper can stop a binary that a browser received.
    /// `macos/uninstall.sh` answers that condition. The two lists must agree,
    /// therefore this test reads the script. See T-31.
    #[test]
    fn the_script_of_macos_names_the_same_paths() {
        let script =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/macos/uninstall.sh"))
                .expect("macos/uninstall.sh must be in the repository");

        for part in [
            "Library/Preferences/toutui",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            ".local/share/toutui",
            "/usr/local/bin",
            "db.sqlite3",
            "toutui.log",
            "downloads",
        ] {
            assert!(script.contains(part), "the script must name {}", part);
        }

        // The script gives no entry of a launcher, because `install.sh`
        // writes that file on Linux only.
        assert!(!script.contains(".desktop"));
        // The fork gives no bundle of an application. See T-31. A bundle
        // lives in a directory whose name ends with `.app`, and the system
        // keeps such a bundle in `/Applications`.
        assert!(!script.contains(".app/"));
        assert!(!script.contains("/Applications"));
    }

    /// The script of macOS deletes nothing.
    ///
    /// Every command `rm` of that script is inside a line that writes text.
    /// Therefore no line of the script removes a path.
    #[test]
    fn the_script_of_macos_deletes_nothing() {
        let script =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/macos/uninstall.sh"))
                .expect("macos/uninstall.sh must be in the repository");

        for line in script.lines() {
            let line = line.trim_start();
            assert!(
                !line.starts_with("rm ") && !line.starts_with("sudo rm "),
                "this line deletes a path: {}",
                line
            );
        }
    }
}
