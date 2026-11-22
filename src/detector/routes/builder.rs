//! Shared helpers for building, deduplicating, and formatting discovered routes.

use super::{RouteEndpoint, has_param};

pub fn tagged(route: RouteEndpoint, tag: &str, service: &str, base_url: &str) -> RouteEndpoint {
    let url = format!("{}{}", base_url, route.path);
    RouteEndpoint {
        path: route.path,
        name: format!("[{}] {}", tag, route.name),
        method: route.method,
        service: Some(service.to_string()),
        base_url: Some(base_url.to_string()),
        url: Some(url),
    }
}

pub fn dedup(mut routes: Vec<RouteEndpoint>) -> Vec<RouteEndpoint> {
    let mut seen = std::collections::HashSet::new();
    routes.retain(|route| seen.insert((route.method.clone(), route.path.clone())));
    routes
}

pub fn apply_limit(routes: Vec<RouteEndpoint>, max_routes: usize) -> Vec<RouteEndpoint> {
    if max_routes == 0 {
        return routes;
    }
    let (mut frontend, mut backend): (Vec<_>, Vec<_>) = routes
        .into_iter()
        .partition(|route| route.service.as_deref() == Some("frontend"));
    let limit_frontend = (max_routes + 1) / 2;
    frontend.truncate(limit_frontend);
    backend.truncate(max_routes.saturating_sub(frontend.len()));
    frontend.extend(backend);
    frontend
}

pub fn filter_profilable(routes: Vec<RouteEndpoint>) -> Vec<RouteEndpoint> {
    routes
        .into_iter()
        .filter(|route| route.method == "GET" && !has_param(&route.path))
        .collect()
}
