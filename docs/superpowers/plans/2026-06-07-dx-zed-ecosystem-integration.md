# DX Code Ecosystem Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate the DX ecosystem into the Zed code editor as a production-grade DX cockpit covering DX WWW, DX Style, DX Icon, DX extensions/defaults, Search, Media, Forge, Flow readiness, and evidence-first AI surfaces.

**Architecture:** Zed owns editor UX, docking, native Web Preview hosting, source guards, and safe defaults. DX crates remain the source of truth for framework contracts, icon data, style generation, media/search/forge receipts, and Flow runtime readiness. Cross-system behavior must move through typed adapters, bounded receipts, source guards, and explicit command boundaries instead of render-path filesystem scans or dummy UI.

**Tech Stack:** Rust, GPUI, Zed extensions, DX CLI contracts, DX WWW receipts/devtools, DX Style contracts, DX Icon data/indexes, Node source guards, lightweight git verification.

---

## Goal Mode Rules

- Use exactly 6 GPT-5.5 extra-high subagents when executing a major implementation pass from this plan.
- Keep work production-ready, professional, maintainable, and split by ownership.
- Do not use placeholder names such as `v1`, `new_ui`, `final_working`, `temp`, `ai_fix`, or broad `utils` modules.
- Do not ship dummy UI, fake state, fake wiring, disconnected controls, or decorative panels.
- Do not run `just run`, Cargo build/check/test/clippy, servers, browser automation, or live WebView proof unless explicitly authorized.
- Use source inspection, targeted source guards, `rustfmt --edition 2024 --check <touched Rust files>`, `git diff --check`, and conflict-marker scans as the default verification path.
- Update `todo.txt` and `changelog.txt` for implementation changes.
- Make focused professional commits after meaningful checkpoints.
- Push/sync only after a clean checkpoint and only when remote state is understood.

## Six Planning Lanes Used

- DX WWW and Web Preview integration: Schrodinger.
- DX Style Web Preview and source-apply contracts: Avicenna.
- DX Icon provider and icon themes: Herschel.
- DX extension/default policy: Euler.
- Search, Media, Forge, and Flow readiness: Hilbert.
- Execution governance and commit discipline: Epicurus.

## Ownership Map

- Workspace, docks, screen routing: `G:\Dx\code\crates\workspace`.
- Agent panel, AI screen, DX rails, voice/composer, Forge/Check/Style shells: `G:\Dx\code\crates\agent_ui`.
- Web Preview, DX Studio bridge, native WebView lifecycle: `G:\Dx\code\crates\web_preview`.
- Title bar, screen dock, top-right tools: `G:\Dx\code\crates\title_bar`.
- Shared actions: `G:\Dx\code\crates\zed_actions`.
- File/icon themes: `G:\Dx\code\crates\file_icons`, `G:\Dx\code\crates\theme`, `G:\Dx\code\extensions`.
- Source guards: `G:\Dx\code\script`.
- DX WWW source contracts: `G:\Dx\www`.
- DX Style source contracts: `G:\Dx\style`.
- DX Icon source contracts: `G:\Dx\icon`.
- DX ecosystem services: `G:\Dx\media`, `G:\Dx\metasearch`, `G:\Dx\forge`, `G:\Dx\flow`.
- Extension packaging source: `G:\Dx\extensions`.

---

## Phase 1: DX Project Detector And Shared Integration Spine

**Goal:** Add a small shared detection spine so Zed can understand DX projects without each panel re-scanning differently.

**Files:**
- Create: `G:\Dx\code\crates\agent_ui\src\dx_project_context.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\agent_ui.rs`
- Modify: `G:\Dx\code\crates\web_preview\src\dx_studio\project.rs`
- Modify: `G:\Dx\code\crates\web_preview\src\dx_studio_session.rs`
- Add guard: `G:\Dx\code\script\dx-project-context-source.test.ts`

