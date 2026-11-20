//! Built-in Code Quality & Architecture Analyzer in Pure Rust
//! Menghitung NCLOC, Cyclomatic Complexity (CCN), Maintainability Index (MI), dan Code Smells

use regex::Regex;
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
pub struct CodeSmell {
    pub smell_type: String,
    pub severity: String,
    pub message: String,
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
    /// Jumlah file kode yang ditemukan sebelum filter file diterapkan.
    pub total_files_considered: usize,
    pub total_ncloc: usize,
    pub total_ccn: u32,
    pub average_ccn_per_file: f64,
    pub average_maintainability_index: f64,
    pub maintainability_grade: String,
    pub total_code_smells: usize,
    /// Sebaran file per bahasa (mis. php, typescript) — membuktikan cakupan monorepo.
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
    /// Kata kunci filter file (case-insensitive, cocok pada path ATAU isi file).
    pub file_filters: Vec<String>,
    /// Kata kunci exclude file (case-insensitive, cocok pada path ATAU isi file).
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

    /// Buat analyzer dengan filter file. File hanya dianalisis jika path atau
    /// isinya mengandung salah satu kata kunci `filters`, dan tidak mengandung
    /// kata kunci `excludes`.
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

    fn matches_file_filters(&self, path_lower: &str, content_lower: &str) -> bool {
        let passes_filter = self.file_filters.is_empty()
            || self.file_filters
                .iter()
                .any(|k| path_lower.contains(k) || content_lower.contains(k));
        if !passes_filter {
            return false;
        }
        let excluded = self
            .file_excludes
            .iter()
            .any(|k| path_lower.contains(k) || content_lower.contains(k));
        !excluded
    }

