use colored::*;
use reqwest::blocking::Client;
use serde_json::Value;
use std::{
    io,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use winconsole::console::{clear, set_title};

use wuwa_downloader::{
    config::status::Status,
    download::progress::DownloadProgress,
    io::{
        console::print_results,
        file::get_dir,
        logging::{
            bytes_to_human, calculate_total_size, format_duration, log_error, setup_logging,
        },
    },
    network::client::{download_file, fetch_index, get_predownload},
};

fn main() {
    set_title("Wuthering Waves Downloader").unwrap();
    let log_file = setup_logging();
    let client = Client::new();

    let config = match get_predownload(&client) {
        Ok(c) => c,
        Err(e) => {
            log_error(&log_file, &e);
            clear().unwrap();
            println!("{} {}", Status::error(), e);
            println!("\n{} Press Enter to exit...", Status::warning());
            let _ = io::stdin().read_line(&mut String::new());
            std::process::exit(1);
        }
    };

    let folder = get_dir();
    clear().unwrap();
    println!(
        "\n{} Download folder: {}\n",
        Status::info(),
        folder.display().to_string().cyan()
    );

    clear().unwrap();
    let data = fetch_index(&client, &config, &log_file);

    let resources = match data.get("resource").and_then(Value::as_array) {
        Some(res) => res,
        None => {
            log_error(&log_file, "No resources found in index file");
            println!("{} No resources found in index file", Status::warning());
            println!("\n{} Press Enter to exit...", Status::warning());
            let _ = io::stdin().read_line(&mut String::new());
            return;
        }
    };

    println!(
        "{} Found {} files to download\n",
        Status::info(),
        resources.len().to_string().cyan()
    );
    let total_size = calculate_total_size(resources, &client, &config);
    clear().unwrap();

    let should_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let success = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let total_files = resources.len();

    let progress = DownloadProgress {
        total_bytes: Arc::new(std::sync::atomic::AtomicU64::new(total_size)),
        downloaded_bytes: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        start_time: Instant::now(),
    };

    let success_clone = success.clone();
    let should_stop_clone = should_stop.clone();
    let progress_clone = progress.clone();
    let title_thread = thread::spawn(move || {
        while !should_stop_clone.load(std::sync::atomic::Ordering::SeqCst) {
            let elapsed = progress_clone.start_time.elapsed();
            let elapsed_secs = elapsed.as_secs();
            let downloaded_bytes = progress_clone
                .downloaded_bytes
                .load(std::sync::atomic::Ordering::SeqCst);
            let total_bytes = progress_clone
                .total_bytes
                .load(std::sync::atomic::Ordering::SeqCst);
            let current_success = success_clone.load(std::sync::atomic::Ordering::SeqCst);

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

            let remaining_files = total_files - current_success;
            let remaining_bytes = total_bytes.saturating_sub(downloaded_bytes);

            let eta_secs = if speed > 0 && remaining_files > 0 {
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
                "Wuthering Waves Downloader - {}/{} files - Current File: {}{} - Speed: {}{} - Total ETA: {}",
                current_success,
                total_files,
                bytes_to_human(downloaded_bytes),
                progress_percent,
                speed_value,
                speed_unit,
                eta_str
            );

            set_title(&title).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    let should_stop_ctrlc = should_stop.clone();

    ctrlc::set_handler(move || {
        should_stop_ctrlc.store(true, std::sync::atomic::Ordering::SeqCst);
        clear().unwrap();
        println!("\n{} Download interrupted by user", Status::warning());
    })
    .unwrap();

    for item in resources.iter() {
        if should_stop.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        if let Some(dest) = item.get("dest").and_then(Value::as_str) {
            let md5 = item.get("md5").and_then(Value::as_str);
            if download_file(
                &client,
                &config,
                dest,
                &folder,
                md5,
                &log_file,
                &should_stop,
                &progress,
            ) {
                success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    should_stop.store(true, std::sync::atomic::Ordering::SeqCst);
    title_thread.join().unwrap();

    clear().unwrap();
    print_results(
        success.load(std::sync::atomic::Ordering::SeqCst),
        total_files,
        &folder,
    );
}
