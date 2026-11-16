#!/usr/bin/env node
/**
 * GreenCode Profiler - Node.js CLI Wrapper
 * Memungkinkan pemanggilan 'npx greencode' atau 'greencode' dari ekosistem Node.js / npm.
 * Memprioritaskan eksekusi Native Rust Binary (Zero-Overhead) dan fallback ke Python Engine.
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const args = process.argv.slice(2);
const isWindows = process.platform === 'win32';
const rustBinaryName = isWindows ? 'greencode.exe' : 'greencode';
const rustBinaryPath = path.resolve(__dirname, rustBinaryName);
const pythonScript = path.resolve(__dirname, '..', 'greencode', 'cli.py');

// 1. Cek apakah Native Rust Binary tersedia (Rekomendasi Utama)
if (fs.existsSync(rustBinaryPath)) {
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
} else {
    // 2. Fallback ke Python CLI jika binary belum dikompilasi
    const pythonCommands = ['python', 'python3', 'py'];
    function findPython() {
        const { spawnSync } = require('child_process');
        for (const cmd of pythonCommands) {
            try {
                const res = spawnSync(cmd, ['--version']);
                if (res.status === 0) return cmd;
            } catch (e) {}
        }
        if (isWindows && process.env.LOCALAPPDATA) {
            const pyDir = path.join(process.env.LOCALAPPDATA, 'Programs', 'Python');
            if (fs.existsSync(pyDir)) {
                const dirs = fs.readdirSync(pyDir).sort().reverse();
                for (const d of dirs) {
                    const exe = path.join(pyDir, d, 'python.exe');
                    if (fs.existsSync(exe)) return exe;
                }
            }
        }
        return 'python';
    }

    const py = findPython();
    const child = spawn(py, [pythonScript, ...args], {
        stdio: 'inherit',
        cwd: process.cwd()
    });

    child.on('error', (err) => {
        console.error('\x1b[31m[GreenCode Error]\x1b[0m', err.message);
        process.exit(1);
    });

    child.on('exit', (code) => {
        process.exit(code || 0);
    });
}
