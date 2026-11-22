//! Laravel route extraction from `routes/api.php` and `routes/web.php`.

use super::push_route;
use crate::detector::RouteEndpoint;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

const RESOURCE_ACTIONS: [(&str, &str); 6] = [
    ("GET", ""),
    ("POST", ""),
    ("GET", "/{id}"),
    ("PUT", "/{id}"),
    ("PATCH", "/{id}"),
    ("DELETE", "/{id}"),
];

struct PrefixState {
    stack: Vec<(usize, String)>,
    pending: Option<String>,
    depth: usize,
}

impl PrefixState {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            pending: None,
            depth: 0,
        }
    }

    fn active(&self, base: &str) -> String {
        self.stack
            .iter()
            .fold(base.to_string(), |mut path, (_, prefix)| {
                path.push('/');
                path.push_str(prefix);
                path
            })
    }

    fn register(&mut self, line: &str, prefix_re: &Regex) {
        if let Some(captures) = prefix_re.captures(line) {
            self.pending = Some(captures[1].trim_matches('/').to_string());
        }
    }

    fn update_depth(&mut self, line: &str) {
        let opens = line.matches('{').count();
        if opens > 0 {
            self.depth += opens;
            self.check_open_group(line);
        }
        self.depth = self.depth.saturating_sub(line.matches('}').count());
        while let Some((depth, _)) = self.stack.last() {
            if self.depth >= *depth {
                break;
            }
            self.stack.pop();
        }
    }

    fn check_open_group(&mut self, line: &str) {
        let opens_group = line.contains("group(") || line.contains("function");
        if opens_group {
            if let Some(prefix) = self.pending.take() {
                self.stack.push((self.depth, prefix));
            }
        }
    }
}

pub(super) fn discover_laravel(dir: &Path) -> Vec<RouteEndpoint> {
    let method_re =
        Regex::new(r#"Route::(get|post|put|delete|patch)\s*\(\s*['"]([^'"]*)['"]"#).unwrap();
    let resource_re =
        Regex::new(r#"Route::(?:apiResource|resource)\s*\(\s*['"]([^'"]*)['"]"#).unwrap();
    let prefix_re = Regex::new(r#"prefix\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
    let multiline_re =
        Regex::new(r"Route::(get|post|put|delete|patch|apiResource|resource)\s*\(\s+").unwrap();

    let mut routes = Vec::new();
    for (file, base_prefix) in route_files(dir) {
        let Ok(raw) = fs::read_to_string(&file) else {
            continue;
        };
        let content = multiline_re.replace_all(&raw, "Route::$1(");
        parse_laravel_file(
            &content,
            base_prefix,
            &LaravelRegexes {
                method: &method_re,
                resource: &resource_re,
                prefix: &prefix_re,
            },
            &mut routes,
        );
    }
    dedup(routes)
}

struct LaravelRegexes<'a> {
    method: &'a Regex,
    resource: &'a Regex,
    prefix: &'a Regex,
}

fn route_files(dir: &Path) -> [(PathBuf, &'static str); 2] {
    [
        (dir.join("routes").join("api.php"), "/api"),
        (dir.join("routes").join("web.php"), ""),
    ]
}

fn parse_laravel_file(
    content: &str,
    base_prefix: &str,
    regexes: &LaravelRegexes,
    routes: &mut Vec<RouteEndpoint>,
) {
    let mut state = PrefixState::new();
    for line in content.lines() {
        state.register(line, regexes.prefix);
        let active = state.active(base_prefix);
        extract_method_routes(line, regexes.method, &active, routes);
        extract_resource_routes(line, regexes.resource, &active, routes);
        state.update_depth(line);
    }
}

fn extract_method_routes(
    line: &str,
    method_re: &Regex,
    active_prefix: &str,
    routes: &mut Vec<RouteEndpoint>,
) {
    for captures in method_re.captures_iter(line) {
        push_route(
            routes,
            captures[1].to_uppercase(),
            active_prefix,
            &captures[2],
        );
    }
}

fn extract_resource_routes(
    line: &str,
    resource_re: &Regex,
    active_prefix: &str,
    routes: &mut Vec<RouteEndpoint>,
) {
    for captures in resource_re.captures_iter(line) {
        let resource = captures[1].trim_start_matches('/');
        for (method, suffix) in RESOURCE_ACTIONS {
            push_route(
                routes,
                method.to_string(),
                active_prefix,
                &format!("{}{}", resource, suffix),
            );
        }
    }
}

fn dedup(mut routes: Vec<RouteEndpoint>) -> Vec<RouteEndpoint> {
    let mut seen = std::collections::HashSet::new();
    routes.retain(|route| seen.insert((route.method.clone(), route.path.clone())));
    routes
}
