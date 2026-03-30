#!/bin/bash
set -e

echo "📦 Bundling binaries for npm publish..."

BIN_DIR="$(pwd)/bin"
REQUIRED=("agent-click-macos-arm64" "agent-click-macos-x64")

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
for bin in agent-click-linux-x64 agent-click-linux-arm64 agent-click-windows-x64.exe; do
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

# Make sure agent-click.js is there
if [ ! -f "$BIN_DIR/agent-click.js" ]; then
  echo "❌ bin/agent-click.js missing"
  exit 1
fi

echo ""
echo "✅ Ready to publish. Run: pnpm changeset publish"
