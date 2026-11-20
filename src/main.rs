//! GreenCode Profiler v1.7.2 - Rust Native Architecture
//! Universal Process-Level Energy Profiler & Green-Maintainability Framework

mod analyzer;
mod attributor;
mod correlator;
mod detector;
mod engine;
mod php_auditor;
mod reporter;

use analyzer::CodeQualityAnalyzer;
use attributor::ModuleAttributor;
use correlator::GreenCorrelator;
use detector::{RouteDiscoverer, RuntimeDetector};
use engine::ProcessEnergyEngine;
use php_auditor::PhpAuditor;
use reporter::ReportGenerator;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

fn print_banner() {
    println!("================================================================");
    println!("   GREENCODE PROFILER v1.7.2 (Rust Native Architecture)         ");
    println!("   Universal Process-Level Energy & Code Quality Framework      ");
    println!("================================================================");
}

fn get_simple_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let secs = now % 60;
    let mins = (now / 60) % 60;
    let hours = (now / 3600 + 7) % 24; // WIB UTC+7
    format!("2026-11-16 {:02}:{:02}:{:02} WIB", hours, mins, secs)
}

// ============================ CLI FLAGS ============================

/// Flag CLI yang didukung lintas perintah.
/// `--filter` dan `--exclude` BOLEH diulang (logika OR untuk filter,
/// dan semua ketentuan exclude harus lolos).
#[derive(Debug, Default)]
struct CliFlags {
    /// Argumen posisional (mis. `dir` pada `routes`, `config` pada `run`)
    positional: Vec<String>,
    /// `--limit N` — batasi jumlah rute (0 = tanpa batas)
    limit: usize,
    /// `-f` / `--force`
    force: bool,
    /// `--no-open`
    no_open: bool,
    /// `--filter <kata>` — boleh diulang
    filters: Vec<String>,
    /// `--exclude <kata>` — boleh diulang
    excludes: Vec<String>,
}

impl CliFlags {
    fn has_filter(&self) -> bool {
        !self.filters.is_empty() || !self.excludes.is_empty()
    }
}

/// Parse flag dari sisa argumen setelah nama perintah.
fn parse_flags(args: &[String]) -> CliFlags {
    let mut f = CliFlags::default();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--limit" | "-n" => {
                if i + 1 < args.len() {
                    f.limit = args[i + 1].parse().unwrap_or(0);
                    i += 2;
                    continue;
                }
            }
            "--filter" | "-F" => {
                if i + 1 < args.len() {
                    f.filters.push(args[i + 1].clone());
                    i += 2;
                    continue;
                }
            }
            "--exclude" | "-X" => {
                if i + 1 < args.len() {
                    f.excludes.push(args[i + 1].clone());
                    i += 2;
                    continue;
                }
            }
            "--force" | "-f" => f.force = true,
            "--no-open" => f.no_open = true,
            _ => {
                if !a.starts_with('-') {
                    f.positional.push(a.to_string());
                } else {
                    println!("[GreenCode] Flag tidak dikenal diabaikan: {}", a);
                }
            }
        }
        i += 1;
    }
    f
}

/// Terapkan `--filter` (OR) dan `--exclude` pada daftar endpoint.
/// Pencocokan case-insensitive substring terhadap `path` DAN `name`.
fn apply_filters(
    routes: Vec<detector::RouteEndpoint>,
    filters: &[String],
    excludes: &[String],
) -> Vec<detector::RouteEndpoint> {
    if filters.is_empty() && excludes.is_empty() {
        return routes;
    }
    routes
        .into_iter()
        .filter(|r| {
            let hay = format!("{} {}", r.path, r.name).to_lowercase();
            let pass_filter =
                filters.is_empty() || filters.iter().any(|f| hay.contains(&f.to_lowercase()));
            let pass_exclude = !excludes.iter().any(|e| hay.contains(&e.to_lowercase()));
            pass_filter && pass_exclude
        })
        .collect()
}

/// Pesan bantuan ketika discovery tidak menemukan rute sama sekali.
fn print_no_route_hints(dir: &str) {
    println!("[GreenCode] 0 rute ditemukan di '{}'.", dir);
    println!("            -> Tidak ada penanda project di folder ini");
    println!("               (package.json / composer.json / go.mod /");
    println!("                Cargo.toml / requirements.txt / manage.py)");
    println!("            -> Untuk monorepo, jalankan dari root:");
    println!("                 greencode routes");
    println!("            -> Atau arahkan ke sub-project (contoh):");
    println!("                 greencode routes ./frontend");
}

