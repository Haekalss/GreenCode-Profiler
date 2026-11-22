//! Shared `greencode.json` reading and endpoint/URL helpers.

use crate::detector::RouteEndpoint;
use serde_json::{json, Value};
use std::fs;

/// Baca config JSON; kembalikan objek kosong bila berkas tidak valid.
pub fn read(path: &str) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}))
}

pub fn endpoints(config: &Value) -> Vec<RouteEndpoint> {
    config["endpoints"]
        .as_array()
        .map(|list| list.iter().map(parse_endpoint).collect())
        .unwrap_or_default()
}

fn parse_endpoint(value: &Value) -> RouteEndpoint {
    RouteEndpoint {
        path: value["path"].as_str().unwrap_or("/").to_string(),
        name: value["name"].as_str().unwrap_or("").to_string(),
        method: value["method"].as_str().unwrap_or("GET").to_string(),
        service: value["service"].as_str().map(str::to_string),
        base_url: value["base_url"].as_str().map(str::to_string),
        url: value["url"].as_str().map(str::to_string),
    }
}

/// Direktori sumber: `source_dirs` (monorepo) atau `source_dir` tunggal.
pub fn targets(config: &Value) -> Vec<String> {
    if let Some(list) = config["source_dirs"].as_array() {
        return list
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect();
    }
    config["source_dir"]
        .as_str()
        .map(|dir| vec![dir.to_string()])
        .unwrap_or_default()
}

/// URL final untuk satu endpoint, menghormati `url` eksplisit dan `services`.
pub fn endpoint_url(endpoint: &RouteEndpoint, base_url: &str, services: &Value) -> String {
    if let Some(url) = &endpoint.url {
        return url.clone();
    }
    if let Some(service) = &endpoint.service {
        if let Some(service_url) = services[service].as_str() {
            return format!("{}{}", service_url.trim_end_matches('/'), endpoint.path);
        }
    }
    format!("{}{}", base_url.trim_end_matches('/'), endpoint.path)
}
