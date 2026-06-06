use serde_json::json;

use super::parser::panel_from_receipt_value;
use super::*;

#[test]
fn panel_receipt_keeps_detected_config_out_of_score() {
    let receipt = json!({
        "schema_version": "dx.check.receipt.v1",
        "next_actions": ["Review skipped expensive checks before final launch."],
        "pass_count": 9,
        "fail_count": 0,
        "warn_count": 2,
        "skipped_count": 5,
        "duration_ms": 37,
        "checked_paths": ["G:\\Dx", "G:\\Dx\\www"],
        "skipped_expensive_checks": [
            "Lighthouse execution skipped by default.",
            "Full E2E execution skipped by default."
        ],
        "zed": {
            "schema_version": "dx.check.zed_panel.v1",
            "status": "warning",
            "score_value": 410,
            "score_max": 500,
            "score_percent": 82,
            "score_estimated": true,
            "weight_profile": "dx-check.launch-default.v1",
            "generated_at_unix_ms": 1779400000000_u64,
            "refresh_command": "dx check --json",
            "detail_command": "dx check score --json",
            "scoring_config": {
                "status": "detected_not_applied",
                "config_path": ".dx/check/config.json",
                "applies_to_score": false
            },
            "sections": [
                {
                    "title": "Structure",
                    "score": 88,
                    "max_score": 100,
                    "estimated": false,
                    "status": "ready"
                }
            ],
            "warnings": [
                {
                    "code": "score-config-detected-not-applied",
                    "message": "Config detected, but launch scoring still uses defaults.",
                    "next_action": "Review configured weights."
                }
            ],
            "quick_fixes": [
                {
                    "label": "Review scoring config",
                    "next_action": "Open .dx/check/config.json.",
                    "risk_level": "config-review",
                    "requires_user_approval": false,
                    "writes_receipts": false
                }
            ]
        }
    });

    let snapshot = panel_from_receipt_value(PathBuf::from("check-latest.json"), &receipt);

    assert_eq!(snapshot.score_value, Some(410));
    assert_eq!(snapshot.score_max, Some(500));
    assert_eq!(snapshot.score_percent, Some(82));
    assert!(snapshot.score_estimated);
    assert_eq!(snapshot.last_run_label, "Last run Unix ms: 1779400000000");
    assert_eq!(snapshot.pass_count, Some(9));
    assert_eq!(snapshot.fail_count, Some(0));
    assert_eq!(snapshot.warn_count, Some(2));
    assert_eq!(snapshot.skipped_count, Some(5));
    assert_eq!(snapshot.duration_ms, Some(37));
    assert_eq!(snapshot.checked_paths, vec!["G:\\Dx", "G:\\Dx\\www"]);
    assert_eq!(snapshot.skipped_expensive_checks.len(), 2);
    assert_eq!(
        snapshot.skipped_expensive_checks[0],
        "Lighthouse execution skipped by default."
    );
    assert_eq!(snapshot.scoring_config_status, "detected_not_applied");
    assert!(!snapshot.scoring_config_applies_to_score);
    assert!(snapshot.scoring_config_summary.contains("not applied"));
    assert_eq!(snapshot.sections.len(), 1);
    assert_eq!(
        snapshot.warnings[0].code,
        "score-config-detected-not-applied"
    );
    assert_eq!(snapshot.quick_fixes[0].label, "Review scoring config");
    assert_eq!(snapshot.quick_fixes[0].risk_level, "config-review");
    assert!(!snapshot.quick_fixes[0].requires_user_approval);
    assert!(!snapshot.quick_fixes[0].writes_receipts);
    assert!(snapshot.web_audits.is_empty());
}

