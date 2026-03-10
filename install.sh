#!/bin/sh
set -e

# 0diff installer
# Usage: curl -fsSL https://0diff.dev/install.sh | sh

REPO="zerosuite-inc/0diff"
BINARY="0diff"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
DIM='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m'

info() { printf "${CYAN}>${NC} %s\n" "$1"; }
success() { printf "${GREEN}>${NC} %s\n" "$1"; }
error() { printf "${RED}error:${NC} %s\n" "$1" >&2; exit 1; }

print_success() {
    printf "\n"
    printf "  ${BOLD}${CYAN}0diff${NC} installed successfully!\n"
    printf "\n"
    printf "  ${DIM}Get started:${NC}\n"
    printf "    ${GREEN}\$${NC} cd your-project\n"
    printf "    ${GREEN}\$${NC} 0diff init\n"
    printf "    ${GREEN}\$${NC} 0diff watch\n"
    printf "\n"
    printf "  ${DIM}Docs:${NC} https://github.com/${REPO}\n"
    printf "  ${DIM}Site:${NC} https://0diff.dev\n"
    printf "\n"
}

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS="linux" ;;
        Darwin) OS="darwin" ;;
        *)      error "Unsupported OS: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH="x86_64" ;;
        aarch64|arm64)  ARCH="aarch64" ;;
        *)              error "Unsupported architecture: $ARCH" ;;
    esac

    PLATFORM="${OS}-${ARCH}"
}

# Try to install from GitHub releases
try_binary_install() {
    # Get latest release tag
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | \
        grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/' || true)

    if [ -z "$VERSION" ]; then
        return 1
    fi

    ARCHIVE="${BINARY}-${VERSION}-${PLATFORM}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

    info "Downloading 0diff ${VERSION} for ${PLATFORM}..."

    TMPD=$(mktemp -d)
    trap 'rm -rf "$TMPD"' EXIT

    if ! curl -fsSL "$URL" -o "${TMPD}/${ARCHIVE}" 2>/dev/null; then
        return 1
    fi

    tar -xzf "${TMPD}/${ARCHIVE}" -C "$TMPD"

    if [ -w "$INSTALL_DIR" ]; then
        mv "${TMPD}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    else
        info "Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo mv "${TMPD}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    fi

    chmod +x "${INSTALL_DIR}/${BINARY}"
    success "Installed 0diff ${VERSION} to ${INSTALL_DIR}/${BINARY}"
    return 0
}

# Build from source using cargo
cargo_install() {
    if ! command -v cargo >/dev/null 2>&1; then
        error "cargo not found. Install Rust first: https://rustup.rs"
    fi

    info "Building 0diff from source..."
    info "This may take a minute on first install."
    cargo install --git "https://github.com/${REPO}.git" --bin 0diff
    success "Installed 0diff via cargo"
}

# Main
printf "\n"
printf "  ${BOLD}${CYAN}0diff${NC} installer\n"
printf "  ${DIM}Know who changed what. Even when it's not human.${NC}\n"
printf "\n"

detect_platform

if try_binary_install; then
    print_success
else
    info "No pre-built binary available, building from source..."
    cargo_install
    print_success
fi
