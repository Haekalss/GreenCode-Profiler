//! Color labels and status tag helpers.

use super::badges::band;

pub fn latency_color(latency_ms: f64) -> &'static str {
    const BANDS: [(f64, &str); 2] = [(1000.0, "#dc2626"), (200.0, "#d97706")];
    *band(latency_ms, &BANDS, &"#059669")
}

pub fn latency_label(latency_ms: f64) -> &'static str {
    const BANDS: [(f64, &str); 2] = [(1000.0, "Lambat"), (200.0, "Sedang")];
    *band(latency_ms, &BANDS, &"Cepat")
}

pub fn endpoint_latency_label(latency_ms: f64) -> &'static str {
    const BANDS: [(f64, &str); 2] = [(1000.0, "🔴 Lambat"), (200.0, "🟡 Sedang")];
    *band(latency_ms, &BANDS, &"🟢 Cepat")
}

pub fn energy_color(energy: f64) -> &'static str {
    const BANDS: [(f64, &str); 2] = [(5.0, "#dc2626"), (1.0, "#d97706")];
    *band(energy, &BANDS, &"#059669")
}
