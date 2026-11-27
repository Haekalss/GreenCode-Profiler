//! Detektor fungsi PHP berulang / intensif CPU yang layak dikompilasi menjadi Native PHP Extension.

use super::FileAnalysisResult;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpExtensionCandidate {
    pub function_name: String,
    pub defined_in: String,
    pub definition_count: usize,
    pub call_count: usize,
    pub caller_files: usize,
    pub loop_invocations: usize,
    pub max_ccn: u32,
    pub workload_kind: String,
    pub estimated_cpu_saving: String,
    pub offload_score: u32,
    pub recommendation: String,
}

#[derive(Default)]
struct PhpFuncAggregate {
    definitions: Vec<(String, usize)>,
    max_ccn: u32,
    max_body_lines: usize,
    has_internal_loop: bool,
    workload_tags: BTreeSet<&'static str>,
    call_count: usize,
    caller_files: BTreeSet<String>,
    loop_invocations: usize,
}

const IGNORED_PHP_NAMES: &[&str] = &[
    // Keywords & language constructs
    "if", "elseif", "for", "foreach", "while", "switch", "match", "catch", "function", "fn",
    "return", "array", "isset", "empty", "unset", "list", "echo", "print", "require", "include",
    "require_once", "include_once", "new", "clone", "die", "exit",
    // Magic & standard lifecycle/CRUD methods
    "__construct", "__destruct", "__invoke", "__call", "__callStatic", "__get", "__set",
    "__toString", "boot", "booted", "register", "up", "down", "handle", "rules", "authorize",
    "messages", "attributes", "index", "store", "show", "create", "edit", "update", "destroy",
    // Common Eloquent / Collection / Request / Framework methods
    "where", "whereIn", "whereNotNull", "whereNull", "whereBetween", "orWhere", "whereHas",
    "with", "load", "select", "orderBy", "latest", "oldest", "limit", "take", "skip", "get",
    "first", "firstOrFail", "find", "findOrFail", "pluck", "count", "sum", "avg", "exists",
    "paginate", "save", "delete", "fill", "forceFill", "createMany", "insert", "upsert",
    "belongsTo", "hasMany", "hasOne", "belongsToMany", "morphTo", "morphMany",
    "collect", "map", "filter", "reject", "reduce", "each", "values", "keys", "all", "toArray",
    "merge", "only", "except", "input", "query", "boolean", "integer", "string", "validated",
    "validate", "response", "json", "download", "view", "redirect", "route", "url", "asset",
    "config", "env", "app", "auth", "user", "now", "today", "abort", "abort_if", "abort_unless",
    "transaction", "beginTransaction", "commit", "rollBack", "lockForUpdate", "table",
    // Trivial PHP stdlib calls
    "count", "is_array", "is_string", "is_null", "is_numeric", "in_array", "array_merge",
    "array_keys", "array_values", "array_filter", "array_map", "array_column", "trim", "strtolower",
    "strtoupper", "substr", "strlen", "str_replace", "explode", "implode", "sprintf", "date",
    "time", "strtotime", "intval", "floatval", "round", "min", "max", "abs", "dd", "dump",
];

fn classify_php_body(snippet: &str) -> BTreeSet<&'static str> {
    let mut tags = BTreeSet::new();
    let lower = snippet.to_lowercase();
    if lower.contains("domdocument")
        || lower.contains("simplexmlelement")
        || lower.contains("ziparchive")
        || lower.contains("w:p")
        || lower.contains("xml")
        || lower.contains("htmlspecialchars")
    {
        tags.insert("XML/DOCX & String Builder");
    }
    if lower.contains("preg_match") || lower.contains("preg_replace") || lower.contains("preg_split") {
        tags.insert("Regex & Text Parsing");
    }
    if lower.contains("hash")
        || lower.contains("openssl_")
        || lower.contains("Sodium")
        || lower.contains("base64_")
        || lower.contains("crypt")
    {
        tags.insert("Kriptografi & Encoding");
    }
    if lower.contains("json_encode") || lower.contains("json_decode") || lower.contains("serialize") {
        tags.insert("Serialisasi Data");
    }
    if lower.contains("foreach") || lower.contains("while") || lower.contains("for (") || lower.contains("for(") {
        tags.insert("Iterasi & Transformasi Koleksi");
    }
    tags
}

