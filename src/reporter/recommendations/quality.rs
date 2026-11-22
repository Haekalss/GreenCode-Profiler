//! Source file and code smell refactoring recommendations.

pub fn file_recommendation(ccn: u64, mi: f64, ncloc: u64, smells: usize) -> &'static str {
    match () {
        _ if ccn > 20 && mi < 40.0 => "URGENT: Refactor besar - pecah class, extract method, kurangi kompleksitas",
        _ if ccn > 20 => "Extract methods, kurangi percabangan, gunakan strategy pattern",
        _ if mi < 40.0 && ncloc > 300 => "Modularisasi file besar, pecah per tanggung jawab (SRP)",
        _ if mi < 40.0 => "Perbaiki naming, kurangi nesting, tambah dokumentasi",
        _ if smells > 3 => "Bersihkan code smells: hapus dead code, dedup, early return",
        _ if ccn > 10 => "Pertimbangkan extract method untuk logic kompleks",
        _ => "✅ Bersih & maintainable",
    }
}

pub fn smell_recommendation(smell_type: &str) -> &'static str {
    const RULES: [(&str, &str); 7] = [
        ("GodClass", "Pecah class jadi beberapa class kecil (SRP), extract service"),
        ("FatController", "Pindah logika bisnis ke Service/Action, controller hanya handle HTTP"),
        ("DeepNesting", "Early return, guard clause, extract method, flatten conditional"),
        ("LongMethod", "Extract method, gunakan composer pattern, kurangi tanggung jawab"),
        ("LongParameterList", "Gunakan DTO/Parameter Object, builder pattern"),
        ("DuplicateCode", "Extract common logic ke helper/trait, DRY principle"),
        ("DeadCode", "Hapus kode tidak terpakai, gunakan static analysis"),
    ];
    RULES
        .iter()
        .find(|(kind, _)| *kind == smell_type)
        .map(|(_, advice)| *advice)
        .unwrap_or("Refactor sesuai pola: SRP, KISS, DRY, Clean Architecture")
}
