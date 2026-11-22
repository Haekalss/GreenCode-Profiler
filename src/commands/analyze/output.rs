//! Console output and report persistence for `greencode analyze`.

use crate::analyzer::CodeQualityReport;
use crate::cli::{resolve_report_paths, CliFlags};
use crate::reporter::ReportGenerator;
use serde_json::json;

pub fn print_header(targets: &[String], flags: &CliFlags) {
    println!("================================================================");
    println!("   GREENCODE STATIC CODE QUALITY ANALYZER (Rust Native)         ");
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
    println!("Total Files Scanned      : {} files", ov.total_files);
    println!("Total NCLOC              : {} lines", ov.total_ncloc);
    println!("Total Complexity (CCN)   : {}", ov.total_ccn);
    println!("Average CCN per File     : {:.2}", ov.average_ccn_per_file);
    println!(
        "Maintainability Index    : {:.1} / 100 (Grade: {})",
        ov.average_maintainability_index, ov.maintainability_grade
    );
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
}

pub fn write_reports(report: &CodeQualityReport, flags: &CliFlags) -> String {
    let (html_path, json_path) = resolve_report_paths(
        &flags.output,
        "reports/code_quality.html",
        "reports/code_quality.json",
    );
    let json_val = json!(report);
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
