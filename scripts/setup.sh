#!/usr/bin/env bash
# hkwatch setup — macOS, Linux, and Windows (WSL / Git Bash)
# Installs dependencies and builds/downloads the hkwatch binary.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_DIR"

echo "==> hkwatch setup ($(uname -s) / $(uname -m))"

# ---- 1. Node.js ----
if ! command -v node >/dev/null 2>&1; then
  echo "==> Installing Node.js (via nvm)"
  curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
  export NVM_DIR="$HOME/.nvm"
  [ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
  nvm install --lts
  nvm alias default 'lts/*'
else
  echo "==> Node.js already installed: $(node --version)"
fi

# ---- 2. Rust ----
if ! command -v cargo >/dev/null 2>&1; then
  echo "==> Installing Rust (rustup)"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  . "$HOME/.cargo/env"
else
  echo "==> Rust already installed: $(cargo --version)"
fi
export PATH="$HOME/.cargo/bin:$PATH"

# ---- 3. Playwright stealth fetcher ----
echo "==> Installing Node deps for the cloak fetcher"
cd cloak
npm install --no-fund --no-audit
npx playwright install chromium
cd "$REPO_DIR"

# ---- 4. opencode2 (solver) ----
if ! command -v opencode2 >/dev/null 2>&1; then
  echo "==> Installing opencode2 (@opencode-ai/cli)"
  npm install -g @opencode-ai/cli
else
  echo "==> opencode2 already installed: $(opencode2 --version 2>/dev/null || true)"
fi

# ---- 5. hkwatch binary ----
BIN_DIR="$REPO_DIR/bin"
mkdir -p "$BIN_DIR"
if [ -f "$BIN_DIR/hkwatch" ]; then
  echo "==> hkwatch binary already present"
else
  echo "==> Building hkwatch from source (cargo)"
  cd watcher
  cargo build --release
  cp target/release/hkwatch "$BIN_DIR/hkwatch"
  cd "$REPO_DIR"
fi

# ---- 6. credentials ----
if [ ! -f .hkwatch.env ]; then
  cp .hkwatch.env.example .hkwatch.env
  echo "==> Created .hkwatch.env — EDIT IT with your HackerRank credentials"
else
  echo "==> .hkwatch.env already present"
fi

echo
echo "=== Setup complete ==="
echo "Next steps:"
echo "  1. nano $REPO_DIR/.hkwatch.env   # add your HackerRank username/password"
echo "  2. $BIN_DIR/hkwatch watch --headless --skip-current"
echo "     (kill -USR1 <pid> toggles auto-solving without stopping the watcher)"
