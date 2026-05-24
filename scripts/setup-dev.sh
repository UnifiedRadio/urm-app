#!/usr/bin/env bash
set -euo pipefail

echo "=== OpenRadio dev environment setup ==="

# 1. Rust toolchain
if ! command -v cargo &>/dev/null; then
  echo "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi
echo "Rust $(rustc --version)"

# 2. Tauri CLI
if ! cargo tauri --version &>/dev/null 2>&1; then
  echo "Installing tauri-cli..."
  cargo install tauri-cli --version "^2"
fi

# 3. Node / npm
if ! command -v node &>/dev/null; then
  echo "ERROR: Node.js not found. Install via https://nodejs.org or nvm."
  exit 1
fi
echo "Node $(node --version), npm $(npm --version)"

# 4. Frontend dependencies
npm install

# 5. Python + CHIRP check
bash "$(dirname "$0")/check-chirp.sh"

echo ""
echo "✓ Setup complete. Run: npm run tauri:dev"