- [ ] Define `DxProjectContext` with bounded fields: workspace root, DX config path, `.dx` root, diagnostics receipt path, check receipt path, forge receipt root, WWW route manifest path, and style receipt root.
- [ ] Make detection cacheable and root-scoped. Do not scan from render functions.
- [ ] Prefer project-local `.dx` receipts over shared `G:\Dx\.dx` receipts.
- [ ] Add a source guard that rejects direct render-path receipt scans in Agent fullscreen, Web Preview status, Style panel, and Forge/Check summaries.
- [ ] Verify with `node --test script\dx-project-context-source.test.ts`.
- [ ] Verify touched Rust with `rustfmt --edition 2024 --check <touched Rust files>`.
- [ ] Commit: `feat(dx): add shared project context detection`.

---

## Phase 2: DX WWW Syntax, Project Preview, Devtools, And Diagnostics

**Goal:** Make Zed recognize DX WWW projects and DX files, open the correct preview route, attach to DX hot reload/devtools, and surface diagnostics from real DX receipts.

**Files:**
- Modify: `G:\Dx\code\crates\web_preview\src\dx_studio.rs`
- Modify: `G:\Dx\code\crates\web_preview\src\dx_studio\project.rs`
- Modify: `G:\Dx\code\crates\web_preview\src\dx_studio\routes.rs`
- Modify: `G:\Dx\code\crates\web_preview\src\web_preview_view.rs`
- Modify: `G:\Dx\code\crates\zed_actions\src\lib.rs`
- Create: `G:\Dx\code\crates\web_preview\src\dx_www_status\mod.rs`
- Create: `G:\Dx\code\crates\agent_ui\src\dx_www_diagnostics.rs`
- Create: `G:\Dx\code\extensions\dx-www\extension.toml`
- Create: `G:\Dx\code\extensions\dx-www\languages\dx-www\config.toml`
- Create: `G:\Dx\code\extensions\dx-www\languages\dx-www\highlights.scm`
- Source reference: `G:\Dx\www\vscode-syntax\dx-www.tmLanguage.json`
- Add guard: `G:\Dx\code\script\dx-www-zed-integration-source.test.ts`

- [ ] Add bounded project detection for `dx`, `.dx`, `.dx\receipts\check\check-latest.json`, `.dx\diagnostics\latest.json`, `.dx\forge`, app folders, and DX WWW route manifests.
- [ ] Add actions: `dx_www::OpenProjectPreview`, `OpenProjectPreviewToTheSide`, `RefreshStatus`, and `OpenFeedbackDashboard`.
- [ ] Add file association for extensionless `dx`, `*.dx`, and source-owned DX WWW file patterns.
- [ ] Keep syntax support honest: use current Zed grammar mechanics and do not claim full TextMate parity until parser-backed highlighting exists.
- [ ] Read `.dx\diagnostics\latest.json` with size bounds and map severity/file/range/code-frame/next-action into Zed diagnostics.
- [ ] Attach Web Preview status to `/_dx/hot-reload/version`, `/_dx/hot-reload/events`, and `/_dx/feedback/*` when a DX WWW dev server is available.
- [ ] Keep native WebView2 DevTools separate from DX WWW framework devtools.
- [ ] Add source guards for bounded JSON reads, no hardcoded port assumption, and no fake LSP source.
- [ ] Verify with `node --test script\dx-www-zed-integration-source.test.ts script\dx-studio-project-source.test.ts script\dx-check-panel-source.test.ts script\dx-www-launch-evidence-source.test.ts`.
- [ ] Commit: `feat(dx-www): detect and preview DX WWW projects`.
- [ ] Commit: `feat(dx-www): surface diagnostics and devtools status`.

---

## Phase 3: DX Style Web Preview Cockpit

**Goal:** Re-enable the native Style panel as a GPUI shell for the Web Preview-backed DX Style generator, with active cursor context, grouped-class receipts, reverse CSS review, and review-only source-apply receipts.

**Files:**
- Modify: `G:\Dx\code\crates\agent_ui\src\agent_ui.rs`
- Modify: `G:\Dx\code\crates\title_bar\src\title_bar.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_style_panel\panel.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_style_panel\panel_view.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_style_panel\panel_cards.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_style_panel\active_context.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_style_panel\cursor_context.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_style_panel\group_context.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_style_panel\group_registry.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_style_panel\reverse_css_map.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_style_panel\apply_gate.rs`
- Modify: `G:\Dx\code\crates\web_preview\src\web_preview_view.rs`
- Modify: `G:\Dx\code\crates\web_preview\src\dx_style_generator_surface\*.rs`
- Source references: `G:\Dx\style\src\core\engine\visual_generator_*.rs`, `G:\Dx\style\src\core\engine\grouped_class_*.rs`, `G:\Dx\style\fixtures\*.json`

