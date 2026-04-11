#!/bin/bash
set -e

echo "🔨 Building agent-computer-use for all platforms..."
echo ""

BIN_DIR="$(pwd)/bin"
mkdir -p "$BIN_DIR"

# macOS (native only — must run on macOS)
if [ "$(uname)" = "Darwin" ]; then
  echo "→ macOS ARM64..."
  cd cli && cargo build --release --target aarch64-apple-darwin 2>/dev/null && cd ..
  cp cli/target/aarch64-apple-darwin/release/agent-cu "$BIN_DIR/agent-computer-use-macos-arm64"
  echo "✅ agent-computer-use-macos-arm64"

  echo "→ macOS x64..."
  cd cli && cargo build --release --target x86_64-apple-darwin 2>/dev/null && cd ..
  cp cli/target/x86_64-apple-darwin/release/agent-cu "$BIN_DIR/agent-computer-use-macos-x64"
  echo "✅ agent-computer-use-macos-x64"
else
  echo "⚠  Skipping macOS builds (not on macOS)"
fi

# Linux + Windows via Docker
if command -v docker &>/dev/null; then
  echo "→ Linux + Windows (via Docker)..."
  cd docker && docker compose up --build 2>&1 | tail -5 && cd ..
  echo "✅ Linux + Windows builds"
else
  echo "⚠  Skipping Linux/Windows builds (Docker not found)"
fi

echo ""
echo "📦 Built binaries:"
ls -lh "$BIN_DIR"/agent-computer-use-* 2>/dev/null || echo "   (none)"
