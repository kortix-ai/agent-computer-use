#!/usr/bin/env node

const { execFileSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

function getBinaryName() {
  const platform = os.platform();
  const arch = os.arch();

  const map = {
    'darwin-arm64': 'agent-computer-use-macos-arm64',
    'darwin-x64': 'agent-computer-use-macos-x64',
    'linux-x64': 'agent-computer-use-linux-x64',
    'linux-arm64': 'agent-computer-use-linux-arm64',
    'win32-x64': 'agent-computer-use-windows-x64.exe',
  };

  const key = `${platform}-${arch}`;
  return map[key] || null;
}

function findBinary() {
  const binDir = __dirname;

  const platformBinary = getBinaryName();
  if (platformBinary) {
    const platformPath = path.join(binDir, platformBinary);
    if (fs.existsSync(platformPath)) return platformPath;
  }

  const devPaths = [
    path.join(binDir, '..', 'cli', 'target', 'release', 'agent-cu'),
    path.join(binDir, '..', 'cli', 'target', 'debug', 'agent-cu'),
  ];

  for (const p of devPaths) {
    if (fs.existsSync(p)) return p;
  }

  try {
    execFileSync('which', ['agent-cu'], { stdio: 'pipe' });
    return 'agent-cu';
  } catch {}

  console.error(
    'agent-cu binary not found.\n\n' +
      'Either:\n' +
      '  1. Build from source: cd cli && cargo build --release\n' +
      '  2. Install via cargo: cargo install --git https://github.com/kortix-ai/agent-computer-use\n',
  );
  process.exit(1);
}

const binary = findBinary();
const args = process.argv.slice(2);

// Self-heal: actions/upload-artifact@v4 strips the exec bit on POSIX, so the
// tarball we publish can land with mode 0644 and fail silently. Re-apply 755
// before every invocation — cheap and idempotent. Ignore EPERM on read-only
// filesystems and on Windows where chmod is a no-op.
try {
  fs.chmodSync(binary, 0o755);
} catch {}

try {
  execFileSync(binary, args, {
    stdio: 'inherit',
    env: process.env,
  });
} catch (err) {
  // err.status is a number when the child ran and exited non-zero — the
  // binary already wrote its own stderr, so just propagate the code.
  // Otherwise spawn itself failed (EACCES, ENOENT, ...) and nothing reached
  // the user's terminal — surface the reason instead of silent-exiting.
  if (typeof err.status !== 'number') {
    console.error(`agent-cu: failed to run ${binary}`);
    console.error(`  ${err.code || 'error'}: ${err.message}`);
    console.error(`  try reinstalling: npm i -g @kortix-ai/agent-computer-use@latest`);
  }
  process.exit(typeof err.status === 'number' ? err.status : 1);
}
