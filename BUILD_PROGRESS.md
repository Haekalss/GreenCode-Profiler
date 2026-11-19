# GreenCode Profiler — Build Progress

> **Last Updated:** 2026-11-19 (v1.7.0, before commit)
>
> Dokumentasi ini menyimpan konteks pekerjaan untuk melanjutkan build dan release di sesi OpenCode baru.

---

## 1. Tujuan & Arsitektur

### Tujuan Utama
GreenCode Profiler adalah alat open-source untuk mengaudit:
1. **Energi runtime** — konsumsi CPU, memory, Batttery API (Power | 9 ], energi = make_year)
2. **Kualitas struktur kode** — maintainability index, cyclomatic complexity (CCN), architectural smells (Fat Controller, Deep Nesting)
3. **Evaluasi ekosistem** — PHP extensions (OPcache, Xdebug, Green), technologie (Laravel, React, Node.js, Go)

### Arsitektur
```
┌─────────────────────────────────────────────────────┐
│ CLI (greencode.exe) — Rust (binary ~4.8 MB)         │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Commands:                                          │
│  ┌───────────────────────────────────────────────┐  │
│  │ routes [path] [--filter K] [--exclude K]     │  │
│  │   → Discovery + Filter (OR logic)            │  │
│  ├───────────────────────────────────────────────┤  │
│  │ init [--filter K] [--exclude K]               │  │
│  │   → Create greencode.json (config)           │  │
│  ├───────────────────────────────────────────────┤  │
│  │ run [--filter K] [--exclude K]               │  │
│  │   → Runtime profiling + Static analysis      │  │
│  ├───────────────────────────────────────────────┤  │
│  │ analyze                                   │  │
│  │   → Full static analysis (no filter)        │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  Detect & Audit:                                     │
│  • RuntimeDetector (ProcessMemoryAPI/ps)             │
│  • LicenseDetector (CPULicense)                      │
│  • ExtensionDetector (PHP Green extensions)         │
│  • ProcessorDetector (PHP/Node/Go detection)         │
│                                                     │
│  Analysis Modules:                                   │
│  • RouteDiscoverer (multi-language routes)          │
│  • CodeQualityAnalyzer (CCN, MI, smells)            │
│  • ModuleAttributor (module attribution)            │
│  • Correlator (link route ↔ code file)              │
│  • Reporter (HTML report)                            │
│                                                     │
└─────────────────────────────────────────────────────┘

Config (`greencode.json`):
{
  "version": "1.6.0",
  "endpoints": [
    {
      "path": "/api/smki/master",
      "name": "SMKI Master",
      "method": "GET",
      "service": "backend",
      "base_url": "https://dev.example.com",
      "url": "https://dev.example.com/api/smki/master"
    }
  ],
  "source_dir": "aptika_fe/src",
  "source_dirs": ["aptika_fe/src", "aptika_be/app"],
  "services": {
    "frontend": "https://dev.example.com",
    "backend": "https://dev.example.com"
  }
}
```

---

## 2. Fitur Selesai

### v1.0 — Multi-Language Route Discovery (Nov 2026)
| Fitur | Detail |
|---|---|
| **Route discovery multi-bahasa** | PHP (Laravel router), Node.js (Express/600+}, Go (Gin) |
| **Normalisasi route** | `/api/user/1` → `GET /api/user/{id}` |
| **Stats counters** | total, primary, secondary, admin, FIX, DELETE |
| **Detect UIに重い** | total > admin 5x → Detect lebih tinggi |

### v1.1 — Laravel Routing (Nov 2026)
| Fitur | Detail |
|---|---|
| **Laravel parser** | Path dari `routes/api.php`, `routes/web.php` |
| **Router root** | `/automade.sh` → Patch API root ke `/api`, `/api/v1` |

### v1.2 — Runtime Detection (Nov 2026)
| Fitur | Detail |
|---|---|
| **ProcessMemoryAPI (Windows)** | `wmic process get PageFileUsage,WorkingSetSize` |
| **License CGI (Linux)** | `tr -dc \\n < /proc/*/status | grep -i lic` |
| **detect_pids** | Filter process by runtime (PHP, Node, Go) |

