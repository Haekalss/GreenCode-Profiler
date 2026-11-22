//! GreenCode Profiler v1.8.1 - Rust Native Architecture
//! Universal Process-Level Energy Profiler & Green-Maintainability Framework

pub mod analyzer;
pub mod attributor;
pub mod cli;
pub mod commands;
pub mod correlator;
pub mod detector;
pub mod engine;
pub mod php_auditor;
pub mod reporter;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        cli::print_help();
        return;
    }

    commands::dispatch(&args);
}
