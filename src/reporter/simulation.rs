//! HTML section & standalone report generator for PHP Extension Candidates and Multi-User Simulation.

use super::badges::{integer, number, text};
use super::render::render;
use super::LOGO_BASE64;
use serde_json::Value;
use std::fs;
use std::path::Path;

const TEMPLATE: &str = include_str!("templates/simulation.html");

pub fn generate(sim_data: &Value, path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let html = render(
        TEMPLATE,
        &[
            ("LOGO", LOGO_BASE64.to_string()),
            ("PROJECT_NAME", text(sim_data, "project_name").to_string()),
            ("SOURCE_MODE", text(sim_data, "source_mode").to_string()),
            (
                "MAX_USERS_80",
                integer(sim_data, "max_concurrent_users_80pct").to_string(),
            ),
            (
                "MAX_RPS_80",
                format!("{:.1}", number(sim_data, "max_sustainable_rps_80pct")),
            ),
            (
                "OPT_SAVING_PCT",
                format!("{:.1}", number(sim_data, "estimated_opt_saving_pct")),
            ),
            ("CPU_CORES", integer(sim_data, "cpu_cores").to_string()),
            ("SIMULATION_SECTION", format_multi_user_simulation_section(sim_data)),
        ],
    );
    fs::write(path, html)
}

pub fn format_php_ext_candidates_section(candidates: &Value) -> String {
    let Some(list) = candidates.as_array() else {
        return String::new();
    };
    if list.is_empty() {
        return String::new();
    }

    let mut rows = String::new();
    for (idx, item) in list.iter().enumerate() {
        let loop_calls = integer(item, "loop_invocations");
        let loop_badge = if loop_calls > 0 {
            format!(
                r#" <span style="background:#fee2e2;color:#991b1b;padding:1px 6px;border-radius:4px;font-size:11px;font-weight:700;">{}x in-loop</span>"#,
                loop_calls
            )
        } else {
            String::new()
        };

        rows.push_str(&format!(
            r#"<tr>
                <td style="text-align:center;font-weight:600;">{}</td>
                <td style="font-family:monospace;font-weight:700;color:#4f46e5;">{}()</td>
                <td style="text-align:center;"><strong>{}x</strong> ({} file){}</td>
                <td style="text-align:center;">{}x (CCN {})</td>
                <td><span style="background:#eef2ff;color:#3730a3;padding:2px 8px;border-radius:4px;font-size:12px;font-weight:600;">{}</span></td>
                <td style="color:#059669;font-weight:700;">{}</td>
                <td style="font-size:12px;">{}</td>
            </tr>"#,
            idx + 1,
            text(item, "function_name"),
            integer(item, "call_count"),
            integer(item, "caller_files"),
            loop_badge,
            integer(item, "definition_count"),
            integer(item, "max_ccn"),
            text(item, "workload_kind"),
            text(item, "estimated_cpu_saving"),
            text(item, "recommendation")
        ));
    }

    format!(
        r#"<div class="section-title" style="border-left-color:#6366f1;">⚡ Kandidat Optimasi Native PHP Extension (Hot &amp; Repeated Functions)</div>
        <div class="desc-box" style="background:#eef2ff;border:1px solid #c7d2fe;color:#312e81;">
            <h3 style="color:#1e1b4b;">🦀 Rekomendasi Kompilasi Fungsi PHP Berulang ke Native Extension</h3>
            <p style="margin:0;">Ditemukan <strong>{}</strong> fungsi/method PHP yang dipanggil berulang kali lintas berkas, dieksekusi di dalam perulangan (<em>hot loop</em>), atau diimplementasikan berulang di beberapa kelas. Mengonversi fungsi komputasi ini menjadi <strong>Native PHP Extension</strong> menggunakan <code>ext-php-rs</code> (Rust), <code>Zend API</code> (C), atau <code>Zephir</code> dapat memangkas overhead interpretasi opcode Zend VM sebesar <strong>40% – 75%</strong>.</p>
        </div>
        <table>
            <thead>
                <tr>
                    <th style="width:5%;text-align:center;">#</th>
                    <th style="width:16%;">Nama Fungsi</th>
                    <th style="width:14%;text-align:center;">Frekuensi Panggil</th>
                    <th style="width:11%;text-align:center;">Definisi &amp; CCN</th>
                    <th style="width:15%;">Karakteristik Beban</th>
                    <th style="width:14%;">Estimasi Penghematan</th>
                    <th style="width:25%;">💡 Saran Implementasi PHP Extension</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>"#,
        list.len(),
        rows
    )
}

