#!/usr/bin/env node
/**
 * GreenCode Profiler - Node.js CLI Runner
 * Menjalankan Native Standalone Binary Rust (Zero-Overhead & Low Energy).
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const args = process.argv.slice(2);
const isWindows = process.platform === 'win32';
const rustBinaryName = isWindows ? 'greencode.exe' : 'greencode';
const rustBinaryPath = path.resolve(__dirname, rustBinaryName);

if (!fs.existsSync(rustBinaryPath)) {
    console.error('\x1b[31m[GreenCode Error]\x1b[0m Native Rust binary tidak ditemukan di:', rustBinaryPath);
    console.error('Silakan jalankan "cargo build --release" untuk mengompilasi binary.');
    process.exit(1);
}

const child = spawn(rustBinaryPath, args, {
    stdio: 'inherit',
    cwd: process.cwd()
});

child.on('error', (err) => {
    console.error('\x1b[31m[GreenCode Error]\x1b[0m Gagal menjalankan Rust binary:', err.message);
    process.exit(1);
});

child.on('exit', (code) => {
    process.exit(code || 0);
});
