//! Collector for source code files across supported language extensions.

use std::fs;
use std::path::{Path, PathBuf};

pub fn collect_code_files(dir: &Path, ignore: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if ignore.contains(&name.as_str()) || name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            collect_code_files(&path, ignore, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}

pub fn detect_language(ext: &str) -> Option<&'static str> {
    match ext {
        "php" => Some("php"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" => Some("javascript"),
        "py" => Some("python"),
        "go" => Some("go"),
        "rs" => Some("rust"),
        _ => None,
    }
}
