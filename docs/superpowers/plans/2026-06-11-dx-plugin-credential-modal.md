# DX Plugin Credential Modal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a secure GPUI credential modal for DX workflow-node plugins that maps credential input metadata to masked fields and persists secret material only in the platform keychain.

**Architecture:** Keep receipt parsing in the DX Agent bridge, render the modal from parsed credential metadata, and send Agent only a non-secret local save summary after save. The modal never writes raw credentials or keychain item names to settings, receipts, prompts, logs, or source-rendered labels.

**Tech Stack:** Rust GPUI/Zed `agent_ui`, `ui_input::InputField`, platform keychain via `cx.write_credentials`, focused Node source guards, rustfmt, git diff hygiene.

---

### Task 1: Credential Input Metadata

**Files:**
- Create: `crates/agent_ui/src/dx_agent_bridge/workflow_nodes/credentials.rs`
- Modify: `crates/agent_ui/src/dx_agent_bridge/workflow_nodes/contract.rs`
- Modify: `crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs`
- Test: `script/dx-plugin-credentials-source.test.ts`

- [x] **Step 1: Write the source guard first**

Create a focused source guard that requires a credential input summary type, parsing for nested credential `inputs`/`fields`, and secure defaults.

- [x] **Step 2: Parse credential inputs**

Add `DxWorkflowNodeCredentialInputSummary` with id, label, kind, required, secret, placeholder, and parse nested credential input rows. If a credential has no declared inputs, synthesize one required secret input from the credential id/type.

### Task 2: Secure GPUI Credential Modal

**Files:**
- Create: `crates/agent_ui/src/dx_plugin_credentials.rs`
- Modify: `crates/agent_ui/src/agent_ui.rs`
- Modify: `crates/agent_ui/src/agent_panel.rs`
- Modify: `crates/agent_ui/src/dx_launch_workspace/tools_screen/workflow_nodes.rs`
- Test: `script/dx-plugin-credentials-source.test.ts`

- [x] **Step 1: Guard secure modal behavior**

The source guard must require masked `InputField` controls, bounded input reads, `cx.write_credentials`, stable keychain handle construction, and must forbid settings/receipt/log writes from the modal.

- [x] **Step 2: Implement modal model**

Create a `DxPluginCredentialModal` that builds one masked input per parsed credential input, validates required values and byte/character limits, rejects untrusted plugin sources, and writes each non-empty field to the platform keychain.

- [x] **Step 3: Wire Configure to the modal**

The Plugins Configure action opens the modal for credential-backed plugins and keeps the existing Agent draft path for no-credential plugin contract review.

- [x] **Step 4: Send only a non-secret save summary to Agent**

After successful keychain writes, clear the fields, dismiss the modal, and draft an Agent message containing plugin id, source metadata, saved field count, and a non-secret local save reference only.

### Task 3: Docs And Verification

**Files:**
- Modify: `todo.txt`
- Modify: `changelog.txt`
- Modify: `DX.md`

- [x] **Step 1: Update status docs**

Record that credential capture is source-verified, keychain-only, and not yet runtime-proven.

- [x] **Step 2: Run light verification**

Run focused source guards, touched-file `rustfmt --check`, forbidden-source/raw-secret scans, conflict-marker scan, and `git diff --check`. Keep Cargo, `just run`, native launch, and local servers deferred.
