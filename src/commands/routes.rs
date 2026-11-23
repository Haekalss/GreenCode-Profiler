//! `greenprofiler routes`: discover and print real application routes.

use crate::cli::{apply_filters, print_filter_keywords, print_no_route_hints, CliFlags};
use crate::detector::{RouteDiscoverer, RouteEndpoint};
use std::path::Path;

pub fn execute(path: &str, flags: &CliFlags) {
    print_header();
    if !Path::new(path).exists() {
        print_missing_path(path);
        return;
    }
    print_target(path);

    let all_routes = RouteDiscoverer::discover(path, flags.limit);
    if all_routes.is_empty() {
        print_empty(path);
        return;
    }
    let total = all_routes.len();
    let routes = apply_filters(all_routes, &flags.filters, &flags.excludes);
    print_count(routes.len(), total, flags.has_filter());
    if routes.is_empty() {
        print_no_match(flags);
        return;
    }
    print_routes(&routes);
}

fn print_header() {
    println!("================================================================");
    println!("      GREENCODE AUTOMATIC ROUTE DISCOVERY (Rust Engine)         ");
    println!("================================================================");
}

fn print_missing_path(path: &str) {
    println!("[GreenError] Folder tidak ditemukan: {}", path);
    println!("            Periksa kembali path, atau jalankan dari root monorepo:");
    println!("              greenprofiler routes");
    println!("================================================================\n");
}

fn print_target(path: &str) {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.into());
    println!("Direktori Target : {}", canonical.display());
}

fn print_empty(path: &str) {
    println!("Total Ditemukan  : 0 rute nyata");
    println!("----------------------------------------------------------------");
    print_no_route_hints(path);
    println!("================================================================\n");
}

fn print_count(found: usize, total: usize, filtered: bool) {
    let suffix = if filtered {
        format!(" (disaring dari {})", total)
    } else {
        String::new()
    };
    println!("Total Ditemukan  : {} rute nyata{}", found, suffix);
    println!("----------------------------------------------------------------");
}

fn print_no_match(flags: &CliFlags) {
    println!("[GreenProfiler] 0 rute cocok dengan --filter / --exclude.");
    println!("            Kata kunci yang dipakai:");
    print_filter_keywords(&flags.filters, &flags.excludes);
    println!("================================================================\n");
}

fn print_routes(routes: &[RouteEndpoint]) {
    for route in routes {
        let service = route
            .service
            .as_ref()
            .map(|name| format!("[{}] ", name.to_uppercase()))
            .unwrap_or_default();
        println!(
            " - [{:<4}] {}{:<35} ({})",
            route.method, service, route.path, route.name
        );
    }
    print_footer();
}

fn print_footer() {
    println!("================================================================\n");
}
