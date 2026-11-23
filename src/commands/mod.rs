//! CLI command implementations and dispatch.

mod analyze;
pub mod config;
mod detect;
mod init;
mod routes;
pub mod run;

use crate::cli::{parse_flags, print_help};
use crate::reporter::ReportGenerator;
use std::path::Path;

/// Route a command-line invocation to its implementation.
pub fn dispatch(args: &[String]) {
    match args[1].as_str() {
        "init" => {
            let flags = parse_flags(&args[2..]);
            init::execute(flags.force, flags.filters, flags.excludes);
        }
        "routes" => {
            let flags = parse_flags(&args[2..]);
            let dir = flags.positional.first().map(String::as_str).unwrap_or(".");
            routes::execute(dir, &flags);
        }
        "detect" => {
            let language = args.get(2).map(String::as_str).unwrap_or("auto");
            detect::execute(language, None);
        }
        "analyze" => analyze::execute(parse_flags(&args[2..])),
        "run" => execute_run(&args[2..]),
        "ui" => open_reports(),
        "version" | "-v" | "--version" => println!("Green Profiler v1.8.2"),
        "help" | "-h" | "--help" => print_help(),
        other => {
            println!("[GreenProfiler] Perintah tidak dikenal: '{}'", other);
            println!("            Gunakan 'greencode help' untuk melihat daftar perintah.");
        }
    }
}

fn execute_run(args: &[String]) {
    let flags = parse_flags(args);
    let config = flags
        .positional
        .first()
        .cloned()
        .unwrap_or_else(|| "greencode.json".to_string());
    run::execute(&config, flags);
}

fn open_reports() {
    const CANDIDATES: [&str; 2] = ["reports/greencode_report.html", "reports/code_quality.html"];
    let found = CANDIDATES.iter().find(|path| Path::new(path).exists());
    if let Some(path) = found {
        ReportGenerator::open_in_browser(path);
        return;
    }
    println!("[GreenProfiler] Laporan belum dibuat. Jalankan 'greenprofiler run' terlebih dahulu.");
}
