"""
GreenCode Profiler (GCP) CLI Runner
Menjalankan profiling energi terisolasi per proses dan per modul SMKI.
Mengintegrasikan konsumsi Joule dengan metrik Cyclomatic Complexity dan LOC.
"""
import argparse, json, os, sys, psutil
from core.engine import ProcessEnergyEngine
from core.attributor import ModuleAttributor
from core.correlator import GreenCorrelator

def find_target_pids(pattern='8001'):
    pids = []
    for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
        try:
            name = (proc.info['name'] or '').lower()
            cmd = ' '.join(proc.info['cmdline'] or []).lower()
            if 'php' in name and pattern in cmd:
                pids.append(proc.pid)
        except Exception:
            pass
    return pids

def login_backend(base_url, email, password):
    import urllib.request
    body = json.dumps({'email': email, 'password': password}).encode('utf-8')
    req = urllib.request.Request(base_url + '/api/login', data=body, headers={'Content-Type': 'application/json'}, method='POST')
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            data = json.loads(resp.read().decode('utf-8'))
            return data.get('access_token')
    except Exception as e:
        print(f"Gagal login ke {base_url}: {e}")
        return None

def main():
    parser = argparse.ArgumentParser(description="GreenCode Profiler Framework CLI")
    parser.add_argument('--base-url', default='http://127.0.0.1:8001', help='Base URL backend Laravel')
    parser.add_argument('--static-summary', default='../smki-analysis/before/results/summary.json', help='Path ke summary.json statis')
    parser.add_argument('--repeats', type=int, default=5, help='Jumlah perulangan request per endpoint')
    parser.add_argument('--output', default='reports/greencode_before_report.json', help='Path file JSON output')
    args = parser.parse_args()

    print("================================================================")
    print("      GREENCODE PROFILER (GCP) FRAMEWORK v1.0 (PROTOTYPE)       ")
    print("================================================================")

    # 1. Temukan proses PHP target
    pids = find_target_pids('8001')
    if not pids:
        print("PERINGATAN: Tidak menemukan proses PHP yang mendengarkan port 8001.")
        print("Mencari proses php.exe aktif apapun...")
        pids = [p.pid for p in psutil.process_iter(['name']) if 'php' in (p.info['name'] or '').lower()]
    
    print(f"Target PID PHP terdeteksi: {pids}")
    if not pids:
        print("ERROR: Proses PHP tidak ditemukan. Pastikan server backend aktif.")
        sys.exit(1)

    # 2. Inisialisasi Engine & Attributor
    engine = ProcessEnergyEngine(pids, cpu_tdp_watts=15.0, ram_watt_per_gb=0.375)
    attributor = ModuleAttributor(engine)

    # 3. Autentikasi
    print(f"Melakukan autentikasi ke: {args.base_url}/api/login...")
    token = login_backend(args.base_url, 'user@aptika.com', 'password')
    if not token:
        print("Gagal mendapatkan access token.")
        sys.exit(1)
    print("Autentikasi Berhasil. Token didapatkan.")

    headers = {'Authorization': f'Bearer {token}', 'Accept': 'application/json'}

    # 4. Daftar Endpoint SMKI yang Diuji (Representatif Sub-Modul Hotspot)
    endpoints = [
        {'path': '/api/smki/laporan-kerusakan', 'name': 'List Laporan Kerusakan', 'is_docx': False},
        {'path': '/api/smki/daftar-rekaman', 'name': 'List Daftar Rekaman', 'is_docx': False},
        {'path': '/api/smki/notulen', 'name': 'List Notulen', 'is_docx': False},
        {'path': '/api/smki/evaluasi-penyedia', 'name': 'List Evaluasi Penyedia', 'is_docx': False},
        {'path': '/api/smki/review-hak-akses', 'name': 'List Review Hak Akses', 'is_docx': False},
        {'path': '/api/smki/daftar-rekaman/export-docx', 'name': 'Export DOCX Rekaman', 'is_docx': True},
        {'path': '/api/smki/berita-acara/export-docx', 'name': 'Export DOCX Berita Acara', 'is_docx': True},
        {'path': '/api/smki/software-standar/export-docx', 'name': 'Export DOCX Software Standar', 'is_docx': True}
    ]

    print(f"\nMemulai profiling {len(endpoints)} endpoints x {args.repeats} perulangan...")

    for ep in endpoints:
        url = args.base_url + ep['path']
        ep_hdrs = dict(headers)
        if ep['is_docx']:
            ep_hdrs['Accept'] = '*/*'
        
        print(f"\n-> Menguji: {ep['name']} ({ep['path']})")
        for i in range(1, args.repeats + 1):
            rec = attributor.trace_http_get(url, headers=ep_hdrs, expected_status=200)
            print(f"   [Run {i}/{args.repeats}] status={rec['status']} | lat={rec['latency_ms']:6.1f}ms | CPU Active={rec['cpu_active_time_s']:6.4f}s | Energy={rec['energy_total_joules']:8.6f} J")

    # 5. Agregasi Per Modul
    module_summary = attributor.get_summary_per_module()

    # 6. Korelasikan dengan Metrik Statis (Komponen 3)
    correlated_summary = []
    if os.path.exists(args.static_summary):
        print(f"\nMengkorelasikan data energi dengan metrik struktur statis dari: {args.static_summary}...")
        correlator = GreenCorrelator(args.static_summary)
        correlated_summary = correlator.correlate(module_summary)
    else:
        print(f"File summary statis tidak ditemukan di {args.static_summary}, melewatkan korelasi statis.")

    # 7. Simpan Output Laporan Terpadu
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    report_data = {
        'framework_name': 'GreenCode Profiler (GCP)',
        'version': '1.0.0-prototype',
        'target_pids': pids,
        'repeats_per_endpoint': args.repeats,
        'raw_request_records': attributor.endpoint_records,
        'module_energy_summary': module_summary,
        'green_maintainability_correlation': correlated_summary
    }

    with open(args.output, 'w', encoding='utf-8') as f:
        json.dump(report_data, f, indent=2)

    print(f"\n[SUKSES] Laporan lengkap tersimpan di: {args.output}")

    # Cetak Tabel Ringkasan ke Terminal
    print("\n" + "="*85)
    print("HASIL EVALUASI GREEN-MAINTAINABILITY PER SUB-MODUL (FRAMEWORK BUATAN SENDIRI)")
    print("="*85)
    print(f"{'Sub-Modul':<24} | {'Req':<4} | {'Avg Lat':<8} | {'CPU Act':<8} | {'Total J':<9} | {'J/Req':<9} | {'Max CCN':<7} | {'Risk Level'}")
    print("-"*85)
    for c in (correlated_summary or module_summary):
        mod = c['module']
        req = c['total_requests']
        lat = f"{c.get('avg_latency_ms', 0):.1f}ms"
        cpu = f"{c.get('cpu_active_time_s', 0):.3f}s"
        joules = f"{c.get('total_joules', 0):.4f}J"
        j_req = f"{c.get('joules_per_request', 0):.4f}J"
        m_ccn = str(c.get('max_ccn', '-'))
        risk = c.get('refactoring_risk_level', '-')
        print(f"{mod:<24} | {req:<4} | {lat:<8} | {cpu:<8} | {joules:<9} | {j_req:<9} | {m_ccn:<7} | {risk}")
    print("="*85)

if __name__ == '__main__':
    main()

