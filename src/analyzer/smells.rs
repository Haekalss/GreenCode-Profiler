//! Code smell models and detector functions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSmell {
    pub smell_type: String,
    pub severity: String,
    pub message: String,
    pub file: String,
    pub line: usize,
}

pub fn make_deep_nesting(file: &str, line: usize, cf_depth: usize, strict: bool) -> CodeSmell {
    let severity = if strict || cf_depth >= 5 {
        "High"
    } else {
        "Medium"
    };
    CodeSmell {
        smell_type: "Deep Nesting".to_string(),
        severity: severity.to_string(),
        message: format!(
            "Alur kontrol bersarang terlalu dalam ({} level percabangan/perulangan di dalam fungsi)",
            cf_depth
        ),
        file: file.to_string(),
        line,
    }
}

pub fn make_fat_controller(file: &str, ncloc: usize, max_ncloc: usize, strict: bool) -> CodeSmell {
    let severity = if strict || ncloc > max_ncloc + 150 {
        "High"
    } else {
        "Medium"
    };
    CodeSmell {
        smell_type: "Fat Controller / Large Class".to_string(),
        severity: severity.to_string(),
        message: format!(
            "Ukuran berkas melebihi batas modular ({} baris NCLOC, batas wajar <= {})",
            ncloc, max_ncloc
        ),
        file: file.to_string(),
        line: 1,
    }
}
