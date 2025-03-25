use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant, SystemTime},
};

use colored::*;
use flate2::read::GzDecoder;
use md5::{Digest, Md5};
use reqwest::{StatusCode, blocking::Client};
use serde_json::{Value, from_reader, from_str};
use winconsole::console::{clear, set_title};

const INDEX_URL: &str = "https://gist.githubusercontent.com/yuhkix/b8796681ac2cd3bab11b7e8cdc022254/raw/30a8e747debe9e333d5f4ec5d8700dab500594a2/wuwa.json";
const MAX_RETRIES: usize = 3;
const DOWNLOAD_TIMEOUT: u64 = 300; // minutes in seconds

struct Status;

impl Status {
    fn info() -> ColoredString {
        "[*]".cyan()
    }
    fn success() -> ColoredString {
        "[+]".green()
    }
    fn warning() -> ColoredString {
        "[!]".yellow()
    }
    fn error() -> ColoredString {
        "[-]".red()
    }
    fn question() -> ColoredString {
        "[?]".blue()
    }
    fn progress() -> ColoredString {
        "[→]".magenta()
    }
}

#[derive(Clone)]
struct DownloadConfig {
    index_url: String,
    zip_bases: Vec<String>,
}

fn setup_logging() -> fs::File {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs.log")
        .expect("Failed to create/open log file");

    log_file
}

fn log_error(mut log_file: &fs::File, message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    writeln!(log_file, "[{}] ERROR: {}", timestamp, message).expect("Failed to write to log file");
}

fn fetch_gist(client: &Client) -> Result<String, String> {

    let mut response = client
        .get(INDEX_URL)
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server error: HTTP {}", response.status()));
    }

    let content_encoding = response
        .headers()
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let gist_data: Value = if content_encoding.contains("gzip") {
        let mut buffer = Vec::new();
        response
            .copy_to(&mut buffer)
            .map_err(|e| format!("Error reading response bytes: {}", e))?;

        let mut gz = GzDecoder::new(&buffer[..]);
        let mut decompressed = String::new();
        gz.read_to_string(&mut decompressed)
            .map_err(|e| format!("Error decompressing content: {}", e))?;

        from_str(&decompressed).map_err(|e| format!("Invalid JSON: {}", e))?
    } else {
        from_reader(response).map_err(|e| format!("Invalid JSON: {}", e))?
    };

    println!("{} Available versions:", Status::info());
    println!("1. Preload - OS");
    println!("2. Live - CN (Needs Update)");
    println!("3. Beta - OS (Needs Update)");
    println!("4. Beta - CN (Needs Update)");

    loop {
        print!("{} Select version to download: ", Status::question());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                return gist_data["live"]["os-live"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or("Missing os-live URL".to_string());
            }
            "2" => {
                return gist_data["live"]["cn-live"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or("Missing cn-live URL".to_string());
            }
            "3" => {
                return gist_data["beta"]["os-beta"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or("Missing os-beta URL".to_string());
            }
            "4" => {
                return gist_data["beta"]["cn-beta"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or("Missing cn-beta URL".to_string());
            }
            _ => println!("{} Invalid selection, please try again", Status::error()),
        }
    }
}

fn get_predownload(client: &Client) -> Result<DownloadConfig, String> {
    let selected_index_url = fetch_gist(client)?;
    println!("{} Fetching download configuration...", Status::info());

    let mut response = client
        .get(&selected_index_url)
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server error: HTTP {}", response.status()));
    }

    let content_encoding = response
        .headers()
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let config: Value = if content_encoding.contains("gzip") {
        let mut buffer = Vec::new();
        response
            .copy_to(&mut buffer)
            .map_err(|e| format!("Error reading response bytes: {}", e))?;

        let mut gz = GzDecoder::new(&buffer[..]);
        let mut decompressed = String::new();
        gz.read_to_string(&mut decompressed)
            .map_err(|e| format!("Error decompressing content: {}", e))?;

        from_str(&decompressed).map_err(|e| format!("Invalid JSON: {}", e))?
    } else {
        from_reader(response).map_err(|e| format!("Invalid JSON: {}", e))?
    };

    let predownload_config = config
        .get("predownload")
        .and_then(|p| p.get("config"))
        .ok_or("Missing predownload.config in response")?;

    let base_url = predownload_config
        .get("baseUrl")
        .and_then(Value::as_str)
        .ok_or("Missing or invalid baseUrl")?;

    let index_file = predownload_config
        .get("indexFile")
        .and_then(Value::as_str)
        .ok_or("Missing or invalid indexFile")?;

    let default_config = config
        .get("default")
        .ok_or("Missing default config in response")?;

    let cdn_list = default_config
        .get("cdnList")
        .and_then(Value::as_array)
        .ok_or("Missing or invalid cdnList")?;

    let mut cdn_urls = Vec::new();
    for cdn in cdn_list {
        if let Some(url) = cdn.get("url").and_then(Value::as_str) {
            cdn_urls.push(url.trim_end_matches('/').to_string());
        }
    }

    if cdn_urls.is_empty() {
        return Err("No valid CDN URLs found".to_string());
    }

    let full_index_url = format!("{}/{}", cdn_urls[0], index_file.trim_start_matches('/'));
    let zip_bases = cdn_urls
        .iter()
        .map(|cdn| format!("{}/{}", cdn, base_url.trim_start_matches('/')))
        .collect();

    Ok(DownloadConfig {
        index_url: full_index_url,
        zip_bases,
    })
}

