# Zed Animation Project - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### AI Composer and Editor Effects - 2026-03-31

#### Added
- Added a redesigned real Agent composer shell with a top quick-action strip for changes, files, browser context, and additional tools.
- Added a responsive media-mode switcher with native icon assets for text, audio, video, 3D, live, AR, VR, PDF, and chart workflows, including overflow handling for narrow layouts.
- Added a new `editor::typing_effects` module that cycles through particles, fireworks, flames, and magic-style bursts as the user types.

#### Changed
- Updated the real Agent panel input experience to feel closer to the latest Copilot/Codex-style composer surface while keeping Zed's existing mode, model, profile, and send controls functional.
- Updated prompt resolution so non-text media modes prepend delivery guidance to the actual content blocks sent to the agent, making the switcher behavior functional instead of cosmetic.
- Updated the real editor cursor renderer to draw an animated rainbow caret for the newest local cursor.

### AI Provider Hub and Model Picker - 2026-03-31

#### Added
- Added a new `language_models::provider_hub` runtime that merges `models.dev`, OpenRouter, and optional LiteLLM `/models` metadata into one provider catalog.
- Added manifest-backed dynamic provider registration so synced OpenAI-compatible providers can appear in the real Zed AI settings and model picker without hand-written Rust clients.
- Added provider-hub disk caching under Zed's data directory so the editor can reuse the last synced provider catalog offline while background refresh runs.
- Added curated quick-add provider presets in the real AI settings modal for OpenAI-compatible, OpenRouter, GitHub Models, Groq, Together AI, Fireworks AI, Perplexity, DeepInfra, Hugging Face, Baseten, Replicate, LiteLLM Proxy, vLLM, Llamafile, and Text Generation WebUI.
- Added async live model discovery in the Add Provider modal by querying compatible `/models` endpoints and replacing the manual model list with discovered models.

#### Changed
- Updated the real AI settings panel to sort providers by synced category, display category/model-count badges per provider, expose provider-catalog sync status, and include a manual Refresh Catalog action.
- Updated the right-side model picker to react to provider-hub changes and show synced context-window plus capability badges inline for models.
- Updated the right-side AI model picker popover so provider sections are collapsible and show a per-provider model-count badge next to the disclosure control.
- Cleaned up the provider settings render path and model-picker test scaffolding during a file-only preflight pass to reduce first-run compile risk before runtime validation.

### Rich File Preview and Embedded Web Panel - 2026-03-31

#### Added
- Added the new `crates/rich_file_preview/` workspace crate and wired it into the root workspace plus the `zed` application crate.
- Added an extension-based project item registry for video, 3D model, LaTeX, PDF, DOCX, spreadsheet, presentation, markdown, SVG, and audio files.
- Added a native `UniversalPreviewView` item that asynchronously loads and renders document, media, and model previews inside normal workspace tabs.
- Added a dockable EmbeddedWebPreviewPanel built on wry with devtools, DOM element picking, and Agent-panel copy-to-AI handoff.
- Added browser-profile discovery for Chromium- and Firefox-family browsers so the embedded preview can detect real installed extensions from the user's local browser profiles.
- Added isolated embedded-web session policies for shared, per-origin, and incognito browsing, with localhost auth clearing to avoid sticky dev auth collisions.
- Added a stronger embedded inspector flow with live hover capture, click-to-send element capture for the Agent panel, and CSS override application directly back into the previewed page.

#### Changed
- Updated Zed startup initialization so the new preview registry and embedded web panel are registered alongside the existing image, markdown, CSV, and SVG preview systems.
- Updated the rich preview crate dependency set to pin the requested document, media, rendering, and webview libraries in one workspace module.


### Phase 1 - Project Setup & Infrastructure - 2026-03-31

#### Added
- Created the initial `crates/animation_demo/` scaffold with `src/`, `src/utils/`, and `shaders/` directories for the GPUI animation crate.
- Added the new `animation_demo` workspace crate and wired it into the root workspace plus the `zed` application crate.
- Added `crates/animation_demo/src/lib.rs` and an item-backed `demo_tab.rs` surface for Phase 1 animation validation.
- Added a live dimension panel that reports the current viewport size, target FPS, frame budget, and preview surface intent.
- Added placeholder video and 3D preview panels inside the demo tab so later rendering work has a visible target area from the start.
- Verified that the web reference source is available under `www/` for side-by-side implementation work.