fn cmd_init(force: bool, filters: Vec<String>, excludes: Vec<String>) {
    let ecosystem = RuntimeDetector::detect_project_ecosystem(".");
    println!("[GreenCode] Mendeteksi ekosistem project: {}", ecosystem.to_uppercase());

    let mut config = if ecosystem == "fullstack" {
        let fs_info = RuntimeDetector::detect_fullstack_components(".");
        let fe = fs_info.frontend.unwrap();
        let be = fs_info.backend.unwrap();

        println!("[GreenCode] Komponen Frontend terdeteksi : {} ({} di port {})", fe.dir, fe.framework.to_uppercase(), fe.default_port);
        println!("[GreenCode] Komponen Backend terdeteksi  : {} ({} di port {})", be.dir, be.framework.to_uppercase(), be.default_port);

        json!({
            "project_name": format!("Full-Stack Monorepo ({} + {})", fe.dir, be.dir),
            "runtime": "fullstack",
            "source_dirs": [be.source_dir, fe.source_dir],
            "services": {
                "backend": be.base_url,
                "frontend": fe.base_url
            },
            "base_url": be.base_url,
            "auth": { "type": "none" },
            "endpoints": [],
            "repeats": 5,
            "cpu_tdp_watts": 15.0
        })
    } else {
        let default_port = match ecosystem.as_str() {
            "node" => 3000,
            "php" => 8000,
            "go" => 8080,
            "rust" => 8080,
            "python" => 8000,
            _ => 8000,
        };
        let base_url = format!("http://127.0.0.1:{}", default_port);

        println!("[GreenCode] Mengatur base_url otomatis ke port {}: {}", default_port, base_url);

        json!({
            "project_name": format!("{} Application", ecosystem.to_uppercase()),
            "runtime": ecosystem,
            "source_dir": ".",
            "base_url": base_url,
            "auth": { "type": "none" },
            "endpoints": [],
            "repeats": 5,
            "cpu_tdp_watts": 15.0
        })
    };

    // Baca SEMUA rute (tanpa batas), saring yang aman ditembak otomatis
    // (GET tanpa parameter), lalu terapkan --filter / --exclude bila ada.
    let all_profilable = RouteDiscoverer::profilable(RouteDiscoverer::discover(".", 0));
    let total_before = all_profilable.len();
    let discovered = apply_filters(all_profilable, &filters, &excludes);

    if !discovered.is_empty() {
        if filters.is_empty() && excludes.is_empty() {
            println!(
                "[GreenCode] Berhasil mendeteksi {} rute profilable dari project secara otomatis!",
                discovered.len()
            );
        } else {
            println!(
                "[GreenCode] {} rute profilable cocok dengan filter (dari {} di seluruh project).",
                discovered.len(),
                total_before
            );
        }
        config["endpoints"] = json!(discovered);
    } else if total_before > 0 {
        // Ada rute, tapi tidak ada yang cocok dengan filter
        println!(
            "[GreenCode] 0 rute cocok dengan --filter/--exclude (padahal ada {} rute profilable).",
            total_before
        );
        println!("            Periksa kembali kata kunci, atau jalankan 'greencode routes' untuk melihat daftar.");
        return;
    } else {
        println!("[GreenCode] Menggunakan template default endpoints.");
        config["endpoints"] = json!([
            { "path": "/", "name": "Homepage", "method": "GET" },
            { "path": "/api/users", "name": "API Users", "method": "GET" }
        ]);
    }

    let out_file = "greencode.json";
    if Path::new(out_file).exists() && !force {
        println!("[GreenCode] Berkas '{}' sudah ada. Gunakan flag -f / --force untuk menimpa.", out_file);
        return;
    }

    if let Ok(_) = fs::write(out_file, serde_json::to_string_pretty(&config).unwrap()) {
        println!("[GreenCode] Sukses membuat template konfigurasi: {}", out_file);
        println!("            Jalankan 'greencode run' untuk memulai profiling!");
    }
}

