"""
Report Generator for GreenCode Profiler
Menghasilkan laporan profiling yang bersih, sederhana, dan mudah dibaca (Clean & Minimalist).
Cocok untuk dokumentasi teknis, evaluasi arsitektur, dan ekspor PDF.
"""
import json
import os
import webbrowser
from datetime import datetime

class ReportGenerator:
    @staticmethod
    def save_json(data, output_path):
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2)
        return output_path

    @staticmethod
    def open_in_browser(html_path):
        try:
            full_path = "file://" + os.path.abspath(html_path).replace("\\", "/")
            webbrowser.open(full_path)
        except Exception:
            pass

    @staticmethod
    def generate_html(report_data, output_path):
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        
        meta = report_data.get('metadata', {})
        totals = report_data.get('totals', {})
        modules = report_data.get('correlated_modules', [])
        endpoints = report_data.get('endpoints_detail', [])
        code_analysis = report_data.get('code_quality', {})
        code_overview = code_analysis.get('overview', {})
        code_smells = code_analysis.get('code_smells', [])
        top_funcs = code_analysis.get('top_complex_functions', [])

        # 1. Baris Tabel Modul & Energi
        matrix_rows = ""
        for m in modules:
            prio = m.get('refactoring_risk_level', 'LOW_PRIORITY')
            prio_badge = "badge-danger" if prio == 'HIGH_PRIORITY' else ("badge-warning" if prio == 'MEDIUM_PRIORITY' else "badge-normal")
            prio_text = "Tinggi" if prio == 'HIGH_PRIORITY' else ("Sedang" if prio == 'MEDIUM_PRIORITY' else "Rendah")
            
            matrix_rows += f"""
            <tr>
                <td><strong>{m['module']}</strong></td>
                <td><span class="badge {prio_badge}">{prio_text}</span></td>
                <td>{m['total_requests']}</td>
                <td>{m['avg_latency_ms']:.1f} ms</td>
                <td>{m['cpu_active_time_s']:.4f} s</td>
                <td><strong>{m['total_joules']:.4f} J</strong></td>
                <td>{m['joules_per_request']:.4f} J</td>
                <td>{m.get('total_ccn', '-')}</td>
                <td>{m.get('loc', '-')}</td>
            </tr>
            """

        # 2. Baris Tabel Fungsi Kompleks
        func_rows = ""
        if top_funcs:
            for fn in top_funcs:
                ccn_badge = "badge-danger" if fn['ccn'] > 15 else ("badge-warning" if fn['ccn'] > 10 else "badge-normal")
                func_rows += f"""
                <tr>
                    <td><strong>{fn['name']}</strong></td>
                    <td><code>{fn['file']}:{fn['line']}</code></td>
                    <td><span class="badge {ccn_badge}">CCN {fn['ccn']}</span></td>
                    <td>{fn['lines']} baris</td>
                </tr>
                """
        else:
            func_rows = "<tr><td colspan='4' class='text-muted' style='text-align:center;'>Tidak ada data fungsi kompleks.</td></tr>"

        # 3. Baris Tabel Code Smells
        smell_rows = ""
        if code_smells:
            for s in code_smells[:15]:
                sev_badge = "badge-danger" if s['severity'] == 'HIGH' else "badge-warning"
                smell_rows += f"""
                <tr>
                    <td><span class="badge {sev_badge}">{s['type']}</span></td>
                    <td><code>{s['file']}</code></td>
                    <td>{s['message']}</td>
                </tr>
                """
        else:
            smell_rows = "<tr><td colspan='3' class='text-muted' style='text-align:center;'>Tidak ada code smell kritis yang terdeteksi.</td></tr>"

        # 4. Baris Detail Request
        ep_rows = ""
        for ep in endpoints[:25]:
            status_cls = "text-success" if ep['success'] else "text-danger"
            ep_rows += f"""
            <tr>
                <td><code>{ep.get('method', 'GET')}</code></td>
                <td><code>{ep['url']}</code></td>
                <td><span class="{status_cls} font-semibold">{ep['status']}</span></td>
                <td>{ep['latency_ms']:.1f} ms</td>
                <td>{ep['cpu_active_time_s']:.4f} s</td>
                <td>{ep['energy_total_joules']:.6f} J</td>
                <td>{ep.get('avg_rss_mb', 0):.1f} MB</td>
            </tr>
            """

        html_content = f"""<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Laporan GreenCode Profiler - {meta.get('project_name', 'Aplikasi')}</title>
    <style>
        :root {{
            --bg: #ffffff;
            --text: #1e293b;
            --text-muted: #64748b;
            --border: #e2e8f0;
            --card-bg: #f8fafc;
            --primary: #0f766e;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background: var(--bg);
            color: var(--text);
            line-height: 1.5;
            padding: 30px 20px;
            font-size: 14px;
        }}
        .container {{ max-width: 960px; margin: 0 auto; }}
        
        /* HEADER */
        header {{
            border-bottom: 2px solid var(--border);
            padding-bottom: 16px;
            margin-bottom: 24px;
        }}
        h1 {{ font-size: 22px; font-weight: 700; color: #0f172a; margin-bottom: 4px; }}
        .meta-info {{ color: var(--text-muted); font-size: 13px; }}

        /* SUMMARY KPI */
        .summary-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 12px;
            margin-bottom: 32px;
        }}
        .summary-card {{
            background: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 14px 16px;
        }}
        .summary-label {{ font-size: 12px; color: var(--text-muted); font-weight: 600; text-transform: uppercase; }}
        .summary-value {{ font-size: 22px; font-weight: 700; color: #0f172a; margin: 4px 0; }}
        .summary-sub {{ font-size: 12px; color: var(--text-muted); }}

        /* SECTIONS */
        section {{ margin-bottom: 32px; }}
        h2 {{ font-size: 16px; font-weight: 700; color: #0f172a; margin-bottom: 12px; }}
        
        /* TABLES */
        table {{
            width: 100%;
            border-collapse: collapse;
            font-size: 13px;
            background: #fff;
            border: 1px solid var(--border);
            border-radius: 6px;
            overflow: hidden;
        }}
        th, td {{
            padding: 10px 12px;
            text-align: left;
            border-bottom: 1px solid var(--border);
        }}
        th {{
            background: var(--card-bg);
            font-weight: 600;
            color: #334155;
            font-size: 12px;
        }}
        tr:last-child td {{ border-bottom: none; }}
        tr:hover td {{ background: #f8fafc; }}

        /* BADGES */
        .badge {{
            display: inline-block;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 11px;
            font-weight: 600;
        }}
        .badge-danger {{ background: #fee2e2; color: #991b1b; }}
        .badge-warning {{ background: #fef3c7; color: #92400e; }}
        .badge-normal {{ background: #e2e8f0; color: #334155; }}

        code {{
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
            background: #f1f5f9;
            padding: 2px 4px;
            border-radius: 4px;
            font-size: 12px;
            color: #334155;
        }}
        .text-muted {{ color: var(--text-muted); }}
        .text-success {{ color: #15803d; }}
        .text-danger {{ color: #b91c1c; }}
        .font-semibold {{ font-weight: 600; }}

        footer {{
            border-top: 1px solid var(--border);
            padding-top: 16px;
            text-align: center;
            font-size: 12px;
            color: var(--text-muted);
            margin-top: 40px;
        }}

        @media print {{
            body {{ padding: 0; font-size: 12px; }}
            .container {{ max-width: 100%; }}
            .summary-card {{ border: 1px solid #ccc; }}
            table {{ border: 1px solid #ccc; }}
            th, td {{ border-bottom: 1px solid #ccc; padding: 6px 8px; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Laporan Profiling GreenCode</h1>
            <div class="meta-info">
                Proyek: <strong>{meta.get('project_name', 'Web Application')}</strong> &bull;
                Runtime: <strong>{meta.get('runtime', 'Auto').upper()}</strong> &bull;
                Waktu: {meta.get('timestamp', datetime.now().strftime('%Y-%m-%d %H:%M:%S'))}
            </div>
        </header>

        <!-- RINGKASAN METRIK -->
        <div class="summary-grid">
            <div class="summary-card">
                <div class="summary-label">Total Energi</div>
                <div class="summary-value">{totals.get('total_joules', 0.0):.4f} J</div>
                <div class="summary-sub">{totals.get('total_kwh', 0.0):.8f} kWh</div>
            </div>
            <div class="summary-card">
                <div class="summary-label">Waktu CPU Aktif</div>
                <div class="summary-value">{totals.get('total_cpu_active_s', 0.0):.3f} s</div>
                <div class="summary-sub">Murni waktu komputasi proses</div>
            </div>
            <div class="summary-card">
                <div class="summary-label">Maintainability Index</div>
                <div class="summary-value">{code_overview.get('average_maintainability_index', 80.0)} <small style="font-size: 13px; color: var(--text-muted);">/ 100</small></div>
                <div class="summary-sub">Grade: <strong>{code_overview.get('maintainability_grade', 'B')}</strong> &bull; Avg CCN: {code_overview.get('average_ccn_per_file', '-')}</div>
            </div>
            <div class="summary-card">
                <div class="summary-label">Total Request</div>
                <div class="summary-value">{totals.get('total_requests', len(endpoints))}</div>
                <div class="summary-sub">Rata-rata latensi: {totals.get('avg_latency_ms', 0.0):.1f} ms</div>
            </div>
        </div>

        <!-- 1. KONSUMSI ENERGI PER MODUL -->
        <section>
            <h2>1. Konsumsi Energi &amp; Kompleksitas Per Modul</h2>
            <table>
                <thead>
                    <tr>
                        <th>Modul</th>
                        <th>Prioritas Refactor</th>
                        <th>Request</th>
                        <th>Latensi</th>
                        <th>CPU Aktif</th>
                        <th>Total Energi</th>
                        <th>Energi/Req</th>
                        <th>CCN</th>
                        <th>LOC</th>
                    </tr>
                </thead>
                <tbody>
                    {matrix_rows}
                </tbody>
            </table>
        </section>

        <!-- 2. ANALISIS KUALITAS KODE STATIS -->
        <section>
            <h2>2. Fungsi dengan Kompleksitas Tertinggi (CCN)</h2>
            <table>
                <thead>
                    <tr>
                        <th>Nama Fungsi</th>
                        <th>Lokasi Berkas</th>
                        <th>Kompleksitas (CCN)</th>
                        <th>Ukuran Baris</th>
                    </tr>
                </thead>
                <tbody>
                    {func_rows}
                </tbody>
            </table>
        </section>

        <!-- 3. CODE SMELLS -->
        <section>
            <h2>3. Temuan Arsitektur &amp; Code Smells</h2>
            <table>
                <thead>
                    <tr>
                        <th>Tipe</th>
                        <th>Berkas</th>
                        <th>Keterangan</th>
                    </tr>
                </thead>
                <tbody>
                    {smell_rows}
                </tbody>
            </table>
        </section>

        <!-- 4. DETAIL REQUEST RUNTIME -->
        <section>
            <h2>4. Detail Request Pengujian</h2>
            <table>
                <thead>
                    <tr>
                        <th>Method</th>
                        <th>URL</th>
                        <th>Status</th>
                        <th>Latensi</th>
                        <th>CPU Aktif</th>
                        <th>Energi</th>
                        <th>RAM RSS</th>
                    </tr>
                </thead>
                <tbody>
                    {ep_rows}
                </tbody>
            </table>
        </section>

        <footer>
            Laporan dibuat otomatis oleh GreenCode Profiler v1.0
        </footer>
    </div>
</body>
</html>
"""
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_content)
        return output_path
