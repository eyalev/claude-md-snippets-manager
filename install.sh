#!/bin/bash

set -e

# Claude MD Snippets Manager - One-line installer
echo "🚀 Installing claude-md-snippets-manager..."

# Detect architecture
ARCH=$(uname -m)
OS=$(uname -s)

if [[ "$OS" != "Linux" ]]; then
    echo "❌ This installer currently supports Linux only"
    exit 1
fi

if [[ "$ARCH" != "x86_64" ]]; then
    echo "❌ This installer currently supports x86_64 architecture only"
    exit 1
fi

# Download URL
BINARY_URL="https://github.com/eyalev/claude-md-snippets-manager/releases/latest/download/claude-md-snippets-manager-linux-x64"

# Create temporary directory
TMP_DIR=$(mktemp -d)
BINARY_PATH="$TMP_DIR/claude-md-snippets-manager"

# Download binary
echo "📥 Downloading claude-md-snippets-manager..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$BINARY_URL" -o "$BINARY_PATH"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$BINARY_URL" -O "$BINARY_PATH"
else
    echo "❌ Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Make executable
chmod +x "$BINARY_PATH"

# Install to system
INSTALL_DIR="/usr/local/bin"
if [[ -w "$INSTALL_DIR" ]]; then
    mv "$BINARY_PATH" "$INSTALL_DIR/claude-md-snippets-manager"
    echo "✅ Installed to $INSTALL_DIR/claude-md-snippets-manager"
else
    echo "🔐 Installing to system directory (requires sudo)..."
    sudo mv "$BINARY_PATH" "$INSTALL_DIR/claude-md-snippets-manager"
    echo "✅ Installed to $INSTALL_DIR/claude-md-snippets-manager"
fi

# Cleanup
rm -rf "$TMP_DIR"

# Verify installation
if command -v claude-md-snippets-manager >/dev/null 2>&1; then
    echo ""
    echo "🎉 Installation successful!"
    echo ""
    echo "Get started:"
    echo "  claude-md-snippets-manager setup    # Setup your first repository"  
    echo "  claude-md-snippets-manager --help   # See all available commands"
    echo ""
    claude-md-snippets-manager --version
else
    echo "❌ Installation failed. Binary not found in PATH."
    exit 1
fi