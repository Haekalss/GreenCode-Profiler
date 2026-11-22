//! Project ecosystem and full-stack component detection.

use super::{component, ComponentInfo, FullstackDetection, RuntimeDetector};
use std::fs;
use std::path::Path;

const IGNORED_DIRS: [&str; 11] = [
    "node_modules",
    "vendor",
    ".git",
    ".next",
    "dist",
    "build",
    "tools",
    "reports",
    "storage",
    "venv",
    "target",
];

impl RuntimeDetector {
    /// Mendeteksi struktur monorepo/fullstack yang memisahkan Frontend dan Backend.
    pub fn detect_fullstack_components<P: AsRef<Path>>(project_dir: P) -> FullstackDetection {
        let mut backend: Option<ComponentInfo> = None;
        let mut frontend: Option<ComponentInfo> = None;

        for path in subdirectories(project_dir.as_ref()) {
            let dirname = dir_name(&path);
            if IGNORED_DIRS.contains(&dirname.as_str()) {
                continue;
            }
            if let Some(found) = detect_backend(&path, &dirname) {
                backend = Some(found);
            }
            if let Some(found) = detect_frontend(&path, &dirname) {
                frontend = Some(found);
            }
        }

        FullstackDetection {
            is_fullstack: backend.is_some() && frontend.is_some(),
            backend,
            frontend,
        }
    }

    /// Deteksi ekosistem project pada satu direktori.
    pub fn detect_project_ecosystem<P: AsRef<Path>>(project_dir: P) -> String {
        let fullstack = Self::detect_fullstack_components(&project_dir);
        if fullstack.is_fullstack {
            return "fullstack".to_string();
        }
        ecosystem_at(project_dir.as_ref()).to_string()
    }
}

fn subdirectories(dir: &Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect()
        })
        .unwrap_or_default()
}

fn dir_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn detect_backend(path: &Path, dirname: &str) -> Option<ComponentInfo> {
    if has_any(path, &["artisan", "composer.json"]) {
        let source = source_with_subdir(path, dirname, "app");
        return Some(component("php", "laravel", dirname, source, 8000));
    }
    if path.join("go.mod").exists() {
        return Some(component("go", "go", dirname, dirname.to_string(), 8080));
    }
    if path.join("Cargo.toml").exists() {
        let source = source_with_subdir(path, dirname, "src");
        return Some(component("rust", "actix_axum", dirname, source, 8080));
    }
    if has_any(
        path,
        &["requirements.txt", "pyproject.toml", "manage.py", "Pipfile"],
    ) {
        return Some(component("python", "python", dirname, dirname.to_string(), 8000));
    }
    None
}

fn detect_frontend(path: &Path, dirname: &str) -> Option<ComponentInfo> {
    if !path.join("package.json").exists() || !is_nextjs(path) {
        return None;
    }
    let source = source_with_subdir(path, dirname, "src");
    Some(component("node", "nextjs", dirname, source, 3000))
}

fn is_nextjs(path: &Path) -> bool {
    const MARKERS: [&str; 6] = [
        "next.config.js",
        "next.config.mjs",
        "next.config.ts",
        ".next",
        "src/app",
        "app",
    ];
    MARKERS.iter().any(|marker| path.join(marker).exists())
}

fn has_any(path: &Path, markers: &[&str]) -> bool {
    markers.iter().any(|marker| path.join(marker).exists())
}

fn source_with_subdir(path: &Path, dirname: &str, subdir: &str) -> String {
    if path.join(subdir).exists() {
        return format!("{}/{}", dirname, subdir);
    }
    dirname.to_string()
}

fn ecosystem_at(path: &Path) -> &'static str {
    const RULES: [(&str, &[&str]); 5] = [
        ("php", &["composer.json", "artisan"]),
        ("node", &["package.json"]),
        ("python", &["requirements.txt", "pyproject.toml"]),
        ("rust", &["Cargo.toml"]),
        ("go", &["go.mod"]),
    ];
    RULES
        .iter()
        .find(|(_, markers)| has_any(path, markers))
        .map(|(language, _)| *language)
        .unwrap_or("auto")
}
