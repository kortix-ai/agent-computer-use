#!/usr/bin/env bash
set -euo pipefail

echo "Running agent-click benchmarks..."
echo "────────────────────────────"
echo ""

cd "$(dirname "$0")/../cli"

if [ "${1:-}" = "--save" ]; then
    echo "Running with baseline save..."
    cargo bench -- --save-baseline main
elif [ "${1:-}" = "--compare" ]; then
    echo "Comparing against saved baseline..."
    cargo bench -- --baseline main
else
    cargo bench "$@"
fi

echo ""
echo "Reports: cli/target/criterion/report/index.html"
