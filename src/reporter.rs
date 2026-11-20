//! HTML and JSON Report Generator in Rust
//! Menghasilkan laporan evaluasi bersih, profesional, edukatif, dan print-friendly untuk lampiran skripsi/magang

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct ReportGenerator;

pub const LOGO_BASE64: &str = include_str!("logo_base64.txt");

impl ReportGenerator {
    pub fn save_json<P: AsRef<Path>>(data: &Value, path: P) -> std::io::Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(data)?;
        fs::write(p, json_str)
    }

    pub fn generate_html<P: AsRef<Path>>(report_data: &Value, path: P) -> std::io::Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }

        let meta = &report_data["metadata"];
        let totals = &report_data["totals"];
        let correlated = report_data["correlated_modules"].as_array();
        let endpoints = report_data["endpoints_detail"].as_array();
        let code_quality = &report_data["code_quality"];
        let quality = &code_quality["overview"];

        let total_joules = totals["total_joules"].as_f64().unwrap_or(0.0);
        let total_kwh = totals["total_kwh"].as_f64().unwrap_or(0.0);
        let total_cpu_active_s = totals["total_cpu_active_s"].as_f64().unwrap_or(0.0);
        let total_co2e_g = totals["total_co2e_g"].as_f64().unwrap_or(0.0);

        let kwh_formatted = if total_kwh < 0.0001 && total_kwh > 0.0 {
            format!("{:.2e} kWh", total_kwh)
        } else {
            format!("{:.4} kWh", total_kwh)
        };

        let mut rows_corr = String::new();
        if let Some(list) = correlated {
            for m in list {
                let energy = m["total_energy_joules"].as_f64().unwrap_or(0.0);
                let score = m["green_efficiency_score"].as_f64().unwrap_or(0.0);
                let avg_latency = m["avg_latency_ms"].as_f64().unwrap_or(0.0);
                let avg_cpu = m["avg_cpu_active_s"].as_f64().unwrap_or(0.0);
                let total_calls = m["total_calls"].as_u64().unwrap_or(0);

                // Pewarnaan dinamis untuk Energi (Joule)
                let energy_badge = if energy > 10.0 {
                    format!(r#"<span style="background:#fee2e2;color:#991b1b;padding:3px 8px;border-radius:6px;font-weight:700;">{:.3} J (Tinggi)</span>"#, energy)
                } else if energy > 2.0 {
                    format!(r#"<span style="background:#fef3c7;color:#92400e;padding:3px 8px;border-radius:6px;font-weight:600;">{:.3} J (Sedang)</span>"#, energy)
                } else {
                    format!(r#"<span style="background:#ecfdf5;color:#065f46;padding:3px 8px;border-radius:6px;font-weight:600;">{:.3} J (Efisien)</span>"#, energy)
                };

                // Pewarnaan dinamis untuk Green Score
                let score_badge = if score >= 85.0 {
                    format!(r#"<span style="background:#ecfdf5;color:#065f46;padding:3px 8px;border-radius:12px;font-weight:bold;">{:.1} (Optimal)</span>"#, score)
                } else if score >= 60.0 {
                    format!(r#"<span style="background:#fef3c7;color:#92400e;padding:3px 8px;border-radius:12px;font-weight:bold;">{:.1} (Cukup)</span>"#, score)
                } else {
                    format!(r#"<span style="background:#fee2e2;color:#991b1b;padding:3px 8px;border-radius:12px;font-weight:bold;">{:.1} (Boros)</span>"#, score)
                };

                // Latency color coding
                let latency_color = if avg_latency > 1000.0 {
                    "#dc2626"  // Red > 1s
                } else if avg_latency > 200.0 {
                    "#d97706"  // Yellow 200ms - 1s
                } else {
                    "#059669"  // Green < 200ms
                };
                let latency_label = if avg_latency > 1000.0 {
                    "Lambat"
                } else if avg_latency > 200.0 {
                    "Sedang"
                } else {
                    "Cepat"
                };

                // Recommendation based on metrics
                let recommendation = if score < 60.0 && energy > 5.0 {
                    "Refactor kritis: Optimasi query + cache + kurangi kompleksitas"
                } else if score < 85.0 && avg_latency > 500.0 {
                    "Optimasi DB index + gunakan query builder + pagination"
                } else if avg_latency > 1000.0 {
                    "Investigasi N+1 query, tambah Redis cache, async processing"
                } else if energy > 2.0 {
                    "Aktifkan OPcache JIT, kurangi middleware berlebih"
                } else if score < 60.0 {
                    "Perbaiki arsitektur: pecah God Class, kurangi nesting"
                } else {
                    "✅ Sudah optimal"
                };

                rows_corr.push_str(&format!(
                    r#"<tr>
                        <td style="font-family:monospace;font-size:13px;">{}</td>
                        <td style="text-align:center;">{}</td>
                        <td style="text-align:right;color:{};font-weight:600;">{:.2} ms ({})</td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:right;">{:.3} s</td>
                        <td style="text-align:center;">{}</td>
                        <td style="font-size:12px;max-width:300px;">{}</td>
                    </tr>"#,
                    m["module_name"].as_str().unwrap_or(""),
                    total_calls,
                    latency_color,
                    avg_latency,
                    latency_label,
                    energy_badge,
                    avg_cpu,
                    score_badge,
                    recommendation
                ));
            }
        }

        let mut rows_ep = String::new();
        if let Some(list) = endpoints {
            for (i, ep) in list.iter().enumerate() {
                let status = ep["status"].as_u64().unwrap_or(0);
                let status_badge = if status == 200 {
                    format!(r#"<span style="background:#ecfdf5;color:#065f46;padding:2px 6px;border-radius:4px;font-weight:bold;">{}</span>"#, status)
                } else {
                    format!(r#"<span style="background:#fee2e2;color:#991b1b;padding:2px 6px;border-radius:4px;font-weight:bold;">{}</span>"#, status)
                };

                let ep_energy = ep["energy_total_joules"].as_f64().unwrap_or(0.0);
                let ep_latency = ep["latency_ms"].as_f64().unwrap_or(0.0);
                let ep_cpu = ep["cpu_active_time_s"].as_f64().unwrap_or(0.0);

                let ep_energy_color = if ep_energy > 5.0 {
                    "#dc2626"
                } else if ep_energy > 1.0 {
                    "#d97706"
                } else {
                    "#059669"
                };

                // Latency color coding
                let latency_color = if ep_latency > 1000.0 {
                    "#dc2626"
                } else if ep_latency > 200.0 {
                    "#d97706"
                } else {
                    "#059669"
                };
                let latency_label = if ep_latency > 1000.0 {
                    "🔴 Lambat"
                } else if ep_latency > 200.0 {
                    "🟡 Sedang"
                } else {
                    "🟢 Cepat"
                };

                // Recommendation per endpoint
                let recommendation = if ep_latency > 1000.0 && ep_energy > 5.0 {
                    "URGENT: Query N+1 + no cache + boros energi"
                } else if ep_latency > 1000.0 {
                    "Optimasi DB index + cache Redis + async job"
                } else if ep_energy > 5.0 {
                    "Aktifkan OPcache JIT, kurangi overhead PHP"
                } else if ep_latency > 200.0 {
                    "Pagination + eager loading + query optimization"
                } else {
                    "✅ Optimal"
                };

                rows_ep.push_str(&format!(
                    r#"<tr>
                        <td style="text-align:center;">{}</td>
                        <td style="font-weight:600;color:#2563eb;">{}</td>
                        <td style="font-family:monospace;font-size:12px;">{}</td>
                        <td style="text-align:center;">{}</td>
                        <td style="text-align:right;color:{};font-weight:600;">{:.2} ms ({})</td>
                        <td style="text-align:right;">{:.3} s</td>
                        <td style="text-align:right;font-weight:600;color:{};">{:.4} J</td>
                        <td style="font-size:12px;max-width:280px;">{}</td>
                    </tr>"#,
                    i + 1,
                    ep["method"].as_str().unwrap_or("GET"),
                    ep["endpoint"].as_str().unwrap_or(""),
                    status_badge,
                    latency_color,
                    ep_latency,
                    latency_label,
                    ep_cpu,
                    ep_energy_color,
                    ep_energy,
                    recommendation
                ));
            }
        }

        let smells = code_quality["code_smells"].as_array();
        let mut rows_smells = String::new();
        if let Some(list) = smells {
            for s in list.iter().take(20) {
                let badge_color = match s["severity"].as_str().unwrap_or("") {
                    "High" => "#ef4444",
                    "Medium" => "#f59e0b",
                    _ => "#3b82f6",
                };
                let smell_type = s["smell_type"].as_str().unwrap_or("");
                let message = s["message"].as_str().unwrap_or("");
                let file = s["file"].as_str().unwrap_or("");
                let line = s["line"].as_u64().unwrap_or(0);

                // Recommendation based on smell type
                let recommendation = match smell_type {
                    "GodClass" => "Pecah class jadi beberapa class kecil (SRP), extract service",
                    "FatController" => "Pindah logika bisnis ke Service/Action, controller hanya handle HTTP",
                    "DeepNesting" => "Early return, guard clause, extract method, flatten conditional",
                    "LongMethod" => "Extract method, gunakan composer pattern, kurangi tanggung jawab",
                    "LongParameterList" => "Gunakan DTO/Parameter Object, builder pattern",
                    "DuplicateCode" => "Extract common logic ke helper/trait, DRY principle",
                    "DeadCode" => "Hapus kode tidak terpakai, gunakan static analysis",
                    _ => "Refactor sesuai pola: SRP, KISS, DRY, Clean Architecture",
                };

                rows_smells.push_str(&format!(
                    r#"<tr>
                        <td style="text-align:center;"><span style="background:{};color:white;padding:2px 8px;border-radius:4px;font-size:11px;font-weight:bold;">{}</span></td>
                        <td style="font-weight:600;">{}</td>
                        <td>{}</td>
                        <td style="font-family:monospace;font-size:12px;">{}:{}</td>
                        <td style="font-size:12px;max-width:300px;">{}</td>
                    </tr>"#,
                    badge_color,
                    s["severity"].as_str().unwrap_or(""),
                    smell_type,
                    message,
                    file,
                    line,
                    recommendation
                ));
            }
        }

        let files = code_quality["files"].as_array();
        let mut rows_files = String::new();
        if let Some(list) = files {
            for f in list.iter().take(25) {
                let ccn = f["ccn"].as_u64().unwrap_or(0);
                let ncloc = f["ncloc"].as_u64().unwrap_or(0);
                let mi = f["maintainability_index"].as_f64().unwrap_or(0.0);
                let smell_count = f["code_smells"].as_array().map(|a| a.len()).unwrap_or(0);

                let ccn_badge = if ccn > 20 {
                    format!(r#"<span style="color:#dc2626;font-weight:bold;">{} (Tinggi)</span>"#, ccn)
                } else if ccn > 10 {
                    format!(r#"<span style="color:#d97706;font-weight:600;">{} (Sedang)</span>"#, ccn)
                } else {
                    format!(r#"<span style="color:#059669;">{} (Aman)</span>"#, ccn)
                };

                let mi_badge = if mi >= 85.0 {
                    format!(r#"<span style="color:#059669;font-weight:bold;">{:.1} (A)</span>"#, mi)
                } else if mi >= 65.0 {
                    format!(r#"<span style="color:#d97706;font-weight:bold;">{:.1} (B)</span>"#, mi)
                } else {
                    format!(r#"<span style="color:#dc2626;font-weight:bold;">{:.1} (C)</span>"#, mi)
                };

                let smell_badge = if smell_count > 0 {
                    format!(r#"<span style="background:#fee2e2;color:#991b1b;padding:2px 6px;border-radius:4px;font-weight:bold;">{}</span>"#, smell_count)
                } else {
                    r#"<span style="color:#94a3b8;">0</span>"#.to_string()
                };

                // Recommendation based on file metrics
                let recommendation = if ccn > 20 && mi < 65.0 {
                    "URGENT: Refactor besar - pecah class, extract method, kurangi kompleksitas"
                } else if ccn > 20 {
                    "Extract methods, kurangi percabangan, gunakan strategy pattern"
                } else if mi < 65.0 && ncloc > 300 {
                    "Modularisasi file besar, pecah per tanggung jawab (SRP)"
                } else if mi < 65.0 {
                    "Perbaiki naming, kurangi nesting, tambah dokumentasi"
                } else if smell_count > 3 {
                    "Bersihkan code smells: hapus dead code, dedup, early return"
                } else if ccn > 10 {
                    "Pertimbangkan extract method untuk logic kompleks"
                } else {
                    "✅ Bersih & maintainable"
                };

                rows_files.push_str(&format!(
                    r#"<tr>
                        <td style="font-family:monospace;font-size:12px;">{}</td>
                        <td style="text-align:center;"><span style="background:#f1f5f9;padding:2px 6px;border-radius:4px;">{}</span></td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:center;">{}</td>
                        <td style="font-size:12px;max-width:300px;">{}</td>
                    </tr>"#,
                    f["file_path"].as_str().unwrap_or(""),
                    f["language"].as_str().unwrap_or(""),
                    ncloc,
                    ccn_badge,
                    mi_badge,
                    smell_badge,
                    recommendation
                ));
            }
        }

        let php_env = &report_data["php_environment"];
        let mut php_section_html = String::new();
        if !php_env.is_null() && php_env["is_php_available"].as_bool().unwrap_or(false) {
            let mut ext_rows = String::new();
            if let Some(exts) = php_env["extension_details"].as_array() {
                for e in exts {
                    let badge_style = match e["status_badge"].as_str().unwrap_or("INFO") {
                        "OK" => "background:#10b981;color:white;",
                        "WARN" => "background:#f59e0b;color:white;",
                        "DANGER" => "background:#ef4444;color:white;",
                        _ => "background:#64748b;color:white;",
                    };
                    ext_rows.push_str(&format!(
                        r#"<tr>
                            <td style="font-weight:600;">{}</td>
                            <td style="text-align:center;"><span style="padding:2px 8px;border-radius:4px;font-size:11px;font-weight:bold;{}">{}</span></td>
                            <td>{}</td>
                            <td>{}</td>
                        </tr>"#,
                        e["name"].as_str().unwrap_or(""),
                        badge_style,
                        e["status_badge"].as_str().unwrap_or(""),
                        e["description"].as_str().unwrap_or(""),
                        e["green_impact"].as_str().unwrap_or("")
                    ));
                }
            }

            let mut rec_items = String::new();
            if let Some(recs) = php_env["recommendations"].as_array() {
                for r in recs {
                    rec_items.push_str(&format!("<li>{}</li>", r.as_str().unwrap_or("")));
                }
            }

            php_section_html = format!(
                r#"<div class="section-title" style="border-left-color:#8b5cf6;">6. Audit PHP Runtime & Green Extensions</div>
                <div class="desc-box" style="background:#f5f3ff;border-color:#ddd6fe;color:#5b21b6;">
                    <h3>⚡ Efisiensi Energi Konfigurasi PHP (PHP Extension Health)</h3>
                    <p style="margin:0 0 10px 0;">Versi Server: <strong>{}</strong> | Ekstensi Terpasang: <strong>{}</strong></p>
                    <p style="margin:0;">Dalam prinsip Green Software Engineering, penghematan energi server PHP sangat dipengaruhi oleh penggunaan opcode caching (OPcache) dan ketiadaan overhead profiling debug (Xdebug).</p>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th style="width:24%;">Nama Ekstensi</th>
                            <th style="width:12%;text-align:center;">Status</th>
                            <th style="width:30%;">Fungsi</th>
                            <th>Dampak Efisiensi Energi (Green Computing)</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
                {}"#,
                php_env["php_version"].as_str().unwrap_or("PHP"),
                php_env["loaded_extensions_count"].as_u64().unwrap_or(0),
                ext_rows,
                if !rec_items.is_empty() {
                    format!(r#"<div class="desc-box" style="background:#fffbeb;border-color:#fef3c7;color:#92400e;margin-top:15px;"><h3>💡 Rekomendasi Optimasi Runtime PHP</h3><ul>{}</ul></div>"#, rec_items)
                } else {
                    "".to_string()
                }
            );
        }

        // Banner filter aktif (jika ada) + kartu sebaran bahasa (bukti cakupan FE+BE)
        let filter_banner = build_filter_banner(&report_data["applied_filter"]);
        let lang_cards = build_language_cards(&quality["language_breakdown"]);

        let html = format!(
            r##"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GreenCode Profiler v1.7.2 (Rust Native Engine) Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; background: #f8fafc; color: #1e293b; margin: 0; padding: 30px; }}
        .container {{ max-width: 1100px; margin: 0 auto; background: #ffffff; border-radius: 12px; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1); padding: 40px; border: 1px solid #e2e8f0; }}
        .header {{ display: flex; justify-content: space-between; align-items: center; border-bottom: 2px solid #10b981; padding-bottom: 20px; margin-bottom: 30px; }}
        .header-brand {{ display: flex; align-items: center; gap: 18px; }}
        .logo-img {{ width: 68px; height: 68px; object-fit: contain; filter: drop-shadow(0 2px 4px rgba(0,0,0,0.08)); }}
        .badge {{ background: #10b981; color: white; padding: 4px 12px; border-radius: 20px; font-size: 13px; font-weight: 600; }}
        .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 20px; margin-bottom: 35px; }}
        .card {{ background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 20px; text-align: center; }}
        .card .val {{ font-size: 26px; font-weight: 700; color: #0f172a; margin-top: 8px; }}
        .card .lbl {{ font-size: 13px; color: #64748b; font-weight: 500; text-transform: uppercase; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 15px; font-size: 14px; }}
        th, td {{ border: 1px solid #cbd5e1; padding: 10px 14px; }}
        th {{ background: #f1f5f9; color: #334155; font-weight: 600; text-align: left; }}
        tr:nth-child(even) {{ background: #f8fafc; }}
        .section-title {{ font-size: 18px; font-weight: 700; color: #0f172a; margin-top: 35px; margin-bottom: 15px; border-left: 4px solid #10b981; padding-left: 10px; }}
        .section-title.blue {{ border-left-color: #3b82f6; }}
        .desc-box {{ background: #f0fdf4; border: 1px solid #bbf7d0; border-radius: 8px; padding: 18px 22px; margin-bottom: 30px; line-height: 1.6; font-size: 14px; color: #166534; }}
        .desc-box.blue {{ background: #eff6ff; border-color: #bfdbfe; color: #1e40af; margin-top: 15px; }}
        .desc-box h3 {{ margin-top: 0; margin-bottom: 10px; font-size: 16px; color: #14532d; }}
        .desc-box.blue h3 {{ color: #1e3a8a; }}
        .desc-box ul {{ margin: 0; padding-left: 20px; }}
        .desc-box li {{ margin-bottom: 8px; }}
        .desc-box strong {{ color: #0f172a; }}
        .footer {{ margin-top: 40px; text-align: center; font-size: 12px; color: #94a3b8; border-top: 1px solid #e2e8f0; padding-top: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="header-brand">
                <img src="data:image/png;base64,{}" class="logo-img" alt="GreenCode Logo" />
                <div>
                    <h1 style="margin:0;font-size:24px;color:#0f172a;">GreenCode Profiler Report</h1>
                    <p style="margin:5px 0 0 0;color:#64748b;">Project: <strong>{}</strong> | Engine: <strong>Rust Native Profiler v1.2.0</strong></p>
                </div>
            </div>
            <div>
                <span class="badge">GREEN AUDIT VERIFIED</span>
                <p style="margin:5px 0 0 0;font-size:12px;color:#64748b;text-align:right;">{}</p>
            </div>
        </div>
        __FILTER_BANNER__

        <div class="grid">
            <div class="card">
                <div class="lbl">Total Energi Murni</div>
                <div class="val" style="color:#059669;">{:.2} J</div>
                <div style="font-size:12px;color:#64748b;margin-top:4px;">({})</div>
            </div>
            <div class="card">
                <div class="lbl">Waktu Aktif CPU Murni</div>
                <div class="val" style="color:#2563eb;">{:.3} s</div>
                <div style="font-size:12px;color:#64748b;margin-top:4px;">Bebas I/O Wait Bias</div>
            </div>
            <div class="card">
                <div class="lbl">Jejak Karbon Bersih</div>
                <div class="val" style="color:#7c3aed;">{:.4} g</div>
                <div style="font-size:12px;color:#64748b;margin-top:4px;">CO2e (Grid 850g/kWh)</div>
            </div>
            <div class="card">
                <div class="lbl">Maintainability Index</div>
                <div class="val" style="color:#ea580c;">{}/100</div>
                <div style="font-size:12px;color:#64748b;margin-top:4px;">{}</div>
            </div>
        </div>

        <div class="desc-box">
            <h3>📖 Panduan & Penjelasan Parameter Profiling Energi Dinamis</h3>
            <ul>
                <li><strong>Total Energi Murni (Joule & kWh):</strong> Total daya listrik aktual yang ditarik CPU selama memproses transaksi ($E = P_{{avg}} \times t_{{active}}$). 
                    <br><span style="display:inline-block;margin-top:2px;font-size:12px;">🔴 <em>Merah (&gt; 10 J): Konsumsi tinggi / berisiko hotspot</em> | 🟡 <em>Kuning (2 - 10 J): Konsumsi moderat</em> | 🟢 <em>Hijau (&lt; 2 J): Sangat efisien</em></span>
                </li>
                <li><strong>Waktu Aktif CPU Murni (Detik):</strong> Durasi thread CPU aktif mengeksekusi instruksi kode program ($t_{{user}} + t_{{system}}$ dari kernel). <strong>Kebal terhadap keterlambatan network I/O, waiting query database, atau koneksi lambat</strong>.</li>
                <li><strong>Jejak Karbon Bersih (gCO2e):</strong> Estimasi emisi gas rumah kaca berbasis faktor emisi pembangkit listrik Indonesia (~850 gCO2e/kWh).</li>
                <li><strong>Green Score (0 - 100):</strong> Indeks rasio efisiensi energi terhadap latensi dan throughput endpoint.
                    <br><span style="display:inline-block;margin-top:2px;font-size:12px;">🟢 <em>Hijau (≥ 85): Optimal</em> | 🟡 <em>Kuning (60 - 84): Cukup</em> | 🔴 <em>Merah (&lt; 60): Boros energi (perlu refactoring)</em></span>
                </li>
            </ul>
        </div>

        <div class="section-title">1. Ringkasan Efisiensi Energi per Modul</div>
        <table>
            <thead>
                <tr>
                    <th>Endpoint / URL Target</th>
                    <th style="text-align:center;">Jumlah Request</th>
                    <th style="text-align:right;">Rata-rata Latensi</th>
                    <th style="text-align:right;">Total Energi (Joule)</th>
                    <th style="text-align:right;">Waktu CPU Murni</th>
                    <th style="text-align:center;">Green Score</th>
                    <th>💡 Saran Optimasi</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="section-title">2. Detail Pengukuran Transaksi HTTP</div>
        <table>
            <thead>
                <tr>
                    <th style="text-align:center;">#</th>
                    <th>Method</th>
                    <th>Target URL</th>
                    <th style="text-align:center;">HTTP Status</th>
                    <th style="text-align:right;">Latensi</th>
                    <th style="text-align:right;">CPU Aktif</th>
                    <th style="text-align:right;">Energi Bersih</th>
                    <th>💡 Saran Optimasi</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="section-title blue">3. Ringkasan Kualitas Kode & Arsitektur Statis</div>
        <div class="grid" style="margin-top: 15px;">
            <div class="card">
                <div class="lbl">Total File Discan</div>
                <div class="val">{}</div>
            </div>
            <div class="card">
                <div class="lbl">Total NCLOC</div>
                <div class="val">{}</div>
            </div>
            <div class="card">
                <div class="lbl">Rata-rata CCN</div>
                <div class="val">{}</div>
            </div>
            <div class="card">
                <div class="lbl">Code Smells Terdeteksi</div>
                <div class="val" style="color:#ef4444;">{}</div>
            </div>
            __LANG_CARDS__
        </div>

        <div class="desc-box blue">
            <h3>📐 Panduan & Penjelasan Parameter Kualitas Struktur Kode</h3>
            <ul>
                <li><strong>NCLOC (Non-Comment Lines of Code):</strong> Jumlah baris instruksi logika program yang dieksekusi tanpa menghitung baris kosong dan komentar. Semakin proporsional NCLOC per berkas (&lt; 300 baris), semakin mudah dirawat.</li>
                <li><strong>CCN (Cyclomatic Complexity Number):</strong> Jumlah jalur percabangan independen (if/else/switch/loop) dalam logika fungsi. 
                    <br><span style="display:inline-block;margin-top:2px;font-size:12px;">🟢 <em>1 - 10: Rendah & Aman</em> | 🟡 <em>11 - 20: Sedang (rawan bug)</em> | 🔴 <em>&gt; 20: Tinggi & Kompleks (wajib modularisasi)</em></span>
                </li>
                <li><strong>MI (Maintainability Index, Skala 0 - 100):</strong> Indeks komprehensif kemudahan pemeliharaan kode berbasis perpaduan logaritma Halstead Volume, CCN, dan NCLOC.
                    <br><span style="display:inline-block;margin-top:2px;font-size:12px;">🟢 <em>Grade A (≥ 85): Bersih & Sangat Mudah Dipelihara</em> | 🟡 <em>Grade B (65 - 84): Cukup</em> | 🔴 <em>Grade C (&lt; 65): Technical Debt Tinggi</em></span>
                </li>
                <li><strong>Smells (Code Smells):</strong> Deteksi gejala arsitektur buruk: <em>God Class</em> (kelas terlalu besar), <em>Fat Controller</em> (logika bertumpuk di controller), dan <em>Deep Nesting</em> (percabangan bersarang &gt; 4 level).</li>
            </ul>
        </div>

        <div class="section-title blue">4. Temuan Code Smells & Arsitektur Buruk</div>
        <table>
            <thead>
                <tr>
                    <th style="text-align:center;">Severity</th>
                    <th>Tipe Masalah</th>
                    <th>Deskripsi</th>
                    <th>Lokasi Berkas</th>
                    <th>💡 Saran Perbaikan</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="section-title blue">5. Sample Metrik per Berkas Kode</div>
        <table>
            <thead>
                <tr>
                    <th>Nama Berkas</th>
                    <th style="text-align:center;">Bahasa</th>
                    <th style="text-align:right;">NCLOC</th>
                    <th style="text-align:right;">CCN (Kompleksitas)</th>
                    <th style="text-align:right;">MI (0-100)</th>
                    <th style="text-align:center;">Smells</th>
                    <th>💡 Saran Optimasi</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        {}

        <div class="footer">
            Dihasilkan secara otomatis oleh <strong>GreenCode Profiler v1.7.2 (Rust Native Architecture)</strong>.<br>
            Laporan ini memenuhi standar audit perangkat lunak ramah lingkungan (Green Software Engineering).
        </div>
    </div>
</body>
</html>"##,
            LOGO_BASE64,
            meta["project_name"].as_str().unwrap_or("Web Project"),
            meta["timestamp"].as_str().unwrap_or(""),
            total_joules,
            kwh_formatted,
            total_cpu_active_s,
            total_co2e_g,
            quality["average_maintainability_index"],
            quality["maintainability_grade"].as_str().unwrap_or("Grade A"),
            rows_corr,
            rows_ep,
            quality["total_files"],
            quality["total_ncloc"].as_u64().unwrap_or(0),
            quality["average_ccn_per_file"],
            quality["total_code_smells"],
            rows_smells,
            rows_files,
            php_section_html
        );

        let html = html
            .replace("__FILTER_BANNER__", &filter_banner)
            .replace("__LANG_CARDS__", &lang_cards);

        fs::write(p, html)
    }

    pub fn generate_code_quality_html<P: AsRef<Path>>(report_data: &Value, path: P) -> std::io::Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }

        let ov = &report_data["overview"];
        let files = report_data["files"].as_array();
        let smells = report_data["code_smells"].as_array();

        let mut rows_files = String::new();
        if let Some(list) = files {
            for f in list {
                let ccn = f["ccn"].as_u64().unwrap_or(0);
                let ccn_badge = if ccn > 20 {
                    format!(r#"<span style="color:#dc2626;font-weight:bold;">{} (Tinggi)</span>"#, ccn)
                } else if ccn > 10 {
                    format!(r#"<span style="color:#d97706;font-weight:600;">{} (Sedang)</span>"#, ccn)
                } else {
                    format!(r#"<span style="color:#059669;">{} (Aman)</span>"#, ccn)
                };

                let mi = f["maintainability_index"].as_f64().unwrap_or(0.0);
                let mi_badge = if mi >= 85.0 {
                    format!(r#"<span style="color:#059669;font-weight:bold;">{:.1} (A)</span>"#, mi)
                } else if mi >= 65.0 {
                    format!(r#"<span style="color:#d97706;font-weight:bold;">{:.1} (B)</span>"#, mi)
                } else {
                    format!(r#"<span style="color:#dc2626;font-weight:bold;">{:.1} (C)</span>"#, mi)
                };

                let smell_count = f["code_smells"].as_array().map(|a| a.len()).unwrap_or(0);
                let smell_badge = if smell_count > 0 {
                    format!(r#"<span style="background:#fee2e2;color:#991b1b;padding:2px 6px;border-radius:4px;font-weight:bold;">{}</span>"#, smell_count)
                } else {
                    r#"<span style="color:#94a3b8;">0</span>"#.to_string()
                };

                // Recommendation based on file metrics
                let ccn = f["ccn"].as_u64().unwrap_or(0);
                let ncloc = f["ncloc"].as_u64().unwrap_or(0);
                let mi = f["maintainability_index"].as_f64().unwrap_or(0.0);
                let smell_count = f["code_smells"].as_array().map(|a| a.len()).unwrap_or(0);

                let recommendation = if ccn > 20 && mi < 65.0 {
                    "URGENT: Refactor besar - pecah class, extract method, kurangi kompleksitas"
                } else if ccn > 20 {
                    "Extract methods, kurangi percabangan, gunakan strategy pattern"
                } else if mi < 65.0 && ncloc > 300 {
                    "Modularisasi file besar, pecah per tanggung jawab (SRP)"
                } else if mi < 65.0 {
                    "Perbaiki naming, kurangi nesting, tambah dokumentasi"
                } else if smell_count > 3 {
                    "Bersihkan code smells: hapus dead code, dedup, early return"
                } else if ccn > 10 {
                    "Pertimbangkan extract method untuk logic kompleks"
                } else {
                    "✅ Bersih & maintainable"
                };

                rows_files.push_str(&format!(
                    r#"<tr>
                        <td style="font-family:monospace;font-size:12px;">{}</td>
                        <td style="text-align:center;"><span style="background:#f1f5f9;padding:2px 6px;border-radius:4px;">{}</span></td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:center;">{}</td>
                        <td style="font-size:12px;max-width:300px;">{}</td>
                    </tr>"#,
                    f["file_path"].as_str().unwrap_or(""),
                    f["language"].as_str().unwrap_or(""),
                    ncloc,
                    ccn_badge,
                    mi_badge,
                    smell_badge,
                    recommendation
                ));
            }
        }


        let mut rows_smells = String::new();
        if let Some(list) = smells {
            for s in list.iter().take(25) {
                let badge_color = match s["severity"].as_str().unwrap_or("") {
                    "High" => "#ef4444",
                    "Medium" => "#f59e0b",
                    _ => "#3b82f6",
                };
                let smell_type = s["smell_type"].as_str().unwrap_or("");
                let message = s["message"].as_str().unwrap_or("");
                let file = s["file"].as_str().unwrap_or("");
                let line = s["line"].as_u64().unwrap_or(0);

                // Recommendation based on smell type
                let recommendation = match smell_type {
                    "GodClass" => "Pecah class jadi beberapa class kecil (SRP), extract service",
                    "FatController" => "Pindah logika bisnis ke Service/Action, controller hanya handle HTTP",
                    "DeepNesting" => "Early return, guard clause, extract method, flatten conditional",
                    "LongMethod" => "Extract method, gunakan composer pattern, kurangi tanggung jawab",
                    "LongParameterList" => "Gunakan DTO/Parameter Object, builder pattern",
                    "DuplicateCode" => "Extract common logic ke helper/trait, DRY principle",
                    "DeadCode" => "Hapus kode tidak terpakai, gunakan static analysis",
                    _ => "Refactor sesuai pola: SRP, KISS, DRY, Clean Architecture",
                };

                rows_smells.push_str(&format!(
                    r#"<tr>
                        <td style="text-align:center;"><span style="background:{};color:white;padding:2px 8px;border-radius:4px;font-size:11px;font-weight:bold;">{}</span></td>
                        <td style="font-weight:600;">{}</td>
                        <td>{}</td>
                        <td style="font-family:monospace;font-size:12px;">{}:{}</td>
                        <td style="font-size:12px;max-width:300px;">{}</td>
                    </tr>"#,
                    badge_color,
                    s["severity"].as_str().unwrap_or(""),
                    smell_type,
                    message,
                    file,
                    line,
                    recommendation
                ));
            }
        }

        let filter_banner = build_filter_banner(&report_data["applied_filter"]);
        let lang_cards = build_language_cards(&ov["language_breakdown"]);

        let html = format!(
            r##"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <title>GreenCode Static Code Quality Report (Rust)</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f8fafc; color: #1e293b; margin: 0; padding: 30px; }}
        .container {{ max-width: 1100px; margin: 0 auto; background: #ffffff; border-radius: 12px; padding: 40px; border: 1px solid #e2e8f0; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1); }}
        .header {{ display: flex; justify-content: space-between; align-items: center; border-bottom: 2px solid #3b82f6; padding-bottom: 20px; margin-bottom: 30px; }}
        .header-brand {{ display: flex; align-items: center; gap: 18px; }}
        .logo-img {{ width: 68px; height: 68px; object-fit: contain; filter: drop-shadow(0 2px 4px rgba(0,0,0,0.08)); }}
        .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 25px 0; }}
        .card {{ background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 20px; text-align: center; }}
        .card .val {{ font-size: 26px; font-weight: 700; color: #0f172a; margin-top: 6px; }}
        .card .lbl {{ font-size: 12px; color: #64748b; font-weight: 600; text-transform: uppercase; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 15px; font-size: 13px; }}
        th, td {{ border: 1px solid #cbd5e1; padding: 10px 12px; }}
        th {{ background: #f1f5f9; color: #334155; font-weight: 600; text-align: left; }}
        tr:nth-child(even) {{ background: #f8fafc; }}
        .section-title {{ font-size: 18px; font-weight: 700; color: #0f172a; margin-top: 35px; margin-bottom: 12px; border-left: 4px solid #3b82f6; padding-left: 10px; }}
        .desc-box {{ background: #eff6ff; border: 1px solid #bfdbfe; border-radius: 8px; padding: 18px 22px; margin-bottom: 25px; line-height: 1.6; font-size: 14px; color: #1e40af; }}
        .desc-box h3 {{ margin-top: 0; margin-bottom: 10px; font-size: 16px; color: #1e3a8a; }}
        .desc-box ul {{ margin: 0; padding-left: 20px; }}
        .desc-box li {{ margin-bottom: 8px; }}
        .desc-box strong {{ color: #0f172a; }}
        .footer {{ margin-top: 40px; text-align: center; font-size: 12px; color: #94a3b8; border-top: 1px solid #e2e8f0; padding-top: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="header-brand">
                <img src="data:image/png;base64,{}" class="logo-img" alt="GreenCode Logo" />
                <div>
                    <h1 style="margin:0;font-size:24px;color:#0f172a;">Static Code Quality & Architecture Report</h1>
                    <p style="margin:5px 0 0 0;color:#64748b;">Dianalisis menggunakan GreenCode Static Analyzer (Rust Native Engine)</p>
                </div>
            </div>
            <div>
                <span style="background:#3b82f6;color:white;padding:4px 12px;border-radius:20px;font-size:13px;font-weight:600;">STATIC AUDIT</span>
            </div>
        </div>
        __FILTER_BANNER__

        <div class="grid">
            <div class="card">
                <div class="lbl">Total File</div>
                <div class="val">{}</div>
            </div>
            <div class="card">
                <div class="lbl">Total NCLOC</div>
                <div class="val">{}</div>
            </div>
            <div class="card">
                <div class="lbl">Avg CCN / File</div>
                <div class="val">{}</div>
            </div>
            <div class="card">
                <div class="lbl">Maintainability Index</div>
                <div class="val" style="color:#059669;">{}/100</div>
                <div style="font-size:11px;color:#64748b;margin-top:2px;">{}</div>
            </div>
            <div class="card">
                <div class="lbl">Code Smells</div>
                <div class="val" style="color:#ef4444;">{}</div>
            </div>
            __LANG_CARDS__
        </div>

        <div class="desc-box">
            <h3>📐 Panduan & Penjelasan Parameter Kualitas Struktur Kode</h3>
            <ul>
                <li><strong>NCLOC (Non-Comment Lines of Code):</strong> Jumlah baris instruksi logika program yang dieksekusi tanpa menghitung baris kosong dan komentar. Semakin proporsional NCLOC per berkas (&lt; 300 baris), semakin mudah dirawat.</li>
                <li><strong>CCN (Cyclomatic Complexity Number):</strong> Jumlah jalur percabangan independen (if/else/switch/loop) dalam logika fungsi.
                    <br><span style="display:inline-block;margin-top:2px;font-size:12px;">🟢 <em>1 - 10: Rendah & Aman</em> | 🟡 <em>11 - 20: Sedang (rawan bug)</em> | 🔴 <em>&gt; 20: Tinggi & Kompleks (wajib modularisasi)</em></span>
                </li>
                <li><strong>MI (Maintainability Index, Skala 0 - 100):</strong> Indeks komprehensif kemudahan pemeliharaan kode berbasis perpaduan logaritma Halstead Volume, CCN, dan NCLOC.
                    <br><span style="display:inline-block;margin-top:2px;font-size:12px;">🟢 <em>Grade A (≥ 85): Bersih & Sangat Mudah Dipelihara</em> | 🟡 <em>Grade B (65 - 84): Cukup</em> | 🔴 <em>Grade C (&lt; 65): Technical Debt Tinggi</em></span>
                </li>
                <li><strong>Smells (Code Smells):</strong> Deteksi gejala arsitektur buruk: <em>God Class</em> (kelas terlalu besar), <em>Fat Controller</em> (logika bertumpuk di controller), dan <em>Deep Nesting</em> (percabangan bersarang &gt; 4 level).</li>
            </ul>
        </div>

        <div class="section-title">Daftar Temuan Code Smells & Arsitektur</div>
        <table>
            <thead>
                <tr>
                    <th style="text-align:center;">Severity</th>
                    <th>Tipe Masalah</th>
                    <th>Deskripsi</th>
                    <th>Lokasi Berkas</th>
                    <th>💡 Saran Perbaikan</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="section-title">Rincian Metrik per Berkas Kode</div>
        <table>
            <thead>
                <tr>
                    <th>Nama Berkas</th>
                    <th style="text-align:center;">Bahasa</th>
                    <th style="text-align:right;">NCLOC</th>
                    <th style="text-align:right;">CCN (Kompleksitas)</th>
                    <th style="text-align:right;">MI (0-100)</th>
                    <th style="text-align:center;">Smells</th>
                    <th>💡 Saran Optimasi</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="footer">
            Dihasilkan secara otomatis oleh <strong>GreenCode Profiler v1.7.2 (Rust Native Architecture)</strong>.
        </div>
    </div>
</body>
</html>"##,
            LOGO_BASE64,
            ov["total_files"],
            ov["total_ncloc"].as_u64().unwrap_or(0),
            ov["average_ccn_per_file"],
            ov["average_maintainability_index"],
            ov["maintainability_grade"].as_str().unwrap_or(""),
            ov["total_code_smells"],
            rows_smells,
            rows_files
        );

        let html = html
            .replace("__FILTER_BANNER__", &filter_banner)
            .replace("__LANG_CARDS__", &lang_cards);

        fs::write(p, html)
    }

    pub fn open_in_browser<P: AsRef<Path>>(path: P) {
        let p = path.as_ref();
        let path_str = p.to_string_lossy();
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd")
                .args(["/C", "start", "", &path_str])
                .spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open").arg(&path_str).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xdg-open").arg(&path_str).spawn();
        }
    }
}

/// Banner yang menampilkan keyword filter aktif + jumlah file yang cocok.
fn build_filter_banner(applied_filter: &Value) -> String {
    if applied_filter.is_null() {
        return String::new();
    }
    let filters: Vec<&str> = applied_filter["filters"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let excludes: Vec<&str> = applied_filter["excludes"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let matched = applied_filter["matched_files"].as_u64().unwrap_or(0);
    let considered = applied_filter["considered_files"].as_u64().unwrap_or(0);

    let exclude_part = if excludes.is_empty() {
        String::new()
    } else {
        format!(" | Exclude: <strong>{}</strong>", excludes.join(", "))
    };

    format!(
        r#"<div class="desc-box" style="background:#ecfeff;border-color:#a5f3fc;color:#155e75;">
            <h3>🔎 Filter Kode Aktif</h3>
            <p style="margin:0;">Keyword: <strong>{}</strong>{} — <strong>{}</strong> dari <strong>{}</strong> file kode cocok (path atau isi).</p>
        </div>"#,
        filters.join(", "),
        exclude_part,
        matched,
        considered
    )
}

/// Kartu sebaran bahasa (mis. PHP + TypeScript) sebagai bukti cakupan monorepo.
fn build_language_cards(language_breakdown: &Value) -> String {
    let mut cards = String::new();
    if let Some(list) = language_breakdown.as_array() {
        for l in list {
            cards.push_str(&format!(
                r#"<div class="card"><div class="lbl">{}</div><div class="val">{}</div><div style="font-size:12px;color:#64748b;margin-top:4px;">file</div></div>"#,
                l["language"].as_str().unwrap_or("-"),
                l["files"].as_u64().unwrap_or(0)
            ));
        }
    }
    cards
}
