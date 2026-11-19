//! Runtime Detector & Route Discoverer in Pure Rust
//! Mendeteksi proses server aktif dan mengekstrak rute nyata dari berbagai ekosistem:
//! Node (Next.js/Express/NestJS), PHP (Laravel), Python (Flask/FastAPI/Django),
//! Go (net/http/gin/chi), dan Rust (axum/actix/rocket)

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEndpoint {
    pub path: String,
    pub name: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub language: String,
    pub framework: String,
    pub dir: String,
    pub source_dir: String,
    pub default_port: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullstackDetection {
    pub is_fullstack: bool,
    pub backend: Option<ComponentInfo>,
    pub frontend: Option<ComponentInfo>,
}

pub struct RuntimeDetector;

impl RuntimeDetector {
    /// Mendeteksi struktur monorepo/fullstack yang memisahkan Frontend dan Backend
    pub fn detect_fullstack_components<P: AsRef<Path>>(project_dir: P) -> FullstackDetection {
        let p = project_dir.as_ref();
        let ignore = [
            "node_modules", "vendor", ".git", ".next", "dist", "build", "tools",
            "reports", "storage", "venv", "target",
        ];

        let mut backend_info: Option<ComponentInfo> = None;
        let mut frontend_info: Option<ComponentInfo> = None;

        if let Ok(entries) = fs::read_dir(p) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let dirname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if ignore.contains(&dirname.as_str()) {
                    continue;
                }

                // 1. Cek Backend (Laravel / PHP / Python / Go)
                if path.join("artisan").exists() || path.join("composer.json").exists() {
                    let src = if path.join("app").exists() {
                        format!("{}/app", dirname)
                    } else {
                        dirname.clone()
                    };
                    backend_info = Some(ComponentInfo {
                        language: "php".to_string(),
                        framework: "laravel".to_string(),
                        dir: dirname.clone(),
                        source_dir: src,
                        default_port: 8000,
                        base_url: "http://127.0.0.1:8000".to_string(),
                    });
                } else if path.join("go.mod").exists() {
                    backend_info = Some(ComponentInfo {
                        language: "go".to_string(),
                        framework: "go".to_string(),
                        dir: dirname.clone(),
                        source_dir: dirname.clone(),
                        default_port: 8080,
                        base_url: "http://127.0.0.1:8080".to_string(),
                    });
                } else if path.join("Cargo.toml").exists() {
                    let src = if path.join("src").exists() {
                        format!("{}/src", dirname)
                    } else {
                        dirname.clone()
                    };
                    backend_info = Some(ComponentInfo {
                        language: "rust".to_string(),
                        framework: "actix_axum".to_string(),
                        dir: dirname.clone(),
                        source_dir: src,
                        default_port: 8080,
                        base_url: "http://127.0.0.1:8080".to_string(),
                    });
                } else if path.join("requirements.txt").exists()
                    || path.join("pyproject.toml").exists()
                    || path.join("manage.py").exists()
                    || path.join("Pipfile").exists()
                {
                    backend_info = Some(ComponentInfo {
                        language: "python".to_string(),
                        framework: "python".to_string(),
                        dir: dirname.clone(),
                        source_dir: dirname.clone(),
                        default_port: 8000,
                        base_url: "http://127.0.0.1:8000".to_string(),
                    });
                }

                // 2. Cek Frontend (Next.js / React)
                if path.join("package.json").exists() {
                    let has_next = path.join("next.config.js").exists()
                        || path.join("next.config.mjs").exists()
                        || path.join("next.config.ts").exists()
                        || path.join(".next").exists()
                        || path.join("src").join("app").exists()
                        || path.join("app").exists();

                    if has_next {
                        let src = if path.join("src").exists() {
                            format!("{}/src", dirname)
                        } else {
                            dirname.clone()
                        };
                        frontend_info = Some(ComponentInfo {
                            language: "node".to_string(),
                            framework: "nextjs".to_string(),
                            dir: dirname.clone(),
                            source_dir: src,
                            default_port: 3000,
                            base_url: "http://127.0.0.1:3000".to_string(),
                        });
                    }
                }
            }
        }

        if backend_info.is_some() && frontend_info.is_some() {
            FullstackDetection {
                is_fullstack: true,
                backend: backend_info,
                frontend: frontend_info,
            }
        } else {
            FullstackDetection {
                is_fullstack: false,
                backend: backend_info,
                frontend: frontend_info,
            }
        }
    }

    /// Deteksi ekosistem project
    pub fn detect_project_ecosystem<P: AsRef<Path>>(project_dir: P) -> String {
        let fs_detect = Self::detect_fullstack_components(&project_dir);
        if fs_detect.is_fullstack {
            return "fullstack".to_string();
        }

        let p = project_dir.as_ref();
        if p.join("composer.json").exists() || p.join("artisan").exists() {
            return "php".to_string();
        }
        if p.join("package.json").exists() {
            return "node".to_string();
        }
        if p.join("requirements.txt").exists() || p.join("pyproject.toml").exists() {
            return "python".to_string();
        }
        if p.join("Cargo.toml").exists() {
            return "rust".to_string();
        }
        if p.join("go.mod").exists() {
            return "go".to_string();
        }

        "auto".to_string()
    }

    /// Mencari PID proses server aktif menggunakan netstat OS bawaan
    pub fn find_pids(language: &str, target_port: Option<u16>) -> Vec<u32> {
        let mut pids = Vec::new();

        // Cari PID berdasarkan Port target via netstat
        let ports_to_check: Vec<u16> = if let Some(p) = target_port {
            vec![p]
        } else if language == "fullstack" {
            vec![8000, 3000]
        } else if language == "php" {
            vec![8000]
        } else if language == "node" {
            vec![3000]
        } else if language == "rust" || language == "go" {
            vec![8080, 3000]
        } else {
            vec![8000, 3000, 8080, 5000]
        };

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("netstat").args(["-ano"]).output() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    for &port in &ports_to_check {
                        let needle = format!(":{}", port);
                        if line.contains(&needle) && line.contains("LISTENING") {
                            if let Some(last_token) = line.split_whitespace().last() {
                                if let Ok(pid) = last_token.parse::<u32>() {
                                    pids.push(pid);
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            for &port in &ports_to_check {
                if let Ok(output) = Command::new("lsof").args(["-ti", &format!(":{}", port)]).output() {
                    let text = String::from_utf8_lossy(&output.stdout);
                    for line in text.lines() {
                        if let Ok(pid) = line.trim().parse::<u32>() {
                            pids.push(pid);
                        }
                    }
                }
            }
        }

        pids.sort_unstable();
        pids.dedup();
        pids
    }
}

pub struct RouteDiscoverer;

impl RouteDiscoverer {
    /// Menemukan rute nyata FE & BE dari ekosistem apa pun yang terdeteksi.
    /// `max_routes == 0` berarti TANPA BATAS: semua rute dibaca, termasuk yang bersarang
    /// (sub-direktori FE beberapa level, group prefix di routes/*.php, dsb).
    pub fn discover<P: AsRef<Path>>(project_dir: P, max_routes: usize) -> Vec<RouteEndpoint> {
        let p = project_dir.as_ref();
        let fs_info = RuntimeDetector::detect_fullstack_components(p);

        fn take_limit(v: Vec<RouteEndpoint>, limit: usize) -> Vec<RouteEndpoint> {
            if limit == 0 {
                v
            } else {
                v.into_iter().take(limit).collect()
            }
        }
        fn dedup(mut v: Vec<RouteEndpoint>) -> Vec<RouteEndpoint> {
            let mut seen = std::collections::HashSet::new();
            v.retain(|r| seen.insert((r.method.clone(), r.path.clone())));
            v
        }

        let mut collected: Vec<RouteEndpoint> = Vec::new();

        // 1. Komponen Frontend (bisa berada di subdirektori)
        if let Some(fe) = fs_info.frontend.clone() {
            let fe_dir = p.join(&fe.dir);
            let mut fe_routes = if fe.framework == "nextjs" {
                let r = Self::discover_nextjs(&fe_dir);
                if r.is_empty() {
                    Self::discover_node(&fe_dir)
                } else {
                    r
                }
            } else {
                Self::discover_node(&fe_dir)
            };
            fe_routes.sort_by(|a, b| a.path.cmp(&b.path));
            for r in take_limit(fe_routes, usize::MAX) {
                collected.push(RouteEndpoint {
                    path: r.path.clone(),
                    name: format!("[FE] {}", r.name),
                    method: r.method,
                    service: Some("frontend".to_string()),
                    base_url: Some(fe.base_url.clone()),
                    url: Some(format!("{}{}", fe.base_url, r.path)),
                });
            }
        }

        // 2. Komponen Backend (dispatch menurut bahasa terdeteksi)
        if let Some(be) = fs_info.backend.clone() {
            let be_dir = p.join(&be.dir);
            let mut be_routes = match be.language.as_str() {
                "php" => Self::discover_laravel(&be_dir),
                "go" => Self::discover_go(&be_dir),
                "python" => Self::discover_python(&be_dir),
                "rust" => Self::discover_rust(&be_dir),
                "node" => Self::discover_node(&be_dir),
                _ => {
                    // Bahasa tidak dikenal: coba semua discoverer sampai ketemu
                    let mut v = Self::discover_laravel(&be_dir);
                    if v.is_empty() {
                        v = Self::discover_go(&be_dir);
                    }
                    if v.is_empty() {
                        v = Self::discover_python(&be_dir);
                    }
                    if v.is_empty() {
                        v = Self::discover_rust(&be_dir);
                    }
                    v
                }
            };
            be_routes.sort_by(|a, b| (&a.path, &a.method).cmp(&(&b.path, &b.method)));
            for r in be_routes {
                collected.push(RouteEndpoint {
                    path: r.path.clone(),
                    name: format!("[BE] {}", r.name),
                    method: r.method,
                    service: Some("backend".to_string()),
                    base_url: Some(be.base_url.clone()),
                    url: Some(format!("{}{}", be.base_url, r.path)),
                });
            }
        }

        // 3. Belum ada komponen terdeteksi di subdirektori: deteksi ekosistem pada akar proyek
        if collected.is_empty() {
            let eco = RuntimeDetector::detect_project_ecosystem(p);
            let mut v = match eco.as_str() {
                "node" => {
                    let mut v = Self::discover_nextjs(p);
                    if v.is_empty() {
                        v = Self::discover_node(p);
                    }
                    v
                }
                "php" => Self::discover_laravel(p),
                "python" => Self::discover_python(p),
                "rust" => Self::discover_rust(p),
                "go" => Self::discover_go(p),
                _ => {
                    // "auto"/"fullstack": gabungkan hasil semua discoverer
                    let mut all = Vec::new();
                    all.append(&mut Self::discover_nextjs(p));
                    all.append(&mut Self::discover_node(p));
                    all.append(&mut Self::discover_laravel(p));
                    all.append(&mut Self::discover_python(p));
                    all.append(&mut Self::discover_go(p));
                    all.append(&mut Self::discover_rust(p));
                    all
                }
            };
            v.sort_by(|a, b| (&a.path, &a.method).cmp(&(&b.path, &b.method)));
            collected = v;
        }

        // 4. Dedup, lalu batasi jumlahnya bila diminta (0 = tanpa batas)
        collected = dedup(collected);
        if max_routes > 0 {
            let (mut fe_v, mut be_v): (Vec<RouteEndpoint>, Vec<RouteEndpoint>) = collected
                .into_iter()
                .partition(|r| r.service.as_deref() == Some("frontend"));
            let limit_fe = (max_routes + 1) / 2;
            fe_v.truncate(limit_fe);
            be_v.truncate(max_routes.saturating_sub(fe_v.len()));
            fe_v.extend(be_v);
            collected = fe_v;
        }

        collected
    }

    /// Saring rute yang aman ditembak otomatis saat profiling:
    /// hanya GET tanpa placeholder parameter (`{id}`, `[id]`, `<id>`, `:id`, `*`).
    /// Dipakai tahap `init`/`run`, bukan tahap discovery, supaya `routes` tetap menampilkan semua.
    pub fn profilable(routes: Vec<RouteEndpoint>) -> Vec<RouteEndpoint> {
        routes.into_iter().filter(|r| r.method == "GET" && !has_param(&r.path)).collect()
    }

    pub fn discover_nextjs<P: AsRef<Path>>(dir: P) -> Vec<RouteEndpoint> {
        let mut routes = Vec::new();
        let app_dir = dir.as_ref().join("src").join("app");
        let target_dir = if app_dir.exists() {
            app_dir
        } else {
            dir.as_ref().join("app")
        };

        if target_dir.exists() {
            let mut file_paths = Vec::new();
            collect_files_recursive(&target_dir, &mut file_paths);

            for path in file_paths {
                let fname = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if fname.starts_with("page.")
                    && (fname.ends_with(".tsx")
                        || fname.ends_with(".jsx")
                        || fname.ends_with(".js")
                        || fname.ends_with(".ts"))
                {
                    let rel = path.strip_prefix(&target_dir).unwrap_or(&path);
                    let parent = rel
                        .parent()
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .unwrap_or_default();

                    let clean_path = if parent.is_empty() {
                        "/".to_string()
                    } else {
                        let segs: Vec<&str> = parent
                            .split('/')
                            .filter(|s| !s.starts_with('(') && !s.ends_with(')'))
                            .collect();
                        format!("/{}", segs.join("/"))
                    };

                    // Rute berparameter ([id], [rfc]) ikut didaftarkan agar discovery
                    // menampilkan semua rute; penyaringan untuk profiling (agar tidak 404)
                    // dilakukan oleh RouteDiscoverer::profilable().

                    let name = if clean_path == "/" {
                        "Halaman Utama".to_string()
                    } else {
                        clean_path
                            .trim_start_matches('/')
                            .replace('/', " ")
                            .replace('-', " ")
                    };

                    if !routes.iter().any(|r: &RouteEndpoint| r.path == clean_path) {
                        routes.push(RouteEndpoint {
                            path: clean_path,
                            name: format!("Halaman {}", name),
                            method: "GET".to_string(),
                            service: None,
                            base_url: None,
                            url: None,
                        });
                    }
                }
            }
        }
        routes
    }

    /// Membaca rute Laravel dari routes/api.php dan routes/web.php secara generik.
    /// Mendukung: semua metode HTTP, rute di dalam Route::prefix('...')->group(...)
    /// (prefix aktif dilacak lewat kedalaman kurawal, bukan daftar nama tertentu),
    /// serta deklarasi Route::apiResource / Route::resource yang diekspansi ke rute REST.
    pub fn discover_laravel<P: AsRef<Path>>(dir: P) -> Vec<RouteEndpoint> {
        let mut routes = Vec::new();

        let method_re =
            Regex::new(r#"Route::(get|post|put|delete|patch)\s*\(\s*['"]([^'"]*)['"]"#).unwrap();
        let resource_re =
            Regex::new(r#"Route::(?:apiResource|resource)\s*\(\s*['"]([^'"]*)['"]"#).unwrap();
        let prefix_re = Regex::new(r#"prefix\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
        // Rapikan deklarasi multi-baris: Route::apiResource(\n 'nama', ...) -> satu baris
        let multiline_re =
            Regex::new(r"Route::(get|post|put|delete|patch|apiResource|resource)\s*\(\s+").unwrap();

        // (berkas, awalan path): api.php memakai /api, web.php tanpa awalan
        let files = [
            (dir.as_ref().join("routes").join("api.php"), "/api"),
            (dir.as_ref().join("routes").join("web.php"), ""),
        ];

        for (file, base_prefix) in files {
            let raw_content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let content = multiline_re.replace_all(&raw_content, "Route::$1(").to_string();

            // Stack prefix grup: (kedalaman kurawal saat grup dibuka, prefix)
            let mut prefix_stack: Vec<(usize, String)> = Vec::new();
            let mut pending_prefix: Option<String> = None;
            let mut depth = 0usize;

            for line in content.lines() {
                if let Some(c) = prefix_re.captures(line) {
                    pending_prefix = Some(c[1].trim_matches('/').to_string());
                }

                // Prefix aktif baris ini, mis. "/api" atau "/api/smki/master"
                let mut active = String::from(base_prefix);
                for (_, p) in &prefix_stack {
                    active.push('/');
                    active.push_str(p);
                }

                for cap in method_re.captures_iter(line) {
                    push_route(&mut routes, cap[1].to_uppercase(), &active, &cap[2]);
                }

                for cap in resource_re.captures_iter(line) {
                    let raw = cap[1].trim_start_matches('/');
                    // Ekspansi REST standar: index, store, show, update, destroy
                    for (m, suffix) in [
                        ("GET", ""),
                        ("POST", ""),
                        ("GET", "/{id}"),
                        ("PUT", "/{id}"),
                        ("PATCH", "/{id}"),
                        ("DELETE", "/{id}"),
                    ] {
                        push_route(
                            &mut routes,
                            m.to_string(),
                            &active,
                            &format!("{}{}", raw, suffix),
                        );
                    }
                }

                // Lacak pembukaan/penutupan kurawal agar prefix grup selalu benar
                let opens = line.matches('{').count();
                let closes = line.matches('}').count();
                if opens > 0 {
                    depth += opens;
                    if line.contains("group(") || line.contains("function") {
                        if let Some(p) = pending_prefix.take() {
                            prefix_stack.push((depth, p));
                        }
                    }
                }
                depth = depth.saturating_sub(closes);
                while let Some((d, _)) = prefix_stack.last() {
                    if depth < *d {
                        prefix_stack.pop();
                    } else {
                        break;
                    }
                }
            }
        }

        // Dedup per (metode, path)
        let mut seen = std::collections::HashSet::new();
        routes.retain(|r| seen.insert((r.method.clone(), r.path.clone())));
        routes
    }

    // ===================== NODE (Express / Fastify / Koa / NestJS / Next API) =====================
    /// Mengekstrak rute Node.js secara generik: panggilan router
    /// (`app.get('/x')`, `router.post(...)`), dekorator NestJS (`@Get('/x')`
    /// dengan prefix `@Controller('...')`), dan file rute Next.js
    /// (`app/**/route.ts` serta `pages/api/**`).
    pub fn discover_node<P: AsRef<Path>>(dir: P) -> Vec<RouteEndpoint> {
        let mut routes = Vec::new();
        let dir = dir.as_ref();

        let files = collect_source_files(
            dir,
            &["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"],
            &[
                "node_modules", ".next", "dist", "build", "coverage", "vendor", ".git",
                "out", ".nuxt",
            ],
        );

        // receiver umum: app/router/server/... .get('/path', ...) -> Express/Fastify/Koa
        let method_re = Regex::new(
            r#"(?i)\b(?:app|router|server|api|fastify|koa|r|http|web|group|routes|appRouter|apiRouter|mainRouter)\s*\.\s*(get|post|put|patch|delete|head|options|all)\s*\(\s*['"`]([^'"`\?]+)['"`]"#,
        )
        .unwrap();
        // NestJS: @Get('/x'), @Post("/x")
        let nest_re = Regex::new(
            r#"@(Get|Post|Put|Patch|Delete|Head|Options|All)\s*\(\s*['"`]([^'"`]*)['"`]"#,
        )
        .unwrap();
        // NestJS tanpa argumen: @Get() -> path = prefix controller
        let nest_empty_re =
            Regex::new(r#"@(Get|Post|Put|Patch|Delete|Head|Options|All)\s*\(\s*\)"#).unwrap();
        let controller_re = Regex::new(r#"@Controller\s*\(\s*['"`]([^'"`]*)['"`]"#).unwrap();

        for f in &files {
            let content = match fs::read_to_string(f) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut controller = String::new();

            for line in content.lines() {
                if let Some(c) = controller_re.captures(line) {
                    controller = c[1].trim_matches('/').to_string();
                }

                for cap in method_re.captures_iter(line) {
                    // Hanya path diawali '/' agar tidak tertangkap app.get('setting')
                    if cap[2].starts_with('/') {
                        push_route(&mut routes, cap[1].to_uppercase(), "", &cap[2]);
                    }
                }

                for cap in nest_re.captures_iter(line) {
                    push_route(&mut routes, cap[1].to_uppercase(), &controller, &cap[2]);
                }

                if let Some(cap) = nest_empty_re.captures(line) {
                    push_route(&mut routes, cap[1].to_uppercase(), &controller, "");
                }
            }

            // File rute Next.js: src/app/api/x/route.ts -> GET /api/x ; pages/api/x.js -> GET /api/x
            let fname = f
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if fname == "route.ts" || fname == "route.js" || fname == "route.mts" || fname == "route.cts"
            {
                if let Some(parent) = f.parent() {
                    if let Ok(rel) = parent.strip_prefix(dir) {
                        let segs: Vec<String> = rel
                            .components()
                            .map(|c| c.as_os_str().to_string_lossy().to_string())
                            .collect();
                        if let Some(pos) = segs.iter().position(|s| s == "app" || s == "pages") {
                            let rest: Vec<&str> = segs[pos + 1..]
                                .iter()
                                .map(|s| s.as_str())
                                .filter(|s| !s.starts_with('('))
                                .collect();
                            if !rest.is_empty() {
                                push_route(
                                    &mut routes,
                                    "GET".to_string(),
                                    "",
                                    &format!("/{}", rest.join("/")),
                                );
                            }
                        }
                    }
                }
            }
        }
        routes
    }

    // ===================== GO (net/http / gin / chi / gorilla) =====================
    /// Mengekstrak rute Go: `r.GET("/x")`, `mux.HandleFunc("/x")`,
    /// `http.HandleFunc("GET /x")` (pola stdlib Go 1.22+), `r.Get("/x")` (chi).
    pub fn discover_go<P: AsRef<Path>>(dir: P) -> Vec<RouteEndpoint> {
        let mut routes = Vec::new();
        let files = collect_source_files(
            dir.as_ref(),
            &["go"],
            &["vendor", ".git", "testdata", "node_modules"],
        );

        let call_re = Regex::new(
            r#"(?i)\.(get|post|put|patch|delete|head|handlefunc|handle)\s*\(\s*"([^"]*)""#,
        )
        .unwrap();
        let std_methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

        for f in &files {
            let content = match fs::read_to_string(f) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for line in content.lines() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                for cap in call_re.captures_iter(line) {
                    let mut method = cap[1].to_uppercase();
                    let mut raw = cap[2].to_string();

                    // Pola stdlib Go 1.22+: http.HandleFunc("GET /users", ...)
                    if let Some((m, rest)) = raw.split_once(' ') {
                        if std_methods.contains(&m) {
                            method = m.to_string();
                            raw = rest.to_string();
                        }
                    }
                    if method == "HANDLE" || method == "HANDLEFUNC" {
                        method = "GET".to_string();
                    }
                    // Lewati pemanggilan client (http.Get("url...")) yang bukan rute
                    if raw.starts_with('/') {
                        push_route(&mut routes, method, "", &raw);
                    }
                }
            }
        }
        routes
    }

    // ===================== PYTHON (Flask / FastAPI / Django) =====================
    /// Mengekstrak rute Python: `@app.route('/x', methods=[...])` (Flask),
    /// `@router.get("/x")` (FastAPI), dan `path('x/', view)` (Django).
    pub fn discover_python<P: AsRef<Path>>(dir: P) -> Vec<RouteEndpoint> {
        let mut routes = Vec::new();
        let files = collect_source_files(
            dir.as_ref(),
            &["py"],
            &[
                "venv", ".venv", "env", "site-packages", "__pycache__", ".git", "node_modules",
                "migrations", ".tox", "build", "dist", ".eggs",
            ],
        );

        let flask_re = Regex::new(r#"@(\w+)\.route\s*\(\s*['"]([^'"]*)['"]"#).unwrap();
        let fastapi_re = Regex::new(
            r#"@(?:\w+)\.(get|post|put|patch|delete|head|api_route)\s*\(\s*['"]([^'"]*)['"]"#,
        )
        .unwrap();
        let django_re = Regex::new(r#"^\s*path\s*\(\s*['"]([^'"]*)['"]"#).unwrap();
        let methods_re = Regex::new(r#"methods\s*=\s*\[([^\]]*)\]"#).unwrap();
        let word_re = Regex::new(r#"['"]([A-Za-z]+)['"]"#).unwrap();

        for f in &files {
            let content = match fs::read_to_string(f) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for line in content.lines() {
                if line.trim_start().starts_with('#') {
                    continue;
                }

                if let Some(cap) = flask_re.captures(line) {
                    // Metode dari methods=[...], default GET
                    let mut methods: Vec<String> = Vec::new();
                    if let Some(mc) = methods_re.captures(line) {
                        for wc in word_re.captures_iter(&mc[1]) {
                            methods.push(wc[1].to_uppercase());
                        }
                    }
                    if methods.is_empty() {
                        methods.push("GET".to_string());
                    }
                    for m in methods {
                        push_route(&mut routes, m, "", &cap[2]);
                    }
                    continue;
                }

                if let Some(cap) = fastapi_re.captures(line) {
                    let base = cap[1].to_uppercase();
                    let mut methods: Vec<String> = Vec::new();
                    if base == "API_ROUTE" {
                        if let Some(mc) = methods_re.captures(line) {
                            for wc in word_re.captures_iter(&mc[1]) {
                                methods.push(wc[1].to_uppercase());
                            }
                        }
                        if methods.is_empty() {
                            methods.push("GET".to_string());
                        }
                    } else {
                        methods.push(base);
                    }
                    for m in methods {
                        push_route(&mut routes, m, "", &cap[2]);
                    }
                    continue;
                }

                if let Some(cap) = django_re.captures(line) {
                    push_route(&mut routes, "GET".to_string(), "", &cap[1]);
                }
            }
        }
        routes
    }

    // ===================== RUST (axum / actix-web / rocket) =====================
    /// Mengekstrak rute Rust: `.route("/x", get(h))` (axum/actix),
    /// `web::resource("/x")` (actix), `#[get("/x")]` dan `#[route("/x", method = "GET")]`.
    pub fn discover_rust<P: AsRef<Path>>(dir: P) -> Vec<RouteEndpoint> {
        let mut routes = Vec::new();
        let files =
            collect_source_files(dir.as_ref(), &["rs"], &["target", ".git", "node_modules"]);

        let route_re = Regex::new(
            r#"(?i)\.route\s*\(\s*"([^"]*)"\s*,\s*(?:\w+::)?(get|post|put|delete|patch|head)\s*\("#,
        )
        .unwrap();
        let resource_re = Regex::new(r#"web::resource\s*\(\s*"([^"]*)""#).unwrap();
        let macro_re =
            Regex::new(r#"(?i)#\[\s*(get|post|put|delete|patch|head)\s*\(\s*"([^"]*)""#).unwrap();
        let route_macro_re = Regex::new(
            r#"(?i)#\[\s*route\s*\(\s*"([^"]*)"\s*,\s*method\s*=\s*"([^"]*)""#,
        )
        .unwrap();

        for f in &files {
            let content = match fs::read_to_string(f) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for line in content.lines() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                for cap in route_re.captures_iter(line) {
                    push_route(&mut routes, cap[2].to_uppercase(), "", &cap[1]);
                }
                for cap in resource_re.captures_iter(line) {
                    push_route(&mut routes, "GET".to_string(), "", &cap[1]);
                }
                for cap in macro_re.captures_iter(line) {
                    push_route(&mut routes, cap[1].to_uppercase(), "", &cap[2]);
                }
                for cap in route_macro_re.captures_iter(line) {
                    push_route(&mut routes, cap[2].to_uppercase(), "", &cap[1]);
                }
            }
        }
        routes
    }
}

/// Menambahkan satu rute ke daftar: membangun path gabungan (prefix + raw),
/// lalu men-dedup per (metode, path).
fn push_route(routes: &mut Vec<RouteEndpoint>, method: String, active_prefix: &str, raw: &str) {
    let prefix = active_prefix.trim_matches('/');
    let raw = raw.trim_start_matches('/');

    let mut path = String::new();
    if !prefix.is_empty() {
        path.push('/');
        path.push_str(prefix);
    }
    if !raw.is_empty() {
        // Path selalu diawali '/' — bahkan bila prefix kosong (mis. "/users", bukan "users")
        if path.is_empty() {
            path.push('/');
        } else if !path.ends_with('/') {
            path.push('/');
        }
        path.push_str(raw);
    }
    if path.is_empty() {
        path.push('/');
    }
    while path.contains("//") {
        path = path.replace("//", "/");
    }

    let name = path
        .trim_start_matches("/api/")
        .trim_start_matches('/')
        .replace('/', " ")
        .replace('-', " ");
    if name.trim().is_empty() {
        return;
    }

    if routes.iter().any(|r| r.method == method && r.path == path) {
        return;
    }

    routes.push(RouteEndpoint {
        path,
        name: format!("API {}", name),
        method,
        service: None,
        base_url: None,
        url: None,
    });
}

/// Apakah path mengandung placeholder parameter sehingga tak bisa ditembak otomatis?
/// Mendukung gaya `{id}` (Laravel/Go/axum), `[id]` (Next.js), `<id>` (Flask/actix/rocket),
/// `:id` (Express/Koa), dan wildcard `*`.
fn has_param(path: &str) -> bool {
    if path.contains('{') || path.contains('[') || path.contains('<') || path.contains('*') {
        return true;
    }
    path.split('/').any(|seg| seg.starts_with(':'))
}

/// Kumpulkan semua berkas sumber secara rekursif dengan ekstensi tertentu,
/// melewati direktori yang biasanya bukan kode aplikasi.
fn collect_source_files(dir: &Path, exts: &[&str], ignore: &[&str]) -> Vec<PathBuf> {
    fn walk(dir: &Path, exts: &[&str], ignore: &[&str], out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if !ignore.contains(&name.as_ref()) {
                        walk(&path, exts, ignore, out);
                    }
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if exts.contains(&ext) {
                        out.push(path);
                    }
                }
            }
        }
    }

    let mut out = Vec::new();
    walk(dir, exts, ignore, &mut out);
    out
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name != "node_modules" && name != ".next" && name != ".git" {
                    collect_files_recursive(&path, files);
                }
            } else {
                files.push(path);
            }
        }
    }
}
