//! Module & endpoint performance recommendations.

pub fn module_recommendation(score: f64, energy: f64, latency_ms: f64) -> &'static str {
    match () {
        _ if score < 60.0 && energy > 5.0 => "Refactor kritis: Optimasi query + cache + kurangi kompleksitas",
        _ if score < 85.0 && latency_ms > 500.0 => "Optimasi DB index + gunakan query builder + pagination",
        _ if latency_ms > 1000.0 => "Investigasi N+1 query, tambah Redis cache, async processing",
        _ if energy > 2.0 => "Aktifkan OPcache JIT, kurangi middleware berlebih",
        _ if score < 60.0 => "Perbaiki arsitektur: pecah God Class, kurangi nesting",
        _ => "✅ Sudah optimal",
    }
}

pub fn endpoint_recommendation(latency_ms: f64, energy: f64) -> &'static str {
    match () {
        _ if latency_ms > 1000.0 && energy > 5.0 => "URGENT: Query N+1 + no cache + boros energi",
        _ if latency_ms > 1000.0 => "Optimasi DB index + cache Redis + async job",
        _ if energy > 5.0 => "Aktifkan OPcache JIT, kurangi overhead PHP",
        _ if latency_ms > 200.0 => "Pagination + eager loading + query optimization",
        _ => "✅ Optimal",
    }
}
