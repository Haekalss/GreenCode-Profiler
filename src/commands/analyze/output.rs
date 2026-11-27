//! Console output and report persistence for `greenprofiler analyze`.

use crate::analyzer::CodeQualityReport;
use crate::cli::{resolve_report_paths, CliFlags};
use crate::reporter::ReportGenerator;
use crate::simulator::{AuditSimulator, SimulationParams};
use serde_json::json;

pub fn print_header(targets: &[String], flags: &CliFlags) {
    println!("================================================================");
    println!("   GREEN PROFILER STATIC CODE QUALITY ANALYZER (Rust Native)    ");
    println!("================================================================");
    if flags.has_filter() {
        println!("Target direktori (monorepo, semua source_dirs): {}", targets.join(", "));
        return;
    }
    for target in targets {
        let canonical = std::fs::canonicalize(target).unwrap_or_else(|_| target.into());
        println!("Scanning Directory : {}", canonical.display());
    }
    println!("----------------------------------------------------------------");
}

pub fn print_overview(report: &CodeQualityReport, flags: &CliFlags) {
    let ov = &report.overview;
    if flags.has_filter() {
        println!(
            "Filter aktif             : {} file cocok dengan keyword {:?} (dari {} total file kode)",
            ov.total_files, flags.filters, ov.total_files_considered
        );
    }
    println!("Standar Pengukuran       : {}", ov.analysis_profile);
    println!("Total Files Scanned      : {} files", ov.total_files);
    println!("Total NCLOC              : {} lines", ov.total_ncloc);
    println!("Total Complexity (CCN)   : {}", ov.total_ccn);
    println!("Average CCN per File     : {:.2}", ov.average_ccn_per_file);
    println!(
        "Maintainability Index    : {:.1} / 100 (Grade: {})",
        ov.average_maintainability_index, ov.maintainability_grade
    );
    println!("Architectural Smells     : {} detected", ov.total_code_smells);
    println!(
        "PHP Extension Candidates : {} fungsi berulang/intensif terdeteksi",
        ov.php_extension_candidates_count
    );
    if !ov.language_breakdown.is_empty() {
        let langs: Vec<String> = ov
            .language_breakdown
            .iter()
            .map(|l| format!("{} {} file", l.language, l.files))
            .collect();
        println!("Bahasa Terdeteksi        : {}", langs.join(", "));
    }

    if !report.php_extension_candidates.is_empty() {
        println!("----------------------------------------------------------------");
        println!("Top Kandidat Optimasi Native PHP Extension:");
        for (i, c) in report.php_extension_candidates.iter().take(5).enumerate() {
            println!(
                "  {}. {}() — dipanggil {}x ({}x loop), def {}x [{} | Hemat {}]",
                i + 1,
                c.function_name,
                c.call_count,
                c.loop_invocations,
                c.definition_count,
                c.workload_kind,
                c.estimated_cpu_saving
            );
        }
    }
    println!("----------------------------------------------------------------");
}

pub fn write_reports(report: &CodeQualityReport, flags: &CliFlags) -> String {
    let (html_path, json_path) = resolve_report_paths(
        &flags.output,
        "reports/code_quality.html",
        "reports/code_quality.json",
    );

    let mut json_val = json!(report);
    if flags.has_filter() {
        json_val["applied_filter"] = json!({
            "filters": &flags.filters,
            "excludes": &flags.excludes,
            "matched_files": report.overview.total_files,
            "considered_files": report.overview.total_files_considered
        });
    }

    let mut sim_params = SimulationParams::default();
    if flags.users > 0 {
        sim_params.custom_users = Some(flags.users);
    }
    if flags.requests_per_user > 0 {
        sim_params.requests_per_user = flags.requests_per_user;
    }
    if flags.cores > 0 {
        sim_params.cpu_cores = flags.cores;
    }
    if flags.window_sec > 0.0 {
        sim_params.concurrency_window_sec = flags.window_sec;
    }
    let sim_report = AuditSimulator::simulate(&json_val, &sim_params);
    json_val["multi_user_simulation"] = json!(sim_report);

    let _ = ReportGenerator::save_json(&json_val, &json_path);
    let _ = ReportGenerator::generate_code_quality_html(&json_val, &html_path);

    println!(
        "\nHasil JSON tersimpan di : {}",
        std::fs::canonicalize(&json_path)
            .unwrap_or_else(|_| json_path.into())
            .display()
    );
    println!(
        "Laporan HTML tersimpan di: {}",
        std::fs::canonicalize(&html_path)
            .unwrap_or_else(|_| html_path.clone().into())
            .display()
    );
    println!("================================================================\n");
    html_path
}
