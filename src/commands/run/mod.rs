//! `greenprofiler run`: full energy & code-quality profiling orchestration.

pub mod aggregate;
pub mod profiler;
pub mod report;

use super::config;
use crate::attributor::ModuleAttributor;
use crate::cli::{apply_filters, print_banner, print_filter_keywords, CliFlags};
use crate::detector::{RouteEndpoint, RuntimeDetector};
use crate::engine::ProcessEnergyEngine;
use serde_json::Value;
use std::path::Path;

pub fn execute(config_path: &str, flags: CliFlags) {
    if !Path::new(config_path).exists() {
        println!("[GreenProfiler] Berkas konfigurasi '{}' tidak ditemukan.", config_path);
        println!("            Jalankan 'greenprofiler init' untuk membuatnya otomatis.");
        return;
    }

    let cfg = config::read(config_path);
    let project_name = cfg["project_name"].as_str().unwrap_or("Web Project");
    let runtime = cfg["runtime"].as_str().unwrap_or("auto");
    let base_url = cfg["base_url"].as_str().unwrap_or("http://127.0.0.1:8000");
    let repeats = cfg["repeats"].as_u64().unwrap_or(5) as usize;
    let cpu_tdp = cfg["cpu_tdp_watts"].as_f64().unwrap_or(15.0);

    print_run_header(project_name, runtime, base_url, repeats);

    let Some(endpoints) = prepare_endpoints(&cfg, &flags, config_path) else {
        return;
    };

    let quality_report = profiler::run_quality_analysis(&cfg, &flags);
    let php_audit = profiler::audit_php_runtime(runtime);

    let pids = RuntimeDetector::find_pids(runtime, None);
    if pids.is_empty() {
        println!("[GreenProfiler] PERINGATAN: Tidak ada proses {} terdeteksi.", runtime.to_uppercase());
        println!("            Pastikan server aplikasi Anda sudah dijalankan.");
        return;
    }
    println!("[GreenProfiler] Terhubung ke {} proses aktif: PID {:?}", pids.len(), pids);

    let engine = ProcessEnergyEngine::new(pids, cpu_tdp);
    let mut attributor = ModuleAttributor::new(&engine);

    profiler::profile_endpoints(&mut attributor, &endpoints, &cfg, base_url, repeats);

    let report_data = report::build_report_data(
        &attributor,
        &quality_report,
        &php_audit,
        &flags,
        project_name,
        runtime,
        base_url,
        cpu_tdp,
        repeats,
    );

    report::save_and_open_reports(&report_data, &flags);
}

fn print_run_header(project: &str, runtime: &str, base_url: &str, repeats: usize) {
    print_banner();
    println!("Project  : {}", project);
    println!("Runtime  : {}", runtime.to_uppercase());
    println!("Base URL : {}", base_url);
    println!("Repeats  : {}x per endpoint", repeats);
    println!("----------------------------------------------------------------");
}

fn prepare_endpoints(cfg: &Value, flags: &CliFlags, config_path: &str) -> Option<Vec<RouteEndpoint>> {
    let all = config::endpoints(cfg);
    let total_before = all.len();
    let filtered = apply_filters(all, &flags.filters, &flags.excludes);

    if flags.has_filter() {
        println!(
            "[GreenProfiler] Filter aktif: {} endpoint dipilih dari {} endpoint di {}.",
            filtered.len(),
            total_before,
            config_path
        );
        if filtered.is_empty() {
            println!("[GreenProfiler Error] 0 endpoint cocok dengan --filter / --exclude.");
            println!("            Kata kunci yang dipakai:");
            print_filter_keywords(&flags.filters, &flags.excludes);
            println!("            Jalankan 'greenprofiler routes' untuk melihat daftar endpoint.");
            return None;
        }
    }
    Some(filtered)
}
