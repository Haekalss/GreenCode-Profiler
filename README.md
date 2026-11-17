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
> * ✅ **Pengguna umum cukup download file ZIP**, ekstrak file `greencode.exe`, dan langsung jalankan di folder project manapun!
>
> *(Rust hanya dibutuhkan oleh developer yang ingin memodifikasi atau meng-compile ulang kode sumber GreenCode).*

---

## 📦 Panduan Praktis Penggunaan `greencode.exe` (Standalone Portable)

Jika Anda mengunduh paket **`GreenCode-v1.2.0-windows-x64.zip`**, ikuti panduan praktis berikut:

### 🚀 Cara A — Penggunaan Cepat (Langsung Taruh di Folder Project)
*Paling mudah jika Anda hanya ingin menguji satu project web tertentu.*

1. **Ekstrak file ZIP**, ambil file **`greencode.exe`**.
2. **Copy / Salin** file `greencode.exe` ke dalam folder project web yang ingin Anda profil (misal di root project Laravel, Next.js, Express, atau FastAPI).
3. **Pastikan server web Anda sudah menyala** di terminal lain (misal: `npm run dev` di port 3000, atau `php artisan serve` di port 8000).
4. **Buka Terminal (PowerShell / Command Prompt)** di folder project tersebut, lalu jalankan:

   * **Langkah 1: Cek Versi**
     ```powershell
     .\greencode.exe version
     ```
   * **Langkah 2: Buat Konfigurasi Otomatis (Auto-Detect Routes)**
     ```powershell
     .\greencode.exe init
     ```
     *(Engine otomatis memindai rute halaman Frontend atau endpoint API Backend dan membuat file `greencode.json`)*.
   * **Langkah 3: Jalankan Profiling Energi & Kualitas Kode**
     ```powershell
     .\greencode.exe run
     ```
   * **Langkah 4: Hasil Laporan**
     Laporan audit HTML interaktif dengan logo resmi akan langsung terbuka secara otomatis di browser Anda di `reports/greencode_report.html`!

---

### 🌐 Cara B — Jadikan Perintah Global di Windows (Rekomendasi Jangka Panjang)
*Agar perintah `greencode` bisa langsung dipanggil di folder mana saja tanpa perlu copy-paste file `.exe` berkali-kali.*

1. Buat folder baru di komputer Anda, misal: `C:\Tools\GreenCode\`
2. Letakkan file `greencode.exe` di dalam folder tersebut.
3. Daftarkan folder tersebut ke **Environment Variables (PATH)** Windows:
   * Tekan tombol **Windows**, ketik **env**, pilih **Edit the system environment variables**.
   * Klik tombol **Environment Variables...** di kanan bawah.
   * Pada bagian **User variables**, pilih **Path** lalu klik **Edit...**
   * Klik tombol **New**, lalu masukkan: `C:\Tools\GreenCode`
   * Klik **OK** $\rightarrow$ **OK** $\rightarrow$ **OK**.
4. **Selesai!** Buka terminal baru di folder project mana saja, Anda cukup mengetik:
   ```bash
   greencode init
   greencode run
   ```

---

### 🟢 Cara C — Via NPM Global (Untuk Web Developer)
Jika di komputer Anda sudah terbiasa menggunakan Node.js:
```bash
npm install -g --force git+https://github.com/Haekalss/GreenCode-Profiler.git
```
Lalu panggil `greencode run` di terminal.

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
