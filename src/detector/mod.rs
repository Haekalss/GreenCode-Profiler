//! Runtime detection and route discovery for Node, PHP, Python, Go, and Rust.

mod ecosystem;
mod files;
mod routes;
mod runtime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEndpoint {
    pub path: String,
    pub name: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub language: String,
    pub framework: String,
    pub dir: String,
    pub source_dir: String,
    pub default_port: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullstackDetection {
    pub is_fullstack: bool,
    pub backend: Option<ComponentInfo>,
    pub frontend: Option<ComponentInfo>,
}

pub struct RuntimeDetector;

pub struct RouteDiscoverer;

/// Bangun `ComponentInfo` dengan `base_url` mengikuti port default.
pub(super) fn component(
    language: &str,
    framework: &str,
    dir: &str,
    source_dir: String,
    default_port: u16,
) -> ComponentInfo {
    ComponentInfo {
        language: language.to_string(),
        framework: framework.to_string(),
        dir: dir.to_string(),
        source_dir,
        default_port,
        base_url: format!("http://127.0.0.1:{}", default_port),
    }
}