fn fetch_index(client: &Client, config: &DownloadConfig, log_file: &fs::File) -> Value {
    println!("{} Fetching index file...", Status::info());

    let mut response = match client
        .get(&config.index_url)
        .timeout(Duration::from_secs(30))
        .send()
    {
        Ok(resp) => resp,
        Err(e) => {
            log_error(log_file, &format!("Error fetching index file: {}", e));
            clear().unwrap();
            println!("{} Error fetching index file: {}", Status::error(), e);
            println!("\n{} Press Enter to exit...", Status::warning());
            let _ = io::stdin().read_line(&mut String::new());
            std::process::exit(1);
        }
    };

    if !response.status().is_success() {
        let msg = format!("Error fetching index file: HTTP {}", response.status());
        log_error(log_file, &msg);
        clear().unwrap();
        println!("{} {}", Status::error(), msg);
        println!("\n{} Press Enter to exit...", Status::warning());
        let _ = io::stdin().read_line(&mut String::new());
        std::process::exit(1);
    }

    let content_encoding = response
        .headers()
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let text = if content_encoding.contains("gzip") {
        let mut buffer = Vec::new();
        if let Err(e) = response.copy_to(&mut buffer) {
            log_error(log_file, &format!("Error reading index file bytes: {}", e));
            clear().unwrap();
            println!("{} Error reading index file: {}", Status::error(), e);
            println!("\n{} Press Enter to exit...", Status::warning());
            let _ = io::stdin().read_line(&mut String::new());
            std::process::exit(1);
        }

        let mut gz = GzDecoder::new(&buffer[..]);
        let mut decompressed_text = String::new();
        if let Err(e) = gz.read_to_string(&mut decompressed_text) {
            log_error(log_file, &format!("Error decompressing index file: {}", e));
            clear().unwrap();
            println!("{} Error decompressing index file: {}", Status::error(), e);
            println!("\n{} Press Enter to exit...", Status::warning());
            let _ = io::stdin().read_line(&mut String::new());
            std::process::exit(1);
        }
        decompressed_text
    } else {
        match response.text() {
            Ok(t) => t,
            Err(e) => {
                log_error(
                    log_file,
                    &format!("Error reading index file response: {}", e),
                );
                clear().unwrap();
                println!("{} Error reading index file: {}", Status::error(), e);
                println!("\n{} Press Enter to exit...", Status::warning());
                let _ = io::stdin().read_line(&mut String::new());
                std::process::exit(1);
            }
        }
    };

    println!("{} Index file downloaded successfully", Status::success());

    match from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            log_error(log_file, &format!("Error parsing index file JSON: {}", e));
            clear().unwrap();
            println!("{} Error parsing index file: {}", Status::error(), e);
            println!("\n{} Press Enter to exit...", Status::warning());
            let _ = io::stdin().read_line(&mut String::new());
            std::process::exit(1);
        }
    }
}

