//! Endpoint & keyword filtering logic.

use crate::detector::RouteEndpoint;

pub fn apply_filters(
    routes: Vec<RouteEndpoint>,
    filters: &[String],
    excludes: &[String],
) -> Vec<RouteEndpoint> {
    if filters.is_empty() && excludes.is_empty() {
        return routes;
    }
    routes
        .into_iter()
        .filter(|route| matches_keywords(route, filters, excludes))
        .collect()
}

fn matches_keywords(route: &RouteEndpoint, filters: &[String], excludes: &[String]) -> bool {
    let haystack = format!("{} {}", route.path, route.name).to_lowercase();
    let included = filters.is_empty()
        || filters
            .iter()
            .any(|keyword| haystack.contains(&keyword.to_lowercase()));
    let excluded = excludes
        .iter()
        .any(|keyword| haystack.contains(&keyword.to_lowercase()));
    included && !excluded
}
