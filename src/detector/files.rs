//! Recursive source-file collection shared by the route discoverers.

use std::fs;
use std::path::{Path, PathBuf};

/// Kumpulkan berkas sumber dengan ekstensi tertentu, melewati direktori non-aplikasi.
pub(super) fn collect_source_files(dir: &Path, exts: &[&str], ignore: &[&str]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(dir, exts, ignore, &mut out);
    out
}

fn walk(dir: &Path, exts: &[&str], ignore: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if should_visit(&path, ignore) {
                walk(&path, exts, ignore, out);
            }
            continue;
        }
        if has_extension(&path, exts) {
            out.push(path);
        }
    }
}

fn should_visit(path: &Path, ignore: &[&str]) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    !ignore.contains(&name.as_ref())
}

fn has_extension(path: &Path, exts: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| exts.contains(&ext))
}

/// Kumpulkan seluruh berkas di bawah `dir`, melewati direktori build umum.
pub(super) fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    const IGNORED: [&str; 3] = ["node_modules", ".next", ".git"];
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !IGNORED.contains(&name.as_ref()) {
                collect_files_recursive(&path, files);
            }
        } else {
            files.push(path);
        }
    }
}
