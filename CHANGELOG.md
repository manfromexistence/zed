# Zed Animation Project - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

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
**Status:** 🟡 In Progress

- Create `animation_demo` crate structure
- Set up project dependencies
- Copy web reference code to `www/` folder
- Create demo tab infrastructure
- Configure video/3D rendering preview

### Phase 2: Core Animation Components (Planned)
**Target:** April 1-2, 2026  
**Status:** 🔜 Pending

- Implement spring physics system
- Implement easing curves
- Create animated value system
- Build transition helpers
- Create gesture handling system

### Phase 3: Friday Border Effect (Planned)
**Target:** April 3-4, 2026  
**Status:** ⏸️ Not Started

- Rainbow gradient shader implementation
- Border slide-in animation
- Dual glow layers
- Scroll bounce effect

### Phase 4: Hello Glow Effect (Planned)
**Target:** April 5, 2026  
**Status:** ⏸️ Not Started

- 25-color HSL gradient shader
- Dual-layer blur system
- Background-position animation

### Phase 5: Sidebar Animations (Planned)
**Target:** April 6-7, 2026  
**Status:** ⏸️ Not Started

- Width transition animations
- Content fade animations
- Folder collapse system
- Workspace scroll with spring
- Replace Zed's default sidebar

### Phase 6: Screen Carousel (Planned)
**Target:** April 8-10, 2026  
**Status:** ⏸️ Not Started

- Resizable screen system
- Directional gravity logic
- Smooth positioning with springs
- Circular wrapping
- Replace Zed's tab system

### Phase 7: macOS-Style Dock (Planned)
**Target:** April 11-12, 2026  
**Status:** ⏸️ Not Started

- Dock icon rendering
- Hover magnification
- Active indicator with glow
- Drag-to-reorder functionality

### Phase 8: Drag-and-Drop System (Planned)
**Target:** April 13-14, 2026  
**Status:** ⏸️ Not Started

- Event handling system
- Sortable tabs and folders
- Drop position indicators
- Visual feedback system

### Phase 9: Video & 3D Rendering (Planned)
**Target:** April 15-17, 2026  
**Status:** ⏸️ Not Started

- Video player component
- 3D viewer component
- Preview in demo tab
- Performance optimization

### Phase 10: Integration & Polish (Planned)
**Target:** April 18-19, 2026  
**Status:** ⏸️ Not Started

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
