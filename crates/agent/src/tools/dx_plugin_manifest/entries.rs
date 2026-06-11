use super::{
    DX_PLUGIN_MANIFEST_SCHEMA, DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA, DxPluginCredential,
    DxPluginManifest, DxPluginPermission, DxPluginPort, DxPluginReceipt, DxPluginRuntime,
    DxPluginRuntimeStatusAlias, DxPluginTrustStatus,
};

pub(super) fn first_party_plugin_manifests(receipt_root: String) -> Vec<DxPluginManifest> {
    vec![
        browser_manifest(receipt_root.clone()),
        computer_manifest(receipt_root.clone()),
        driven_manifest(receipt_root),
    ]
}

fn browser_manifest(receipt_root: String) -> DxPluginManifest {
    DxPluginManifest {
        schema: DX_PLUGIN_MANIFEST_SCHEMA,
        id: "dx.browser",
        name: "Browser",
        category: "browser_automation",
        description: "Controls the in-app Web Preview browser through DX-owned navigation, inspection, screenshot, and receipt handoff surfaces.",
        permissions: vec![
            permission(
                "web_preview.read",
                "read",
                false,
                "Read active page title, URL, ready state, and bounded page summaries.",
            ),
            permission(
                "web_preview.inspect_dom",
                "read",
                false,
                "Read bounded DOM, selector, and element evidence for Agent context.",
            ),
            permission(
                "web_preview.navigate",
                "action",
                true,
                "Navigate the in-app browser only through the permissioned Browser action bridge.",
            ),
            permission(
                "web_preview.input",
                "input",
                true,
                "Click, type, key, and scroll actions require fresh preflight and receipts.",
            ),
        ],
        runtime: runtime(
            "web_preview_native",
            "zed_web_preview",
            "repo_web_preview",
            "crates/web_preview/src/web_preview_view.rs",
            false,
            &receipt_root,
        ),
        runtime_status_alias: runtime_status_alias(
            "dx.browser",
            vec!["zed.browser"],
            vec!["browser_webpreview"],
            vec!["plugins.browser"],
            vec![
                "runtime_green_readiness_scorecard.lanes[browser_webpreview]",
                "plugins.browser.panel_live_proof_readiness_card",
            ],
            "alias_only_existing_browser_runtime_evidence_required",
        ),
        inputs: vec![
            port("url", "string", false, "Target URL or search text."),
            port(
                "selector",
                "css_selector",
                false,
                "Optional DOM selector for inspection or action preflight.",
            ),
            port(
                "text_payload",
                "redacted_text",
                false,
                "Explicit text payload imported through the Browser payload bridge.",
            ),
            port(
                "viewport",
                "viewport_request",
                false,
                "Responsive viewport dimensions or preset id.",
            ),
        ],
        outputs: vec![
            port(
                "page_state_summary",
                "json",
                true,
                "Bounded page URL, title, readiness, and event summary.",
            ),
            port(
                "dom_snapshot",
                "json",
                false,
                "Bounded selector or page DOM snapshot.",
            ),
            port(
                "screenshot_artifact",
                "artifact",
                false,
                "Viewport or selected-area screenshot artifact with metadata.",
            ),
            port(
                "browser_action_receipt",
                "receipt",
                true,
                "Receipt emitted for permissioned Browser actions.",
            ),
        ],
        credentials: Vec::new(),
        trust_status: trust_status(false),
        receipts: vec![
            receipt(
                "browser_payload_import",
                "zed.web_preview.agent_browser_action_payload_import_receipt.v1",
                &receipt_root,
                "payload import and action preflight",
            ),
            receipt(
                "browser_final_validation",
                "zed.web_preview.agent_browser_final_validation_result.v1",
                &receipt_root,
                "manual runtime validation evidence",
            ),
        ],
        source_root_ids: vec![
            "repo_agent_tools",
            "repo_web_preview",
            "workspace_agent_plugins",
        ],
        available_to: vec!["zed_plugins_panel", "dx_agents_bridge", "agent_panel"],
    }
}

