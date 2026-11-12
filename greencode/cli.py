"""
GreenCode Profiler (GCP) Universal CLI
Command line interface for profiling PHP, Node.js, Python, and Go web applications.
Complete Package: Static Code Quality Analysis + Dynamic Process Energy Profiling.
"""
import argparse
import sys
import os
import json
import webbrowser
from datetime import datetime

# Handle both direct script execution and installed package execution
if __package__ is None or __package__ == "":
    sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    from greencode.engine import ProcessEnergyEngine
    from greencode.detector import RuntimeDetector, RouteDiscoverer
    from greencode.attributor import ModuleAttributor
    from greencode.correlator import GreenCorrelator
    from greencode.analyzer import CodeQualityAnalyzer
    from greencode.reporter import ReportGenerator
else:
    from .engine import ProcessEnergyEngine
    from .detector import RuntimeDetector, RouteDiscoverer
    from .attributor import ModuleAttributor
    from .correlator import GreenCorrelator
    from .analyzer import CodeQualityAnalyzer
    from .reporter import ReportGenerator

PRESETS = {
    'php': {
        "project_name": "PHP / Laravel Backend",
        "runtime": "php",
        "source_dir": "app",
        "base_url": "http://127.0.0.1:8000",
        "auth": {
            "type": "bearer",
            "token": ""
        },
        "endpoints": [
            {"path": "/api/users", "name": "Get Users", "method": "GET"},
            {"path": "/api/reports", "name": "Generate Report", "method": "GET"}
        ],
        "repeats": 5,
        "cpu_tdp_watts": 15.0,
        "static_summary": ""
    },
    'node': {
        "project_name": "Node.js / Next.js / Express App",
        "runtime": "node",
        "source_dir": "src",
        "base_url": "http://127.0.0.1:3000",
        "auth": {
            "type": "none"
        },
        "endpoints": [
            {"path": "/api/hello", "name": "Hello API", "method": "GET"},
            {"path": "/api/items", "name": "List Items", "method": "GET"}
        ],
        "repeats": 5,
        "cpu_tdp_watts": 15.0,
        "static_summary": ""
    },
    'python': {
        "project_name": "Python / FastAPI / Django API",
        "runtime": "python",
        "source_dir": ".",
        "base_url": "http://127.0.0.1:8000",
        "auth": {
            "type": "none"
        },
        "endpoints": [
            {"path": "/items", "name": "Get Items", "method": "GET"},
            {"path": "/predict", "name": "Inference / Task", "method": "POST", "body": {"data": "test"}}
        ],
        "repeats": 5,
        "cpu_tdp_watts": 15.0,
        "static_summary": ""
    },
    'go': {
        "project_name": "Go / Gin / Fiber Microservice",
        "runtime": "go",
        "source_dir": ".",
        "base_url": "http://127.0.0.1:8080",
        "auth": {
            "type": "none"
        },
        "endpoints": [
            {"path": "/health", "name": "Health Check", "method": "GET"},
            {"path": "/api/v1/data", "name": "Process Data", "method": "GET"}
        ],
        "repeats": 5,
        "cpu_tdp_watts": 15.0,
        "static_summary": ""
    }
}

