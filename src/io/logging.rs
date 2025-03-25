use std::{fs::{self, OpenOptions}, io::Write, time::{Duration, SystemTime}};

pub fn setup_logging() -> fs::File {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs.log")
        .expect("Failed to create/open log file")
}

pub fn log_error(mut log_file: &fs::File, message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    writeln!(log_file, "[{}] ERROR: {}", timestamp, message).unwrap();
}

pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}

pub fn bytes_to_human(bytes: u64) -> String {
    match bytes {
        b if b > 1_000_000_000 => format!("{:.2} GB", b as f64 / 1_000_000_000.0),
        b if b > 1_000_000 => format!("{:.2} MB", b as f64 / 1_000_000.0),
        b if b > 1_000 => format!("{:.2} KB", b as f64 / 1_000.0),
        b => format!("{} B", b),
    }
}