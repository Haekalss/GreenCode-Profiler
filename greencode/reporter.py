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

        /* GLOSSARY & EXPLANATION */
        .glossary-section {{
            margin-top: 36px;
            background: #f8fafc;
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 20px;
        }}
        .glossary-title {{
            font-size: 15px;
            font-weight: 700;
            color: #0f172a;
            margin-bottom: 4px;
        }}
        .glossary-subtitle {{
            font-size: 12.5px;
            color: var(--text-muted);
            margin-bottom: 16px;
        }}
        .glossary-category {{
            font-size: 12.5px;
            font-weight: 700;
            color: #0f766e;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            margin: 16px 0 10px 0;
            padding-bottom: 4px;
            border-bottom: 1px solid var(--border);
        }}
        .glossary-category:first-of-type {{ margin-top: 0; }}
        .glossary-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 12px;
        }}
        .glossary-item {{
            background: #ffffff;
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 12px 14px;
            font-size: 12.5px;
        }}
        .glossary-item-title {{
            font-weight: 700;
            color: #0f172a;
            margin-bottom: 4px;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}
        .glossary-item-tag {{
            font-size: 10.5px;
            font-weight: 600;
            padding: 1px 6px;
            border-radius: 4px;
            background: #e2e8f0;
            color: #475569;
        }}
        .glossary-desc {{
            color: #334155;
            line-height: 1.5;
            margin-bottom: 6px;
        }}
        .glossary-impact {{
            background: #f1f5f9;
            border-left: 3px solid #0f766e;
            padding: 6px 8px;
            font-size: 11.5px;
            color: #1e293b;
            border-radius: 0 4px 4px 0;
            line-height: 1.45;
        }}
        th[title] {{
            cursor: help;
            border-bottom: 1px dotted #94a3b8;
        }}

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
                        <th title="Nama modul atau kelompok fitur yang dianalisis">Modul</th>
                        <th title="Tingkat urgensi refactoring berdasarkan kombinasi konsumsi energi (Joule) dan kompleksitas kode (CCN)">Prioritas Refactor</th>
                        <th title="Jumlah pemanggilan HTTP request ke modul selama durasi pengujian">Request</th>
                        <th title="Rata-rata waktu tanggap dari klien mengirim request hingga selesai menerima respon (milidetik)">Latensi</th>
                        <th title="Waktu prosesor murni (t_user + t_system) saat menjalankan instruksi kode (detik)">CPU Aktif</th>
                        <th title="Total energi listrik fisik (Joule) yang dihabiskan prosesor untuk memproses modul ini">Total Energi</th>
                        <th title="Rata-rata konsumsi energi listrik per 1 kali pemanggilan HTTP request">Energi/Req</th>
                        <th title="Cyclomatic Complexity: Jumlah total cabang keputusan (if, switch, loop) pada fungsi-fungsi modul">CCN</th>
                        <th title="Lines of Code: Total baris kode instruksi (NCLOC) pada modul ini">LOC</th>
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
                        <th title="Nama fungsi, method, atau controller handler">Nama Fungsi</th>
                        <th title="Lokasi berkas dan baris fungsi dalam proyek">Lokasi Berkas</th>
                        <th title="Cyclomatic Complexity Number (CCN): Jalur logika independen. Standar: 1-10 (Sehat), 11-20 (Sedang), >20 (Kritis)">Kompleksitas (CCN)</th>
                        <th title="Panjang baris fungsi. Fungsi > 50 baris dianjurkan dipecah">Ukuran Baris</th>
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
                        <th title="Kategori bau kode arsitektural (misal: FAT_CONTROLLER, GOD_CLASS, DEEP_NESTING)">Tipe Smell</th>
                        <th title="Berkas yang mengalami pelanggaran prinsip desain">Berkas</th>
                        <th title="Deskripsi pelanggaran dan rekomendasi tindakan">Keterangan</th>
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
                        <th title="Metode HTTP (GET, POST, PUT, DELETE)">Method</th>
                        <th title="Alamat URL endpoint API atau halaman yang diuji">URL</th>
                        <th title="HTTP Status Code (200 = Berhasil, 404 = Not Found, 500 = Server Error)">Status</th>
                        <th title="Total waktu tunggu respon dari sudut pandang klien (ms)">Latensi</th>
                        <th title="Waktu komputasi murni CPU proses server (detik), mengabaikan waktu tunggu I/O">CPU Aktif</th>
                        <th title="Energi listrik komputasi prosesor untuk request ini (Joule)">Energi</th>
                        <th title="Resident Set Size: Memori fisik RAM aktual yang ditempati proses server (MB)">RAM RSS</th>
                    </tr>
                </thead>
                <tbody>
                    {ep_rows}
                </tbody>
            </table>
        </section>

        <!-- 5. GLOSARIUM & PANDUAN VARIABEL METRIK -->
        <section class="glossary-section">
            <div class="glossary-title">📖 Panduan Membaca Variabel &amp; Dampak Kualitas</div>
            <div class="glossary-subtitle">Panduan singkat arti metrik, ambang batas, dan dampaknya terhadap efisiensi energi (Green Software) serta pemeliharaan sistem. <em>Arahkan kursor ke judul kolom tabel di atas untuk bantuan ringkas.</em></div>

            <div class="glossary-category">⚡ 1. Metrik Komputasi &amp; Efisiensi Energi (Dynamic Profiling)</div>
            <div class="glossary-grid">
                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>CPU Aktif (Active Time)</span>
                        <span class="glossary-item-tag">Detik (s)</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Waktu prosesor nyata mengeksekusi instruksi komputasi aplikasi (<code>t_user + t_system</code>), tanpa menghitung waktu idle saat menunggu I/O jaringan atau database.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Penentu utama konsumsi daya listrik CPU. Semakin tinggi CPU Aktif, semakin boros energi dan panas yang dihasilkan server.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>Total Energi &amp; Energi/Req</span>
                        <span class="glossary-item-tag">Joule (J)</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Estimasi konsumsi energi listrik fisik prosesor (<code>E = P_active × t_CPU</code>). 1 Joule = 1 Watt·detik.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Metrik inti <em>Green Software Engineering</em>. Endpoint dengan Joule tinggi menandakan *energy hotspot* yang perlu dioptimasi algoritmanya.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>Latensi (Latency)</span>
                        <span class="glossary-item-tag">Milidetik (ms)</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Total waktu tunggu dari saat request dikirim oleh klien hingga seluruh respon diterima kembali (Wall-Clock Time).</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Menentukan kepuasan pengguna (UX). Jika Latensi tinggi tapi CPU Aktif rendah, bottleneck ada di koneksi jaringan atau query database lambat.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>RAM RSS (Resident Set Size)</span>
                        <span class="glossary-item-tag">Megabyte (MB)</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Kapasitas memori fisik (RAM) aktual yang dialokasikan dan sedang aktif digunakan oleh proses server di sistem operasi.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> RAM yang terus merangkak naik mengindikasikan <em>memory leak</em> atau alokasi buffer berlebih yang memicu lonjakan beban Garbage Collection dan risiko Out of Memory (OOM).</div>
                </div>
            </div>

            <div class="glossary-category">📐 2. Metrik Kualitas Struktur Kode (Static Analysis)</div>
            <div class="glossary-grid">
                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>CCN (Cyclomatic Complexity)</span>
                        <span class="glossary-item-tag">Skala Jalur Logika</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Jumlah jalur percabangan independen dalam fungsi (<code>if</code>, <code>else</code>, <code>switch</code>, <code>for</code>, <code>while</code>, operator logika).<br>
                    <strong>Standar:</strong> 1-10 (Sehat/Rendah), 11-20 (Sedang), &gt;20 (Kritis/Rumit).</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> CCN tinggi menandakan logika berbelit-belit, rawan bug tersembunyi, sangat sulit diuji (unit testing), dan memaksa CPU mengevaluasi banyak instruksi percabangan.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>LOC &amp; NCLOC</span>
                        <span class="glossary-item-tag">Baris Kode Murni</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> <em>Non-Comment Lines of Code</em> — jumlah baris instruksi kode efektif tanpa menghitung baris kosong dan komentar.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Berkas atau method dengan ribuan baris kode memperberat beban pemeliharaan (cognitive fatigue) dan menandakan minimnya modularitas program.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>Maintainability Index (MI)</span>
                        <span class="glossary-item-tag">0 - 100 Skor</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Indeks gabungan dari Halstead Volume, CCN, dan LOC yang mengukur seberapa mudah kode dirawat dan dikembangkan di masa depan.<br>
                    <strong>Grade:</strong> &gt;80 (A), 70-79 (B), 50-69 (C), &lt;50 (D/E).</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Skor MI yang rendah menandakan kode rapuh, sulit dimodifikasi oleh tim, dan berbiaya pemeliharaan tinggi.</div>
                </div>
            </div>

            <div class="glossary-category">🚨 3. Temuan Pelanggaran Arsitektur (Code Smells)</div>
            <div class="glossary-grid">
                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>FAT_CONTROLLER</span>
                        <span class="glossary-item-tag">High Severity</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Controller menampung terlalu banyak logika bisnis, kalkulasi data rumit, atau query database langsung (melebihi 300 baris / query inline) alih-alih mendelegasikannya ke Service / Repository Layer.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Melanggar prinsip <em>Single Responsibility</em>. Controller menjadi gemuk, tidak dapat digunakan ulang (non-reusable), dan sulit dilakukan unit testing.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>GOD_CLASS_OR_BLOAT</span>
                        <span class="glossary-item-tag">High Severity</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Satu class raksasa yang mencoba mengelola terlalu banyak hal sekaligus (&gt;500 baris atau &gt;15 method), menjadi pusat ketergantungan seluruh modul.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Menjadi titik kegagalan tunggal (<em>Single Point of Failure</em>). Perubahan kecil pada satu fitur berisiko merusak fitur lainnya yang tidak terkait.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>DEEP_NESTING</span>
                        <span class="glossary-item-tag">Medium / High</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Percabangan bersarang terlalu dalam (lebih dari 3-4 tingkat struktur <code>if / for / foreach</code>) membentuk kode piramida atau anak tangga.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Alur logika sangat sulit diikuti secara visual, meningkatkan beban mental developer, dan sering melewatkan kondisi batas (edge cases). Solusi: gunakan <em>Guard Clauses</em> (return early).</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>LONG_METHOD</span>
                        <span class="glossary-item-tag">Medium Severity</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Satu fungsi/method memiliki panjang baris kode berlebihan (&gt;50 baris), menandakan fungsi melakukan lebih dari satu tugas.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Menyulitkan pengujian otomatis dan rawan menyembunyikan duplikasi algoritma. Solusi: terapkan refactoring <em>Extract Method</em>.</div>
                </div>
            </div>
        </section>

        <footer>
            Laporan dibuat otomatis oleh GreenCode Profiler v1.1.0 &bull; Green Software Engineering Framework
        </footer>
    </div>
</body>
</html>
"""
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_content)
        return output_path

    @staticmethod
    def generate_code_quality_html(data, output_path):
        """Menghasilkan laporan HTML bersih khusus untuk analisis struktur kode statis (greencode analyze)."""
        os.makedirs(os.path.dirname(output_path), exist_ok=True)

        overview = data.get('overview', {})
        top_funcs = data.get('top_complex_functions', [])
        code_smells = data.get('code_smells', [])
        files = data.get('files', [])

        grade = overview.get('maintainability_grade', 'B')
        grade_badge = "badge-danger" if grade in ['D', 'E'] else ("badge-warning" if grade == 'C' else "badge-normal")

        # 1. Baris Top Fungsi Kompleks
        func_rows = ""
        if top_funcs:
            for fn in top_funcs[:25]:
                ccn = fn.get('ccn', 1)
                ccn_badge = "badge-danger" if ccn > 15 else ("badge-warning" if ccn > 10 else "badge-normal")
                func_rows += f"""
                <tr>
                    <td><strong>{fn['name']}</strong></td>
                    <td><code>{fn['file']}:{fn['line']}</code></td>
                    <td><span class="badge {ccn_badge}">CCN {ccn}</span></td>
                    <td>{fn['lines']} baris</td>
                </tr>
                """
        else:
            func_rows = "<tr><td colspan='4' class='text-muted' style='text-align:center;'>Tidak ada data fungsi.</td></tr>"

        # 2. Baris Code Smells
        smell_rows = ""
        if code_smells:
            for s in code_smells[:35]:
                sev_badge = "badge-danger" if s['severity'] == 'HIGH' else "badge-warning"
                sev_text = "Tinggi" if s['severity'] == 'HIGH' else "Sedang"
                smell_rows += f"""
                <tr>
                    <td><span class="badge {sev_badge}">{s['type']}</span></td>
                    <td><code>{s['file']}</code></td>
                    <td><span class="badge {sev_badge}">{sev_text}</span></td>
                    <td>{s['message']}</td>
                </tr>
                """
        else:
            smell_rows = "<tr><td colspan='4' class='text-muted' style='text-align:center;'>Tidak ada code smell terdeteksi.</td></tr>"

        # 3. Baris Berkas Terpindai
        file_rows = ""
        for f in files[:40]:
            mi = f.get('maintainability_index', 100)
            mi_badge = "badge-danger" if mi < 50 else ("badge-warning" if mi < 70 else "badge-normal")
            file_rows += f"""
            <tr>
                <td><code>{f['file']}</code></td>
                <td>{f.get('language', '-')}</td>
                <td>{f.get('ncloc', 0):,}</td>
                <td><strong>{f.get('ccn', 1)}</strong></td>
                <td><span class="badge {mi_badge}">{mi}</span></td>
                <td>{f.get('functions_count', 0)}</td>
                <td>{f.get('smells_count', 0)}</td>
            </tr>
            """

        html_content = f"""<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Laporan Kualitas Struktur Kode - GreenCode Analyzer</title>
    <style>
        :root {{
            --bg: #ffffff;
            --text: #1e293b;
            --text-muted: #64748b;
            --border: #e2e8f0;
            --card-bg: #f8fafc;
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
        
        header {{
            border-bottom: 2px solid var(--border);
            padding-bottom: 16px;
            margin-bottom: 24px;
        }}
        h1 {{ font-size: 22px; font-weight: 700; color: #0f172a; margin-bottom: 4px; }}
        .meta-info {{ color: var(--text-muted); font-size: 13px; }}

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

        section {{ margin-bottom: 32px; }}
        h2 {{ font-size: 16px; font-weight: 700; color: #0f172a; margin-bottom: 12px; }}
        
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

        /* GLOSSARY & EXPLANATION */
        .glossary-section {{
            margin-top: 36px;
            background: #f8fafc;
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 20px;
        }}
        .glossary-title {{
            font-size: 15px;
            font-weight: 700;
            color: #0f172a;
            margin-bottom: 4px;
        }}
        .glossary-subtitle {{
            font-size: 12.5px;
            color: var(--text-muted);
            margin-bottom: 16px;
        }}
        .glossary-category {{
            font-size: 12.5px;
            font-weight: 700;
            color: #0f766e;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            margin: 16px 0 10px 0;
            padding-bottom: 4px;
            border-bottom: 1px solid var(--border);
        }}
        .glossary-category:first-of-type {{ margin-top: 0; }}
        .glossary-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 12px;
        }}
        .glossary-item {{
            background: #ffffff;
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 12px 14px;
            font-size: 12.5px;
        }}
        .glossary-item-title {{
            font-weight: 700;
            color: #0f172a;
            margin-bottom: 4px;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}
        .glossary-item-tag {{
            font-size: 10.5px;
            font-weight: 600;
            padding: 1px 6px;
            border-radius: 4px;
            background: #e2e8f0;
            color: #475569;
        }}
        .glossary-desc {{
            color: #334155;
            line-height: 1.5;
            margin-bottom: 6px;
        }}
        .glossary-impact {{
            background: #f1f5f9;
            border-left: 3px solid #0f766e;
            padding: 6px 8px;
            font-size: 11.5px;
            color: #1e293b;
            border-radius: 0 4px 4px 0;
            line-height: 1.45;
        }}
        th[title] {{
            cursor: help;
            border-bottom: 1px dotted #94a3b8;
        }}

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
            <h1>Laporan Kualitas Struktur Kode</h1>
            <div class="meta-info">
                Direktori: <code>{overview.get('scanned_directory', '-')}</code> &bull;
                Waktu: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
            </div>
        </header>

        <!-- RINGKASAN METRIK -->
        <div class="summary-grid">
            <div class="summary-card">
                <div class="summary-label">Total Berkas &amp; Baris</div>
                <div class="summary-value">{overview.get('total_files', 0)} file</div>
                <div class="summary-sub">{overview.get('total_ncloc', 0):,} NCLOC (baris kode murni)</div>
            </div>
            <div class="summary-card">
                <div class="summary-label">Kompleksitas (CCN)</div>
                <div class="summary-value">{overview.get('total_ccn', 0)}</div>
                <div class="summary-sub">Rata-rata: {overview.get('average_ccn_per_file', 0)} per berkas</div>
            </div>
            <div class="summary-card">
                <div class="summary-label">Maintainability Index</div>
                <div class="summary-value">{overview.get('average_maintainability_index', 0)} <small style="font-size: 13px; color: var(--text-muted);">/ 100</small></div>
                <div class="summary-sub">Predikat: <strong>Grade {grade}</strong></div>
            </div>
            <div class="summary-card">
                <div class="summary-label">Temuan Code Smells</div>
                <div class="summary-value">{overview.get('total_code_smells', 0)}</div>
                <div class="summary-sub">Fat Controller, God Class, Deep Nesting</div>
            </div>
        </div>

        <!-- 1. TOP FUNGSI KOMPLEKS -->
        <section>
            <h2>1. Fungsi dengan Kompleksitas Tertinggi (CCN > 10)</h2>
            <table>
                <thead>
                    <tr>
                        <th title="Nama fungsi, method, atau closure">Nama Fungsi</th>
                        <th title="Lokasi berkas sumber dan nomor baris fungsi">Lokasi Berkas</th>
                        <th title="Cyclomatic Complexity Number (CCN): Jumlah jalur percabangan logika independen. Skala: 1-10 (Sehat), 11-20 (Sedang), >20 (Kritis/Rumit)">Kompleksitas (CCN)</th>
                        <th title="Panjang baris kode fungsi (NCLOC). Fungsi > 50 baris disarankan dipecah menjadi sub-fungsi">Ukuran Baris</th>
                    </tr>
                </thead>
                <tbody>
                    {func_rows}
                </tbody>
            </table>
        </section>

        <!-- 2. CODE SMELLS -->
        <section>
            <h2>2. Temuan Architectural Code Smells</h2>
            <table>
                <thead>
                    <tr>
                        <th title="Kategori pelanggaran arsitektur (misal: FAT_CONTROLLER, GOD_CLASS_OR_BLOAT, DEEP_NESTING, LONG_METHOD)">Tipe Smell</th>
                        <th title="Berkas yang melanggar prinsip desain clean architecture">Berkas</th>
                        <th title="Tingkat keparahan dampak (Tinggi = Kritis, Sedang = Waspada)">Tingkat</th>
                        <th title="Deskripsi pelanggaran dan rekomendasi tindakan refactoring">Keterangan</th>
                    </tr>
                </thead>
                <tbody>
                    {smell_rows}
                </tbody>
            </table>
        </section>

        <!-- 3. DETAIL BERKAS -->
        <section>
            <h2>3. Ringkasan Per Berkas</h2>
            <table>
                <thead>
                    <tr>
                        <th title="Jalur berkas kode sumber yang dipindai">Nama Berkas</th>
                        <th title="Bahasa pemrograman berkas">Bahasa</th>
                        <th title="Non-Comment Lines of Code: Jumlah baris instruksi efektif tanpa baris kosong dan komentar">NCLOC</th>
                        <th title="Total Cyclomatic Complexity dalam berkas ini">CCN</th>
                        <th title="Maintainability Index (0-100): Kemudahan pemeliharaan kode (Grade A: >80, B: 70-79, C: 50-69, D/E: <50)">Maintainability Index</th>
                        <th title="Jumlah fungsi atau method yang ditemukan dalam berkas">Fungsi</th>
                        <th title="Jumlah pelanggaran bau kode (code smells) dalam berkas">Smells</th>
                    </tr>
                </thead>
                <tbody>
                    {file_rows}
                </tbody>
            </table>
        </section>

        <!-- 4. GLOSARIUM & PANDUAN VARIABEL METRIK -->
        <section class="glossary-section">
            <div class="glossary-title">📖 Panduan Membaca Variabel Kualitas Kode</div>
            <div class="glossary-subtitle">Panduan singkat arti metrik kualitas kode, ambang batas ideal, dan dampaknya terhadap pemeliharaan serta stabilitas sistem. <em>Arahkan kursor ke judul kolom tabel di atas untuk bantuan ringkas.</em></div>

            <div class="glossary-category">📐 1. Metrik Kualitas Struktur Kode (Static Analysis)</div>
            <div class="glossary-grid">
                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>CCN (Cyclomatic Complexity)</span>
                        <span class="glossary-item-tag">Skala Jalur Logika</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Mengukur jumlah jalur percabangan logika independen (<code>if</code>, <code>else</code>, <code>switch</code>, <code>for</code>, <code>while</code>, operator logika).<br>
                    <strong>Standar:</strong> 1-10 (Sehat/Rendah), 11-20 (Sedang), &gt;20 (Kritis/Rumit).</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> CCN tinggi menandakan fungsi berbelit-belit, rawan bug tersembunyi, sangat sulit diuji (unit testing), dan memperlambat pemahaman tim pengembang.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>LOC &amp; NCLOC</span>
                        <span class="glossary-item-tag">Baris Kode Murni</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> <em>Non-Comment Lines of Code</em> — jumlah baris instruksi kode efektif tanpa menghitung baris kosong dan komentar.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Berkas atau fungsi dengan baris terlalu panjang memperbesar beban kognitif developer dan mengindikasikan pelanggaran modularitas sistem.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>Maintainability Index (MI)</span>
                        <span class="glossary-item-tag">0 - 100 Skor</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Indeks gabungan dari Halstead Volume, CCN, dan LOC yang mengukur kemudahan merawat kode di masa depan.<br>
                    <strong>Grade:</strong> &gt;80 (A), 70-79 (B), 50-69 (C), &lt;50 (D/E).</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Skor MI rendah menandakan kode rapuh, rentan menimbulkan regresi saat ditambahkan fitur baru, dan berbiaya perawatan tinggi.</div>
                </div>
            </div>

            <div class="glossary-category">🚨 2. Temuan Pelanggaran Arsitektur (Architectural Code Smells)</div>
            <div class="glossary-grid">
                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>FAT_CONTROLLER</span>
                        <span class="glossary-item-tag">Tingkat Tinggi (High)</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Controller menampung terlalu banyak logika bisnis, manipulasi data rumit, atau query database langsung (&gt;300 baris atau query inline) alih-alih mendelegasikannya ke Service / Repository Layer.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Melanggar prinsip <em>Single Responsibility</em>. Controller menjadi kembung, sulit diuji secara independen, dan logikanya tidak dapat dipakai ulang (non-reusable).</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>GOD_CLASS_OR_BLOAT</span>
                        <span class="glossary-item-tag">Tingkat Tinggi (High)</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Satu class raksasa yang mencoba mengelola terlalu banyak tanggung jawab sekaligus (&gt;500 baris atau &gt;15 method), menjadi pusat ketergantungan seluruh modul.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Menjadi titik kegagalan tunggal (<em>Single Point of Failure</em>). Perubahan kecil pada satu fitur berisiko merusak fitur lainnya yang tidak terkait.</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>DEEP_NESTING</span>
                        <span class="glossary-item-tag">Tingkat Sedang / Tinggi</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Percabangan bersarang terlalu dalam (lebih dari 3-4 tingkat struktur <code>if / for / foreach</code>) membentuk kode piramida atau anak tangga.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Alur logika sangat sulit dipahami secara visual dan sering melewatkan kondisi batas (edge cases). Solusi: gunakan <em>Guard Clauses</em> (return early).</div>
                </div>

                <div class="glossary-item">
                    <div class="glossary-item-title">
                        <span>LONG_METHOD</span>
                        <span class="glossary-item-tag">Tingkat Sedang</span>
                    </div>
                    <div class="glossary-desc"><strong>Arti:</strong> Satu fungsi/method memiliki panjang baris kode berlebihan (&gt;50 baris), menandakan fungsi melakukan lebih dari satu tugas.</div>
                    <div class="glossary-impact"><strong>Dampak:</strong> Menyulitkan pengujian otomatis dan rawan menyembunyikan duplikasi algoritma. Solusi: terapkan refactoring <em>Extract Method</em>.</div>
                </div>
            </div>
        </section>

        <footer>
            Laporan dibuat otomatis oleh GreenCode Analyzer v1.1.0 &bull; Profiler Kualitas &amp; Efisiensi Perangkat Lunak
        </footer>
    </div>
</body>
</html>
"""
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_content)
        return output_path
