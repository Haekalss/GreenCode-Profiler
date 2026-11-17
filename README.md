<p align="center">
  <img src="assets/logo.png" alt="GreenCode Profiler Logo" width="180" />
</p>

# GreenCode Profiler

**Universal Process-Level Energy & Code Quality Profiler for Modern Web Applications**

[![License: MIT](https://img.shields.io/badge/License-MIT-emerald.svg)](LICENSE)
[![Engine: Rust](https://img.shields.io/badge/Engine-Rust%20Native%201.100+-F74C00.svg?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Runtime: Standalone](https://img.shields.io/badge/Runtime-Zero%20Dependency-10b981.svg)]()
[![Distribution: NPM](https://img.shields.io/badge/NPM-Launcher-339933.svg?logo=node.js&logoColor=white)](https://nodejs.org)
[![Version](https://img.shields.io/badge/Version-1.2.0-blue.svg)](https://github.com/Haekalss/GreenCode-Profiler/releases)

GreenCode Profiler (GCP) v1.2.0 adalah framework audit efisiensi energi perangkat lunak (*Green Software Engineering*) dan arsitektur kode (*Static Code Quality Analysis*). Dibangun dengan arsitektur **Rust Native**, framework ini mengisolasi waktu CPU kernel dari bias jaringan I/O dan menghasilkan metrik komputasi yang objektif tanpa membutuhkan interpreter atau runtime eksternal.

---

## Gambaran Umum

GreenCode Profiler dirancang dengan prinsip **Zero-Dependency Runtime**. Pengguna akhir (*end-user*, auditor, atau pengembang aplikasi) **tidak diwajibkan** memasang compiler Rust, Python runtime, atau PHP runtime di sistem operasi mereka. Aplikasi didistribusikan dalam bentuk biner mandiri (*standalone executable*) siap pakai.

### Fitur Utama

1. **Rust Native Engine:** Menggunakan abstraksi berbiaya nol (*zero-cost abstractions*) dengan konsumsi memori minimal dan kecepatan eksekusi tinggi.
2. **Isolasi Bias Jaringan (I/O Wait Isolation):** Pengukuran energi didasarkan pada waktu aktif kernel CPU ($t_{\text{user}} + t_{\text{system}}$) melalui Windows NT API (`GetProcessTimes`) dan Linux `/proc/[PID]/stat`, sehingga kebal terhadap latensi koneksi internet atau antrean query basis data.
3. **Analisis Kualitas Kode Bawaan:** Memeriksa Cyclomatic Complexity (CCN), Maintainability Index (MI), Non-Comment Lines of Code (NCLOC), serta mendeteksi *Architectural Code Smells* (*God Class*, *Fat Controller*, *Deep Nesting*).
4. **Dukungan Monorepo & Multi-Service:** Mendeteksi dan memprofil arsitektur monorepo (Frontend Next.js dan Backend Laravel) secara simultan.
5. **Bahasa-Agnostik:** Bekerja dari luar proses aplikasi target (*external process profiling*). Mendukung Node.js, PHP, Python, dan Go.
6. **Laporan HTML Komprehensif:** Menghasilkan dokumen laporan profesional berlatar terang dengan logo tersemat (*offline base64*), indikator ambang batas dinamis, dan deskripsi parameter teknis.

---

## Panduan Instalasi dan Penggunaan

Pilih salah satu metode instalasi yang sesuai dengan alur kerja Anda:

### Metode 1: Biner Portabel (Rekomendasi untuk Pengguna Umum)

Metode ini tidak membutuhkan instalasi dependensi apapun.

1. Unduh arsip rilis resmi: [GreenCode-v1.2.0-windows-x64.zip](https://github.com/Haekalss/GreenCode-Profiler/releases/latest).
2. Ekstrak arsip tersebut untuk mendapatkan berkas `greencode.exe`.
3. Tempatkan berkas `greencode.exe` di direktori utama (*root*) proyek web yang akan diuji.
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

### Metode 2: CLI Global via NPM

Bagi lingkungan pengembangan yang telah memiliki Node.js, GreenCode dapat dipasang sebagai perintah global:

```bash
npm install -g --force git+https://github.com/Haekalss/GreenCode-Profiler.git
```

Setelah terpasang, perintah `greencode` dapat dipanggil langsung dari direktori mana saja:

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

| Perintah | Parameter | Deskripsi |
|---|---|---|
| `greencode init` | `[--force]` | Membuat berkas konfigurasi `greencode.json` dengan deteksi rute otomatis |
| `greencode run` | `[config.json] [--no-open]` | Menjalankan profiling energi runtime dan analisis kualitas struktur kode |
| `greencode analyze` | `[dir] [--no-open]` | Menjalankan analisis kualitas kode statis (NCLOC, CCN, MI, Smells) |
| `greencode routes` | `[dir]` | Memindai dan menampilkan rute nyata dari kode sumber aplikasi target |
| `greencode detect` | `[language]` | Memindai proses server aktif beserta nomor Process ID (PID) dan port |
| `greencode ui` | - | Membuka laporan dashboard HTML hasil profiling terakhir di peramban |
| `greencode version` | - | Menampilkan informasi versi dan arsitektur engine |
| `greencode help` | `[cmd]` | Menampilkan panduan penggunaan perintah |

---

## Struktur Konfigurasi (`greencode.json`)

Contoh konfigurasi standar untuk lingkungan monorepo:

```json
{
  "project_name": "Full-Stack Monorepo (aptika_fe + aptika_be)",
  "runtime": "fullstack",
  "base_url": "http://127.0.0.1:8000",
  "services": {
    "frontend": "http://127.0.0.1:3000",
    "backend": "http://127.0.0.1:8000"
  },
  "source_dirs": [
    "aptika_be/app",
    "aptika_fe/src"
  ],
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

| Parameter | Satuan / Skala | Ambang Batas Optimal | Keterangan Teknis |
|---|---|---|---|
| **Total Energi** | Joule (J) / kWh | < 2.0 J per transaksi | Total energi listrik murni yang ditarik thread CPU ($E = P_{\text{avg}} \times t_{\text{active}}$) |
| **Waktu CPU Aktif** | Detik (s) | Sesuai beban logika | Durasi aktif prosesor mengeksekusi instruksi, bebas dari waktu tunggu I/O |
| **Green Score** | 0 – 100 | $\ge$ 85 (Optimal) | Indeks efisiensi rasio konsumsi energi terhadap throughput dan latensi |
| **CCN** | Angka Bulat | 1 – 10 (Rendah / Aman) | Cyclomatic Complexity Number, mengukur jalur percabangan logika independen |
| **Maintainability Index** | 0 – 100 | $\ge$ 85 (Grade A) | Indeks kemudahan pemeliharaan berbasis perpaduan logaritma Halstead, CCN, dan NCLOC |
| **Code Smells** | Jumlah Temuan | 0 temuan | Deteksi pelanggaran arsitektur (*God Class*, *Fat Controller*, *Deep Nesting*) |

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
