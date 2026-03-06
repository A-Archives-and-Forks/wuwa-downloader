<div align="center">

<br>

<img src="https://i.ibb.co/4gDjPqF9/wuwa.png" width="128" height="128" alt="Wuthering Waves Logo">

# Wuthering Waves Downloader

[![Rust nightly](https://img.shields.io/badge/Rust-1.87.0--nightly-orange?logo=rust)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)

High-performance, resilient downloader for Wuthering Waves with multi-CDN fallback, integrity verification, and a clean TUI experience.

[✨ Features](#-features) •
[📦 Requirements](#-requirements) •
[🛠️ Installation](#️-installation) •
[▶️ Usage](#️-usage) •
[🔍 Technical Details](#-technical-details) •
[⚙️ Configuration](#️-configuration) •
[📚 Documentation](https://deepwiki.com/yuhkix/wuwa-downloader/) •
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

## ▶️ Usage
### Running the Application
- **Windows**: `target\release\wuwa-downloader.exe`
- **Linux**: `./target/release/wuwa-downloader`

### Workflow
1. Select a version to download (Live/Beta and OS/CN)
2. Choose a download directory or press Enter for current directory
3. Wait for index fetching and size estimation
4. Monitor download progress with progress bars
5. Review final summary and press Enter to exit

## 🔍 Technical Details
### How It Works
- Remote config discovery via JSON
- Index parsing for resource listing
- HEAD request preflight checks
- Range-based downloads with resume capability
- MD5 checksum validation

### Key Components
- `src/network/client.rs`: Config and download management
- `src/io/util.rs`: Progress tracking and formatting
- `src/io/file.rs`: File operations and path handling
- `src/io/logging.rs`: Error logging system
- `src/download/progress.rs`: Progress state management

## ⚙️ Configuration
- **Retry Policy**: 3 attempts per CDN
- **Timeouts**: 30s for metadata, extended for transfers
- **Logging**: 
  - Errors: `logs.log`
  - URLs: `urls.txt` (optional)
- **Progress**: Live window title updates (Windows)

## 📚 Documentation
For detailed guides, workflow overview, and deeper technical explanations, see the [official documentation](https://deepwiki.com/yuhkix/wuwa-downloader/).

## ❓ FAQ
- **Download location?** User-selected at runtime
- **Safe interruption?** Yes, via CTRL-C
- **Why MD5?** Matches upstream checksums for integrity

## 🧪 Development
### Environment Setup
- **Required**: Rust nightly (1.87.0-nightly+)
- **Dependencies**: 
  - `reqwest` (blocking)
  - `indicatif`
  - `flate2`
  - `colored`
  - `ctrlc`
  - `serde_json`

### Build Optimization
Release profile includes:
- Strip symbols
- Link-time optimization
- Maximum optimization level
- Single codegen unit

### Quick Start
```bash
cargo run --release
```

## 🤝 Contributing
Pull requests are welcome. Please ensure:
- Focused changes
- Clear documentation
- Brief motivation explanation

## 📜 License
Licensed under the **MIT License**. See [LICENSE](LICENSE).