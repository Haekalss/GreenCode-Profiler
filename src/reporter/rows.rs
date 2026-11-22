//! Row formatting for tables used in reports.

use super::badges::*;
use super::labels::*;
use super::recommendations::*;
use serde_json::Value;

pub fn correlated_row(module: &Value) -> String {
    let energy = number(module, "total_energy_joules");
    let score = number(module, "green_efficiency_score");
    let latency = number(module, "avg_latency_ms");
    let cpu = number(module, "avg_cpu_active_s");
    let calls = integer(module, "total_calls");
    format!(
        r#"<tr>
                        <td style="font-family:monospace;font-size:13px;">{}</td>
                        <td style="text-align:center;">{}</td>
                        <td style="text-align:right;color:{};font-weight:600;">{:.2} ms ({})</td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:right;">{:.3} s</td>
                        <td style="text-align:center;">{}</td>
                        <td style="font-size:12px;max-width:300px;">{}</td>
                    </tr>"#,
        text(module, "module_name"),
        calls,
        latency_color(latency),
        latency,
        latency_label(latency),
        energy_badge(energy),
        cpu,
        score_badge(score),
        module_recommendation(score, energy, latency)
    )
}

pub fn endpoint_row(index: usize, endpoint: &Value) -> String {
    let status = integer(endpoint, "status");
    let energy = number(endpoint, "energy_total_joules");
    let latency = number(endpoint, "latency_ms");
    let cpu = number(endpoint, "cpu_active_time_s");
    format!(
        r#"<tr>
                        <td style="text-align:center;">{}</td>
                        <td style="font-weight:600;color:#2563eb;">{}</td>
                        <td style="font-family:monospace;font-size:12px;">{}</td>
                        <td style="text-align:center;">{}</td>
                        <td style="text-align:right;color:{};font-weight:600;">{:.2} ms ({})</td>
                        <td style="text-align:right;">{:.3} s</td>
                        <td style="text-align:right;font-weight:600;color:{};">{:.4} J</td>
                        <td style="font-size:12px;max-width:280px;">{}</td>
                    </tr>"#,
        index + 1,
        text(endpoint, "method"),
        text(endpoint, "endpoint"),
        status_badge(status),
        latency_color(latency),
        latency,
        endpoint_latency_label(latency),
        cpu,
        energy_color(energy),
        energy,
        endpoint_recommendation(latency, energy)
    )
}

pub fn smell_row(smell: &Value) -> String {
    let severity = text(smell, "severity");
    let smell_type = text(smell, "smell_type");
    format!(
        r#"<tr>
                        <td style="text-align:center;"><span style="background:{};color:white;padding:2px 8px;border-radius:4px;font-size:11px;font-weight:bold;">{}</span></td>
                        <td style="font-weight:600;">{}</td>
                        <td>{}</td>
                        <td style="font-family:monospace;font-size:12px;word-break:break-all;max-width:280px;">{}:{}</td>
                        <td style="font-size:12px;max-width:280px;">{}</td>
                    </tr>"#,
        severity_color(severity),
        severity,
        smell_type,
        text(smell, "message"),
        text(smell, "file"),
        integer(smell, "line"),
        smell_recommendation(smell_type)
    )
}

pub fn file_row(file: &Value) -> String {
    let ccn = integer(file, "ccn");
    let ncloc = integer(file, "ncloc");
    let mi = number(file, "maintainability_index");
    let smells = file["code_smells"].as_array().map(|a| a.len()).unwrap_or(0);
    format!(
        r#"<tr>
                        <td style="font-family:monospace;font-size:12px;word-break:break-all;max-width:320px;">{}</td>
                        <td style="text-align:center;"><span style="background:#f1f5f9;padding:2px 6px;border-radius:4px;">{}</span></td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:right;">{}</td>
                        <td style="text-align:center;">{}</td>
                        <td style="font-size:12px;max-width:260px;">{}</td>
                    </tr>"#,
        text(file, "file_path"),
        text(file, "language"),
        ncloc,
        ccn_badge(ccn),
        mi_badge(mi),
        smell_badge(smells),
        file_recommendation(ccn, mi, ncloc, smells)
    )
}

pub fn php_extension_row(extension: &Value) -> String {
    let status = text(extension, "status_badge");
    format!(
        r#"<tr>
                            <td style="font-weight:600;">{}</td>
                            <td style="text-align:center;"><span style="padding:2px 8px;border-radius:4px;font-size:11px;font-weight:bold;{}">{}</span></td>
                            <td>{}</td>
                            <td>{}</td>
                        </tr>"#,
        text(extension, "name"),
        php_badge_style(status),
        status,
        text(extension, "description"),
        text(extension, "green_impact")
    )
}

pub fn filter_banner(applied_filter: &Value) -> String {
    if applied_filter.is_null() {
        return String::new();
    }
    let list = |key: &str| -> Vec<String> {
        applied_filter[key]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let filters = list("filters");
    let excludes = list("excludes");
    let matched = integer(applied_filter, "matched_files");
    let considered = integer(applied_filter, "considered_files");
    let exclude_part = if excludes.is_empty() {
        String::new()
    } else {
        format!(" | Exclude: <strong>{}</strong>", excludes.join(", "))
    };
    format!(
        r#"<div class="desc-box" style="background:#ecfeff;border-color:#a5f3fc;color:#155e75;">
            <h3>🔎 Filter Kode Aktif</h3>
            <p style="margin:0;">Keyword: <strong>{}</strong>{} — <strong>{}</strong> dari <strong>{}</strong> file kode cocok (path atau isi).</p>
        </div>"#,
        filters.join(", "),
        exclude_part,
        matched,
        considered
    )
}

pub fn language_cards(language_breakdown: &Value) -> String {
    language_breakdown
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| {
                    format!(
                        r#"<div class="card"><div class="lbl">{}</div><div class="val">{}</div><div style="font-size:12px;color:#64748b;margin-top:4px;">file</div></div>"#,
                        text(item, "language"),
                        integer(item, "files")
                    )
                })
                .collect::<String>()
        })
        .unwrap_or_default()
}
