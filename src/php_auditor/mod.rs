//! PHP Runtime & Green Extensions Auditor in Pure Rust
//! Memeriksa ketersediaan OPcache, JIT, status Xdebug, dan ekstensi modern.

pub mod detector;
pub mod evaluator;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionStatus {
    pub name: String,
    pub is_loaded: bool,
    pub category: String,
    pub description: String,
    pub green_impact: String,
    pub status_badge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpAuditReport {
    pub is_php_available: bool,
    pub php_version: String,
    pub loaded_extensions_count: usize,
    pub opcache_enabled: bool,
    pub jit_enabled: bool,
    pub xdebug_active: bool,
    pub recommendations: Vec<String>,
    pub extension_details: Vec<ExtensionStatus>,
}

pub struct PhpAuditor;

impl PhpAuditor {
    pub fn audit() -> PhpAuditReport {
        let (available, version_line) = detector::query_version();
        if !available {
            return unavailable_report();
        }

        let loaded_modules = detector::query_modules();
        let has_ext = |name: &str| loaded_modules.iter().any(|m| m == name);

        let opcache = has_ext("zend opcache") || has_ext("opcache");
        let xdebug = has_ext("xdebug");
        let jit = detector::detect_jit_enabled();

        let mut details = Vec::new();
        let mut recs = Vec::new();

        evaluator::evaluate_accelerators(&has_ext, opcache, &mut details, &mut recs);
        evaluator::evaluate_profilers(xdebug, &mut details, &mut recs);
        evaluator::evaluate_essentials(&has_ext, &mut details, &mut recs);

        PhpAuditReport {
            is_php_available: true,
            php_version: version_line,
            loaded_extensions_count: loaded_modules.len(),
            opcache_enabled: opcache,
            jit_enabled: jit,
            xdebug_active: xdebug,
            recommendations: recs,
            extension_details: details,
        }
    }
}

fn unavailable_report() -> PhpAuditReport {
    PhpAuditReport {
        is_php_available: false,
        php_version: "PHP binary not found".to_string(),
        loaded_extensions_count: 0,
        opcache_enabled: false,
        jit_enabled: false,
        xdebug_active: false,
        recommendations: vec!["Install PHP atau tambahkan php ke PATH sistem.".to_string()],
        extension_details: Vec::new(),
    }
}
