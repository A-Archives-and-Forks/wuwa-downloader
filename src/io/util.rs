use std::time::Duration;
use colored::Colorize;
use reqwest::blocking::Client;
use serde_json::Value;

use crate::config::{cfg::Config, status::Status};

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

pub fn calculate_total_size(resources: &[Value], client: &Client, config: &Config) -> u64 {
    let mut total_size = 0;
    let mut failed_urls = 0;

    println!("{} Processing files...", Status::info());
    
    for (i, item) in resources.iter().enumerate() {
        if let Some(dest) = item.get("dest").and_then(Value::as_str) {
            let mut file_size = 0;
            let mut found_valid_url = false;
            
            for base_url in &config.zip_bases {
                let url = format!("{}/{}", base_url, dest);
                match client.head(&url).send() {
                    Ok(response) => {
                        if let Some(len) = response.headers().get("content-length") {
                            if let Ok(len_str) = len.to_str() {
                                if let Ok(len_num) = len_str.parse::<u64>() {
                                    file_size = len_num;
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

            if found_valid_url {
                total_size += file_size;
            } else {
                failed_urls += 1;
                println!("{} Could not determine size for file: {}", Status::error(), dest);
            }
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
        "{} Total download size: {}",
        Status::info(),
        bytes_to_human(total_size).cyan()
    );

    total_size
}

pub fn get_version(data: &Value, category: &str, version: &str) -> Result<String, String> {
    data[category][version]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing {} URL", version))
}