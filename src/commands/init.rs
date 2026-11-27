//! `greenprofiler init`: generate `greenprofiler.json` with discovered endpoints.

use crate::cli::apply_filters;
use crate::detector::{RouteDiscoverer, RuntimeDetector};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "greenprofiler.json";

pub fn execute(force: bool, filters: Vec<String>, excludes: Vec<String>) {
    let ecosystem = RuntimeDetector::detect_project_ecosystem(".");
    println!(
        "[GreenProfiler] Mendeteksi ekosistem project: {}",
        ecosystem.to_uppercase()
    );

    let mut config = build_config(&ecosystem);
    if !attach_endpoints(&mut config, &filters, &excludes) {
        return;
    }
    if Path::new(CONFIG_FILE).exists() && !force {
        println!(
            "[GreenProfiler] Berkas '{}' sudah ada. Gunakan flag -f / --force untuk menimpa.",
            CONFIG_FILE
        );
        return;
    }
    write_config(&config);
}

fn build_config(ecosystem: &str) -> Value {
    if ecosystem == "fullstack" {
        if let Some(config) = fullstack_config() {
            return config;
        }
    }
    single_config(ecosystem)
}

fn fullstack_config() -> Option<Value> {
    let info = RuntimeDetector::detect_fullstack_components(".");
    let (frontend, backend) = (info.frontend?, info.backend?);
    println!(
        "[GreenProfiler] Komponen Frontend terdeteksi : {} ({} di port {})",
        frontend.dir,
        frontend.framework.to_uppercase(),
        frontend.default_port
    );
    println!(
        "[GreenProfiler] Komponen Backend terdeteksi  : {} ({} di port {})",
        backend.dir,
        backend.framework.to_uppercase(),
        backend.default_port
    );
    Some(json!({
        "project_name": format!("Full-Stack Monorepo ({} + {})", frontend.dir, backend.dir),
        "runtime": "fullstack",
        "source_dirs": [backend.source_dir, frontend.source_dir],
        "services": {
            "backend": backend.base_url,
            "frontend": frontend.base_url
        },
        "base_url": backend.base_url,
        "auth": { "type": "none" },
        "endpoints": [],
        "repeats": 5,
        "cpu_tdp_watts": 15.0
    }))
}

fn single_config(ecosystem: &str) -> Value {
    let port = default_port(ecosystem);
    let base_url = format!("http://127.0.0.1:{}", port);
    println!(
        "[GreenProfiler] Mengatur base_url otomatis ke port {}: {}",
        port, base_url
    );
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
}

fn default_port(ecosystem: &str) -> u16 {
    match ecosystem {
        "node" => 3000,
        "go" | "rust" => 8080,
        _ => 8000,
    }
}

/// Isi `endpoints`; kembalikan `false` bila filter tidak cocok (batalkan init).
fn attach_endpoints(config: &mut Value, filters: &[String], excludes: &[String]) -> bool {
    let profilable = RouteDiscoverer::profilable(RouteDiscoverer::discover(".", 0));
    let total = profilable.len();
    let discovered = apply_filters(profilable, filters, excludes);

    if discovered.is_empty() {
        return attach_fallback(config, total);
    }
    report_match_count(discovered.len(), total, filters, excludes);
    config["endpoints"] = json!(discovered);
    true
}

fn attach_fallback(config: &mut Value, total: usize) -> bool {
    if total > 0 {
        println!(
            "[GreenProfiler] 0 rute cocok dengan --filter/--exclude (padahal ada {} rute profilable).",
            total
        );
        println!("            Periksa kembali kata kunci, atau jalankan 'greenprofiler routes' untuk melihat daftar.");
        return false;
    }
    println!("[GreenProfiler] Menggunakan template default endpoints.");
    config["endpoints"] = json!([
        { "path": "/", "name": "Homepage", "method": "GET" },
        { "path": "/api/users", "name": "API Users", "method": "GET" }
    ]);
    true
}

fn report_match_count(found: usize, total: usize, filters: &[String], excludes: &[String]) {
    if filters.is_empty() && excludes.is_empty() {
        println!(
            "[GreenProfiler] Berhasil mendeteksi {} rute profilable dari project secara otomatis!",
            found
        );
        return;
    }
    println!(
        "[GreenProfiler] {} rute profilable cocok dengan filter (dari {} di seluruh project).",
        found, total
    );
}

fn write_config(config: &Value) {
    let Ok(contents) = serde_json::to_string_pretty(config) else {
        return;
    };
    if fs::write(CONFIG_FILE, contents).is_ok() {
        println!("[GreenProfiler] Sukses membuat template konfigurasi: {}", CONFIG_FILE);
        println!("            Jalankan 'greenprofiler run' untuk memulai profiling!");
    }
}
