#!/bin/bash
set -e

echo "🔧 Setting up agent-click development environment..."
echo ""

# Check Rust
if command -v rustc &>/dev/null; then
  echo "✅ Rust $(rustc --version | awk '{print $2}')"
else
  echo "❌ Rust not found"
  echo "   Install: https://rustup.rs"
  echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 1
fi

# Check Node
if command -v node &>/dev/null; then
  echo "✅ Node $(node --version)"
else
  echo "❌ Node not found"
  echo "   Install: https://nodejs.org"
  exit 1
fi

# Check pnpm
if command -v pnpm &>/dev/null; then
  echo "✅ pnpm $(pnpm --version)"
else
  echo "❌ pnpm not found — installing..."
  npm install -g pnpm
  echo "✅ pnpm installed"
fi

echo ""
echo "📦 Installing dependencies..."
pnpm install

echo ""
echo "🔨 Building CLI..."
cd cli && cargo build --release && cd ..

echo ""
echo "✅ All set! Try these:"
echo ""
echo "   pnpm dev          # Start docs site"
echo "   pnpm build        # Build CLI (release)"
echo "   pnpm test         # Run tests"
echo "   pnpm test:e2e     # Run e2e tests (needs macOS)"
echo "   pnpm lint         # Check formatting + clippy"
echo ""
echo "   ./cli/target/release/agent-click --help"
