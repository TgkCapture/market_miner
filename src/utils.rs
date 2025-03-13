use log::{error, info, warn};
use std::fs::File;
use std::io::Write;
use chrono::Local;

/// Logs an error message and writes it to a log file.
pub fn log_error(message: &str) {
    error!("{}", message);
    if let Err(e) = write_to_log_file(format!("ERROR: {}", message)) {
        eprintln!("Failed to write to log file: {}", e);
    }
}

/// Logs an info message and writes it to a log file.
pub fn log_info(message: &str) {
    info!("{}", message);
    if let Err(e) = write_to_log_file(format!("INFO: {}", message)) {
        eprintln!("Failed to write to log file: {}", e);
    }
}

/// Logs a warning message and writes it to a log file.
pub fn log_warning(message: &str) {
    warn!("{}", message);
    if let Err(e) = write_to_log_file(format!("WARNING: {}", message)) {
        eprintln!("Failed to write to log file: {}", e);
    }
}

/// Writes a message to a log file.
fn write_to_log_file(message: String) -> Result<(), std::io::Error> {
    let log_dir = std::path::Path::new("logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)?;
    }
    let mut file = File::options()
        .append(true)
        .create(true)
        .open(log_dir.join("app.log"))?;

    writeln!(file, "{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), message)?;
    Ok(())
}