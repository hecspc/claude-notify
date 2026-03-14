#!/bin/sh
set -e

REPO="hecspc/claude-notify"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.bin}"
VERSION="${1:-latest}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux)  ARTIFACT="claude-notify-linux-x86_64" ;;
  darwin)
    case "$ARCH" in
      arm64|aarch64) ARTIFACT="claude-notify-macos-aarch64" ;;
      x86_64)        ARTIFACT="claude-notify-macos-x86_64" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Resolve version
if [ "$VERSION" = "latest" ]; then
  TAG=$(curl -sI "https://github.com/$REPO/releases/latest" | grep -i '^location:' | sed 's|.*/||' | tr -d '\r')
  if [ -z "$TAG" ]; then
    echo "Failed to determine latest version"
    exit 1
  fi
else
  case "$VERSION" in
    v*) TAG="$VERSION" ;;
    *)  TAG="v$VERSION" ;;
  esac
fi

URL="https://github.com/$REPO/releases/download/$TAG/$ARTIFACT"

echo "Installing claude-notify $TAG ($ARTIFACT)..."
echo "  → $INSTALL_DIR/claude-notify"

# Download
mkdir -p "$INSTALL_DIR"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" -o "$INSTALL_DIR/claude-notify"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$INSTALL_DIR/claude-notify" "$URL"
else
  echo "Error: curl or wget required"
  exit 1
fi

chmod +x "$INSTALL_DIR/claude-notify"

echo "Installed claude-notify $TAG to $INSTALL_DIR/claude-notify"

# Check if install dir is in PATH
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Note: Add $INSTALL_DIR to your PATH if not already done" ;;
esac
