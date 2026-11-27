//! Terminal output, banners, and help text formatting.

pub fn print_banner() {
    println!("================================================================");
    println!("   GREEN PROFILER v1.9.0 (Rust Native Architecture)             ");
    println!("   Universal Process-Level Energy & Code Quality Framework      ");
    println!("================================================================");
}

pub fn get_simple_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs = now % 60;
    let mins = (now / 60) % 60;
    let hours = (now / 3600 + 7) % 24; // WIB UTC+7
    format!("2026-11-27 {:02}:{:02}:{:02} WIB", hours, mins, secs)
}

pub fn print_no_route_hints(dir: &str) {
    println!("[GreenProfiler] 0 rute ditemukan di '{}'.", dir);
    println!("            -> Tidak ada penanda project di folder ini");
    println!("               (package.json / composer.json / go.mod /");
    println!("                Cargo.toml / requirements.txt / manage.py)");
    println!("            -> Untuk monorepo, jalankan dari root:");
    println!("                 greenprofiler routes");
    println!("            -> Atau arahkan ke sub-project (contoh):");
    println!("                 greenprofiler routes ./frontend");
}

pub fn print_filter_keywords(filters: &[String], excludes: &[String]) {
    for keyword in filters {
        println!("              --filter {}", keyword);
    }
    for keyword in excludes {
        println!("              --exclude {}", keyword);
    }
}

pub fn print_help() {
    print_banner();
    println!("Penggunaan: greenprofiler <command> [options]");
    println!("            gp <command> [options]           (alias singkat)\n");
    println!("Perintah Utama:");
    println!("  init [--force] [--filter K] [--exclude K]");
    println!("                        Inisialisasi greenprofiler.json + deteksi rute otomatis");
    println!("  routes [dir] [--limit N] [--filter K] [--exclude K]");
    println!("                        Deteksi rute nyata (Node/PHP/Python/Go/Rust)");
    println!("  detect                Pindai proses server aktif dan PID-nya");
    println!("  analyze [dir] [--strict] [--no-open] [--filter K] [--exclude K] [--output file]");
    println!("                        Analisis kualitas kode adaptif, deteksi kandidat PHP Extension,");
    println!("                        serta proyeksi simulasi beban multi-user");
    println!("  run [config.json] [--strict] [--users N] [--no-open] [--filter K] [--output file]");
    println!("                        Jalankan profiling energi dinamis, kualitas kode & simulasi skala");
    println!("  simulate [report.json] [--users N] [--rpu N] [--cores C] [--window S] [--no-open]");
    println!("                        Simulasi dampak banyak user (energi, CO2, latensi P95, kapasitas)");
    println!("                        menggunakan hasil audit yang sudah ada");
    println!("  ui                    Buka laporan HTML terbaru di peramban web");
    println!("  version               Tampilkan versi Green Profiler");
    println!("  help                  Tampilkan panduan bantuan ini\n");
    println!("Mode Standar Kualitas:");
    println!("  Default               Adaptif harian + riset (Backend <=300 NCLOC, UI .tsx/.jsx <=450 NCLOC,");
    println!("                        Deep Nesting khusus blok control-flow murni)");
    println!("  --strict              Mode audit ketat seragam (seluruh berkas <=300 NCLOC)\n");
    println!("Contoh:");
    println!("  gp analyze --filter smki --no-open");
    println!("  gp simulate reports/greenprofiler_report.json --users 2500 --cores 8");
    println!("  greenprofiler run --users 5000 --filter api\n");
}
