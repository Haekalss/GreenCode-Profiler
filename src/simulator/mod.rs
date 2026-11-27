//! Multi-User Load, Energy, Carbon & Server Capacity Simulator berbasis Hasil Audit.
//! Mendukung simulasi dari laporan dinamis (`greenprofiler_report.json`) maupun statis (`code_quality.json`).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    pub custom_users: Option<u64>,
    pub requests_per_user: u64,
    pub concurrency_window_sec: f64,
    pub cpu_cores: u32,
    pub peak_hours_per_day: f64,
    pub grid_co2_g_per_kwh: f64,
    pub tariff_idr_per_kwh: f64,
}

impl Default for SimulationParams {
    fn default() -> Self {
        let detected_cores = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(4);
        Self {
            custom_users: None,
            requests_per_user: 10,
            concurrency_window_sec: 60.0,
            cpu_cores: detected_cores.max(1),
            peak_hours_per_day: 8.0,
            grid_co2_g_per_kwh: 850.0,
            tariff_idr_per_kwh: 1444.70,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointScaleHotspot {
    pub endpoint: String,
    pub method: String,
    pub energy_per_req_j: f64,
    pub cpu_per_req_s: f64,
    pub base_latency_ms: f64,
    pub max_rps_single_core: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationTierResult {
    pub concurrent_users: u64,
    pub total_requests_in_window: u64,
    pub throughput_rps: f64,
    pub cpu_utilization_pct: f64,
    pub required_cpu_cores: u32,
    pub projected_p95_latency_ms: f64,
    pub burst_energy_joules: f64,
    pub daily_kwh: f64,
    pub monthly_kwh: f64,
    pub monthly_co2_kg: f64,
    pub monthly_cost_idr: f64,
    pub optimized_monthly_kwh: f64,
    pub saved_monthly_kwh: f64,
    pub capacity_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiUserSimulationReport {
    pub project_name: String,
    pub source_mode: String,
    pub baseline_energy_j_per_req: f64,
    pub baseline_cpu_s_per_req: f64,
    pub baseline_latency_ms: f64,
    pub cpu_tdp_watts: f64,
    pub cpu_cores: u32,
    pub requests_per_user: u64,
    pub concurrency_window_sec: f64,
    pub peak_hours_per_day: f64,
    pub max_sustainable_rps_80pct: f64,
    pub max_concurrent_users_80pct: u64,
    pub estimated_opt_saving_pct: f64,
    pub optimization_driver: String,
    pub tiers: Vec<SimulationTierResult>,
    pub endpoint_hotspots: Vec<EndpointScaleHotspot>,
    pub capacity_recommendations: Vec<String>,
}

pub struct AuditSimulator;

impl AuditSimulator {
    pub fn simulate(audit_data: &Value, params: &SimulationParams) -> MultiUserSimulationReport {
        let (
            project_name,
            source_mode,
            energy_per_req,
            cpu_per_req,
            base_latency_ms,
            cpu_tdp_watts,
            opt_saving_ratio,
            opt_driver,
            hotspots,
        ) = extract_baseline(audit_data);

        let cores = params.cpu_cores.max(1);
        let rpu = params.requests_per_user.max(1);
        let window = params.concurrency_window_sec.max(1.0);

        let mut user_counts: Vec<u64> = vec![100, 500, 1_000, 5_000, 10_000, 50_000];
        if let Some(custom) = params.custom_users {
            if custom > 0 && !user_counts.contains(&custom) {
                user_counts.push(custom);
                user_counts.sort_unstable();
            }
        }

        let windows_per_day = (params.peak_hours_per_day * 3600.0) / window;
        let mut tiers = Vec::with_capacity(user_counts.len());

        for users in user_counts {
            let total_reqs = users.saturating_mul(rpu);
            let rps = total_reqs as f64 / window;
            let cpu_demand_cores = rps * cpu_per_req;
            let util_pct = (cpu_demand_cores / cores as f64) * 100.0;
            let rho = cpu_demand_cores / cores as f64;

            let p95_latency = if rho < 0.98 {
                base_latency_ms * (1.25 + (rho * rho) / (2.0 * (1.0 - rho)))
            } else {
                base_latency_ms * (25.0 + (rho - 0.98) * 120.0)
            };

            let req_cores = ((cpu_demand_cores / 0.75).ceil() as u32).max(1);
            let burst_j = total_reqs as f64 * energy_per_req;
            let daily_kwh = (burst_j * windows_per_day) / 3_600_000.0;
            let monthly_kwh = daily_kwh * 30.0;
            let monthly_co2_kg = (monthly_kwh * params.grid_co2_g_per_kwh) / 1000.0;
            let monthly_cost_idr = monthly_kwh * params.tariff_idr_per_kwh;
            let opt_monthly_kwh = monthly_kwh * (1.0 - opt_saving_ratio);
            let saved_kwh = (monthly_kwh - opt_monthly_kwh).max(0.0);

            let status = if util_pct <= 60.0 {
                "OPTIMAL"
            } else if util_pct <= 85.0 {
                "MODERAT (Mulai Antre)"
            } else if util_pct <= 100.0 {
                "TINGGI (Hampir Penuh)"
            } else {
                "OVERLOAD (CPU Bottleneck)"
            };

            tiers.push(SimulationTierResult {
                concurrent_users: users,
                total_requests_in_window: total_reqs,
                throughput_rps: round2(rps),
                cpu_utilization_pct: round1(util_pct),
                required_cpu_cores: req_cores,
                projected_p95_latency_ms: round1(p95_latency),
                burst_energy_joules: round2(burst_j),
                daily_kwh: round4(daily_kwh),
                monthly_kwh: round3(monthly_kwh),
                monthly_co2_kg: round3(monthly_co2_kg),
                monthly_cost_idr: round2(monthly_cost_idr),
                optimized_monthly_kwh: round3(opt_monthly_kwh),
                saved_monthly_kwh: round3(saved_kwh),
                capacity_status: status.to_string(),
            });
        }

        let max_rps_80 = (0.80 * cores as f64) / cpu_per_req.max(0.0001);
        let max_users_80 = ((max_rps_80 * window) / rpu as f64).floor() as u64;

        let mut recs = Vec::new();
        recs.push(format!(
            "Kapasitas aman server saat ini ({} vCPU, batas utilisasi 80%) mampu melayani hingga ~{} user konkuren ({:.1} req/detik).",
            cores, max_users_80, max_rps_80
        ));
        if opt_saving_ratio >= 0.20 {
            recs.push(format!(
                "Potensi penghematan energi & CPU sebesar {:.0}% ({}) dapat meningkatkan kapasitas user konkuren dari ~{} menjadi ~{} user tanpa menambah vCPU.",
                opt_saving_ratio * 100.0,
                opt_driver,
                max_users_80,
                ((max_users_80 as f64) / (1.0 - opt_saving_ratio).max(0.25)).round() as u64
            ));
        }
        if let Some(t10k) = tiers.iter().find(|t| t.concurrent_users == 10_000) {
            if t10k.cpu_utilization_pct > 85.0 {
                recs.push(format!(
                    "Pada skala 10.000 user konkuren ({:.1} RPS), utilisasi mencapai {:.1}% -> siapkan minimal {} vCPU (horizontal scaling / PHP-FPM pool + Redis cache).",
                    t10k.throughput_rps, t10k.cpu_utilization_pct, t10k.required_cpu_cores
                ));
            }
        }

        MultiUserSimulationReport {
            project_name,
            source_mode,
            baseline_energy_j_per_req: round6(energy_per_req),
            baseline_cpu_s_per_req: round6(cpu_per_req),
            baseline_latency_ms: round2(base_latency_ms),
            cpu_tdp_watts: round1(cpu_tdp_watts),
            cpu_cores: cores,
            requests_per_user: rpu,
            concurrency_window_sec: window,
            peak_hours_per_day: params.peak_hours_per_day,
            max_sustainable_rps_80pct: round2(max_rps_80),
            max_concurrent_users_80pct: max_users_80,
            estimated_opt_saving_pct: round1(opt_saving_ratio * 100.0),
            optimization_driver: opt_driver,
            tiers,
            endpoint_hotspots: hotspots,
            capacity_recommendations: recs,
        }
    }
}

fn extract_baseline(
    data: &Value,
) -> (
    String,
    String,
    f64,
    f64,
    f64,
    f64,
    f64,
    String,
    Vec<EndpointScaleHotspot>,
) {
    let total_reqs = data["totals"]["total_requests"].as_u64().unwrap_or(0);
    let cq = if data["code_quality"].is_object() {
        &data["code_quality"]
    } else {
        data
    };

    let php_ext_count = cq["php_extension_candidates"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let smells_count = cq["overview"]["total_code_smells"].as_u64().unwrap_or(0) as usize;
    let opcache_off = data["php_environment"]["is_php_available"]
        .as_bool()
        .unwrap_or(false)
        && !data["php_environment"]["opcache_enabled"]
            .as_bool()
            .unwrap_or(true);
    let xdebug_on = data["php_environment"]["xdebug_active"]
        .as_bool()
        .unwrap_or(false);

    let mut opt_ratio: f64 = 0.12;
    let mut drivers: Vec<String> = Vec::new();
    if php_ext_count > 0 {
        let add = (0.10 + php_ext_count as f64 * 0.035).min(0.38);
        opt_ratio += add;
        drivers.push(format!("Kompilasi {} fungsi berulang ke Native PHP Extension", php_ext_count));
    }
    if opcache_off {
        opt_ratio += 0.25;
        drivers.push("Aktivasi Zend OPcache".to_string());
    }
    if xdebug_on {
        opt_ratio += 0.30;
        drivers.push("Penonaktifan Xdebug di Production".to_string());
    }
    if smells_count > 5 {
        opt_ratio += 0.08;
        drivers.push("Modularisasi Code Smells".to_string());
    }
    if drivers.is_empty() {
        drivers.push("Tuning Cache & Query Pool".to_string());
    }
    let opt_saving_ratio = opt_ratio.min(0.70);
    let opt_driver = drivers.join(" + ");

    if total_reqs > 0 {
        let project = data["metadata"]["project_name"]
            .as_str()
            .unwrap_or("Audited Application")
            .to_string();
        let tdp = data["metadata"]["cpu_tdp_watts"].as_f64().unwrap_or(15.0);
        let total_j = data["totals"]["total_joules"].as_f64().unwrap_or(0.0);
        let total_cpu = data["totals"]["total_cpu_active_s"].as_f64().unwrap_or(0.0);
        let avg_lat = data["totals"]["avg_latency_ms"].as_f64().unwrap_or(20.0);

        let e_per_req = (total_j / total_reqs as f64).max(0.001);
        let cpu_per_req = (total_cpu / total_reqs as f64).max(0.0002);

        let mut hotspots: Vec<EndpointScaleHotspot> = data["endpoints_detail"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|ep| {
                        let c = ep["cpu_active_time_s"].as_f64().unwrap_or(cpu_per_req).max(0.0001);
                        EndpointScaleHotspot {
                            endpoint: ep["endpoint"].as_str().unwrap_or("/").to_string(),
                            method: ep["method"].as_str().unwrap_or("GET").to_string(),
                            energy_per_req_j: ep["energy_total_joules"].as_f64().unwrap_or(e_per_req),
                            cpu_per_req_s: c,
                            base_latency_ms: ep["latency_ms"].as_f64().unwrap_or(avg_lat),
                            max_rps_single_core: round1(0.80 / c),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        hotspots.sort_by(|a, b| {
            b.energy_per_req_j
                .partial_cmp(&a.energy_per_req_j)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hotspots.truncate(8);

        return (
            project,
            "Empiris (Hasil Profiling Runtime)".to_string(),
            e_per_req,
            cpu_per_req,
            avg_lat.max(2.0),
            tdp,
            opt_saving_ratio,
            opt_driver,
            hotspots,
        );
    }

    // Fallback: Estimasi dari hasil analisis statis (`code_quality.json`)
    let avg_ccn = cq["overview"]["average_ccn_per_file"].as_f64().unwrap_or(10.0);
    let avg_mi = cq["overview"]["average_maintainability_index"]
        .as_f64()
        .unwrap_or(75.0);
    let tdp = 15.0;
    let complexity_factor = 1.0 + (avg_ccn / 15.0) + ((100.0 - avg_mi).max(0.0) / 120.0);
    let cpu_per_req = (0.0075 * complexity_factor).max(0.001);
    let e_per_req = cpu_per_req * tdp;
    let base_lat = 22.0 * complexity_factor;

    (
        "Static Audited Codebase".to_string(),
        "Estimasi Kompleksitas Statis (CCN & MI)".to_string(),
        e_per_req,
        cpu_per_req,
        base_lat,
        tdp,
        opt_saving_ratio,
        opt_driver,
        Vec::new(),
    )
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}
fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}
fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}
