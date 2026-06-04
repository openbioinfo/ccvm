#!/bin/sh
set -e

REPO="openbioinfo/ccvm"
INSTALL_DIR="${CCVM_INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  ARTIFACT="ccvm-linux-x64" ;;
      aarch64) ARTIFACT="ccvm-linux-arm64" ;;
      *)       echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  ARTIFACT="ccvm-macos-x64" ;;
      arm64)   ARTIFACT="ccvm-macos-arm64" ;;
      *)       echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS" >&2
    echo "For Windows, use: irm https://raw.githubusercontent.com/$REPO/master/install.ps1 | iex" >&2
    exit 1
    ;;
esac

# Resolve version
if [ -z "$VERSION" ]; then
  echo "Fetching latest release..."
  VERSION=$(curl -fsSL --proxy "" "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"v\([^"]*\)".*/\1/') \
    || VERSION=$(curl -fsSL --proxy http://127.0.0.1:7890 "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"v\([^"]*\)".*/\1/')
fi

if [ -z "$VERSION" ]; then
  echo "Could not determine latest version. Set VERSION env var to install a specific version." >&2
  exit 1
fi

echo "Installing ccvm $VERSION ($ARTIFACT)..."

URL="https://github.com/$REPO/releases/download/v${VERSION}/${ARTIFACT}.tar.gz"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Download with proxy fallback
curl -fsSL "$URL" -o "$TMP/ccvm.tar.gz" \
  || curl -fsSL --proxy http://127.0.0.1:7890 "$URL" -o "$TMP/ccvm.tar.gz"

tar -xzf "$TMP/ccvm.tar.gz" -C "$TMP"

# Install binaries
mkdir -p "$INSTALL_DIR"
install -m 755 "$TMP/ccvm" "$INSTALL_DIR/ccvm"
install -m 755 "$TMP/ccvm-shim" "$INSTALL_DIR/ccvm-shim"
install -m 755 "$TMP/ccvm-codex-shim" "$INSTALL_DIR/ccvm-codex-shim"

echo "Installed ccvm to $INSTALL_DIR"

# Ensure INSTALL_DIR is in PATH for this session
export PATH="$INSTALL_DIR:$PATH"

# Run setup
"$INSTALL_DIR/ccvm" setup

echo ""
echo "Done! Restart your terminal or run:"
echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
echo ""
echo "Then: ccvm install latest"