#### Changed
- Updated Zed startup initialization to open the new animation demo tab automatically for each workspace during Phase 1 development.
- Updated the `just run` workflow to build both `zed` and `cli` before launch so local development no longer trips the missing `zed-cli` runtime error.

### Phase 2 - Core Animation Components - 2026-03-31

#### Added
- Added `spring.rs` with Apple-style presets, analytical spring solving, and mid-flight retargeting support.
- Added `easing.rs` with reusable Apple-inspired cubic-bezier curves and a bezier solver.
- Added `animator.rs` with `AnimatedValue`, `AnimatedPoint`, `AnimatedColor`, and RGBA helpers for scalar, point, and color animation state.
- Added `transition.rs` with reusable Apple-style helpers for fades, sheet presentation, slide-up motion, and sidebar width changes.
- Added `gesture.rs` with drag-axis modeling, elastic constraints, drag velocity tracking, and hold detection primitives for later interactive work.

#### Changed
- Expanded the animation demo tab to surface Phase 2 motion-toolkit diagnostics so spring, easing, and gesture behavior are visible from the demo environment.

### Phase 3-4 - Friday Border + Hello Glow Previews - 2026-03-31

#### Added
- Added `friday.rs` with a cycling Friday border preview that includes rainbow segmented borders, staged edge reveals, dual glow layers, and a bounce-shifted content area.
- Added `hello_glow.rs` with a GPUI-native glow card preview that cycles a 25-color rainbow field with inner and outer glow layers.
- Added `sidebar.rs`, `carousel.rs`, and `macos_dock.rs` preview components to start expressing the later UI replacement phases inside the demo surface.

#### Changed
- Expanded the demo tab to render live Friday Border, Hello Glow, Sidebar, Carousel, and Dock previews alongside the Phase 2 motion toolkit.

### Phase 8-9 - Drag Preview + Media Direction - 2026-03-31

#### Added
- Added `drag_drop.rs` with a visual drag-and-drop preview showing a ghost tab card plus blue dot-and-line drop indicators.

#### Changed
- Updated the demo tab's video and 3D placeholders to reflect the selected Phase 9 implementation direction: `gstreamer` or `ffmpeg-next` for video, and `wgpu` or `three-d` for 3D rendering.

### Phase 5-8 - Production Sidebar + Dock Integration - 2026-03-31

#### Changed
- Updated the shared `ui::Tab` component to behave more like a dock surface with pill styling, hover emphasis, and a pulsing active indicator.
- Updated the shared `ui::TabBar` layout so pane tabs render inside a centered dock-like chrome while preserving the existing split, add, and zoom controls.
- Updated `workspace::MultiWorkspace` so the real workspace sidebar opens with directional slide-in motion and an elevated container instead of appearing abruptly.
- Updated `workspace::MultiWorkspace` sidebar rendering to animate live width and opacity during open and close transitions instead of snapping visibility.
- Updated the production sidebar renderer to animate in from the active edge, aligning the real workspace chrome with the animation system used in the demo crate.
- Updated the production dragged-tab overlay to use reduced opacity and elevated shadow so reordering feedback matches the dock-like tab system more closely.
- Updated the production tab drop targets to surface blue line-and-dot indicators and stronger selection-colored borders during tab and project-entry drags.

### Phase 5-6 - Production Sidebar Motion + Carousel Gravity - 2026-03-31

#### Changed
- Added real per-project collapse animation state to the production sidebar so workspace groups animate their child rows with height, opacity, and slide transitions instead of snapping open and closed.
- Updated the production sidebar's keyboard navigation and selection recovery so collapsed or animating-hidden rows are skipped correctly during folder and workspace interactions.
- Added subtle hover scaling to project headers so workspace interactions carry visible motion even before a collapse or expand finishes.
- Upgraded the production pane carousel to use a true resizable active screen width with a 400px minimum, live resize feedback, and persisted left/right resize bias.
- Updated the carousel positioning spring to match the web reference target more closely with `stiffness: 300` and `damping: 30`.
- Added left, right, centered, and wrap-around neighbor reveal behavior so resized screens expose adjacent tabs with directional gravity at the beginning and end of the tab strip.

### Project Initialization - 2026-03-31

#### Added
- Created AGENTS.md with comprehensive project instructions for Codex CLI
- Created TODO.md with 10-phase task breakdown and progress tracking
- Created CHANGELOG.md for tracking all project changes
- Documented low-end device constraints (only use `just run`, never cargo commands)

#### Project Structure
- Planned `animation_demo` crate for all custom animations
- Planned `www/` folder for web reference code
- Planned demo tab for live animation testing with dimensions display

