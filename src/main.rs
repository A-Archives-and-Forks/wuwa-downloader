use colored::*;
use reqwest::blocking::Client;
use serde_json::Value;
use std::{sync::Arc, time::{Duration, Instant}, io, thread};
use winconsole::console::clear;

use wuwa_downloader::{
    config::status::Status,
    download::progress::DownloadProgress,
    io::{console::{print_results, update_title}, file::get_dir, logging::{log_error, setup_logging}},
    network::client::{download_file, fetch_index, get_predownload},
};

fn main() {
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

    let should_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let success = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let total_files = resources.len();

    let progress = DownloadProgress {
        total_bytes: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        downloaded_bytes: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        start_time: Instant::now(),
    };

    let success_clone = success.clone();
    let should_stop_clone = should_stop.clone();
    let progress_clone = progress.clone();
    let title_thread = thread::spawn(move || {
        while !should_stop_clone.load(std::sync::atomic::Ordering::SeqCst) {
            update_title(
                progress_clone.start_time,
                success_clone.load(std::sync::atomic::Ordering::SeqCst),
                total_files,
                &progress_clone,
            );
            thread::sleep(Duration::from_secs(1));
        }
    });

    let success_clone = success.clone();
    let should_stop_clone = should_stop.clone();
    let log_file_clone = log_file.try_clone().unwrap();
    let folder_clone2 = folder.clone();

    ctrlc::set_handler(move || {
        should_stop_clone.store(true, std::sync::atomic::Ordering::SeqCst);

        clear().unwrap();
        println!("{} Download interrupted by user", Status::warning());
        let success_count = success_clone.load(std::sync::atomic::Ordering::SeqCst);

        let title = if success_count == total_files {
            " DOWNLOAD COMPLETE ".on_blue().white().bold()
        } else {
            " PARTIAL DOWNLOAD ".on_blue().white().bold()
        };

        println!("\n{}\n", title);
        println!(
            "{} Successfully downloaded: {}",
            Status::success(),
            success_count.to_string().green()
        );
        println!(
            "{} Failed downloads: {}",
            Status::error(),
            (total_files - success_count).to_string().red()
        );
        println!(
            "{} Files saved to: {}",
            Status::info(),
            folder_clone2.display().to_string().cyan()
        );
        println!("\n{} Press Enter to exit...", Status::warning());

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        log_error(
            &log_file_clone,
            &format!(
                "Download interrupted by user. Success: {}/{}",
                success_count, total_files
            ),
        );
        std::process::exit(0);
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