fn cmd_routes(path: &str, flags: &CliFlags) {
    println!("================================================================");
    println!("      GREENCODE AUTOMATIC ROUTE DISCOVERY (Rust Engine)         ");
    println!("================================================================");

    // Validasi path sejak awal: error yang jelas, bukan diam-diam 0 rute.
    if !Path::new(path).exists() {
        println!("[GreenError] Folder tidak ditemukan: {}", path);
        println!("            Periksa kembali path, atau jalankan dari root monorepo:");
        println!("              greencode routes");
        println!("================================================================\n");
        return;
    }

    println!("Direktori Target : {}", std::fs::canonicalize(path).unwrap_or_else(|_| path.into()).display());

    let all_routes = RouteDiscoverer::discover(path, flags.limit);
    if all_routes.is_empty() {
        println!("Total Ditemukan  : 0 rute nyata");
        println!("----------------------------------------------------------------");
        print_no_route_hints(path);
        println!("================================================================\n");
        return;
    }

    let total_before = all_routes.len();
    let routes = apply_filters(all_routes, &flags.filters, &flags.excludes);

    println!(
        "Total Ditemukan  : {} rute nyata{}",
        routes.len(),
        if flags.has_filter() {
            format!(" (disaring dari {})", total_before)
        } else {
            String::new()
        }
    );
    println!("----------------------------------------------------------------");

    if routes.is_empty() {
        println!("[GreenCode] 0 rute cocok dengan --filter / --exclude.");
        println!("            Kata kunci yang dipakai:");
        for f in &flags.filters {
            println!("              --filter {}", f);
        }
        for x in &flags.excludes {
            println!("              --exclude {}", x);
        }
        println!("================================================================\n");
        return;
    }

    for r in routes {
        let s_tag = r.service.map(|s| format!("[{}] ", s.to_uppercase())).unwrap_or_default();
        println!(" - [{:<4}] {}{:<35} ({})", r.method, s_tag, r.path, r.name);
    }
    println!("================================================================\n");
}

fn cmd_detect(lang: &str, port: Option<u16>) {
    println!("================================================================");
    println!("      GREENCODE RUNTIME PROCESS DETECTOR (Rust Engine)          ");
    println!("================================================================");
    println!("Target Language : {}", lang.to_uppercase());
    println!("Target Port     : {}", port.map(|p| p.to_string()).unwrap_or_else(|| "Semua".to_string()));

    if lang == "php" || lang == "auto" || lang == "fullstack" {
        let php_report = PhpAuditor::audit();
        if php_report.is_php_available {
            println!("----------------------------------------------------------------");
            println!(" [PHP GREEN EXTENSION AUDIT]");
            println!("  Versi PHP       : {}", php_report.php_version);
            println!("  Total Ekstensi  : {} modules loaded", php_report.loaded_extensions_count);
            println!("  Zend OPcache    : {}", if php_report.opcache_enabled { "[OK] Aktif (Hemat CPU ~50%)" } else { "[PERINGATAN] Nonaktif (Kompilasi berulang, boros energi!)" });
            println!("  JIT Compiler    : {}", if php_report.jit_enabled { "[OK] Aktif (Akselerasi instruksi mesin)" } else { "[INFO] Nonaktif" });
            println!("  Xdebug Status   : {}", if php_report.xdebug_active { "[BAHAYA] Aktif di runtime! (Overhead CPU +200%)" } else { "[OK] Nonaktif (Optimal & ringan)" });
            if !php_report.recommendations.is_empty() {
                println!("  Rekomendasi     :");
                for r in &php_report.recommendations {
                    println!("   * {}", r);
                }
            }
        }
    }

    let pids = RuntimeDetector::find_pids(lang, port);
    println!("Ditemukan       : {} proses aktif", pids.len());
    println!("----------------------------------------------------------------");
    for pid in &pids {
        println!(" - PID {:<6}", pid);
    }
    println!("================================================================\n");
}

