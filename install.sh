#!/bin/bash
set -e

echo "🔨 Building hyprgrid in release mode..."
cargo build --release

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

echo "📦 Installing binary to ${INSTALL_DIR}/hyprgrid..."
rm -f "${INSTALL_DIR}/hyprgrid"
cp target/release/hyprgrid "${INSTALL_DIR}/hyprgrid"
chmod +x "${INSTALL_DIR}/hyprgrid"

if [ -d "${HOME}/.cargo/bin" ]; then
    rm -f "${HOME}/.cargo/bin/hyprgrid"
    cp target/release/hyprgrid "${HOME}/.cargo/bin/hyprgrid"
fi

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/hyprgrid"
if [ ! -f "${CONFIG_DIR}/config.toml" ]; then
    echo "⚙️ Creating default configuration at ${CONFIG_DIR}/config.toml..."
    mkdir -p "$CONFIG_DIR"
    cp hyprgrid.example.toml "${CONFIG_DIR}/config.toml"
fi

echo "✅ hyprgrid installed successfully!"
echo "   Binary size: $(du -h "${INSTALL_DIR}/hyprgrid" | cut -f1)"
echo "   Version:     $(${INSTALL_DIR}/hyprgrid --version)"
