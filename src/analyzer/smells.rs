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

pub fn make_deep_nesting(file: &str, line: usize, indent_level: usize) -> CodeSmell {
    CodeSmell {
        smell_type: "Deep Nesting".to_string(),
        severity: "Medium".to_string(),
        message: format!("Logika percabangan bersarang dalam (level {})", indent_level),
        file: file.to_string(),
        line,
    }
}

pub fn make_fat_controller(file: &str, ncloc: usize) -> CodeSmell {
    CodeSmell {
        smell_type: "Fat Controller / Large Class".to_string(),
        severity: "High".to_string(),
        message: format!(
            "Ukuran kelas terlalu besar ({} baris NCLOC, ambang batas normal <= 300)",
            ncloc
        ),
        file: file.to_string(),
        line: 1,
    }
}
