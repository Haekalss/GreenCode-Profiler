//! Static code-quality report (CCN, MI, NCLOC, code smells, PHP ext candidates, simulation).

use super::render;
use super::simulation::{format_multi_user_simulation_section, format_php_ext_candidates_section};
use super::LOGO_BASE64;
use serde_json::Value;
use std::fs;
use std::path::Path;

const TEMPLATE: &str = include_str!("templates/quality.html");

pub fn generate(report_data: &Value, path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let overview = &report_data["overview"];
    let html = render::render(
        TEMPLATE,
        &[
            ("LOGO", LOGO_BASE64.to_string()),
            ("TOTAL_FILES", render::integer(overview, "total_files").to_string()),
            ("TOTAL_NCLOC", render::integer(overview, "total_ncloc").to_string()),
            (
                "AVG_CCN",
                format!("{:.2}", render::number(overview, "average_ccn_per_file")),
            ),
            (
                "MI",
                format!("{:.1}", render::number(overview, "average_maintainability_index")),
            ),
            ("GRADE", render::text(overview, "maintainability_grade").to_string()),
            (
                "TOTAL_SMELLS",
                render::integer(overview, "total_code_smells").to_string(),
            ),
            (
                "ANALYSIS_PROFILE",
                render::text(overview, "analysis_profile").to_string(),
            ),
            ("ROWS_SMELLS", smell_rows(report_data)),
            ("ROWS_FILES", file_rows(report_data)),
            ("FILTER_BANNER", render::filter_banner(&report_data["applied_filter"])),
            (
                "LANG_CARDS",
                render::language_cards(&overview["language_breakdown"]),
            ),
            (
                "PHP_EXT_SECTION",
                format_php_ext_candidates_section(&report_data["php_extension_candidates"]),
            ),
            (
                "SIMULATION_SECTION",
                format_multi_user_simulation_section(&report_data["multi_user_simulation"]),
            ),
        ],
    );
    fs::write(path, html)
}

fn smell_rows(report_data: &Value) -> String {
    let rows: String = report_data["code_smells"]
        .as_array()
        .map(|list| list.iter().take(50).map(render::smell_row).collect())
        .unwrap_or_default();
    if rows.is_empty() {
        r#"<tr><td colspan="5" style="text-align:center;color:#059669;font-weight:600;padding:18px;">✅ Tidak ditemukan pelanggaran arsitektur (0 Code Smells)</td></tr>"#.to_string()
    } else {
        rows
    }
}

fn file_rows(report_data: &Value) -> String {
    report_data["files"]
        .as_array()
        .map(|list| list.iter().map(render::file_row).collect())
        .unwrap_or_default()
}
