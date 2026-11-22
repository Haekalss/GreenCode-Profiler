//! Go route extraction (net/http, gin, chi, gorilla).

use super::super::files::collect_source_files;
use super::push_route;
use crate::detector::RouteEndpoint;
use regex::Regex;
use std::fs;
use std::path::Path;

const STD_METHODS: [&str; 7] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

pub(super) fn discover_go(dir: &Path) -> Vec<RouteEndpoint> {
    let files = collect_source_files(dir, &["go"], &["vendor", ".git", "testdata", "node_modules"]);
    let call_re = Regex::new(
        r#"(?i)\.(get|post|put|patch|delete|head|handlefunc|handle)\s*\(\s*"([^"]*)""#,
    )
    .unwrap();

    let mut routes = Vec::new();
    for file in &files {
        let Ok(content) = fs::read_to_string(file) else {
            continue;
        };
        extract_routes_from_go_source(&content, &call_re, &mut routes);
    }
    routes
}

fn extract_routes_from_go_source(
    content: &str,
    call_re: &Regex,
    routes: &mut Vec<RouteEndpoint>,
) {
    for line in content.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        for captures in call_re.captures_iter(line) {
            if let Some((method, path)) = parse_call(&captures[1], &captures[2]) {
                push_route(routes, method, "", &path);
            }
        }
    }
}

fn parse_call(method: &str, raw: &str) -> Option<(String, String)> {
    let (mut method, mut path) = split_std_method(method.to_uppercase(), raw.to_string());
    if method == "HANDLE" || method == "HANDLEFUNC" {
        method = "GET".to_string();
    }
    if !path.starts_with('/') {
        return None;
    }
    path = path.to_string();
    Some((method, path))
}

/// Pola stdlib Go 1.22+: `http.HandleFunc("GET /users", ...)`.
fn split_std_method(method: String, raw: String) -> (String, String) {
    let Some((prefix, rest)) = raw.split_once(' ') else {
        return (method, raw);
    };
    if !STD_METHODS.contains(&prefix) {
        return (method, raw);
    }
    (prefix.to_string(), rest.to_string())
}
