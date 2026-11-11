"""
Static-Green Correlation Engine
Menggabungkan konsumsi energi per-modul dengan metrik kualitas kode (CCN, LOC, Duplikasi, Smell).
Menghitung Green-Maintainability Index untuk evaluasi Before vs After Refactoring.
"""
import json

class GreenCorrelator:
    def __init__(self, static_summary_path):
        """
        :param static_summary_path: Path ke file summary.json hasil scan Lizard, Sonar, jscpd, PHPMD
        """
        with open(static_summary_path, 'r', encoding='utf-8') as f:
            self.static_data = json.load(f)
        self.per_module_static = self.static_data.get('perModule', {})

    def correlate(self, module_energy_summary):
        """
        Menggabungkan data energi per-modul dengan data statis.
        :param module_energy_summary: List of dict dari ModuleAttributor.get_summary_per_module()
        """
        correlated = []
        for em in module_energy_summary:
            slug = em['module']
            st = self.per_module_static.get(slug, {})
            
            loc = st.get('loc', 0)
            ccn = st.get('ccn', 0)
            avg_ccn = st.get('avgCcn', 0.0)
            max_ccn = st.get('maxCcn', 0)
            dup_pct = st.get('dupPct', 0.0)
            phpmd = st.get('phpmd', 0)
            files = st.get('files', 0)
            
            total_joules = em['total_joules']
            joules_per_req = em['joules_per_request']
            
            # 1. Energy per Complexity Ratio (Joule per unit CCN)
            joules_per_ccn = round(total_joules / max(ccn, 1), 6)
            
            # 2. Energy per 100 Lines of Code
            joules_per_100_loc = round((total_joules / max(loc, 1)) * 100.0, 6)
            
            # 3. Green-Maintainability Risk Level
            # Ditandai TINGGI jika CCN tinggi DAN konsumsi energi per request tinggi
            if max_ccn > 25 or joules_per_req > 0.05:
                risk_level = 'HIGH_PRIORITY'
            elif max_ccn > 15 or dup_pct > 20:
                risk_level = 'MEDIUM_PRIORITY'
            else:
                risk_level = 'LOW_PRIORITY'

            correlated.append({
                'module': slug,
                'files': files,
                'loc': loc,
                'total_ccn': ccn,
                'avg_ccn': avg_ccn,
                'max_ccn': max_ccn,
                'duplication_pct': dup_pct,
                'phpmd_violations': phpmd,
                'total_requests': em['total_requests'],
                'avg_latency_ms': em['avg_latency_ms'],
                'cpu_active_time_s': em['total_cpu_active_s'],
                'total_joules': total_joules,
                'joules_per_request': joules_per_req,
                'joules_per_ccn': joules_per_ccn,
                'joules_per_100_loc': joules_per_100_loc,
                'refactoring_risk_level': risk_level
            })
            
        return sorted(correlated, key=lambda x: x['total_joules'], reverse=True)

