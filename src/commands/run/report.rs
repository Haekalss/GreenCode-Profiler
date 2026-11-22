//! Report data builder and file output handler for `greencode run`.

use super::aggregate::aggregate_endpoint_details;
use crate::analyzer::CodeQualityReport;
use crate::attributor::ModuleAttributor;
use crate::cli::{get_simple_timestamp, resolve_report_paths, CliFlags};
use crate::correlator::GreenCorrelator;
use crate::php_auditor::PhpAuditReport;
use crate::reporter::ReportGenerator;
use serde_json::{json, Value};

pub fn build_report_data(
    attributor: &ModuleAttributor,
    quality_report: &CodeQualityReport,
    php_audit: &Option<PhpAuditReport>,
    flags: &CliFlags,
    project_name: &str,
    runtime: &str,
    base_url: &str,
    cpu_tdp: f64,
    repeats: usize,
) -> Value {
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

    let endpoints_detail = aggregate_endpoint_details(&attributor.records);

    json!({
        "metadata": {
            "project_name": project_name,
            "runtime": runtime,
            "base_url": base_url,
            "engine": "Rust Native Architecture v1.8.1",
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
        "php_environment": serde_json::to_value(php_audit).unwrap_or(json!(null)),
        "correlated_modules": correlated,
        "code_quality": serde_json::to_value(quality_report).unwrap(),
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
    })
}

pub fn save_and_open_reports(report_data: &Value, flags: &CliFlags) {
    let (html_path, json_path) = resolve_report_paths(
        &flags.output,
        "reports/greencode_report.html",
        "reports/greencode_report.json",
    );

    let _ = ReportGenerator::save_json(report_data, &json_path);
    let _ = ReportGenerator::generate_html(report_data, &html_path);

    println!("\n================================================================");
    println!("                     PROFILING SELESAI                          ");
    println!("================================================================");
    println!("Laporan Komprehensif tersimpan di:");
    println!(" - JSON : {}", std::fs::canonicalize(&json_path).unwrap_or_else(|_| json_path.clone().into()).display());
    println!(" - HTML : {}", std::fs::canonicalize(&html_path).unwrap_or_else(|_| html_path.clone().into()).display());
    println!("================================================================\n");

    if !flags.no_open {
        ReportGenerator::open_in_browser(&html_path);
    }
}
