//! Single-file scanning logic and metric extraction.

use super::metrics::{branch_regex, calculate_maintainability_index, function_regex};
use super::smells::{make_deep_nesting, make_fat_controller};
use super::{FileAnalysisResult, FunctionMetric};
use std::path::Path;

pub fn is_ignorable_line(trimmed: &str) -> bool {
    const PREFIXES: [&str; 4] = ["//", "#", "/*", "*"];
    trimmed.is_empty() || PREFIXES.iter().any(|p| trimmed.starts_with(p))
}

pub fn analyze_source_file(
    path: &Path,
    lang: &str,
    content: &str,
) -> FileAnalysisResult {
    let b_re = branch_regex();
    let f_re = function_regex();
    let file_str = path.to_string_lossy().to_string();

    let mut ncloc = 0;
    let mut file_ccn = 1;
    let mut file_smells = Vec::new();

    let lines: Vec<&str> = content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_ignorable_line(trimmed) {
            continue;
        }
        ncloc += 1;

        let matches = b_re.find_iter(line).count();
        file_ccn += matches as u32;

        let spaces = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        let indent_level = spaces / 4;
        if indent_level >= 4 && b_re.is_match(line) {
            file_smells.push(make_deep_nesting(&file_str, idx + 1, indent_level));
        }
    }

    let file_functions = extract_functions(content, &f_re, &file_str);

    if ncloc > 300 {
        file_smells.push(make_fat_controller(&file_str, ncloc));
    }

    let mi = calculate_maintainability_index(ncloc, file_ccn);
    let rel_path = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    FileAnalysisResult {
        file_path: file_str,
        relative_path: rel_path,
        language: lang.to_string(),
        ncloc,
        ccn: file_ccn,
        maintainability_index: mi,
        functions: file_functions,
        code_smells: file_smells,
    }
}

fn extract_functions(content: &str, f_re: &regex::Regex, file_str: &str) -> Vec<FunctionMetric> {
    let mut file_functions = Vec::new();
    for cap in f_re.captures_iter(content) {
        let fname = cap.get(1).or_else(|| cap.get(2)).or_else(|| cap.get(3)).or_else(|| cap.get(4));
        if let Some(m) = fname {
            file_functions.push(FunctionMetric {
                name: m.as_str().to_string(),
                ccn: 1,
                file: file_str.to_string(),
                line: 1,
            });
        }
    }
    file_functions
}
