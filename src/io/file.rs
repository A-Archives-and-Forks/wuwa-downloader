use md5::{Digest, Md5};
use std::{
    fs,
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
};

use crate::config::status::Status;

fn calculate_md5_sync(path: &Path) -> io::Result<String> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::with_capacity(262_144, file);
    let mut hasher = Md5::new();
    io::copy(&mut reader, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn calculate_md5(path: &Path) -> Result<String, String> {
    let path_buf = path.to_path_buf();
    tokio::task::spawn_blocking(move || calculate_md5_sync(&path_buf))
        .await
        .map_err(|e| format!("Failed to join MD5 task: {}", e))?
        .map_err(|e| format!("Failed to calculate MD5: {}", e))
}

pub async fn check_existing_file(
    path: &Path,
    expected_md5: Option<&str>,
    expected_size: Option<u64>,
) -> bool {
    let metadata = match tokio::fs::metadata(path).await {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };

    if let Some(size) = expected_size
        && metadata.len() != size
    {
        return false;
    }

    if let Some(md5) = expected_md5 {
        match calculate_md5(path).await {
            Ok(actual_md5) if actual_md5 == md5 => {}
            _ => return false,
        }
    }

    true
}

pub async fn file_size(path: &Path) -> u64 {
    tokio::fs::metadata(path)
        .await
        .map(|meta| meta.len())
        .unwrap_or(0)
}

pub fn get_filename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

pub fn get_dir() -> PathBuf {
    loop {
        print!(
            "{} Please specify the directory where the game should be downloaded (press Enter to use the current directory): ",
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
            "{} Directory does not exist. Create? (y/n): ",
            Status::warning()
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim().eq_ignore_ascii_case("y") {
            fs::create_dir_all(&path).unwrap();
            return path;
        }
    }
}
