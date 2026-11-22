//! Next.js App Router page discovery.

use super::super::files::collect_files_recursive;
use crate::detector::RouteEndpoint;
use std::path::Path;

pub(super) fn discover_nextjs(dir: &Path) -> Vec<RouteEndpoint> {
    let app_dir = dir.join("src").join("app");
    let target = if app_dir.exists() {
        app_dir
    } else {
        dir.join("app")
    };
    if !target.exists() {
        return Vec::new();
    }
    let mut files = Vec::new();
    collect_files_recursive(&target, &mut files);

    let mut routes: Vec<RouteEndpoint> = Vec::new();
    for path in files {
        if let Some(route) = page_route(&path, &target) {
            if routes.iter().all(|existing| existing.path != route.path) {
                routes.push(route);
            }
        }
    }
    routes
}

fn page_route(path: &Path, target: &Path) -> Option<RouteEndpoint> {
    if !is_page_file(path) {
        return None;
    }
    let relative = path.strip_prefix(target).unwrap_or(path);
    let parent = relative
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let route_path = clean_path(&parent);
    let name = if route_path == "/" {
        "Halaman Utama".to_string()
    } else {
        format!("Halaman {}", page_name(&route_path))
    };
    Some(RouteEndpoint {
        path: route_path,
        name,
        method: "GET".to_string(),
        service: None,
        base_url: None,
        url: None,
    })
}

fn is_page_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    const EXTENSIONS: [&str; 4] = [".tsx", ".jsx", ".js", ".ts"];
    name.starts_with("page.") && EXTENSIONS.iter().any(|ext| name.ends_with(ext))
}

fn clean_path(parent: &str) -> String {
    if parent.is_empty() {
        return "/".to_string();
    }
    let segments: Vec<&str> = parent
        .split('/')
        .filter(|segment| !segment.starts_with('(') && !segment.ends_with(')'))
        .collect();
    format!("/{}", segments.join("/"))
}

fn page_name(path: &str) -> String {
    path.trim_start_matches('/')
        .replace('/', " ")
        .replace('-', " ")
}
