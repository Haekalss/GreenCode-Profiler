//! Detection logic for PHP version, extensions, and runtime info.

use std::process::Command;

pub fn query_version() -> (bool, String) {
    let output = Command::new("php").args(["-v"]).output();
    let stdout = match output {
        Ok(ref o) => o.stdout.clone(),
        Err(_) => return (false, String::new()),
    };
    let raw = String::from_utf8_lossy(&stdout);
    let line = raw.lines().next().unwrap_or("PHP (Unknown Version)").to_string();
    (true, line)
}

pub fn query_modules() -> Vec<String> {
    let mut modules = Vec::new();
    if let Ok(mod_output) = Command::new("php").args(["-m"]).output() {
        let text = String::from_utf8_lossy(&mod_output.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('[') {
                modules.push(trimmed.to_lowercase());
            }
        }
    }
    modules
}

pub fn detect_jit_enabled() -> bool {
    let Ok(info_out) = Command::new("php").args(["-i"]).output() else {
        return false;
    };
    let text = String::from_utf8_lossy(&info_out.stdout);
    text.lines().any(is_jit_active_line)
}

fn is_jit_active_line(line: &str) -> bool {
    if !line.contains("opcache.jit") {
        return false;
    }
    const ACTIVE_KEYS: [&str; 4] = ["tracing", "1205", "function", "on"];
    ACTIVE_KEYS.iter().any(|key| line.contains(key))
}
