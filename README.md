<p align="center">
  <img src="assets/logo.png" alt="GreenCode Profiler Logo" width="220" />
</p>

# GreenCode Profiler (GCP)

**Universal Process-Level Energy & Code Quality Profiler for Modern Web Applications**

[![License: MIT](https://img.shields.io/badge/License-MIT-emerald.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.100+-F74C00.svg?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Node.js](https://img.shields.io/badge/NPM-Launcher-339933.svg?logo=node.js&logoColor=white)](https://nodejs.org)
[![Architecture](https://img.shields.io/badge/Architecture-Rust%20Native%20Zero%20Cost-10b981.svg)]()
[![Version](https://img.shields.io/badge/Version-1.2.0-blue.svg)]()

> **GreenCode Profiler v1.2.0** adalah framework pengukuran efisiensi energi perangkat lunak (_Green Software Engineering_) berbasis **Rust Native Engine** yang menggabungkan **Analisis Kualitas Struktur Kode (Static Analysis)** dan **Konsumsi Energi Komputasi Murni (Dynamic Energy Profiling)** secara granular per-modul dan per-endpoint — tanpa overhead interpreter, tanpa dependensi eksternal.

---

## 🌟 Fitur Utama

1. **🦀 Rust Native Engine — Zero-Cost Abstraction:**  
   Seluruh core engine ditulis dalam Rust murni. Binary standalone ~4.5 MB, tidak membutuhkan Python, PHP, Node.js runtime, atau compiler C/GCC untuk berjalan.

2. **⚡ Kebal Bias Jaringan (I/O Wait Isolation):**  
   Hanya mengukur waktu CPU aktif murni ($t_{\text{user}} + t_{\text{system}}$) via kernel API (`GetProcessTimes` Windows / `/proc/PID/stat` Linux). Bebas dari bias latency database atau koneksi lambat yang mengacaukan alat konvensional seperti CodeCarbon.

3. **📐 Built-in Static Code Quality Analyzer:**  
   Menganalisis Cyclomatic Complexity (CCN), Maintainability Index (MI), NCLOC, dan mendeteksi arsitektur buruk (_God Class, Fat Controller, Deep Nesting_) tanpa perlu SonarQube atau Lizard.

4. **🗂️ Full-Stack Monorepo (FE + BE) Discovery:**  
   Memindai workspace monorepo dengan folder Frontend (`Next.js / React`) dan Backend (`Laravel / Python / Go`) secara simultan, menemukan rute kedua sisi secara otomatis.

5. **🔍 Automatic Route & Endpoint Discovery:**  
   Mendeteksi otomatis rute dari Next.js App Router, Laravel `routes/api.php`, tanpa konfigurasi manual.

6. **🌐 Language-Agnostic Profiling:**  
   Bekerja dari luar proses target. Mendukung **Node.js (Next.js/Express), PHP (Laravel), Python (FastAPI/Django), dan Go (Gin/Fiber)** tanpa modifikasi kode aplikasi.

7. **📄 Clean HTML Reports:**  
   Laporan HTML profesional berlatar terang, siap cetak sebagai lampiran PDF/skripsi.

---

## 📦 Panduan Instalasi

### ✅ Metode 1 — NPM Global (Rekomendasi)

Install GreenCode sebagai CLI global via NPM. NPM hanya bertugas sebagai **launcher** — binary yang berjalan adalah Rust native (`.exe`/ELF), bukan Node.js.

```bash
npm install -g --force git+https://github.com/Haekalss/GreenCode-Profiler.git
```

Verifikasi instalasi:

```bash
greencode version
# GreenCode Profiler v1.2.0 (Rust Native Architecture - Zero Cost Abstraction)
```

> **Catatan:** Perintah `greencode` di terminal diarahkan ke binary Rust `greencode.exe` secara langsung. Node.js tidak terlibat dalam proses profiling.

---

### 🦀 Metode 2 — Cargo (Untuk Developer Rust)

Clone dan build dari source menggunakan Rust toolchain:

```bash
git clone https://github.com/Haekalss/GreenCode-Profiler.git
cd GreenCode-Profiler
cargo build --release
```

Binary hasil build tersedia di:

```
target/release/greencode.exe   # Windows
target/release/greencode       # Linux / macOS
```

Tambahkan ke PATH atau salin ke folder yang ada di PATH:

```bash
# Windows (PowerShell)
Copy-Item target\release\greencode.exe $env:USERPROFILE\.cargo\bin\

# Linux / macOS
cp target/release/greencode ~/.cargo/bin/
```

---

### 📥 Metode 3 — Download Binary Langsung (GitHub Releases)

Tidak perlu Node.js, Rust, atau compiler apapun. Unduh binary standalone dari halaman [**Releases**](https://github.com/Haekalss/GreenCode-Profiler/releases):

| Platform | File |
|---|---|
| Windows x64 | `greencode-windows-x64.exe` |
| Linux x64 | `greencode-linux-x64` |
| macOS arm64 | `greencode-macos-arm64` |

Setelah diunduh, letakkan binary di folder yang ada di `PATH` sistem Anda, lalu jalankan langsung:

```bash
greencode version
```

---

## 🛠️ Perintah CLI Lengkap

| Perintah | Deskripsi |
|---|---|
| `greencode routes [dir]` | Deteksi otomatis rute dan endpoint dari codebase (Next.js, Laravel) |
| `greencode run [config.json]` | Profiling energi runtime + analisis kualitas kode → laporan HTML |
| `greencode analyze [dir]` | Analisis kualitas kode saja (CCN, LOC, MI, Smells) → `greencode_quality_report.html` |
| `greencode detect` | Pindai proses server aktif (Node, PHP, Python, Go) beserta PID dan port |
| `greencode init [--preset ...]` | Buat template konfigurasi `greencode.json` dengan rute terdeteksi otomatis |
| `greencode ui` | Buka laporan HTML dashboard di browser |
| `greencode version` | Tampilkan versi dan arsitektur engine |
| `greencode help [cmd]` | Tampilkan bantuan semua perintah atau perintah spesifik |

---

## 🚀 Alur Penggunaan Standar

1. **Pastikan server aplikasi sudah berjalan** (misal: `npm run dev` port 3000, `php artisan serve` port 8000).
2. **Masuk ke folder project:**
   ```bash
   cd path/to/project
   ```
3. **Cek rute yang terdeteksi:**
   ```bash
   greencode routes
   ```
4. **Generate konfigurasi starter:**
   ```bash
   greencode init
   ```
   _(File `greencode.json` dibuat otomatis dengan rute-rute nyata)_
5. **Jalankan Profiling:**
   ```bash
   greencode run
   ```
6. **Buka Laporan:**
   - `reports/greencode_report.html` — Profiling Energi + Kualitas Kode
   - `reports/greencode_quality_report.html` — Analisis Kualitas Kode saja

---

## ⚙️ Format Konfigurasi (`greencode.json`)

```json
{
  "project_name": "My Web Application",
  "runtime": "php",
  "source_dir": "app",
  "base_url": "http://127.0.0.1:8000",
  "repeats": 5,
  "cpu_tdp_watts": 15.0,
  "auth": {
    "type": "bearer",
    "token": "optional-jwt-token"
  },
  "endpoints": [
    { "path": "/api/users", "name": "Get Users", "method": "GET" },
    { "path": "/api/reports/export", "name": "Export Document", "method": "GET" }
  ]
}
```

Template preset tersedia di folder [`examples/`](examples/):

- [`examples/php_laravel.json`](examples/php_laravel.json)
- [`examples/node_nextjs.json`](examples/node_nextjs.json)
- [`examples/python_fastapi.json`](examples/python_fastapi.json)
- [`examples/go_fiber.json`](examples/go_fiber.json)

---

## 📊 Laporan Hasil Pengukuran (HTML Reports)

GreenCode Profiler menghasilkan laporan HTML terstruktur dan siap cetak:

### 1. `reports/greencode_report.html` (dari `greencode run`)

- **Executive Summary Cards:** Total Energi (Joule), Konsumsi Listrik (kWh), Estimasi Emisi Karbon (gCO2e), CPU Active Time (detik), Rata-rata Maintainability Index (Grade A/B/C/D).
- **Tab 1 — Green-Architecture Scorecard:** Korelasi konsumsi Joule vs CCN per-modul untuk mendeteksi _energy hotspot_.
- **Tab 2 — Static Code Quality & Smells:** Fungsi terkompleks, Fat Controller, God Class, Deep Nesting.
- **Tab 3 — Runtime Latency & Memory:** Latensi per-run, RSS RAM (MB), HTTP status code.
- **Tab 4 — Refactoring Action Plan:** Rekomendasi refactoring berbasis _green architecture_.

### 2. `reports/greencode_quality_report.html` (dari `greencode analyze`)

- Analisis statis dengan skor CCN rata-rata & maksimum.
- Tabel rincian kompleksitas seluruh fungsi/kelas di seluruh codebase.
- Deteksi pelanggaran arsitektur (_Architectural Smells_) siap ekspor ke dokumen teknis/skripsi.

---

## 🏗️ Arsitektur Internal (v1.2.0)

```
GreenCode Profiler v1.2.0
├── src/
│   ├── main.rs        — CLI entrypoint (init/routes/detect/analyze/run/ui/version)
│   ├── engine.rs      — ProcessEnergyEngine (kernel CPU times, Windows FFI / Linux proc)
│   ├── detector.rs    — RuntimeDetector (PID via netstat) & RouteDiscoverer
│   ├── analyzer.rs    — CodeQualityAnalyzer (NCLOC, CCN, MI, Code Smells)
│   ├── attributor.rs  — ModuleAttributor + HTTP tracer native via TcpStream
│   ├── correlator.rs  — GreenCorrelator (korelasi metrik energi & kualitas kode)
│   └── reporter.rs    — HTML & JSON report generator
├── bin/
│   ├── greencode.js   — NPM launcher (menjalankan Rust binary langsung)
│   └── greencode.exe  — Rust binary untuk distribusi NPM (Windows)
└── Cargo.toml         — Dependencies: serde, serde_json, regex (pure-Rust only)
```

**Zero external C dependencies.** Tidak ada `gcc`, `dlltool`, `OpenSSL`, atau `ring`.

---

## 📄 Lisensi

Didistribusikan di bawah lisensi open-source **MIT License**. Lihat [`LICENSE`](LICENSE) untuk informasi lebih lanjut.
