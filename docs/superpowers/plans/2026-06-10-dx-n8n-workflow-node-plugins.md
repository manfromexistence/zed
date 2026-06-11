# DX n8n Workflow Node Plugins Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render n8n workflow nodes as DX-native configurable plugins in the DX Code Tools screen, then expose approved configured plugins to the AI panel and agent runtime.

**Architecture:** DX Code owns the GPUI plugin experience and never embeds the n8n editor as the primary surface. A DX JS sidecar reads built n8n node packages, resolves dynamic node options, and executes configured nodes through a DX-owned JSON-RPC bridge. Rust/GPUI consumes bounded catalog and receipt contracts, while credentials remain in DX-owned storage and agents only see approved configured plugins.

**Tech Stack:** Rust, GPUI, DX Agent bridge receipts, DX CLI public commands, DX JS `G:\Dx\js\build\release\bun.exe`, n8n built node packages, JSON-RPC over stdio first, optional localhost debug transport later.

---

## Current Evidence

- `G:\Dx\js\build\release\bun.exe` can build `n8n-nodes-base` and `@n8n/n8n-nodes-langchain`.
- The DX JS manual node build pipeline generated 420 base node definitions and 97 LangChain node definitions.
- DX JS can import representative built node modules and read their `description` metadata.
- `G:\Dx\code` already has a Tools screen at `crates\agent_ui\src\dx_launch_workspace\tools_screen.rs`.
- `G:\Dx\code` already has a receipt-backed bridge at `crates\agent_ui\src\dx_agent_bridge.rs`.

## Ownership Map

- GPUI Tools screen: `G:\Dx\code\crates\agent_ui\src\dx_launch_workspace\tools_screen.rs`.
- Native screen wrapper: `G:\Dx\code\crates\agent_ui\src\tools_screen.rs`.
- Rust bridge snapshot and command safety: `G:\Dx\code\crates\agent_ui\src\dx_agent_bridge.rs` and `G:\Dx\code\crates\agent_ui\src\dx_agent_bridge\*.rs`.
- DX sidecar runtime and CLI contracts: `G:\Dx\cli` or `G:\Dx\agent`, with the sidecar executed by DX JS.
- n8n source package input: `G:\Temp\n8n-node-experiment\n8n\packages\nodes-base` and `G:\Temp\n8n-node-experiment\n8n\packages\@n8n\nodes-langchain`.
- Generated DX receipts: `G:\Dx\.dx\receipts\plugins\n8n`.

---

## Phase 1: Catalog Contract

**Goal:** Produce a compact DX-owned catalog from built n8n node metadata.

**Files:**
- Create: `G:\Dx\cli\src\agents\plugins\n8n_catalog.rs`
- Create: `G:\Dx\cli\src\agents\plugins\n8n_schema.rs`
- Create: `G:\Dx\cli\tests\agents_n8n_catalog.rs`
- Create: `G:\Dx\code\crates\agent_ui\src\dx_agent_bridge\workflow_nodes.rs`

- [ ] Define `DxWorkflowNodeCatalog` with `schema`, `generated_at`, `source_packages`, `node_count`, `credential_type_count`, `nodes`, and `warnings`.
- [ ] Define `DxWorkflowNodeSummary` with `id`, `source_package`, `source_node_type`, `display_name`, `description`, `category`, `icon`, `inputs`, `outputs`, `parameter_count`, `credential_types`, `dynamic_option_count`, `trust_status`, and `configure_action`.
- [ ] Keep n8n identifiers stable in source fields. Do not rename `n8n-nodes-base.*` or `@n8n/n8n-nodes-langchain.*`.
- [ ] Generate `catalog-latest.json` under `.dx\receipts\plugins\n8n`.
- [ ] Verify counts are at least 420 base nodes and 97 LangChain nodes when both packages are present.
- [ ] Command: `dx agents plugins n8n catalog --json`.
- [ ] Expected receipt schema: `dx.agents.plugins.n8n.catalog`.

## Phase 2: DX JS Sidecar

**Goal:** Use DX JS as the node metadata and runtime adapter.

**Files:**
- Create: `G:\Dx\agent\bridges\n8n\src\main.ts`
- Create: `G:\Dx\agent\bridges\n8n\src\catalog.ts`
- Create: `G:\Dx\agent\bridges\n8n\src\describe-node.ts`
- Create: `G:\Dx\agent\bridges\n8n\src\resolve-options.ts`
- Create: `G:\Dx\agent\bridges\n8n\src\execute-node.ts`
- Create: `G:\Dx\agent\bridges\n8n\src\rpc.ts`
- Create: `G:\Dx\agent\bridges\n8n\tests\catalog.test.ts`

- [ ] Spawn the sidecar with `G:\Dx\js\build\release\bun.exe`.
- [ ] Support JSON-RPC methods: `catalog.listNodes`, `catalog.describeNode`, `node.resolveOptions`, `plugin.validateConfig`, `plugin.executeConfigured`.
- [ ] Load only allowlisted local n8n package roots.
- [ ] Never accept raw credential secrets over RPC from GPUI.
- [ ] Accept credential handles and ask the DX credential service for ephemeral values only at execution time.
- [ ] Return bounded errors with `code`, `message`, `node_id`, and `safe_next_action`.