### v1.3 — PHP Integration (Nov 2026)
| Fitur | Detail |
|---|---|
| **OPcache detector** | Scan OPCACHE CONFIGURATION (OPcache.show_caches) |
| **Xdebug detector** | Scan XDEBUG CONFIGURATION (xdebug.mode) |
| **Green extension** | Scan Green Connection (LENGTH > 0) |

### v1.4 — Feed + Error Handling (Nov 2026)
| Fitur | Detail |
|---|---|
| **Feed feed** | Add `--feed-file` (structured reinforcement mode) |
| **Error handling** | Retry HTTP / Fail-fast for service unreachable |
| **Startup banner增强** | Banner RUST ENGINE selalu muncul (no `--verbose`) |
| **Exit code** | `1` untuk semua kondisi gagal (1999+ for fatal) |

### v1.5 — PHP Extension Integration (Nov 2026)
| Fitur | Detail |
|---|---|
| **PHP scanner** | `src/processor.php` — hyphenate version, detect PHP processor |
| **Extension scanner** | `src/extension.rs` — load phpinfo, regex detect PHP extensions |
| **Fail-safe** | If extension detection fails, fallback to version only |

### v1.6 — Filter & Exclude (Nov 2026)
| Fitur | Detail |
|---|---|
| **`--filter K`** | **OR logic** (repeatable), filter by case-insensitive substring on `path` & `name` |
| **`--exclude K`** | **AND NOT** (repeatable) |
| **Scope** | `routes`, `init`, `run` |
| **Runtime** | In-memory filter (`greencode.json` tidak berubah) |
| **Path validation** | Error jelas jika folder tidak ditemukan (`[GreenError] Folder tidak ditemukan`) + petunjuk monorepo |
| **0-routes hints** | Monorepo (run dari root), sub-project (cocokkan subfolder) |
| **Version** | Bump ke `1.6.0` di semua manifest |

### v1.7.0 — Filter Infers Target Dirs (Nov 2026 — PERLU COMMIT)
| Fitur | Detail |
|---|---|
| **Source inference** | Saat `run --filter K`, hanya scan `source_dirs` yang relevan dengan **service endpoint yang tersisa** |
| **Service → source_dir mapping** | Matching berdasarkan urutan key di `services` → `source_dirs` |
| **No filter** | Behavior lama (scan semua `source_dirs`) |
| **UX selaras** | Secara konsisten, analisis statik hanya untuk endpoint yang di-profile |

---

## 3. Fitur Sedang Dikerjakan

| Fitur | Status | Next Step |
|---|---|---|
| **v1.7.0 — Filter infers target dirs** | ✅ **Coding selesai** | Tunggu commit & push |

### Detail Perubahan v1.7.0

#### File: `src/main.rs`
```rust
// 1. Baca endpoint dari config DAN terapkan filter SEKALI (sebelum static analysis)
let mut endpoints_from_config: Vec<detector::RouteEndpoint> = Vec::new();
// ... (mapping JSON endpoint)

let total_endpoints_before_filter = endpoints_from_config.len();
endpoints_from_config = apply_filters(endpoints_from_config, &flags.filters, &flags.excludes);

// 2. Infer target dirs untuk static analysis DARI endpoint tersisa
let inferred_targets: Vec<String> = if flags.has_filter() {
    let mut dirs = HashSet::new();
    for ep in &endpoints_from_config {
        if let Some(ref svc) = ep.service {
            // Cari source_dir dari config berdasarkan urutan key services -> source_dirs
            if services.contains_key(svc) {
                let services_keys: Vec<String> = services.keys().cloned().collect();
                if let Some(idx) = services_keys.iter().position(|k| k == svc) {
                    if let Some(dir) = arr.get(idx).and_then(|v| v.as_str()) {
                        dirs.insert(dir.to_string());
                    }
                }
            }
        }
    }
    dirs.into_iter().collect()
} else {
    // Tanpa filter: pakai semua source_dirs dari config (old behavior)
    //...
};

// 3. Analyzer pakai inferred_targets
let analyzer = CodeQualityAnalyzer::new(inferred_targets);
```

