//! Route discovery orchestration and shared route helpers.

pub mod builder;
mod go;
mod laravel;
mod nextjs;
mod node;
pub mod path_util;
mod python;
mod rust;

pub use builder::{apply_limit, dedup, filter_profilable, tagged};
pub use path_util::{has_param, push_route};

use self::go::discover_go;
use self::laravel::discover_laravel;
use self::nextjs::discover_nextjs;
use self::node::discover_node;
use self::python::discover_python;
use self::rust::discover_rust;
use super::{ComponentInfo, RouteDiscoverer, RouteEndpoint, RuntimeDetector};
use std::path::Path;

impl RouteDiscoverer {
    pub fn discover<P: AsRef<Path>>(project_dir: P, max_routes: usize) -> Vec<RouteEndpoint> {
        let root = project_dir.as_ref();
        let fullstack = RuntimeDetector::detect_fullstack_components(root);
        let mut collected = Vec::new();

        if let Some(frontend) = fullstack.frontend {
            collected.extend(frontend_routes(root, &frontend));
        }
        if let Some(backend) = fullstack.backend {
            collected.extend(backend_routes(root, &backend));
        }
        if collected.is_empty() {
            collected = root_routes(root);
        }

        apply_limit(dedup(collected), max_routes)
    }

    pub fn profilable(routes: Vec<RouteEndpoint>) -> Vec<RouteEndpoint> {
        filter_profilable(routes)
    }
}

fn frontend_routes(root: &Path, frontend: &ComponentInfo) -> Vec<RouteEndpoint> {
    let dir = root.join(&frontend.dir);
    let mut routes = if frontend.framework == "nextjs" {
        let discovered = discover_nextjs(&dir);
        if discovered.is_empty() {
            discover_node(&dir)
        } else {
            discovered
        }
    } else {
        discover_node(&dir)
    };
    routes.sort_by(|a, b| a.path.cmp(&b.path));
    routes
        .into_iter()
        .map(|route| tagged(route, "FE", "frontend", &frontend.base_url))
        .collect()
}

fn backend_routes(root: &Path, backend: &ComponentInfo) -> Vec<RouteEndpoint> {
    let dir = root.join(&backend.dir);
    let mut routes = discover_for_language(&dir, &backend.language);
    routes.sort_by(|a, b| (&a.path, &a.method).cmp(&(&b.path, &b.method)));
    routes
        .into_iter()
        .map(|route| tagged(route, "BE", "backend", &backend.base_url))
        .collect()
}

fn discover_for_language(dir: &Path, language: &str) -> Vec<RouteEndpoint> {
    match language {
        "php" => discover_laravel(dir),
        "go" => discover_go(dir),
        "python" => discover_python(dir),
        "rust" => discover_rust(dir),
        "node" => discover_node(dir),
        _ => discover_any(dir),
    }
}

fn discover_any(dir: &Path) -> Vec<RouteEndpoint> {
    let candidates = [
        discover_laravel(dir),
        discover_go(dir),
        discover_python(dir),
        discover_rust(dir),
    ];
    candidates
        .into_iter()
        .find(|routes| !routes.is_empty())
        .unwrap_or_default()
}

fn root_routes(root: &Path) -> Vec<RouteEndpoint> {
    let ecosystem = RuntimeDetector::detect_project_ecosystem(root);
    let mut routes = match ecosystem.as_str() {
        "node" => discover_node_project(root),
        "php" => discover_laravel(root),
        "python" => discover_python(root),
        "rust" => discover_rust(root),
        "go" => discover_go(root),
        _ => discover_all(root),
    };
    routes.sort_by(|a, b| (&a.path, &a.method).cmp(&(&b.path, &b.method)));
    routes
}

fn discover_node_project(root: &Path) -> Vec<RouteEndpoint> {
    let next = discover_nextjs(root);
    if next.is_empty() {
        return discover_node(root);
    }
    next
}

fn discover_all(root: &Path) -> Vec<RouteEndpoint> {
    let mut routes = discover_nextjs(root);
    routes.extend(discover_node(root));
    routes.extend(discover_laravel(root));
    routes.extend(discover_python(root));
    routes.extend(discover_go(root));
    routes.extend(discover_rust(root));
    routes
}
