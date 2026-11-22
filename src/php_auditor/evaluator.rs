//! Extension inspection, categories, and green energy evaluation.

use super::ExtensionStatus;

pub fn evaluate_accelerators(
    has_ext: &dyn Fn(&str) -> bool,
    opcache: bool,
    details: &mut Vec<ExtensionStatus>,
    recs: &mut Vec<String>,
) {
    if opcache {
        details.push(ExtensionStatus {
            name: "Zend OPcache".to_string(),
            is_loaded: true,
            category: "green_accelerator".to_string(),
            description: "Menghindari kompilasi ulang script PHP per request".to_string(),
            green_impact: "Sangat Tinggi: Menghemat 40-70% CPU cycles".to_string(),
            status_badge: "OK".to_string(),
        });
    } else {
        details.push(ExtensionStatus {
            name: "Zend OPcache".to_string(),
            is_loaded: false,
            category: "green_accelerator".to_string(),
            description: "Menghindari kompilasi ulang script PHP per request".to_string(),
            green_impact: "KRITIS: Aplikasi mengkompilasi ulang opcode di tiap request (Boros)".to_string(),
            status_badge: "DANGER".to_string(),
        });
        recs.push("Aktifkan OPcache di php.ini (zend_extension=opcache) untuk efisiensi energi drastis.".to_string());
    }

    if has_ext("apcu") {
        details.push(ExtensionStatus {
            name: "APCu".to_string(),
            is_loaded: true,
            category: "green_accelerator".to_string(),
            description: "In-memory caching untuk variabel PHP di RAM".to_string(),
            green_impact: "Tinggi: Mengurangi akses disk dan deserialisasi".to_string(),
            status_badge: "OK".to_string(),
        });
    }
}

pub fn evaluate_profilers(
    xdebug: bool,
    details: &mut Vec<ExtensionStatus>,
    recs: &mut Vec<String>,
) {
    if xdebug {
        details.push(ExtensionStatus {
            name: "Xdebug".to_string(),
            is_loaded: true,
            category: "overhead_profiler".to_string(),
            description: "Step debugger & trace profiler".to_string(),
            green_impact: "BERBAHAYA: Menambah overhead CPU 200-500% dan latensi per request".to_string(),
            status_badge: "DANGER".to_string(),
        });
        recs.push("Matikan Xdebug di lingkungan benchmarking/production (xdebug.mode=off).".to_string());
    } else {
        details.push(ExtensionStatus {
            name: "Xdebug".to_string(),
            is_loaded: false,
            category: "overhead_profiler".to_string(),
            description: "Step debugger".to_string(),
            green_impact: "Normal: Tidak ada overhead debugger aktif".to_string(),
            status_badge: "OK".to_string(),
        });
    }
}

pub fn evaluate_essentials(
    has_ext: &dyn Fn(&str) -> bool,
    details: &mut Vec<ExtensionStatus>,
    recs: &mut Vec<String>,
) {
    const ESSENTIALS: [(&str, &str); 6] = [
        ("pdo_mysql", "Koneksi basis data MySQL/MariaDB"),
        ("openssl", "Kriptografi dan pertukaran HTTPS aman"),
        ("mbstring", "Pemrosesan karakter multibyte UTF-8"),
        ("curl", "Koneksi transfer data jaringan HTTP"),
        ("tokenizer", "Tokenisasi sintaks aplikasi"),
        ("xml", "Pemrosesan dokumen terstruktur XML"),
    ];

    for (ext, desc) in ESSENTIALS {
        let loaded = has_ext(ext);
        let (impact, badge) = if loaded {
            ("Normal (Siap digunakan)", "OK")
        } else {
            ("Peringatan: Dependensi wajib aplikasi belum aktif", "WARN")
        };

        details.push(ExtensionStatus {
            name: ext.to_string(),
            is_loaded: loaded,
            category: "essential".to_string(),
            description: desc.to_string(),
            green_impact: impact.to_string(),
            status_badge: badge.to_string(),
        });
        if !loaded {
            recs.push(format!("Ekstensi esensial '{}' belum aktif di php.ini.", ext));
        }
    }
}
