"""
Runtime & Process Detector + Automatic Route Discoverer
1. Mendeteksi proses server target (PHP, Node.js, Python, Go) secara otomatis berdasarkan port/proses.
2. Mendeteksi rute & endpoint nyata dari codebase aplikasi secara otomatis (Next.js, Laravel, Express, FastAPI, Go)
   sehingga developer tidak perlu mengedit JSON secara manual!
"""
import os
import re
import psutil
from urllib.parse import urlparse

# Pola proses yang didukung per ekosistem
PROCESS_SIGNATURES = {
    'php': ['php', 'php.exe', 'php-fpm', 'httpd', 'httpd.exe', 'nginx', 'nginx.exe'],
    'node': ['node', 'node.exe', 'bun', 'bun.exe', 'deno', 'deno.exe'],
    'python': ['python', 'python.exe', 'python3', 'python3.exe', 'uvicorn', 'gunicorn'],
    'go': ['go', 'go.exe']
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
                        pids.append({
                            'pid': proc.pid,
                            'name': proc.info['name'],
                            'cmdline': cmd,
                            'matched_by': f'signature {language}'
                        })
                except Exception:
                    pass

        # 3. Prioritas C: Custom Process Name
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


class RouteDiscoverer:
    """
    Penganalisis rute cerdas: Memindai codebase dan mengekstrak rute/endpoint nyata secara otomatis.
    Mendukung: Next.js (App & Pages Router), Laravel, Express.js, FastAPI, dan Go.
    """
    @classmethod
    def discover(cls, project_dir=".", max_routes=12):
        project_dir = os.path.abspath(project_dir)
        eco = RuntimeDetector.detect_project_ecosystem(project_dir)

        routes = []

        # 1. Deteksi Next.js (App Router atau Pages Router)
        next_routes = cls._discover_nextjs(project_dir)
        if next_routes:
            return next_routes[:max_routes]

        # 2. Deteksi Laravel / PHP
        laravel_routes = cls._discover_laravel(project_dir)
        if laravel_routes:
            return laravel_routes[:max_routes]

        # 3. Deteksi Express / Node.js
        express_routes = cls._discover_express(project_dir)
        if express_routes:
            return express_routes[:max_routes]

        # 4. Deteksi Python (FastAPI / Flask)
        python_routes = cls._discover_python(project_dir)
        if python_routes:
            return python_routes[:max_routes]

        # 5. Deteksi Go (Gin / Fiber)
        go_routes = cls._discover_go(project_dir)
        if go_routes:
            return go_routes[:max_routes]

        # Fallback default
        return [
            {"path": "/", "name": "Root Home", "method": "GET"}
        ]

    @classmethod
    def _discover_nextjs(cls, project_dir):
        routes = []
        app_dirs = [
            os.path.join(project_dir, 'src', 'app'),
            os.path.join(project_dir, 'app')
        ]
        pages_dirs = [
            os.path.join(project_dir, 'src', 'pages'),
            os.path.join(project_dir, 'pages')
        ]

        # 1. App Router: Hanya file page.tsx / page.jsx / page.js
        for base in app_dirs:
            if not os.path.exists(base):
                continue
            for root, dirs, files in os.walk(base):
                # Abaikan direktori components internal
                if 'components' in root.lower():
                    continue
                for f in files:
                    if f in ['page.tsx', 'page.jsx', 'page.js']:
                        rel = os.path.relpath(root, base).replace('\\', '/')
                        route = '/' if rel == '.' else '/' + rel.strip('/')
                        # Filter parameter dinamis bertanda [param] agar tidak 404
                        if '[' not in route and route not in [r['path'] for r in routes]:
                            name = route.strip('/').replace('/', ' ').replace('-', ' ').title() or 'Halaman Utama'
                            routes.append({"path": route, "name": f"Halaman {name}", "method": "GET"})

        # 2. Pages Router: File dalam direktori pages/
        for base in pages_dirs:
            if not os.path.exists(base):
                continue
            for root, dirs, files in os.walk(base):
                for f in files:
                    if f.endswith(('.tsx', '.jsx', '.js', '.ts')) and not f.startswith('_') and not f.startswith('['):
                        fn_clean = os.path.splitext(f)[0]
                        rel = os.path.relpath(root, base).replace('\\', '/')
                        if fn_clean == 'index':
                            route = '/' if rel == '.' else '/' + rel.strip('/')
                        else:
                            route = ('/' if rel == '.' else '/' + rel.strip('/') + '/') + fn_clean
                        if route not in [r['path'] for r in routes]:
                            name = route.strip('/').replace('/', ' ').replace('-', ' ').title() or 'Halaman Utama'
                            routes.append({"path": route, "name": f"Halaman {name}", "method": "GET"})

        return routes

    @classmethod
    def _discover_laravel(cls, project_dir):
        routes = []
        route_files = [
            (os.path.join(project_dir, 'routes', 'api.php'), '/api'),
            (os.path.join(project_dir, 'routes', 'web.php'), '')
        ]

        pattern = re.compile(r'Route::(get|post)\s*\(\s*[\'\"]([^\'\"]+)[\'\"]', re.IGNORECASE)

        for fpath, prefix in route_files:
            if not os.path.exists(fpath):
                continue
            try:
                with open(fpath, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                matches = pattern.findall(content)
                for method, p in matches:
                    clean_p = p.strip('/')
                    # Filter parameter dinamis seperti {id} atau {slug}
                    if '{' not in clean_p:
                        full_route = f"{prefix}/{clean_p}" if clean_p else (prefix or '/')
                        if full_route not in [r['path'] for r in routes]:
                            name = clean_p.replace('-', ' ').title() or 'Root API'
                            routes.append({
                                "path": full_route,
                                "name": f"API {name}",
                                "method": method.upper()
                            })
            except Exception:
                pass
        return routes

    @classmethod
    def _discover_express(cls, project_dir):
        routes = []
        pattern = re.compile(r'(?:app|router)\.(get|post)\s*\(\s*[\'\"]([^\'\"]+)[\'\"]', re.IGNORECASE)
        for root, dirs, files in os.walk(project_dir):
            if any(ign in root for ign in ['node_modules', '.next', 'dist', 'build', '.git']):
                continue
            for f in files:
                if f.endswith(('.js', '.ts')):
                    try:
                        with open(os.path.join(root, f), 'r', encoding='utf-8', errors='ignore') as fh:
                            m = pattern.findall(fh.read())
                            for method, p in m:
                                if ':' not in p and p not in [r['path'] for r in routes]:
                                    routes.append({
                                        "path": p,
                                        "name": f"Route {p}",
                                        "method": method.upper()
                                    })
                    except Exception:
                        pass
        return routes

    @classmethod
    def _discover_python(cls, project_dir):
        routes = []
        pattern = re.compile(r'@(?:app|router)\.(get|post)\s*\(\s*[\'\"]([^\'\"]+)[\'\"]', re.IGNORECASE)
        for root, dirs, files in os.walk(project_dir):
            if any(ign in root for ign in ['venv', '__pycache__', '.git']):
                continue
            for f in files:
                if f.endswith('.py'):
                    try:
                        with open(os.path.join(root, f), 'r', encoding='utf-8', errors='ignore') as fh:
                            m = pattern.findall(fh.read())
                            for method, p in m:
                                if '{' not in p and p not in [r['path'] for r in routes]:
                                    routes.append({
                                        "path": p,
                                        "name": f"Endpoint {p}",
                                        "method": method.upper()
                                    })
                    except Exception:
                        pass
        return routes

    @classmethod
    def _discover_go(cls, project_dir):
        routes = []
        pattern = re.compile(r'\.(?:Get|Post|GET|POST)\s*\(\s*[\'\"]([^\'\"]+)[\'\"]')
        for root, dirs, files in os.walk(project_dir):
            if any(ign in root for ign in ['vendor', '.git']):
                continue
            for f in files:
                if f.endswith('.go'):
                    try:
                        with open(os.path.join(root, f), 'r', encoding='utf-8', errors='ignore') as fh:
                            m = pattern.findall(fh.read())
                            for p in m:
                                if ':' not in p and '*' not in p and p not in [r['path'] for r in routes]:
                                    routes.append({
                                        "path": p,
                                        "name": f"Handler {p}",
                                        "method": "GET"
                                    })
                    except Exception:
                        pass
        return routes
