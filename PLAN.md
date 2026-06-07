# Zed DX Editor Feature Plan

## Summary

This checkpoint defines the next Zed DX editor feature batch. It is a planning-only document: no source implementation starts in this checkpoint commit.

## Task Groups

### 0. Editor Shell And Panel Polish From Latest Brief

- Workspace sidebar:
  - Add a real collapsed activity-bar mode with one icon column, hover tooltips, and an expand/collapse button.
  - Keep expanded behavior available and preserve existing panel/workspace actions.
  - Replace filler top controls with real actions only.
  - Add a bottom add-project/open-folder affordance wired to existing project-opening commands.
  - Add a persisted workspace shortcut/bookmark grid with three columns and up to twelve visible entries before overflow.
- Empty workspace AI:
  - When onboarding is complete and no project/folder is open, show the AI panel as the primary full-screen surface instead of an untitled editor.
  - Keep the sidebar visible enough for opening folders/projects.
  - Keep AI empty states honest when no folders or model/provider data are available.
- AI model picker:
  - Improve only the AI panel picker, not the settings page picker.
  - Group models by provider with collapsible provider rows.
  - Show a model-count badge beside each provider collapse control.
- Top bar and panel buttons:
  - Use product labels `Icons`, `Fonts`, `Media`, `UI`, and `Style`.
  - Keep top-right buttons wired to real right-dock panels.
  - Remove duplicate bottom/status-bar buttons for panels promoted to the top-right tool cluster.
  - Keep active-click behavior consistent with dock panel buttons: clicking the active right-panel tool closes the right dock.
  - Use more appropriate icons for Browser, UI, and Style surfaces.
- Dock panel stacking:
  - Add vertical split/stack controls to compatible left and right panels.
  - Allow up to three visible stacked panels per dock area with half/third-height regions.
  - Preserve current single-panel behavior by default and reuse existing panel stack persistence where possible.
- Project/file panel:
  - Add drag-select marquee selection for files/folders.
  - Show selected-count and file operation actions when multiple entries are selected.
  - Keep existing tree, rename, context menu, reveal, drag/drop, and keyboard behavior intact.
- Tool panels:
  - Icons panel should enlarge icon cells, select without auto-inserting, show explicit install/copy/apply actions, restrict insertion to supported source files, and preserve React JSX component casing.
  - Fonts panel should move slow font metadata into a cache/artifact format before UI rendering.
  - Media panel should support the broader DX media-source plan rather than only local/minimal sources.
  - UI panel should expand beyond the small Shadcn subset through a source-backed component ecosystem catalog.
  - Style panel should stay a real GPUI shell backed by the Web Preview generator surface, not a dummy native mock.
- Web Preview:
  - Use a live spinner loader with the existing UI icon system while native WebView content mounts.

### 1. Project Panel Performance And Media Explorer

- Diagnose why the project panel/file explorer feels laggy, with special attention to expensive visible-tree derivation, large expanded directories, root-drive/system-file scanning, file metadata fanout, and render work done per visible row.
- Keep the existing tree explorer behavior for normal code and document files.
- Add a lazy media presentation for folders that contain images, videos, or audio:
  - Images render as direct visual previews in a grid or rectangle layout.
  - Videos render as a representative preview frame when available.
  - Audio renders as centered rectangle cards showing duration and file name.
- Keep the explorer fast by virtualizing media grids, doing thumbnail and duration extraction off the UI path, caching results, and never blocking tree navigation on media probing.
- Preserve current selection, rename, drag/drop, context menu, reveal, and keyboard behavior unless explicitly changed by the implementation task.

### 2. Web Preview Onboarding

- Replace the current Zed onboarding surface with a local Web Preview onboarding surface.
- For the first version, the onboarding page contains a minimal website with a single `Complete` button.
- Clicking `Complete` dismisses the onboarding and returns the editor to the normal workspace state.
- Keep this page local and lightweight for now so future onboarding work can expand it without requiring network access or a separate app server.
- Preserve existing onboarding completion state semantics where practical, so users do not repeatedly see onboarding after completion.

### 3. Dock And Sidebar Stacking

- Add support for showing two or three panels within one dock area, using half-height or one-third-height stacked regions.
- Begin conservatively with the right dock and AI panel layout path, then generalize only where the existing dock model supports it cleanly.
- Reuse existing dock panel registration, activation, focus, zoom, resize, icon button, tooltip, and persistence patterns.
- Add controls that let a user open compatible panels at the same time without replacing the currently visible panel.
- Keep panel stacking compatible with existing single-panel dock behavior.

### 4. Workspace Activity Bar

- Add a collapsed workspace/sidebar mode that behaves like a VS Code-style activity bar.
- In collapsed mode, show only one icon-width worth of UI and remove detailed text from the main sidebar surface.
- Show removed details through hover tooltips.
- Preserve the expanded workspace/sidebar behavior for users who want the current fuller sidebar.
- Avoid decorative or dummy controls; each icon must map to a real existing panel, workspace action, or navigation target.

### 5. Empty Workspace AI Mode

