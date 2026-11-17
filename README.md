<p align="center">
  <img src="assets/logo.png" alt="GreenCode Profiler Logo" width="220" />
</p>

# GreenCode Profiler (GCP)

**Universal Process-Level Energy & Code Quality Profiler for Modern Web Applications**

[![License: MIT](https://img.shields.io/badge/License-MIT-emerald.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Engine-Rust%20Native%201.100+-F74C00.svg?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Zero Dependency](https://img.shields.io/badge/Runtime-Zero%20Dependency%20(Standalone)-10b981.svg)]()
[![Node.js](https://img.shields.io/badge/NPM-Launcher-339933.svg?logo=node.js&logoColor=white)](https://nodejs.org)
[![Version](https://img.shields.io/badge/Version-1.2.0-blue.svg)]()

> **GreenCode Profiler v1.2.0** adalah framework audit efisiensi energi perangkat lunak (*Green Software Engineering*) berbasis **Rust Native Engine** yang menggabungkan **Analisis Kualitas Struktur Kode (Static Analysis)** dan **Konsumsi Energi Komputasi Murni (Dynamic Energy Profiling)** secara granular per-modul dan per-endpoint — tanpa overhead interpreter, tanpa dependensi eksternal.

---

## 💡 Apakah Pengguna Harus Install Rust di Komputernya?

> ### 🛑 **JAWABAN SINGKAT: TIDAK PERLU!**
> 
> GreenCode Profiler memang **dibangun menggunakan bahasa Rust**, tetapi hasil akhirnya telah dikompilasi menjadi sebuah **Native Standalone Executable Binary (`greencode.exe`)**.
> 
> * ❌ **TIDAK PERLU** install Rust compiler (`rustc` / `cargo`).
> * ❌ **TIDAK PERLU** install Python atau PHP runtime.
> * ❌ **TIDAK PERLU** install library C / GCC / Visual C++ Redistributable.
> * ✅ **Pengguna umum cukup download file ZIP / `.exe`**, ekstrak ke folder mana saja, dan aplikasi bisa langsung dijalankan!
>
> *(Rust hanya dibutuhkan oleh developer yang ingin memodifikasi atau meng-compile ulang kode sumber GreenCode).*

---

## 🌟 Fitur Utama

1. **🦀 Rust Native Engine — Zero-Cost Abstraction:**  
   Seluruh core engine ditulis dalam Rust murni. Binary standalone ~4.5 MB mandiri, tanpa runtime eksternal.

2. **⚡ Kebal Bias Jaringan (I/O Wait Isolation):**  
   Hanya mengukur waktu CPU aktif murni ($t_{\text{user}} + t_{\text{system}}$) via kernel API (`GetProcessTimes` Windows / `/proc/PID/stat` Linux). Bebas dari bias latency database atau koneksi internet lambat yang mengacaukan alat konvensional seperti CodeCarbon.

3. **📐 Built-in Static Code Quality Analyzer:**  
   Menganalisis Cyclomatic Complexity (CCN), Maintainability Index (MI), NCLOC, dan mendeteksi arsitektur buruk (*God Class, Fat Controller, Deep Nesting*) tanpa perlu SonarQube atau Lizard.

4. **🗂️ Full-Stack Monorepo (FE + BE) Discovery:**  
   Memindai workspace monorepo dengan folder Frontend (`Next.js / React`) dan Backend (`Laravel / Python / Go`) secara simultan, memprofil kedua port (misal 3000 & 8000) dan menyatukan hasilnya dalam satu laporan utuh.

5. **🔍 Automatic Route & Endpoint Discovery:**  
   Mendeteksi otomatis rute dari Next.js App Router dan Laravel `routes/api.php` secara cerdas (otomatis memfilter rute dinamis dan mencocokkan port).

6. **🌐 Language-Agnostic Profiling:**  
   Bekerja dari luar proses target. Mendukung **Node.js (Next.js/Express), PHP (Laravel), Python (FastAPI/Django), dan Go (Gin/Fiber)** tanpa modifikasi kode aplikasi target.

7. **📄 Print-Friendly HTML Reports dengan Logo Resmi:**  
   Menghasilkan laporan HTML profesional berlatar terang dengan logo GreenCode tersemat rapi (offline base64), badge warna peringatan dinamis, dan box penjelasan parameter lengkap untuk lampiran skripsi/magang.

---

## 📦 Pilihan Cara Pakai & Instalasi

Pilih cara yang paling mudah sesuai kenyamanan Anda:

### 🟢 Cara 1 — Paling Mudah untuk Pengguna Umum (Standalone Portable ZIP)
*Cocok untuk dosen, penguji, rekan tim, atau pengguna yang tidak mau ribet install apa-apa.*

1. Unduh paket portable dari rilis repositori:
   * 📥 **[Download GreenCode-v1.2.0-windows-x64.zip](https://github.com/Haekalss/GreenCode-Profiler/releases/latest)**
2. Ekstrak file `.zip` tersebut.
3. Di dalamnya sudah ada file `greencode.exe` siap pakai!
4. Buka terminal (CMD / PowerShell) di folder tersebut, lalu jalankan:
   ```bash
   .\greencode.exe version
   .\greencode.exe init
   .\greencode.exe run
   ```

---

### 🟢 Cara 2 — Via NPM Global (Rekomendasi untuk Web Developer)
*Cocok jika Anda sudah terbiasa dengan Node.js / NPM di komputer Anda.*

NPM hanya bertindak sebagai *launcher* pintasan global; binary yang dijalankan di latar belakang tetaplah Rust native murni:

```bash
npm install -g --force git+https://github.com/Haekalss/GreenCode-Profiler.git
```

Setelah itu perintah `greencode` langsung aktif di terminal mana saja:
```bash
greencode version
```

---

### 🦀 Cara 3 — Compile Manual via Cargo (Khusus Developer Rust)
*Hanya jika Anda ingin memodifikasi source code GreenCode Profiler sendiri.*

```bash
git clone https://github.com/Haekalss/GreenCode-Profiler.git
cd GreenCode-Profiler
cargo build --release
```
Binary hasil kompilasi akan berada di `target/release/greencode.exe`.

---

## 🛠️ Perintah CLI Lengkap

| Perintah | Deskripsi |
|---|---|
| `greencode routes [dir]` | Deteksi otomatis rute dan endpoint nyata dari codebase (Next.js, Laravel) |
| `greencode init [--force]` | Buat template konfigurasi `greencode.json` cerdas sesuai posisi folder (FE/BE/Monorepo) |
| `greencode run [config.json]` | Jalankan profiling energi runtime + analisis kualitas kode → laporan HTML |
| `greencode analyze [dir]` | Analisis kualitas kode statis saja (CCN, LOC, MI, Smells) → `reports/code_quality.html` |
| `greencode detect` | Pindai proses server aktif (Node, PHP, Python, Go) beserta PID dan port-nya |
| `greencode ui` | Buka laporan HTML dashboard di browser |
| `greencode version` | Tampilkan versi dan status arsitektur engine |
| `greencode help [cmd]` | Tampilkan panduan bantuan semua perintah |

---

## 🚀 Alur Penggunaan Standar

1. **Pastikan server aplikasi target Anda sudah berjalan** (misal: `npm run dev` di port 3000 atau `php artisan serve` di port 8000).
2. **Buka terminal di folder project Anda:**
   ```bash
   cd path/to/project
   ```
3. **Inisialisasi konfigurasi otomatis:**
   ```bash
   greencode init
   ```
   *(Engine otomatis mendeteksi apakah Anda berada di folder FE, BE, atau monorepo, mencocokkan port, dan menyusun rute yang siap diprofil).*
4. **Jalankan Profiling:**
   ```bash
   greencode run
   ```
5. **Buka Laporan:**
   Laporan komprehensif akan langsung terbuka di browser Anda:
   - `reports/greencode_report.html` (Hasil gabungan Profiling Energi + Kualitas Struktur Kode)
   - `reports/code_quality.html` (Hasil Analisis Kualitas Kode statis)

---

## ⚙️ Format Konfigurasi (`greencode.json`)

Contoh file konfigurasi hasil deteksi monorepo otomatis:

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
      "name": "[FE] Halaman admin dashboard",
      "method": "GET",
      "path": "/admin/dashboard",
      "service": "frontend",
      "url": "http://127.0.0.1:3000/admin/dashboard"
    },
    {
      "name": "[BE] API services",
      "method": "GET",
      "path": "/api/services",
      "service": "backend",
      "url": "http://127.0.0.1:8000/api/services"
    }
  ]
}
```

---

## 📊 Laporan Hasil Audit (HTML Reports)

GreenCode Profiler menghasilkan laporan HTML interaktif, profesional, dan siap cetak (*print-friendly*):

### `reports/greencode_report.html` (Hasil `greencode run`)
- **Branded Header:** Menampilkan logo resmi GreenCode Profiler (embedded offline base64).
- **Executive Metric Cards:** Total Energi Murni (Joule & kWh ringkas), Waktu Aktif CPU Murni (detik), Emisi Karbon (gCO2e), dan Skor Maintainability Index (Grade A/B/C).
- **Panduan Edukatif Parameter:** Kotak penjelasan resmi arti setiap metrik energi dan kualitas kode.
- **Tabel 1 — Efisiensi Energi per Modul:** Dilengkapi **Threshold Color Badges** (Hijau: Efisien, Kuning: Moderat, Merah: Boros / Hotspot).
- **Tabel 2 — Detail Transaksi HTTP:** Log latensi milidetik dan status HTTP per transaksi.
- **Tabel 3 — Static Code Quality & Smells:** Ringkasan NCLOC, Cyclomatic Complexity (CCN), dan deteksi *God Class, Fat Controller, Deep Nesting*.
- **Tabel 4 — Rincian Metrik per Berkas:** Kompleksitas dan Maintainability Index setiap file sumber aplikasi.

---

## 🏗️ Arsitektur Internal (v1.2.0)

```
GreenCode Profiler v1.2.0
├── src/
│   ├── main.rs        — CLI entrypoint & auto port detection
│   ├── engine.rs      — ProcessEnergyEngine (kernel CPU times via Windows FFI / Linux proc)
│   ├── detector.rs    — RuntimeDetector (PID netstat) & Smart RouteDiscoverer
│   ├── analyzer.rs    — CodeQualityAnalyzer (NCLOC, CCN, MI, Code Smells)
│   ├── attributor.rs  — ModuleAttributor + HTTP tracer native via TcpStream
│   ├── correlator.rs  — GreenCorrelator (korelasi metrik energi & kualitas kode)
│   ├── reporter.rs    — HTML & JSON report generator (with embedded base64 logo)
│   └── logo_base64.txt— Embedded official logo
├── bin/
│   ├── greencode.js   — NPM launcher global
│   └── greencode.exe  — Pre-compiled Standalone Windows x64 Native Binary
└── Cargo.toml         — Zero C dependencies (serde, serde_json, regex)
```

---

## 📄 Lisensi

Didistribusikan di bawah lisensi open-source **MIT License**. Lihat [`LICENSE`](LICENSE) untuk informasi lebih lanjut.
