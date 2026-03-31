# Zed Animation Project - Task Tracker

**Project Start Date:** March 31, 2026  
**Last Updated:** March 31, 2026  
**Status:** ðŸŸ¡ In Progress

---

## Project Phases

### Phase 1: Project Setup & Infrastructure â³ IN PROGRESS
- [x] Create `animation_demo` crate structure
- [x] Set up project dependencies in `Cargo.toml`
- [x] Copy web reference code to `www/` folder
- [x] Create demo tab infrastructure (like onboarding tab)
- [x] Set up dimension display panel for live testing
- [x] Configure video/3D rendering preview in demo tab
- [ ] Verify `just run` command works correctly

**Estimated Time:** 2-3 hours  
**Priority:** ðŸ”´ Critical  
**Dependencies:** None

---

### Phase 2: Core Animation Components ðŸ”œ PENDING
- [x] Implement spring physics system (`spring.rs`)
- [x] Implement easing curves (`easing.rs`)
- [x] Create animated value system (`animator.rs`)
- [x] Build transition helpers (`transition.rs`)
- [x] Create gesture handling system (`gesture.rs`)
- [x] Test all animation primitives in demo tab

**Estimated Time:** 4-5 hours  
**Priority:** ðŸ”´ Critical  
**Dependencies:** Phase 1

---

### Phase 3: Friday Border Effect â¸ï¸ NOT STARTED
- [x] Create `friday.rs` component
- [ ] Implement rainbow gradient shader (Metal/Vulkan)
- [x] Add border slide-in animation (750ms)
- [x] Implement dual glow layers (8px + 12px blur)
- [x] Add scroll bounce effect with spring physics
- [ ] Test gradient shift animation (0% â†’ 200%)
- [ ] Verify 120 FPS performance
- [x] Add to demo tab with dimensions display

**Estimated Time:** 3-4 hours  
**Priority:** ðŸ”´ Critical  
**Dependencies:** Phase 2

---

### Phase 4: Hello Glow Effect â¸ï¸ NOT STARTED
- [x] Create `hello_glow.rs` component
- [ ] Implement 25-color HSL gradient shader
- [x] Add dual-layer blur (6px + 18px)
- [x] Implement background-position animation (6s cycle)
- [x] Add border radius rendering (12px)
- [ ] Test glow intensity and performance
- [x] Add to demo tab with live preview

**Estimated Time:** 2-3 hours  
**Priority:** ðŸ”´ Critical  
**Dependencies:** Phase 2

---

### Phase 5: Sidebar Animations ðŸš§ IN PROGRESS
- [x] Create `sidebar.rs` component
- [x] Implement width transition (56px â†” 360px, 200ms)
- [x] Add content fade animation (opacity 0 â†’ 1, 150ms)
- [x] Implement folder collapse (height auto â†’ 0, 200ms)
- [ ] Add workspace scroll with spring animation
- [ ] Implement media player slide-up (y: 20 â†’ 0)
- [ ] Add hover effects (scale 1.0 â†’ 1.25)
- [ ] Implement drag constraints with elastic (0.1-0.2)
- [ ] Test smooth scroll with continuous hold detection
- [x] Replace Zed's default sidebar in `workspace/src/sidebar.rs`

**Estimated Time:** 5-6 hours  
**Priority:** ðŸ”´ Critical  
**Dependencies:** Phase 2

---

### Phase 6: Screen Carousel ðŸš§ IN PROGRESS
- [x] Create `carousel.rs` component
- [x] Implement resizable screen system
- [x] Add directional gravity logic:
  - [x] Left resize â†’ push screen right
  - [x] Right resize â†’ push screen left
  - [x] Both sides â†’ center
- [x] Implement smooth positioning (spring: stiffness 300, damping 30)
- [x] Add live resize with wrapper width updates
- [x] Implement circular wrapping (first/last screens)
- [ ] Add screen transitions (scale 0.95 â†’ 1, opacity 0 â†’ 1)
- [ ] Test with multiple screens
- [ ] Replace Zed's tab system in `editor/src/tabs.rs`

**Estimated Time:** 6-7 hours  
**Priority:** ðŸ”´ Critical  
**Dependencies:** Phase 2

---

### Phase 7: macOS-Style Dock â¸ï¸ NOT STARTED
- [x] Create `macos_dock.rs` component
- [x] Implement dock icon rendering
- [ ] Add hover magnification (scale 1.0 â†’ 1.3 with spring)
- [x] Implement active indicator with pulsing glow
- [x] Add click-to-switch functionality
- [x] Implement drag-to-reorder with visual feedback
- [x] Add plus button for new screen
- [ ] Add close button (X) with animation
- [x] Connect dock to carousel state
- [x] Integrate into workspace top bar

**Estimated Time:** 4-5 hours  
**Priority:** ðŸ”´ Critical  
**Dependencies:** Phase 6

---

### Phase 8: Drag-and-Drop System â¸ï¸ NOT STARTED
- [x] Create drag-and-drop event handling system
- [x] Implement sortable tabs
- [ ] Implement sortable folders
- [ ] Add drop position indicators (before/after/inside)
- [x] Create drag overlay with shadow
- [ ] Implement logo container as drop target
- [ ] Add visual feedback:
  - [x] Dragging: 50% opacity, cursor-grabbing
  - [x] Drop indicators: blue dots (8px) + lines (2px)
  - [x] Hover states with background transitions
- [ ] Test workspace organization