- After onboarding is complete, when no project is open, show the AI panel in a full-screen default state.
- In that empty-workspace state, include the DX/Codex progress and DCP-oriented rail on the right side.
- Keep the rail connected to real DX/Codex status data where available, and show honest empty or missing states where data is not available.
- Preserve normal editor and project behavior once a project is opened.

### 6. DX Plugins, Workflow Nodes, And Agent Integrations

- Treat the Zed-facing product surface as `Plugins`, backed only by DX-owned plugin packages, DX workflows, and DX Agent tools.
- Build the plugin and workflow model around DX concepts:
  - Workflows are DX-owned node graphs.
  - Plugins expose triggers, actions, transforms, credentials, permissions, execution metadata, and receipts.
  - Visual or multi-step plugins should normalize their capabilities into a DX plugin manifest before Zed or DX Agents can use them.
- Run plugin and workflow execution through `G:\Dx\js` / DXJS where practical, so TypeScript and JavaScript plugins can run through the DX-owned Bun fork instead of a generic Node runtime.
- Keep the first milestone fully DX-native:
  - Do not include external inspiration projects as plugin sources.
  - Do not expose upstream project labels in Zed UI or DX plugin metadata.
  - Zed UI copy should say `Plugins`, `Workflows`, `Nodes`, `Credentials`, and `Receipts`.
- Use only DX-owned source roots and explicitly approved permissive packages when expanding the plugin catalog.
- Build a DX plugin registry bridge before building large UI:
  - Scan DX-owned plugin and node sources.
  - Normalize name, category, description, inputs, outputs, credentials, permissions, runtime, source path, and trust status.
  - Cache the catalog outside render paths.
  - Expose only enabled/trusted plugins to the AI tool layer.
  - Record plugin enablement, execution, errors, and credential status as DX receipts.
- Connect enabled plugins to DX Agents and the Zed AI profiles through a narrow tool bridge:
  - `Ask` can reference read-only plugin data when explicitly used.
  - `Agents` can run approved workflow/tool plugins.
  - `Search`, `Study`, and `Media` should prefer Web Preview-backed plugin UIs where the interaction is visual or workflow-heavy.
- Create these DX-native first-party plugins:
  - `Browser`: agent control of the in-app browser, including navigation, click/type, DOM inspection, screenshots, and page-state summaries.
  - `Computer`: agent control of the user's Chrome/desktop browser path with action recording; show recordings as Agent-screen thumbnails and open them in the Zed Web Preview video player when clicked.
  - `Driven`: DX-native agent workflow plugin inspired by Superpowers, but wired to DX lanes, worker prompts, goals, checkpoints, receipts, source guards, and verification policy.
- Improve the DX Agents integration as part of the plugin work:
  - Surface social/account connections with real status, QR-code login flows where providers support it, credential health, and receipt-backed connection history.
  - Keep connection UI visually polished in GPUI with clear account/provider cards, not raw command output.
  - Avoid fake connected states; missing auth, expired credentials, and unavailable providers must be visible and actionable.
- Preferred implementation order:
  1. Define the DX plugin manifest and source catalog scanner.
  2. Index DX-owned plugin and node sources.
  3. Add the Zed Plugins panel catalog view with trust/enable controls.
  4. Add the DXJS execution adapter with permissions, cancellation, receipts, and error reporting.
  5. Wire enabled plugins into DX Agents as approved AI tools.
  6. Build the `Browser`, `Computer`, and `Driven` first-party plugins.
  7. Add social connection cards and QR login UX for DX Agents integrations.

### 7. DX Automations Foundation

- Treat Automations as a DX Agents receipt-backed surface first, not a native Zed scheduler.
- Automation records should carry typed fields for name, prompt, schedule, status, destination, last run, next run, receipts, and history.
- The composer surface should expose the backend contract and draft fields, but `Save Draft`, `Enable`, and run controls must stay disabled or unavailable until DX Agents emits an explicit composer/runtime receipt.
- Launch rail Automations should remain read-only status/handoff UI.
- Settings/Agent Configuration is the correct home for interactive automation composition because it already owns workspace roots, receipt roots, and public DX Agents action gating.
- Future runtime work should add a typed DX Agents create/update/run contract before enabling scheduled execution.

## Constraints

- Do not run `just run` unless the user explicitly authorizes it.
- Do not run broad Cargo commands or heavy validation unless explicitly authorized.
- Use source inspection and lightweight checks first.
- Preserve existing Zed behavior unless a task explicitly changes it.
- Avoid dummy UI, fake state, and disconnected demo-only controls.
- Keep new surfaces wired to real editor state, real panel registrations, or honest unavailable states.
- Keep changes maintainable, typed, and consistent with existing Zed/GPUI patterns.

## Checkpoint Verification

- `git status --short`
- `git diff -- PLAN.md PLAN_DEPRICATED.md`
- `git diff --check`

## Checkpoint Commit

```bash
git add PLAN.md PLAN_DEPRICATED.md
git commit -m "docs: checkpoint zed editor feature plan"
```

## Assumptions

- `PLAN_DEPRICATED.md` is intentionally spelled this way because the user requested that exact filename.
- This checkpoint is documentation-only.
- No push is included unless the user separately requests it.