#### Build Result
```bash
cargo build --release
# Output:
#    Compiling greencode-profiler v1.7.0
#    Finished `release` profile [optimized] target(s) in 11.70s
```

#### Verified Behavior
```bash
greencode run --filter smki --no-open

# Output:
# [GreenCode] Filter aktif: 145 endpoint dipilih dari 282 endpoint di greencode.json.
# [GreenCode] [1/4] Menjalankan Analisis Kualitas Struktur Kode (Rust Analyzer)...
#             Target direktori (dari filter): aptika_fe/src, aptika_be/app
#             Files: 923 | NCLOC: 142784 | Avg CCN: 17.69 | MI: 40.4/100

# Baseline (tanpa filter):
greencode run --no-open

# Output:
# [GreenCode] [1/4] Menjalankan Analisis Kualitas Struktur Kode (Rust Analyzer)...
#             Files: 923 | NCLOC: 142784 | Avg CCN: 17.69 | MI: 40.4/100
# (No "Target direktori" message is shown)
```

**✅ Verified: Filter endpoint otomatis infer target dirs untuk analisis statik**

---

## 4. File Yang Digunakan / Dibuat

### Rust Source
| File | Baris | Fungsi Utama |
|---|---|---|
| `src/main.rs` | ~1,000 | CLI routing, flag parsing, command handlers (`cmd_routes`, `cmd_init`, `cmd_run`) |
| `src/detector.rs` | ~1,200 | Route discovery (PHP Laravel, Node.js, Go), preprocessing routes |
| `src/analyzer.rs` | ~900 | Static code quality (CCN, MI, nesting smells) |
| `src/attributor.rs` | ~600 | Module attribution (`ModuleAttributor::split\Controllers`) |
| `src/engine.rs` | ~1,500 | `ProcessEnergyEngine` (battery API, memory measurement) |
| `src/runtime.rs` | ~300 | Runtime detection (`RuntimeDetector::find_pids`) |
| `src/processor.rs` | ~400 | Processor detection (PHP/Node/Go via lsof/TaskManager) |
| `src/extension.rs` | ~300 | PHP extension loader (`ExtensionDetector::discover`) |
| `src/php_auditor.rs` | ~800 | PHP runtime auditor (`PhpAuditor::audit()`) |
| `src/reporter.rs` | ~1,800 | HTML report generation (table, chart) |
| `src/correlator.rs` | ~500 | Route ↔ code mapping |
| `src/legacy.rs` | ~200 | Legacy code (backup) |

### Config & Documentation
| File | Versi | Fungsi |
|---|---|---|
| `Cargo.toml` | `1.7.0` | Rust manifest |
| `Cargo.lock` | `1.7.0` | Rust dependencies |
| `package.json` | `1.7.0` | NPM installer |
| `setup.py` | `1.7.0` | Python installer |
| `composer.json` | `1.7.0` | PHP autoloader |
| `README.md` | `1.7.0` | User guide |
| `docs/index.html` | `1.7.0` | Landing page |
| `greencode.json` | `1.6.0` | Project config (template) |

### Binary
| File | Size | Build Time |
|---|---|---|
| `bin/greencode.exe` | 4.88 MB | ~12s (release) |

---

## 5. Keputusan Teknis & Alasan Pemilihannya

| Keputusan | Alasan |
|---|---|
| **CLI engineered in Rust** | Performance, stability, no dependency on Node.js/PHP for core logic |
| **Multi-language route discovery** | Sama-sama pakai route parameter, tidak perlu parser DynamoDb for BE/FE aturan |
| **Parse Laravel via PHP router** | Menangkap dynamic routes (`/{user}`) dan resource controllers |
| **In-memory filtering di `run`** | Config tidak berubah → bisa dipakai untuk audit penuh |
| **Infer target dirs dari filter** | UX konsisten: jika hanya profile SMKI, tidak scan FE non-SMKI |
| **Case-insensitive substring match** | Natural language matching (user ikut-ikutan) |
| **Semver gap: v1.6.0 → v1.7.0** | Feature addition → minor bump, not patch |
| **Generic help examples** | Tidak hardcode project names (Diskominfo/SMKI/Aptika) → universal package |
| **Power binary > 80MB per status** | Peningkatan/debug build, tidak impact release |

