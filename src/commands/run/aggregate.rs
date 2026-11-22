//! Aggregation helpers for benchmark runs.

use crate::attributor::RequestTraceRecord;
use std::collections::HashMap;

pub fn aggregate_endpoint_details(records: &[RequestTraceRecord]) -> Vec<RequestTraceRecord> {
    let mut ep_order: Vec<String> = Vec::new();
    let mut ep_groups: HashMap<String, Vec<&RequestTraceRecord>> = HashMap::new();

    for rec in records {
        let key = format!("{} {}", rec.method, rec.endpoint);
        if !ep_groups.contains_key(&key) {
            ep_order.push(key.clone());
        }
        ep_groups.entry(key).or_default().push(rec);
    }

    let mut details: Vec<RequestTraceRecord> = Vec::new();
    for key in &ep_order {
        let list = &ep_groups[key];
        let n = list.len() as f64;
        let mean = |f: fn(&RequestTraceRecord) -> f64| list.iter().map(|r| f(r)).sum::<f64>() / n;
        let first = list[0];
        details.push(RequestTraceRecord {
            endpoint: first.endpoint.clone(),
            method: first.method.clone(),
            status: first.status,
            latency_ms: (mean(|r| r.latency_ms) * 10.0).round() / 10.0,
            cpu_active_time_s: (mean(|r| r.cpu_active_time_s) * 10000.0).round() / 10000.0,
            energy_total_joules: (mean(|r| r.energy_total_joules) * 1000000.0).round() / 1000000.0,
            co2e_grams: (mean(|r| r.co2e_grams) * 1000000.0).round() / 1000000.0,
            cpu_utilization: (mean(|r| r.cpu_utilization) * 10.0).round() / 10.0,
        });
    }
    details
}
