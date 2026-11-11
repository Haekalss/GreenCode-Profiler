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
3. **Green-Architecture Scorecard:**  
   Mengawinkan data statis (CCN & LOC) dengan data dinamis (Joule) untuk menghitung **Energy-to-Complexity Ratio** dan otomatis mengidentifikasi modul yang menjadi *energy hotspot*.
4. **Universal & Language-Agnostic:**  
   Bekerja dari luar proses. Mendukung **PHP (Laravel), Node.js (Next.js/Express), Python (FastAPI/Django), dan Go (Gin/Fiber)** tanpa modifikasi kode aplikasi target.
5. **Modern Interactive Web UI Dashboard:**  
   Secara otomatis menghasilkan dashboard HTML interaktif berdesain modern (Dark Theme) dengan metrik KPI, grafik, tabel korelasi, dan roadmap refactoring.

---

## 🚀 Panduan Cepat (2 Langkah Kerja)

### 🐍 Opsi 1: Menggunakan Python (`pip`)
```bash
# 1. Install package
pip install greencode-profiler

# 2. Jalankan Profiling & Buka Dashboard
greencode run
```

### 🟢 Opsi 2: Menggunakan Node.js / Next.js (`npm` / `npx`)
```bash
# 1. Install global via npm
npm install -g greencode-profiler

# Atau langsung via npx tanpa install:
npx greencode-profiler run
```

### 🐘 Opsi 3: Menggunakan PHP / Laravel (`composer`)
```bash
# 1. Buat konfigurasi starter
php bin/greencode.php init --preset php

# 2. Jalankan profiling
php bin/greencode.php run
```

### 🐹 Opsi 4: Menggunakan Go (`golang`)
```bash
# 1. Build binary executable
go build -o bin/greencode.exe cmd/greencode/main.go

# 2. Jalankan binary
./bin/greencode.exe run
```

---

## 🛠️ Perintah CLI Lengkap

| Perintah | Deskripsi |
|---|---|
| `greencode run [config.json]` | Menjalankan analisis struktur kode + profiling energi runtime, dan membuka Web Dashboard |
| `greencode analyze [dir]` | Menjalankan analisis kualitas struktur kode saja (CCN, LOC, MI, Smells) |
| `greencode detect` | Memindai dan menampilkan seluruh proses server aktif di komputer |
| `greencode init [--preset ...]` | Membuat template file konfigurasi `greencode.json` otomatis |
| `greencode ui` | Membuka laporan HTML dashboard hasil profiling di browser |
| `greencode help [cmd]` | Menampilkan panduan bantuan semua perintah atau perintah spesifik (misal: `greencode help run`) |

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

## 📊 Tampilan Dashboard Web UI

Dashboard yang dihasilkan (`reports/greencode_report.html`) menyajikan:
- **Hero KPI Cards:** Total Joule, Total kWh, gram emisi CO2e, CPU Active Time, dan Maintainability Index (Grade A/B/C/D).
- **Tab 1: Green-Architecture Scorecard:** Tabel korelasi konsumsi Joule vs Cyclomatic Complexity (CCN) per-modul.
- **Tab 2: Code Quality & Smells:** Top 10 fungsi terumit, Fat Controller, God Class, dan Deep Nesting.
- **Tab 3: Runtime Profiler:** Latensi detail per-run, RSS memori (RAM MB), dan status HTTP.
- **Tab 4: Refactoring Action Plan:** Rekomendasi konkret tindakan refactoring untuk efisiensi energi.

---

## 📦 Cara Mempublikasikan ke GitHub & Umum

1. **Inisialisasi Git di folder framework:**
   ```bash
   cd framework-greencode
   git init
   git add .
   git commit -m "feat: release GreenCode Profiler v1.0.0"
   ```
2. **Hubungkan ke Repositori GitHub Anda:**
   ```bash
   git remote add origin https://github.com/USERNAME_ANDA/greencode-profiler.git
   git branch -M main
   git push -u origin main
   ```
3. **Orang lain sekarang bisa menggunakannya langsung:**
   ```bash
   pip install git+https://github.com/USERNAME_ANDA/greencode-profiler.git
   ```

---

## 📄 Lisensi
Didistribusikan di bawah lisensi open-source **MIT License**. Lihat [`LICENSE`](LICENSE) untuk informasi lebih lanjut.
