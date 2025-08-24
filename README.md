# 🌊 Wuthering Waves Downloader

[![Rust nightly](https://img.shields.io/badge/Rust-1.87.0--nightly-orange?logo=rust)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)

High-performance, resilient downloader for Wuthering Waves with multi-CDN fallback, integrity verification, and a clean TUI experience.

[✨ Features](#-features) •
[📦 Requirements](#-requirements) •
[🛠️ Installation](#️-installation) •
[▶️ Usage](#️-usage) •
[🔍 Technical Details](#-technical-details) •
[⚙️ Configuration](#️-configuration) •
[❓ FAQ](#-faq) •
[🧪 Development](#-development) •
[🤝 Contributing](#-contributing)

![Ferris](https://i.ibb.co/QVThVkd/Ferris.png)

</div>

## ✨ Features
- **Multi-CDN fallback**: Automatically tries multiple mirrors on failures
- **Interactive version selection**: Choose Live/Beta and OS/CN variants
- **Integrity checks**: Per-file MD5 verification; corrupted files are removed
- **Smart retries**: Up to 3 retry attempts per CDN with robust timeouts
- **Streaming downloads**: Chunked I/O for low memory usage
- **Clear progress**: Per-file progress bars with speed, ETA, totals
- **Graceful interrupt**: CTRL-C to stop safely with a final summary
- **Detailed logs**: Errors recorded with timestamps in `logs.log`

## 📦 Requirements
- **Rust nightly toolchain**: 1.87.0-nightly or newer
- **Windows**: Full console experience
- **Linux**: Fully supported

## 🛠️ Installation
```bash
rustup toolchain install nightly
rustup default nightly

git clone https://github.com/yuhkix/wuwa-downloader.git
cd wuwa-downloader

cargo build --release
```

## 🌟 Key Features

### 🚀 Download Management
- **Multi-CDN Fallback** - Automatically tries all available CDN mirrors

- **Version Selection** - Interactive menu for Live/Beta versions

- **Verified Downloads** - MD5 checksum validation for every file

- **Smart Retry Logic** - 3 retry attempts per CDN with timeout protection

- **GZIP Support** - Handles compressed responses efficiently

### 🛡️ Reliability
- **Atomic Operations** - Thread-safe progress tracking

- **Graceful Interrupt** - CTRL-C handling with summary display

- **Comprehensive Logging** - Detailed error logging with timestamps

- **Validation Failures** - Auto-removes files with checksum mismatches

### 📂 File Management
- **Smart Path Handling** - Cross-platform path support

- **Auto-directory Creation** - Builds full directory trees as needed

- **Clean Failed Downloads** - Removes corrupted files automatically

### 💻 User Interface
- **Color-coded Output** - Clear visual feedback (success/warning/error)

- **Dynamic Title Updates** - Real-time progress in window title

- **Clean Progress Display** - Simplified download status without clutter

- **Formatted Duration** - Clear elapsed time display (HH:MM:SS)

### ⚙️ Technical Details
- **Streaming Downloads** - Memory-efficient chunked transfers

- **HEAD Request Verification** - Pre-checks file availability

- **Multi-threaded** - Safe concurrent progress tracking

- **Configurable Timeouts** - 30s for metadata, 10000s for downloads