## Phase 3: Rust Bridge Snapshot

**Goal:** Make the existing DX Agent bridge expose workflow-node catalog and configured plugin state to GPUI.

**Files:**
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_agent_bridge.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_agent_bridge\commands.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_agent_bridge\command_safety.rs`
- Create: `G:\Dx\code\crates\agent_ui\src\dx_agent_bridge\workflow_nodes.rs`
- Create: `G:\Dx\code\crates\agent_ui\src\dx_agent_bridge\workflow_node_tests.rs`

- [ ] Add `workflow_node_catalog` and `configured_plugins` fields to `DxAgentBridgeSnapshot`.
- [ ] Parse only bounded receipt files from `.dx\receipts\plugins\n8n`.
- [ ] Add public commands: `PluginsN8nCatalog`, `PluginsN8nDescribe`, `PluginsN8nConfigure`, `PluginsN8nApprove`, and `PluginsN8nRun`.
- [ ] Reject command arguments containing secret markers, shell separators, paths outside approved roots, or unbounded JSON.
- [ ] Keep approved agent tools separate from visible catalog nodes.

## Phase 4: GPUI Tools Screen

**Goal:** Render all nodes with icons, search, categories, add/configure, and approval status.

**Files:**
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_launch_workspace\tools_screen.rs`
- Create: `G:\Dx\code\crates\agent_ui\src\dx_launch_workspace\workflow_nodes.rs`
- Create: `G:\Dx\code\crates\agent_ui\src\dx_launch_workspace\workflow_node_config.rs`
- Create: `G:\Dx\code\crates\agent_ui\src\dx_launch_workspace\workflow_node_cards.rs`

- [ ] Add a `Workflow Nodes` section under Tools, next to Browser, Computer, MCP, and DX Plugins.
- [ ] Render loading, empty, stale, warning, and error states from catalog receipts.
- [ ] Render searchable node cards with icon, display name, category, credential requirement, and trust state.
- [ ] Add a real `Add` action that opens a configuration panel for the selected node.
- [ ] Render static parameter controls from metadata: string, number, boolean, options, multi-options, fixed collection, collection, JSON, and expression.
- [ ] Render dynamic parameter controls through `node.resolveOptions`.
- [ ] Render credential pickers as DX credential handles, never raw secret fields.
- [ ] Save configured plugin instances with stable ids and receipt-backed approval state.

## Phase 5: Agent Tool Exposure

**Goal:** Let the AI panel use configured workflow plugins safely.

**Files:**
- Modify: `G:\Dx\code\crates\agent_ui\src\agent_panel.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_agent_bridge\runtime.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_launch_workspace\agents\automations.rs`

- [ ] Expose only configured and approved plugins to the AI panel.
- [ ] Name agent tools from configured plugin intent, not raw n8n package names.
- [ ] Keep one generic execution bridge internally: `dx.agents.plugin.run_configured`.
- [ ] Show execution output, warnings, and receipt paths in the agent response context.
- [ ] Keep disabled or missing-credential plugins visible in Tools but unavailable to agents.

## Phase 6: Receipts, Trust, and Verification

**Goal:** Prove the bridge is source-guarded, credential-safe, and not dependent on the n8n website.

**Files:**
- Create: `G:\Dx\code\script\dx-workflow-node-plugin-source.test.ts`
- Modify: `G:\Dx\code\todo.txt`
- Modify: `G:\Dx\code\changelog.txt`

- [ ] Add a source guard that rejects n8n website iframe dependencies in the Tools screen.
- [ ] Add a source guard that requires credential handles instead of raw secret fields.
- [ ] Add a source guard that verifies public bridge commands remain allowlisted.
- [ ] Add a source guard that checks catalog and configured-plugin receipts are size bounded.
- [ ] Verify with `node --test script\dx-workflow-node-plugin-source.test.ts`.
- [ ] Verify touched Rust files with `rustfmt --edition 2024 --check <touched Rust files>`.
- [ ] Verify final source with `git diff --check`.

## Product Acceptance

- User opens Tools and sees a Workflow Nodes section with the full n8n-backed catalog.
- Search and category filtering work without launching the n8n website.
- Clicking Add opens real configuration controls generated from node metadata.
- Credential fields use DX credential handles only.
- Saving a configured plugin creates a receipt and an approval state.
- The AI panel can call only configured, approved plugins.
- Dynamic options are resolved through DX JS RPC.
- Execution receipts include node id, configured plugin id, inputs redaction status, output summary, errors, and next action.
- n8n package and workflow ids remain internally stable for compatibility.

## First Implementation Checkpoint

Build the catalog-only path first:

1. DX JS sidecar reads both n8n node packages.
2. DX CLI writes `catalog-latest.json`.
3. DX Code reads that receipt.
4. Tools screen renders searchable Workflow Node cards.
5. No configuration or execution is exposed until catalog rendering is stable.