    pub fn run_analysis(&self) -> CodeQualityReport {
        let ignore_dirs = [
            "node_modules", "vendor", ".git", ".next", "dist", "build",
            "storage", "tools", "mariadb", "backup", "data", "temp", "tmp",
        ];

        let branch_regex = Regex::new(
            r"(?i)\b(if|else\s*if|elif|for|foreach|while|case|catch|except|match)\b|&&|\|\||\band\b|\bor\b|\?\s*[^:]+:|\?\?"
        ).unwrap();

        let func_regex = Regex::new(
            r"(?m)^\s*(?:pub\s+(?:\([^\)]+\)\s+)?)?(?:async\s+)?(?:unsafe\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(|^\s*(?:public|protected|private|static|\s)*\s*function\s+([a-zA-Z0-9_]+)\s*\(|^\s*(?:export\s+)?(?:async\s+)?function\s+([a-zA-Z0-9_]+)\s*\(|const\s+([a-zA-Z0-9_]+)\s*=\s*(?:async\s*)?\("
        ).unwrap();

        let mut files_result = Vec::new();
        let mut all_functions = Vec::new();
        let mut all_smells = Vec::new();
        let mut files_considered = 0usize;

        let mut code_files = Vec::new();
        for target in &self.target_dirs {
            collect_code_files(target, &ignore_dirs, &mut code_files);
        }

        for path in code_files {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            let lang = match ext.as_str() {
                "php" => "php",
                "ts" | "tsx" => "typescript",
                "js" | "jsx" => "javascript",
                "py" => "python",
                "go" => "go",
                "rs" => "rust",
                _ => continue,
            };

            files_considered += 1;

            if let Ok(content) = fs::read_to_string(&path) {
                // Terapkan filter file (path ATAU isi) sebelum menghitung metrik.
                if self.file_filters.len() > 0 || self.file_excludes.len() > 0 {
                    let path_lower = path.to_string_lossy().to_lowercase();
                    let content_lower = content.to_lowercase();
                    if !self.matches_file_filters(&path_lower, &content_lower) {
                        continue;
                    }
                }

                let mut ncloc = 0;
                let mut file_ccn = 1;
                let mut file_smells = Vec::new();
                let mut file_functions = Vec::new();

                let lines: Vec<&str> = content.lines().collect();

                for (idx, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.is_empty()
                        || trimmed.starts_with("//")
                        || trimmed.starts_with('#')
                        || trimmed.starts_with("/*")
                        || trimmed.starts_with('*')
                    {
                        continue;
                    }
                    ncloc += 1;

                    let matches = branch_regex.find_iter(line).count();
                    file_ccn += matches as u32;

                    let indent_level = line.chars().take_while(|c| *c == ' ' || *c == '\t').count() / 4;
                    if indent_level >= 4 && branch_regex.is_match(line) {
                        file_smells.push(CodeSmell {
                            smell_type: "Deep Nesting".to_string(),
                            severity: "Medium".to_string(),
                            message: format!("Logika percabangan bersarang dalam (level {})", indent_level),
                            file: path.to_string_lossy().to_string(),
                            line: idx + 1,
                        });
                    }
                }

                for cap in func_regex.captures_iter(&content) {
                    let fname = cap.get(1).or_else(|| cap.get(2)).or_else(|| cap.get(3)).or_else(|| cap.get(4));
                    if let Some(m) = fname {
                        file_functions.push(FunctionMetric {
                            name: m.as_str().to_string(),
                            ccn: 1,
                            file: path.to_string_lossy().to_string(),
                            line: 1,
                        });
                    }
                }

                if ncloc > 300 {
                    file_smells.push(CodeSmell {
                        smell_type: "Fat Controller / Large Class".to_string(),
                        severity: "High".to_string(),
                        message: format!("Ukuran kelas terlalu besar ({} baris NCLOC, ambang batas normal <= 300)", ncloc),
                        file: path.to_string_lossy().to_string(),
                        line: 1,
                    });
                }

                let mi = calculate_maintainability_index(ncloc, file_ccn);

                all_functions.extend(file_functions.clone());
                all_smells.extend(file_smells.clone());

                files_result.push(FileAnalysisResult {
                    file_path: path.to_string_lossy().to_string(),
                    relative_path: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    language: lang.to_string(),
                    ncloc,
                    ccn: file_ccn,
                    maintainability_index: mi,
                    functions: file_functions,
                    code_smells: file_smells,
                });
            }
        }

        let total_files = files_result.len();
        let total_ncloc: usize = files_result.iter().map(|f| f.ncloc).sum();
        let total_ccn: u32 = files_result.iter().map(|f| f.ccn).sum();
        let avg_ccn = if total_files > 0 { total_ccn as f64 / total_files as f64 } else { 0.0 };
        let avg_mi = if total_files > 0 {
            files_result.iter().map(|f| f.maintainability_index).sum::<f64>() / total_files as f64
        } else {
            100.0
        };

        let grade = if avg_mi >= 85.0 {
            "A (High Maintainability)"
        } else if avg_mi >= 65.0 {
            "B (Moderate)"
        } else {
            "C (Low / Technical Debt)"
        };

        all_functions.sort_by(|a, b| b.ccn.cmp(&a.ccn));
        all_functions.truncate(10);

        // Sebaran file per bahasa — memastikan cakupan monorepo (FE + BE) terlihat.
        let mut lang_map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for f in &files_result {
            *lang_map.entry(f.language.clone()).or_insert(0) += 1;
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
                language_breakdown,
            },
            files: files_result,
            top_complex_functions: all_functions,
            code_smells: all_smells,
        }
    }
}

fn collect_code_files(dir: &Path, ignore: &[&str], out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if ignore.contains(&name.as_str()) || name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                collect_code_files(&path, ignore, out);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
}

fn calculate_maintainability_index(ncloc: usize, ccn: u32) -> f64 {
    if ncloc == 0 {
        return 100.0;
    }
    let ncloc_f = ncloc as f64;
    let ccn_f = ccn as f64;
    let volume = (ncloc_f * 5.0).max(1.0);

    let raw_mi = 171.0 - 5.2 * volume.ln() - 0.23 * ccn_f - 16.2 * ncloc_f.ln();
    let normalized = (raw_mi * 100.0 / 171.0).clamp(0.0, 100.0);
    (normalized * 10.0).round() / 10.0
}
