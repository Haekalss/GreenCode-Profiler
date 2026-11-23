//! HTML and JSON report generation.
//!
//! Templates live in `templates/*.html` so presentation markup stays out of
//! Rust source files. Rendering helpers are shared by both report types.

pub mod badges;
pub mod energy;
pub mod labels;
pub mod quality;
pub mod recommendations;
pub mod render;
pub mod rows;

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct ReportGenerator;

pub const LOGO_BASE64: &str = include_str!("../logo_base64.txt");

impl ReportGenerator {
    pub fn save_json<P: AsRef<Path>>(data: &Value, path: P) -> std::io::Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(data)?;
        fs::write(p, json_str)
    }

    pub fn generate_html<P: AsRef<Path>>(report_data: &Value, path: P) -> std::io::Result<()> {
        energy::generate(report_data, path.as_ref())
    }

    pub fn generate_code_quality_html<P: AsRef<Path>>(
        report_data: &Value,
        path: P,
    ) -> std::io::Result<()> {
        quality::generate(report_data, path.as_ref())
    }

    pub fn open_in_browser<P: AsRef<Path>>(path: P) {
        let p = path.as_ref();
        let path_str = p.to_string_lossy().into_owned();
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd")
                .args(["/C", "start", "", &path_str])
                .spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open").arg(&path_str).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xdg-open").arg(&path_str).spawn();
        }
    }
}
