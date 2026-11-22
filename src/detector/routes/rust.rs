//! Rust route extraction (axum, actix-web, rocket).

use super::super::files::collect_source_files;
use super::push_route;
use crate::detector::RouteEndpoint;
use regex::Regex;
use std::fs;
use std::path::Path;

const RUST_ROUTE_PATTERN: &str = r#"(?i)\.route\s*\(\s*"([^"]*)"\s*,\s*(?:\w+::)?(get|post|put|delete|patch|head)\s*\("#;

struct RustRegexes {
    route: Regex,
    resource: Regex,
    method_macro: Regex,
    route_macro: Regex,
}

impl RustRegexes {
    fn compile() -> Self {
        Self {
            route: Regex::new(RUST_ROUTE_PATTERN).unwrap(),
            resource: Regex::new(r#"web::resource\s*\(\s*"([^"]*)""#).unwrap(),
            method_macro: Regex::new(r#"(?i)#\[\s*(get|post|put|delete|patch|head)\s*\(\s*"([^"]*)""#)
                .unwrap(),
            route_macro: Regex::new(
                r#"(?i)#\[\s*route\s*\(\s*"([^"]*)"\s*,\s*method\s*=\s*"([^"]*)""#,
            )
            .unwrap(),
        }
    }
}

pub(super) fn discover_rust(dir: &Path) -> Vec<RouteEndpoint> {
    let files = collect_source_files(dir, &["rs"], &["target", ".git", "node_modules"]);
    let regexes = RustRegexes::compile();
    let mut routes = Vec::new();
    for file in &files {
        let Ok(content) = fs::read_to_string(file) else {
            continue;
        };
        for line in content.lines() {
            if !line.trim_start().starts_with("//") {
                extract_line(line, &regexes, &mut routes);
            }
        }
    }
    routes
}

fn extract_line(line: &str, regexes: &RustRegexes, routes: &mut Vec<RouteEndpoint>) {
    for captures in regexes.route.captures_iter(line) {
        push_route(routes, captures[2].to_uppercase(), "", &captures[1]);
    }
    for captures in regexes.resource.captures_iter(line) {
        push_route(routes, "GET".to_string(), "", &captures[1]);
    }
    for captures in regexes.method_macro.captures_iter(line) {
        push_route(routes, captures[1].to_uppercase(), "", &captures[2]);
    }
    for captures in regexes.route_macro.captures_iter(line) {
        push_route(routes, captures[2].to_uppercase(), "", &captures[1]);
    }
}