pub fn detect_php_extension_candidates(
    php_sources: &[(FileAnalysisResult, String)],
) -> Vec<PhpExtensionCandidate> {
    if php_sources.is_empty() {
        return Vec::new();
    }

    let ignored: BTreeSet<&str> = IGNORED_PHP_NAMES.iter().copied().collect();
    let call_re = Regex::new(r"(?:->|::|\b)([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
    let def_re = Regex::new(r"\bfunction\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
    let loop_re = Regex::new(r"\b(?:foreach|for|while)\s*\(").unwrap();

    let mut agg: BTreeMap<String, PhpFuncAggregate> = BTreeMap::new();

    // Pass 1: Kumpulkan seluruh definisi fungsi/method PHP buatan developer beserta profil bodinya
    for (file_res, content) in php_sources {
        let lines: Vec<&str> = content.lines().collect();
        for fm in &file_res.functions {
            if ignored.contains(fm.name.as_str()) || fm.name.len() < 3 {
                continue;
            }
            let start = fm.line.saturating_sub(1);
            let end = (start + fm.body_lines.max(5)).min(lines.len());
            let snippet = if start < end {
                lines[start..end].join("\n")
            } else {
                String::new()
            };

            // Abaikan method relasi Eloquent murni (belongsTo, hasMany, dll.)
            if snippet.contains("->hasMany(")
                || snippet.contains("->belongsTo(")
                || snippet.contains("->hasOne(")
                || snippet.contains("->belongsToMany(")
                || snippet.contains("->morphMany(")
                || snippet.contains("->morphTo(")
            {
                continue;
            }

            let entry = agg.entry(fm.name.clone()).or_default();
            entry.definitions.push((file_res.file_path.clone(), fm.line));
            entry.max_ccn = entry.max_ccn.max(fm.ccn);
            entry.max_body_lines = entry.max_body_lines.max(fm.body_lines);
            entry.has_internal_loop |= fm.has_loop;
            for tag in classify_php_body(&snippet) {
                entry.workload_tags.insert(tag);
            }
        }
    }

    if agg.is_empty() {
        return Vec::new();
    }

    // Pass 2: Hitung frekuensi pemanggilan fungsi-fungsi tersebut di seluruh codebase PHP (termasuk di dalam loop)
    for (file_res, content) in php_sources {
        let mut loop_brace_depths: Vec<i32> = Vec::new();
        let mut brace_depth: i32 = 0;

        for raw_line in content.lines() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with('#') {
                continue;
            }

            let def_name_on_line = def_re
                .captures(trimmed)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str());

            let in_loop = !loop_brace_depths.is_empty();

            for cap in call_re.captures_iter(trimmed) {
                let Some(m) = cap.get(1) else { continue };
                let name = m.as_str();
                if Some(name) == def_name_on_line {
                    continue;
                }
                if let Some(entry) = agg.get_mut(name) {
                    entry.call_count += 1;
                    entry.caller_files.insert(file_res.file_path.clone());
                    if in_loop {
                        entry.loop_invocations += 1;
                    }
                }
            }

            let opens_loop = loop_re.is_match(trimmed);
            let open_braces = trimmed.chars().filter(|&c| c == '{').count() as i32;
            let close_braces = trimmed.chars().filter(|&c| c == '}').count() as i32;

            if opens_loop && open_braces > 0 {
                loop_brace_depths.push(brace_depth);
            }
            brace_depth = (brace_depth + open_braces - close_braces).max(0);
            while let Some(&ld) = loop_brace_depths.last() {
                if brace_depth <= ld {
                    loop_brace_depths.pop();
                } else {
                    break;
                }
            }
        }
    }

    // Pass 3: Evaluasi kandidat yang layak dijadikan Native PHP Extension (Rust ext-php-rs / C Zend / Zephir)
    let mut candidates: Vec<PhpExtensionCandidate> = Vec::new();

    for (name, data) in agg {
        let def_count = data.definitions.len();
        let caller_files_count = data.caller_files.len();

        // Kriteria kelayakan komputasi (bukan getter 1 baris trivial tanpa percabangan/transformasi):
        let has_computational_work =
            data.max_ccn >= 2 || data.has_internal_loop || !data.workload_tags.is_empty() || data.max_body_lines >= 8;
        if !has_computational_work {
            continue;
        }

        let is_hot_called = data.call_count >= 5 || data.loop_invocations >= 2;
        let is_repeated_impl = def_count >= 2 && (data.max_body_lines >= 6 || data.max_ccn >= 2);
        let is_heavy_reused = data.call_count >= 3 && data.max_ccn >= 4 && !data.workload_tags.is_empty();

        if !(is_hot_called || is_repeated_impl || is_heavy_reused) {
            continue;
        }

        let workload_kind = if data.workload_tags.is_empty() {
            if def_count >= 2 {
                "Helper Logika Berulang Lintas Kelas".to_string()
            } else {
                "Utilitas Komputasi Berfrekuensi Tinggi".to_string()
            }
        } else {
            data.workload_tags.iter().copied().collect::<Vec<_>>().join(" + ")
        };

        let score = (data.call_count as u32 * 3)
            + (data.loop_invocations as u32 * 8)
            + (def_count.saturating_sub(1) as u32 * 10)
            + (caller_files_count as u32 * 4)
            + (data.max_ccn * 2)
            + if data.has_internal_loop { 10 } else { 0 };

        let estimated_cpu_saving = if data.loop_invocations >= 2 || data.has_internal_loop {
            "50% - 75% siklus CPU (Hot Loop Native)".to_string()
        } else if !data.workload_tags.is_empty() {
            "40% - 65% siklus CPU & alokasi memori".to_string()
        } else {
            "30% - 50% overhead Zend VM opcode".to_string()
        };

        let (first_file, first_line) = &data.definitions[0];
        let defined_in = if def_count > 1 {
            format!("{}:{} (+{} duplikasi di file lain)", first_file, first_line, def_count - 1)
        } else {
            format!("{}:{}", first_file, first_line)
        };

        let reason = if def_count > 1 && data.call_count > 0 {
            format!(
                "Fungsi `{}` didefinisikan ulang {}x dan dipanggil {}x di {} file",
                name, def_count, data.call_count, caller_files_count.max(1)
            )
        } else if def_count > 1 {
            format!(
                "Fungsi `{}` diimplementasikan berulang di {} kelas/berkas berbeda (CCN maks {})",
                name, def_count, data.max_ccn
            )
        } else if data.loop_invocations > 0 {
            format!(
                "Fungsi `{}` dipanggil {}x ({}x di dalam loop) lintas {} file",
                name, data.call_count, data.loop_invocations, caller_files_count.max(1)
            )
        } else {
            format!(
                "Fungsi `{}` dipanggil berulang {}x lintas {} file (CCN {})",
                name, data.call_count, caller_files_count.max(1), data.max_ccn
            )
        };

        let recommendation = format!(
            "{}. Konsolidasikan dan kompilasi ke Native PHP Extension berbasis Rust (`ext-php-rs`), C (Zend API), atau Zephir untuk mengeliminasi interpretasi berulang di Zend Engine.",
            reason
        );

        candidates.push(PhpExtensionCandidate {
            function_name: name,
            defined_in,
            definition_count: def_count,
            call_count: data.call_count,
            caller_files: caller_files_count,
            loop_invocations: data.loop_invocations,
            max_ccn: data.max_ccn,
            workload_kind,
            estimated_cpu_saving,
            offload_score: score,
            recommendation,
        });
    }

    candidates.sort_by(|a, b| b.offload_score.cmp(&a.offload_score));
    candidates.truncate(15);
    candidates
}
