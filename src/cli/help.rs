//! Terminal output, banners, and help text formatting.

pub fn print_banner() {
    println!("================================================================");
    println!("   GREEN PROFILER v1.8.2 (Rust Native Architecture)             ");
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
    format!("2026-11-16 {:02}:{:02}:{:02} WIB", hours, mins, secs)
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
    println!("                        Inisialisasi greencode.json + deteksi rute otomatis");
    println!("  routes [dir] [--limit N] [--filter K] [--exclude K]");
    println!("                        Deteksi rute nyata (default: tanpa batas, semua");
    println!("                        bahasa: Node/PHP/Python/Go/Rust)");
    println!("  detect                Pindai proses server aktif dan PID-nya");
    println!("  analyze [dir] [--no-open] [--filter K] [--exclude K] [--output file]");
    println!("                        Analisis kualitas kode statis (CCN, MI, LOC, Smells)");
    println!("  run [config.json] [--no-open] [--filter K] [--exclude K] [--output file]");
    println!("                        Jalankan profiling energi dan kualitas kode");
    println!("                        (--output: simpan laporan ke path kustom, JSON ikut)");
    println!("  ui                    Buka laporan HTML di peramban web");
    println!("  version               Tampilkan versi Green Profiler");
    println!("  help                  Tampilkan panduan bantuan ini\n");
    println!("Catatan Alias:");
    println!("  Perintah 'greenprofiler' dapat dipanggil dengan alias singkat 'gp'");
    println!("  (mis. 'gp run' atau 'gp init'). Kompatibilitas 'greencode' tetap didukung.\n");
    println!("Filter (boleh diulang, cocok case-insensitive pada path & nama):");
    println!("  --filter K   hanya endpoint yang mengandung K (logika OR)");
    println!("  --exclude K  buang endpoint yang mengandung K\n");
    println!("Contoh:");
    println!("  gp routes --filter api");
    println!("  greenprofiler run --filter api --exclude admin");
    println!("  gp init --filter api --filter data --limit 20\n");
}
