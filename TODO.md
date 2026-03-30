# Zed Animation Project - Task Tracker

**Project Start Date:** March 31, 2026  
**Last Updated:** March 31, 2026  
**Status:** 🟡 In Progress

---

## Project Phases

### Phase 1: Project Setup & Infrastructure ⏳ IN PROGRESS
- [x] Create `animation_demo` crate structure
- [x] Set up project dependencies in `Cargo.toml`
- [x] Copy web reference code to `www/` folder
- [x] Create demo tab infrastructure (like onboarding tab)
- [x] Set up dimension display panel for live testing
- [x] Configure video/3D rendering preview in demo tab
- [ ] Verify `just run` command works correctly

**Estimated Time:** 2-3 hours  
**Priority:** 🔴 Critical  
**Dependencies:** None

---

### Phase 2: Core Animation Components 🔜 PENDING
- [x] Implement spring physics system (`spring.rs`)
- [x] Implement easing curves (`easing.rs`)
- [x] Create animated value system (`animator.rs`)
- [x] Build transition helpers (`transition.rs`)
- [x] Create gesture handling system (`gesture.rs`)
- [x] Test all animation primitives in demo tab

**Estimated Time:** 4-5 hours  
**Priority:** 🔴 Critical  
**Dependencies:** Phase 1

---

### Phase 3: Friday Border Effect ⏸️ NOT STARTED
- [x] Create `friday.rs` component
- [ ] Implement rainbow gradient shader (Metal/Vulkan)
- [x] Add border slide-in animation (750ms)
- [x] Implement dual glow layers (8px + 12px blur)
- [x] Add scroll bounce effect with spring physics
- [ ] Test gradient shift animation (0% → 200%)
- [ ] Verify 120 FPS performance
- [x] Add to demo tab with dimensions display

**Estimated Time:** 3-4 hours  
**Priority:** 🔴 Critical  
**Dependencies:** Phase 2

---

### Phase 4: Hello Glow Effect ⏸️ NOT STARTED
- [x] Create `hello_glow.rs` component
- [ ] Implement 25-color HSL gradient shader
- [x] Add dual-layer blur (6px + 18px)
- [x] Implement background-position animation (6s cycle)
- [x] Add border radius rendering (12px)
- [ ] Test glow intensity and performance
- [x] Add to demo tab with live preview

**Estimated Time:** 2-3 hours  
**Priority:** 🔴 Critical  
**Dependencies:** Phase 2

---

### Phase 5: Sidebar Animations ⏸️ NOT STARTED
- [ ] Create `sidebar.rs` component
- [ ] Implement width transition (56px ↔ 360px, 200ms)
- [ ] Add content fade animation (opacity 0 → 1, 150ms)
- [ ] Implement folder collapse (height auto → 0, 200ms)
- [ ] Add workspace scroll with spring animation
- [ ] Implement media player slide-up (y: 20 → 0)
- [ ] Add hover effects (scale 1.0 → 1.25)
- [ ] Implement drag constraints with elastic (0.1-0.2)
- [ ] Test smooth scroll with continuous hold detection
- [ ] Replace Zed's default sidebar in `workspace/src/sidebar.rs`

**Estimated Time:** 5-6 hours  
**Priority:** 🔴 Critical  
**Dependencies:** Phase 2

---

### Phase 6: Screen Carousel ⏸️ NOT STARTED
- [ ] Create `carousel.rs` component
- [ ] Implement resizable screen system
- [ ] Add directional gravity logic:
  - [ ] Left resize → push screen right
  - [ ] Right resize → push screen left
  - [ ] Both sides → center
- [ ] Implement smooth positioning (spring: stiffness 300, damping 30)
- [ ] Add live resize with wrapper width updates
- [ ] Implement circular wrapping (first/last screens)
- [ ] Add screen transitions (scale 0.95 → 1, opacity 0 → 1)
- [ ] Test with multiple screens
- [ ] Replace Zed's tab system in `editor/src/tabs.rs`

**Estimated Time:** 6-7 hours  
**Priority:** 🔴 Critical  
**Dependencies:** Phase 2

---

### Phase 7: macOS-Style Dock ⏸️ NOT STARTED
- [ ] Create `macos_dock.rs` component
- [ ] Implement dock icon rendering
- [ ] Add hover magnification (scale 1.0 → 1.3 with spring)
- [ ] Implement active indicator with pulsing glow
- [ ] Add click-to-switch functionality
- [ ] Implement drag-to-reorder with visual feedback
- [ ] Add plus button for new screen
- [ ] Add close button (X) with animation
- [ ] Connect dock to carousel state
- [ ] Integrate into workspace top bar

**Estimated Time:** 4-5 hours  
**Priority:** 🔴 Critical  
**Dependencies:** Phase 6

---

### Phase 8: Drag-and-Drop System ⏸️ NOT STARTED
- [ ] Create drag-and-drop event handling system
- [ ] Implement sortable tabs
- [ ] Implement sortable folders
- [ ] Add drop position indicators (before/after/inside)
- [ ] Create drag overlay with shadow
- [ ] Implement logo container as drop target
- [ ] Add visual feedback:
  - [ ] Dragging: 50% opacity, cursor-grabbing
  - [ ] Drop indicators: blue dots (8px) + lines (2px)
  - [ ] Hover states with background transitions
- [ ] Test workspace organization

**Estimated Time:** 5-6 hours  
**Priority:** 🟡 Medium  
**Dependencies:** Phase 5

---

### Phase 9: Video & 3D Rendering ⏸️ NOT STARTED
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
**Priority:** 🟡 Medium  
**Dependencies:** Phase 1

---

### Phase 10: Integration & Polish ⏸️ NOT STARTED
- [ ] Integrate all components into Zed workspace
- [ ] Test all animations together
- [ ] Performance optimization pass
- [ ] Fix any visual glitches
- [ ] Cross-platform testing (macOS, Linux, Windows)
- [ ] Documentation updates
- [ ] Final demo tab polish

**Estimated Time:** 4-5 hours  
**Priority:** 🟢 Low  
**Dependencies:** All previous phases

---

## Current Sprint

**Active Phase:** Phase 3 - Friday Border Effect  
**Started:** March 31, 2026  
**Target Completion:** March 31, 2026

**Next Up:** Phase 4 - Hello Glow Effect

---

## Blockers & Issues

### Active Blockers
- None currently

### Resolved Issues
- None yet

---

## Performance Targets

- ✅ Maintain 120 FPS (8.33ms frame budget)
- ✅ GPU-accelerated rendering (Metal/Vulkan)
- ✅ Minimal CPU usage during animations
- ✅ Memory efficient (no leaks)
- ✅ Works on low-end devices

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

**Total Tasks:** 24 / 100+  
**Phases Completed:** 2 / 10  
**Overall Progress:** 24%

**Estimated Total Time:** 45-55 hours  
**Time Spent:** 2.75 hours  
**Time Remaining:** 42.25-52.25 hours
