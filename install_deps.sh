#!/bin/bash
set -e

echo "Installing system dependencies via dnf..."
sudo dnf install -y webkit2gtk4.1-devel openssl-devel librsvg2-devel libappindicator-gtk3-devel libxdo-devel gcc gcc-c++ qpdf poppler-utils tesseract libreoffice nodejs

echo "Installing Rust via rustup if not present..."
if ! command -v rustup &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
else
    echo "rustup is already installed."
fi

# Ensure ~/.cargo/bin is in PATH for the script
source "$HOME/.cargo/env"

echo "Ensuring pnpm is installed..."
if ! command -v pnpm &> /dev/null; then
    sudo npm install -g pnpm
else
    echo "pnpm is already installed."
fi

echo "Installing cargo extensions..."
cargo install cargo-deny cargo-audit cargo-nextest cargo-fuzz

echo "All dependencies installed successfully!"
