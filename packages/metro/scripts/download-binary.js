#!/usr/bin/env node
import { existsSync, mkdirSync, chmodSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const PACKAGE_ROOT = join(__dirname, '..');
const BIN_DIR = join(PACKAGE_ROOT, 'bin');

// Get platform and architecture
const PLATFORM_MAP = {
  darwin: 'apple-darwin',
  linux: 'unknown-linux-gnu',
  win32: 'pc-windows-msvc',
};

const ARCH_MAP = {
  x64: 'x86_64',
  arm64: 'aarch64',
};

const platform = PLATFORM_MAP[process.platform];
const arch = ARCH_MAP[process.arch];

if (!platform || !arch) {
  console.error(`Unsupported platform: ${process.platform}-${process.arch}`);
  process.exit(1);
}

const target = `${arch}-${platform}`;
const binaryName = process.platform === 'win32' ? 'fabrik.exe' : 'fabrik';
const finalBinaryPath = join(BIN_DIR, binaryName);

// Check if binary already exists
if (existsSync(finalBinaryPath)) {
  console.log('Fabrik binary already installed');
  process.exit(0);
}

// Get version from package.json
const packageJson = JSON.parse(
  readFileSync(join(PACKAGE_ROOT, 'package.json'), 'utf-8')
);
const version = packageJson.version;

// Download URL from GitHub releases
const downloadUrl = `https://github.com/tuist/fabrik/releases/download/v${version}/fabrik-v${version}-${target}.tar.gz`;

console.log(`Downloading Fabrik v${version} for ${target}...`);
console.log(`URL: ${downloadUrl}`);

// Create bin directory
if (!existsSync(BIN_DIR)) {
  mkdirSync(BIN_DIR, { recursive: true });
}

try {
  // Download and extract using curl + tar (available on all platforms)
  if (process.platform === 'win32') {
    // Windows: Download zip and extract
    const zipUrl = `https://github.com/tuist/fabrik/releases/download/v${version}/fabrik-v${version}-${target}.zip`;
    execSync(
      `curl -L "${zipUrl}" -o fabrik.zip && tar -xf fabrik.zip -C "${BIN_DIR}" && del fabrik.zip`,
      { stdio: 'inherit', cwd: PACKAGE_ROOT }
    );
  } else {
    // Unix: Download tar.gz and extract
    execSync(
      `curl -L "${downloadUrl}" | tar -xz -C "${BIN_DIR}"`,
      { stdio: 'inherit' }
    );
  }

  // Make binary executable
  if (process.platform !== 'win32') {
    chmodSync(finalBinaryPath, 0o755);
  }

  console.log(`✓ Fabrik binary installed to ${finalBinaryPath}`);
} catch (error) {
  console.error('Failed to download Fabrik binary:', error.message);
  console.error('\nYou can manually install Fabrik using:');
  console.error('  mise use -g ubi:tuist/fabrik');
  console.error('  or download from: https://github.com/tuist/fabrik/releases');
  process.exit(1);
}
