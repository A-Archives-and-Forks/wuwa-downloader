# 🌊 Wuthering Waves Downloader

*A high-performance, reliable downloader for Wuthering Waves with verification and graceful error handling*

[![Rust](https://img.shields.io/badge/Rust-1.87.0--nightly-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)

## 🚀 Features

### 🛠️ Core Functionality
- **Verified Downloads** - MD5 checksum validation for every file
- **Batch Processing** - Downloads all game resources sequentially
- **Network Resiliency** - Timeout protection (30s/60s) with retry logic

### 📂 File Management
- **Smart Path Handling** - Cross-platform path support with tilde (~) expansion
- **Auto-directory Creation** - Builds full directory trees as needed
- **Clean Failed Downloads** - Automatically removes corrupted files

### 🌈 User Interface
- **Color-coded Output** - Instant visual feedback (success/warning/error)
- **Progress Tracking** - Real-time counters (`[X/Y]`) for batch downloads  
- **Interactive Prompts** - Guided directory selection with validation

### ⚡ Performance & Safety
- **Streaming Downloads** - Chunked transfers for memory efficiency
- **Atomic Operations** - Thread-safe progress tracking
- **Graceful Interrupt** - CTRL-C handling with summary display

### 🔒 Reliability
- **Pre-flight Checks** - HEAD requests verify availability before download
- **Comprehensive Error Handling** - Network, filesystem, and validation errors
- **Consistent State** - Never leaves partial downloads on failure

## 📦 Requirements

- Rust nightly toolchain (1.87.0-nightly or newer)
- Windows (for full console feature support)

Install the nightly toolchain with:
```bash
rustup toolchain install nightly
rustup default nightly
