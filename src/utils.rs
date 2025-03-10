use log::{error, info, warn};
use std::fs::File;
use std::io::Write;
use serde::Serialize;

/// Logs an error message and writes it to a log file.
pub fn log_error(message: &str) {
    error!("{}", message);
    write_to_log_file(format!("ERROR: {}", message));
}

/// Logs an info message and writes it to a log file.
pub fn log_info(message: &str) {
    info!("{}", message);
    write_to_log_file(format!("INFO: {}", message));
}

/// Logs a warning message and writes it to a log file.
pub fn log_warning(message: &str) {
    warn!("{}", message);
    write_to_log_file(format!("WARNING: {}", message));
}

/// Writes a message to a log file.
fn write_to_log_file(message: String) {
    let log_dir = std::path::Path::new("logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir).unwrap_or_else(|_| panic!("Failed to create log directory"));
    }
    let mut file = File::options()
        .append(true)
        .create(true)
        .open(log_dir.join("app.log"))
        .unwrap_or_else(|_| panic!("Failed to open log file"));

    writeln!(file, "{}", message).unwrap_or_else(|_| panic!("Failed to write to log file"));
}

/// Saves data to a JSON file.
pub fn save_to_json<T: Serialize>(data: &T, file_path: &str) {
    let json = serde_json::to_string_pretty(data).unwrap_or_else(|_| panic!("Failed to serialize data to JSON"));
    let mut file = File::create(file_path).unwrap_or_else(|_| panic!("Failed to create file: {}", file_path));
    file.write_all(json.as_bytes()).unwrap_or_else(|_| panic!("Failed to write data to file: {}", file_path));
    log_info(&format!("Data saved to {}", file_path));
}