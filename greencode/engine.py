"""
Process Energy Engine
Isolates CPU Active Time (user + system time) and RSS Memory consumption.
Language-Agnostic: Profiles any process (PHP, Node.js, Python, Go, etc.) without I/O wait bias.
"""
import time
import psutil

class ProcessEnergyEngine:
    def __init__(self, target_pids, cpu_tdp_watts=15.0, ram_watt_per_gb=0.375, carbon_intensity_g_per_kwh=850.0):
        """
        :param target_pids: List PID proses yang dipantau (misal: php.exe, node.exe, python.exe, go binary)
        :param cpu_tdp_watts: Estimasi daya TDP CPU aktif (default 15.0W utk Ultrabook/Mobile, bisa disesuaikan)
        :param ram_watt_per_gb: Estimasi daya dinamis RAM per GB (default 0.375W/GB)
        :param carbon_intensity_g_per_kwh: Faktor emisi karbon grid listrik (default 850 g CO2e / kWh utk PLN Jawa-Bali)
        """
        self.target_pids = list(target_pids)
        self.cpu_tdp_watts = float(cpu_tdp_watts)
        self.ram_watt_per_gb = float(ram_watt_per_gb)
        self.carbon_intensity = float(carbon_intensity_g_per_kwh)
        self._procs = []
        self._init_processes()

    def _init_processes(self):
        self._procs = []
        for pid in self.target_pids:
            try:
                p = psutil.Process(pid)
                if p.is_running():
                    self._procs.append(p)
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass

    def start_measurement(self):
        """Merekam snapshot titik awal (CPU times, RAM RSS, timestamp)."""
        self._init_processes()
        self.start_wall_time = time.perf_counter()
        self.start_cpu_user = 0.0
        self.start_cpu_sys = 0.0
        self.start_rss_mb = 0.0
        
        for p in self._procs:
            try:
                ct = p.cpu_times()
                self.start_cpu_user += ct.user
                self.start_cpu_sys += ct.system
                self.start_rss_mb += p.memory_info().rss / (1024.0 * 1024.0)
            except Exception:
                pass

    def stop_measurement(self):
        """
        Merekam snapshot titik akhir dan menghitung delta CPU murni, energi (Joule, kWh), dan emisi CO2e.
        Mengabaikan waktu tunggu socket/jaringan murni (I/O sleep).
        """
        end_wall_time = time.perf_counter()
        end_cpu_user = 0.0
        end_cpu_sys = 0.0
        end_rss_mb = 0.0
        
        for p in self._procs:
            try:
                ct = p.cpu_times()
                end_cpu_user += ct.user
                end_cpu_sys += ct.system
                end_rss_mb += p.memory_info().rss / (1024.0 * 1024.0)
            except Exception:
                pass
        
        wall_duration_s = max(end_wall_time - self.start_wall_time, 0.0001)
        delta_user_s = max(end_cpu_user - self.start_cpu_user, 0.0)
        delta_sys_s = max(end_cpu_sys - self.start_cpu_sys, 0.0)
        delta_cpu_active_s = delta_user_s + delta_sys_s
        avg_rss_mb = (self.start_rss_mb + end_rss_mb) / 2.0 if self.start_rss_mb > 0 else end_rss_mb

        # 1. Energi CPU Murni (Joule = Watt x Detik CPU Aktif)
        energy_cpu_joules = self.cpu_tdp_watts * delta_cpu_active_s

        # 2. Energi RAM Proporsional (Joule = (RSS_GB x Watt/GB) x Durasi)
        rss_gb = avg_rss_mb / 1024.0
        energy_ram_joules = (self.ram_watt_per_gb * rss_gb) * wall_duration_s

        total_joules = energy_cpu_joules + energy_ram_joules
        total_kwh = total_joules / 3600000.0
        emissions_kg_co2e = total_kwh * (self.carbon_intensity / 1000.0)

        # CPU load percentage process murni terhadap jam dinding
        proc_cpu_percent = (delta_cpu_active_s / wall_duration_s) * 100.0

        return {
            'wall_duration_s': round(wall_duration_s, 4),
            'cpu_active_time_s': round(delta_cpu_active_s, 4),
            'cpu_user_s': round(delta_user_s, 4),
            'cpu_system_s': round(delta_sys_s, 4),
            'proc_cpu_percent': round(proc_cpu_percent, 2),
            'avg_rss_mb': round(avg_rss_mb, 2),
            'energy_cpu_joules': round(energy_cpu_joules, 6),
            'energy_ram_joules': round(energy_ram_joules, 6),
            'energy_total_joules': round(total_joules, 6),
            'energy_total_kwh': total_kwh,
            'emissions_kg_co2e': emissions_kg_co2e
        }

