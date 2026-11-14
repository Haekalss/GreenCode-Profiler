"""
GreenCode Built-in Code Quality & Architecture Analyzer
Menganalisis struktur kode statis tanpa dependensi eksternal berat.
Mendukung: PHP (.php), TypeScript/JavaScript (.ts, .tsx, .js, .jsx), Python (.py), dan Go (.go).
Menghitung: NCLOC, Cyclomatic Complexity (CCN), Maintainability Index (MI), dan Code Smells arsitektur.
"""
import os
import re
import math
from collections import defaultdict

SUPPORTED_EXTENSIONS = {
    '.php': 'php',
    '.ts': 'typescript',
    '.tsx': 'typescript-react',
    '.js': 'javascript',
    '.jsx': 'javascript-react',
    '.py': 'python',
    '.go': 'go'
}

# Pola percabangan logika untuk menghitung Cyclomatic Complexity
BRANCH_PATTERNS = [
    r'\bif\b',
    r'\belse\s*if\b',
    r'\belif\b',
    r'\bfor\b',
    r'\bforeach\b',
    r'\bwhile\b',
    r'\bcase\b',
    r'\bcatch\b',
    r'\bexcept\b',
    r'&&',
    r'\|\|',
    r'\band\b',
    r'\bor\b',
    r'\?\s*[^:]+:',  # Ternary operator
    r'\?\?'          # Null coalescing
]
BRANCH_REGEX = re.compile('|'.join(BRANCH_PATTERNS), re.IGNORECASE)

# Pola deteksi deklarasi fungsi/method
FUNC_PATTERNS = {
    'php': re.compile(r'^\s*(?:public|protected|private|static|\s)*\s*function\s+([a-zA-Z0-9_]+)\s*\((.*?)\)', re.MULTILINE),
    'typescript': re.compile(r'^\s*(?:export\s+)?(?:async\s+)?function\s+([a-zA-Z0-9_]+)\s*\((.*?)\)|^\s*(?:public|protected|private|static|\s)*\s*([a-zA-Z0-9_]+)\s*\((.*?)\)\s*[:{]|const\s+([a-zA-Z0-9_]+)\s*=\s*(?:async\s*)?\((.*?)\)\s*=>', re.MULTILINE),
    'python': re.compile(r'^\s*(?:async\s+)?def\s+([a-zA-Z0-9_]+)\s*\((.*?)\):', re.MULTILINE),
    'go': re.compile(r'^\s*func\s+(?:\(.*?\)\s*)?([a-zA-Z0-9_]+)\s*\((.*?)\)', re.MULTILINE)
}