- [ ] Re-enable `dx_style_panel::panel::init(cx)` and restore the Style titlebar entry.
- [ ] Update source guards that currently expect the panel to be parked.
- [ ] Make the panel primary action context-aware through `OpenGeneratorPreviewForContext`.
- [ ] Preserve boundaries: static classes can be reviewed, dynamic classes stay read-only, CSS declarations get hints, source mutation remains disabled.
- [ ] Resolve grouped class expansion from project-local grouped-class registry receipts when available.
- [ ] Keep reverse CSS mapping review-only until runtime mutation proof exists.
- [ ] Sync generator catalogs from `G:\Dx\style` and guard fixture drift.
- [ ] Expose `can_review_request=true` and `can_mutate_source=false` in Web Preview source-apply contracts.
- [ ] Verify with `node --test script\dx-style-panel-source.test.ts script\dx-title-bar-source.test.ts script\web-preview-payload-source.test.ts script\web-preview-platform-lifecycle.test.ts`.
- [ ] Verify fixture drift with `node G:\Dx\style\scripts\sync_zed_visual_generator_fixtures.mjs --check`.
- [ ] Commit: `feat(dx-style): enable Web Preview style cockpit`.
- [ ] Commit: `feat(dx-style): pass active editor context to generators`.
- [ ] Commit later only after proof: `feat(dx-style): enable guarded source mutation`.

---

## Phase 4: DX Icon Provider, Icon Picker, Chat Icons, And File Icon Theme

**Goal:** Stop one-off icon substitution and make DX Icon a first-class provider for Zed UI icons, icon picker search, chat icon customization, dragged icon assets, and a curated file/folder icon theme.

**Files:**
- Create: `G:\Dx\code\crates\dx_icon_provider\Cargo.toml`
- Create: `G:\Dx\code\crates\dx_icon_provider\src\lib.rs`
- Create: `G:\Dx\code\crates\dx_icon_provider\src\provider.rs`
- Create: `G:\Dx\code\crates\dx_icon_provider\src\cache.rs`
- Create: `G:\Dx\code\crates\dx_icon_provider\src\env.rs`
- Modify: `G:\Dx\code\Cargo.toml`
- Modify: `G:\Dx\code\crates\icon_picker\src\icon_picker.rs`
- Modify: `G:\Dx\code\crates\icon_picker\Cargo.toml`
- Modify: `G:\Dx\code\crates\editor\src\items.rs`
- Modify: `G:\Dx\code\crates\workspace\src\pane.rs`
- Create: `G:\Dx\code\crates\agent_ui\src\dx_icon_picker.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\conversation_view\thread_view.rs`
- Modify: `G:\Dx\code\crates\title_bar\src\title_bar.rs`
- Modify: `G:\Dx\code\crates\workspace\src\dock.rs`
- Create: `G:\Dx\code\extensions\dx-icon-theme\extension.toml`
- Create: `G:\Dx\code\extensions\dx-icon-theme\icon_themes\dx-icons.json`
- Create curated SVG assets under: `G:\Dx\code\extensions\dx-icon-theme\icons`

