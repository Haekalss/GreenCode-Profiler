//! Python route extraction (Flask, FastAPI, Django).

use super::super::files::collect_source_files;
use super::push_route;
use crate::detector::RouteEndpoint;
use regex::Regex;
use std::fs;
use std::path::Path;

const EXTENSIONS: [&str; 1] = ["py"];
const IGNORED_DIRS: [&str; 12] = [
    "venv",
    ".venv",
    "env",
    "site-packages",
    "__pycache__",
    ".git",
    "node_modules",
    "migrations",
    ".tox",
    "build",
    "dist",
    ".eggs",
];

struct PythonRegexes {
    flask: Regex,
    fastapi: Regex,
    django: Regex,
    methods: Regex,
    word: Regex,
}

impl PythonRegexes {
    fn compile() -> Self {
        Self {
            flask: Regex::new(r#"@(\w+)\.route\s*\(\s*['"]([^'"]*)['"]"#).unwrap(),
            fastapi: Regex::new(
                r#"@(?:\w+)\.(get|post|put|patch|delete|head|api_route)\s*\(\s*['"]([^'"]*)['"]"#,
            )
            .unwrap(),
            django: Regex::new(r#"^\s*path\s*\(\s*['"]([^'"]*)['"]"#).unwrap(),
            methods: Regex::new(r#"methods\s*=\s*\[([^\]]*)\]"#).unwrap(),
            word: Regex::new(r#"['"]([A-Za-z]+)['"]"#).unwrap(),
        }
    }
}

pub(super) fn discover_python(dir: &Path) -> Vec<RouteEndpoint> {
    let files = collect_source_files(dir, &EXTENSIONS, &IGNORED_DIRS);
    let regexes = PythonRegexes::compile();
    let mut routes = Vec::new();
    for file in &files {
        let Ok(content) = fs::read_to_string(file) else {
            continue;
        };
        for line in content.lines() {
            if !line.trim_start().starts_with('#') {
                extract_line(line, &regexes, &mut routes);
            }
        }
    }
    routes
}

fn extract_line(line: &str, regexes: &PythonRegexes, routes: &mut Vec<RouteEndpoint>) {
    if let Some(captures) = regexes.flask.captures(line) {
        for method in methods_from(line, regexes, &["GET"]) {
            push_route(routes, method, "", &captures[2]);
        }
        return;
    }
    if let Some(captures) = regexes.fastapi.captures(line) {
        let method = captures[1].to_uppercase();
        let methods = if method == "API_ROUTE" {
            methods_from(line, regexes, &["GET"])
        } else {
            vec![method]
        };
        for method in methods {
            push_route(routes, method, "", &captures[2]);
        }
        return;
    }
    if let Some(captures) = regexes.django.captures(line) {
        push_route(routes, "GET".to_string(), "", &captures[1]);
    }
}

/// Baca `methods=[...]`, atau pakai `defaults` bila tidak ada.
fn methods_from(line: &str, regexes: &PythonRegexes, defaults: &[&str]) -> Vec<String> {
    let Some(captures) = regexes.methods.captures(line) else {
        return defaults.iter().map(|method| method.to_string()).collect();
    };
    let methods: Vec<String> = regexes
        .word
        .captures_iter(&captures[1])
        .map(|word| word[1].to_uppercase())
        .collect();
    if methods.is_empty() {
        return defaults.iter().map(|method| method.to_string()).collect();
    }
    methods
}
