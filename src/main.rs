use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Duration,
    sync::Arc,
};

use colored::*;
use md5::{Digest, Md5};
use reqwest::{blocking::Client, StatusCode};
use serde_json::{Value, from_str};
use winconsole::console::{clear, set_title};

const INDEX_URL: &str = "https://hw-pcdownload-aws.aki-game.net/launcher/game/G153/2.2.0/onnOqcAkPIKgfEoFdwJcgRzLRNLohWAm/resource/50004/2.2.0/indexFile.json";
const ZIP_BASE: &str = "https://hw-pcdownload-aws.aki-game.net/launcher/game/G153/2.2.0/onnOqcAkPIKgfEoFdwJcgRzLRNLohWAm/zip/";

struct Status;

impl Status {
    fn info() -> ColoredString { "[*]".cyan() }
    fn success() -> ColoredString { "[+]".green() }
    fn warning() -> ColoredString { "[!]".yellow() }
    fn error() -> ColoredString { "[-]".red() }
    fn question() -> ColoredString { "[?]".blue() }
    fn progress() -> ColoredString { "[→]".magenta() }
}

fn get_dir() -> PathBuf {
    loop {
        print!("{} Enter download directory (Enter for current): ", Status::question());
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
        
        print!("{} Directory doesn't exist. Create? (y/n): ", Status::warning());
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        if input.trim().to_lowercase() == "y" {
            fs::create_dir_all(&path).unwrap();
            return path;
        }
    }
}

fn fetch_index(client: &Client) -> Value {
    println!("{} Fetching index file...", Status::info());
    
    let response = client.get(INDEX_URL)
        .timeout(Duration::from_secs(30))
        .send()
        .unwrap_or_else(|e| {
            clear().unwrap();
            println!("{} Error fetching index file: {}", Status::error(), e);
            println!("\n{} Press Enter to exit...", Status::warning());
            let _ = io::stdin().read_line(&mut String::new());
            std::process::exit(1);
        });
    
    if !response.status().is_success() {
        clear().unwrap();
        println!("{} Error fetching index file: HTTP {}", Status::error(), response.status());
        println!("\n{} Press Enter to exit...", Status::warning());
        let _ = io::stdin().read_line(&mut String::new());
        std::process::exit(1);
    }
    
    let text = response.text().unwrap();
    println!("{} Index file downloaded successfully", Status::success());
    from_str(&text).unwrap()
}

fn calculate_md5(path: &Path) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut hasher = Md5::new();
    io::copy(&mut file, &mut hasher).unwrap();
    format!("{:x}", hasher.finalize())
}

fn download_file(client: &Client, dest: &str, folder: &Path, expected_md5: Option<&str>) -> bool {
    let dest = dest.replace('\\', "/");
    let url = format!("{}{}", ZIP_BASE, dest);
    let path = folder.join(&dest);
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    
    // Check if file exists first
    let head_response = match client.head(&url).timeout(Duration::from_secs(10)).send() {
        Ok(resp) => resp,
        Err(_) => {
            println!("{} File not available: {}", Status::warning(), dest.yellow());
            return false;
        }
    };
    
    if head_response.status() != StatusCode::OK {
        println!("{} File not available: {}", Status::warning(), dest.yellow());
        return false;
    }
    
    println!("{} Downloading: {}", Status::progress(), dest);
    
    let response = match client.get(&url)
        .timeout(Duration::from_secs(60))
        .send() {
            Ok(resp) => resp,
            Err(e) => {
                clear().unwrap();
                println!("{} Download failed: {} - {}", Status::error(), dest.red(), e);
                return false;
            }
        };
    
    if !response.status().is_success() {
        clear().unwrap();
        println!("{} Download failed: {} - HTTP {}", Status::error(), dest.red(), response.status());
        return false;
    }
    
    let mut file = fs::File::create(&path).unwrap();
    let mut content = response;
    io::copy(&mut content, &mut file).unwrap();
    
    if let Some(expected) = expected_md5 {
        let actual = calculate_md5(&path);
        if actual != expected {
            fs::remove_file(&path).unwrap();
            println!("{} Checksum failed: {}", Status::error(), dest.red());
            return false;
        }
    }
    
    println!("{} {}: {}", 
        Status::success(), 
        if expected_md5.is_some() { "Verified" } else { "Downloaded" }, 
        dest
    );
    
    true
}

fn main() {
    set_title("Wuthering Waves Downloader").unwrap();
    clear().unwrap();
    
    let client = Client::new();
    let folder = get_dir();
    println!("\n{} Download folder: {}\n", Status::info(), folder.display().to_string().cyan());
    
    let data = fetch_index(&client);
    let resources = match data.get("resource").and_then(Value::as_array) {
        Some(res) => res,
        None => {
            println!("{} No resources found in index file", Status::warning());
            println!("\n{} Press Enter to exit...", Status::warning());
            let _ = io::stdin().read_line(&mut String::new());
            return;
        }
    };
    
    println!("{} Found {} files to download\n", Status::info(), resources.len().to_string().cyan());
    let success = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let total_files = resources.len();
    let folder_clone = folder.clone();
    
    let success_clone = success.clone();
    ctrlc::set_handler(move || {
        clear().unwrap();
        println!("{} Download interrupted by user", Status::warning());
        let success_count = success_clone.load(std::sync::atomic::Ordering::SeqCst);
        print_results(success_count, total_files, &folder_clone);
        std::process::exit(0);
    }).unwrap();
    
    for (i, item) in resources.iter().enumerate() {
        if let Some(dest) = item.get("dest").and_then(Value::as_str) {
            print!("{} ", format!("[{}/{}]", i+1, resources.len()).magenta());
            let md5 = item.get("md5").and_then(Value::as_str);
            if download_file(&client, dest, &folder, md5) {
                success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
    
    print_results(success.load(std::sync::atomic::Ordering::SeqCst), total_files, &folder);
}

fn print_results(success: usize, total: usize, folder: &Path) {
    let title = if success == total {
        " DOWNLOAD COMPLETE ".on_blue().white().bold()
    } else {
        " PARTIAL DOWNLOAD ".on_blue().white().bold()
    };
    
    println!("\n{}\n", title);
    println!("{} Successfully downloaded: {}", Status::success(), success.to_string().green());
    println!("{} Failed downloads: {}", Status::error(), (total - success).to_string().red());
    println!("{} Files saved to: {}", Status::info(), folder.display().to_string().cyan());
    println!("\n{} Press Enter to exit...", Status::warning());
    let _ = io::stdin().read_line(&mut String::new());
}