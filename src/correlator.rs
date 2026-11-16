//! Green-Maintainability Correlator in Rust
//! Mengawinkan metrik statis kode (CCN, NCLOC) dengan profil energi dinamis (Joules)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedModule {
    pub module_name: String,
    pub total_calls: usize,
    pub avg_latency_ms: f64,
    pub total_energy_joules: f64,
    pub avg_cpu_active_s: f64,
    pub green_efficiency_score: f64,
}

pub struct GreenCorrelator;

impl GreenCorrelator {
    pub fn correlate(
        endpoint_records: &[crate::attributor::RequestTraceRecord],
    ) -> Vec<CorrelatedModule> {
        use std::collections::HashMap;

        let mut groups: HashMap<String, Vec<&crate::attributor::RequestTraceRecord>> = HashMap::new();
        for r in endpoint_records {
            groups.entry(r.endpoint.clone()).or_default().push(r);
        }

        let mut result = Vec::new();
        for (ep, list) in groups {
            let total_calls = list.len();
            let avg_latency = list.iter().map(|r| r.latency_ms).sum::<f64>() / total_calls as f64;
            let total_energy = list.iter().map(|r| r.energy_total_joules).sum::<f64>();
            let avg_cpu = list.iter().map(|r| r.cpu_active_time_s).sum::<f64>() / total_calls as f64;

            // Green Efficiency Score: Skala 0 - 100 (Semakin tinggi semakin hemat)
            let score = (100.0 - (total_energy * 20.0)).clamp(10.0, 100.0);

            result.push(CorrelatedModule {
                module_name: ep,
                total_calls,
                avg_latency_ms: (avg_latency * 10.0).round() / 10.0,
                total_energy_joules: (total_energy * 1000000.0).round() / 1000000.0,
                avg_cpu_active_s: (avg_cpu * 10000.0).round() / 10000.0,
                green_efficiency_score: (score * 10.0).round() / 10.0,
            });
        }

        result.sort_by(|a, b| b.total_energy_joules.partial_cmp(&a.total_energy_joules).unwrap());
        result
    }
}