def cmd_init(args):
    """Membuat file konfigurasi greencode.json awal berdasarkan preset bahasa dan auto-discover rute."""
    preset_key = (args.preset or 'auto').lower()
    if preset_key == 'auto':
        detected = RuntimeDetector.detect_project_ecosystem()
        preset_key = detected if detected != 'auto' else 'php'
        print(f"[GreenCode] Mendeteksi ekosistem project: {preset_key.upper()}")

    template = dict(PRESETS.get(preset_key, PRESETS['php']))
    
    # Deteksi rute/endpoint nyata dari codebase secara otomatis!
    discovered_routes = RouteDiscoverer.discover(".", max_routes=10)
    if discovered_routes:
        template["endpoints"] = discovered_routes
        print(f"[GreenCode] Berhasil mendeteksi {len(discovered_routes)} rute nyata dari project secara otomatis!")
    else:
        print("[GreenCode] Tidak menemukan rute otomatis, menggunakan contoh template.")

    output_path = args.output or "greencode.json"
    
    if os.path.exists(output_path) and not args.force:
        print(f"[GreenCode] Berkas '{output_path}' sudah ada. Gunakan flag --force untuk menimpa.")
        return

    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(template, f, indent=2)
    
    print(f"[GreenCode] Sukses membuat template konfigurasi: {output_path}")
    print(f"            Preset : {preset_key.upper()}")
    print("            Jalankan 'greencode run' untuk memulai profiling!")

def cmd_detect(args):
    """Mendiagnosa dan menampilkan proses server yang sedang aktif."""
    print("================================================================")
    print("           GREENCODE RUNTIME PROCESS DETECTOR                   ")
    print("================================================================")
    
    det = RuntimeDetector.find_pids(language=args.language, port=args.port, url=args.url)
    print(f"Target Language : {det['detected_language'].upper()}")
    print(f"Target Port     : {det['port'] or 'Semua'}")
    print(f"Ditemukan       : {len(det['pids'])} proses aktif")
    print("----------------------------------------------------------------")
    for p in det['processes']:
        print(f" - PID {p['pid']:<6} | {p['name']:<15} | Cocok via: {p['matched_by']}")
        print(f"   Cmd: {p['cmdline'][:90]}...")
    print("================================================================")

def cmd_analyze(args):
    """Menganalisis kualitas struktur kode statis (NCLOC, CCN, MI, Code Smells)."""
    target_path = args.path or "."
    print("================================================================")
    print(f"      GREENCODE STATIC CODE QUALITY ANALYZER                    ")
    print("================================================================")
    print(f"Scanning Directory : {os.path.abspath(target_path)}")

    analyzer = CodeQualityAnalyzer(target_path)
    res = analyzer.run_analysis()
    ov = res['overview']

    print("----------------------------------------------------------------")
    print(f"Total Files Scanned      : {ov['total_files']} files")
    print(f"Total NCLOC              : {ov['total_ncloc']:,} lines")
    print(f"Total Complexity (CCN)   : {ov['total_ccn']}")
    print(f"Average CCN per File     : {ov['average_ccn_per_file']}")
    print(f"Maintainability Index    : {ov['average_maintainability_index']} / 100 (Grade: {ov['maintainability_grade']})")
    print(f"Architectural Smells     : {ov['total_code_smells']} detected")
    print("----------------------------------------------------------------")

    if res['top_complex_functions']:
        print("\n[TOP 5 HOTSPOT FUNGSI PALING KOMPLEKS (CCN TINGGI)]:")
        for fn in res['top_complex_functions'][:5]:
            print(f" - {fn['name']:<25} | CCN: {fn['ccn']:<3} | {fn['file']}:{fn['line']}")

    if res['code_smells']:
        print("\n[CONTOH CODE SMELLS TERDETEKSI]:")
        for sm in res['code_smells'][:5]:
            print(f" ! [{sm['severity']}] {sm['type']}: {sm['message'][:80]}...")

    output_json = os.path.join("reports", "code_quality.json")
    output_html = os.path.join("reports", "code_quality.html")
    ReportGenerator.save_json(res, output_json)
    ReportGenerator.generate_code_quality_html(res, output_html)
    print(f"\nHasil JSON tersimpan di : {os.path.abspath(output_json)}")
    print(f"Laporan HTML tersimpan di: {os.path.abspath(output_html)}")
    print("================================================================\n")
    if not getattr(args, 'no_open', False):
        ReportGenerator.open_in_browser(output_html)
    return res

