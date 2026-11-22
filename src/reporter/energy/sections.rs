//! Section formatters for the energy & performance profiling report.

use super::super::badges::*;
use super::super::render::*;
use serde_json::Value;

pub fn format_endpoint_rows(endpoints: &Value) -> String {
    endpoints
        .as_array()
        .map(|arr| {
            arr.iter()
                .enumerate()
                .map(|(i, ep)| endpoint_row(i, ep))
                .collect()
        })
        .unwrap_or_default()
}

pub fn format_module_rows(modules: &Value) -> String {
    modules
        .as_array()
        .map(|arr| arr.iter().map(correlated_row).collect())
        .unwrap_or_default()
}

pub fn format_file_rows(files: &Value) -> String {
    files
        .as_array()
        .map(|arr| arr.iter().map(file_row).collect())
        .unwrap_or_default()
}

pub fn format_smell_rows(smells: &Value) -> String {
    smells
        .as_array()
        .map(|arr| arr.iter().take(25).map(smell_row).collect())
        .unwrap_or_default()
}

pub fn format_files_summary(total_files_count: usize) -> String {
    let shown_files = total_files_count.min(25);
    if total_files_count > shown_files {
        format!(
            r#"<div style="font-size:12px;color:#64748b;margin:-6px 0 10px 0;">Menampilkan <strong>{}</strong> dari <strong>{}</strong> file — diurutkan berdasarkan risiko (code smells, CCN, MI). Data lengkap tersedia di <code>reports/code_quality.json</code>.</div>"#,
            shown_files, total_files_count
        )
    } else {
        format!(
            r#"<div style="font-size:12px;color:#64748b;margin:-6px 0 10px 0;">Menampilkan seluruh <strong>{}</strong> file — diurutkan berdasarkan risiko (code smells, CCN, MI).</div>"#,
            total_files_count
        )
    }
}

pub fn format_php_section(php: &Value) -> String {
    if php.is_null() || !php["is_php_available"].as_bool().unwrap_or(false) {
        return String::new();
    }

    let mut ext_rows = String::new();
    if let Some(exts) = php["extension_details"].as_array() {
        for e in exts {
            let status = text(e, "status_badge");
            ext_rows.push_str(&format!(
                r#"<tr>
                    <td style="font-weight:600;">{}</td>
                    <td style="text-align:center;"><span style="padding:2px 8px;border-radius:4px;font-size:11px;font-weight:bold;{}">{}</span></td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                text(e, "name"),
                php_badge_style(status),
                status,
                text(e, "description"),
                text(e, "green_impact")
            ));
        }
    }

    let mut rec_items = String::new();
    if let Some(recs) = php["recommendations"].as_array() {
        for r in recs {
            rec_items.push_str(&format!("<li>{}</li>", r.as_str().unwrap_or("")));
        }
    }

    let rec_box = if !rec_items.is_empty() {
        format!(
            r#"<div class="desc-box" style="background:#fffbeb;border-color:#fef3c7;color:#92400e;margin-top:15px;"><h3>💡 Rekomendasi Optimasi Runtime PHP</h3><ul>{}</ul></div>"#,
            rec_items
        )
    } else {
        String::new()
    };

    format!(
        r#"<div class="section-title" style="border-left-color:#8b5cf6;">6. Audit PHP Runtime & Green Extensions</div>
        <div class="desc-box" style="background:#f5f3ff;border-color:#ddd6fe;color:#5b21b6;">
            <h3>⚡ Efisiensi Energi Konfigurasi PHP (PHP Extension Health)</h3>
            <p style="margin:0 0 10px 0;">Versi Server: <strong>{}</strong> | Ekstensi Terpasang: <strong>{}</strong></p>
            <p style="margin:0;">Dalam prinsip Green Software Engineering, penghematan energi server PHP sangat dipengaruhi oleh penggunaan opcode caching (OPcache) dan ketiadaan overhead profiling debug (Xdebug).</p>
        </div>
        <table>
            <thead>
                <tr>
                    <th style="width:24%;">Nama Ekstensi</th>
                    <th style="width:12%;text-align:center;">Status</th>
                    <th style="width:30%;">Fungsi</th>
                    <th>Dampak Efisiensi Energi (Green Computing)</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
        {}"#,
        text(php, "php_version"),
        integer(php, "loaded_extensions_count"),
        ext_rows,
        rec_box
    )
}
