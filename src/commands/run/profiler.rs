//! Orchestrator for profiling endpoints and invoking audits.

use super::super::config;
use crate::analyzer::{CodeQualityAnalyzer, CodeQualityReport};
use crate::attributor::ModuleAttributor;
use crate::cli::CliFlags;
use crate::detector::RouteEndpoint;
use crate::php_auditor::{PhpAuditReport, PhpAuditor};
use serde_json::Value;

pub fn run_quality_analysis(cfg: &Value, flags: &CliFlags) -> CodeQualityReport {
    let targets = config::targets(cfg);
    let inferred = if targets.is_empty() {
        vec![".".to_string()]
    } else {
        targets
    };

    println!("[GreenProfiler] [1/4] Menjalankan Analisis Kualitas Struktur Kode (Rust Analyzer)...");
    if flags.has_filter() && !inferred.is_empty() {
        println!(
            "            Target direktori (monorepo, semua source_dirs): {}",
            inferred.join(", ")
        );
    }

    let analyzer = if flags.has_filter() {
        CodeQualityAnalyzer::with_file_filters(
            inferred,
            flags.filters.clone(),
            flags.excludes.clone(),
        )
    } else {
        CodeQualityAnalyzer::new(inferred)
    }
    .with_strict(flags.strict);

    let report = analyzer.run_analysis();
    let q_ov = &report.overview;
    if flags.has_filter() {
        println!(
            "            Filter file: {} cocok dengan keyword {:?} dari {} total file kode.",
            q_ov.total_files, flags.filters, q_ov.total_files_considered
        );
    }
    println!(
        "            Files: {} | NCLOC: {} | Avg CCN: {:.2} | MI: {:.1}/100 | Smells: {} | PHP Ext Candidates: {}",
        q_ov.total_files,
        q_ov.total_ncloc,
        q_ov.average_ccn_per_file,
        q_ov.average_maintainability_index,
        q_ov.total_code_smells,
        q_ov.php_extension_candidates_count
    );
    if !q_ov.language_breakdown.is_empty() {
        let langs: Vec<String> = q_ov
            .language_breakdown
            .iter()
            .map(|l| format!("{} {} file", l.language, l.files))
            .collect();
        println!("            Bahasa terdeteksi: {}", langs.join(", "));
    }
    report
}

pub fn audit_php_runtime(runtime: &str) -> Option<PhpAuditReport> {
    if runtime != "php" && runtime != "fullstack" && runtime != "auto" {
        return None;
    }
    println!("\n[GreenProfiler] [2/4] Mengaudit PHP Runtime & Green Extensions (OPcache, JIT, Xdebug)...");
    let rep = PhpAuditor::audit();
    if rep.is_php_available {
        println!(
            "            PHP: {} | Extensions: {}",
            rep.php_version, rep.loaded_extensions_count
        );
        println!(
            "            OPcache: {} | Xdebug: {}",
            if rep.opcache_enabled {
                "Aktif [OK]"
            } else {
                "Nonaktif [BOROS ENERGI]"
            },
            if rep.xdebug_active {
                "Aktif [OVERHEAD TINGGI]"
            } else {
                "Nonaktif [OK]"
            }
        );
    }
    Some(rep)
}

pub fn profile_endpoints(
    attributor: &mut ModuleAttributor,
    endpoints: &[RouteEndpoint],
    cfg: &Value,
    base_url: &str,
    repeats: usize,
) {
    println!(
        "\n[GreenProfiler] [4/4] Memulai profiling runtime {} endpoint...",
        endpoints.len()
    );
    for ep in endpoints {
        let url = config::endpoint_url(ep, base_url, &cfg["services"]);
        println!("\n-> [{}] {} ({})", ep.method, ep.name, url);
        for i in 1..=repeats {
            let rec = attributor.trace_http_request(&url, &ep.method, &[], None);
            println!(
                "   Run {}/{} | HTTP {:<3} | Latency: {:6.1} ms | CPU: {:6.4} s | Energi: {:8.6} J",
                i, repeats, rec.status, rec.latency_ms, rec.cpu_active_time_s, rec.energy_total_joules
            );
        }
    }
}
