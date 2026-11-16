//! GreenCode Profiler v1.2.0 - Rust Native Architecture
//! Universal Process-Level Energy Profiler & Green-Maintainability Framework

mod analyzer;
mod attributor;
mod correlator;
mod detector;
mod engine;
mod reporter;

use analyzer::CodeQualityAnalyzer;
use attributor::ModuleAttributor;
use correlator::GreenCorrelator;
use detector::{RouteDiscoverer, RuntimeDetector};
use engine::ProcessEnergyEngine;
use reporter::ReportGenerator;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

fn print_banner() {
    println!("================================================================");
    println!("   GREENCODE PROFILER v1.2.0 (Rust Native Architecture)         ");
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

fn cmd_init(force: bool) {
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
        json!({
            "project_name": "Web Service Application",
            "runtime": ecosystem,
            "source_dir": ".",
            "base_url": "http://127.0.0.1:8000",
            "auth": { "type": "none" },
            "endpoints": [],
            "repeats": 5,
            "cpu_tdp_watts": 15.0
        })
    };

    let discovered = RouteDiscoverer::discover(".", 16);
    if !discovered.is_empty() {
        println!("[GreenCode] Berhasil mendeteksi {} rute nyata dari project secara otomatis!", discovered.len());
        config["endpoints"] = json!(discovered);
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

fn cmd_routes(path: &str, limit: usize) {
    println!("================================================================");
    println!("      GREENCODE AUTOMATIC ROUTE DISCOVERY (Rust Engine)         ");
    println!("================================================================");
    println!("Direktori Target : {}", std::fs::canonicalize(path).unwrap_or_else(|_| path.into()).display());

    let routes = RouteDiscoverer::discover(path, limit);
    println!("Total Ditemukan  : {} rute nyata", routes.len());
    println!("----------------------------------------------------------------");
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

    let pids = RuntimeDetector::find_pids(lang, port);
    println!("Ditemukan       : {} proses aktif", pids.len());
    println!("----------------------------------------------------------------");
    for pid in &pids {
        println!(" - PID {:<6}", pid);
    }
    println!("================================================================\n");
}

fn cmd_analyze(target_path: &str, no_open: bool) {
    println!("================================================================");
    println!("   GREENCODE STATIC CODE QUALITY ANALYZER (Rust Native)         ");
    println!("================================================================");
    println!("Scanning Directory : {}", std::fs::canonicalize(target_path).unwrap_or_else(|_| target_path.into()).display());

    let analyzer = CodeQualityAnalyzer::new(vec![target_path]);
    let report = analyzer.run_analysis();
    let ov = &report.overview;

    println!("----------------------------------------------------------------");
    println!("Total Files Scanned      : {} files", ov.total_files);
    println!("Total NCLOC              : {} lines", ov.total_ncloc);
    println!("Total Complexity (CCN)   : {}", ov.total_ccn);
    println!("Average CCN per File     : {:.2}", ov.average_ccn_per_file);
    println!("Maintainability Index    : {:.1} / 100 (Grade: {})", ov.average_maintainability_index, ov.maintainability_grade);
    println!("Architectural Smells     : {} detected", ov.total_code_smells);
    println!("----------------------------------------------------------------");

    let val = serde_json::to_value(&report).unwrap();
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

fn cmd_run(config_path: &str, no_open: bool) {
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

    // 1. Static Code Analysis
    println!("[GreenCode] [1/3] Menjalankan Analisis Kualitas Struktur Kode (Rust Analyzer)...");
    let targets: Vec<String> = if let Some(arr) = cfg["source_dirs"].as_array() {
        arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
    } else if let Some(dir) = cfg["source_dir"].as_str() {
        vec![dir.to_string()]
    } else {
        vec![".".to_string()]
    };

    let analyzer = CodeQualityAnalyzer::new(targets);
    let quality_report = analyzer.run_analysis();
    let q_ov = &quality_report.overview;
    println!("            Files: {} | NCLOC: {} | Avg CCN: {:.2} | MI: {:.1}/100", q_ov.total_files, q_ov.total_ncloc, q_ov.average_ccn_per_file, q_ov.average_maintainability_index);

    // 2. Process Detection
    println!("[GreenCode] [2/3] Mendeteksi proses server aktif ({runtime})...");
    let pids = RuntimeDetector::find_pids(runtime, None);
    if pids.is_empty() {
        println!("[GreenCode] PERINGATAN: Tidak ada proses {} terdeteksi.", runtime.to_uppercase());
        println!("            Pastikan server aplikasi Anda sudah dijalankan.");
        return;
    }
    println!("[GreenCode] Terhubung ke {} proses aktif: PID {:?}", pids.len(), pids);

    // 3. Dynamic HTTP Profiling
    let mut endpoints_to_trace: Vec<detector::RouteEndpoint> = Vec::new();
    if let Some(eps) = cfg["endpoints"].as_array() {
        for ep in eps {
            endpoints_to_trace.push(detector::RouteEndpoint {
                path: ep["path"].as_str().unwrap_or("/").to_string(),
                name: ep["name"].as_str().unwrap_or("").to_string(),
                method: ep["method"].as_str().unwrap_or("GET").to_string(),
                service: ep["service"].as_str().map(|s| s.to_string()),
                base_url: ep["base_url"].as_str().map(|s| s.to_string()),
                url: ep["url"].as_str().map(|s| s.to_string()),
            });
        }
    }

    if endpoints_to_trace.is_empty() {
        endpoints_to_trace = RouteDiscoverer::discover(".", 10);
    }

    let engine = ProcessEnergyEngine::new(pids, cpu_tdp);
    let mut attributor = ModuleAttributor::new(&engine);

    println!("\n[GreenCode] [3/3] Memulai profiling runtime {} endpoint...", endpoints_to_trace.len());
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

    let report_data = json!({
        "metadata": {
            "project_name": project_name,
            "runtime": runtime,
            "base_url": base_url,
            "engine": "Rust Native Architecture v1.2.0",
            "timestamp": get_simple_timestamp(),
            "cpu_tdp_watts": cpu_tdp
        },
        "totals": {
            "total_requests": attributor.records.len(),
            "total_joules": (total_joules * 1000000.0).round() / 1000000.0,
            "total_kwh": total_kwh,
            "total_cpu_active_s": (total_cpu_s * 10000.0).round() / 10000.0,
            "avg_latency_ms": (avg_latency * 10.0).round() / 10.0,
            "total_co2e_g": (total_co2e_g * 1000000.0).round() / 1000000.0
        },
        "correlated_modules": correlated,
        "code_quality": serde_json::to_value(&quality_report).unwrap(),
        "endpoints_detail": attributor.records
    });

    let json_path = "reports/greencode_report.json";
    let html_path = "reports/greencode_report.html";

    let _ = ReportGenerator::save_json(&report_data, json_path);
    let _ = ReportGenerator::generate_html(&report_data, html_path);

    println!("\n================================================================");
    println!("                     PROFILING SELESAI                          ");
    println!("================================================================");
    println!("Total Energi Terukur : {:.6} Joules ({:.8} kWh)", total_joules, total_kwh);
    println!("Total CPU Aktif      : {:.4} detik (Bebas I/O Wait Bias)", total_cpu_s);
    println!("Jejak Karbon Bersih  : {:.6} gram CO2e", total_co2e_g);
    println!("----------------------------------------------------------------");
    println!("Laporan JSON      : {}", std::fs::canonicalize(json_path).unwrap_or_else(|_| json_path.into()).display());
    println!("Dashboard Web UI  : {}", std::fs::canonicalize(html_path).unwrap_or_else(|_| html_path.into()).display());
    println!("================================================================\n");

    if !no_open {
        ReportGenerator::open_in_browser(html_path);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = if args.len() > 1 { args[1].as_str() } else { "run" };

    match cmd {
        "init" => {
            let force = args.iter().any(|a| a == "-f" || a == "--force");
            cmd_init(force);
        }
        "detect" => {
            let lang = if args.len() > 2 { &args[2] } else { "auto" };
            let port = args.get(3).and_then(|p| p.parse().ok());
            cmd_detect(lang, port);
        }
        "analyze" => {
            let path = if args.len() > 2 { &args[2] } else { "." };
            let no_open = args.iter().any(|a| a == "--no-open");
            cmd_analyze(path, no_open);
        }
        "routes" => {
            let path = if args.len() > 2 { &args[2] } else { "." };
            let limit = args.get(3).and_then(|l| l.parse().ok()).unwrap_or(30);
            cmd_routes(path, limit);
        }
        "ui" => {
            let path = if args.len() > 2 { &args[2] } else { "reports/greencode_report.html" };
            ReportGenerator::open_in_browser(path);
        }
        "version" | "--version" | "-v" => {
            println!("GreenCode Profiler v1.2.0 (Rust Native Architecture - Zero Cost Abstraction)");
        }
        "run" => {
            let config_path = if args.len() > 2 && !args[2].starts_with('-') { &args[2] } else { "greencode.json" };
            let no_open = args.iter().any(|a| a == "--no-open");
            cmd_run(config_path, no_open);
        }
        "help" | "--help" | "-h" => {
            print_banner();
            println!("Penggunaan: greencode <command> [opsi]");
            println!("\nDaftar Perintah:");
            println!("  init     : Membuat template greencode.json & mendeteksi rute otomatis");
            println!("  detect   : Diagnosa proses server aktif di port target");
            println!("  analyze  : Analisis kualitas kode statis (NCLOC, CCN, Maintainability, Smells)");
            println!("  routes   : Menampilkan rute otomatis dari Next.js & Laravel");
            println!("  run      : Menjalankan profiling lengkap (Kualitas Kode + Energi Runtime)");
            println!("  ui       : Membuka dashboard HTML di web browser");
            println!("  version  : Menampilkan versi GreenCode Profiler");
        }
        _ => {
            let config_path = "greencode.json";
            let no_open = args.iter().any(|a| a == "--no-open");
            cmd_run(config_path, no_open);
        }
    }
}
