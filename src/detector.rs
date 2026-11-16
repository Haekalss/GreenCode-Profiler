//! Runtime Detector & Route Discoverer in Pure Rust
//! Mendeteksi proses server aktif dan mengekstrak rute nyata dari Next.js & Laravel

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
    pub fn discover<P: AsRef<Path>>(project_dir: P, max_routes: usize) -> Vec<RouteEndpoint> {
        let p = project_dir.as_ref();
        let fs_info = RuntimeDetector::detect_fullstack_components(p);

        if fs_info.is_fullstack {
            let mut routes = Vec::new();
            let fe = fs_info.frontend.unwrap();
            let be = fs_info.backend.unwrap();

            let fe_dir = p.join(&fe.dir);
            let be_dir = p.join(&be.dir);

            // 1. Frontend routes (Next.js) - abaikan route parameter mentah [id], [rfc] agar tidak 404
            let fe_routes = Self::discover_nextjs(&fe_dir);
            let limit_fe = max_routes / 2;
            for r in fe_routes.into_iter().take(limit_fe) {
                routes.push(RouteEndpoint {
                    path: r.path.clone(),
                    name: format!("[FE] {}", r.name),
                    method: r.method,
                    service: Some("frontend".to_string()),
                    base_url: Some(fe.base_url.clone()),
                    url: Some(format!("{}{}", fe.base_url, r.path)),
                });
            }

            // 2. Backend routes (Laravel) - hanya GET dan abaikan placeholder parameter {id}
            let be_routes = Self::discover_laravel(&be_dir);
            let limit_be = max_routes.saturating_sub(routes.len());
            for r in be_routes.into_iter().take(limit_be) {
                routes.push(RouteEndpoint {
                    path: r.path.clone(),
                    name: format!("[BE] {}", r.name),
                    method: r.method,
                    service: Some("backend".to_string()),
                    base_url: Some(be.base_url.clone()),
                    url: Some(format!("{}{}", be.base_url, r.path)),
                });
            }

            if !routes.is_empty() {
                return routes;
            }
        }

        let next_routes = Self::discover_nextjs(p);
        if !next_routes.is_empty() {
            return next_routes.into_iter().take(max_routes).collect();
        }

        let laravel_routes = Self::discover_laravel(p);
        if !laravel_routes.is_empty() {
            return laravel_routes.into_iter().take(max_routes).collect();
        }

        Vec::new()
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

                    // Filter rute dinamis seperti /[rfc], /[id], /ticket/[rfc] agar tidak otomatis ditembak dan menghasilkan HTTP 404
                    if clean_path.contains('[') || clean_path.contains(']') {
                        continue;
                    }

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

    pub fn discover_laravel<P: AsRef<Path>>(dir: P) -> Vec<RouteEndpoint> {
        let mut routes = Vec::new();
        let api_php = dir.as_ref().join("routes").join("api.php");
        let re =
            Regex::new(r#"Route::(get|post|put|delete|patch)\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

        if let Ok(content) = fs::read_to_string(api_php) {
            for cap in re.captures_iter(&content) {
                let method = cap[1].to_uppercase();
                let mut path = cap[2].to_string();
                if !path.starts_with('/') {
                    path = format!("/api/{}", path);
                } else if !path.starts_with("/api") {
                    path = format!("/api{}", path);
                }

                // Untuk profiling otomatis tanpa payload body, utamakan metode GET dan abaikan rute dinamis {id}
                if method != "GET" {
                    continue;
                }

                if !path.contains('{') && !routes.iter().any(|r: &RouteEndpoint| r.path == path) {
                    let name = path
                        .trim_start_matches("/api/")
                        .replace('/', " ")
                        .replace('-', " ");
                    
                    if name.trim().is_empty() {
                        continue;
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
            }
        }
        routes
    }
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
