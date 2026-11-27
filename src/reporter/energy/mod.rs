//! HTML report generator for energy & performance profiling.

pub mod sections;

use super::render::*;
use super::simulation::{format_multi_user_simulation_section, format_php_ext_candidates_section};
use super::LOGO_BASE64;
use serde_json::Value;
use std::fs;
use std::path::Path;

const TEMPLATE: &str = include_str!("../templates/energy.html");

pub fn generate<P: AsRef<Path>>(data: &Value, path: P) -> std::io::Result<()> {
    let p = path.as_ref();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }

    let meta = &data["metadata"];
    let totals = &data["totals"];
    let quality = &data["code_quality"];
    let q_overview = &quality["overview"];
    let php = &data["php_environment"];

    let total_files_count = quality["files"].as_array().map(|v| v.len()).unwrap_or(0);

    let vars = [
        ("LOGO", LOGO_BASE64.to_string()),
        ("PROJECT_NAME", text(meta, "project_name").to_string()),
        ("TIMESTAMP", text(meta, "timestamp").to_string()),
        ("RUNTIME", text(meta, "runtime").to_uppercase()),
        ("BASE_URL", text(meta, "base_url").to_string()),
        ("CPU_TDP", format!("{:.1}", number(meta, "cpu_tdp_watts"))),
        ("REPEATS_PER_EP", integer(meta, "repeats_per_endpoint").to_string()),
        ("DETAIL_AGGREGATION", text(meta, "detail_aggregation").to_string()),
        ("TOTAL_REQUESTS", integer(totals, "total_requests").to_string()),
        ("TOTAL_JOULES", format!("{:.4}", number(totals, "total_joules"))),
        ("TOTAL_KWH", format!("{:.8}", number(totals, "total_kwh"))),
        ("TOTAL_CPU", format!("{:.3}", number(totals, "total_cpu_active_s"))),
        ("AVG_LATENCY_MS", format!("{:.1}", number(totals, "avg_latency_ms"))),
        ("TOTAL_CO2", format!("{:.4}", number(totals, "total_co2e_g"))),
        ("TOTAL_FILES", integer(q_overview, "total_files").to_string()),
        ("TOTAL_NCLOC", integer(q_overview, "total_ncloc").to_string()),
        ("AVG_CCN", format!("{:.2}", number(q_overview, "average_ccn_per_file"))),
        ("AVG_MI", format!("{:.1}", number(q_overview, "average_maintainability_index"))),
        ("MI", format!("{:.1}", number(q_overview, "average_maintainability_index"))),
        ("GRADE", text(q_overview, "maintainability_grade").to_string()),
        ("TOTAL_SMELLS", integer(q_overview, "total_code_smells").to_string()),
        ("FILTER_BANNER", filter_banner(&data["applied_filter"])),
        ("LANGUAGE_CARDS", language_cards(&q_overview["language_breakdown"])),
        ("LANG_CARDS", language_cards(&q_overview["language_breakdown"])),
        ("ROWS_CORRELATED", sections::format_module_rows(&data["correlated_modules"])),
        ("ROWS_ENDPOINTS", sections::format_endpoint_rows(&data["endpoints_detail"])),
        ("FILES_SUMMARY", sections::format_files_summary(total_files_count)),
        ("ROWS_FILES", sections::format_file_rows(&quality["files"])),
        ("ROWS_SMELLS", sections::format_smell_rows(&quality["code_smells"])),
        (
            "PHP_EXT_SECTION",
            format_php_ext_candidates_section(&quality["php_extension_candidates"]),
        ),
        (
            "SIMULATION_SECTION",
            format_multi_user_simulation_section(&data["multi_user_simulation"]),
        ),
        ("PHP_SECTION", sections::format_php_section(php)),
    ];

    let html = render(TEMPLATE, &vars);
    fs::write(p, html)
}
