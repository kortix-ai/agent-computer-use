#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -f package.json ]]; then
  echo "sync-cargo-version: package.json not found in $ROOT_DIR" >&2
  exit 1
fi
if [[ ! -f cli/Cargo.toml ]]; then
  echo "sync-cargo-version: cli/Cargo.toml not found in $ROOT_DIR" >&2
  exit 1
fi

VERSION="$(node -p "require('./package.json').version")"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
  echo "sync-cargo-version: refusing to sync non-semver version '$VERSION'" >&2
  exit 1
fi

node -e '
  const fs = require("fs");
  const path = "cli/Cargo.toml";
  const newVersion = process.argv[1];
  const content = fs.readFileSync(path, "utf8");
  let replaced = 0;
  const next = content.replace(/^version = "\d+\.\d+\.\d+"/m, () => {
    replaced++;
    return `version = "${newVersion}"`;
  });
  if (replaced !== 1) {
    console.error(`sync-cargo-version: expected 1 top-level version in ${path}, found ${replaced}`);
    process.exit(2);
  }
  if (next !== content) {
    fs.writeFileSync(path, next);
    console.log(`Synced ${path} → version = "${newVersion}"`);
  } else {
    console.log(`Already at version ${newVersion}, no change`);
  }
' "$VERSION"
