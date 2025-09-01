#!/bin/bash

set -e

# Claude MD Snippets - One-line installer
echo "🚀 Installing claude-md-snippets..."

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
BINARY_URL="https://github.com/eyalev/claude-md-snippets/releases/latest/download/claude-md-snippets-linux-x64"

# Create temporary directory
TMP_DIR=$(mktemp -d)
BINARY_PATH="$TMP_DIR/claude-md-snippets"

# Download binary
echo "📥 Downloading claude-md-snippets..."
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
    mv "$BINARY_PATH" "$INSTALL_DIR/claude-md-snippets"
    echo "✅ Installed to $INSTALL_DIR/claude-md-snippets"
else
    echo "🔐 Installing to system directory (requires sudo)..."
    sudo mv "$BINARY_PATH" "$INSTALL_DIR/claude-md-snippets"
    echo "✅ Installed to $INSTALL_DIR/claude-md-snippets"
fi

# Cleanup
rm -rf "$TMP_DIR"

# Verify installation
if command -v claude-md-snippets >/dev/null 2>&1; then
    echo ""
    echo "🎉 Installation successful!"
    echo ""
    echo "Get started:"
    echo "  claude-md-snippets setup    # Setup your first repository"  
    echo "  claude-md-snippets --help   # See all available commands"
    echo ""
    claude-md-snippets --version
else
    echo "❌ Installation failed. Binary not found in PATH."
    exit 1
fi