fn get_dir() -> PathBuf {
    loop {
        print!(
            "{} Enter download directory (Enter for current): ",
            Status::question()
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let path = input.trim();

        let path = if path.is_empty() {
            std::env::current_dir().unwrap()
        } else {
            PathBuf::from(shellexpand::tilde(path).into_owned())
        };

        if path.is_dir() {
            return path;
        }

        print!(
            "{} Directory doesn't exist. Create? (y/n): ",
            Status::warning()
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() == "y" {
            fs::create_dir_all(&path).unwrap();
            return path;
        }
    }
}

fn calculate_md5(path: &Path) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut hasher = Md5::new();
    io::copy(&mut file, &mut hasher).unwrap();
    format!("{:x}", hasher.finalize())
}

fn download_file(
    client: &Client,
    config: &DownloadConfig,
    dest: &str,
    folder: &Path,
    expected_md5: Option<&str>,
    log_file: &fs::File,
) -> bool {
    let dest = dest.replace('\\', "/");
    let path = folder.join(&dest);

    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            log_error(
                log_file,
                &format!("Error creating directory for {}: {}", dest, e),
            );
            println!("{} Error creating directory: {}", Status::error(), e);
            return false;
        }
    }

    for (i, base_url) in config.zip_bases.iter().enumerate() {
        let url = format!("{}{}", base_url, dest);

        let head_response = match client.head(&url).timeout(Duration::from_secs(10)).send() {
            Ok(resp) => resp,
            Err(e) => {
                log_error(
                    log_file,
                    &format!("CDN {} failed for {} - {}", i + 1, dest, e),
                );
                continue;
            }
        };

        if head_response.status() != StatusCode::OK {
            log_error(
                log_file,
                &format!(
                    "CDN {} failed for {} (HTTP {})",
                    i + 1,
                    dest,
                    head_response.status()
                ),
            );
            continue;
        }

        println!("{} Downloading: {}", Status::progress(), dest);

        let mut retries = MAX_RETRIES;
        let mut last_error = None;

        while retries > 0 {
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let response = client
                    .get(&url)
                    .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT))
                    .send()?;

                if !response.status().is_success() {
                    return Err(response.error_for_status().unwrap_err().into());
                }

                let mut file = fs::File::create(&path)?;
                let mut content = response;
                io::copy(&mut content, &mut file)?;

                Ok(())
            })();

            match result {
                Ok(_) => break,
                Err(e) => {
                    last_error = Some(e.to_string());
                    log_error(
                        log_file,
                        &format!(
                            "Download attempt failed for {} ({} retries left): {}",
                            dest,
                            retries - 1,
                            e
                        ),
                    );

                    retries -= 1;
                    let _ = fs::remove_file(&path);

                    if retries > 0 {
                        println!(
                            "{} Retrying download for {}... ({} attempts left)",
                            Status::warning(),
                            dest,
                            retries
                        );
                    }
                }
            }
        }

        if retries == 0 {
            log_error(
                log_file,
                &format!(
                    "Download failed after retries for {}: {}",
                    dest,
                    last_error.unwrap_or_default()
                ),
            );
            println!("{} Download failed: {}", Status::error(), dest.red());
            return false;
        }

        if let Some(expected) = expected_md5 {
            let actual = calculate_md5(&path);
            if actual != expected {
                log_error(
                    log_file,
                    &format!(
                        "Checksum failed for {}: expected {}, got {}",
                        dest, expected, actual
                    ),
                );
                fs::remove_file(&path).unwrap();
                println!("{} Checksum failed: {}", Status::error(), dest.red());
                return false;
            }
        }

        println!(
            "{} {}: {}",
            Status::success(),
            if expected_md5.is_some() {
                "Verified"
            } else {
                "Downloaded"
            },
            dest
        );

        return true;
    }

    log_error(log_file, &format!("All CDNs failed for {}", dest));
    println!("{} All CDNs failed for {}", Status::error(), dest.red());
    false
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn update_title(start_time: Instant, success: usize, total: usize) {
    let elapsed = start_time.elapsed();
    let elapsed_str = format_duration(elapsed);
    let progress = if total > 0 {
        format!(" ({}%)", (success as f32 / total as f32 * 100.0).round())
    } else {
        String::new()
    };

    let title = format!(
        "Wuthering Waves Downloader - Elapsed: {} - {}/{} files{}",
        elapsed_str, success, total, progress
    );

    set_title(&title).unwrap();
}

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
    let success = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let total_files = resources.len();
    let folder_clone = folder.clone();

    let start_time = Instant::now();

    let success_clone = success.clone();
    let title_thread = thread::spawn(move || {
        loop {
            update_title(
                start_time,
                success_clone.load(std::sync::atomic::Ordering::SeqCst),
                total_files,
            );
            thread::sleep(Duration::from_secs(1));
        }
    });

    let success_clone = success.clone();
    let log_file_clone = log_file.try_clone().unwrap();

    ctrlc::set_handler(move || {
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
            folder_clone.display().to_string().cyan()
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
        if let Some(dest) = item.get("dest").and_then(Value::as_str) {
            let md5 = item.get("md5").and_then(Value::as_str);
            if download_file(&client, &config, dest, &folder, md5, &log_file) {
                success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    drop(title_thread);

    print_results(
        success.load(std::sync::atomic::Ordering::SeqCst),
        total_files,
        &folder,
    );
}

fn print_results(success: usize, total: usize, folder: &Path) {
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
