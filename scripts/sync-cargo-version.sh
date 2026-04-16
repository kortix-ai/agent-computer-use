#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PKG_JSON="$ROOT_DIR/package.json"
CARGO_TOML="$ROOT_DIR/cli/Cargo.toml"

if [[ ! -f "$PKG_JSON" ]]; then
  echo "sync-cargo-version: $PKG_JSON not found" >&2
  exit 1
fi
if [[ ! -f "$CARGO_TOML" ]]; then
  echo "sync-cargo-version: $CARGO_TOML not found" >&2
  exit 1
fi

VERSION="$(node -p "require('$PKG_JSON').version")"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
  echo "sync-cargo-version: refusing to sync non-semver version '$VERSION'" >&2
  exit 1
fi

python3 - "$CARGO_TOML" "$VERSION" <<'PY'
import re, sys
path, new_version = sys.argv[1], sys.argv[2]
with open(path, 'r') as f:
    content = f.read()
new_content, n = re.subn(
    r'(?m)^version = "\d+\.\d+\.\d+"',
    f'version = "{new_version}"',
    content,
    count=1,
)
if n != 1:
    sys.stderr.write(
        f"sync-cargo-version: expected 1 top-level `version = \"...\"` in {path}, found {n}\n"
    )
    sys.exit(2)
if new_content != content:
    with open(path, 'w') as f:
        f.write(new_content)
    print(f"Synced {path} → version = \"{new_version}\"")
else:
    print(f"Already at version {new_version}, no change")
PY
