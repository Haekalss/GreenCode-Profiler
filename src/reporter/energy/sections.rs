//! Section formatters for the energy & performance profiling report.

use super::super::render::*;
use serde_json::Value;

pub fn format_endpoint_rows(endpoints: &Value) -> String {
    endpoints
        .as_array()
        .map(|arr| {
            arr.iter()
                .enumerate()
                .map(|(i, ep)| endpoint_row(i, ep))
                .collect()
        })
        .unwrap_or_default()
}

pub fn format_module_rows(modules: &Value) -> String {
    modules
        .as_array()
        .map(|arr| arr.iter().map(correlated_row).collect())
        .unwrap_or_default()
}

pub fn format_file_rows(files: &Value) -> String {
    files
        .as_array()
        .map(|arr| arr.iter().map(file_row).collect())
        .unwrap_or_default()
}

pub fn format_smell_rows(smells: &Value) -> String {
    smells
        .as_array()
        .map(|arr| arr.iter().map(smell_row).collect())
        .unwrap_or_default()
}

pub fn format_php_section(php: &Value) -> (String, String) {
    if php.is_null() || !php["is_php_available"].as_bool().unwrap_or(false) {
        return (
            "<p>Audit PHP tidak tersedia atau runtime bukan PHP.</p>".to_string(),
            String::new(),
        );
    }

    let summary = format!(
        "<p>PHP Version: <strong>{}</strong> | OPcache: <strong>{}</strong> | JIT: <strong>{}</strong> | Xdebug: <strong>{}</strong></p>",
        text(php, "php_version"),
        if php["opcache_enabled"].as_bool().unwrap_or(false) { "Aktif [OK]" } else { "Nonaktif [Boros Energi]" },
        if php["jit_enabled"].as_bool().unwrap_or(false) { "Aktif [OK]" } else { "Nonaktif" },
        if php["xdebug_active"].as_bool().unwrap_or(false) { "Aktif [Overhead Tinggi]" } else { "Nonaktif [OK]" }
    );

    let rows = php["extension_details"]
        .as_array()
        .map(|arr| arr.iter().map(php_extension_row).collect())
        .unwrap_or_default();

    (summary, rows)
}
