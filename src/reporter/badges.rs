//! Badges and numerical value formatters for reports.

use serde_json::Value;

pub fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or("")
}

pub fn number(value: &Value, key: &str) -> f64 {
    value[key].as_f64().unwrap_or(0.0)
}

pub fn integer(value: &Value, key: &str) -> u64 {
    value[key].as_u64().unwrap_or(0)
}

pub fn band<'a, T>(value: f64, bands: &'a [(f64, T)], fallback: &'a T) -> &'a T {
    bands
        .iter()
        .find(|(threshold, _)| value > *threshold)
        .map(|(_, label)| label)
        .unwrap_or(fallback)
}

pub fn band_ge<'a, T>(value: f64, bands: &'a [(f64, T)], fallback: &'a T) -> &'a T {
    bands
        .iter()
        .find(|(threshold, _)| value >= *threshold)
        .map(|(_, label)| label)
        .unwrap_or(fallback)
}

pub fn energy_badge(energy: f64) -> String {
    const HIGH: (&str, &str, &str, &str) = ("#fee2e2", "#991b1b", "700", "Tinggi");
    const MID: (&str, &str, &str, &str) = ("#fef3c7", "#92400e", "600", "Sedang");
    const LOW: (&str, &str, &str, &str) = ("#ecfdf5", "#065f46", "600", "Efisien");
    let &(bg, fg, weight, label) = band(energy, &[(10.0, HIGH), (2.0, MID)], &LOW);
    format!(
        r#"<span style="background:{bg};color:{fg};padding:3px 8px;border-radius:6px;font-weight:{weight};">{energy:.3} J ({label})</span>"#
    )
}

pub fn score_badge(score: f64) -> String {
    const GOOD: (&str, &str, &str) = ("#ecfdf5", "#065f46", "Optimal");
    const FAIR: (&str, &str, &str) = ("#fef3c7", "#92400e", "Cukup");
    const POOR: (&str, &str, &str) = ("#fee2e2", "#991b1b", "Boros");
    let &(bg, fg, label) = band_ge(score, &[(85.0, GOOD), (60.0, FAIR)], &POOR);
    format!(
        r#"<span style="background:{bg};color:{fg};padding:3px 8px;border-radius:12px;font-weight:bold;">{score:.1} ({label})</span>"#
    )
}

pub fn status_badge(status: u64) -> String {
    const OK: (&str, &str) = ("#ecfdf5", "#065f46");
    const FAIL: (&str, &str) = ("#fee2e2", "#991b1b");
    let (bg, fg) = if status == 200 { OK } else { FAIL };
    format!(
        r#"<span style="background:{bg};color:{fg};padding:2px 6px;border-radius:4px;font-weight:bold;">{status}</span>"#
    )
}

pub fn ccn_badge(ccn: u64) -> String {
    const HIGH: (&str, &str, &str) = ("#dc2626", "bold", "Tinggi");
    const MID: (&str, &str, &str) = ("#d97706", "600", "Sedang");
    const LOW: (&str, &str, &str) = ("#059669", "400", "Aman");
    let &(color, weight, label) = band(ccn as f64, &[(20.0, HIGH), (10.0, MID)], &LOW);
    format!(r#"<span style="color:{color};font-weight:{weight};">{ccn} ({label})</span>"#)
}

pub fn mi_badge(mi: f64) -> String {
    const A: (&str, &str) = ("#059669", "A");
    const B: (&str, &str) = ("#2563eb", "B");
    const C: (&str, &str) = ("#d97706", "C");
    const D: (&str, &str) = ("#dc2626", "D");
    let (color, grade) = if mi >= 70.0 {
        A
    } else if mi >= 55.0 {
        B
    } else if mi >= 40.0 {
        C
    } else {
        D
    };
    format!(r#"<span style="color:{color};font-weight:bold;">{mi:.1} ({grade})</span>"#)
}

pub fn smell_badge(count: usize) -> String {
    if count == 0 {
        return r#"<span style="color:#94a3b8;">0</span>"#.to_string();
    }
    format!(
        r#"<span style="background:#fee2e2;color:#991b1b;padding:2px 6px;border-radius:4px;font-weight:bold;">{count}</span>"#
    )
}

pub fn severity_color(severity: &str) -> &'static str {
    const BANDS: [(&str, &str); 2] = [("High", "#ef4444"), ("Medium", "#f59e0b")];
    BANDS
        .iter()
        .find(|(level, _)| *level == severity)
        .map(|(_, color)| *color)
        .unwrap_or("#3b82f6")
}

pub fn php_badge_style(status: &str) -> &'static str {
    const STYLES: [(&str, &str); 3] = [
        ("OK", "background:#10b981;color:white;"),
        ("WARN", "background:#f59e0b;color:white;"),
        ("DANGER", "background:#ef4444;color:white;"),
    ];
    STYLES
        .iter()
        .find(|(level, _)| *level == status)
        .map(|(_, style)| *style)
        .unwrap_or("background:#64748b;color:white;")
}
