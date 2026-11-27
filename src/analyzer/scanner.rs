//! Single-file scanning logic, control-flow nesting tracker, and function metric extraction.

use super::metrics::{
    arrow_callback_regex, branch_regex, calculate_maintainability_index, control_flow_regex,
    function_regex, loop_control_regex, max_control_flow_nesting, max_ncloc_for_extension,
};
use super::smells::{make_deep_nesting, make_fat_controller};
use super::{FileAnalysisResult, FunctionMetric};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BraceScope {
    FunctionBoundary,
    ControlFlow,
    Neutral,
}

pub fn is_ignorable_line(trimmed: &str) -> bool {
    const PREFIXES: [&str; 4] = ["//", "#", "/*", "*"];
    trimmed.is_empty() || PREFIXES.iter().any(|p| trimmed.starts_with(p))
}

/// Menghapus isi literal string sederhana agar kurung kurawal di dalam string tidak mengacaukan pelacak blok.
fn strip_string_literals(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_single = false;
    let mut in_double = false;
    let mut prev_backslash = false;

    for ch in line.chars() {
        if !in_double && ch == '\'' && !prev_backslash {
            in_single = !in_single;
            out.push(' ');
            prev_backslash = false;
            continue;
        }
        if !in_single && ch == '"' && !prev_backslash {
            in_double = !in_double;
            out.push(' ');
            prev_backslash = false;
            continue;
        }
        if in_single || in_double {
            prev_backslash = ch == '\\' && !prev_backslash;
            out.push(' ');
        } else {
            prev_backslash = false;
            out.push(ch);
        }
    }
    if let Some(idx) = out.find("//") {
        out.truncate(idx);
    }
    out
}

fn current_cf_depth(stack: &[BraceScope]) -> usize {
    let mut depth = 0;
    for scope in stack.iter().rev() {
        match scope {
            BraceScope::FunctionBoundary => break,
            BraceScope::ControlFlow => depth += 1,
            BraceScope::Neutral => {}
        }
    }
    depth
}

