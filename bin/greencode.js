#!/usr/bin/env node
/**
 * GreenCode Profiler - Node.js CLI Wrapper
 * Memungkinkan pemanggilan 'npx greencode' atau 'greencode' dari ekosistem Node.js / npm.
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const args = process.argv.slice(2);
const pythonScript = path.resolve(__dirname, '..', 'greencode', 'cli.py');

// Cek perintah python di sistem (python, python3, py, atau default Windows path)
const pythonCommands = ['python', 'python3', 'py'];

function findPython() {
    const { spawnSync } = require('child_process');
    for (const cmd of pythonCommands) {
        try {
            const res = spawnSync(cmd, ['--version']);
            if (res.status === 0) {
                return cmd;
            }
        } catch (e) {}
    }

    // Windows fallback: cek LocalAppData Programs
    if (process.platform === 'win32' && process.env.LOCALAPPDATA) {
        const pyDir = path.join(process.env.LOCALAPPDATA, 'Programs', 'Python');
        if (fs.existsSync(pyDir)) {
            const dirs = fs.readdirSync(pyDir).sort().reverse();
            for (const d of dirs) {
                const exe = path.join(pyDir, d, 'python.exe');
                if (fs.existsSync(exe)) {
                    return exe;
                }
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
    if (err.code === 'ENOENT') {
        console.error('\x1b[31m[GreenCode Error]\x1b[0m Python 3.9+ tidak ditemukan di sistem PATH.');
        console.error('Silakan pastikan Python sudah terinstal untuk menjalankan engine profiler.');
    } else {
        console.error('\x1b[31m[GreenCode Error]\x1b[0m', err.message);
    }
    process.exit(1);
});

child.on('close', (code) => {
    process.exit(code || 0);
});
