use serde_json::json;

use super::{models, providers};

#[test]
fn provider_rows_read_agent_cli_provider_receipts() {
    let receipt = json!({
        "schema_version": "dx.agents.zed.providers_list.v1",
        "status": "ok",
        "providers": [{
            "id": "deepseek",
            "display_name": "DeepSeek",
            "local": false,
            "source": "dx_providers_catalog",
            "model_count": 2,
            "configured": true,
            "configured_aliases": ["deepseek-chat"]
        }]
    });

    let rows = providers(&receipt);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "deepseek");
    assert_eq!(rows[0].display_name, "DeepSeek");
    assert_eq!(rows[0].status, "ok");
    assert!(rows[0].configured);
    assert!(!rows[0].local);
    assert_eq!(
        rows[0].compatibility,
        vec![
            "source dx_providers_catalog".to_string(),
            "2 models".to_string(),
            "alias deepseek-chat".to_string()
        ]
    );
}

#[test]
fn model_rows_flatten_agent_cli_provider_model_groups() {
    let receipt = json!({
        "schema_version": "dx.agents.zed.models_list.v1",
        "status": "partial",
        "providers": [
            {
                "id": "deepseek",
                "display_name": "DeepSeek",
                "source": "dx_providers_catalog",
                "model_count": 2,
                "models": ["deepseek-chat", "deepseek-reasoner"],
                "catalog_error": null
            },
            {
                "id": "missing-provider",
                "display_name": "missing-provider",
                "source": "native",
                "model_count": 0,
                "models": [],
                "catalog_error": "provider was not found in the provider catalog"
            }
        ]
    });

    let rows = models(&receipt);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, "deepseek/deepseek-chat");
    assert_eq!(rows[0].provider_id, "deepseek");
    assert_eq!(rows[0].model_id, "deepseek-chat");
    assert_eq!(rows[0].status, "partial");
    assert!(!rows[0].active);
    assert_eq!(
        rows[0].compatibility,
        vec![
            "source dx_providers_catalog".to_string(),
            "2 models".to_string()
        ]
    );
    assert_eq!(rows[1].id, "deepseek/deepseek-reasoner");
}

#[test]
fn legacy_flat_model_rows_still_parse() {
    let receipt = json!({
        "models": [{
            "id": "openai/gpt-5",
            "provider_id": "openai",
            "model_id": "gpt-5",
            "status": "ready",
            "active": true,
            "compatibility": ["tools"]
        }]
    });

    let rows = models(&receipt);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "openai/gpt-5");
    assert_eq!(rows[0].provider_id, "openai");
    assert_eq!(rows[0].model_id, "gpt-5");
    assert_eq!(rows[0].status, "ready");
    assert!(rows[0].active);
    assert_eq!(rows[0].compatibility, vec!["tools".to_string()]);
}
