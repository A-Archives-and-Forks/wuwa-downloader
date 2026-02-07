use std::{
    fs::{self, OpenOptions},
    io::Write,
    sync::{Arc, Mutex},
    time::SystemTime,
};

pub type SharedLogFile = Arc<Mutex<fs::File>>;

pub fn setup_logging() -> SharedLogFile {
    Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs.log")
            .expect("Failed to create/open log file"),
    ))
}

pub fn log_error(log_file: &SharedLogFile, message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if let Ok(mut file) = log_file.lock() {
        let _ = writeln!(file, "[{}] ERROR: {}", timestamp, message);
    }
}