pub fn analyze_source_file(
    path: &Path,
    ext: &str,
    lang: &str,
    content: &str,
    strict: bool,
) -> FileAnalysisResult {
    let b_re = branch_regex();
    let cf_re = control_flow_regex();
    let loop_re = loop_control_regex();
    let f_re = function_regex();
    let arrow_cb_re = arrow_callback_regex();

    let file_str = path.to_string_lossy().to_string();
    let max_ncloc = max_ncloc_for_extension(ext, strict);
    let nesting_limit = max_control_flow_nesting(strict);

    let mut ncloc = 0;
    let mut file_ccn = 1;
    let mut file_smells = Vec::new();
    let mut scope_stack: Vec<BraceScope> = Vec::new();
    let mut py_cf_indents: Vec<usize> = Vec::new();
    let mut py_fn_indent: usize = 0;
    let mut pending_scope: Option<BraceScope> = None;

    let mut file_functions: Vec<FunctionMetric> = Vec::new();
    let mut active_fn: Option<(FunctionMetric, usize, bool)> = None; // (metric, start_stack_len, has_loop)

    for (idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if is_ignorable_line(trimmed) {
            continue;
        }
        ncloc += 1;

        let clean = strip_string_literals(raw_line);
        let clean_trim = clean.trim();

        let matches = b_re.find_iter(&clean).count() as u32;
        file_ccn += matches;

        if let Some((ref mut fm, _, ref mut fn_loop)) = active_fn {
            fm.ccn += matches;
            fm.body_lines += 1;
            if loop_re.is_match(clean_trim) {
                *fn_loop = true;
            }
        }

        if ext == "py" {
            let spaces = raw_line
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .map(|c| if c == '\t' { 4 } else { 1 })
                .sum::<usize>();
            if clean_trim.starts_with("def ") || clean_trim.starts_with("async def ") {
                py_fn_indent = spaces;
                py_cf_indents.clear();
            } else {
                while let Some(&last_ind) = py_cf_indents.last() {
                    if spaces <= last_ind {
                        py_cf_indents.pop();
                    } else {
                        break;
                    }
                }
                if spaces > py_fn_indent && cf_re.is_match(clean_trim) && clean_trim.ends_with(':') {
                    py_cf_indents.push(spaces);
                    if py_cf_indents.len() >= nesting_limit {
                        file_smells.push(make_deep_nesting(
                            &file_str,
                            idx + 1,
                            py_cf_indents.len(),
                            strict,
                        ));
                    }
                }
            }
            continue;
        }

        // Deteksi deklarasi fungsi pada baris ini (cepat: lewati regex berat bila kata kunci fungsi tidak ada)
        let mut line_fn_name: Option<String> = None;
        if clean.contains("function ") || clean.contains("fn ") || clean.contains("=>") {
            if let Some(cap) = f_re.captures(&clean) {
                let fname = cap
                    .get(1)
                    .or_else(|| cap.get(2))
                    .or_else(|| cap.get(3))
                    .or_else(|| cap.get(4));
                if let Some(m) = fname {
                    line_fn_name = Some(m.as_str().to_string());
                }
            }
        }

        let is_cf_line = cf_re.is_match(clean_trim);
        let is_arrow_block = !strict && arrow_cb_re.is_match(&clean) && line_fn_name.is_none();

        let next_open_kind = if line_fn_name.is_some() || is_arrow_block {
            Some(BraceScope::FunctionBoundary)
        } else if is_cf_line {
            Some(BraceScope::ControlFlow)
        } else {
            pending_scope.take()
        };

        let mut first_open_used = false;
        let mut opened_cf_on_line = false;

        for ch in clean.chars() {
            match ch {
                '{' => {
                    let kind = if !first_open_used {
                        first_open_used = true;
                        next_open_kind.unwrap_or(BraceScope::Neutral)
                    } else {
                        BraceScope::Neutral
                    };

                    if let Some(fname) = line_fn_name.take() {
                        if let Some((mut prev_fm, _, prev_loop)) = active_fn.take() {
                            prev_fm.has_loop = prev_loop;
                            file_functions.push(prev_fm);
                        }
                        active_fn = Some((
                            FunctionMetric {
                                name: fname,
                                ccn: 1,
                                file: file_str.clone(),
                                line: idx + 1,
                                body_lines: 1,
                                has_loop: false,
                            },
                            scope_stack.len(),
                            false,
                        ));
                    }

                    scope_stack.push(kind);
                    if kind == BraceScope::ControlFlow {
                        opened_cf_on_line = true;
                    }
                }
                '}' => {
                    scope_stack.pop();
                    if let Some((_, start_len, _)) = active_fn {
                        if scope_stack.len() <= start_len {
                            if let Some((mut done_fm, _, done_loop)) = active_fn.take() {
                                done_fm.has_loop = done_loop;
                                file_functions.push(done_fm);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Jika header fungsi/control-flow memiliki `{` di baris berikutnya (gaya Allman/PSR-12 PHP)
        if !first_open_used && !clean_trim.ends_with(';') {
            if let Some(fname) = line_fn_name {
                if let Some((mut prev_fm, _, prev_loop)) = active_fn.take() {
                    prev_fm.has_loop = prev_loop;
                    file_functions.push(prev_fm);
                }
                active_fn = Some((
                    FunctionMetric {
                        name: fname,
                        ccn: 1,
                        file: file_str.clone(),
                        line: idx + 1,
                        body_lines: 1,
                        has_loop: false,
                    },
                    scope_stack.len(),
                    false,
                ));
                pending_scope = Some(BraceScope::FunctionBoundary);
            } else if is_cf_line {
                pending_scope = Some(BraceScope::ControlFlow);
            }
        }

        if opened_cf_on_line {
            let cf_depth = current_cf_depth(&scope_stack);
            if cf_depth >= nesting_limit {
                file_smells.push(make_deep_nesting(&file_str, idx + 1, cf_depth, strict));
            }
        }
    }

    if let Some((mut last_fm, _, last_loop)) = active_fn.take() {
        last_fm.has_loop = last_loop;
        file_functions.push(last_fm);
    }

    if ncloc > max_ncloc {
        file_smells.push(make_fat_controller(&file_str, ncloc, max_ncloc, strict));
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
