#!/usr/bin/env node

const { execFileSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

function getBinaryName() {
  const platform = os.platform();
  const arch = os.arch();

  const map = {
    'darwin-arm64': 'agent-click-macos-arm64',
    'darwin-x64': 'agent-click-macos-x64',
    'linux-x64': 'agent-click-linux-x64',
    'linux-arm64': 'agent-click-linux-arm64',
    'win32-x64': 'agent-click-windows-x64.exe',
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
    path.join(binDir, '..', 'cli', 'target', 'release', 'agent-click'),
    path.join(binDir, '..', 'cli', 'target', 'debug', 'agent-click'),
  ];

  for (const p of devPaths) {
    if (fs.existsSync(p)) return p;
  }

  try {
    execFileSync('which', ['agent-click'], { stdio: 'pipe' });
    return 'agent-click';
  } catch {}

  console.error(
    'agent-click binary not found.\n\n' +
      'Either:\n' +
      '  1. Build from source: cd cli && cargo build --release\n' +
      '  2. Install via cargo: cargo install agent-click\n',
  );
  process.exit(1);
}

const binary = findBinary();
const args = process.argv.slice(2);

try {
  execFileSync(binary, args, {
    stdio: 'inherit',
    env: process.env,
  });
} catch (err) {
  process.exit(err.status || 1);
}