#### Documentation
- ANIMATIONS.md: Complete animation specifications (3183 lines)
- DX.md: Developer experience guide with smooth animation patterns (3128 lines)
- AGENTS.md: Codex CLI instructions with coding standards and constraints

---

## Project Roadmap

### Phase 1: Project Setup & Infrastructure (Planned)
**Target:** March 31, 2026  
**Status:** ðŸŸ¡ In Progress

- Create `animation_demo` crate structure
- Set up project dependencies
- Copy web reference code to `www/` folder
- Create demo tab infrastructure
- Configure video/3D rendering preview

### Phase 2: Core Animation Components (Planned)
**Target:** April 1-2, 2026  
**Status:** ðŸ”œ Pending

- Implement spring physics system
- Implement easing curves
- Create animated value system
- Build transition helpers
- Create gesture handling system

### Phase 3: Friday Border Effect (Planned)
**Target:** April 3-4, 2026  
**Status:** â¸ï¸ Not Started

- Rainbow gradient shader implementation
- Border slide-in animation
- Dual glow layers
- Scroll bounce effect

### Phase 4: Hello Glow Effect (Planned)
**Target:** April 5, 2026  
**Status:** â¸ï¸ Not Started

- 25-color HSL gradient shader
- Dual-layer blur system
- Background-position animation

### Phase 5: Sidebar Animations (Planned)
**Target:** April 6-7, 2026  
**Status:** â¸ï¸ Not Started

- Width transition animations
- Content fade animations
- Folder collapse system
- Workspace scroll with spring
- Replace Zed's default sidebar

### Phase 6: Screen Carousel (Planned)
**Target:** April 8-10, 2026  
**Status:** â¸ï¸ Not Started

- Resizable screen system
- Directional gravity logic
- Smooth positioning with springs
- Circular wrapping
- Replace Zed's tab system

### Phase 7: macOS-Style Dock (Planned)
**Target:** April 11-12, 2026  
**Status:** â¸ï¸ Not Started

- Dock icon rendering
- Hover magnification
- Active indicator with glow
- Drag-to-reorder functionality

### Phase 8: Drag-and-Drop System (Planned)
**Target:** April 13-14, 2026  
**Status:** â¸ï¸ Not Started

- Event handling system
- Sortable tabs and folders
- Drop position indicators
- Visual feedback system

### Phase 9: Video & 3D Rendering (Planned)
**Target:** April 15-17, 2026  
**Status:** â¸ï¸ Not Started

- Video player component
- 3D viewer component
- Preview in demo tab
- Performance optimization

### Phase 10: Integration & Polish (Planned)
**Target:** April 18-19, 2026  
**Status:** â¸ï¸ Not Started

- Component integration
- Performance optimization
- Cross-platform testing
- Final polish

---

## Version History

### [0.0.1] - 2026-03-31 (Current)

#### Added
- Initial project setup
- Documentation structure
- Task tracking system
- Codex CLI integration

#### Notes
- Project started on March 31, 2026
- Using Codex CLI for systematic development
- Low-end device constraints documented
- Only `just run` command allowed for testing

---

## Development Guidelines

### Commit Message Format
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `docs`: Documentation changes
- `test`: Test additions or changes
- `chore`: Build process or auxiliary tool changes

**Example:**
```
feat(friday-border): implement rainbow gradient shader

- Added Metal shader for 25-color HSL gradient
- Implemented border slide-in animation (750ms)
- Added dual glow layers (8px + 12px blur)

Closes #1
```

### Testing Notes
- **CRITICAL:** Only use `just run` for testing
- Never use `cargo build`, `cargo test`, `cargo check`, or `cargo clippy`
- Low-end device cannot handle multiple cargo processes
- All tests must be run through `just run` command

### Performance Requirements
- Maintain 120 FPS (8.33ms frame budget)
- GPU-accelerated rendering only
- Minimal CPU usage during animations
- No memory leaks

---

## Statistics

**Total Commits:** 0  
**Total Features Added:** 0  
**Total Bugs Fixed:** 0  
**Total Refactors:** 0  
**Lines of Code:** 0  
**Test Coverage:** 0%

---

## Contributors

- Project Lead: [Your Name]
- AI Assistant: Codex CLI (OpenAI)
- Framework: Zed + GPUI

---

## License

[Your License Here]

---

**Last Updated:** March 31, 2026  
**Next Review:** April 1, 2026



