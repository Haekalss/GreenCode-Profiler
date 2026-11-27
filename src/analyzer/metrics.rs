//! Metrics calculation: Cyclomatic Complexity (CCN), thresholds, & Maintainability Index (MI).

use regex::Regex;
use std::sync::OnceLock;

pub fn branch_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(if|else\s*if|elif|for|foreach|while|case|catch|except|match)\b|&&|\|\||\band\b|\bor\b|\?\s*[^:]+:|\?\?"
        )
        .unwrap()
    })
}

/// Regex khusus kontrol alur struktural (bukan operator boolean `&&`/`||` atau ekspresi JSX).
pub fn control_flow_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^\s*(?:\}\s*)?(?:else\s+if|if|elif|else|for|foreach|while|switch|match|try|catch|except)\b",
        )
        .unwrap()
    })
}

pub fn loop_control_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*(?:for|foreach|while)\b").unwrap())
}

pub fn arrow_callback_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"=>\s*\{").unwrap())
}

pub fn function_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?:pub\s+(?:\([^\)]+\)\s+)?)?(?:async\s+)?(?:unsafe\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(|(?:public|protected|private|static|\s)*\bfunction\s+([a-zA-Z0-9_]+)\s*\(|(?:export\s+(?:default\s+)?)?(?:async\s+)?function\s+([a-zA-Z0-9_]+)\s*\(|(?:export\s+)?const\s+([a-zA-Z0-9_]+)\s*=\s*(?:async\s*)?\([^)]*\)\s*(?::\s*[^=]+)?=>"
        )
        .unwrap()
    })
}

/// Ambang batas NCLOC adaptif:
/// - Mode default (harian + riset): komponen UI deklaratif (.tsx/.jsx/.vue/.svelte) <= 450 NCLOC,
///   sedangkan backend/logic (.php/.py/.ts/.js/.rs/.go) <= 300 NCLOC.
/// - Mode `--strict`: seragam <= 300 NCLOC untuk seluruh jenis berkas.
pub fn max_ncloc_for_extension(ext: &str, strict: bool) -> usize {
    if strict {
        return 300;
    }
    match ext {
        "tsx" | "jsx" | "vue" | "svelte" => 450,
        _ => 300,
    }
}

/// Ambang batas kedalaman control-flow bersarang di dalam satu fungsi.
pub fn max_control_flow_nesting(strict: bool) -> usize {
    if strict {
        3
    } else {
        4
    }
}

/// Menghitung Maintainability Index (MI) terkalibrasi (skala 0 - 100).
pub fn calculate_maintainability_index(ncloc: usize, ccn: u32) -> f64 {
    if ncloc == 0 {
        return 100.0;
    }
    let ncloc_f = ncloc as f64;
    let ccn_f = ccn as f64;
    let volume = (ncloc_f * 4.5).max(1.0);

    let raw_mi = 171.0 - 5.2 * volume.ln() - 0.23 * ccn_f - 16.2 * ncloc_f.ln();

    let calibrated = if raw_mi > 0.0 {
        let sei_mi = raw_mi * (100.0 / 171.0);
        (sei_mi * 2.15 - 5.0).clamp(15.0, 100.0)
    } else {
        10.0
    };

    (calibrated * 10.0).round() / 10.0
}
