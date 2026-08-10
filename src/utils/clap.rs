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
