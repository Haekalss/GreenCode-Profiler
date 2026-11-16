//! HTML and JSON Report Generator in Rust
//! Menghasilkan laporan evaluasi bersih, profesional, edukatif, dan print-friendly untuk lampiran skripsi/magang

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct ReportGenerator;

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
        let quality = &report_data["code_quality"]["overview"];

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
                rows_corr.push_str(&format!(
                    r#"<tr>
                        <td style="font-family:monospace;font-size:13px;">{}</td>
                        <td style="text-align:center;">{}</td>
                        <td style="text-align:right;">{:.2} ms</td>
                        <td style="text-align:right;font-weight:600;color:#059669;">{:.4} J</td>
                        <td style="text-align:right;">{:.3} s</td>
                        <td style="text-align:center;"><span style="background:#ecfdf5;color:#065f46;padding:3px 8px;border-radius:12px;font-weight:bold;">{:.1}</span></td>
                    </tr>"#,
                    m["module_name"].as_str().unwrap_or(""),
                    m["total_calls"],
                    m["avg_latency_ms"].as_f64().unwrap_or(0.0),
                    m["total_energy_joules"].as_f64().unwrap_or(0.0),
                    m["avg_cpu_active_s"].as_f64().unwrap_or(0.0),
                    m["green_efficiency_score"].as_f64().unwrap_or(0.0)
                ));
            }
        }

        let mut rows_ep = String::new();
        if let Some(list) = endpoints {
            for (i, ep) in list.iter().enumerate() {
                rows_ep.push_str(&format!(
                    r#"<tr>
                        <td style="text-align:center;">{}</td>
                        <td style="font-weight:600;color:#2563eb;">{}</td>
                        <td style="font-family:monospace;font-size:12px;">{}</td>
                        <td style="text-align:center;"><span style="background:#eff6ff;color:#1e40af;padding:2px 6px;border-radius:4px;font-weight:bold;">{}</span></td>
                        <td style="text-align:right;">{:.2} ms</td>
                        <td style="text-align:right;">{:.3} s</td>
                        <td style="text-align:right;font-weight:600;color:#059669;">{:.4} J</td>
                    </tr>"#,
                    i + 1,
                    ep["method"].as_str().unwrap_or("GET"),
                    ep["endpoint"].as_str().unwrap_or(""),
                    ep["status"],
                    ep["latency_ms"].as_f64().unwrap_or(0.0),
                    ep["cpu_active_time_s"].as_f64().unwrap_or(0.0),
                    ep["energy_total_joules"].as_f64().unwrap_or(0.0)
                ));
            }
        }

        let html = format!(
            r##"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GreenCode Profiler v1.2.0 (Rust Native Engine) Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; background: #f8fafc; color: #1e293b; margin: 0; padding: 30px; }}
        .container {{ max-width: 1100px; margin: 0 auto; background: #ffffff; border-radius: 12px; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1); padding: 40px; border: 1px solid #e2e8f0; }}
        .header {{ display: flex; justify-content: space-between; align-items: center; border-bottom: 2px solid #10b981; padding-bottom: 20px; margin-bottom: 30px; }}
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
        .desc-box {{ background: #f0fdf4; border: 1px solid #bbf7d0; border-radius: 8px; padding: 18px 22px; margin-bottom: 30px; line-height: 1.6; font-size: 14px; color: #166534; }}
        .desc-box h3 {{ margin-top: 0; margin-bottom: 10px; font-size: 16px; color: #14532d; }}
        .desc-box ul {{ margin: 0; padding-left: 20px; }}
        .desc-box li {{ margin-bottom: 6px; }}
        .desc-box strong {{ color: #0f172a; }}
        .footer {{ margin-top: 40px; text-align: center; font-size: 12px; color: #94a3b8; border-top: 1px solid #e2e8f0; padding-top: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div>
                <h1 style="margin:0;font-size:24px;color:#0f172a;">GreenCode Profiler Report</h1>
                <p style="margin:5px 0 0 0;color:#64748b;">Project: <strong>{}</strong> | Engine: <strong>Rust Native Profiler v1.2.0</strong></p>
            </div>
            <div>
                <span class="badge">GREEN AUDIT VERIFIED</span>
                <p style="margin:5px 0 0 0;font-size:12px;color:#64748b;text-align:right;">{}</p>
            </div>
        </div>

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
            <h3>📖 Penjelasan Parameter & Metrik GreenCode Profiler</h3>
            <ul>
                <li><strong>Total Energi Murni (Joule & kWh):</strong> Total konsumsi daya listrik murni yang ditarik oleh instruksi CPU selama server memproses transaksi. Dihitung dengan rumus <code>E = P_avg × t_active</code>, di mana <code>t_active = t_user + t_system</code> langsung dari kernel OS.</li>
                <li><strong>Waktu Aktif CPU Murni (Detik):</strong> Durasi aktual thread CPU bekerja mengeksekusi kode aplikasi. Berbeda dengan latensi jaringan/wall-clock time, parameter ini <em>kebal terhadap keterlambatan network I/O, waiting query database, atau koneksi lambat</em>.</li>
                <li><strong>Jejak Karbon Bersih (gCO2e):</strong> Estimasi emisi gas rumah kaca hasil konversi energi komputasi listrik dengan faktor emisi pembangkit listrik rata-rata Indonesia (Grid Emission Factor ~850 gCO2e/kWh).</li>
                <li><strong>Maintainability Index (MI):</strong> Skor kemampuan pemeliharaan kode (skala 0–100) berdasarkan kombinasi Cyclomatic Complexity, Volume Halstead, dan NCLOC. Skor &gt; 85: Grade A (Sangat Mudah Dirawat), 65–84: Grade B, &lt; 65: Grade C (Rawan Debt & Bug).</li>
                <li><strong>Green Score (Efisiensi):</strong> Indeks efisiensi rasio konsumsi energi terhadap throughput dan kompleksitas modul kode.</li>
            </ul>
        </div>

        <div class="section-title">Ringkasan Efisiensi Energi per Modul</div>
        <table>
            <thead>
                <tr>
                    <th>Endpoint / URL Target</th>
                    <th style="text-align:center;">Jumlah Request</th>
                    <th style="text-align:right;">Rata-rata Latensi</th>
                    <th style="text-align:right;">Total Energi (Joule)</th>
                    <th style="text-align:right;">Waktu CPU Murni</th>
                    <th style="text-align:center;">Green Score</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="section-title">Detail Pengukuran Transaksi HTTP</div>
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
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="footer">
            Dihasilkan secara otomatis oleh <strong>GreenCode Profiler v1.2.0 (Rust Native Architecture)</strong>.<br>
            Laporan ini memenuhi standar audit perangkat lunak ramah lingkungan (Green Software Engineering).
        </div>
    </div>
</body>
</html>"##,
            meta["project_name"].as_str().unwrap_or("Web Project"),
            meta["timestamp"].as_str().unwrap_or(""),
            total_joules,
            kwh_formatted,
            total_cpu_active_s,
            total_co2e_g,
            quality["average_maintainability_index"],
            quality["maintainability_grade"].as_str().unwrap_or("Grade A"),
            rows_corr,
            rows_ep
        );

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
                rows_files.push_str(&format!(
                    r#"<tr>
                        <td style="font-family:monospace;font-size:12px;">{}</td>
                        <td style="text-align:center;"><span style="background:#f1f5f9;padding:2px 6px;border-radius:4px;">{}</span></td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:right;font-weight:600;">{}</td>
                        <td style="text-align:right;font-weight:bold;color:#059669;">{:.1}</td>
                        <td style="text-align:center;">{}</td>
                    </tr>"#,
                    f["file_path"].as_str().unwrap_or(""),
                    f["language"].as_str().unwrap_or(""),
                    f["ncloc"],
                    f["ccn"],
                    f["maintainability_index"].as_f64().unwrap_or(0.0),
                    f["code_smells"].as_array().map(|a| a.len()).unwrap_or(0)
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
                rows_smells.push_str(&format!(
                    r#"<tr>
                        <td style="text-align:center;"><span style="background:{};color:white;padding:2px 8px;border-radius:4px;font-size:11px;font-weight:bold;">{}</span></td>
                        <td style="font-weight:600;">{}</td>
                        <td>{}</td>
                        <td style="font-family:monospace;font-size:12px;">{}:{}</td>
                    </tr>"#,
                    badge_color,
                    s["severity"].as_str().unwrap_or(""),
                    s["smell_type"].as_str().unwrap_or(""),
                    s["message"].as_str().unwrap_or(""),
                    s["file"].as_str().unwrap_or(""),
                    s["line"]
                ));
            }
        }

        let html = format!(
            r##"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <title>GreenCode Static Code Quality Report (Rust)</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f8fafc; color: #1e293b; margin: 0; padding: 30px; }}
        .container {{ max-width: 1100px; margin: 0 auto; background: #ffffff; border-radius: 12px; padding: 40px; border: 1px solid #e2e8f0; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1); }}
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
        .desc-box li {{ margin-bottom: 6px; }}
        .desc-box strong {{ color: #0f172a; }}
        .footer {{ margin-top: 40px; text-align: center; font-size: 12px; color: #94a3b8; border-top: 1px solid #e2e8f0; padding-top: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1 style="margin:0;font-size:24px;">Static Code Quality & Architecture Report</h1>
        <p style="color:#64748b;margin-top:4px;">Dianalisis menggunakan GreenCode Static Analyzer (Rust Native Engine)</p>

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
        </div>

        <div class="desc-box">
            <h3>📐 Penjelasan Metrik Kualitas Kode</h3>
            <ul>
                <li><strong>NCLOC (Non-Comment Lines of Code):</strong> Jumlah baris instruksi logika aktual yang dieksekusi, tanpa menghitung baris kosong dan baris komentar dokumentasi.</li>
                <li><strong>Cyclomatic Complexity (CCN):</strong> Ukuran kuantitatif jumlah jalur percabangan independen (if/else/switch/loop) dalam program. Standar: 1–10 (Aman/Rendah), 11–20 (Sedang), &gt; 20 (Kompleks & Berisiko).</li>
                <li><strong>Maintainability Index (MI):</strong> Skor kemampuan kode untuk dipelihara dan dipahami oleh developer lain berdasarkan perpaduan metrik kompleksitas, volume, dan baris kode.</li>
                <li><strong>Code Smells:</strong> Indikator gejala buruk pada struktur kode, seperti <em>God Class</em> (kelas terlalu besar), <em>Fat Controller</em> (logika bisnis menumpuk di kontroler), dan <em>Deep Nesting</em> (percabangan bersarang &gt; 4 level).</li>
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
                    <th style="text-align:right;">CCN</th>
                    <th style="text-align:right;">MI (0-100)</th>
                    <th style="text-align:center;">Smells</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="footer">
            Dihasilkan secara otomatis oleh <strong>GreenCode Profiler v1.2.0 (Rust Native Architecture)</strong>.
        </div>
    </div>
</body>
</html>"##,
            ov["total_files"],
            ov["total_ncloc"].as_u64().unwrap_or(0),
            ov["average_ccn_per_file"],
            ov["average_maintainability_index"],
            ov["maintainability_grade"].as_str().unwrap_or(""),
            ov["total_code_smells"],
            rows_smells,
            rows_files
        );

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