- [ ] Resolve icon paths in this order: `DX_ICON_INDEX`, `DX_ICON_DATA`, `DX_ICON_ROOT`, legacy `DX_ICONS_DATA_DIR`, `DX_HOME\icon`, `G:\Dx\icon`, `%USERPROFILE%\.dx\icon`.
- [ ] Expose typed APIs: `search(query, limit)`, `svg(pack, name)`, `preview_path(pack, name)`, and `theme_icon_path(...)`.
- [ ] Keep index and disk cache loading lazy and off the UI thread.
- [ ] Cache by `pack:name` with sanitized paths and atomic writes.
- [ ] Replace hardcoded `G:/Assets/icon/data` resolution with `DxIconProvider`.
- [ ] Preserve existing built-in icon results, drag/drop behavior, editor insertion, pinned icons, and recent icons.
- [ ] Persist chat icon choices as stable `{ pack, name }` plus optional cache path, not raw unbounded SVG.
- [ ] Add a curated DX icon theme from safe selected assets rather than packaging the full icon dataset.
- [ ] Verify with source guards that no render path scans icon JSON packs and no old hardcoded icon paths remain.
- [ ] Commit: `feat(dx-icon): add lazy DX icon provider`.
- [ ] Commit: `feat(dx-icon): wire provider into icon picker and dragged assets`.
- [ ] Commit: `feat(dx-icon): add chat icon selection`.
- [ ] Commit: `feat(dx-icon): add curated DX file icon theme`.

---

## Phase 5: DX Extensions, Theme Defaults, And Safe Policy

**Goal:** Keep built-in defaults inside the Zed fork, keep `G:\Dx\extensions` as the source for installable DX host extensions, and avoid auto-installing unproven extensions or broad capabilities.

**Files:**
- Modify: `G:\Dx\code\assets\settings\default.json`
- Modify: `G:\Dx\code\assets\settings\initial_user_settings.json`
- Modify: `G:\Dx\code\assets\themes\dx\dx.json`
- Modify: `G:\Dx\code\crates\grammars\src\dx_serializer\config.toml`
- Modify: `G:\Dx\code\crates\languages\src\lib.rs`
- Modify: `G:\Dx\code\crates\theme\src\icon_theme.rs`
- Modify: `G:\Dx\code\crates\theme\src\registry.rs`
- Optional mirror paths: `G:\Dx\extensions\hosts\zed\dx-language`, `G:\Dx\extensions\hosts\zed\dx-icons`, `G:\Dx\extensions\hosts\zed\dx-theme`
- Existing extension source: `G:\Dx\extensions\hosts\zed\dx-zed`

- [ ] Keep `Dx Light` and `Dx Dark` as bundled theme defaults.
- [ ] Add a DX icon theme only after it is built-in or packaged and loadable.
- [ ] Keep DX language default owned by Zed unless public/gallery distribution needs a mirrored extension.
- [ ] Keep `context_servers` and `agent_servers` empty by default until local-service proof exists.
- [ ] Keep `session.trust_all_worktrees=false`.
- [ ] Keep `agent.tool_permissions.default="confirm"`.
- [ ] Keep `auto_install_extensions` minimal; use a recommendation surface instead of silent install.
- [ ] Keep granted extension capabilities scoped and reject wildcard process/download permissions.
- [ ] Add source guards for DX language, theme, icon theme, extension policy, and capability defaults.
- [ ] Commit: `chore(dx): guard DX language and theme defaults`.
- [ ] Commit: `chore(dx): lock safe DX extension defaults`.
- [ ] Commit later: `chore(dx): track DX Code command center readiness`.

---

## Phase 6: AI Search, Media, Forge, Flow Readiness, And Evidence Basket

**Goal:** Connect existing DX ecosystem anchors into an evidence-first AI UX without direct-linking every DX crate into Zed or reintroducing render-path lag.

**Files:**
- Modify: `G:\Dx\code\assets\settings\default.json`
- Modify: `G:\Dx\code\crates\agent_settings\src\agent_profile.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\conversation_view\composer_profile_options.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\conversation_view\thread_view.rs`
- Modify: `G:\Dx\code\crates\agent\src\thread.rs`
- Create: `G:\Dx\code\crates\agent_ui\src\dx_evidence_basket.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_launch_workspace.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_launch_workspace\sources\attachments.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_launch_workspace\sources\rows.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_launch_prompts\source.rs`
- Modify: `G:\Dx\code\crates\media_panel\src\dx_media_bridge.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_forge_panel\snapshot.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_forge_panel\package_status.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\dx_receipt_history\forge_history.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\flow_speech_runtime.rs`
- Modify: `G:\Dx\code\crates\agent_ui\src\conversation_view\voice_controls.rs`

