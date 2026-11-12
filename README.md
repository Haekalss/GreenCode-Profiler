# ⚡ GreenCode Profiler (GCP)
**Universal Process-Level Energy & Code Quality Profiler for Modern Web Applications**

[![License: MIT](https://img.shields.io/badge/License-MIT-emerald.svg)](LICENSE)
[![Python](https://img.shields.io/badge/Python-3.9+-3776AB.svg?logo=python&logoColor=white)](https://python.org)
[![Node.js](https://img.shields.io/badge/Node.js-16+-339933.svg?logo=node.js&logoColor=white)](https://nodejs.org)
[![PHP](https://img.shields.io/badge/PHP-8.0+-777BB4.svg?logo=php&logoColor=white)](https://php.net)
[![Go](https://img.shields.io/badge/Go-1.20+-00ADD8.svg?logo=go&logoColor=white)](https://golang.org)
[![Architecture](https://img.shields.io/badge/Architecture-Clean%20%26%20Green-10b981.svg)]()

> **GreenCode Profiler** adalah framework pengukuran efisiensi energi perangkat lunak (*Green Software Engineering*) yang menggabungkan **Analisis Kualitas Struktur Kode (Static Analysis)** dan **Konsumsi Energi Komputasi Murni (Dynamic Energy Profiling)** secara granular per-modul dan per-endpoint.

---

## 🌟 Fitur Utama (The Complete Package)

1. **Kebal Bias Jaringan (I/O Wait Isolation):**  
   Hanya mengukur waktu CPU aktif murni ($t_{\text{user}} + t_{\text{system}}$). Bebas dari bias latency database atau koneksi internet lambat yang sering mengacaukan alat ukur konvensional seperti CodeCarbon.
2. **Built-in Static Code Quality Analyzer:**  
   Menganalisis Cyclomatic Complexity (CCN), Maintainability Index (MI), NCLOC, dan mendeteksi arsitektur buruk (*God Class, Fat Controller, Deep Nesting*) secara instan tanpa perlu menginstall SonarQube atau Lizard.
3. **Automatic Route & Endpoint Discovery:**  
   Mendeteksi otomatis seluruh halaman dan rute API nyata dari struktur kode project (Next.js App/Pages Router, Laravel Routes, Express, FastAPI, Go) tanpa perlu konfigurasi manual.
4. **Universal & Language-Agnostic:**  
   Bekerja dari luar proses. Mendukung **Node.js (Next.js/Express), PHP (Laravel), Python (FastAPI/Django), dan Go (Gin/Fiber)** tanpa modifikasi kode aplikasi target.
5. **Clean & Print-Friendly HTML Reports:**  
   Menghasilkan laporan HTML yang bersih, profesional, berlatar terang (putih), siap dicetak sebagai lampiran PDF dokumen teknis/skripsi, baik untuk profiling energi (`greencode run`) maupun analisis kualitas kode (`greencode analyze`).

---

## 📦 Panduan Instalasi (Via GitHub Repository)

GreenCode Profiler dapat di-install langsung dari repositori GitHub resmi:  
`https://github.com/Haekalss/GreenCode-Profiler.git`

Pilih metode instalasi sesuai ekosistem bahasa dan project Anda:

### 🟢 1. Node.js / Frontend (NPM)
Cocok untuk project **Next.js, React, Express, atau CLI global**.

**A. Install Global CLI (Rekomendasi - Bisa dipakai di semua folder):**
```bash
npm install -g --force git+https://github.com/Haekalss/GreenCode-Profiler.git
```
*(Gunakan `--force` untuk memastikan NPM selalu mengambil versi dan commit terbaru dari GitHub).*

**B. Install Lokal sebagai Dev Dependency di Project:**
```bash
npm install --save-dev git+https://github.com/Haekalss/GreenCode-Profiler.git
```
Lalu jalankan via npx:
```bash
npx greencode routes
npx greencode run
```

---

### 🐍 2. Python (Pip)
Cocok untuk lingkungan **Python, FastAPI, Django, Flask, atau Data Science**.

**Install / Upgrade dari GitHub:**
```bash
pip install --upgrade --force-reinstall git+https://github.com/Haekalss/GreenCode-Profiler.git
```

Setelah terinstall, perintah `greencode` langsung aktif di terminal:
```bash
greencode routes
greencode run
```

---

### 🐘 3. PHP / Backend (Composer)
Cocok untuk project **Laravel, Symfony, CodeIgniter, atau native PHP**.

**Opsi A — Menggunakan Global CLI (Paling Mudah):**  
Cukup install GreenCode secara global via NPM atau Pip di atas, lalu masuk ke folder project Laravel Anda dan langsung jalankan:
```bash
cd path/to/laravel-project
greencode routes   # Otomatis deteksi routes/api.php & routes/web.php
greencode run      # Jalankan profiling ke server php artisan serve
```

**Opsi B — Pasang ke `composer.json` Project:**  
Tambahkan VCS repository ke `composer.json` project Anda:
```json
{
  "repositories": [
    {
      "type": "vcs",
      "url": "https://github.com/Haekalss/GreenCode-Profiler.git"
    }
  ],
  "require-dev": {
    "haekalss/greencode-profiler": "dev-main"
  }
}
```
Kemudian jalankan:
```bash
composer update haekalss/greencode-profiler
```
Binary runner tersedia di `vendor/bin/greencode` atau script wrapper `php bin/greencode.php run`.

---

### 🐹 4. Go (Golang)
Cocok untuk aplikasi berkinerja tinggi menggunakan **Gin, Fiber, Echo, atau Chi**.

**A. Install Binary via `go install`:**
```bash
go install github.com/Haekalss/GreenCode-Profiler/cmd/greencode@latest
```

**B. Clone & Build Manual Executable:**
```bash
git clone https://github.com/Haekalss/GreenCode-Profiler.git
cd GreenCode-Profiler
go build -o bin/greencode.exe cmd/greencode/main.go

# Jalankan binary hasil kompilasi:
./bin/greencode.exe run
```

---

## 🛠️ Perintah CLI Lengkap

| Perintah | Deskripsi |
|---|---|
| `greencode routes [dir]` | **(Baru)** Deteksi otomatis rute dan endpoint nyata dari codebase project (Next.js, Laravel, dll) |
| `greencode run [config.json]` | Menjalankan analisis struktur kode + profiling energi runtime, dan menghasilkan laporan HTML |
| `greencode analyze [dir]` | Menjalankan analisis kualitas struktur kode saja (CCN, LOC, MI, Smells) dan menghasilkan `reports/greencode_quality_report.html` |
| `greencode detect` | Memindai dan mendiagnosa proses server aktif (Node, PHP, Python, Go) beserta PID dan port-nya |
| `greencode init [--preset ...]` | Membuat template file konfigurasi `greencode.json` dengan rute yang otomatis terdeteksi |
| `greencode ui` | Membuka laporan HTML dashboard hasil profiling di browser |
| `greencode help [cmd]` | Menampilkan panduan bantuan semua perintah atau perintah spesifik |

---

## 🚀 Alur Penggunaan Standar

1. **Pastikan server aplikasi Anda sudah berjalan** (misal: `npm run dev` pada port 3000 atau `php artisan serve` pada port 8000).
2. **Masuk ke folder project Anda:**
   ```bash
   cd path/to/project
   ```
3. **Cek rute yang terdeteksi secara otomatis:**
   ```bash
   greencode routes
   ```
4. **Generate konfigurasi starter:**
   ```bash
   greencode init
   ```
   *(File `greencode.json` akan dibuat otomatis terisi dengan rute-rute nyata yang ditemukan)*.
5. **Jalankan Profiling:**
   ```bash
   greencode run
   ```
6. **Buka Laporan:**  
   Laporan HTML bersih dan siap cetak akan tersimpan di:
   - `reports/greencode_report.html` (Hasil Profiling Energi + Kualitas Kode)
   - `reports/greencode_quality_report.html` (Hasil Analisis Kualitas Kode)

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

Template preset siap pakai tersedia di folder [`examples/`](examples/):
- [`examples/php_laravel.json`](examples/php_laravel.json)
- [`examples/node_nextjs.json`](examples/node_nextjs.json)
- [`examples/python_fastapi.json`](examples/python_fastapi.json)
- [`examples/go_fiber.json`](examples/go_fiber.json)
- [`examples/enterprise_laravel.json`](examples/enterprise_laravel.json)

---

## 📊 Laporan Hasil Pengukuran (HTML Reports)

GreenCode Profiler menghasilkan dua jenis laporan HTML berdesain bersih, terstruktur, dan siap cetak (*print-friendly*):

### 1. `reports/greencode_report.html` (Hasil `greencode run`)
- **Executive Summary Cards:** Total Energi (Joule), Konsumsi Listrik (kWh), Estimasi Emisi Karbon (gCO2e), CPU Active Time (detik), dan Rata-rata Skor Maintainability Index (Grade A/B/C/D).
- **Tab 1 — Green-Architecture Scorecard:** Tabel analisis korelasi konsumsi Joule vs Cyclomatic Complexity (CCN) per-modul untuk mendeteksi *energy hotspot*.
- **Tab 2 — Static Code Quality & Smells:** Ringkasan temuan fungsi terkompleks, Fat Controller, God Class, dan Deep Nesting.
- **Tab 3 — Runtime Latency & Memory:** Tabel latensi detail per-run, RSS RAM (MB), dan HTTP status code.
- **Tab 4 — Refactoring Action Plan:** Panduan rekomendasi langkah refactoring berbasis arsitektur hijau.

### 2. `reports/greencode_quality_report.html` (Hasil `greencode analyze`)
- Analisis struktur kode statis independen dengan skor CCN rata-rata & maksimum.
- Tabel rincian kompleksitas seluruh fungsi dan kelas di seluruh codebase.
- Deteksi pelanggaran arsitektur (*Architectural Smells*) yang dapat diekspor langsung ke dokumen skripsi/teknis.

---

## 📄 Lisensi
Didistribusikan di bawah lisensi open-source **MIT License**. Lihat [`LICENSE`](LICENSE) untuk informasi lebih lanjut.
