use clap::{Arg, Command};

/// The address of the fork. The user reads it in the message below.
const FORK_URL: &str = "https://github.com/ealtun21/abstui";

/// The message that `--update` and `--uninstall` give.
///
/// The script of the original project installs the original project. Every
/// address in that script names `AlbanDAVID/Toutui`, and that repository is
/// archived. Therefore the command took away this fork and put the original
/// program in its place. The user then lost the corrections of the fork, and
/// the token came back into the list of processes.
///
/// The commands do nothing now. A command that takes away a correction of
/// security is worse than a command that does nothing. See T-21.
fn explain(action: &str) {
    eprintln!("The command --{} does nothing now.", action);
    eprintln!();
    eprintln!("That command ran a script of the original project. The original");
    eprintln!("project is archived, and every address in that script names it.");
    eprintln!("Therefore the command took away this fork and put the original");
    eprintln!("program in its place. The token then came back into the list of");
    eprintln!("processes.");
    eprintln!();
    eprintln!("To get a new version, use the repository of the fork:");
    eprintln!("    {}", FORK_URL);
}

pub fn clap() {
    let matches = Command::new("abstui")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("update")
                .long("update")
                .help("Not available. It installed the archived original project.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .help("Not available. It ran a script of the archived original project.")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("uninstall") {
        explain("uninstall");
        std::process::exit(1);
    }

    if matches.get_flag("update") {
        explain("update");
        std::process::exit(1);
    }
}
