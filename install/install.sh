#!/usr/bin/env bash
# Green Profiler Installer for macOS and Linux
# Usage: curl -fsSL https://greenprofiler.github.io/install/install.sh | bash

set -e

echo "=========================================================="
echo "   Installing Green Profiler CLI (Rust Native Engine)     "
echo "=========================================================="

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux*)
    ASSET_NAME="GreenProfiler-v1.8.2-linux-x64.zip"
    ;;
  Darwin*)
    if [ "$ARCH" = "arm64" ]; then
      ASSET_NAME="GreenProfiler-v1.8.2-darwin-arm64.zip"
    else
      ASSET_NAME="GreenProfiler-v1.8.2-darwin-x64.zip"
    fi
    ;;
  *)
    echo "[ERROR] Unsupported operating system: $OS"
    exit 1
    ;;
esac

INSTALL_DIR="${HOME}/.greenprofiler/bin"
mkdir -p "$INSTALL_DIR"

URL="https://github.com/Haekalss/GreenCode-Profiler/releases/latest/download/${ASSET_NAME}"
TEMP_ZIP="/tmp/greenprofiler.zip"
TEMP_DIR="/tmp/greenprofiler_extract"

echo "-> Downloading ${ASSET_NAME}..."
curl -fsSL "$URL" -o "$TEMP_ZIP"

echo "-> Extracting to ${INSTALL_DIR}..."
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"
unzip -q -o "$TEMP_ZIP" -d "$TEMP_DIR"

BIN_PATH=$(find "$TEMP_DIR" -type f -name "greenprofiler" | head -n 1)
if [ -z "$BIN_PATH" ]; then
  # Fallback check
  BIN_PATH=$(find "$TEMP_DIR" -type f -name "greenprofiler.exe" | head -n 1)
fi

if [ -n "$BIN_PATH" ]; then
  cp "$BIN_PATH" "${INSTALL_DIR}/greenprofiler"
  ln -sf "${INSTALL_DIR}/greenprofiler" "${INSTALL_DIR}/gp" 2>/dev/null || cp "$BIN_PATH" "${INSTALL_DIR}/gp"
  chmod +x "${INSTALL_DIR}/greenprofiler" "${INSTALL_DIR}/gp"
else
  echo "[ERROR] Executable binary not found in downloaded archive."
  exit 1
fi

rm -f "$TEMP_ZIP"
rm -rf "$TEMP_DIR"

# Check shell profile for PATH
SHELL_NAME=$(basename "$SHELL")
PROFILE_FILE=""
if [ "$SHELL_NAME" = "zsh" ]; then
  PROFILE_FILE="$HOME/.zshrc"
elif [ "$SHELL_NAME" = "bash" ]; then
  if [ -f "$HOME/.bashrc" ]; then
    PROFILE_FILE="$HOME/.bashrc"
  elif [ -f "$HOME/.bash_profile" ]; then
    PROFILE_FILE="$HOME/.bash_profile"
  fi
fi

if [ -n "$PROFILE_FILE" ] && ! grep -q ".greenprofiler/bin" "$PROFILE_FILE" 2>/dev/null; then
  echo 'export PATH="$HOME/.greenprofiler/bin:$PATH"' >> "$PROFILE_FILE"
  echo "-> Added ${INSTALL_DIR} to ${PROFILE_FILE}."
fi

echo ""
echo "Green Profiler v1.8.2 successfully installed to ${INSTALL_DIR}!"
echo "Commands available: 'greenprofiler' or 'gp' (alias)"
echo "Website: https://greenprofiler.github.io"
