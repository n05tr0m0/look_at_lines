#!/usr/bin/env bash
set -euo pipefail
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "unsupported architecture: $ARCH" && exit 1 ;;
esac
case "$OS" in
  darwin) TARGET="${ARCH}-apple-darwin" ;;
  linux)  TARGET="${ARCH}-unknown-linux-gnu" ;;
  *) echo "unsupported OS: $OS" && exit 1 ;;
esac
TAG=$(curl -fsSL https://api.github.com/repos/n05tr0m0/look_at_lines/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
curl -fsSL "https://github.com/n05tr0m0/look_at_lines/releases/download/${TAG}/ll-${TAG}-${TARGET}.tar.gz" \
  | tar -xz
chmod +x ll && sudo mv ll /usr/local/bin/
echo "installed: $(ll --version)"
