//! Metrics aggregation and summary calculations.

use super::php_ext::PhpExtensionCandidate;
use super::{CodeQualityReport, FileAnalysisResult, FunctionMetric, LanguageStat, OverviewMetrics};
use std::collections::BTreeMap;

pub fn aggregate_report(
    mut files_result: Vec<FileAnalysisResult>,
    files_considered: usize,
    mut all_functions: Vec<FunctionMetric>,
    php_extension_candidates: Vec<PhpExtensionCandidate>,
    strict_mode: bool,
) -> CodeQualityReport {
    let total_files = files_result.len();
    let total_ncloc: usize = files_result.iter().map(|f| f.ncloc).sum();
    let total_ccn: u32 = files_result.iter().map(|f| f.ccn).sum();
    let avg_ccn = if total_files > 0 {
        total_ccn as f64 / total_files as f64
    } else {
        0.0
    };
    let avg_mi = if total_files > 0 {
        files_result
            .iter()
            .map(|f| f.maintainability_index)
            .sum::<f64>()
            / total_files as f64
    } else {
        100.0
    };

    let grade = match () {
        _ if avg_mi >= 70.0 => "A (High Maintainability)",
        _ if avg_mi >= 55.0 => "B (Moderate Maintainability)",
        _ if avg_mi >= 40.0 => "C (Low / Technical Debt)",
        _ => "D (High Risk / Severe Technical Debt)",
    };

    // Urutkan file berdasarkan risiko (jumlah smells desc, CCN desc, MI asc)
    files_result.sort_by(|a, b| {
        b.code_smells
            .len()
            .cmp(&a.code_smells.len())
            .then_with(|| b.ccn.cmp(&a.ccn))
            .then_with(|| {
                a.maintainability_index
                    .partial_cmp(&b.maintainability_index)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    all_functions.sort_by(|a, b| b.ccn.cmp(&a.ccn));
    all_functions.truncate(10);

    let mut all_smells = Vec::new();
    let mut lang_map: BTreeMap<String, usize> = BTreeMap::new();
    for f in &files_result {
        *lang_map.entry(f.language.clone()).or_insert(0) += 1;
        all_smells.extend(f.code_smells.clone());
    }

    let language_breakdown: Vec<LanguageStat> = lang_map
        .into_iter()
        .map(|(language, files)| LanguageStat { language, files })
        .collect();

    CodeQualityReport {
        overview: OverviewMetrics {
            total_files,
            total_files_considered: files_considered,
            total_ncloc,
            total_ccn,
            average_ccn_per_file: (avg_ccn * 100.0).round() / 100.0,
            average_maintainability_index: (avg_mi * 100.0).round() / 100.0,
            maintainability_grade: grade.to_string(),
            total_code_smells: all_smells.len(),
            php_extension_candidates_count: php_extension_candidates.len(),
            analysis_profile: if strict_mode {
                "Strict Audit (Uniform 300 NCLOC)".to_string()
            } else {
                "Pragmatic + Research (Backend <=300, UI <=450 NCLOC, Control-Flow Nesting)".to_string()
            },
            language_breakdown,
        },
        files: files_result,
        top_complex_functions: all_functions,
        code_smells: all_smells,
        php_extension_candidates,
    }
}
