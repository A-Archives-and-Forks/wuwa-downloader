#!/bin/bash

set -e

BIN_NAME="wuwa-downloader"
DIST_DIR="dist"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function info() {
    echo -e "${CYAN}==> $1${NC}"
}

function success() {
    echo -e "${GREEN}✔ $1${NC}"
}

function warn() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

function error() {
    echo -e "${RED}✖ $1${NC}"
}

function build_linux() {
    local OUT_DIR="target/release"
    local PACKAGE_NAME="${BIN_NAME}-linux-x86_64"
    local PACKAGE_DIR="${DIST_DIR}/${PACKAGE_NAME}"
    local ARCHIVE_NAME="${PACKAGE_NAME}.tar.gz"

    clear

    info "Cleaning binaries to rebuild..."
    cargo clean

    info "Building release binary for Linux..."
    cargo build --release
    clear
    success "Linux build finished"

    info "Creating package directory..."
    rm -rf "$PACKAGE_DIR"
    mkdir -p "$PACKAGE_DIR"
    success "Package directory ready: $PACKAGE_DIR"

    info "Copying binary..."
    cp "$OUT_DIR/$BIN_NAME" "$PACKAGE_DIR/"
    success "Copied binary to package directory"

    info "Creating archive..."
    cd "$DIST_DIR"
    tar -czf "$ARCHIVE_NAME" "$PACKAGE_NAME"
    cd -
    success "Archive created: ${DIST_DIR}/${ARCHIVE_NAME}"
}

function build_windows() {
    local TARGET="x86_64-pc-windows-gnu"
    local OUT_DIR="target/${TARGET}/release"
    local PACKAGE_NAME="${BIN_NAME}-windows-x86_64"
    local PACKAGE_DIR="${DIST_DIR}/${PACKAGE_NAME}"
    local ARCHIVE_NAME="${PACKAGE_NAME}.zip"

    clear

    info "Cleaning binaries to rebuild..."
    cargo clean

    info "Building release binary for Windows..."
    cargo build --release --target "$TARGET"
    clear
    success "Windows build finished"

    info "Creating package directory..."
    rm -rf "$PACKAGE_DIR"
    mkdir -p "$PACKAGE_DIR"
    success "Package directory ready: $PACKAGE_DIR"

    info "Copying binary..."
    cp "$OUT_DIR/${BIN_NAME}.exe" "$PACKAGE_DIR/"
    success "Copied binary to package directory"

    info "Creating archive..."
    cd "$DIST_DIR"
    zip -r "$ARCHIVE_NAME" "$PACKAGE_NAME"
    cd -
    success "Archive created: ${DIST_DIR}/${ARCHIVE_NAME}"
}

if [[ $# -ne 1 ]]; then
    error "Usage: $0 [linux|windows]"
    exit 1
fi

case "$1" in
    linux)
        build_linux
        ;;
    windows)
        build_windows
        ;;
    *)
        error "Unknown target: $1"
        error "Usage: $0 [linux|windows]"
        exit 1
        ;;
esac
