//! Node.js route extraction (Express/Fastify/Koa/NestJS/Next API).

use super::super::files::collect_source_files;
use super::push_route;
use crate::detector::RouteEndpoint;
use regex::Regex;
use std::fs;
use std::path::Path;

const EXTENSIONS: [&str; 8] = ["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"];
const IGNORED_DIRS: [&str; 8] = [
    "node_modules",
    ".next",
    "dist",
    "build",
    "coverage",
    "vendor",
    ".git",
    "out",
];
const ROUTE_FILES: [&str; 4] = ["route.ts", "route.js", "route.mts", "route.cts"];

const NODE_METHOD_PATTERN: &str = r#"(?i)\b(?:app|router|server|api|fastify|koa|r|http|web|group|routes|appRouter|apiRouter|mainRouter)\s*\.\s*(get|post|put|patch|delete|head|options|all)\s*\(\s*['"`]([^'"`\?]+)['"`]"#;

struct NodeRegexes {
    method: Regex,
    nest: Regex,
    nest_empty: Regex,
    controller: Regex,
}

impl NodeRegexes {
    fn compile() -> Self {
        Self {
            method: Regex::new(NODE_METHOD_PATTERN).unwrap(),
            nest: Regex::new(
                r#"@(Get|Post|Put|Patch|Delete|Head|Options|All)\s*\(\s*['"`]([^'"`]*)['"`]"#,
            )
            .unwrap(),
            nest_empty: Regex::new(
                r#"@(Get|Post|Put|Patch|Delete|Head|Options|All)\s*\(\s*\)"#,
            )
            .unwrap(),
            controller: Regex::new(r#"@Controller\s*\(\s*['"`]([^'"`]*)['"`]"#).unwrap(),
        }
    }
}

pub(super) fn discover_node(dir: &Path) -> Vec<RouteEndpoint> {
    let files = collect_source_files(dir, &EXTENSIONS, &IGNORED_DIRS);
    let regexes = NodeRegexes::compile();
    let mut routes = Vec::new();
    for file in &files {
        let Ok(content) = fs::read_to_string(file) else {
            continue;
        };
        parse_node_file(&content, file, dir, &regexes, &mut routes);
    }
    routes
}

fn parse_node_file(
    content: &str,
    file: &Path,
    root: &Path,
    regexes: &NodeRegexes,
    routes: &mut Vec<RouteEndpoint>,
) {
    let mut controller = String::new();
    for line in content.lines() {
        if let Some(captures) = regexes.controller.captures(line) {
            controller = captures[1].trim_matches('/').to_string();
        }
        extract_express_routes(line, &regexes.method, routes);
        extract_nest_routes(line, &regexes.nest, &controller, routes);
        if let Some(captures) = regexes.nest_empty.captures(line) {
            push_route(routes, captures[1].to_uppercase(), &controller, "");
        }
    }
    if let Some(path) = next_route_path(file, root) {
        push_route(routes, "GET".to_string(), "", &path);
    }
}

fn extract_express_routes(line: &str, method_re: &Regex, routes: &mut Vec<RouteEndpoint>) {
    for captures in method_re.captures_iter(line) {
        if captures[2].starts_with('/') {
            push_route(routes, captures[1].to_uppercase(), "", &captures[2]);
        }
    }
}

fn extract_nest_routes(
    line: &str,
    nest_re: &Regex,
    controller: &str,
    routes: &mut Vec<RouteEndpoint>,
) {
    for captures in nest_re.captures_iter(line) {
        push_route(
            routes,
            captures[1].to_uppercase(),
            controller,
            &captures[2],
        );
    }
}

/// `src/app/api/x/route.ts` -> `/api/x`; `pages/api/x.js` -> `/api/x`.
fn next_route_path(file: &Path, root: &Path) -> Option<String> {
    let name = file
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if !ROUTE_FILES.contains(&name.as_str()) {
        return None;
    }
    let relative = file.parent()?.strip_prefix(root).ok()?;
    let segments: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect();
    let start = segments
        .iter()
        .position(|segment| segment == "app" || segment == "pages")?;
    let rest: Vec<&str> = segments[start + 1..]
        .iter()
        .map(String::as_str)
        .filter(|segment| !segment.starts_with('('))
        .collect();
    if rest.is_empty() {
        return None;
    }
    Some(format!("/{}", rest.join("/")))
}
