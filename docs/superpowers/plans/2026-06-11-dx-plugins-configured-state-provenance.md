# DX Plugins Configured State Provenance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the DX Plugins panel report configured state, catalog provenance, and credential configuration actions from source-backed receipts instead of stale catalog flags or hard-coded serializer claims.

**Architecture:** Keep the bridge as the single receipt reader. Add path-aware local JSON discovery, reconcile workflow-node configured state from configured-plugin receipts before UI rendering, and expose Configure as a no-secret Agent composer request rather than inline credential storage.

**Tech Stack:** Rust GPUI/Zed `agent_ui`, DX Agent bridge receipt parsers, focused Node source guards, rustfmt, git diff hygiene.

---

### Task 1: Catalog Receipt Provenance

**Files:**
- Modify: `crates/agent_ui/src/dx_agent_bridge/local_files.rs`
- Modify: `crates/agent_ui/src/dx_agent_bridge.rs`
- Modify: `script/dx-agent-bridge-source.test.ts`

- [x] **Step 1: Add a path-aware JSON helper**

Add helpers next to `read_first_json`:

```rust
pub(super) fn read_first_json_with_path(root: &Path, names: &[&str]) -> Option<(PathBuf, Value)> {
    names.iter().find_map(|name| {
        let path = root.join(name);
        read_json(&path).map(|value| (path, value))
    })
}
```

- [x] **Step 2: Use the helper for workflow-node catalogs**

In `read_bridge_snapshot`, replace the separate top-level/fallback workflow-node catalog reads with a single `(PathBuf, Option<Value>)` discovery that includes `plugins/workflow-node-catalog-latest.json`. Pass the winning path to `workflow_node_catalog_summary`, or the default path when no catalog exists.

- [x] **Step 3: Guard the source contract**

Update `script/dx-agent-bridge-source.test.ts` to assert `read_first_json_with_path`, `read_first_json_with_default_path`, `(workflow_node_catalog_path, workflow_node_catalog_value)`, and the `plugins/workflow-node-catalog-latest.json` fallback path are present.

### Task 2: Configured State Reconciliation

**Files:**
- Modify: `crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs`
- Modify: `crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured.rs`
- Modify: `script/dx-agent-bridge-source.test.ts`
- Modify: `script/dx-agent-workspaces-source.test.ts`

- [x] **Step 1: Parse a full configured-node index**

Add a helper in `configured.rs` that inspects `configured_plugins`, or `enabled_plugins` only when `configured_plugins` is absent, and returns whether configured plugin data is authoritative plus the full set of configured `node_id` values.

- [x] **Step 2: Reconcile nodes before rendering**

In `workflow_node_catalog_summary`, derive `configured` from the configured-node index whenever configured plugin data exists. Fall back to raw `nodes[].configured` only when no configured plugin list exists. Derive `configured_plugin_count` from reconciled configured nodes, not from a stale scalar.

- [x] **Step 3: Guard stale-state behavior**

Update source guards to assert the parser contains configured index logic, does not trust `configured_plugin_count` first, and that Plugins filters/cards still use the reconciled `node.configured` field.

### Task 3: Honest Serializer And Configure Bridge

**Files:**
- Modify: `crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs`
- Modify: `crates/agent_ui/src/dx_launch_workspace/tools_screen/details.rs`
- Modify: `crates/agent_ui/src/dx_launch_workspace/tools_screen/workflow_nodes.rs`
- Modify: `crates/agent_ui/src/agent_panel.rs`
- Modify: `script/dx-agent-workspaces-source.test.ts`
- Modify: `script/dx-agent-bridge-source.test.ts`

- [x] **Step 1: Stop defaulting missing serializer facts to ready values**

Change missing `schema_version` and missing `serializer_format` fallbacks to honest missing values. Render the selected detail pane from the catalog serializer/schema fields instead of hard-coded `dx.serializer.machine`.

- [x] **Step 2: Add a no-secret Configure request action**

Add an AgentPanel method that inserts a structured credential setup request into the active Agent composer. The prompt must include plugin name, node id, credential types, credential status, configure action, source package, source root id, and source path, and must not include secret fields.

- [x] **Step 3: Wire Configure menu entry to the action bridge**

Pass the AgentPanel weak entity from the catalog row into `workflow_node_card`. The Configure menu keeps status rows and adds a source-backed draft action (`Draft setup request`, `Review configuration`, or `Review plugin contract`) that calls the AgentPanel method.

- [x] **Step 4: Guard the UI/action contract**

Update source guards to assert the Configure menu uses `draft_dx_workflow_node_configuration_prompt`, includes source identifiers, avoids raw secret names, and still performs no direct receipt IO in render modules.

### Task 4: Documentation And Verification

**Files:**
- Modify: `todo.txt`
- Modify: `changelog.txt`
- Modify: `DX.md`

- [x] **Step 1: Record source-only status**

Update status docs with the configured-state provenance, catalog actual-path, serializer honesty, and no-secret Configure bridge work.

- [x] **Step 2: Run light verification**

Run:

```powershell
rustfmt --edition 2024 --check crates/agent_ui/src/dx_agent_bridge.rs crates/agent_ui/src/dx_agent_bridge/local_files.rs crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured.rs crates/agent_ui/src/dx_launch_workspace/tools_screen/catalog.rs crates/agent_ui/src/dx_launch_workspace/tools_screen/workflow_nodes.rs crates/agent_ui/src/dx_launch_workspace/tools_screen/details.rs crates/agent_ui/src/agent_panel.rs
node --test script\dx-agent-bridge-source.test.ts script\dx-agent-workspaces-source.test.ts script\dx-agent-thread-view-source.test.ts script\dx-plugin-catalog-source.test.ts
git diff --check
```

Expected: rustfmt exits 0, Node source guards pass, and git diff has no whitespace errors. Cargo, `just run`, local servers, and native visual proof remain deferred unless explicitly authorized.