#[test]
fn unsupported_zed_schema_does_not_claim_score() {
    let receipt = json!({
        "schema_version": "dx.check.receipt.v1",
        "zed": {
            "schema_version": "dx.check.zed_panel.v0",
            "score_value": 500,
            "score_max": 500
        }
    });

    let snapshot = panel_from_receipt_value(PathBuf::from("check-latest.json"), &receipt);

    assert_eq!(snapshot.status, "malformed");
    assert_eq!(snapshot.score_value, None);
    assert_eq!(snapshot.score_max, Some(500));
    assert_eq!(snapshot.blockers.len(), 1);
}

#[test]
fn view_model_only_receipt_can_render_without_zed_panel() {
    let receipt = json!({
        "schema_version": "dx.check.receipt.v1",
        "weight_profile": "dx-check.launch-default.v1",
        "pass_count": 9,
        "fail_count": 0,
        "warn_count": 2,
        "skipped_count": 5,
        "duration_ms": 37,
        "checked_paths": ["."],
        "skipped_expensive_checks": ["CDP/browser metrics skipped by default."],
        "view_model": {
            "schema_version": "dx.www.check_panel_view_model.v1",
            "status": "ready",
            "title": "dx-check project health",
            "score_meter": {
                "value": 410,
                "max": 500,
                "percent": 82,
                "estimated": true
            },
            "last_run_unix_ms": 1779400000000_u64,
            "last_run_label": "2 minutes ago",
            "bucket_rows": [
                {
                    "title": "Web performance",
                    "score": 70,
                    "max_score": 100,
                    "estimated": true,
                    "status": "warning"
                }
            ],
            "blocker_rows": [],
            "warning_rows": [
                {
                    "code": "web-lighthouse-skipped",
                    "message": "Lighthouse did not run.",
                    "next_action": "Run an approved Lighthouse adapter later."
                }
            ],
            "quick_fix_rows": [
                {
                    "label": "Run web probe",
                    "next_action": "Collect bounded HTTP metadata.",
                    "command": "dx check web --url http://localhost:3000 --json"
                }
            ],
            "primary_action": {
                "command": "dx check --json"
            },
            "secondary_action": {
                "command": "dx check score --json"
            },
            "scoring_config": {
                "status": "default",
                "applies_to_score": true
            }
        }
    });

    let snapshot = panel_from_receipt_value(PathBuf::from("check-latest.json"), &receipt);

    assert_eq!(snapshot.source_schema, "dx.www.check_panel_view_model.v1");
    assert_eq!(snapshot.status, "ready");
    assert_eq!(snapshot.score_value, Some(410));
    assert_eq!(snapshot.score_max, Some(500));
    assert_eq!(snapshot.last_run_label, "2 minutes ago");
    assert_eq!(snapshot.pass_count, Some(9));
    assert_eq!(snapshot.fail_count, Some(0));
    assert_eq!(snapshot.warn_count, Some(2));
    assert_eq!(snapshot.skipped_count, Some(5));
    assert_eq!(snapshot.duration_ms, Some(37));
    assert_eq!(snapshot.checked_paths, vec!["."]);
    assert_eq!(
        snapshot.skipped_expensive_checks,
        vec!["CDP/browser metrics skipped by default."]
    );
    assert_eq!(snapshot.sections[0].title, "Web performance");
    assert_eq!(snapshot.warnings[0].code, "zed-panel-fallback-view-model");
    assert_eq!(snapshot.warnings[1].code, "web-lighthouse-skipped");
    assert_eq!(snapshot.quick_fixes[0].label, "Run web probe");
    assert_eq!(snapshot.quick_fixes[0].risk_level, "receipt-write");
    assert!(!snapshot.quick_fixes[0].requires_user_approval);
    assert!(snapshot.quick_fixes[0].writes_receipts);
    assert_eq!(snapshot.refresh_command, "dx check --json");
    assert_eq!(
        snapshot.detail_command.as_deref(),
        Some("dx check score --json")
    );
}

