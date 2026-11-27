//! `greenprofiler detect`: audit PHP runtime and find active server processes.

use crate::detector::RuntimeDetector;
use crate::php_auditor::{PhpAuditReport, PhpAuditor};

pub fn execute(language: &str, port: Option<u16>) {
    print_header(language, port);
    if matches!(language, "php" | "auto" | "fullstack") {
        print_php_audit(&PhpAuditor::audit());
    }
    print_processes(language, port);
    println!("================================================================\n");
}

fn print_header(language: &str, port: Option<u16>) {
    println!("================================================================");
    println!("      GREEN PROFILER RUNTIME PROCESS DETECTOR (Rust Engine)          ");
    println!("================================================================");
    println!("Target Language : {}", language.to_uppercase());
    let port_label = port
        .map(|value| value.to_string())
        .unwrap_or_else(|| "Semua".to_string());
    println!("Target Port     : {}", port_label);
}

fn print_php_audit(report: &PhpAuditReport) {
    if !report.is_php_available {
        return;
    }
    println!("----------------------------------------------------------------");
    println!(" [PHP GREEN EXTENSION AUDIT]");
    println!("  Versi PHP       : {}", report.php_version);
    println!("  Total Ekstensi  : {} modules loaded", report.loaded_extensions_count);
    println!("  Zend OPcache    : {}", flag(report.opcache_enabled, "[OK] Aktif (Hemat CPU ~50%)", "[PERINGATAN] Nonaktif (Kompilasi berulang, boros energi!)"));
    println!("  JIT Compiler    : {}", flag(report.jit_enabled, "[OK] Aktif (Akselerasi instruksi mesin)", "[INFO] Nonaktif"));
    println!("  Xdebug Status   : {}", flag(report.xdebug_active, "[BAHAYA] Aktif di runtime! (Overhead CPU +200%)", "[OK] Nonaktif (Optimal & ringan)"));
    if report.recommendations.is_empty() {
        return;
    }
    println!("  Rekomendasi     :");
    for recommendation in &report.recommendations {
        println!("   * {}", recommendation);
    }
}

fn flag(enabled: bool, when_true: &'static str, when_false: &'static str) -> &'static str {
    if enabled {
        return when_true;
    }
    when_false
}

fn print_processes(language: &str, port: Option<u16>) {
    let pids = RuntimeDetector::find_pids(language, port);
    println!("Ditemukan       : {} proses aktif", pids.len());
    println!("----------------------------------------------------------------");
    for pid in &pids {
        println!(" - PID {:<6}", pid);
    }
}
