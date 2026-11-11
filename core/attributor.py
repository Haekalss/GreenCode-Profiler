"""
Request & Module Attribution Tracker
Memetakan konsumsi energi terisolasi ke masing-masing Endpoint dan Sub-Modul aplikasi secara granular.
"""
import urllib.request
import urllib.error
import time

class ModuleAttributor:
    def __init__(self, energy_engine, module_mapping=None):
        """
        :param energy_engine: Instance dari ProcessEnergyEngine
        :param module_mapping: Dict {path_keyword: module_slug}
        """
        self.engine = energy_engine
        self.module_mapping = module_mapping or self._default_smki_mapping()
        self.endpoint_records = []

    def _default_smki_mapping(self):
        return {
            'laporan-kerusakan': 'laporan-kerusakan',
            'daftar-rekaman': 'daftar-rekaman',
            'notulen': 'notulen',
            'evaluasi-penyedia': 'evaluasi-penyedia',
            'review-hak-akses': 'review-hak-akses',
            'berita-acara': 'berita-acara',
            'software-standar': 'software-standar',
            'laporan-audit': 'laporan-audit',
            'hak-akses-fisik': 'hak-akses-fisik',
            'formulir-hardening': 'formulir-hardening',
            'matrix-competency': 'matrix-competency',
            'perubahan-it': 'perubahan-it',
            'pelaporan-insiden': 'pelaporan-insiden'
        }

    def detect_module(self, path):
        path_lower = path.lower()
        for kw, slug in self.module_mapping.items():
            if kw in path_lower:
                return slug
        return 'shared-smki'

    def trace_http_get(self, url, headers=None, expected_status=200):
        """
        Menjalankan 1 request HTTP seraya mengisolasi pengukuran energi murni untuk request tersebut.
        """
        mod_slug = self.detect_module(url)
        headers = headers or {}
        req = urllib.request.Request(url, headers=headers, method='GET')
        
        # Mulai pengukuran energi terisolasi untuk request ini
        self.engine.start_measurement()
        t0 = time.perf_counter()
        
        status = 0
        body_len = 0
        error_msg = None
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                status = resp.status
                data = resp.read()
                body_len = len(data)
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

        record = {
            'url': url,
            'module': mod_slug,
            'status': status,
            'latency_ms': round(latency_ms, 2),
            'body_bytes': body_len,
            'success': (status == expected_status) if expected_status else (status < 400),
            'error': error_msg,
            **energy_metrics
        }
        self.endpoint_records.append(record)
        return record

    def get_summary_per_module(self):
        """Mengelompokkan konsumsi energi dan metrik per sub-modul."""
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
                    'avg_latency_ms': 0.0,
                    'latencies': [],
                    'avg_rss_mb': 0.0
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
            g['avg_rss_mb'] = r['avg_rss_mb']

        summary = []
        for m, g in grouped.items():
            lats = g['latencies']
            summary.append({
                'module': m,
                'total_requests': g['total_requests'],
                'successful_requests': g['successful_requests'],
                'avg_latency_ms': round(sum(lats) / len(lats), 2) if lats else 0.0,
                'total_cpu_active_s': round(g['total_cpu_active_s'], 4),
                'total_joules': round(g['total_joules'], 6),
                'total_kwh': g['total_kwh'],
                'total_co2e_kg': g['total_co2e_kg'],
                'joules_per_request': round(g['total_joules'] / g['total_requests'], 6) if g['total_requests'] else 0.0,
                'avg_rss_mb': g['avg_rss_mb']
            })
        return sorted(summary, key=lambda x: x['total_joules'], reverse=True)

