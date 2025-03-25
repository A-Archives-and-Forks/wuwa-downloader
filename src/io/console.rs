use std::{io, path::Path, time::{Duration, Instant}};
use colored::Colorize;
use winconsole::console::set_title;
use crate::{config::status::Status, download::progress::DownloadProgress};
use super::logging::{bytes_to_human, format_duration};

pub fn print_results(success: usize, total: usize, folder: &Path) {
    let title = if success == total {
        " DOWNLOAD COMPLETE ".on_blue().white().bold()
    } else {
        " PARTIAL DOWNLOAD ".on_blue().white().bold()
    };

    println!("\n{}\n", title);
    println!(
        "{} Successfully downloaded: {}",
        Status::success(),
        success.to_string().green()
    );
    println!(
        "{} Failed downloads: {}",
        Status::error(),
        (total - success).to_string().red()
    );
    println!(
        "{} Files saved to: {}",
        Status::info(),
        folder.display().to_string().cyan()
    );
    println!("\n{} Press Enter to exit...", Status::warning());
    let _ = io::stdin().read_line(&mut String::new());
}

pub fn update_title(
    start_time: Instant,
    success: usize,
    total: usize,
    progress: &DownloadProgress,
) {
    let elapsed = start_time.elapsed();
    let elapsed_secs = elapsed.as_secs();
    let downloaded_bytes = progress.downloaded_bytes.load(std::sync::atomic::Ordering::SeqCst);
    let total_bytes = progress.total_bytes.load(std::sync::atomic::Ordering::SeqCst);

    let speed = if elapsed_secs > 0 {
        downloaded_bytes / elapsed_secs
    } else {
        0
    };

    let (speed_value, speed_unit) = if speed > 1_000_000 {
        (speed / 1_000_000, "MB/s")
    } else {
        (speed / 1_000, "KB/s")
    };

    let remaining_bytes = total_bytes.saturating_sub(downloaded_bytes);
    let eta_secs = if speed > 0 {
        remaining_bytes / speed
    } else {
        0
    };
    let eta_str = format_duration(Duration::from_secs(eta_secs));

    let progress_percent = if total_bytes > 0 {
        format!(" ({}%)", (downloaded_bytes * 100 / total_bytes))
    } else {
        String::new()
    };

    let title = format!(
        "Wuthering Waves Downloader - {}/{} files - {}{} - Speed: {}{} - ETA: {}",
        success,
        total,
        bytes_to_human(downloaded_bytes),
        progress_percent,
        speed_value,
        speed_unit,
        eta_str
    );

    set_title(&title).unwrap();
}