fn computer_manifest(receipt_root: String) -> DxPluginManifest {
    DxPluginManifest {
        schema: DX_PLUGIN_MANIFEST_SCHEMA,
        id: "dx.computer",
        name: "Computer",
        category: "computer_control",
        description: "Controls managed Chrome and future desktop-browser paths with action_recording receipts, thumbnails, and video handoffs.",
        permissions: vec![
            permission(
                "managed_browser.read",
                "read",
                false,
                "Read managed Chrome queue, runner, execution, and target status.",
            ),
            permission(
                "managed_browser.launch",
                "action",
                true,
                "Launch or attach only to managed browser profiles after explicit permission.",
            ),
            permission(
                "managed_browser.input",
                "input",
                true,
                "Click, type, key, and scroll through managed browser adapters with receipts.",
            ),
            permission(
                "computer.action_recording",
                "recording",
                true,
                "Record action evidence as receipt-backed Agent-screen thumbnails and video-player handoffs.",
            ),
        ],
        runtime: runtime(
            "dxjs_managed_browser",
            "managed_chrome_playwright_adapter",
            "workspace_playwright_runner",
            "tools/playwright/zed-managed-chrome-runner/managed_chrome_runner.mjs",
            true,
            &receipt_root,
        ),
        runtime_status_alias: runtime_status_alias(
            "dx.computer",
            vec!["zed.chrome", "zed.pc_use"],
            vec!["managed_chrome", "pc_use"],
            vec!["plugins.chrome", "plugins.pc_use"],
            vec![
                "runtime_green_readiness_scorecard.lanes[managed_chrome]",
                "runtime_green_readiness_scorecard.lanes[pc_use]",
            ],
            "alias_only_managed_browser_and_future_pc_use_evidence_required",
        ),
        inputs: vec![
            port("url", "string", false, "Managed browser URL target."),
            port(
                "selector",
                "css_selector",
                false,
                "Managed browser selector target.",
            ),
            port(
                "target_snapshot_id",
                "receipt_id",
                false,
                "Target snapshot id required before future desktop input.",
            ),
            port(
                "action_recording_request",
                "json",
                false,
                "Action recording request metadata.",
            ),
        ],
        outputs: vec![
            port(
                "managed_chrome_receipt",
                "receipt",
                true,
                "Managed Chrome runner or execution receipt.",
            ),
            port(
                "agent_screen_recording_thumbnail",
                "artifact",
                false,
                "Thumbnail shown in Agent screen history.",
            ),
            port(
                "web_preview_video_player_handoff",
                "artifact",
                false,
                "Video handoff opened through the Zed Web Preview player.",
            ),
            port(
                "credential_status",
                "json",
                true,
                "Safe credential health without secret values.",
            ),
        ],
        credentials: vec![credential(
            "dx_chrome_extension",
            "managed_extension",
            "credential_status_required",
            true,
            true,
            "DX Chrome extension readiness and managed profile status.",
        )],
        trust_status: trust_status(true),
        receipts: vec![
            receipt(
                "managed_chrome_runner",
                "zed.agent_plugins.chrome.runner_receipt.v1",
                &receipt_root,
                "managed Chrome run gate",
            ),
            receipt(
                "managed_chrome_execution",
                "zed.agent_plugins.chrome.playwright_execution_receipt.v1",
                &receipt_root,
                "managed Chrome adapter execution",
            ),
            receipt(
                "computer_action_recording",
                "zed.dx_plugins.computer.action_recording_receipt.v1",
                &receipt_root,
                "Agent-screen recording and Web Preview video handoff",
            ),
        ],
        source_root_ids: vec![
            "repo_agent_tools",
            "repo_agent_ui_bridge",
            "workspace_playwright_runner",
        ],
        available_to: vec!["zed_plugins_panel", "dx_agents_bridge", "agent_panel"],
    }
}

