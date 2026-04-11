#!/bin/bash
set -e

echo "📦 Bundling binaries for npm publish..."

BIN_DIR="$(pwd)/bin"
REQUIRED=("agent-computer-use-macos-arm64" "agent-computer-use-macos-x64")

missing=0
for bin in "${REQUIRED[@]}"; do
  if [ -f "$BIN_DIR/$bin" ]; then
    echo "  ✅ $bin ($(du -h "$BIN_DIR/$bin" | awk '{print $1}'))"
  else
    echo "  ❌ $bin missing"
    missing=1
  fi
done

# Optional (from Docker)
for bin in agent-computer-use-linux-x64 agent-computer-use-linux-arm64 agent-computer-use-windows-x64.exe; do
  if [ -f "$BIN_DIR/$bin" ]; then
    echo "  ✅ $bin ($(du -h "$BIN_DIR/$bin" | awk '{print $1}'))"
  else
    echo "  ⚠  $bin not found (optional)"
  fi
done

if [ $missing -eq 1 ]; then
  echo ""
  echo "❌ Required binaries missing. Run: pnpm build:all"
  exit 1
fi

# Make sure agent-cu.js is there
if [ ! -f "$BIN_DIR/agent-cu.js" ]; then
  echo "❌ bin/agent-cu.js missing"
  exit 1
fi

echo ""
echo "✅ Ready to publish. Run: pnpm changeset publish"