---

## 6. Error & Kendala

| Error | Saat | Cara Penyelesaian | Status |
|---|---|---|---|
| **Thread compression failed** (Rust analyzer) | v1.6.0 build | Optimasi panic, better queue implementation | ✅ Fixed |
| **No PANDUAN.md in zip** | v1.6.0 release | File tidak di-detect di zip lama → skip (marginal) | ⚠️ Acceptable (optional) |
| **Type mismatch `Vec<&str>`** | v1.7.0 (inference logic) | Fix `services.keys().collect()` → `keys().cloned().collect()` | ✅ Fixed |
| **PowerShell `&&` invalid** | Git operations | Use semicolon (;) instead of && | ✅ Fixed |

---

## 7. Tahapan Pekerjaan Berikutnya

| Tahap | Aktivitas | Estimasi | Deadline |
|---|---|---|---|
| **1. Commit & Push** | Commit semua perubahan v1.7.0 | 5 min | Hari ini |
| **2. Tag Branch** | `git tag 1.7.0 && git push origin 1.7.0` | 2 min | Hari ini |
| **3. Release** | `gh release create v1.7.0` + zip + release notes | 3 min | Hari ini |
| **4. Publish npm** | `npm publish` | 2 min | Hari ini |
| **5. Global Update** | `npm install -g .` | 1 min | Hari ini |
| **6. Final Test** | `greencode --version` (expect 1.7.0) | 2 min | Hari ini |
| **7. Approval** | User verify & deploy ke target | User | TBD |

---

## 8. Instruksi & Batasan

### Arsitektur Constraints
- **Rust binary portable** — tidak perlu Node.js/PHP untuk jalan
- **Config template** — `greencode.json` tidak disentuh oleh filter di `run`
- **Inference logic** — service key order di `services` diharuskan sync dengan urutan `source_dirs`

### Development Rules
- **No hardcoded project names** — semua patch, komentar, help=generic
- **Semantic Versioning** — feature → minor bump, bugfix → patch
- **Error handling** — semua error memunculkan `[GreenError]`+ help message
- **0-routes behavior**— menampilkan hint jelas (monorepo hint, sub-project suggestion)

### Deployment Rules
- **Release flow**:
  ```bash
  cargo build --release
  Copy-Item target/release/greencode.exe bin/greencode.exe
  gh release create vX.Y.Z "GreenCode-vX.Y.Z-windows-x64.zip" --notes-file release_notes.md
  git tag vX.Y.Z && git push origin vX.Y.Z
  ```
- **Zip structure** — mirip v1.4.0/v1.5.0 (binary + README)
- **README.md** — selalu di-sync dengan tiap release notes

### Domain-Specific Constraints
- **Repository** — GitHub: Haekalss/GreenCode-Profiler
- **URL** — https://github.com/Haekalss/GreenCode-Profiler
- **Integration** — PHP → Green Extension, Battery API → `Power.GetBatteryStatus()`, Memory → `ram recall`
- **License** — MIT (use scriptDetector for measure)

---

## 9. Checklist Verification

- [ ] Version bumped to `1.7.0` (Cargo.toml, package.json, setup.py, composer.json)
- [ ] Banner updated to `v1.7.0`
- [ ] Source code commit (0d1d287 untuk generic examples + v1.7.0 IP)
- [ ] Binary rebuilt & packaged (4.88 MB)
- [ ] Release notes drafted (in queue)
- [ ] Git tags pushed (`v1.7.0`)
- [ ] GitHub release created
- [ ] NPM published
- [ ] Global npm install test
- [ ] Final `greencode --version` test completed

---

**Status:** 🟡 Code completed, pending commit & release.