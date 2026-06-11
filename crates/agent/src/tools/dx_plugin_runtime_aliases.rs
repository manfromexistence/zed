use super::dx_plugin_manifest::{
    DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA, DX_PLUGIN_RUNTIME_STATUS_ALIASES_SCHEMA,
};
use serde_json::Value;

pub(super) fn dx_plugin_runtime_aliases(runtime_green_readiness_scorecard: &Value) -> Value {
    let browser_ready =
        runtime_scorecard_lane_ready(runtime_green_readiness_scorecard, "browser_webpreview");
    let managed_chrome_ready =
        runtime_scorecard_lane_ready(runtime_green_readiness_scorecard, "managed_chrome");
    let pc_use_ready = runtime_scorecard_lane_ready(runtime_green_readiness_scorecard, "pc_use");
    let computer_ready = managed_chrome_ready && pc_use_ready;
    let runtime_green_candidate = runtime_green_readiness_scorecard
        .get("runtime_green_candidate")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    serde_json::json!({
        "schema": DX_PLUGIN_RUNTIME_STATUS_ALIASES_SCHEMA,
        "alias_schema": DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA,
        "status": "alias_metadata_available_runtime_evidence_required",
        "read_only": true,
        "claim_policy": "aliases_are_display_and_lookup_metadata_not_authorization",
        "aliases": {
            "dx.browser": {
                "schema": DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA,
                "canonical_plugin_id": "dx.browser",
                "maps_to_runtime_plugin_ids": ["zed.browser"],
                "maps_to_lane_ids": ["browser_webpreview"],
                "runtime_status_fields": ["plugins.browser"],
                "readiness_fields": [
                    "runtime_green_readiness_scorecard.lanes[browser_webpreview]",
                    "plugins.browser.panel_live_proof_readiness_card"
                ],
                "ready": browser_ready,
                "runtime_green_claim_ready": browser_ready && runtime_green_candidate,
                "claim_source": "derived_from_existing_zed_browser_fields",
                "claim_policy": "alias_only_existing_browser_runtime_evidence_required"
            },
            "dx.computer": {
                "schema": DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA,
                "canonical_plugin_id": "dx.computer",
                "maps_to_runtime_plugin_ids": ["zed.chrome", "zed.pc_use"],
                "maps_to_lane_ids": ["managed_chrome", "pc_use"],
                "runtime_status_fields": ["plugins.chrome", "plugins.pc_use"],
                "readiness_fields": [
                    "runtime_green_readiness_scorecard.lanes[managed_chrome]",
                    "runtime_green_readiness_scorecard.lanes[pc_use]"
                ],
                "ready": computer_ready,
                "runtime_green_claim_ready": computer_ready && runtime_green_candidate,
                "claim_source": "derived_from_managed_chrome_and_pc_use_fields",
                "claim_policy": "alias_only_managed_browser_and_future_pc_use_evidence_required",
                "safety": {
                    "managed_browser_only": true,
                    "pc_use_future_executor_only": true,
                    "os_wide_control": false,
                    "does_not_claim_desktop_input_ready": true
                }
            },
            "dx.driven": {
                "schema": DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA,
                "canonical_plugin_id": "dx.driven",
                "runtime_status_kind": "guarded_workflow_surface",
                "maps_to_runtime_plugin_ids": [],
                "maps_to_lane_ids": ["guarded_workflow_surface"],
                "runtime_status_fields": ["dx_plugin_runtime_aliases.aliases.dx.driven"],
                "guarded_surfaces": [
                    "plan_dx_runtime_proof",
                    "import_dx_runtime_proof",
                    "dx.launch_audit.source_guard.v1",
                    "tools/dx-runtime-proof/plans",
                    "tools/dx-runtime-proof/imports",
                    "tools/dx-runtime-proof/status"
                ],
                "metadata_ready": true,
                "ready": false,
                "runtime_green_claim_ready": false,
                "claim_policy": "metadata_only_no_executor_runtime_claim",
                "does_not_claim_executor_runtime": true
            }
        },
        "safety": {
            "read_only": true,
            "writes_files": false,
            "runs_node": false,
            "launches_browser": false,
            "dispatches_input": false,
            "aliases_do_not_authorize_actions": true
        }
    })
}

fn runtime_scorecard_lane_ready(scorecard: &Value, lane_id: &str) -> bool {
    scorecard
        .get("lanes")
        .and_then(Value::as_array)
        .and_then(|lanes| {
            lanes.iter().find(|lane| {
                lane.get("id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| id == lane_id)
            })
        })
        .and_then(|lane| lane.get("ready"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}