fn cmd_analyze(flags: CliFlags) {
    // Tentukan config file (default greencode.json)
    let config_path = flags.positional.first().cloned().unwrap_or_else(|| "greencode.json".to_string());
    let no_open = flags.no_open;

    let mut inferred_targets: Vec<String> = Vec::new();

    // Coba baca config HANYA jika argumen pertama berupa file (bukan direktori).
    if Path::new(&config_path).is_file() {
        let cfg_str = fs::read_to_string(&config_path).unwrap_or_default();
        let cfg: Value = serde_json::from_str(&cfg_str).unwrap_or(json!({}));

        // Baca endpoint dari config
        let mut endpoints_from_config: Vec<detector::RouteEndpoint> = Vec::new();
        if let Some(eps) = cfg["endpoints"].as_array() {
            for ep in eps {
                endpoints_from_config.push(detector::RouteEndpoint {
                    path: ep["path"].as_str().unwrap_or("/").to_string(),
                    name: ep["name"].as_str().unwrap_or("").to_string(),
                    method: ep["method"].as_str().unwrap_or("GET").to_string(),
                    service: ep["service"].as_str().map(|s| s.to_string()),
                    base_url: ep["base_url"].as_str().map(|s| s.to_string()),
                    url: ep["url"].as_str().map(|s| s.to_string()),
                });
            }
        }

        // Terapkan --filter / --exclude
        let total_endpoints_before_filter = endpoints_from_config.len();
        endpoints_from_config = apply_filters(endpoints_from_config, &flags.filters, &flags.excludes);

        if flags.has_filter() {
            println!(
                "[GreenCode] Filter aktif: {} endpoint dipilih dari {} endpoint di {}.",
                endpoints_from_config.len(),
                total_endpoints_before_filter,
                config_path
            );
            if endpoints_from_config.is_empty() {
                println!("[GreenError] 0 endpoint cocok dengan --filter / --exclude.");
                println!("            Kata kunci yang dipakai:");
                for f in &flags.filters {
                    println!("              --filter {}", f);
                }
                for x in &flags.excludes {
                    println!("              --exclude {}", x);
                }
                println!("            Jalankan 'greencode routes' untuk melihat daftar endpoint.");
                return;
            }
        }

        // SELALU pakai semua source_dirs agar monorepo (FE + BE) tercakup.
        // Penyaringan dilakukan di level FILE (path/isi) memakai keyword filter.
        if let Some(arr) = cfg["source_dirs"].as_array() {
            inferred_targets = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
        } else if let Some(dir) = cfg["source_dir"].as_str() {
            inferred_targets = vec![dir.to_string()];
        }
    }

    // Fallback: jika tidak ada config atau tidak ada inferred targets, pakai positional arg.
    // Mendukung `analyze <direktori>` maupun `analyze <config.json>`.
    if inferred_targets.is_empty() {
        let target = if let Some(first) = flags.positional.first() {
            if Path::new(first).is_dir() {
                first.clone()
            } else if flags.positional.len() > 1 && !flags.positional[1].starts_with('-') {
                flags.positional[1].clone()
            } else {
                ".".to_string()
            }
        } else {
            ".".to_string()
        };
        inferred_targets = vec![target];
    }

    println!("================================================================");
    println!("   GREENCODE STATIC CODE QUALITY ANALYZER (Rust Native)         ");
    println!("================================================================");
    if flags.has_filter() {
        println!("Target direktori (monorepo, semua source_dirs): {}", inferred_targets.join(", "));
    } else {
        for t in &inferred_targets {
            println!("Scanning Directory : {}", std::fs::canonicalize(t).unwrap_or_else(|_| t.into()).display());
        }
    }

    let analyzer = if flags.has_filter() {
        CodeQualityAnalyzer::with_file_filters(
            inferred_targets,
            flags.filters.clone(),
            flags.excludes.clone(),
        )
    } else {
        CodeQualityAnalyzer::new(inferred_targets)
    };
    let report = analyzer.run_analysis();
    let ov = &report.overview;

    println!("----------------------------------------------------------------");
    if flags.has_filter() {
        println!(
            "Filter File              : {} cocok dengan keyword {:?} dari {} total file kode",
            ov.total_files,
            flags.filters,
            ov.total_files_considered
        );
    }
    println!("Total Files Scanned      : {} files", ov.total_files);
    println!("Total NCLOC              : {} lines", ov.total_ncloc);
    println!("Total Complexity (CCN)   : {}", ov.total_ccn);
    println!("Average CCN per File     : {:.2}", ov.average_ccn_per_file);
    println!("Maintainability Index    : {:.1} / 100 (Grade: {})", ov.average_maintainability_index, ov.maintainability_grade);
    println!("Architectural Smells     : {} detected", ov.total_code_smells);
    if !ov.language_breakdown.is_empty() {
        let langs: Vec<String> = ov
            .language_breakdown
            .iter()
            .map(|l| format!("{} {} file", l.language, l.files))
            .collect();
        println!("Bahasa Terdeteksi        : {}", langs.join(", "));
    }
    println!("----------------------------------------------------------------");

    let mut val = serde_json::to_value(&report).unwrap();
    if flags.has_filter() {
        val["applied_filter"] = json!({
            "filters": flags.filters,
            "excludes": flags.excludes,
            "matched_files": ov.total_files,
            "considered_files": ov.total_files_considered
        });
    }
    let json_path = "reports/code_quality.json";
    let html_path = "reports/code_quality.html";

    let _ = ReportGenerator::save_json(&val, json_path);
    let _ = ReportGenerator::generate_code_quality_html(&val, html_path);

    println!("\nHasil JSON tersimpan di : {}", std::fs::canonicalize(json_path).unwrap_or_else(|_| json_path.into()).display());
    println!("Laporan HTML tersimpan di: {}", std::fs::canonicalize(html_path).unwrap_or_else(|_| html_path.into()).display());
    println!("================================================================\n");

    if !no_open {
        ReportGenerator::open_in_browser(html_path);
    }
}

