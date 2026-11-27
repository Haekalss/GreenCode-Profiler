//! `greenprofiler simulate`: simulasi beban multi-user, energi, karbon & kapasitas server dari hasil audit.

use crate::cli::{resolve_report_paths, CliFlags};
use crate::reporter::ReportGenerator;
use crate::simulator::{AuditSimulator, SimulationParams};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn execute(flags: CliFlags) {
    let source_path = resolve_source_report(&flags);
    let Some(src) = source_path else {
        println!("[GreenProfiler] Berkas laporan audit tidak ditemukan.");
        println!("            Jalankan 'greenprofiler run' atau 'greenprofiler analyze' terlebih dahulu,");
        println!("            atau berikan path berkas JSON: greenprofiler simulate reports/greenprofiler_report.json");
        return;
    };

    let Ok(raw) = fs::read_to_string(&src) else {
        println!("[GreenProfiler Error] Gagal membaca berkas audit: {}", src);
        return;
    };
    let Ok(audit_val) = serde_json::from_str::<Value>(&raw) else {
        println!("[GreenProfiler Error] Format JSON tidak valid pada berkas: {}", src);
        return;
    };

    let mut params = SimulationParams::default();
    if flags.users > 0 {
        params.custom_users = Some(flags.users);
    }
    if flags.requests_per_user > 0 {
        params.requests_per_user = flags.requests_per_user;
    }
    if flags.cores > 0 {
        params.cpu_cores = flags.cores;
    }
    if flags.window_sec > 0.0 {
        params.concurrency_window_sec = flags.window_sec;
    }

    let sim = AuditSimulator::simulate(&audit_val, &params);

    println!("================================================================================");
    println!("   GREEN PROFILER MULTI-USER SCALE & CAPACITY SIMULATOR (Rust Native)           ");
    println!("================================================================================");
    println!("Sumber Audit         : {} ({})", src, sim.source_mode);
    println!("Project              : {}", sim.project_name);
    println!(
        "Baseline / Request   : {:.6} J | CPU: {:.4} s | Latensi: {:.1} ms (TDP {:.1}W)",
        sim.baseline_energy_j_per_req,
        sim.baseline_cpu_s_per_req,
        sim.baseline_latency_ms,
        sim.cpu_tdp_watts
    );
    println!(
        "Parameter Simulasi   : {} vCPU | {} req/user per {:.0}s window | {:.0} jam sibuk/hari",
        sim.cpu_cores, sim.requests_per_user, sim.concurrency_window_sec, sim.peak_hours_per_day
    );
    println!(
        "Batas Aman (80% CPU) : ~{} user konkuren ({:.1} RPS)",
        sim.max_concurrent_users_80pct, sim.max_sustainable_rps_80pct
    );
    println!(
        "Potensi Penghematan  : {:.1}% ({})",
        sim.estimated_opt_saving_pct, sim.optimization_driver
    );
    println!("--------------------------------------------------------------------------------");
    println!(
        "{:>8} | {:>9} | {:>8} | {:>11} | {:>11} | {:>10} | {}",
        "Users", "RPS", "CPU %", "Lat P95(ms)", "kWh/Bulan", "CO2(kg/bln)", "Status Kapasitas"
    );
    println!("--------------------------------------------------------------------------------");
    for t in &sim.tiers {
        println!(
            "{:>8} | {:>9.1} | {:>7.1}% | {:>11.1} | {:>11.3} | {:>10.3} | {}",
            t.concurrent_users,
            t.throughput_rps,
            t.cpu_utilization_pct,
            t.projected_p95_latency_ms,
            t.monthly_kwh,
            t.monthly_co2_kg,
            t.capacity_status
        );
    }
    println!("--------------------------------------------------------------------------------");
    for rec in &sim.capacity_recommendations {
        println!(" * {}", rec);
    }

    let (html_path, json_path) = resolve_report_paths(
        &flags.output,
        "reports/simulation_report.html",
        "reports/simulation_report.json",
    );

    let sim_json = json!(sim);
    let _ = ReportGenerator::save_json(&sim_json, &json_path);
    let _ = ReportGenerator::generate_simulation_html(&sim_json, &html_path);

    println!("\nLaporan Simulasi tersimpan di:");
    println!(
        " - JSON : {}",
        fs::canonicalize(&json_path)
            .unwrap_or_else(|_| json_path.into())
            .display()
    );
    println!(
        " - HTML : {}",
        fs::canonicalize(&html_path)
            .unwrap_or_else(|_| html_path.clone().into())
            .display()
    );
    println!("================================================================================\n");

    if !flags.no_open {
        ReportGenerator::open_in_browser(&html_path);
    }
}

fn resolve_source_report(flags: &CliFlags) -> Option<String> {
    if let Some(first) = flags.positional.first() {
        if Path::new(first).exists() {
            return Some(first.clone());
        }
    }
    const CANDIDATES: [&str; 2] = ["reports/greenprofiler_report.json", "reports/code_quality.json"];
    CANDIDATES
        .iter()
        .find(|p| Path::new(p).exists())
        .map(|s| (*s).to_string())
}
