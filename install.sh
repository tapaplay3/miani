#!/bin/bash
# Installation script for miani

set -e

echo "🔨 Building miani..."
cargo build --release

echo "📦 Installing miani..."

# Determine installation directory
if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ -w "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
else
    INSTALL_DIR="$HOME/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Copy binary
cp target/release/miani "$INSTALL_DIR/miani"
chmod +x "$INSTALL_DIR/miani"

echo "✅ miani installed to $INSTALL_DIR/miani"
echo ""

# Check if directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Warning: $INSTALL_DIR is not in your PATH"
    echo ""
    echo "Add this line to your ~/.zshrc or ~/.bashrc:"
    echo "    export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
    echo "Then run: source ~/.zshrc"
else
    echo "🎉 Installation complete! You can now run: miani"
fi

echo ""
echo "Usage examples:"
echo "  miani                    # Run with LM Studio"
echo "  miani -p ollama          # Run with Ollama"
echo "  miani -p ollama -m llama2"
