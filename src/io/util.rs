use colored::Colorize;
use reqwest::Client;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs, io,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;

#[cfg(not(target_os = "windows"))]
use std::process::Command;

#[cfg(windows)]
use winconsole::console::{clear, set_title};

use crate::{
    config::{
        cfg::{Config, DownloadOptions, ResourceItem},
        status::Status,
    },
    download::progress::{DownloadProgress, ProgressDisplay},
    io::{
        file::{file_size, get_filename},
        logging::{SharedLogFile, log_error},
    },
    network::client::{build_download_url, download_file},
};

pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

pub fn bytes_to_human(bytes: u64) -> String {
    match bytes {
        b if b > 1_000_000_000 => format!("{:.2} GB", b as f64 / 1_000_000_000.0),
        b if b > 1_000_000 => format!("{:.2} MB", b as f64 / 1_000_000.0),
        b if b > 1_000 => format!("{:.2} KB", b as f64 / 1_000.0),
        b => format!("{} B", b),
    }
}

fn log_url(url: &str) {
    let sanitized_url = if let Some(index) = url.find("://") {
        let (scheme, rest) = url.split_at(index + 3);
        format!("{}{}", scheme, rest.replace("//", "/"))
    } else {
        url.replace("//", "/")
    };

    if let Ok(mut url_log) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("urls.txt")
    {
        let _ = writeln!(url_log, "{}", sanitized_url);
    }
}

pub fn parse_resources(data: &Value) -> Result<Vec<ResourceItem>, String> {
    let resources = data
        .get("resource")
        .and_then(Value::as_array)
        .ok_or_else(|| "No resources found in index file".to_string())?;

    let mut parsed = Vec::with_capacity(resources.len());
    for item in resources {
        if let Some(dest) = item.get("dest").and_then(Value::as_str) {
            parsed.push(ResourceItem {
                dest: dest.to_string(),
                md5: item
                    .get("md5")
                    .and_then(Value::as_str)
                    .map(|md5| md5.to_string()),
            });
        }
    }

    Ok(parsed)
}

pub fn ask_concurrency() -> DownloadOptions {
    let default_concurrency = DownloadOptions::default().concurrency;

    print!(
        "{} Enter concurrent downloads [default {}]: ",
        Status::question(),
        default_concurrency
    );
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return DownloadOptions {
                concurrency: default_concurrency,
            };
        }

        if let Ok(parsed) = trimmed.parse::<usize>() {
            if parsed > 0 {
                return DownloadOptions {
                    concurrency: parsed,
                };
            }
        }
    }

    println!(
        "{} Invalid value, using default concurrency {}",
        Status::warning(),
        default_concurrency
    );

    DownloadOptions {
        concurrency: default_concurrency,
    }
}

pub async fn calculate_total_size(
    resources: &[ResourceItem],
    client: &Client,
    config: &Config,
    folder: &Path,
) -> (u64, HashMap<String, u64>) {
    let mut total_remaining_size = 0;
    let mut failed_urls = 0;
    let mut size_hints = HashMap::new();

    println!("{} Processing files...", Status::info());

    for (i, item) in resources.iter().enumerate() {
        let mut found_valid_url = false;

        for base_url in &config.zip_bases {
            let url = build_download_url(base_url, &item.dest);
            log_url(&url);

            match client
                .head(&url)
                .timeout(Duration::from_secs(15))
                .send()
                .await
            {
                Ok(response) => {
                    if let Some(len) = response.headers().get("content-length") {
                        if let Ok(len_str) = len.to_str() {
                            if let Ok(total_size) = len_str.parse::<u64>() {
                                let local_path = folder.join(item.dest.replace('\\', "/"));
                                let local_size = file_size(&local_path).await;
                                let remaining = if local_size <= total_size {
                                    total_size - local_size
                                } else {
                                    total_size
                                };

                                size_hints.insert(item.dest.clone(), total_size);
                                total_remaining_size += remaining;
                                found_valid_url = true;
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("{} Failed to HEAD {}: {}", Status::warning(), url, e);
                }
            }
        }

        if !found_valid_url {
            failed_urls += 1;
            println!(
                "{} Could not determine size for file: {}",
                Status::error(),
                item.dest
            );
        }

        if i % 10 == 0 {
            println!(
                "{} Processed {}/{} files...",
                Status::info(),
                i + 1,
                resources.len()
            );
        }
    }

    if failed_urls > 0 {
        println!(
            "{} Warning: Could not determine size for {} files",
            Status::warning(),
            failed_urls
        );
    }

    println!(
        "{} Estimated remaining download size: {}",
        Status::info(),
        bytes_to_human(total_remaining_size).cyan()
    );

    #[cfg(not(target_os = "windows"))]
    Command::new("clear").status().unwrap();
    #[cfg(windows)]
    clear().unwrap();

    (total_remaining_size, size_hints)
}

pub fn get_version(data: &Value, category: &str, version: &str) -> Result<String, String> {
    data[category][version]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing {} URL", version))
}

pub fn exit_with_error(log_file: &SharedLogFile, error: &str) -> ! {
    log_error(log_file, error);

    #[cfg(windows)]
    clear().unwrap();

    println!("{} {}", Status::error(), error);
    println!("\n{} Press Enter to exit...", Status::warning());
    let _ = io::stdin().read_line(&mut String::new());
    std::process::exit(1);
}

pub fn track_progress(
    total_size: u64,
) -> (
    Arc<std::sync::atomic::AtomicBool>,
    Arc<std::sync::atomic::AtomicUsize>,
    DownloadProgress,
) {
    let should_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let success = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let progress = DownloadProgress {
        total_bytes: Arc::new(std::sync::atomic::AtomicU64::new(total_size)),
        downloaded_bytes: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        start_time: Instant::now(),
    };

    (should_stop, success, progress)
}

#[allow(unused_variables)]
pub fn start_title_thread(
    should_stop: Arc<std::sync::atomic::AtomicBool>,
    success: Arc<std::sync::atomic::AtomicUsize>,
    progress: DownloadProgress,
    total_files: usize,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !should_stop.load(std::sync::atomic::Ordering::SeqCst) {
            let elapsed = progress.start_time.elapsed();
            let elapsed_secs = elapsed.as_secs();
            let downloaded_bytes = progress
                .downloaded_bytes
                .load(std::sync::atomic::Ordering::SeqCst);
            let total_bytes = progress
                .total_bytes
                .load(std::sync::atomic::Ordering::SeqCst);
            let current_success = success.load(std::sync::atomic::Ordering::SeqCst);

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
                format!(" ({}%)", downloaded_bytes.saturating_mul(100) / total_bytes)
            } else {
                String::new()
            };

            #[cfg(windows)]
            {
                let title = format!(
                    "Wuthering Waves Downloader - {}/{} files - Downloaded: {}{} - Speed: {}{} - ETA: {}",
                    current_success,
                    total_files,
                    bytes_to_human(downloaded_bytes),
                    progress_percent,
                    speed_value,
                    speed_unit,
                    eta_str
                );
                set_title(&title).unwrap();
            }

            thread::sleep(Duration::from_secs(1));
        }
    })
}