fn cmd_run(config_path: &str, flags: CliFlags) {
    if !Path::new(config_path).exists() {
        println!("[GreenCode] Berkas konfigurasi '{}' tidak ditemukan.", config_path);
        println!("            Jalankan 'greencode init' untuk membuatnya otomatis.");
        return;
    }

    let cfg_str = fs::read_to_string(config_path).unwrap_or_default();
    let cfg: Value = serde_json::from_str(&cfg_str).unwrap_or(json!({}));

    let project_name = cfg["project_name"].as_str().unwrap_or("Web Project");
    let runtime = cfg["runtime"].as_str().unwrap_or("auto");
    let base_url = cfg["base_url"].as_str().unwrap_or("http://127.0.0.1:8000");
    let repeats = cfg["repeats"].as_u64().unwrap_or(5) as usize;
    let cpu_tdp = cfg["cpu_tdp_watts"].as_f64().unwrap_or(15.0);

    print_banner();
    println!("Project  : {}", project_name);
    println!("Runtime  : {}", runtime.to_uppercase());
    println!("Base URL : {}", base_url);
    println!("Repeats  : {}x per endpoint", repeats);
    println!("----------------------------------------------------------------");

    // Baca endpoint dari config DULU, sebelum static analysis
    let mut endpoints_from_config: Vec<detector::RouteEndpoint> = Vec::new();
    if let Some(eps) = cfg["endpoints"].as_array() {
        for ep in eps {
            endpoints_from_config.push(detector::RouteEndpoint {
                path: ep["path"].as_str().unwrap_or("/").to_string(),
                name: ep["name"].as_str().unwrap_or("").to_string(),
                method: ep["method"].as_str().unwrap_or("GET").to_string(),
                service: ep["service"].as_str().map(|s| s.to_string()),
                base_url: ep["base_url"].as_str().map(|s| s.to_string()),
                url: ep["url"].as_str().map(|s| s.to_string()),
            });
        }
    }

    // Terapkan --filter / --exclude SEKARANG (sebelum static analysis)
    let total_endpoints_before_filter = endpoints_from_config.len();
    endpoints_from_config = apply_filters(endpoints_from_config, &flags.filters, &flags.excludes);

    if flags.has_filter() {
        println!(
            "[GreenCode] Filter aktif: {} endpoint dipilih dari {} endpoint di {}.",
            endpoints_from_config.len(),
            total_endpoints_before_filter,
            config_path
        );
        if endpoints_from_config.is_empty() {
            println!("[GreenError] 0 endpoint cocok dengan --filter / --exclude.");
            println!("            Kata kunci yang dipakai:");
            for f in &flags.filters {
                println!("              --filter {}", f);
            }
            for x in &flags.excludes {
                println!("              --exclude {}", x);
            }
            println!("            Jalankan 'greencode routes' untuk melihat daftar endpoint.");
            return;
        }
    }

    // Target direktori: SELALU pakai semua source_dirs agar monorepo (FE + BE)
    // tercakup. Penyaringan dilakukan di level FILE (path/isi) memakai keyword
    // yang sama dengan filter endpoint — bukan dengan memilih salah satu folder.
    let inferred_targets: Vec<String> = if let Some(arr) = cfg["source_dirs"].as_array() {
        arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
    } else if let Some(dir) = cfg["source_dir"].as_str() {
        vec![dir.to_string()]
    } else {
        vec![".".to_string()]
    };

    // 1. Static Code Analysis
    println!("[GreenCode] [1/4] Menjalankan Analisis Kualitas Struktur Kode (Rust Analyzer)...");
    if flags.has_filter() && !inferred_targets.is_empty() {
        println!("            Target direktori (monorepo, semua source_dirs): {}", inferred_targets.join(", "));
    }
    let analyzer = if flags.has_filter() {
        CodeQualityAnalyzer::with_file_filters(
            inferred_targets,
            flags.filters.clone(),
            flags.excludes.clone(),
        )
    } else {
        CodeQualityAnalyzer::new(inferred_targets)
    };
    let quality_report = analyzer.run_analysis();
    let q_ov = &quality_report.overview;
    if flags.has_filter() {
        println!(
            "            Filter file: {} cocok dengan keyword {:?} dari {} total file kode.",
            q_ov.total_files,
            flags.filters,
            q_ov.total_files_considered
        );
    }
    println!("            Files: {} | NCLOC: {} | Avg CCN: {:.2} | MI: {:.1}/100", q_ov.total_files, q_ov.total_ncloc, q_ov.average_ccn_per_file, q_ov.average_maintainability_index);
    if !q_ov.language_breakdown.is_empty() {
        let langs: Vec<String> = q_ov
            .language_breakdown
            .iter()
            .map(|l| format!("{} {} file", l.language, l.files))
            .collect();
        println!("            Bahasa terdeteksi: {}", langs.join(", "));
    }

    // 2. PHP Green Extension & Runtime Audit
    let php_audit_opt = if runtime == "php" || runtime == "fullstack" || runtime == "auto" {
        println!("\n[GreenCode] [2/4] Mengaudit PHP Runtime & Green Extensions (OPcache, JIT, Xdebug)...");
        let rep = PhpAuditor::audit();
        if rep.is_php_available {
            println!("            PHP: {} | Extensions: {}", rep.php_version, rep.loaded_extensions_count);
            println!("            OPcache: {} | Xdebug: {}", 
                if rep.opcache_enabled { "Aktif [OK]" } else { "Nonaktif [BOROS ENERGI]" },
                if rep.xdebug_active { "Aktif [OVERHEAD TINGGI]" } else { "Nonaktif [OK]" }
            );
        }
        Some(rep)
    } else {
        None
    };

    // 3. Process Detection
    println!("\n[GreenCode] [3/4] Mendeteksi proses server aktif ({runtime})...");
    let pids = RuntimeDetector::find_pids(runtime, None);
    if pids.is_empty() {
        println!("[GreenCode] PERINGATAN: Tidak ada proses {} terdeteksi.", runtime.to_uppercase());
        println!("            Pastikan server aplikasi Anda sudah dijalankan.");
        return;
    }
    println!("[GreenCode] Terhubung ke {} proses aktif: PID {:?}", pids.len(), pids);

    // Gunakan endpoint yang sudah difilter di atas (untuk static analysis)
    let endpoints_to_trace = endpoints_from_config;

    let engine = ProcessEnergyEngine::new(pids, cpu_tdp);
    let mut attributor = ModuleAttributor::new(&engine);

    println!("\n[GreenCode] [4/4] Memulai profiling runtime {} endpoint...", endpoints_to_trace.len());
    for ep in &endpoints_to_trace {
        let url = ep.url.clone().unwrap_or_else(|| {
            if let Some(ref s) = ep.service {
                if let Some(srv_url) = cfg["services"][s].as_str() {
                    return format!("{}{}", srv_url.trim_end_matches('/'), ep.path);
                }
            }
            format!("{}{}", base_url.trim_end_matches('/'), ep.path)
        });

        println!("\n-> [{}] {} ({})", ep.method, ep.name, url);
        for i in 1..=repeats {
            let rec = attributor.trace_http_request(&url, &ep.method, &[], None);
            println!("   Run {}/{} | HTTP {:<3} | Latency: {:6.1} ms | CPU: {:6.4} s | Energi: {:8.6} J",
                i, repeats, rec.status, rec.latency_ms, rec.cpu_active_time_s, rec.energy_total_joules
            );
        }
    }

    // 4. Correlator & Grand Totals
    let correlated = GreenCorrelator::correlate(&attributor.records);
    let total_joules: f64 = attributor.records.iter().map(|r| r.energy_total_joules).sum();
    let total_cpu_s: f64 = attributor.records.iter().map(|r| r.cpu_active_time_s).sum();
    let total_kwh = total_joules / 3_600_000.0;
    let total_co2e_g = total_kwh * 850.0;
    let avg_latency: f64 = if !attributor.records.is_empty() {
        attributor.records.iter().map(|r| r.latency_ms).sum::<f64>() / attributor.records.len() as f64
    } else {
        0.0
    };

    // 4b. Ringkas untuk laporan: SATU baris per endpoint (rata-rata dari `repeats` run).
    //     Pengukuran tetap menjalankan `repeats` kali per endpoint di atas;
    //     hanya tampilan laporan yang tidak mengulang endpoint sebanyak jumlah run.
    let mut ep_order: Vec<String> = Vec::new();
    let mut ep_groups: std::collections::HashMap<String, Vec<&attributor::RequestTraceRecord>> =
        std::collections::HashMap::new();
    for rec in &attributor.records {
        let key = format!("{} {}", rec.method, rec.endpoint);
        if !ep_groups.contains_key(&key) {
            ep_order.push(key.clone());
        }
        ep_groups.entry(key).or_default().push(rec);
    }

    let mut endpoints_detail: Vec<attributor::RequestTraceRecord> = Vec::new();
    for key in &ep_order {
        let list = &ep_groups[key];
        let n = list.len() as f64;
        let mean = |f: fn(&attributor::RequestTraceRecord) -> f64| {
            list.iter().map(|r| f(r)).sum::<f64>() / n
        };
        let first = list[0];
        endpoints_detail.push(attributor::RequestTraceRecord {
            endpoint: first.endpoint.clone(),
            method: first.method.clone(),
            status: first.status,
            latency_ms: (mean(|r| r.latency_ms) * 10.0).round() / 10.0,
            cpu_active_time_s: (mean(|r| r.cpu_active_time_s) * 10000.0).round() / 10000.0,
            energy_total_joules: (mean(|r| r.energy_total_joules) * 1000000.0).round() / 1000000.0,
            co2e_grams: (mean(|r| r.co2e_grams) * 1000000.0).round() / 1000000.0,
            cpu_utilization: (mean(|r| r.cpu_utilization) * 10.0).round() / 10.0,
        });
    }

    let report_data = json!({
        "metadata": {
            "project_name": project_name,
            "runtime": runtime,
            "base_url": base_url,
            "engine": "Rust Native Architecture v1.7.2",
            "timestamp": get_simple_timestamp(),
            "cpu_tdp_watts": cpu_tdp,
            "repeats_per_endpoint": repeats,
            "detail_aggregation": format!("rata-rata dari {} run per endpoint", repeats)
        },
        "totals": {
            "total_requests": attributor.records.len(),
            "total_joules": (total_joules * 1000000.0).round() / 1000000.0,
            "total_kwh": total_kwh,
            "total_cpu_active_s": (total_cpu_s * 10000.0).round() / 10000.0,
            "avg_latency_ms": (avg_latency * 10.0).round() / 10.0,
            "total_co2e_g": (total_co2e_g * 1000000.0).round() / 1000000.0
        },
        "php_environment": serde_json::to_value(&php_audit_opt).unwrap_or(json!(null)),
        "correlated_modules": correlated,
        "code_quality": serde_json::to_value(&quality_report).unwrap(),
        "endpoints_detail": endpoints_detail,
        "applied_filter": if flags.has_filter() {
            json!({
                "filters": &flags.filters,
                "excludes": &flags.excludes,
                "matched_files": quality_report.overview.total_files,
                "considered_files": quality_report.overview.total_files_considered
            })
        } else {
            json!(null)
        }
    });

    let json_path = "reports/greencode_report.json";
    let html_path = "reports/greencode_report.html";

    let _ = ReportGenerator::save_json(&report_data, json_path);
    let _ = ReportGenerator::generate_html(&report_data, html_path);

    println!("\n================================================================");
    println!("                     PROFILING SELESAI                          ");
    println!("================================================================");
    println!("Laporan Komprehensif tersimpan di:");
    println!(" - JSON : {}", std::fs::canonicalize(json_path).unwrap_or_else(|_| json_path.into()).display());
    println!(" - HTML : {}", std::fs::canonicalize(html_path).unwrap_or_else(|_| html_path.into()).display());
    println!("================================================================\n");

    if !flags.no_open {
        ReportGenerator::open_in_browser(html_path);
    }
}

