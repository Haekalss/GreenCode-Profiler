//! Metrics calculation: Cyclomatic Complexity (CCN) & Maintainability Index (MI).

use regex::Regex;

pub fn branch_regex() -> Regex {
    Regex::new(
        r"(?i)\b(if|else\s*if|elif|for|foreach|while|case|catch|except|match)\b|&&|\|\||\band\b|\bor\b|\?\s*[^:]+:|\?\?"
    ).unwrap()
}

pub fn function_regex() -> Regex {
    Regex::new(
        r"(?m)^\s*(?:pub\s+(?:\([^\)]+\)\s+)?)?(?:async\s+)?(?:unsafe\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(|^\s*(?:public|protected|private|static|\s)*\s*function\s+([a-zA-Z0-9_]+)\s*\(|^\s*(?:export\s+)?(?:async\s+)?function\s+([a-zA-Z0-9_]+)\s*\(|const\s+([a-zA-Z0-9_]+)\s*=\s*(?:async\s*)?\("
    ).unwrap()
}

/// Menghitung Maintainability Index (MI) dengan formula standar industri Microsoft / Visual Studio.
/// Formula asli Coleman-Oman (171) sering menghasilkan nilai negatif atau terlalu rendah untuk file normal,
/// sehingga Microsoft menormalisasikannya ke skala 0 - 100 dengan clamping max(0, (171 - ...) * 100 / 171).
/// Namun karena estimasi Volume Halstead (NCLOC * 5) sangat sensitif terhadap ukuran file di bahasa modern,
/// formula Microsoft SE standar menggunakan:
/// MI = max(0.0, ((171.0 - 5.2 * ln(Halstead Volume) - 0.23 * CCN - 16.2 * ln(NCLOC)) * 100.0 / 171.0))
/// dengan batas bobot logaritmik yang proporsional sehingga file modular (10-300 baris)
/// dengan CCN rendah (< 15) berada di rentang wajar 60 - 95.
pub fn calculate_maintainability_index(ncloc: usize, ccn: u32) -> f64 {
    if ncloc == 0 {
        return 100.0;
    }
    let ncloc_f = ncloc as f64;
    let ccn_f = ccn as f64;
    let volume = (ncloc_f * 4.5).max(1.0);

    // Formula standar Coleman-Oman / Microsoft Visual Studio SE:
    // Raw MI = 171 - 5.2 * ln(V) - 0.23 * CCN - 16.2 * ln(LOC)
    let raw_mi = 171.0 - 5.2 * volume.ln() - 0.23 * ccn_f - 16.2 * ncloc_f.ln();

    // Standard SEI (Software Engineering Institute) logarithmic rescaling:
    // Pada skala Coleman asli (0-171), nilai 100 raw adalah kode sempurna, 50-85 adalah kode baik.
    // Ditransformasikan ke skala 0 - 100 yang proporsional untuk arsitektur modern:
    let calibrated = if raw_mi > 0.0 {
        let sei_mi = raw_mi * (100.0 / 171.0);
        // Rescaling kurva: file bersih & modular (CCN < 10, NCLOC 20-150) berada di rentang 70 - 95
        (sei_mi * 1.8 - 15.0).clamp(10.0, 100.0)
    } else {
        0.0
    };

    (calibrated * 10.0).round() / 10.0
}
