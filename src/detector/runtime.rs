//! Active server process discovery (PID lookup by listening port).

use super::RuntimeDetector;
use std::process::Command;

impl RuntimeDetector {
    /// Mencari PID proses server aktif menggunakan netstat/lsof bawaan OS.
    pub fn find_pids(language: &str, target_port: Option<u16>) -> Vec<u32> {
        let ports = ports_to_check(language, target_port);
        let mut pids = collect_pids(&ports);
        pids.sort_unstable();
        pids.dedup();
        pids
    }
}

fn ports_to_check(language: &str, target_port: Option<u16>) -> Vec<u16> {
    if let Some(port) = target_port {
        return vec![port];
    }
    match language {
        "fullstack" => vec![8000, 3000],
        "php" => vec![8000],
        "node" => vec![3000],
        "rust" | "go" => vec![8080, 3000],
        _ => vec![8000, 3000, 8080, 5000],
    }
}

#[cfg(target_os = "windows")]
fn collect_pids(ports: &[u16]) -> Vec<u32> {
    let Ok(output) = Command::new("netstat").args(["-ano"]).output() else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter(|line| is_listener_on(line, ports))
        .filter_map(parse_last_pid)
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn collect_pids(ports: &[u16]) -> Vec<u32> {
    ports
        .iter()
        .flat_map(|port| lsof_pids(*port))
        .collect()
}

#[cfg(target_os = "windows")]
fn is_listener_on(line: &str, ports: &[u16]) -> bool {
    line.contains("LISTENING")
        && ports
            .iter()
            .any(|port| line.contains(&format!(":{}", port)))
}

#[cfg(target_os = "windows")]
fn parse_last_pid(line: &str) -> Option<u32> {
    line.split_whitespace().last()?.parse().ok()
}

#[cfg(not(target_os = "windows"))]
fn lsof_pids(port: u16) -> Vec<u32> {
    let Ok(output) = Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect()
}
