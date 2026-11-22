<p align="center">
  <img src="assets/logo.png" alt="GreenCode Profiler Logo" width="180" />
</p>

# GreenCode Profiler

**Universal Process-Level Energy & Code Quality Profiler for Modern Web Applications**

[![License: MIT](https://img.shields.io/badge/License-MIT-emerald.svg)](LICENSE)
[![Engine: Rust](https://img.shields.io/badge/Engine-Rust%20Native%201.100+-F74C00.svg?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Runtime: Standalone](https://img.shields.io/badge/Runtime-Zero%20Dependency-10b981.svg)]()
[![Distribution: NPM](https://img.shields.io/badge/NPM-Launcher-339933.svg?logo=node.js&logoColor=white)](https://nodejs.org)
[![Version](https://img.shields.io/badge/Version-1.8.1-blue.svg)](https://github.com/Haekalss/GreenCode-Profiler/releases)

GreenCode Profiler (GCP) v1.8.1 adalah framework audit efisiensi energi perangkat lunak (_Green Software Engineering_), arsitektur kode (_Static Code Quality Analysis_), dan runtime environment audit (_PHP Extensions & Green Optimization_). Dibangun dengan arsitektur **Rust Native**, framework ini mengisolasi waktu CPU kernel dari bias jaringan I/O dan menghasilkan metrik komputasi yang objektif tanpa membutuhkan interpreter atau runtime eksternal.

---

## Gambaran Umum

GreenCode Profiler dirancang dengan prinsip **Zero-Dependency Runtime**. Pengguna akhir (_end-user_, auditor, atau pengembang aplikasi) **tidak diwajibkan** memasang compiler Rust, Python runtime, atau PHP runtime di sistem operasi mereka. Aplikasi didistribusikan dalam bentuk biner mandiri (_standalone executable_) siap pakai.

### Fitur Utama

1. **Rust Native Engine:** Menggunakan abstraksi berbiaya nol (_zero-cost abstractions_) dengan konsumsi memori minimal dan kecepatan eksekusi tinggi.
2. **Isolasi Bias Jaringan (I/O Wait Isolation):** Pengukuran energi didasarkan pada waktu aktif kernel CPU ($t_{\text{user}} + t_{\text{system}}$) melalui Windows NT API (`GetProcessTimes`) dan Linux `/proc/[PID]/stat`, sehingga kebal terhadap latensi koneksi internet atau antrean query basis data.
3. **Analisis Kualitas Kode Bawaan:** Memeriksa Cyclomatic Complexity (CCN), Maintainability Index (MI), Non-Comment Lines of Code (NCLOC), serta mendeteksi _Architectural Code Smells_ (_God Class_, _Fat Controller_, _Deep Nesting_).
4. **PHP Runtime & Green Extension Audit:** Mengaudit ketersediaan ekstensi PHP kritis (MySQL, OpenSSL, cURL), akselerator penghemat siklus CPU (_Zend OPcache_, _JIT Compiler_), dan peringatan ekstensi boros daya (_Xdebug_).
5. **Dukungan Monorepo & Multi-Service:** Mendeteksi dan memprofil arsitektur monorepo (Frontend Next.js dan Backend Laravel) secara simultan.
6. **Bahasa-Agnostik:** Bekerja dari luar proses aplikasi target (_external process profiling_). Mendukung Node.js, PHP, Python, Go, dan Rust.
7. **Laporan HTML Komprehensif:** Menghasilkan dokumen laporan profesional berlatar terang dengan logo tersemat (_offline base64_), indikator ambang batas dinamis, rekomendasi runtime PHP, dan deskripsi parameter teknis.
8. **Deteksi Rute Lintas Bahasa Tanpa Batas:** Memindai seluruh rute aplikasi tanpa batas jumlah, termasuk rute bersarang dan _prefix group_, untuk Node.js (Next.js, Express, NestJS), PHP (Laravel), Python (Flask, FastAPI, Django), Go (net/http, gin, chi), dan Rust (axum, actix). Rute berparameter otomatis dikecualikan dari _profiling_ agar tidak menghasilkan HTTP 404.
9. **Laporan Teragregasi per Endpoint:** Setiap endpoint ditampilkan dalam **satu baris berisi rata-rata** dari seluruh pengulangan pengukuran, sehingga laporan tidak mengulang endpoint sebanyak jumlah run.
10. **Filter Endpoint & Kode (`--filter` / `--exclude`):** Menyaring endpoint **dan file kode** dengan kata kunci yang sama — boleh diulang (logika OR), cocok tanpa memedulikan huruf besar/kecil pada path maupun isi file. Berlaku untuk `routes`, `init`, `run`, dan `analyze`; penyaringan dilakukan *in-memory* sehingga `greencode.json` tidak berubah. Pada monorepo, semua `source_dirs` (frontend + backend) tetap ditelusuri sehingga kedua bahasa ikut terdeteksi, lalu file disaring per kata kunci.
11. **Output Laporan Kustom (`--output`):** Menyimpan laporan ke path pilihan, mis. `greencode run --filter smki --output reports/smki.html`. Berkas JSON otomatis memakai nama dasar yang sama (`reports/smki.json`), folder tujuan dibuat bila belum ada, dan laporan default tidak tertimpa — sehingga hasil beberapa filter bisa disimpan berdampingan.

---

## Panduan Instalasi dan Penggunaan

Pilih salah satu metode instalasi yang sesuai dengan alur kerja Anda:

### Metode 1: Biner Portabel (Rekomendasi Utama - Tanpa Node.js / Zero Dependency)

Metode ini **sama sekali tidak membutuhkan instalasi dependensi apapun** (tidak perlu Node.js, Python, atau Rust). Sangat cocok untuk pengguna umum, penguji, atau developer bahasa apapun.

1. Unduh arsip rilis resmi: [GreenCode-v1.8.1-windows-x64.zip](https://github.com/Haekalss/GreenCode-Profiler/releases/latest).
2. Ekstrak arsip tersebut untuk mendapatkan berkas `greencode.exe`.
3. Tempatkan berkas `greencode.exe` di direktori utama (_root_) proyek web yang akan diuji.
4. Pastikan server aplikasi web Anda sedang berjalan (contoh: port 3000 atau port 8000).
5. Jalankan perintah berikut melalui terminal (PowerShell / Command Prompt):

```powershell
# 1. Inisialisasi konfigurasi dan deteksi rute otomatis
.\greencode.exe init

# 2. Jalankan audit energi dan kualitas kode
.\greencode.exe run
```

Laporan HTML hasil audit akan dibuat di `reports/greencode_report.html` dan otomatis terbuka di peramban web.

---

### Metode 2: Instalasi Global Melalui Package Manager

GreenCode Profiler menyediakan pembungkus (_launcher_) di berbagai manajer paket agar Anda dapat menggunakannya sebagai perintah global di sistem sesuai preferensi ekosistem:

#### Opsi A: Node.js (npm)

```bash
npm install -g --force git+https://github.com/Haekalss/GreenCode-Profiler.git
```

#### Opsi B: Python (pip)

```bash
pip install git+https://github.com/Haekalss/GreenCode-Profiler.git
```

#### Opsi C: PHP (Composer Dev Tool)

```bash
composer require --dev greencode/profiler
# Jalankan via: ./vendor/bin/greencode
```

#### Opsi D: Go (go install)

```bash
go install github.com/Haekalss/GreenCode-Profiler/cmd/greencode@latest
```

Setelah terpasang, perintah `greencode` (atau `./vendor/bin/greencode` pada PHP) dapat dipanggil langsung dari direktori mana saja:

```bash
greencode init
greencode run
```

---

### Metode 3: Kompilasi dari Kode Sumber (Khusus Pengembang Rust)

Jika Anda ingin memodifikasi atau mengembangkan modul inti GreenCode Profiler:

```bash
git clone https://github.com/Haekalss/GreenCode-Profiler.git
cd GreenCode-Profiler
cargo build --release
```

Berkas biner hasil kompilasi tersedia di `target/release/greencode.exe`.

---

## Daftar Perintah CLI

| Perintah            | Parameter                   | Deskripsi                                                                |
| ------------------- | --------------------------- | ------------------------------------------------------------------------ |
| `greencode init`    | `[--force] [--filter K] [--exclude K]` | Membuat `greencode.json` dengan deteksi rute otomatis (tersaring) |
| `greencode run`     | `[config.json] [--no-open] [--filter K] [--exclude K] [--output file]` | Menjalankan profiling energi dan kualitas struktur kode |
| `greencode analyze` | `[config.json] [--no-open] [--filter K] [--exclude K] [--output file]` | Menjalankan analisis kualitas kode statis (NCLOC, CCN, MI, Smells) dengan filter file |
| `greencode routes`  | `[dir] [--limit N] [--filter K] [--exclude K]` | Memindai seluruh rute nyata dari kode sumber (semua bahasa; `--limit` membatasi jumlah) |
| `greencode detect`  | `[language]`                | Memindai proses server aktif beserta nomor Process ID (PID) dan port     |
| `greencode ui`      | -                           | Membuka laporan dashboard HTML hasil profiling terakhir di peramban      |
| `greencode version` | -                           | Menampilkan informasi versi dan arsitektur engine                        |
| `greencode help`    | `[cmd]`                     | Menampilkan panduan penggunaan perintah                                  |

---

## Struktur Konfigurasi (`greencode.json`)

Contoh konfigurasi standar untuk lingkungan monorepo:

```json
{
  "project_name": "Full-Stack Monorepo (frontend + backend)",
  "runtime": "fullstack",
  "base_url": "http://127.0.0.1:8000",
  "services": {
    "frontend": "http://127.0.0.1:3000",
    "backend": "http://127.0.0.1:8000"
  },
  "source_dirs": ["backend/app", "frontend/src"],
  "repeats": 5,
  "cpu_tdp_watts": 15.0,
  "endpoints": [
    {
      "name": "[FE] Halaman Dashboard",
      "method": "GET",
      "path": "/admin/dashboard",
      "service": "frontend",
      "url": "http://127.0.0.1:3000/admin/dashboard"
    },
    {
      "name": "[BE] API Services",
      "method": "GET",
      "path": "/api/services",
      "service": "backend",
      "url": "http://127.0.0.1:8000/api/services"
    }
  ]
}
```

---

## Parameter dan Indikator Hasil Audit

Laporan HTML (`reports/greencode_report.html`) menyajikan metrik evaluasi dengan klasifikasi ambang batas objektif:

| Parameter                 | Satuan / Skala  | Ambang Batas Optimal   | Keterangan Teknis                                                                                  |
| ------------------------- | --------------- | ---------------------- | -------------------------------------------------------------------------------------------------- |
| **Total Energi**          | Joule (J) / kWh | < 2.0 J per transaksi  | Total energi listrik murni yang ditarik thread CPU ($E = P_{\text{avg}} \times t_{\text{active}}$) |
| **Waktu CPU Aktif**       | Detik (s)       | Sesuai beban logika    | Durasi aktif prosesor mengeksekusi instruksi, bebas dari waktu tunggu I/O                          |
| **Green Score**           | 0 – 100         | $\ge$ 85 (Optimal)     | Indeks efisiensi rasio konsumsi energi terhadap throughput dan latensi                             |
| **CCN**                   | Angka Bulat     | 1 – 10 (Rendah / Aman) | Cyclomatic Complexity Number, mengukur jalur percabangan logika independen                         |
| **Maintainability Index** | 0 – 100         | $\ge$ 85 (Grade A)     | Indeks kemudahan pemeliharaan berbasis perpaduan logaritma Halstead, CCN, dan NCLOC                |
| **Code Smells**           | Jumlah Temuan   | 0 temuan               | Deteksi pelanggaran arsitektur (_God Class_, _Fat Controller_, _Deep Nesting_)                     |

---

## Arsitektur Repositori

```
GreenCode-Profiler/
├── src/
│   ├── main.rs         - CLI dispatch, konfigurasi, dan alur eksekusi
│   ├── engine.rs       - ProcessEnergyEngine (Windows FFI / Linux procfs)
│   ├── detector.rs     - RuntimeDetector dan RouteDiscoverer
│   ├── analyzer.rs     - CodeQualityAnalyzer (NCLOC, CCN, MI, Smells)
│   ├── attributor.rs   - ModuleAttributor dan HTTP tracing berbasis TcpStream
│   ├── correlator.rs   - Korelasi data energi dan metrik kualitas kode
│   ├── reporter.rs     - Generator dokumen HTML dan JSON
│   └── logo_base64.txt - Aset logo tersemat (offline base64)
├── bin/
│   ├── greencode.js    - Skrip runner untuk distribusi NPM
│   └── greencode.exe   - Biner kompilasi rilis Windows x64
├── Cargo.toml          - Konfigurasi dependensi murni Rust
└── LICENSE             - Lisensi sumber terbuka MIT
```

---

## Lisensi

Proyek ini didistribusikan di bawah lisensi terbuka [MIT License](LICENSE).
