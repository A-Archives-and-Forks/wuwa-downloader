# 🌊 Wuthering Waves Downloader

*A high-performance, reliable downloader for Wuthering Waves with verification and graceful error handling*

[![Rust](https://img.shields.io/badge/Rust-1.87.0--nightly-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)

## 📦 Requirements
- **Rust nightly toolchain** - 1.87.0-nightly or newer
- **Windows** - for full console feature support

Install the nightly toolchain with:
```bash
rustup toolchain install nightly
rustup default nightly
```

### 🛠️ Installation
- **Clone the repository:**
```bash
git clone https://github.com/yourusername/wuthering-waves-downloader.git
cd wuthering-waves-downloader
```
- **Build the project:**
```bash
cargo build --release
```
- **Run the downloader:**
```bash
cargo run --release # (or run the built executable inside target/release/)
```

## 🚀 Features

### 🛠️ Core Functionality
- **Verified Downloads** - MD5 checksum validation for every file
- **Batch Processing** - Downloads all game resources sequentially
- **Network Resiliency** - Timeout protection (30s/60s) with retry logic
- **HEAD Request Verification** - Pre-checks file availability before download
- **Automatic Retries** - Configurable retry attempts for failed downloads

### 📂 File Management
- **Smart Path Handling** - Cross-platform path support with tilde (~) expansion
- **Auto-directory Creation** - Builds full directory trees as needed
- **Clean Failed Downloads** - Automatically removes corrupted files
- **Comprehensive Logging** - Detailed error logging with timestamps

### 💻 User Interface
- **Color-coded Output** - Instant visual feedback (success/warning/error)
- **Progress Tracking** - Real-time counters (`[X/Y]`) for batch downloads  
- **Interactive Prompts** - Guided directory selection with validation
- **Dynamic Title Updates** - Real-time progress in window title
- **Formatted Duration Display** - Clear elapsed time tracking (HH:MM:SS)

### ⚡ Performance & Safety
- **Streaming Downloads** - Chunked transfers for memory efficiency
- **Atomic Operations** - Thread-safe progress tracking
- **Graceful Interrupt** - CTRL-C handling with summary display
- **Memory Efficiency** - Minimal allocations during downloads

### 🔒 Reliability
- **Pre-flight Checks** - HEAD requests verify availability before download
- **Comprehensive Error Handling** - Network, filesystem, and validation errors
- **Consistent State** - Never leaves partial downloads on failure
- **Validation Failures** - Auto-removes files with checksum mismatches