def cmd_ui(args):
    """Membuka dashboard HTML di web browser."""
    report_file = args.report or os.path.join("reports", "greencode_report.html")
    if not os.path.exists(report_file):
        # Coba buka index.html di docs jika report belum ada
        docs_file = os.path.join("docs", "index.html")
        if os.path.exists(docs_file):
            report_file = docs_file
        else:
            print(f"[GreenCode Error] File report '{report_file}' belum ditemukan.")
            print("                  Jalankan 'greencode run' terlebih dahulu.")
            return

    print(f"[GreenCode] Membuka Dashboard di Web Browser: {os.path.abspath(report_file)}")
    ReportGenerator.open_in_browser(report_file)

def cmd_run(args):
    """Menjalankan profiling lengkap (Kualitas Kode + Energi Runtime) & membuat Dashboard."""
    config_file = args.config
    if not os.path.exists(config_file):
        print(f"[GreenCode] Berkas konfigurasi '{config_file}' tidak ditemukan.")
        print("            Jalankan 'greencode init' untuk membuatnya otomatis.")
        sys.exit(1)

    with open(config_file, 'r', encoding='utf-8') as f:
        cfg = json.load(f)

    project_name = cfg.get('project_name', 'Web Service')
    runtime = cfg.get('runtime', 'auto')
    base_url = cfg.get('base_url', 'http://127.0.0.1:8000')
    repeats = cfg.get('repeats', 5)
    cpu_tdp = cfg.get('cpu_tdp_watts', 15.0)
    endpoints = cfg.get('endpoints', [])
    auth = cfg.get('auth', {})
    static_summary = cfg.get('static_summary', None)
    source_dir = cfg.get('source_dir', '.')

    print("================================================================")
    print(f"   GREENCODE PROFILER v1.0 - {project_name.upper()}   ")
    print("================================================================")
    print(f"Base URL : {base_url}")
    print(f"Runtime  : {runtime.upper()}")
    print(f"Repeats  : {repeats}x per endpoint")
    print("----------------------------------------------------------------")

    # 1. Analisis Kualitas Struktur Kode (Built-in Analyzer)
    code_quality_data = {}
    if os.path.exists(source_dir):
        print(f"[GreenCode] [1/3] Menjalankan Analisis Kualitas Struktur Kode pada: {source_dir}...")
        analyzer = CodeQualityAnalyzer(source_dir)
        code_quality_data = analyzer.run_analysis()
        ov = code_quality_data['overview']
        print(f"            Files: {ov['total_files']} | NCLOC: {ov['total_ncloc']:,} | Avg CCN: {ov['average_ccn_per_file']} | MI: {ov['average_maintainability_index']}/100")
    else:
        print(f"[GreenCode] Catatan: Direktori source '{source_dir}' tidak ditemukan. Melewati static code analysis.")

    # 2. Deteksi Target PIDs
    print(f"[GreenCode] [2/3] Mendeteksi proses server aktif untuk {base_url}...")
    det = RuntimeDetector.find_pids(language=runtime, url=base_url)
    pids = det['pids']
    
    if not pids:
        print(f"[GreenCode] PERINGATAN: Tidak ada proses {runtime.upper()} terdeteksi di {base_url}.")
        print("            Pastikan server aplikasi Anda sudah dijalankan.")
        sys.exit(1)

    print(f"[GreenCode] Terhubung ke {len(pids)} proses aktif: PID {pids}")

    # 2.5 Auto-Discover Endpoints jika kosong atau masih dummy contoh template (/api/hello, /api/items)
    is_dummy = endpoints and all(ep.get('path') in ['/api/hello', '/api/items'] for ep in endpoints)
    if not endpoints or is_dummy:
        print("[GreenCode] Mendeteksi rute/halaman nyata dari project secara otomatis...")
        discovered = RouteDiscoverer.discover(".", max_routes=10)
        if discovered:
            endpoints = discovered
            print(f"[GreenCode] Menggunakan {len(endpoints)} rute nyata yang terdeteksi otomatis!")

    # 3. Setup Headers
    headers = {'Accept': 'application/json'}
    if auth.get('type') == 'bearer' and auth.get('token'):
        headers['Authorization'] = f"Bearer {auth['token']}"
    elif auth.get('headers'):
        headers.update(auth['headers'])

    # 4. Jalankan Pengukuran Energi Runtime
    engine = ProcessEnergyEngine(pids, cpu_tdp_watts=cpu_tdp)
    attributor = ModuleAttributor(engine)

    print(f"\n[GreenCode] [3/3] Memulai profiling runtime {len(endpoints)} endpoint...")
    for ep in endpoints:
        path = ep.get('path', '/')
        name = ep.get('name', path)
        method = ep.get('method', 'GET')
        body = ep.get('body', None)
        url = base_url + path

        print(f"\n-> [{method}] {name} ({path})")
        for i in range(1, repeats + 1):
            rec = attributor.trace_http_request(url, method=method, headers=headers, body=body)
            print(f"   Run {i}/{repeats} | HTTP {rec['status']:<3} | Latency: {rec['latency_ms']:6.1f} ms | CPU: {rec['cpu_active_time_s']:6.4f} s | Energi: {rec['energy_total_joules']:8.6f} J")

    # 5. Korelasi Metrik Green-Maintainability
    mod_summary = attributor.get_summary_per_module()
    correlator = GreenCorrelator(static_summary)
    correlated = correlator.correlate(mod_summary)

    # 6. Hitung Grand Totals
    total_joules = sum(r['energy_total_joules'] for r in attributor.endpoint_records)
    total_cpu_s = sum(r['cpu_active_time_s'] for r in attributor.endpoint_records)
    total_kwh = total_joules / 3600000.0
    total_co2e_g = total_kwh * 850.0
    avg_lat = sum(r['latency_ms'] for r in attributor.endpoint_records) / max(len(attributor.endpoint_records), 1)

    report_data = {
        'metadata': {
            'project_name': project_name,
            'runtime': runtime,
            'base_url': base_url,
            'timestamp': datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            'cpu_tdp_watts': cpu_tdp
        },
        'totals': {
            'total_requests': len(attributor.endpoint_records),
            'total_joules': total_joules,
            'total_kwh': total_kwh,
            'total_cpu_active_s': total_cpu_s,
            'avg_latency_ms': avg_lat,
            'total_co2e_g': total_co2e_g
        },
        'correlated_modules': correlated,
        'code_quality': code_quality_data,
        'endpoints_detail': attributor.endpoint_records
    }

    # 7. Simpan Laporan & Buka Dashboard
    json_path = os.path.join("reports", "greencode_report.json")
    html_path = os.path.join("reports", "greencode_report.html")

    ReportGenerator.save_json(report_data, json_path)
    ReportGenerator.generate_html(report_data, html_path)

    print("\n================================================================")
    print("                     PROFILING SELESAI                          ")
    print("================================================================")
    print(f"Total Energi Terukur : {total_joules:.6f} Joules ({total_kwh:.8f} kWh)")
    print(f"Total CPU Aktif      : {total_cpu_s:.4f} detik")
    print(f"Jejak Karbon Bersih  : {total_co2e_g:.6f} gram CO2e")
    print("----------------------------------------------------------------")
    print(f"Laporan JSON      : {os.path.abspath(json_path)}")
    print(f"Dashboard Web UI  : {os.path.abspath(html_path)}")
    print("================================================================\n")

    if not args.no_open:
        ReportGenerator.open_in_browser(html_path)

