#!/usr/bin/env node

/**
 * Shadow Secret NPM Bridge
 *
 * This script acts as a bridge between NPM and the native Rust binary.
 * It detects the platform and spawns the correct binary with proper
 * terminal inheritance for an seamless user experience.
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Platform detection
const platform = process.platform;
const isWindows = platform === 'win32';

// Binary name based on platform
const binaryName = isWindows ? 'shadow-secret.exe' : 'shadow-secret';

// Resolve binary path relative to this script
const binDir = path.resolve(__dirname, '..', 'bin');
const binaryPath = path.join(binDir, binaryName);

/**
 * Check if the binary exists
 */
function binaryExists() {
  return fs.existsSync(binaryPath);
}

/**
 * Spawn the native binary with inherited stdio
 */
function spawnBinary() {
  // Check if binary exists
  if (!binaryExists()) {
    console.error(`❌ Error: Binary not found at ${binaryPath}`);
    console.error('');
    console.error('The native Rust binary is not installed.');
    console.error('');
    console.error('If you are developing this package:');
    console.error('  1. Build the binary: cd packages/core && cargo build --release');
    console.error(`  2. Copy the binary to: ${binaryPath}`);
    console.error('');
    console.error('If you are a user:');
    console.error('  Please reinstall this package: npm install -g shadow-secret');
    console.error('');
    console.error('For more information, visit: https://github.com/Pamacea/shadow-secret');
    process.exit(1);
  }

  // Get arguments (skip first two: node executable and this script)
  const args = process.argv.slice(2);

  // Spawn the binary with inherited stdio for seamless terminal integration
  const child = spawn(binaryPath, args, {
    stdio: 'inherit',
    env: process.env
  });

  // Forward exit code
  child.on('close', (code) => {
    process.exit(code ?? 0);
  });

  // Forward errors
  child.on('error', (err) => {
    console.error(`❌ Failed to start binary: ${err.message}`);
    process.exit(1);
  });
}

// Execute
spawnBinary();