class CodeQualityAnalyzer:
    def __init__(self, target_dir=".", ignore_dirs=None):
        if isinstance(target_dir, (list, tuple)):
            self.target_dirs = [os.path.abspath(d) for d in target_dir if os.path.exists(d)]
            self.target_dir = self.target_dirs[0] if self.target_dirs else os.path.abspath(".")
        elif isinstance(target_dir, str) and ',' in target_dir:
            self.target_dirs = [os.path.abspath(d.strip()) for d in target_dir.split(',') if os.path.exists(d.strip())]
            self.target_dir = self.target_dirs[0] if self.target_dirs else os.path.abspath(".")
        else:
            self.target_dirs = [os.path.abspath(target_dir)]
            self.target_dir = self.target_dirs[0]

        self.ignore_dirs = set(ignore_dirs or [
            'vendor', 'node_modules', '.git', 'dist', 'build', '.next',
            'storage', 'reports', 'venv', '__pycache__', '.idea', '.vscode',
            'tools', 'mariadb', 'backup', 'data', 'temp', 'tmp'
        ])
        self.file_results = []
        self.function_results = []
        self.code_smells = []

    def is_ignored(self, root):
        parts = os.path.relpath(root, self.target_dir).split(os.sep)
        return any(p in self.ignore_dirs for p in parts)

    def analyze_file(self, file_path):
        ext = os.path.splitext(file_path)[1].lower()
        if ext not in SUPPORTED_EXTENSIONS:
            return None

        lang = SUPPORTED_EXTENSIONS[ext]
        rel_path = os.path.relpath(file_path, self.target_dir).replace('\\', '/')

        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                lines = f.readlines()
        except Exception as e:
            return None

        total_lines = len(lines)
        if total_lines == 0:
            return None

        ncloc = 0
        comment_lines = 0
        blank_lines = 0
        in_block_comment = False
        max_indent = 0

        code_lines = []

        for line in lines:
            stripped = line.strip()
            # Indentasi tingkat piramida (Deep Nesting)
            leading_spaces = len(line) - len(line.lstrip(' '))
            indent_level = leading_spaces // 4
            if stripped and indent_level > max_indent:
                max_indent = indent_level

            # Blank lines
            if not stripped:
                blank_lines += 1
                continue

            # Multi-line comment tracking
            if in_block_comment:
                comment_lines += 1
                if '*/' in stripped or (lang == 'python' and '"""' in stripped):
                    in_block_comment = False
                continue

            if stripped.startswith('/*') or (lang == 'python' and stripped.startswith('"""')):
                comment_lines += 1
                if not (stripped.endswith('*/') or (lang == 'python' and stripped.endswith('"""') and len(stripped) > 3)):
                    in_block_comment = True
                continue

            # Single-line comment
            if stripped.startswith('//') or stripped.startswith('#') or stripped.startswith('*'):
                comment_lines += 1
                continue

            ncloc += 1
            code_lines.append(line)

        # 1. Cyclomatic Complexity File Total
        file_content = ''.join(code_lines)
        branch_matches = BRANCH_REGEX.findall(file_content)
        file_ccn = 1 + len(branch_matches)

        # 2. Maintainability Index (MI)
        # Standar SEI: 171 - 5.2*ln(Halstead V) - 0.23*CCN - 16.2*ln(LOC)
        # Pendekatan praktis berbasis NCLOC & CCN:
        effective_loc = max(ncloc, 1)
        est_halstead_vol = max(effective_loc * 8.5, 1.0)
        raw_mi = 171.0 - 5.2 * math.log(est_halstead_vol) - 0.23 * file_ccn - 16.2 * math.log(effective_loc)
        mi_score = max(0.0, min(100.0, (raw_mi * 100.0) / 171.0))

        # 3. Analisis Fungsi / Method
        func_list = self._extract_functions(file_content, lines, lang, rel_path)

        # 4. Deteksi Code Smells
        smells = []
        if ncloc > 300:
            smells.append({
                'type': 'GOD_CLASS_OR_BLOAT',
                'severity': 'HIGH',
                'file': rel_path,
                'message': f"Berkas terlalu besar ({ncloc} baris NCLOC). Disarankan dipecah menjadi service/komponen modular."
            })

        if max_indent >= 5:
            smells.append({
                'type': 'DEEP_NESTING',
                'severity': 'MEDIUM',
                'file': rel_path,
                'message': f"Anti-pattern Piramida terdeteksi (Kedalaman indentasi level {max_indent}). Sulit diuji dan boros komputasi."
            })

        if 'controller' in rel_path.lower():
            # Cek jika Controller memuat query DB mentah atau logika berat
            db_keywords = ['db::', 'select(', 'insert(', 'update(', 'delete(', 'join(', 'where(']
            found_db = [kw for kw in db_keywords if kw in file_content.lower()]
            if len(found_db) >= 3:
                smells.append({
                    'type': 'FAT_CONTROLLER',
                    'severity': 'HIGH',
                    'file': rel_path,
                    'message': f"Fat Controller: Controller memuat logika query database ({', '.join(found_db[:3])}). Pindahkan ke Repository/Service."
                })

        for fn in func_list:
            if fn['ccn'] > 15:
                smells.append({
                    'type': 'COMPLEX_METHOD',
                    'severity': 'HIGH' if fn['ccn'] > 25 else 'MEDIUM',
                    'file': rel_path,
                    'message': f"Fungsi '{fn['name']}' memiliki Cyclomatic Complexity sangat tinggi (CCN = {fn['ccn']}). Wajib direfactor!"
                })
            if fn['lines'] > 60:
                smells.append({
                    'type': 'LONG_METHOD',
                    'severity': 'MEDIUM',
                    'file': rel_path,
                    'message': f"Fungsi '{fn['name']}' terlalu panjang ({fn['lines']} baris). Pecah menjadi sub-rutin kecil."
                })

        self.code_smells.extend(smells)

        record = {
            'file': rel_path,
            'language': lang,
            'total_lines': total_lines,
            'ncloc': ncloc,
            'blank_lines': blank_lines,
            'comment_lines': comment_lines,
            'ccn': file_ccn,
            'maintainability_index': round(mi_score, 1),
            'max_indent': max_indent,
            'functions_count': len(func_list),
            'smells_count': len(smells)
        }
        self.file_results.append(record)
        return record

    def _extract_functions(self, content, lines, lang, file_path):
        res = []
        pattern = FUNC_PATTERNS.get(lang.split('-')[0])
        if not pattern:
            return res

        matches = list(pattern.finditer(content))
        for i, match in enumerate(matches):
            name = next((g for g in match.groups() if g and not g.strip().startswith('(')), 'anonymous')
            start_pos = match.start()
            
            # Cari batas akhir fungsi (sampai fungsi berikutnya atau akhir konten)
            if i + 1 < len(matches):
                end_pos = matches[i + 1].start()
            else:
                end_pos = len(content)

            func_block = content[start_pos:end_pos]
            line_no = content[:start_pos].count('\n') + 1
            func_lines = func_block.count('\n')
            
            # Hitung branch keywords murni dalam lingkup fungsi ini
            func_branches = len(BRANCH_REGEX.findall(func_block))
            func_ccn = 1 + func_branches

            f_rec = {
                'file': file_path,
                'name': name,
                'line': line_no,
                'ccn': func_ccn,
                'lines': func_lines
            }
            res.append(f_rec)
            self.function_results.append(f_rec)
        return res

    def run_analysis(self):
        """Menjalankan scan menyeluruh terhadap direktori target."""
        self.file_results = []
        self.function_results = []
        self.code_smells = []

        seen_files = set()
        for t_dir in self.target_dirs:
            if not os.path.exists(t_dir):
                continue
            for root, dirs, files in os.walk(t_dir):
                if self.is_ignored(root):
                    dirs[:] = []
                    continue

                for file in files:
                    ext = os.path.splitext(file)[1].lower()
                    if ext in SUPPORTED_EXTENSIONS:
                        full_path = os.path.join(root, file)
                        if full_path not in seen_files:
                            seen_files.add(full_path)
                            self.analyze_file(full_path)

        total_files = len(self.file_results)
        total_ncloc = sum(f['ncloc'] for f in self.file_results)
        total_ccn = sum(f['ccn'] for f in self.file_results)
        avg_ccn = round(total_ccn / total_files, 2) if total_files > 0 else 0.0
        avg_mi = round(sum(f['maintainability_index'] for f in self.file_results) / total_files, 1) if total_files > 0 else 100.0

        # Identifikasi Hotspot Fungsi Terumit (Top 10)
        top_complex_funcs = sorted(self.function_results, key=lambda x: x['ccn'], reverse=True)[:10]

        summary = {
            'overview': {
                'scanned_directory': ', '.join(self.target_dirs),
                'total_files': total_files,
                'total_ncloc': total_ncloc,
                'total_ccn': total_ccn,
                'average_ccn_per_file': avg_ccn,
                'average_maintainability_index': avg_mi,
                'total_code_smells': len(self.code_smells),
                'maintainability_grade': 'A' if avg_mi >= 85 else ('B' if avg_mi >= 70 else ('C' if avg_mi >= 55 else 'D'))
            },
            'top_complex_functions': top_complex_funcs,
            'code_smells': self.code_smells,
            'files': sorted(self.file_results, key=lambda x: x['ccn'], reverse=True)
        }
        return summary