fn driven_manifest(receipt_root: String) -> DxPluginManifest {
    DxPluginManifest {
        schema: DX_PLUGIN_MANIFEST_SCHEMA,
        id: "dx.driven",
        name: "Driven",
        category: "workflow_nodes",
        description: "Runs DX-native workflow nodes for lanes, worker prompts, goals, checkpoints, source guards, verification policy, and receipts.",
        permissions: vec![
            permission(
                "dx_lanes.read",
                "read",
                false,
                "Read lane ownership, pass status, and checkpoint summaries.",
            ),
            permission(
                "worker_prompts.prepare",
                "action",
                true,
                "Prepare bounded worker prompts for approved DX lanes.",
            ),
            permission(
                "source_guards.run",
                "verification",
                false,
                "Run focused source guard scans without builds or runtime side effects.",
            ),
            permission(
                "checkpoint_receipts.write",
                "receipt",
                true,
                "Write checkpoint receipts for workflow-node decisions and verification evidence.",
            ),
        ],
        runtime: runtime(
            "dx_workflow_nodes",
            "dx_agents_bridge",
            "repo_agent_ui_bridge_module",
            "crates/agent_ui/src/dx_agent_bridge.rs",
            false,
            &receipt_root,
        ),
        runtime_status_alias: runtime_status_alias(
            "dx.driven",
            Vec::new(),
            vec!["guarded_workflow_surface"],
            vec!["dx_plugin_runtime_aliases.aliases.dx.driven"],
            vec![
                "plan_dx_runtime_proof",
                "import_dx_runtime_proof",
                "dx.launch_audit.source_guard.v1",
            ],
            "metadata_only_no_executor_runtime_claim",
        ),
        inputs: vec![
            port("lane", "dx_lane_id", true, "DX lane or pass identifier."),
            port(
                "goal",
                "goal_id",
                false,
                "Optional goal/checkpoint identifier.",
            ),
            port(
                "worker_prompts",
                "prompt_packet",
                false,
                "Bounded worker prompt packet.",
            ),
            port(
                "verification_policy",
                "json",
                true,
                "Source-only or governed runtime verification policy.",
            ),
        ],
        outputs: vec![
            port(
                "dx_lanes",
                "json",
                true,
                "Lane status and ownership evidence.",
            ),
            port(
                "worker_prompt_packet",
                "json",
                false,
                "Prepared worker prompt handoff.",
            ),
            port(
                "source_guard_report",
                "json",
                true,
                "Focused source guard result summary.",
            ),
            port(
                "checkpoint_receipts",
                "receipt",
                true,
                "Receipt-backed workflow checkpoint record.",
            ),
        ],
        credentials: Vec::new(),
        trust_status: trust_status(false),
        receipts: vec![
            receipt(
                "driven_checkpoint",
                "zed.dx_plugins.driven.checkpoint_receipt.v1",
                &receipt_root,
                "workflow checkpoint and lane handoff",
            ),
            receipt(
                "driven_source_guard",
                "zed.dx_plugins.driven.source_guard_receipt.v1",
                &receipt_root,
                "source guard and verification policy evidence",
            ),
        ],
        source_root_ids: vec![
            "repo_agent_tools",
            "repo_agent_ui_bridge",
            "repo_agent_ui_bridge_module",
        ],
        available_to: vec!["zed_plugins_panel", "dx_agents_bridge", "agent_panel"],
    }
}

fn permission(
    id: &'static str,
    level: &'static str,
    receipt_required: bool,
    description: &'static str,
) -> DxPluginPermission {
    DxPluginPermission {
        id,
        level,
        receipt_required,
        description,
    }
}

fn runtime(
    runtime: &'static str,
    engine: &'static str,
    source_root_id: &'static str,
    entrypoint: &'static str,
    dxjs_required: bool,
    receipt_root: &str,
) -> DxPluginRuntime {
    DxPluginRuntime {
        runtime,
        engine,
        source_root_id,
        entrypoint,
        cancellation: "agent_cancellation_token",
        receipt_root: receipt_root.to_string(),
        dxjs_required,
    }
}

fn runtime_status_alias(
    canonical_plugin_id: &'static str,
    maps_to_runtime_plugin_ids: Vec<&'static str>,
    maps_to_lane_ids: Vec<&'static str>,
    runtime_status_fields: Vec<&'static str>,
    readiness_fields: Vec<&'static str>,
    claim_policy: &'static str,
) -> DxPluginRuntimeStatusAlias {
    DxPluginRuntimeStatusAlias {
        schema: DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA,
        canonical_plugin_id,
        maps_to_runtime_plugin_ids,
        maps_to_lane_ids,
        runtime_status_fields,
        readiness_fields,
        claim_policy,
    }
}

fn port(
    id: &'static str,
    kind: &'static str,
    required: bool,
    description: &'static str,
) -> DxPluginPort {
    DxPluginPort {
        id,
        kind,
        required,
        description,
    }
}

fn credential(
    id: &'static str,
    kind: &'static str,
    credential_status: &'static str,
    required: bool,
    receipt_required: bool,
    description: &'static str,
) -> DxPluginCredential {
    DxPluginCredential {
        id,
        kind,
        credential_status,
        required,
        receipt_required,
        description,
    }
}

fn trust_status(requires_user_enablement_for_input: bool) -> DxPluginTrustStatus {
    DxPluginTrustStatus {
        status: "first_party_trusted",
        source_owned: true,
        first_party: true,
        enabled_by_default: true,
        requires_user_enablement_for_input,
    }
}

fn receipt(
    id: &'static str,
    schema: &'static str,
    receipt_root: &str,
    required_for: &'static str,
) -> DxPluginReceipt {
    DxPluginReceipt {
        id,
        schema,
        receipt_root: receipt_root.to_string(),
        required_for,
    }
}
