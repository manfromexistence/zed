use serde_json::json;

use super::{models, providers};

#[test]
fn grouped_model_rows_skip_blank_string_entries() {
    let receipt = json!({
        "schema_version": "dx.agents.zed.models_list.v1",
        "status": "ok",
        "providers": [{
            "id": "nvidia",
            "display_name": "NVIDIA NIM",
            "models": ["  ", "\n\t", "nvidia/nemotron"]
        }]
    });

    let rows = models(&receipt);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "nvidia/nemotron");
    assert_eq!(rows[0].provider_id, "nvidia");
    assert_eq!(rows[0].model_id, "nvidia/nemotron");
}

#[test]
fn provider_and_model_rows_bound_and_redact_display_values() {
    let long_display_name = format!("NVIDIA NIM {}", "x".repeat(260));
    let long_model_id = format!("nemotron-{}", "y".repeat(260));
    let receipt = json!({
        "schema_version": "dx.agents.zed.models_list.v1",
        "status": "ok",
        "providers": [{
            "id": " nvidia ",
            "display_name": long_display_name,
            "status": " ready\nstate ",
            "source": " dx_providers_catalog\t",
            "model_count": 1,
            "configured_aliases": [
                " nvapi-secret-value ",
                " nvidia-nim ",
                " \n ",
                "build.nvidia.com"
            ],
            "models": [{
                "id": " nvapi-leaking-id ",
                "model_id": long_model_id,
                "status": " live\nready ",
                "compatibility": [
                    " tools ",
                    "Bearer leaked-token"
                ],
                "active": true
            }]
        }]
    });

    let provider_rows = providers(&receipt);
    let model_rows = models(&receipt);

    assert_eq!(provider_rows.len(), 1);
    assert_eq!(provider_rows[0].id, "nvidia");
    assert!(provider_rows[0].display_name.starts_with("NVIDIA NIM "));
    assert!(provider_rows[0].display_name.ends_with("..."));
    assert!(provider_rows[0].display_name.chars().count() <= 180);
    assert_eq!(provider_rows[0].status, "ready state");
    assert_eq!(
        provider_rows[0].compatibility,
        vec![
            "source dx_providers_catalog".to_string(),
            "1 model".to_string(),
            "alias <redacted>".to_string(),
            "alias nvidia-nim".to_string(),
            "alias build.nvidia.com".to_string(),
        ]
    );

    assert_eq!(model_rows.len(), 1);
    assert_eq!(model_rows[0].provider_id, "nvidia");
    assert!(model_rows[0].model_id.starts_with("nemotron-"));
    assert!(model_rows[0].model_id.ends_with("..."));
    assert!(model_rows[0].model_id.chars().count() <= 180);
    assert_eq!(model_rows[0].status, "live ready");
    assert!(model_rows[0].active);
    assert_eq!(
        model_rows[0].compatibility,
        vec!["tools".to_string(), "<redacted>".to_string()]
    );
}