#[test]
fn engine_web_audit_results_render_as_panel_rows() {
    let receipt = json!({
        "schema_version": "dx.check.receipt.v1",
        "pass_count": 9,
        "fail_count": 0,
        "warn_count": 1,
        "skipped_count": 0,
        "duration_ms": 37,
        "zed": {
            "schema_version": "dx.check.zed_panel.v1",
            "status": "warning",
            "score_value": 486,
            "score_max": 500,
            "score_percent": 97,
            "score_estimated": false,
            "weight_profile": "dx-check.launch-default.v1",
            "generated_at_unix_ms": 1779400000000_u64,
            "refresh_command": "dx check --json",
            "sections": []
        },
        "engine": {
            "web_audit_results": [
                {
                    "id": "home-run",
                    "target_id": "home",
                    "url": "http://localhost:3000/",
                    "status": 200,
                    "html_bytes": 18200,
                    "title_present": true,
                    "description_present": true,
                    "canonical_present": false,
                    "viewport_present": true,
                    "security_header_count": 3,
                    "source": ".dx/receipts/check/web-home.json"
                }
            ]
        }
    });

    let snapshot = panel_from_receipt_value(PathBuf::from("check-latest.json"), &receipt);

    assert_eq!(snapshot.web_audits.len(), 1);
    assert_eq!(snapshot.web_audits[0].label, "home");
    assert_eq!(snapshot.web_audits[0].status, "ready");
    assert!(snapshot.web_audits[0].detail.contains("HTTP 200"));
    assert!(snapshot.web_audits[0].detail.contains("18.2 KB"));
    assert_eq!(
        snapshot.web_audits[0].source.as_deref(),
        Some(".dx/receipts/check/web-home.json")
    );
}

#[test]
fn runner_lighthouse_report_renders_as_panel_row() {
    let receipt = json!({
        "schema_version": "dx.check.receipt.v1",
        "pass_count": 9,
        "fail_count": 0,
        "warn_count": 1,
        "skipped_count": 0,
        "duration_ms": 37,
        "zed": {
            "schema_version": "dx.check.zed_panel.v1",
            "status": "warning",
            "score_value": 486,
            "score_max": 500,
            "score_percent": 97,
            "score_estimated": false,
            "weight_profile": "dx-check.launch-default.v1",
            "generated_at_unix_ms": 1779400000000_u64,
            "refresh_command": "dx check --json",
            "sections": []
        },
        "web": {
            "lighthouse": {
                "schema_version": "dx.check.web_lighthouse",
                "id": "home-lighthouse",
                "target_id": "home",
                "url": "http://localhost:3000/",
                "score": 350,
                "max_score": 400,
                "categories": [
                    {
                        "id": "performance",
                        "label": "Performance",
                        "score": 80,
                        "max_score": 100,
                        "status": "warning"
                    },
                    {
                        "id": "accessibility",
                        "label": "Accessibility",
                        "score": 90,
                        "max_score": 100,
                        "status": "warning"
                    },
                    {
                        "id": "seo",
                        "label": "SEO",
                        "score": 100,
                        "max_score": 100,
                        "status": "ready"
                    },
                    {
                        "id": "best-practices",
                        "label": "Best Practices",
                        "score": 80,
                        "max_score": 100,
                        "status": "warning"
                    }
                ],
                "audits": []
            }
        }
    });

    let snapshot = panel_from_receipt_value(PathBuf::from("check-latest.json"), &receipt);

    assert_eq!(snapshot.web_audits.len(), 1);
    assert_eq!(snapshot.web_audits[0].label, "home Lighthouse");
    assert_eq!(snapshot.web_audits[0].status, "warning");
    assert!(snapshot.web_audits[0].detail.contains("350/400"));
    assert!(snapshot.web_audits[0].detail.contains("Performance 80"));
    assert!(snapshot.web_audits[0].detail.contains("Accessibility 90"));
    assert!(snapshot.web_audits[0].detail.contains("SEO 100"));
    assert!(snapshot.web_audits[0].detail.contains("Best Practices 80"));
}
