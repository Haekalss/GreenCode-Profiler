"""
Universal Request & Module Attributor
Menjalankan request HTTP (GET, POST, PUT, DELETE) dengan isolasi energi per-endpoint.
Mendukung semua bahasa backend (PHP, Node.js, Python, Go) tanpa modifikasi server.
"""
import urllib.request
import urllib.error
from urllib.parse import urlparse
import time
import json

class ModuleAttributor:
    def __init__(self, energy_engine, module_mapping=None):
        """
        :param energy_engine: Instance ProcessEnergyEngine
        :param module_mapping: Dict kustom {'keyword': 'module_name'}
        """
        self.engine = energy_engine
        self.module_mapping = module_mapping or {}
        self.endpoint_records = []

    def detect_module(self, path):
        """Mendeteksi nama modul dari path URL secara otomatis atau via mapping."""
        path_lower = path.lower()
        # 1. Cek kustom mapping
        for kw, slug in self.module_mapping.items():
            if kw.lower() in path_lower:
                return slug
        
        # 2. Heuristik otomatis: ambil segmen path bermakna
        parsed = urlparse(path)
        segments = [s for s in parsed.path.strip('/').split('/') if s and s not in ['api', 'v1', 'v2', 'v3', 'index.php']]
        if segments:
            return segments[0]
        return 'root'

    def trace_http_request(self, url, method='GET', headers=None, body=None, expected_status=None, module_name=None):
        """
        Menjalankan 1 request HTTP terisolasi, merekam latency, CPU active time, dan Joule.
        """
        method = method.upper()
        headers = dict(headers or {})
        mod_slug = module_name or self.detect_module(url)

        data_bytes = None
        if body is not None:
            if isinstance(body, dict):
                data_bytes = json.dumps(body).encode('utf-8')
                if 'Content-Type' not in headers:
                    headers['Content-Type'] = 'application/json'
            elif isinstance(body, str):
                data_bytes = body.encode('utf-8')
            elif isinstance(body, bytes):
                data_bytes = body

        req = urllib.request.Request(url, data=data_bytes, headers=headers, method=method)
        
        # Mulai isolasi pengukuran energi murni
        self.engine.start_measurement()
        t0 = time.perf_counter()

        status = 0
        body_len = 0
        error_msg = None

        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                status = resp.status
                content = resp.read()
                body_len = len(content)
        except urllib.error.HTTPError as e:
            status = e.code
            try:
                body_len = len(e.read())
            except Exception:
                pass
        except Exception as e:
            error_msg = str(e)

        latency_ms = (time.perf_counter() - t0) * 1000.0
        energy_metrics = self.engine.stop_measurement()

        is_success = (status == expected_status) if expected_status else (status > 0 and status < 400)

        record = {
            'url': url,
            'method': method,
            'module': mod_slug,
            'status': status,
            'latency_ms': round(latency_ms, 2),
            'body_bytes': body_len,
            'success': is_success,
            'error': error_msg,
            **energy_metrics
        }
        self.endpoint_records.append(record)
        return record

    def get_summary_per_module(self):
        """Mengagregasikan konsumsi energi dan performa per modul."""
        grouped = {}
        for r in self.endpoint_records:
            m = r['module']
            if m not in grouped:
                grouped[m] = {
                    'module': m,
                    'total_requests': 0,
                    'successful_requests': 0,
                    'total_joules': 0.0,
                    'total_kwh': 0.0,
                    'total_co2e_kg': 0.0,
                    'total_cpu_active_s': 0.0,
                    'latencies': [],
                    'joules_list': []
                }
            g = grouped[m]
            g['total_requests'] += 1
            if r['success']:
                g['successful_requests'] += 1
            g['total_joules'] += r['energy_total_joules']
            g['total_kwh'] += r['energy_total_kwh']
            g['total_co2e_kg'] += r['emissions_kg_co2e']
            g['total_cpu_active_s'] += r['cpu_active_time_s']
            g['latencies'].append(r['latency_ms'])
            g['joules_list'].append(r['energy_total_joules'])

        summary = []
        for m, data in grouped.items():
            n = data['total_requests']
            avg_lat = sum(data['latencies']) / n if n > 0 else 0.0
            avg_j = data['total_joules'] / n if n > 0 else 0.0
            summary.append({
                'module': m,
                'total_requests': n,
                'successful_requests': data['successful_requests'],
                'avg_latency_ms': round(avg_lat, 2),
                'total_cpu_active_s': round(data['total_cpu_active_s'], 4),
                'total_joules': round(data['total_joules'], 6),
                'joules_per_request': round(avg_j, 6),
                'total_kwh': data['total_kwh'],
                'total_co2e_kg': data['total_co2e_kg']
            })
        return sorted(summary, key=lambda x: x['total_joules'], reverse=True)

