//! Built-in Code Quality & Architecture Analyzer in Pure Rust
//! Menghitung NCLOC, Cyclomatic Complexity (CCN), Maintainability Index (MI), dan Code Smells

pub mod collector;
pub mod metrics;
pub mod scanner;
pub mod smells;
pub mod summary;

pub use smells::CodeSmell;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetric {
    pub name: String,
    pub ccn: u32,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStat {
    pub language: String,
    pub files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewMetrics {
    pub total_files: usize,
    pub total_files_considered: usize,
    pub total_ncloc: usize,
    pub total_ccn: u32,
    pub average_ccn_per_file: f64,
    pub average_maintainability_index: f64,
    pub maintainability_grade: String,
    pub total_code_smells: usize,
    pub language_breakdown: Vec<LanguageStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysisResult {
    pub file_path: String,
    pub relative_path: String,
    pub language: String,
    pub ncloc: usize,
    pub ccn: u32,
    pub maintainability_index: f64,
    pub functions: Vec<FunctionMetric>,
    pub code_smells: Vec<CodeSmell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityReport {
    pub overview: OverviewMetrics,
    pub files: Vec<FileAnalysisResult>,
    pub top_complex_functions: Vec<FunctionMetric>,
    pub code_smells: Vec<CodeSmell>,
}

pub struct CodeQualityAnalyzer {
    pub target_dirs: Vec<PathBuf>,
    pub file_filters: Vec<String>,
    pub file_excludes: Vec<String>,
}

impl CodeQualityAnalyzer {
    pub fn new<P: AsRef<Path>>(targets: Vec<P>) -> Self {
        let target_dirs = targets
            .into_iter()
            .map(|p| p.as_ref().to_path_buf())
            .filter(|p| p.exists())
            .collect();
        Self {
            target_dirs,
            file_filters: Vec::new(),
            file_excludes: Vec::new(),
        }
    }

    pub fn with_file_filters<P: AsRef<Path>>(
        targets: Vec<P>,
        filters: Vec<String>,
        excludes: Vec<String>,
    ) -> Self {
        let target_dirs = targets
            .into_iter()
            .map(|p| p.as_ref().to_path_buf())
            .filter(|p| p.exists())
            .collect();
        Self {
            target_dirs,
            file_filters: filters.into_iter().map(|f| f.to_lowercase()).collect(),
            file_excludes: excludes.into_iter().map(|x| x.to_lowercase()).collect(),
        }
    }

    fn passes_file_filters(&self, path: &Path, content: &str) -> bool {
        let p_low = path.to_string_lossy().to_lowercase();
        let c_low = content.to_lowercase();
        self.matches_filter(&p_low, &c_low)
    }

    fn matches_filter(&self, path_lower: &str, content_lower: &str) -> bool {
        let has_match = |k: &String| path_lower.contains(k) || content_lower.contains(k);
        let passes = self.file_filters.is_empty() || self.file_filters.iter().any(has_match);
        if !passes {
            return false;
        }
        let excluded = self.file_excludes.iter().any(has_match);
        !excluded
    }

    pub fn run_analysis(&self) -> CodeQualityReport {
        const IGNORE_DIRS: &[&str] = &[
            "node_modules", "vendor", ".git", ".next", "dist", "build",
            "storage", "tools", "mariadb", "backup", "data", "temp", "tmp",
        ];

        let mut code_files = Vec::new();
        for target in &self.target_dirs {
            collector::collect_code_files(target, IGNORE_DIRS, &mut code_files);
        }

        let mut files_result = Vec::new();
        let mut all_functions = Vec::new();
        let mut files_considered = 0usize;

        for path in code_files {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            let Some(lang) = collector::detect_language(&ext) else {
                continue;
            };

            files_considered += 1;

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };

            let has_filters = !self.file_filters.is_empty() || !self.file_excludes.is_empty();
            if has_filters && !self.passes_file_filters(&path, &content) {
                continue;
            }

            let result = scanner::analyze_source_file(&path, lang, &content);
            all_functions.extend(result.functions.clone());
            files_result.push(result);
        }

        summary::aggregate_report(files_result, files_considered, all_functions)
    }
}