pub fn format_multi_user_simulation_section(sim: &Value) -> String {
    if sim.is_null() || !sim["tiers"].is_array() {
        return String::new();
    }

    let e_req = number(sim, "baseline_energy_j_per_req");
    let cpu_req = number(sim, "baseline_cpu_s_per_req");
    let lat_req = number(sim, "baseline_latency_ms");
    let cores = integer(sim, "cpu_cores").max(1);
    let rpu = integer(sim, "requests_per_user").max(1);
    let win = number(sim, "concurrency_window_sec").max(1.0);
    let peak_h = number(sim, "peak_hours_per_day").max(1.0);
    let opt_pct = number(sim, "estimated_opt_saving_pct");
    let opt_driver = text(sim, "optimization_driver");
    let source_mode = text(sim, "source_mode");
    let max_users = integer(sim, "max_concurrent_users_80pct");
    let max_rps = number(sim, "max_sustainable_rps_80pct");

    let mut tier_rows = String::new();
    if let Some(tiers) = sim["tiers"].as_array() {
        for t in tiers {
            let status = text(t, "capacity_status");
            let badge_style = if status.starts_with("OPTIMAL") {
                "background:#ecfdf5;color:#065f46;"
            } else if status.starts_with("MODERAT") {
                "background:#fef3c7;color:#92400e;"
            } else if status.starts_with("TINGGI") {
                "background:#ffedd5;color:#9a3412;"
            } else {
                "background:#fee2e2;color:#991b1b;"
            };

            tier_rows.push_str(&format!(
                r#"<tr>
                    <td style="text-align:right;font-weight:700;">{} user</td>
                    <td style="text-align:right;">{:.1} req/s</td>
                    <td style="text-align:right;font-weight:600;">{:.1}% ({} vCPU ideal)</td>
                    <td style="text-align:right;">{:.1} ms</td>
                    <td style="text-align:right;font-weight:600;color:#059669;">{:.3} kWh</td>
                    <td style="text-align:right;">{:.3} kg</td>
                    <td style="text-align:right;">Rp {:.0}</td>
                    <td style="text-align:right;color:#4f46e5;font-weight:600;">-{:.3} kWh</td>
                    <td style="text-align:center;"><span style="padding:3px 8px;border-radius:6px;font-size:11px;font-weight:700;{}">{}</span></td>
                </tr>"#,
                integer(t, "concurrent_users"),
                number(t, "throughput_rps"),
                number(t, "cpu_utilization_pct"),
                integer(t, "required_cpu_cores"),
                number(t, "projected_p95_latency_ms"),
                number(t, "monthly_kwh"),
                number(t, "monthly_co2_kg"),
                number(t, "monthly_cost_idr"),
                number(t, "saved_monthly_kwh"),
                badge_style,
                status
            ));
        }
    }

    let mut rec_html = String::new();
    if let Some(recs) = sim["capacity_recommendations"].as_array() {
        for r in recs {
            rec_html.push_str(&format!("<li>{}</li>", r.as_str().unwrap_or("")));
        }
    }

    format!(
        r#"<div class="section-title" style="border-left-color:#0ea5e9;">🚀 Simulasi Skala Multi-User &amp; Proyeksi Kapasitas Server</div>
        <div class="desc-box" style="background:#f0f9ff;border:1px solid #bae6fd;color:#0c4a6e;">
            <h3 style="color:#075985;">📊 Proyeksi Beban Konkurensi, Energi, Karbon &amp; Antrean Latensi ({source_mode})</h3>
            <p style="margin:0 0 8px 0;">Baseline Audit: <strong>{e_req:.5} J/req</strong> | CPU Aktif: <strong>{cpu_req:.4} s/req</strong> | Latensi Dasar: <strong>{lat_req:.1} ms</strong> | Kapasitas Aman (80% dari {cores} vCPU): <strong>~{max_users} user konkuren ({max_rps:.1} RPS)</strong>.</p>
            <ul style="margin:6px 0 0 0;">{rec_html}</ul>
        </div>

        <div style="background:#f8fafc;border:1px solid #cbd5e1;border-radius:10px;padding:18px 20px;margin-bottom:20px;">
            <div style="font-weight:700;font-size:15px;color:#0f172a;margin-bottom:12px;">🎛️ Kalkulator Simulasi Multi-User Interaktif (Real-Time)</div>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:14px;align-items:end;">
                <div>
                    <label style="display:block;font-size:12px;font-weight:600;color:#475569;margin-bottom:4px;">Jumlah User Konkuren</label>
                    <input id="gpSimUsers" type="number" min="1" max="1000000" value="2500" style="width:100%;box-sizing:border-box;padding:8px 10px;border:1px solid #94a3b8;border-radius:6px;font-size:14px;font-weight:600;" oninput="gpUpdateSim()" />
                </div>
                <div>
                    <label style="display:block;font-size:12px;font-weight:600;color:#475569;margin-bottom:4px;">Request / User (per {win:.0}s)</label>
                    <input id="gpSimRpu" type="number" min="1" max="500" value="{rpu}" style="width:100%;box-sizing:border-box;padding:8px 10px;border:1px solid #94a3b8;border-radius:6px;font-size:14px;font-weight:600;" oninput="gpUpdateSim()" />
                </div>
                <div>
                    <label style="display:block;font-size:12px;font-weight:600;color:#475569;margin-bottom:4px;">Jumlah Core Server (vCPU)</label>
                    <input id="gpSimCores" type="number" min="1" max="256" value="{cores}" style="width:100%;box-sizing:border-box;padding:8px 10px;border:1px solid #94a3b8;border-radius:6px;font-size:14px;font-weight:600;" oninput="gpUpdateSim()" />
                </div>
                <div>
                    <label style="display:flex;align-items:center;gap:8px;font-size:12px;font-weight:700;color:#4338ca;cursor:pointer;padding-bottom:8px;">
                        <input id="gpSimOpt" type="checkbox" onchange="gpUpdateSim()" style="width:16px;height:16px;" />
                        Terapkan Optimasi (-{opt_pct:.0}% CPU/Energi)
                    </label>
                </div>
            </div>
            <div style="font-size:11px;color:#64748b;margin-top:6px;">Faktor optimasi terdeteksi: <em>{opt_driver}</em></div>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:12px;margin-top:14px;">
                <div style="background:white;border:1px solid #e2e8f0;border-radius:8px;padding:12px;text-align:center;">
                    <div style="font-size:11px;color:#64748b;font-weight:600;">THROUGHPUT</div>
                    <div id="gpOutRps" style="font-size:19px;font-weight:700;color:#0f172a;margin-top:4px;">-</div>
                </div>
                <div style="background:white;border:1px solid #e2e8f0;border-radius:8px;padding:12px;text-align:center;">
                    <div style="font-size:11px;color:#64748b;font-weight:600;">UTILISASI CPU</div>
                    <div id="gpOutCpu" style="font-size:19px;font-weight:700;color:#2563eb;margin-top:4px;">-</div>
                </div>
                <div style="background:white;border:1px solid #e2e8f0;border-radius:8px;padding:12px;text-align:center;">
                    <div style="font-size:11px;color:#64748b;font-weight:600;">PROYEKSI LATENSI P95</div>
                    <div id="gpOutLat" style="font-size:19px;font-weight:700;color:#d97706;margin-top:4px;">-</div>
                </div>
                <div style="background:white;border:1px solid #e2e8f0;border-radius:8px;padding:12px;text-align:center;">
                    <div style="font-size:11px;color:#64748b;font-weight:600;">ENERGI / BULAN</div>
                    <div id="gpOutKwh" style="font-size:19px;font-weight:700;color:#059669;margin-top:4px;">-</div>
                </div>
                <div style="background:white;border:1px solid #e2e8f0;border-radius:8px;padding:12px;text-align:center;">
                    <div style="font-size:11px;color:#64748b;font-weight:600;">EMISI KARBON / BULAN</div>
                    <div id="gpOutCo2" style="font-size:19px;font-weight:700;color:#7c3aed;margin-top:4px;">-</div>
                </div>
            </div>
        </div>

        <table>
            <thead>
                <tr>
                    <th style="text-align:right;">Skala User</th>
                    <th style="text-align:right;">Throughput</th>
                    <th style="text-align:right;">Utilisasi CPU</th>
                    <th style="text-align:right;">Latensi P95</th>
                    <th style="text-align:right;">Energi / Bulan</th>
                    <th style="text-align:right;">Emisi CO2e</th>
                    <th style="text-align:right;">Estimasi Biaya</th>
                    <th style="text-align:right;">Hemat (PHP Ext)</th>
                    <th style="text-align:center;">Status Server</th>
                </tr>
            </thead>
            <tbody>
                {tier_rows}
            </tbody>
        </table>
        <script>
        function gpUpdateSim() {{
            var users = Math.max(1, parseFloat(document.getElementById('gpSimUsers').value) || 1);
            var rpu = Math.max(1, parseFloat(document.getElementById('gpSimRpu').value) || 1);
            var cores = Math.max(1, parseFloat(document.getElementById('gpSimCores').value) || 1);
            var opt = document.getElementById('gpSimOpt').checked;
            var factor = opt ? (1.0 - ({opt_pct} / 100.0)) : 1.0;
            var eReq = {e_req} * factor;
            var cpuReq = {cpu_req} * factor;
            var baseLat = {lat_req} * (opt ? (0.65 + 0.35 * factor) : 1.0);
            var win = {win};
            var totalReqs = users * rpu;
            var rps = totalReqs / win;
            var rho = (rps * cpuReq) / cores;
            var utilPct = rho * 100.0;
            var p95 = rho < 0.98 ? baseLat * (1.25 + (rho * rho) / (2.0 * (1.0 - rho))) : baseLat * (25.0 + (rho - 0.98) * 120.0);
            var windowsPerDay = ({peak_h} * 3600.0) / win;
            var monthlyKwh = ((totalReqs * eReq * windowsPerDay) / 3600000.0) * 30.0;
            var monthlyCo2Kg = (monthlyKwh * 850.0) / 1000.0;
            document.getElementById('gpOutRps').textContent = rps.toFixed(1) + ' req/s';
            var cpuEl = document.getElementById('gpOutCpu');
            cpuEl.textContent = utilPct.toFixed(1) + '%';
            cpuEl.style.color = utilPct <= 75 ? '#059669' : (utilPct <= 100 ? '#d97706' : '#dc2626');
            document.getElementById('gpOutLat').textContent = p95.toFixed(1) + ' ms';
            document.getElementById('gpOutKwh').textContent = monthlyKwh.toFixed(3) + ' kWh';
            document.getElementById('gpOutCo2').textContent = monthlyCo2Kg.toFixed(3) + ' kg';
        }}
        gpUpdateSim();
        </script>"#
    )
}
