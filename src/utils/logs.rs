use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use std::fs::OpenOptions;
use std::path::Path;

/// Makes the directory that holds a file.
///
/// The program makes its directory of configuration here, at the start,
/// before the file of the log opens. A new user has no such directory yet,
/// and the program would stop with a panic on the file of the log if this
/// function did not make it first.
pub fn make_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn setup_logs() -> Result<(), fern::InitError> {
    let log_path = crate::paths::log_file();

    // The directory must be present before the file opens.
    make_parent_dir(&log_path)?;

    // Create or append into the file
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path) // path and name
        .unwrap();

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Info)
        .chain(log_file) // redirect logs to the file
        .apply()?;

    Ok(())
}