fn cmd_ui() {
    let html_path = "reports/greencode_report.html";
    if Path::new(html_path).exists() {
        ReportGenerator::open_in_browser(html_path);
    } else {
        let qual_path = "reports/code_quality.html";
        if Path::new(qual_path).exists() {
            ReportGenerator::open_in_browser(qual_path);
        } else {
            println!("[GreenCode] Laporan belum dibuat. Jalankan 'greencode run' terlebih dahulu.");
        }
    }
}

fn print_help() {
    print_banner();
    println!("Penggunaan: greencode <command> [options]\n");
    println!("Perintah Utama:");
    println!("  init [--force] [--filter K] [--exclude K]");
    println!("                        Inisialisasi greencode.json + deteksi rute otomatis");
    println!("  routes [dir] [--limit N] [--filter K] [--exclude K]");
    println!("                        Deteksi rute nyata (default: tanpa batas, semua");
    println!("                        bahasa: Node/PHP/Python/Go/Rust)");
    println!("  detect                Pindai proses server aktif dan PID-nya");
    println!("  analyze [dir]         Analisis kualitas kode statis (CCN, MI, LOC, Smells)");
    println!("  run [config.json] [--no-open] [--filter K] [--exclude K]");
    println!("                        Jalankan profiling energi dan kualitas kode");
    println!("  ui                    Buka laporan HTML di peramban web");
    println!("  version               Tampilkan versi GreenCode Profiler");
    println!("  help                  Tampilkan panduan bantuan ini\n");
    println!("Filter (boleh diulang, cocok case-insensitive pada path & nama):");
    println!("  --filter K   hanya endpoint yang mengandung K (logika OR)");
    println!("  --exclude K  buang endpoint yang mengandung K\n");
    println!("Contoh:");
    println!("  greencode routes --filter api");
    println!("  greencode run --filter api --exclude admin");
    println!("  greencode init --filter api --filter data --limit 20\n");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "init" => {
            // init [--force] [--filter kata]... [--exclude kata]...
            let flags = parse_flags(&args[2..]);
            cmd_init(flags.force, flags.filters, flags.excludes);
        }
        "routes" => {
            // routes [dir] [--limit N] [--filter kata]... [--exclude kata]...
            let flags = parse_flags(&args[2..]);
            let dir = flags
                .positional
                .first()
                .cloned()
                .unwrap_or_else(|| ".".to_string());
            cmd_routes(&dir, &flags);
        }
        "detect" => {
            let lang = if args.len() > 2 { &args[2] } else { "auto" };
            cmd_detect(lang, None);
        }
        "analyze" => {
            let flags = parse_flags(&args[2..]);
            cmd_analyze(flags);
        }
        "run" => {
            // run [config.json] [--no-open] [--filter kata]... [--exclude kata]...
            let flags = parse_flags(&args[2..]);
            let config = flags
                .positional
                .first()
                .cloned()
                .unwrap_or_else(|| "greencode.json".to_string());
            cmd_run(&config, flags);
        }
        "ui" => cmd_ui(),
        "version" | "-v" | "--version" => {
            println!("GreenCode Profiler v1.7.2");
        }
        "help" | "-h" | "--help" => print_help(),
        other => {
            println!("[GreenCode] Perintah tidak dikenal: '{}'", other);
            println!("            Gunakan 'greencode help' untuk melihat daftar perintah.");
        }
    }
}
