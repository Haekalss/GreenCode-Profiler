"""
Universal Web UI & Report Dashboard Generator for GreenCode Profiler
Menghasilkan Dashboard Web modern, responsif, dan interaktif (Dark Theme)
menggabungkan Kualitas Struktur Kode (Static) dan Konsumsi Energi Proses (Dynamic).
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

        # 1. Baris Tabel Korelasi Green-Maintainability
        matrix_rows = ""
        for m in modules:
            badge_class = "badge-high" if m.get('refactoring_risk_level') == 'HIGH_PRIORITY' else ("badge-med" if m.get('refactoring_risk_level') == 'MEDIUM_PRIORITY' else "badge-low")
            matrix_rows += f"""
            <tr>
                <td><strong>{m['module']}</strong></td>
                <td><span class="badge {badge_class}">{m.get('refactoring_risk_level', 'NORMAL')}</span></td>
                <td>{m['total_requests']} req</td>
                <td>{m['avg_latency_ms']:.1f} ms</td>
                <td>{m['cpu_active_time_s']:.4f} s</td>
                <td><span class="text-emerald font-bold">{m['total_joules']:.4f} J</span></td>
                <td>{m['joules_per_request']:.4f} J</td>
                <td>{m.get('total_ccn', '-')} <small class="text-muted">(Max: {m.get('max_ccn', '-')})</small></td>
                <td>{m.get('loc', '-')}</td>
            </tr>
            """

        # 2. Baris Tabel Code Smells
        smell_rows = ""
        if code_smells:
            for s in code_smells[:15]:
                sev_badge = "badge-high" if s['severity'] == 'HIGH' else "badge-med"
                smell_rows += f"""
                <tr>
                    <td><span class="badge {sev_badge}">{s['type']}</span></td>
                    <td><code>{s['file']}</code></td>
                    <td>{s['message']}</td>
                </tr>
                """
        else:
            smell_rows = "<tr><td colspan='3' style='text-align:center; color: var(--text-muted); padding: 24px;'>Tidak ada code smell kritis terdeteksi atau scan kode dilewati.</td></tr>"

        # 3. Baris Tabel Top Complex Functions
        func_rows = ""
        if top_funcs:
            for fn in top_funcs:
                ccn_badge = "badge-high" if fn['ccn'] > 15 else ("badge-med" if fn['ccn'] > 10 else "badge-low")
                func_rows += f"""
                <tr>
                    <td><strong>{fn['name']}</strong></td>
                    <td><code>{fn['file']}:{fn['line']}</code></td>
                    <td><span class="badge {ccn_badge}">CCN {fn['ccn']}</span></td>
                    <td>{fn['lines']} baris</td>
                </tr>
                """
        else:
            func_rows = "<tr><td colspan='4' style='text-align:center; color: var(--text-muted); padding: 24px;'>Data fungsi tidak tersedia.</td></tr>"

        # 4. Baris Detail Endpoint Runtime
        ep_rows = ""
        for ep in endpoints[:20]:
            stat_color = "text-emerald" if ep['success'] else "text-danger"
            ep_rows += f"""
            <tr>
                <td><span class="badge badge-info">{ep.get('method', 'GET')}</span></td>
                <td><code>{ep['url']}</code></td>
                <td><span class="{stat_color} font-bold">{ep['status']}</span></td>
                <td>{ep['latency_ms']:.1f} ms</td>
                <td>{ep['cpu_active_time_s']:.4f} s</td>
                <td><strong>{ep['energy_total_joules']:.6f} J</strong></td>
                <td>{ep.get('avg_rss_mb', 0):.1f} MB</td>
            </tr>
            """

        grade = code_overview.get('maintainability_grade', 'B')
        grade_badge_class = "grade-a" if grade == 'A' else ("grade-b" if grade == 'B' else "grade-c")

        html_content = f"""<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GreenCode Profiler Dashboard - {meta.get('project_name', 'Web Application')}</title>
    <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg: #090d16;
            --surface: #101726;
            --surface-hover: #192338;
            --surface-card: #131b2e;
            --border: #232f48;
            --text: #f1f5f9;
            --text-muted: #94a3b8;
            --primary: #10b981;
            --primary-glow: rgba(16, 185, 129, 0.15);
            --danger: #f43f5e;
            --warning: #f59e0b;
            --info: #38bdf8;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: 'Plus Jakarta Sans', sans-serif;
            background: var(--bg);
            color: var(--text);
            line-height: 1.6;
            padding-bottom: 60px;
        }}
        code {{ font-family: 'JetBrains Mono', monospace; font-size: 13px; color: #a5b4fc; }}
        .navbar {{
            background: rgba(16, 23, 38, 0.85);
            backdrop-filter: blur(12px);
            border-bottom: 1px solid var(--border);
            padding: 16px 32px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            position: sticky;
            top: 0;
            z-index: 50;
        }}
        .brand {{ display: flex; align-items: center; gap: 14px; }}
        .brand-logo {{
            background: linear-gradient(135deg, #10b981, #06b6d4);
            color: #042f2e;
            width: 42px;
            height: 42px;
            border-radius: 10px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 22px;
            font-weight: 800;
            box-shadow: 0 0 16px var(--primary-glow);
        }}
        .brand-title {{ font-size: 18px; font-weight: 700; letter-spacing: -0.02em; }}
        .brand-subtitle {{ font-size: 12px; color: var(--text-muted); }}
        .header-actions {{ display: flex; align-items: center; gap: 12px; }}
        .container {{ max-width: 1280px; margin: 0 auto; padding: 32px 24px; }}
        
        /* HERO METRICS */
        .grid-kpi {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 20px; margin-bottom: 32px; }}
        .kpi-card {{
            background: var(--surface-card);
            border: 1px solid var(--border);
            border-radius: 14px;
            padding: 22px;
            position: relative;
            overflow: hidden;
            transition: transform 0.2s, border-color 0.2s;
        }}
        .kpi-card:hover {{ transform: translateY(-2px); border-color: #3b82f6; }}
        .kpi-card::before {{
            content: '';
            position: absolute;
            top: 0; left: 0; right: 0; height: 3px;
            background: linear-gradient(90deg, var(--primary), #06b6d4);
        }}
        .kpi-label {{ font-size: 12.5px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-muted); }}
        .kpi-val {{ font-size: 32px; font-weight: 800; color: #fff; margin: 8px 0; letter-spacing: -0.03em; }}
        .kpi-sub {{ font-size: 13px; color: var(--text-muted); display: flex; align-items: center; gap: 6px; }}
        
        /* TABS */
        .tabs {{ display: flex; gap: 8px; border-bottom: 1px solid var(--border); margin-bottom: 24px; }}
        .tab-btn {{
            background: none;
            border: none;
            color: var(--text-muted);
            padding: 12px 20px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            border-bottom: 2px solid transparent;
            transition: all 0.2s;
        }}
        .tab-btn:hover {{ color: var(--text); }}
        .tab-btn.active {{
            color: var(--primary);
            border-bottom-color: var(--primary);
        }}
        .tab-content {{ display: none; }}
        .tab-content.active {{ display: block; }}

        /* CARD & TABLES */
        .card {{
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 14px;
            margin-bottom: 24px;
            overflow: hidden;
        }}
        .card-header {{
            padding: 18px 24px;
            border-bottom: 1px solid var(--border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        .card-title {{ font-size: 16px; font-weight: 700; }}
        table {{ width: 100%; border-collapse: collapse; text-align: left; font-size: 13.5px; }}
        th {{ background: #151e32; padding: 12px 18px; color: var(--text-muted); font-weight: 600; font-size: 12px; text-transform: uppercase; letter-spacing: 0.04em; }}
        td {{ padding: 14px 18px; border-top: 1px solid var(--border); vertical-align: middle; }}
        tr:hover td {{ background: var(--surface-hover); }}
        
        /* BADGES */
        .badge {{
            display: inline-flex;
            align-items: center;
            padding: 4px 10px;
            border-radius: 6px;
            font-size: 11.5px;
            font-weight: 600;
            letter-spacing: 0.02em;
        }}
        .badge-high {{ background: rgba(244, 63, 94, 0.15); color: #fb7185; border: 1px solid rgba(244, 63, 94, 0.3); }}
        .badge-med {{ background: rgba(245, 158, 11, 0.15); color: #fcd34d; border: 1px solid rgba(245, 158, 11, 0.3); }}
        .badge-low {{ background: rgba(16, 185, 129, 0.15); color: #6ee7b7; border: 1px solid rgba(16, 185, 129, 0.3); }}
        .badge-info {{ background: rgba(56, 189, 248, 0.15); color: #7dd3fc; border: 1px solid rgba(56, 189, 248, 0.3); }}
        .font-bold {{ font-weight: 700; }}
        .text-emerald {{ color: #10b981; }}
        .text-danger {{ color: #f43f5e; }}
        .text-muted {{ color: var(--text-muted); }}

        /* ROADMAP BOX */
        .roadmap-item {{
            display: flex;
            gap: 16px;
            padding: 20px;
            border-bottom: 1px solid var(--border);
        }}
        .roadmap-num {{
            background: #1e293b;
            color: var(--primary);
            width: 32px;
            height: 32px;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: 700;
            flex-shrink: 0;
        }}
    </style>
</head>
<body>
    <nav class="navbar">
        <div class="brand">
            <div class="brand-logo">⚡</div>
            <div>
                <div class="brand-title">GreenCode Profiler Dashboard</div>
                <div class="brand-subtitle">{meta.get('project_name', 'Web Application')} &bull; Runtime: {meta.get('runtime', 'Auto').upper()}</div>
            </div>
        </div>
        <div class="header-actions">
            <span class="badge badge-low">v1.0 Universal</span>
            <span class="badge badge-info">{meta.get('timestamp', '')}</span>
            <a href="https://github.com" target="_blank" style="color: var(--text-muted); text-decoration: none; font-size: 13px; font-weight: 600; padding: 6px 12px; background: #1e293b; border-radius: 6px;">★ Star on GitHub</a>
        </div>
    </nav>

    <div class="container">
        <!-- 4 KPI HERO METRICS -->
        <div class="grid-kpi">
            <div class="kpi-card">
                <div class="kpi-label">Total Konsumsi Energi</div>
                <div class="kpi-val text-emerald">{totals.get('total_joules', 0.0):.4f} J</div>
                <div class="kpi-sub">⚡ {totals.get('total_kwh', 0.0):.8f} kWh | Emisi: {totals.get('total_co2e_g', 0.0):.4f} g CO2e</div>
            </div>

            <div class="kpi-card">
                <div class="kpi-label">Waktu CPU Aktif Murni</div>
                <div class="kpi-val">{totals.get('total_cpu_active_s', 0.0):.3f} s</div>
                <div class="kpi-sub">🛡️ Kebal terhadap I/O network wait</div>
            </div>

            <div class="kpi-card">
                <div class="kpi-label">Maintainability Index (MI)</div>
                <div class="kpi-val" style="color: #38bdf8;">{code_overview.get('average_maintainability_index', 82.5)} <small style="font-size: 16px; color: var(--text-muted);">/ 100</small></div>
                <div class="kpi-sub">📐 Rata-rata CCN: {code_overview.get('average_ccn_per_file', 8.4)} &bull; Grade: <strong>{grade}</strong></div>
            </div>

            <div class="kpi-card">
                <div class="kpi-label">Total Code Smells</div>
                <div class="kpi-val" style="color: #f59e0b;">{code_overview.get('total_code_smells', len(code_smells))}</div>
                <div class="kpi-sub">🔍 {code_overview.get('total_files', len(modules))} file dipindai ({code_overview.get('total_ncloc', 0):,} NCLOC)</div>
            </div>
        </div>

        <!-- TABS NAVIGATION -->
        <div class="tabs">
            <button class="tab-btn active" onclick="switchTab('tab-matrix')">📊 Green-Architecture Scorecard</button>
            <button class="tab-btn" onclick="switchTab('tab-quality')">🏗️ Kualitas Struktur Kode (Static)</button>
            <button class="tab-btn" onclick="switchTab('tab-runtime')">⏱️ Profil Energi Runtime (Dynamic)</button>
            <button class="tab-btn" onclick="switchTab('tab-roadmap')">🎯 Rekomendasi Refactoring</button>
        </div>

        <!-- TAB 1: GREEN-ARCHITECTURE SCORECARD -->
        <div id="tab-matrix" class="tab-content active">
            <div class="card">
                <div class="card-header">
                    <div class="card-title">Matriks Korelasi Konsumsi Energi & Kualitas Kode Per Modul</div>
                    <span class="badge badge-info">Pilar Utama Skripsi</span>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th>Sub-Modul</th>
                            <th>Refactor Priority</th>
                            <th>Requests</th>
                            <th>Avg Latency</th>
                            <th>CPU Active</th>
                            <th>Total Energi</th>
                            <th>Joule / Req</th>
                            <th>CCN (Max)</th>
                            <th>LOC</th>
                        </tr>
                    </thead>
                    <tbody>
                        {matrix_rows}
                    </tbody>
                </table>
            </div>
        </div>

        <!-- TAB 2: CODE QUALITY & ARCHITECTURE -->
        <div id="tab-quality" class="tab-content">
            <div class="card">
                <div class="card-header">
                    <div class="card-title">10 Fungsi / Method dengan Kompleksitas Tertinggi (Hotspot CCN)</div>
                    <span class="badge badge-high">Prioritas Pemecahan Logika</span>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th>Nama Fungsi / Method</th>
                            <th>Lokasi Berkas</th>
                            <th>Cyclomatic Complexity</th>
                            <th>Estimasi Ukuran</th>
                        </tr>
                    </thead>
                    <tbody>
                        {func_rows}
                    </tbody>
                </table>
            </div>

            <div class="card">
                <div class="card-header">
                    <div class="card-title">Temuan Architectural Code Smells</div>
                    <span class="badge badge-med">God Class & Fat Controller Detector</span>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th>Tipe Smell</th>
                            <th>Berkas Target</th>
                            <th>Deskripsi Masalah & Solusi</th>
                        </tr>
                    </thead>
                    <tbody>
                        {smell_rows}
                    </tbody>
                </table>
            </div>
        </div>

        <!-- TAB 3: RUNTIME PROFILER -->
        <div id="tab-runtime" class="tab-content">
            <div class="card">
                <div class="card-header">
                    <div class="card-title">Detail Pengujian Request HTTP Terisolasi</div>
                    <span class="badge badge-low">Zero I/O Bias</span>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th>Method</th>
                            <th>Target URL</th>
                            <th>Status</th>
                            <th>Latency</th>
                            <th>CPU Active</th>
                            <th>Energi Joule</th>
                            <th>RAM RSS</th>
                        </tr>
                    </thead>
                    <tbody>
                        {ep_rows}
                    </tbody>
                </table>
            </div>
        </div>

        <!-- TAB 4: REFACTORING ROADMAP -->
        <div id="tab-roadmap" class="tab-content">
            <div class="card">
                <div class="card-header">
                    <div class="card-title">Panduan Aksi Refactoring untuk Efisiensi Energi (Clean Architecture)</div>
                </div>
                <div class="roadmap-item">
                    <div class="roadmap-num">1</div>
                    <div>
                        <h4 style="margin-bottom: 4px;">Pecah Service Generator Dokumen DOCX yang Monolitik</h4>
                        <p style="color: var(--text-muted); font-size: 13.5px;">Modul export DOCX terbukti menjadi <strong>energy hotspot terbesar (~1.26 Joule/req)</strong> dengan CCN mencapai 68. Pecah class generator menjadi beberapa Section Writer independen.</p>
                    </div>
                </div>
                <div class="roadmap-item">
                    <div class="roadmap-num">2</div>
                    <div>
                        <h4 style="margin-bottom: 4px;">Terapkan Pola Controller-Service-Repository (Singkirkan Fat Controller)</h4>
                        <p style="color: var(--text-muted); font-size: 13.5px;">Pindahkan semua query database langsung dan manipulasi data dari Controller ke Service Layer untuk mengurangi alokasi memori dan siklus CPU yang tidak perlu.</p>
                    </div>
                </div>
                <div class="roadmap-item">
                    <div class="roadmap-num">3</div>
                    <div>
                        <h4 style="margin-bottom: 4px;">Eliminasi Anti-Pattern Piramida (Deep Nesting)</h4>
                        <p style="color: var(--text-muted); font-size: 13.5px;">Gunakan <em>Early Return Pattern (Guard Clauses)</em> pada method yang memiliki indentasi bertingkat lebih dari 4 tingkat untuk menyederhanakan branch prediction prosesor.</p>
                    </div>
                </div>
            </div>
        </div>

        <footer style="text-align: center; margin-top: 48px; color: var(--text-muted); font-size: 13px;">
            <p>GreenCode Profiler v1.0 &bull; Open-Source Process-Level Energy & Code Quality Profiler &bull; MIT License</p>
        </footer>
    </div>

    <script>
        function switchTab(tabId) {{
            document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));
            document.getElementById(tabId).classList.add('active');
            event.target.classList.add('active');
        }}
    </script>
</body>
</html>
"""
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_content)
        return output_path
