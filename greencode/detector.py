"""
Runtime & Process Detector
Mendeteksi proses server target (PHP, Node.js, Python, Go) secara otomatis
berdasarkan port listen, nama proses, atau berkas project di direktori kerja.
"""
import os
import psutil
from urllib.parse import urlparse

# Pola proses yang didukung per ekosistem
PROCESS_SIGNATURES = {
    'php': ['php', 'php.exe', 'php-fpm', 'httpd', 'httpd.exe', 'nginx', 'nginx.exe'],
    'node': ['node', 'node.exe', 'bun', 'bun.exe', 'deno', 'deno.exe'],
    'python': ['python', 'python.exe', 'python3', 'python3.exe', 'uvicorn', 'gunicorn'],
    'go': ['go', 'go.exe']  # ditambah executable binary kustom
}

class RuntimeDetector:
    @staticmethod
    def detect_project_ecosystem(project_dir="."):
        """
        Mendeteksi jenis bahasa pemrograman project berdasarkan file konfigurasi standar.
        """
        files = os.listdir(project_dir) if os.path.exists(project_dir) else []
        files_lower = [f.lower() for f in files]
        
        if 'composer.json' in files_lower or 'artisan' in files_lower:
            return 'php'
        if 'package.json' in files_lower:
            return 'node'
        if 'requirements.txt' in files_lower or 'pyproject.toml' in files_lower or 'setup.py' in files_lower:
            return 'python'
        if 'go.mod' in files_lower or 'main.go' in files_lower:
            return 'go'
        return 'auto'

    @staticmethod
    def extract_port_from_url(url):
        try:
            parsed = urlparse(url)
            if parsed.port:
                return parsed.port
            if parsed.scheme == 'https':
                return 443
            if parsed.scheme == 'http':
                return 80
        except Exception:
            pass
        return None

    @classmethod
    def find_pids(cls, language='auto', port=None, url=None, custom_process=None, project_dir="."):
        """
        Mencari PID proses server target.
        :param language: 'auto' | 'php' | 'node' | 'python' | 'go' | 'custom'
        :param port: port int (misal 8000, 3000)
        :param url: URL backend (misal 'http://127.0.0.1:8001')
        :param custom_process: nama binary spesifik (misal: 'my-go-server.exe')
        """
        if not port and url:
            port = cls.extract_port_from_url(url)
            
        if language == 'auto':
            language = cls.detect_project_ecosystem(project_dir)

        pids = []
        port_str = str(port) if port else None
        
        # 1. Prioritas A: Cari berdasarkan port di command line arguments
        if port_str:
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    name = (proc.info['name'] or '').lower()
                    cmd = ' '.join(proc.info['cmdline'] or []).lower()
                    
                    # Jika ada port di argumen (misal: --port 8001, :8001, artisan serve ... 8001)
                    if port_str in cmd or f":{port_str}" in cmd:
                        pids.append({
                            'pid': proc.pid,
                            'name': proc.info['name'],
                            'cmdline': cmd,
                            'matched_by': f'port {port_str}'
                        })
                except Exception:
                    pass

        # 2. Prioritas B: Jika belum ketemu, cari berdasarkan signature bahasa
        if not pids and language in PROCESS_SIGNATURES:
            signatures = PROCESS_SIGNATURES[language]
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    name = (proc.info['name'] or '').lower()
                    if any(sig in name for sig in signatures):
                        cmd = ' '.join(proc.info['cmdline'] or []).lower()
                        # Jika port ditentukan, cek apakah nama mencakup
                        pids.append({
                            'pid': proc.pid,
                            'name': proc.info['name'],
                            'cmdline': cmd,
                            'matched_by': f'signature {language}'
                        })
                except Exception:
                    pass

        # 3. Prioritas C: Custom Process Name (terutama untuk binary Go)
        if not pids and custom_process:
            c_proc = custom_process.lower()
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    name = (proc.info['name'] or '').lower()
                    if c_proc in name:
                        pids.append({
                            'pid': proc.pid,
                            'name': proc.info['name'],
                            'cmdline': ' '.join(proc.info['cmdline'] or []),
                            'matched_by': f'custom_name {custom_process}'
                        })
                except Exception:
                    pass

        return {
            'detected_language': language,
            'port': port,
            'processes': pids,
            'pids': [p['pid'] for p in pids]
        }

