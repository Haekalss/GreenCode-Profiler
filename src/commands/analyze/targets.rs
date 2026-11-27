//! Target folder resolution and filtering validation for `greenprofiler analyze`.

use super::super::config;
use crate::cli::{apply_filters, print_filter_keywords, CliFlags};
use serde_json::Value;
use std::path::Path;

pub fn resolve_targets(flags: &CliFlags) -> Option<Vec<String>> {
    let config_path = flags
        .positional
        .first()
        .cloned()
        .unwrap_or_else(|| "greenprofiler.json".to_string());
    let mut targets = Vec::new();

    if Path::new(&config_path).is_file() {
        let cfg = config::read(&config_path);
        if !endpoint_filter_matches(&cfg, flags, &config_path) {
            return None;
        }
        targets = config::targets(&cfg);
    }
    if targets.is_empty() {
        targets = fallback_target(flags);
    }
    Some(targets)
}

fn endpoint_filter_matches(config_value: &Value, flags: &CliFlags, path: &str) -> bool {
    if !flags.has_filter() {
        return true;
    }
    let all = config::endpoints(config_value);
    let total = all.len();
    let matched = apply_filters(all, &flags.filters, &flags.excludes);
    println!(
        "[GreenProfiler] Filter aktif: {} endpoint dipilih dari {} endpoint di {}.",
        matched.len(),
        total,
        path
    );
    if matched.is_empty() {
        println!("[GreenProfiler Error] 0 endpoint cocok dengan --filter / --exclude.");
        println!("            Kata kunci yang dipakai:");
        print_filter_keywords(&flags.filters, &flags.excludes);
        println!("            Jalankan 'greenprofiler routes' untuk melihat daftar endpoint.");
        return false;
    }
    true
}

fn fallback_target(flags: &CliFlags) -> Vec<String> {
    if let Some(first) = flags.positional.first() {
        if Path::new(first).is_dir() {
            return vec![first.clone()];
        }
        if let Some(second) = flags.positional.get(1) {
            return vec![second.clone()];
        }
    }
    vec![".".to_string()]
}
