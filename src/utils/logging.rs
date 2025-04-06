use log::{error, info};
use std::fs::{OpenOptions};
use std::io::Write;
use chrono::Local;
use std::path::Path;

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
// pub fn log_warning(message: &str) {
//     warn!("{}", message);
//     if let Err(e) = write_to_log_file(format!("WARNING: {}", message)) {
//         eprintln!("Failed to write to log file: {}", e);
//     }
// }

/// Logs a debug message and writes it to a log file.
// pub fn log_debug(message: &str) {
//     debug!("{}", message);
//     if let Err(e) = write_to_log_file(format!("DEBUG: {}", message)) {
//         eprintln!("Failed to write to log file: {}", e);
//     }
// }

/// Writes a message to a log file.
fn write_to_log_file(message: String) -> Result<(), std::io::Error> {
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(log_dir) {
            eprintln!("Failed to create log directory: {}", e);
            return Err(e);
        }
    }

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_dir.join("app.log"))?;

    writeln!(file, "{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), message)?;
    file.flush()?; // Ensure data is written immediately
    Ok(())
}
