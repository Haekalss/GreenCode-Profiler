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
    @classmethod
    def detect_fullstack_components(cls, project_dir="."):
        """
        Mendeteksi apakah direktori ini adalah monorepo/fullstack project
        yang memiliki folder Frontend dan Backend terpisah dalam satu root directory.
        """
        project_dir = os.path.abspath(project_dir)
        if not os.path.exists(project_dir):
            return {"is_fullstack": False}

        ignore_names = {'node_modules', 'vendor', '.git', '.next', 'dist', 'build', 'tools', 'reports', 'storage', 'venv', '__pycache__'}
        try:
            entries = [e for e in os.listdir(project_dir) if os.path.isdir(os.path.join(project_dir, e))]
        except Exception:
            return {"is_fullstack": False}

        subdirs = [e for e in entries if e.lower() not in ignore_names and not e.startswith('.')]

        backend_info = None
        frontend_info = None

        for sub in subdirs:
            sub_path = os.path.join(project_dir, sub)
            try:
                sub_files = [f.lower() for f in os.listdir(sub_path) if os.path.isfile(os.path.join(sub_path, f))]
            except Exception:
                continue

            # 1. Check PHP / Laravel backend
            if 'composer.json' in sub_files or 'artisan' in sub_files:
                src_app = os.path.join(sub, 'app').replace('\\', '/') if os.path.exists(os.path.join(sub_path, 'app')) else sub.replace('\\', '/')
                backend_info = {
                    'language': 'php',
                    'framework': 'laravel',
                    'dir': sub,
                    'source_dir': src_app,
                    'default_port': 8000,
                    'base_url': 'http://127.0.0.1:8000'
                }
            # 2. Check Python backend (FastAPI, Django, Flask)
            elif 'requirements.txt' in sub_files or 'pyproject.toml' in sub_files or 'manage.py' in sub_files:
                backend_info = {
                    'language': 'python',
                    'framework': 'fastapi' if 'requirements.txt' in sub_files else 'django',
                    'dir': sub,
                    'source_dir': sub.replace('\\', '/'),
                    'default_port': 8000,
                    'base_url': 'http://127.0.0.1:8000'
                }
            # 3. Check Go backend
            elif 'go.mod' in sub_files or 'main.go' in sub_files:
                backend_info = {
                    'language': 'go',
                    'framework': 'gin',
                    'dir': sub,
                    'source_dir': sub.replace('\\', '/'),
                    'default_port': 8080,
                    'base_url': 'http://127.0.0.1:8080'
                }
            # 4. Check Node.js / Frontend / Next.js
            elif 'package.json' in sub_files:
                has_next = os.path.exists(os.path.join(sub_path, 'next.config.js')) or \
                           os.path.exists(os.path.join(sub_path, 'next.config.mjs')) or \
                           os.path.exists(os.path.join(sub_path, 'next.config.ts')) or \
                           os.path.exists(os.path.join(sub_path, 'src', 'app')) or \
                           os.path.exists(os.path.join(sub_path, 'app')) or \
                           any(tag in sub.lower() for tag in ['fe', 'front', 'client', 'ui', 'web'])
                if has_next:
                    src_fe = os.path.join(sub, 'src').replace('\\', '/') if os.path.exists(os.path.join(sub_path, 'src')) else sub.replace('\\', '/')
                    frontend_info = {
                        'language': 'node',
                        'framework': 'nextjs',
                        'dir': sub,
                        'source_dir': src_fe,
                        'default_port': 3000,
                        'base_url': 'http://127.0.0.1:3000'
                    }
                else:
                    if not backend_info:
                        backend_info = {
                            'language': 'node',
                            'framework': 'express',
                            'dir': sub,
                            'source_dir': sub.replace('\\', '/'),
                            'default_port': 5000,
                            'base_url': 'http://127.0.0.1:5000'
                        }
                    else:
                        frontend_info = {
                            'language': 'node',
                            'framework': 'react',
                            'dir': sub,
                            'source_dir': sub.replace('\\', '/'),
                            'default_port': 3000,
                            'base_url': 'http://127.0.0.1:3000'
                        }

        if backend_info and frontend_info:
            return {
                "is_fullstack": True,
                "backend": backend_info,
                "frontend": frontend_info
            }

        return {"is_fullstack": False}

    @classmethod
    def detect_project_ecosystem(cls, project_dir="."):
        """
        Mendeteksi jenis bahasa pemrograman project berdasarkan file konfigurasi standar.
        Mendukung deteksi Full-Stack Monorepo (FE + BE) secara otomatis!
        """
        # 1. Cek apakah Full-Stack (memiliki subfolder FE dan BE sekaligus)
        fs = cls.detect_fullstack_components(project_dir)
        if fs.get('is_fullstack'):
            return 'fullstack'

        # 2. Pengecekan di root project_dir langsung
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

        # 3. Pengecekan di 1 level subdirektori jika root tidak memuat manifes langsung
        try:
            entries = [e for e in os.listdir(project_dir) if os.path.isdir(os.path.join(project_dir, e))]
        except Exception:
            entries = []

        for sub in entries:
            if sub.lower() in ['node_modules', 'vendor', '.git', 'tools']:
                continue
            sub_files = [f.lower() for f in os.listdir(os.path.join(project_dir, sub)) if os.path.isfile(os.path.join(project_dir, sub, f))]
            if 'composer.json' in sub_files or 'artisan' in sub_files:
                return 'php'
            if 'package.json' in sub_files:
                return 'node'
            if 'requirements.txt' in sub_files or 'pyproject.toml' in sub_files:
                return 'python'
            if 'go.mod' in sub_files:
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

        # Mode Khusus Full-Stack: Memantau proses Backend dan Frontend secara paralel
        if language == 'fullstack':
            fs = cls.detect_fullstack_components(project_dir)
            be_port = str(fs.get('backend', {}).get('default_port', 8000))
            fe_port = str(fs.get('frontend', {}).get('default_port', 3000))
            be_lang = fs.get('backend', {}).get('language', 'php')
            fe_lang = fs.get('frontend', {}).get('language', 'node')

            target_ports = [be_port, fe_port]
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    cmd = ' '.join(proc.info['cmdline'] or []).lower()
                    for tp in target_ports:
                        if tp in cmd or f":{tp}" in cmd:
                            if proc.pid not in [p['pid'] for p in pids]:
                                pids.append({
                                    'pid': proc.pid,
                                    'name': proc.info['name'],
                                    'cmdline': cmd,
                                    'matched_by': f'port {tp} (Fullstack)'
                                })
                except Exception:
                    pass

            # Tambahkan signature proses jika belum tercover
            all_sigs = PROCESS_SIGNATURES.get(be_lang, []) + PROCESS_SIGNATURES.get(fe_lang, [])
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    name = (proc.info['name'] or '').lower()
                    if any(sig in name for sig in all_sigs):
                        cmd = ' '.join(proc.info['cmdline'] or []).lower()
                        if proc.pid not in [p['pid'] for p in pids]:
                            pids.append({
                                'pid': proc.pid,
                                'name': proc.info['name'],
                                'cmdline': cmd,
                                'matched_by': 'signature fullstack'
                            })
                except Exception:
                    pass

            return {
                'detected_language': 'fullstack',
                'port': f"{be_port},{fe_port}",
                'processes': pids,
                'pids': [p['pid'] for p in pids]
            }
        
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
    Mendukung: Full-Stack Monorepo (FE + BE), Next.js, Laravel, Express.js, FastAPI, dan Go.
    """
    @classmethod
    def discover(cls, project_dir=".", max_routes=16):
        project_dir = os.path.abspath(project_dir)
        
        # 0. Cek jika Full-Stack (Monorepo dengan Frontend dan Backend terpisah)
        fs = RuntimeDetector.detect_fullstack_components(project_dir)
        if fs.get('is_fullstack'):
            routes = []
            fe = fs['frontend']
            be = fs['backend']
            
            fe_dir = os.path.join(project_dir, fe['dir'])
            be_dir = os.path.join(project_dir, be['dir'])
            
            # A. Ambil rute Frontend (Next.js / Express / React)
            fe_routes = cls._discover_nextjs(fe_dir) or cls._discover_express(fe_dir)
            limit_fe = max_routes // 2
            for r in fe_routes[:limit_fe]:
                r_copy = dict(r)
                r_copy['service'] = 'frontend'
                r_copy['base_url'] = fe['base_url']
                r_copy['url'] = f"{fe['base_url']}{r['path']}"
                r_copy['name'] = f"[FE] {r['name']}"
                routes.append(r_copy)

            # B. Ambil rute Backend (Laravel / Python / Go)
            be_routes = []
            if be['language'] == 'php':
                be_routes = cls._discover_laravel(be_dir)
            elif be['language'] == 'python':
                be_routes = cls._discover_python(be_dir)
            elif be['language'] == 'go':
                be_routes = cls._discover_go(be_dir)
            elif be['language'] == 'node':
                be_routes = cls._discover_express(be_dir)

            limit_be = max_routes - len(routes)
            for r in be_routes[:limit_be]:
                r_copy = dict(r)
                r_copy['service'] = 'backend'
                r_copy['base_url'] = be['base_url']
                r_copy['url'] = f"{be['base_url']}{r['path']}"
                r_copy['name'] = f"[BE] {r['name']}"
                routes.append(r_copy)

            if routes:
                return routes

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

        # Periksa juga subdirektori 1 tingkat
        try:
            for sub in os.listdir(project_dir):
                sub_path = os.path.join(project_dir, sub)
                if os.path.isdir(sub_path) and sub.lower() not in ['node_modules', '.git', 'vendor', '.next', 'dist']:
                    app_dirs.append(os.path.join(sub_path, 'src', 'app'))
                    app_dirs.append(os.path.join(sub_path, 'app'))
                    pages_dirs.append(os.path.join(sub_path, 'src', 'pages'))
                    pages_dirs.append(os.path.join(sub_path, 'pages'))
        except Exception:
            pass

        # 1. App Router: Hanya file page.tsx / page.jsx / page.js
        for base in app_dirs:
            if not os.path.exists(base):
                continue
            for root, dirs, files in os.walk(base):
                if 'components' in root.lower():
                    continue
                for f in files:
                    if f in ['page.tsx', 'page.jsx', 'page.js']:
                        rel = os.path.relpath(root, base).replace('\\', '/')
                        route = '/' if rel == '.' else '/' + rel.strip('/')
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

        # Periksa juga subdirektori 1 tingkat
        try:
            for sub in os.listdir(project_dir):
                sub_path = os.path.join(project_dir, sub)
                if os.path.isdir(sub_path) and sub.lower() not in ['node_modules', '.git', 'vendor', 'tools']:
                    route_files.append((os.path.join(sub_path, 'routes', 'api.php'), '/api'))
                    route_files.append((os.path.join(sub_path, 'routes', 'web.php'), ''))
        except Exception:
            pass

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
