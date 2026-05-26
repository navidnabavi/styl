#!/usr/bin/env bash
set -euo pipefail

REPO="navidnabavi/styl"
BIN_NAME="styl"
INSTALL_DIR="/usr/local/bin"

# Detect OS and arch
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    case "$ARCH" in
      arm64)  TARGET="aarch64-apple-darwin" ;;
      x86_64) TARGET="x86_64-apple-darwin" ;;
      *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
      *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

# Get version (default to latest if not provided)
VERSION="${1:-}"
if [ -z "$VERSION" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
fi

if [ -z "$VERSION" ]; then
  echo "Failed to fetch latest version" >&2
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${VERSION}/${BIN_NAME}-${VERSION}-${TARGET}"

echo "Installing ${BIN_NAME} ${VERSION} (${TARGET})..."

TMP="$(mktemp)"
curl -fsSL "$URL" -o "$TMP"
chmod +x "$TMP"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP" "${INSTALL_DIR}/${BIN_NAME}"
else
  sudo mv "$TMP" "${INSTALL_DIR}/${BIN_NAME}"
fi

echo "Installed to ${INSTALL_DIR}/${BIN_NAME}"
"${INSTALL_DIR}/${BIN_NAME}" --version