**Estimated Time:** 5-6 hours  
**Priority:** ðŸŸ¡ Medium  
**Dependencies:** Phase 5

---

### Phase 9: Video & 3D Rendering â¸ï¸ NOT STARTED
- [x] Research and select video player crate (e.g., `gstreamer`, `ffmpeg-next`)
- [x] Research and select 3D rendering crate (e.g., `wgpu`, `three-d`)
- [ ] Implement video player component
- [ ] Implement 3D viewer component
- [ ] Add video/3D preview to demo tab
- [ ] Test video playback performance
- [ ] Test 3D rendering performance
- [ ] Ensure 120 FPS for UI while video/3D plays
- [ ] Add controls (play/pause, seek, rotate)

**Estimated Time:** 8-10 hours  
**Priority:** ðŸŸ¡ Medium  
**Dependencies:** Phase 1

---

### Phase 10: Integration & Polish â¸ï¸ NOT STARTED
- [ ] Integrate all components into Zed workspace
- [ ] Test all animations together
- [ ] Performance optimization pass
- [ ] Fix any visual glitches
- [ ] Cross-platform testing (macOS, Linux, Windows)
- [ ] Documentation updates
- [ ] Final demo tab polish

**Estimated Time:** 4-5 hours  
**Priority:** ðŸŸ¢ Low  
**Dependencies:** All previous phases

---

## Additional Preview Work

### Rich File Preview and WebView
- [x] Create `crates/rich_file_preview/` workspace crate
- [x] Register extension-driven `ProjectItem` handlers for media, 3D, document, markdown, SVG, and audio previews
- [x] Add native `gpui::View` preview surface for PDF, DOCX, spreadsheet, presentation, markdown, SVG, audio, video, LaTeX, and 3D assets
- [x] Add dockable embedded web preview panel with Wry devtools actions
- [x] Add DOM element capture flow that forwards HTML/CSS context into the Agent panel composer
- [x] Auto-detect local browser profiles and real installed extensions for the embedded web preview
- [x] Add isolated shared/per-origin/incognito web preview sessions with localhost auth clearing controls
- [x] Upgrade the embedded inspector with live hover capture and CSS override editing
- [ ] Validate all new preview handlers with `just run`

### AI Provider Hub and Model Picker
- [x] Add a live provider-hub catalog sync that merges `models.dev`, OpenRouter, and optional LiteLLM model metadata
- [x] Cache the merged provider catalog to disk and surface cached/live sync state in the real Zed UI
- [x] Register manifest-backed OpenAI-compatible providers dynamically from the synced catalog
- [x] Upgrade the real AI settings Add Provider modal with curated presets and live model discovery
- [x] Add provider category, model-count, context-window, and capability metadata to the real model picker and AI settings panel
- [x] Add collapsible provider groups with per-provider model-count badges to the real AI model picker popover
- [ ] Validate the provider hub, presets, and model discovery flow with `just run`

### AI Composer and Editor Effects
- [x] Redesign the real AI composer with a VS Code Copilot and Codex-inspired shell, top quick actions, and bottom control rail
- [x] Add a responsive media mode switcher with overflow handling for text, audio, video, 3D, live, AR, VR, PDF, and chart workflows
- [x] Add a rainbow animated caret to the real Zed editor paint path
- [x] Intensify the rainbow caret animation with a visible multi-band glow and faster hue motion
- [x] Add cycle-based typing particle effects inspired by the VS Code power-mode extension
- [x] Increase typing particle size, count, and persistence for a stronger power-mode effect

### Onboarding and Dx Theme
- [x] Add bundled `Dx Light` and `Dx Dark` themes and make them the default theme family for fresh setups
- [x] Surface `Dx` directly in the onboarding theme picker
- [x] Add an onboarding theme-editor entry point that applies live per-theme overrides instantly
## Current Sprint

**Active Phase:** Phase 6 - Screen Carousel  
**Started:** March 31, 2026  
**Target Completion:** March 31, 2026

**Next Up:** Phase 7 - macOS-Style Dock

---

## Blockers & Issues

### Active Blockers
- None currently

### Resolved Issues
- None yet

---

## Performance Targets

- âœ… Maintain 120 FPS (8.33ms frame budget)
- âœ… GPU-accelerated rendering (Metal/Vulkan)
- âœ… Minimal CPU usage during animations
- âœ… Memory efficient (no leaks)
- âœ… Works on low-end devices

---

## Testing Checklist

### Per-Component Testing
- [ ] Visual accuracy (matches web reference)
- [ ] Performance (120 FPS maintained)
- [ ] Timing accuracy (matches specifications)
- [ ] Spring physics behavior
- [ ] Edge cases handled
- [ ] Memory usage acceptable

### Integration Testing
- [ ] All animations work together
- [ ] No visual conflicts
- [ ] Performance maintained with all active
- [ ] Cross-platform compatibility

---

## Notes

- **CRITICAL:** Only use `just run` command for testing - NEVER use `cargo build`, `cargo test`, `cargo check`, or `cargo clippy` due to low-end device constraints
- Web reference code should be in `www/` folder for easy comparison
- Demo tab should show live dimensions and animation states
- All animations must be GPU-accelerated for performance

---

## Completion Metrics

**Total Tasks:** 64 / 110+  
**Phases Completed:** 2 / 10  
**Overall Progress:** 58%

**Estimated Total Time:** 45-55 hours  
**Time Spent:** 2.75 hours  
**Time Remaining:** 42.25-52.25 hours




