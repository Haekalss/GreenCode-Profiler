//! PHP Environment & Green Extension Auditor
//! Mengaudit ketersediaan extension PHP, akselerator hemat energi (OPcache, JIT),
//! dan mendeteksi extension boros daya (Xdebug).

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionStatus {
    pub name: String,
    pub is_loaded: bool,
    pub category: String, // "essential", "green_accelerator", "debug_overhead"
    pub description: String,
    pub green_impact: String,
    pub status_badge: String, // "OK", "WARN", "DANGER", "INFO"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpAuditReport {
    pub is_php_available: bool,
    pub php_version: String,
    pub loaded_extensions_count: usize,
    pub opcache_enabled: bool,
    pub jit_enabled: bool,
    pub xdebug_active: bool,
    pub recommendations: Vec<String>,
    pub extension_details: Vec<ExtensionStatus>,
}

pub struct PhpAuditor;

impl PhpAuditor {
    /// Menjalankan audit PHP extensions dan green runtime configurations
    pub fn audit() -> PhpAuditReport {
        let version_output = Command::new("php").args(["-v"]).output();
        if version_output.is_err() {
            return PhpAuditReport {
                is_php_available: false,
                php_version: "PHP runtime tidak terdeteksi di sistem".to_string(),
                loaded_extensions_count: 0,
                opcache_enabled: false,
                jit_enabled: false,
                xdebug_active: false,
                recommendations: vec![
                    "Pastikan PHP sudah terpasang dan terdaftar di PATH lingkungan sistem.".to_string(),
                ],
                extension_details: Vec::new(),
            };
        }

        let ver_stdout = match version_output {
            Ok(ref o) => o.stdout.clone(),
            Err(_) => Vec::new(),
        };
        let ver_raw = String::from_utf8_lossy(&ver_stdout);
        let first_line = ver_raw.lines().next().unwrap_or("PHP (Unknown Version)").to_string();

        let mut loaded_modules = Vec::new();
        if let Ok(mod_output) = Command::new("php").args(["-m"]).output() {
            let mod_text = String::from_utf8_lossy(&mod_output.stdout);
            for line in mod_text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('[') {
                    loaded_modules.push(trimmed.to_lowercase());
                }
            }
        }

        let has_ext = |name: &str| loaded_modules.iter().any(|m| m == name);

        let opcache = has_ext("zend opcache") || has_ext("opcache");
        let xdebug = has_ext("xdebug");

        // Cek JIT via php -i
        let mut jit = false;
        if let Ok(info_out) = Command::new("php").args(["-i"]).output() {
            let info_text = String::from_utf8_lossy(&info_out.stdout);
            for line in info_text.lines() {
                if line.contains("opcache.jit") && (line.contains("tracing") || line.contains("1205") || line.contains("function") || line.contains("on")) {
                    jit = true;
                    break;
                }
            }
        }

        let mut details = Vec::new();
        let mut recs = Vec::new();

        // 1. Green Accelerators
        if opcache {
            details.push(ExtensionStatus {
                name: "Zend OPcache".to_string(),
                is_loaded: true,
                category: "green_accelerator".to_string(),
                description: "Menyimpan bytecode terkompilasi dalam memori bersama".to_string(),
                green_impact: "Mereduksi konsumsi CPU hingga 50% dengan meniadakan kompilasi berulang".to_string(),
                status_badge: "OK".to_string(),
            });
        } else {
            details.push(ExtensionStatus {
                name: "Zend OPcache".to_string(),
                is_loaded: false,
                category: "green_accelerator".to_string(),
                description: "Menyimpan bytecode terkompilasi dalam memori bersama".to_string(),
                green_impact: "Kritis: Tanpa OPcache, server mengompilasi skrip pada setiap request (boros energi)".to_string(),
                status_badge: "WARN".to_string(),
            });
            recs.push("Aktifkan ekstensi 'opcache' di php.ini untuk menghemat siklus komputasi CPU hingga 50%.".to_string());
        }

        if jit {
            details.push(ExtensionStatus {
                name: "JIT Compiler (OPcache)".to_string(),
                is_loaded: true,
                category: "green_accelerator".to_string(),
                description: "Kompilasi kode dinamis ke instruksi mesin murni".to_string(),
                green_impact: "Sangat efisien untuk kalkulasi data intensif".to_string(),
                status_badge: "OK".to_string(),
            });
        } else {
            details.push(ExtensionStatus {
                name: "JIT Compiler (OPcache)".to_string(),
                is_loaded: false,
                category: "green_accelerator".to_string(),
                description: "Kompilasi kode dinamis ke instruksi mesin murni".to_string(),
                green_impact: "Opsional: Dapat diaktifkan untuk efisiensi komputasi berat".to_string(),
                status_badge: "INFO".to_string(),
            });
        }

        // 2. Debug Overhead
        if xdebug {
            details.push(ExtensionStatus {
                name: "Xdebug".to_string(),
                is_loaded: true,
                category: "debug_overhead".to_string(),
                description: "Pemeriksa jejak eksekusi dan debugger PHP".to_string(),
                green_impact: "Bahaya: Menambah overhead CPU 200-300% jika aktif di lingkungan benchmark/produksi".to_string(),
                status_badge: "DANGER".to_string(),
            });
            recs.push("Nonaktifkan ekstensi 'xdebug' saat audit energi/produksi untuk mencegah pemborosan daya CPU berlebih.".to_string());
        } else {
            details.push(ExtensionStatus {
                name: "Xdebug".to_string(),
                is_loaded: false,
                category: "debug_overhead".to_string(),
                description: "Pemeriksa jejak eksekusi dan debugger PHP".to_string(),
                green_impact: "Optimal: Tidak ada penalti performa dan pemborosan CPU".to_string(),
                status_badge: "OK".to_string(),
            });
        }

        // 3. Essential Web Extensions (Laravel / Modern PHP)
        let essentials = [
            ("pdo_mysql", "Koneksi basis data MySQL/MariaDB"),
            ("openssl", "Kriptografi dan pertukaran HTTPS aman"),
            ("mbstring", "Pemrosesan karakter multibyte UTF-8"),
            ("curl", "Koneksi transfer data jaringan HTTP"),
            ("tokenizer", "Tokenisasi sintaks aplikasi"),
            ("xml", "Pemrosesan dokumen terstruktur XML"),
        ];

        for (ext, desc) in essentials {
            let loaded = has_ext(ext);
            details.push(ExtensionStatus {
                name: ext.to_string(),
                is_loaded: loaded,
                category: "essential".to_string(),
                description: desc.to_string(),
                green_impact: if loaded { "Normal (Siap digunakan)" } else { "Peringatan: Dependensi wajib aplikasi belum aktif" }.to_string(),
                status_badge: if loaded { "OK" } else { "WARN" }.to_string(),
            });
            if !loaded {
                recs.push(format!("Ekstensi esensial '{}' belum aktif di php.ini.", ext));
            }
        }

        PhpAuditReport {
            is_php_available: true,
            php_version: first_line,
            loaded_extensions_count: loaded_modules.len(),
            opcache_enabled: opcache,
            jit_enabled: jit,
            xdebug_active: xdebug,
            recommendations: recs,
            extension_details: details,
        }
    }
}