def cmd_routes(args):
    """Menampilkan semua rute yang otomatis terdeteksi dari codebase."""
    target_path = args.path or "."
    limit = getattr(args, 'limit', 30) or 30
    routes = RouteDiscoverer.discover(target_path, max_routes=limit)
    print("================================================================")
    print("      GREENCODE AUTOMATIC ROUTE DISCOVERY                       ")
    print("================================================================")
    print(f"Direktori Target : {os.path.abspath(target_path)}")
    print(f"Total Ditemukan  : {len(routes)} rute nyata")
    print("----------------------------------------------------------------")
    for r in routes:
        print(f" - [{r.get('method', 'GET'):<4}] {r['path']:<35} ({r.get('name', '')})")
    print("================================================================\n")

def main():
    parser = argparse.ArgumentParser(
        prog="greencode",
        description="GreenCode Profiler - Multi-Language Process-Level Energy & Code Quality Profiler"
    )
    subparsers = parser.add_subparsers(dest="command")

    # Subcommand: init
    p_init = subparsers.add_parser("init", help="Generate template konfigurasi greencode.json")
    p_init.add_argument("--preset", choices=["php", "node", "python", "go", "auto"], default="auto", help="Preset bahasa target")
    p_init.add_argument("--output", "-o", default="greencode.json", help="Path file output")
    p_init.add_argument("--force", "-f", action="store_true", help="Timpa file jika sudah ada")

    # Subcommand: detect
    p_detect = subparsers.add_parser("detect", help="Diagnosa proses server aktif")
    p_detect.add_argument("--language", "-l", choices=["php", "node", "python", "go", "auto"], default="auto")
    p_detect.add_argument("--port", "-p", type=int, help="Port server yang ingin dicek")
    p_detect.add_argument("--url", "-u", help="URL server (misal: http://127.0.0.1:8000)")

    # Subcommand: analyze
    p_analyze = subparsers.add_parser("analyze", help="Analisis kualitas struktur kode statis (NCLOC, CCN, MI, Code Smells)")
    p_analyze.add_argument("path", nargs="?", default=".", help="Direktori kode yang ingin dianalisis (default: .)")
    p_analyze.add_argument("--no-open", action="store_true", help="Jangan otomatis buka browser setelah selesai")

    # Subcommand: ui / dashboard
    p_ui = subparsers.add_parser("ui", help="Buka dashboard HTML hasil profiling di browser")
    p_ui.add_argument("--report", "-r", help="Path file HTML report kustom")

    # Subcommand: routes / discover
    p_routes = subparsers.add_parser("routes", help="Deteksi otomatis rute/endpoint dari codebase project")
    p_routes.add_argument("path", nargs="?", default=".", help="Direktori project yang ingin dipindai (default: .)")
    p_routes.add_argument("--limit", type=int, default=30, help="Batas maksimal rute yang ditampilkan")

    # Subcommand: run
    p_run = subparsers.add_parser("run", help="Jalankan profiling lengkap dari file konfigurasi")
    p_run.add_argument("config", nargs="?", default="greencode.json", help="Path file konfigurasi (default: greencode.json)")
    p_run.add_argument("--no-open", action="store_true", help="Jangan otomatis buka browser setelah selesai")

    # Subcommand: help
    p_help = subparsers.add_parser("help", help="Tampilkan panduan bantuan perintah")
    p_help.add_argument("subcommand", nargs="?", choices=["init", "run", "analyze", "detect", "ui"], help="Nama perintah spesifik")

    args = parser.parse_args()

    if args.command == "init":
        cmd_init(args)
    elif args.command == "detect":
        cmd_detect(args)
    elif args.command == "analyze":
        cmd_analyze(args)
    elif args.command == "routes":
        cmd_routes(args)
    elif args.command == "ui":
        cmd_ui(args)
    elif args.command == "help":
        if args.subcommand and args.subcommand in subparsers.choices:
            subparsers.choices[args.subcommand].print_help()
        else:
            parser.print_help()
    elif args.command == "run" or (args.command is None and len(sys.argv) == 1):
        args.config = "greencode.json"
        args.no_open = False
        cmd_run(args)
    else:
        parser.print_help()

if __name__ == "__main__":
    main()
