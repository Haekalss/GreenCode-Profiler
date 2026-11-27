//! Integration tests for calibrated code quality thresholds, PHP extension candidates, and multi-user simulation.
#![allow(dead_code)]

#[path = "../src/analyzer/mod.rs"]
mod analyzer;
#[path = "../src/simulator/mod.rs"]
mod simulator;

use analyzer::php_ext::detect_php_extension_candidates;
use analyzer::scanner::analyze_source_file;
use serde_json::json;
use simulator::{AuditSimulator, SimulationParams};
use std::path::Path;

#[test]
fn tsx_ui_component_respects_relaxed_450_ncloc_and_ignores_jsx_nesting() {
    let mut lines = Vec::new();
    lines.push("export default function BigTable({ items }: { items: any[] }) {".to_string());
    lines.push("  return (".to_string());
    lines.push("    <div><section><article><table><tbody>".to_string());
    lines.push("      {items.map((row) => (".to_string());
    lines.push("        <tr key={row.id}><td>".to_string());
    lines.push("          <select onChange={(e) => {".to_string());
    lines.push("            if (e.target.value) { console.log(e.target.value || 'ok'); }".to_string());
    lines.push("          }}>".to_string());
    for i in 0..340 {
        lines.push(format!("            <option value=\"{i}\">Item {i}</option>"));
    }
    lines.push("          </select>".to_string());
    lines.push("        </td></tr>".to_string());
    lines.push("      ))}".to_string());
    lines.push("    </tbody></table></article></section></div>".to_string());
    lines.push("  );".to_string());
    lines.push("}".to_string());
    let src = lines.join("\n");

    // Default pragmatic mode: ~355 NCLOC TSX should have 0 Fat Class & 0 Deep Nesting!
    let res_default = analyze_source_file(Path::new("BigTable.tsx"), "tsx", "typescript", &src, false);
    assert!(
        res_default.code_smells.is_empty(),
        "Expected 0 smells in default mode for 355-line TSX, got: {:?}",
        res_default.code_smells
    );

    // Strict mode: > 300 NCLOC triggers Fat Controller / Large Class
    let res_strict = analyze_source_file(Path::new("BigTable.tsx"), "tsx", "typescript", &src, true);
    assert!(
        res_strict
            .code_smells
            .iter()
            .any(|s| s.smell_type == "Fat Controller / Large Class"),
        "Expected Fat Class smell in --strict mode"
    );
}

#[test]
fn backend_php_true_deep_control_flow_is_detected() {
    let php_src = r#"<?php
class OrderProcessor {
    public function process($orders) {
        if (!empty($orders)) {
            foreach ($orders as $order) {
                if ($order->isActive()) {
                    if ($order->hasItems()) {
                        return true;
                    }
                }
            }
        }
        return false;
    }
}
"#;
    let res = analyze_source_file(Path::new("OrderProcessor.php"), "php", "php", php_src, false);
    assert!(
        res.code_smells.iter().any(|s| s.smell_type == "Deep Nesting"),
        "Expected true 4-level control-flow nesting inside PHP method to be detected"
    );
}

#[test]
fn detects_repeated_php_functions_as_extension_candidates() {
    let helper_php = r#"<?php
class XmlDocxBuilder {
    public function escapeXmlNode($raw) {
        $clean = htmlspecialchars($raw);
        foreach (['a', 'b'] as $ch) {
            $clean = preg_replace('/\s+/', ' ', $clean);
        }
        return $clean;
    }
}
"#;
    let caller_php = r#"<?php
class ReportExporter {
    public function build($rows, $builder) {
        foreach ($rows as $row) {
            $a = $builder->escapeXmlNode($row['title']);
            $b = $builder->escapeXmlNode($row['desc']);
            $c = $builder->escapeXmlNode($row['note']);
        }
    }
}
"#;
    let f1 = analyze_source_file(Path::new("XmlDocxBuilder.php"), "php", "php", helper_php, false);
    let f2 = analyze_source_file(Path::new("ReportExporter.php"), "php", "php", caller_php, false);
    let candidates = detect_php_extension_candidates(&[
        (f1, helper_php.to_string()),
        (f2, caller_php.to_string()),
    ]);
    assert!(!candidates.is_empty(), "Expected escapeXmlNode to be flagged as PHP Extension candidate");
    assert_eq!(candidates[0].function_name, "escapeXmlNode");
    assert!(candidates[0].loop_invocations >= 3);
}

#[test]
fn simulates_multi_user_load_from_audit_json() {
    let sample_audit = json!({
        "metadata": {
            "project_name": "SMKI Portal",
            "cpu_tdp_watts": 15.0
        },
        "totals": {
            "total_requests": 50,
            "total_joules": 12.5,
            "total_cpu_active_s": 0.75,
            "avg_latency_ms": 35.0
        },
        "code_quality": {
            "overview": { "total_code_smells": 0 },
            "php_extension_candidates": [
                { "function_name": "escapeXml" },
                { "function_name": "buildTableRowXml" }
            ]
        }
    });

    let mut params = SimulationParams::default();
    params.cpu_cores = 8;
    params.custom_users = Some(2_500);

    let sim = AuditSimulator::simulate(&sample_audit, &params);
    assert_eq!(sim.project_name, "SMKI Portal");
    assert!(sim.tiers.iter().any(|t| t.concurrent_users == 2_500));
    assert!(sim.max_concurrent_users_80pct > 0);
    let t100 = sim.tiers.first().unwrap();
    let t50k = sim.tiers.last().unwrap();
    assert!(t50k.monthly_kwh > t100.monthly_kwh);
    assert!(t50k.projected_p95_latency_ms > t100.projected_p95_latency_ms);
}
