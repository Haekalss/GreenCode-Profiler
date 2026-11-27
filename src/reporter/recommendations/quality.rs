//! Source file and code smell refactoring recommendations.

pub fn file_recommendation(ccn: u64, mi: f64, ncloc: u64, smells: usize) -> &'static str {
    match () {
        _ if ccn > 25 && mi < 40.0 => "URGENT: Refactor besar - pecah class, extract method, kurangi kompleksitas",
        _ if ccn > 25 => "Extract methods, kurangi percabangan, gunakan strategy pattern",
        _ if mi < 45.0 && ncloc > 350 => "Modularisasi berkas besar, pecah per tanggung jawab (SRP)",
        _ if mi < 45.0 => "Perbaiki struktur logika, kurangi nesting, tambah modularitas",
        _ if smells >= 2 => "Bersihkan code smells: modularisasi & gunakan early return",
        _ if smells == 1 => "Tinjau batas ukuran/nesting berkas agar tetap modular",
        _ if ccn > 15 => "Pertimbangkan extract helper/method bila logika bertambah",
        _ => "✅ Bersih & maintainable",
    }
}

pub fn smell_recommendation(smell_type: &str) -> &'static str {
    const RULES: [(&str, &str); 9] = [
        ("GodClass", "Pecah class jadi beberapa class kecil (SRP), extract service"),
        ("FatController", "Pindah logika bisnis ke Service/Action, controller hanya handle HTTP"),
        (
            "Fat Controller / Large Class",
            "Ekstrak logika bisnis ke Service/Trait (Backend) atau pecah sub-komponen Atomic Design (Frontend)",
        ),
        ("DeepNesting", "Early return, guard clause, extract method, flatten conditional"),
        (
            "Deep Nesting",
            "Gunakan guard clause / early return atau ekstrak blok terdalam ke fungsi terpisah",
        ),
        ("LongMethod", "Extract method, gunakan composer pattern, kurangi tanggung jawab"),
        ("LongParameterList", "Gunakan DTO/Parameter Object, builder pattern"),
        ("DuplicateCode", "Extract common logic ke helper/trait atau Native PHP Extension"),
        ("DeadCode", "Hapus kode tidak terpakai, gunakan static analysis"),
    ];
    RULES
        .iter()
        .find(|(kind, _)| *kind == smell_type)
        .map(|(_, advice)| *advice)
        .unwrap_or("Refactor sesuai pola: SRP, KISS, DRY, Clean Architecture")
}
