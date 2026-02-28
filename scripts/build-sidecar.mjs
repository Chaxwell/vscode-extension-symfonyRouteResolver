#!/usr/bin/env node

/**
 * Build script for the Rust sidecar binary.
 *
 * Usage:
 *   node scripts/build-sidecar.mjs <target> [--dev]
 *
 * Targets:
 *   linux       x86_64-unknown-linux-gnu   (Linux 64-bit)
 *   linux-arm   aarch64-unknown-linux-gnu  (Linux ARM 64-bit)
 *   mac         x86_64-apple-darwin        (macOS Intel)
 *   mac-arm     aarch64-apple-darwin       (macOS Apple Silicon)
 *   windows     x86_64-pc-windows-msvc     (Windows 64-bit)
 *
 * Flags:
 *   --dev    Debug build (faster to compile, larger binary)
 *
 * Cross-compilation:
 *   Install `cross` for transparent cross-compilation via Docker:
 *     cargo install cross
 *   Without it the script falls back to `cargo`, which requires the target
 *   toolchain to be installed (`rustup target add <triple>`) and the
 *   appropriate system linker.
 */

import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..');

const TARGETS = {
    linux: {
        triple: 'x86_64-unknown-linux-gnu',
        binary: 'symfony-route-resolver-sidecar',
        vsixTarget: 'linux-x64',
    },
    'linux-arm': {
        triple: 'aarch64-unknown-linux-gnu',
        binary: 'symfony-route-resolver-sidecar',
        vsixTarget: 'linux-arm64',
    },
    mac: {
        triple: 'x86_64-apple-darwin',
        binary: 'symfony-route-resolver-sidecar',
        vsixTarget: 'darwin-x64',
    },
    'mac-arm': {
        triple: 'aarch64-apple-darwin',
        binary: 'symfony-route-resolver-sidecar',
        vsixTarget: 'darwin-arm64',
    },
    windows: {
        triple: 'x86_64-pc-windows-msvc',
        binary: 'symfony-route-resolver-sidecar.exe',
        vsixTarget: 'win32-x64',
    },
};

// ---------------------------------------------------------------------------
// Parse arguments
// ---------------------------------------------------------------------------

const args = process.argv.slice(2);
const targetKey = args.find((a) => !a.startsWith('--'));
const isDev = args.includes('--dev');

if (!targetKey || !TARGETS[targetKey]) {
    console.error('Usage: node scripts/build-sidecar.mjs <target> [--dev]');
    console.error(`Available targets: ${Object.keys(TARGETS).path.join(', ')}`);

    process.exit(1);
}

const { triple, binary, vsixTarget } = TARGETS[targetKey];
const profile = isDev ? '' : '--release';
const profileDir = isDev ? 'debug' : 'release';

// ---------------------------------------------------------------------------
// Choose compiler: `cross` if available, else `cargo`
// ---------------------------------------------------------------------------

let compiler = 'cargo';

try {
    execSync('cross --version', { stdio: 'ignore' });
    compiler = 'cross';
    console.log('Using `cross` for cross-compilation.');
} catch {
    console.log('`cross` not found — using `cargo` (native toolchain required).');
}

// ---------------------------------------------------------------------------
// Build
// ---------------------------------------------------------------------------

console.log(`\nBuilding sidecar for ${targetKey} (${triple})…`);

try {
    execSync(`${compiler} build ${profile} --target ${triple}`, {
        cwd: path.join(ROOT, 'sidecar'),
        stdio: 'inherit',
    });
} catch {
    console.error('\nBuild failed.');

    if (compiler === 'cargo') {
        console.error(
            `Tip: install the target with:\n  rustup target add ${triple}\n` +
            `Or install \`cross\` for Docker-based cross-compilation:\n  cargo install cross`
        );
    }

    process.exit(1);
}

// ---------------------------------------------------------------------------
// Copy binary to bin/
// ---------------------------------------------------------------------------

const src = path.join(ROOT, 'sidecar', 'target', triple, profileDir, binary);
const binDir = path.join(ROOT, 'bin');

fs.mkdirSync(binDir, { recursive: true });
fs.cpSync(src, path.join(binDir, binary));

console.log(`\n✓ Binary → bin/${binary}`);
console.log(`  VSIX target : ${vsixTarget}`);
console.log(`  Package with: vsce package --target ${vsixTarget}`);