pub fn setup_ctrlc(should_stop: Arc<std::sync::atomic::AtomicBool>) {
    ctrlc::set_handler(move || {
        should_stop.store(true, std::sync::atomic::Ordering::SeqCst);

        #[cfg(windows)]
        clear().unwrap();

        println!("\n{} Download interrupted by user", Status::warning());
    })
    .unwrap();
}

pub async fn download_resources(
    client: Arc<Client>,
    config: Arc<Config>,
    resources: Vec<ResourceItem>,
    size_hints: Arc<HashMap<String, u64>>,
    folder: PathBuf,
    log_file: SharedLogFile,
    should_stop: Arc<std::sync::atomic::AtomicBool>,
    progress: DownloadProgress,
    success: Arc<std::sync::atomic::AtomicUsize>,
    options: DownloadOptions,
) {
    let concurrency = options.concurrency.max(1);
    let total_size = progress
        .total_bytes
        .load(std::sync::atomic::Ordering::SeqCst);

    let display = Arc::new(ProgressDisplay::new(concurrency, total_size));
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();

    for item in resources {
        if should_stop.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        let permit = match semaphore.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => break,
        };

        let slot = display.slot_pool.acquire_slot().await;

        let client = client.clone();
        let config = config.clone();
        let folder = folder.clone();
        let log_file = log_file.clone();
        let should_stop = should_stop.clone();
        let progress = progress.clone();
        let success = success.clone();
        let size_hints = size_hints.clone();
        let display = display.clone();

        let handle = tokio::spawn(async move {
            let task_bar = display.slot_pool.bar(slot);
            let filename = get_filename(&item.dest);
            let expected_size = size_hints.get(&item.dest).copied();

            task_bar.set_message(format!("downloading {}", filename.clone().cyan()));
            task_bar.set_position(0);
            if let Some(size) = expected_size {
                task_bar.set_length(size);
            } else {
                task_bar.set_length(0);
            }

            let ok = download_file(
                &client,
                &config,
                &item.dest,
                &folder,
                item.md5.as_deref(),
                expected_size,
                &log_file,
                &should_stop,
                &progress,
                &display.total_bar,
                &task_bar,
            )
            .await;

            if ok {
                success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                task_bar.set_message(format!("done {}", filename.green()));
            } else if should_stop.load(std::sync::atomic::Ordering::SeqCst) {
                task_bar.set_message(format!("stopped {}", filename.yellow()));
            } else {
                task_bar.set_message(format!("failed {}", filename.red()));
            }

            task_bar.set_position(0);
            task_bar.set_length(0);
            task_bar.set_message("idle");

            display.slot_pool.release_slot(slot).await;
            drop(permit);
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    for slot in 0..display.slot_pool.len() {
        display.slot_pool.bar(slot).finish_with_message("idle");
    }

    if should_stop.load(std::sync::atomic::Ordering::SeqCst) {
        display.total_bar.finish_with_message("stopped");
    } else {
        display.total_bar.finish_with_message(format!(
            "completed {}",
            bytes_to_human(progress.downloaded())
        ));
    }
}