- [ ] Keep Search profile id as `search`, optionally displayed as `AI Search`.
- [ ] Enable existing metasearch and source-pack tools for Search: `inspect_dx_metasearch`, `search_dx_metasearch`, `extract_dx_metasearch_source`, `prepare_dx_source_attachment`, and `prepare_dx_metasearch_context`.
- [ ] Keep Search backend boundary through existing tools, HTTP contracts, and receipts instead of direct crate links.
- [ ] Add a `DxEvidenceBasket` projection over source set snapshots, receipt history, tool history, media outputs, Forge restore previews, and Flow readiness.
- [ ] Do not scan filesystems from render paths.
- [ ] Surface media provenance using hashes, labels, source paths, model/generator labels, seed, validation state, and license status. Do not expose raw prompts by default.
- [ ] Treat Forge as read-only evidence first: package status, restore previews, restore receipts, cache and media availability.
- [ ] Add only Flow readiness evidence now: Flow root, dictate binary, STT model readiness, Kokoro readiness, and cheap cached device availability if already present.
- [ ] Defer live capture/transcription proof to a later authorized validation window.
- [ ] Verify with `node --test script\dx-source-sets-source.test.ts script\dx-launch-workspace-source.test.ts script\dx-launch-prompts-source.test.ts script\dx-media-panel-source.test.ts script\dx-forge-panel-source.test.ts script\dx-agent-voice-controls-source.test.ts`.
- [ ] Add guards: `script\dx-evidence-basket-source.test.ts` and `script\dx-ai-search-profile-source.test.ts`.
- [ ] Commit: `feat(dx-ai): enable Search profile evidence tools`.
- [ ] Commit: `feat(dx-ai): add DX evidence basket projection`.
- [ ] Commit: `feat(dx-media): surface media provenance evidence`.
- [ ] Commit: `feat(dx-forge): extend Forge receipt evidence`.
- [ ] Commit: `feat(dx-flow): add Flow readiness evidence`.

---

## Phase 7: Governance, Verification, And Handoff

**Goal:** Keep the large integration safe to continue across many agents and days.

**Files:**
- Modify each pass: `G:\Dx\code\todo.txt`
- Modify each pass: `G:\Dx\code\changelog.txt`
- Modify when source guard registry changes: `G:\Dx\code\DX.md`
- Modify when plan scope changes: `G:\Dx\code\PLAN.md`
- Plan artifact: `G:\Dx\code\docs\superpowers\plans\2026-06-07-dx-zed-ecosystem-integration.md`

- [ ] Before each implementation lane, inspect `git status --short --branch`.
- [ ] Read `AGENTS.md`, `DX.md`, `PLAN.md`, `todo.txt`, and `changelog.txt`.
- [ ] Keep one owned surface per commit when possible.
- [ ] Add or update narrow source guards before or alongside implementation.
- [ ] Run targeted source guards for the touched surface.
- [ ] Run `rustfmt --edition 2024 --check <touched Rust files>` for touched Rust.
- [ ] Run `git diff --check -- <touched files>`.
- [ ] Run `rg -n "^(<<<<<<<|=======|>>>>>>>)" <touched files>`.
- [ ] Update `todo.txt` and `changelog.txt`.
- [ ] Commit with a focused message.
- [ ] Push/sync after a clean checkpoint when requested or clearly expected.

## Recommended Execution Order

1. DX project detector and shared context.
2. DX WWW project preview and file association.
3. DX WWW diagnostics and devtools status.
4. DX Style panel re-enable and Web Preview context.
5. DX Icon provider foundation.
6. Icon picker and chat icon integration.
7. Curated DX file icon theme.
8. Safe defaults and extension policy guards.
9. AI Search profile and evidence basket.
10. Media, Forge, and Flow readiness evidence.

## Runtime Validation Window

Only when the user explicitly authorizes runtime validation:

- Run targeted source guards first.
- Run focused `rustfmt --edition 2024 --check` for touched Rust files.
- Run `git diff --check`.
- Then run `just run` with the requested job count.
- Validate project detection, Web Preview, Style panel, icon picker, file icon theme, AI Search profile, evidence basket, and normal editor navigation manually.

