//! Path manipulation, parameter checking, and route registration helpers.

use super::RouteEndpoint;

pub fn push_route(
    routes: &mut Vec<RouteEndpoint>,
    method: String,
    active_prefix: &str,
    raw: &str,
) {
    let path = join_path(active_prefix, raw);
    let name = display_name(&path);
    if name.is_empty() || contains_route(routes, &method, &path) {
        return;
    }
    routes.push(RouteEndpoint {
        path,
        name: format!("API {}", name),
        method,
        service: None,
        base_url: None,
        url: None,
    });
}

pub fn join_path(prefix: &str, raw: &str) -> String {
    let prefix = prefix.trim_matches('/');
    let raw = raw.trim_start_matches('/');
    let mut path = String::from("/");
    if !prefix.is_empty() {
        path.push_str(prefix);
    }
    if !raw.is_empty() {
        if !path.ends_with('/') {
            path.push('/');
        }
        path.push_str(raw);
    }
    while path.contains("//") {
        path = path.replace("//", "/");
    }
    path
}

pub fn display_name(path: &str) -> String {
    path.trim_start_matches("/api/")
        .trim_start_matches('/')
        .replace('/', " ")
        .replace('-', " ")
        .trim()
        .to_string()
}

pub fn contains_route(routes: &[RouteEndpoint], method: &str, path: &str) -> bool {
    routes
        .iter()
        .any(|route| route.method == method && route.path == path)
}

pub fn has_param(path: &str) -> bool {
    const MARKERS: [char; 4] = ['{', '[', '<', '*'];
    if path.chars().any(|c| MARKERS.contains(&c)) {
        return true;
    }
    path.split('/').any(|segment| segment.starts_with(':'))
}
