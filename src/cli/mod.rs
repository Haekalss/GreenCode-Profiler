//! Shared command-line parsing, filtering, and console helpers.

pub mod filter;
pub mod flags;
pub mod help;

pub use filter::apply_filters;
pub use flags::{parse_flags, CliFlags};
pub use help::{
    get_simple_timestamp, print_banner, print_filter_keywords, print_help, print_no_route_hints,
};

use std::path::PathBuf;

/// Tentukan path laporan HTML & JSON.
pub fn resolve_report_paths(
    output: &Option<String>,
    default_html: &str,
    default_json: &str,
) -> (String, String) {
    let Some(raw) = output else {
        return (default_html.to_string(), default_json.to_string());
    };
    let given = PathBuf::from(raw);
    let (html, json) = match given.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("json") => (given.with_extension("html"), given),
        Some(_) => (given.clone(), given.with_extension("json")),
        None => (given.with_extension("html"), given.with_extension("json")),
    };
    (
        html.to_string_lossy().to_string(),
        json.to_string_lossy().to_string(),
    )
}
