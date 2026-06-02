# AI Screen And Panel Composer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Polish the Zed Agent screen and Agent panel into a production-ready AI workspace shell without adding fake controls or dummy state.

**Architecture:** Keep the real AgentPanel, ThreadView, ACP mode/model/profile selectors, DX launch rails, and DX Style readiness surfaces. Reorganize those existing controls into a Codex-style composer and floating fullscreen rails, and render Plan/Goal/Multitask shortcuts only when the active agent exposes matching real session modes.

**Tech Stack:** Rust, GPUI, Zed Agent UI, ACP session modes/config options, existing DX Style and DX launch status snapshots.

---

### Task 1: Composer Structure

**Files:**
- Modify: `crates/agent_ui/src/conversation_view/thread_view.rs`
- Modify: `crates/agent_ui/src/mode_selector.rs`

- [ ] Add read-only accessors to `ModeSelector` for the current mode and available ACP modes.
- [ ] Reorganize `ThreadView::render_message_editor` into status, editor body, and action rows.
- [ ] Render Plan/Goal/Multitask shortcut chips only when those exact mode names or IDs exist in the active session mode list.
- [ ] Keep the existing Add Context menu, profile selector, mode selector, model selector, token usage, thinking/fast controls, queue/send behavior, and message editor.
- [ ] Add a visual-only mic icon with an honest tooltip saying voice input needs the DX voice runtime before it can record.

### Task 2: Fullscreen AI Workspace

**Files:**
- Modify: `crates/agent_ui/src/agent_panel.rs`
- Modify: `crates/agent_ui/src/dx_launch_workspace.rs`

- [ ] Default fullscreen sources and progress rails to visible.
- [ ] Add a top-center response controller using real thread, source, task, style, and readiness status.
- [ ] Keep left/right rails floating over the fullscreen Agent screen and preserve toolbar toggles for hiding/showing them.
- [ ] Do not add fake source rows, fake progress, or placeholder agent state.

### Task 3: Style Cockpit Polish

**Files:**
- Modify: `crates/agent_ui/src/dx_style_panel/panel_view.rs`
- Modify: `crates/agent_ui/src/dx_launch_workspace/style_panel.rs`

- [ ] Name the Style native panel sections around generator host, active target, readiness, and source contracts.
- [ ] Keep the Web Preview generator button wired to existing generator actions.
- [ ] Keep all rows derived from `DxStylePanelSnapshot`; do not invent demo rows.

### Task 4: Verification And Handoff

**Files:**
- Modify: `todo.txt`
- Modify: `changelog.txt`

- [ ] Run `rustfmt --edition 2024 --check` on touched Rust files.
- [ ] Run `git diff --check`.
- [ ] Run focused source scans for no conflict markers and no `DX.txt` staging.
- [ ] Run the authorized `just run` after source checks.
- [ ] Commit and push only this lane plus the checkpoint commit; leave unrelated scratch work unstaged.
