use log::LevelFilter;
use fern::Dispatch;
use chrono::Local;
use std::fs::OpenOptions;

pub fn setup_logs() -> Result<(), fern::InitError> {

    let log_path = crate::paths::log_file();

    // The directory must be present before the file opens. The program made
    // this directory in another place before, and that place is gone.
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

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
