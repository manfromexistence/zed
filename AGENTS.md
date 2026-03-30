# Zed Editor Animation Project - Agent Instructions

**Project Date:** March 31, 2026  
**Project Goal:** Recreate web application animations in Zed code editor using Rust + GPUI

---

## Project Overview

This is a Zed code editor fork that recreates essential animations from our Friday web application. We are building a complete UI transformation of Zed using Rust and GPUI (GPU-accelerated UI framework).

### Technology Stack

**Current Web Stack (Reference):**
- Next.js 16 + React 19
- Framer Motion 12.36 for animations
- @dnd-kit for drag-and-drop
- CSS keyframes and transitions
- Canvas API for complex effects

**Target Stack (Implementation):**
- Rust (Edition 2024)
- GPUI (Zed's UI framework)
- Metal/Vulkan for GPU rendering
- Custom shaders for effects

---

## Core UI Replacements

We are replacing Zed's existing UI components with our custom implementations:

| Zed Component | Our Replacement | Purpose |
|---------------|-----------------|---------|
| Workspace Sidebar (right) | Custom Sidebar | Workspaces, folders, tabs, logo container |
| Editor Tabs (top) | Screen Carousel | Resizable panels with directional gravity |
| Tab Bar | macOS-Style Dock | Visual tab control at top of editor |
| (New) Demo Tab | Animation Showcase | Testing environment with dimensions |

---

## Development Workflow

### CRITICAL - Low-End System Requirements

⚠️ **ONLY USE `just run` COMMAND** ⚠️

This Zed project is massive and runs like a low-level operating system. On low-end devices:

- ❌ **NEVER use:** `cargo build`, `cargo test`, `cargo check`, `cargo clippy`
- ❌ **NEVER use:** Any cargo commands directly
- ✅ **ONLY use:** `just run` to test the application
- ✅ **ONLY use:** `just` commands if available

**Why:** Building Zed requires significant RAM and disk space. The `just run` command is optimized for development on resource-constrained systems. Using other commands will cause out-of-memory errors and system crashes.

**Testing Workflow:**
```bash
# CORRECT - Use this
just run

# WRONG - Never use these on low-end systems
cargo build --release  # ❌ Will crash
cargo test             # ❌ Will crash
cargo check            # ❌ Will crash
```

### Side-by-Side Reference Development

1. **Web Code Reference:** Copy the original website code into `zed/www/` folder so AI agents can easily learn how animations work
2. **Live Demo Tab:** Create a new untitled tab (like onboarding) that displays all animations with their dimensions
3. **Core UI Replacement:** Replace Zed's existing workspace sidebar and tab system with our implementations
4. **Iterative Development:** Reference web code in `www/` while implementing GPUI equivalents
5. **Visual Validation:** Compare web animations with GPUI implementations in real-time

---

## Priority Animations to Implement

### 1. FRIDAY BORDER EFFECT ⭐⭐⭐ (CRITICAL)
**Location:** `components/friday.tsx` (web reference)

**Description:**
Animated rainbow gradient borders that slide in from edges with smooth scroll animation and glow effects.

**Technical Details:**
- 25-color HSL gradient (0° to 360°)
- 10px border thickness
- Clip-path for diagonal corners
- CSS animations with 3s cycle
- Dual glow layers (tight 8px + outer 12px blur)

**Animation Phases:**
1. **Entering:** Bottom/right borders slide in (750ms)
2. **Active:** All borders visible, scroll bounce effect with spring physics
3. **Exiting:** Fade out

**Key Features:**
- Gradient shift animation (background-position: 0% → 200%)
- Blur filters for glow effects
- Spring physics for scroll (stiffness: 0.03, damping: 0.2)
- Z-index layering (9990)

### 2. HELLO GLOW EFFECT ⭐⭐⭐ (CRITICAL)
**Location:** `components/hello-glow.tsx` (web reference)

**Description:**
Animated rainbow gradient background with dual-layer glow effect.

**Implementation:**
- 25-color HSL gradient (0° to 360°)
- Dual-layer blur (6px + 18px)
- Background-position animation (0% → 200%)
- 6-second animation cycle
- Border radius: 12px

### 3. SIDEBAR ANIMATIONS ⭐⭐⭐ (CRITICAL)
**Location:** `components/browser/sidebar-*.tsx` (web reference)

**Description:**
Smooth sidebar expand/collapse with content transitions and interactive elements.

**Animations:**
1. **Width Transition:** 56px ↔ 360px (200ms)
2. **Content Fade:** Opacity 0 → 1 (150ms)
3. **Folder Collapse:** Height auto → 0 (200ms)
4. **Workspace Scroll:** Spring animation with scroll buttons
5. **Media Player:** Slide up from bottom (y: 20 → 0)

**Interactive Elements:**
- Workspace icons with scale on hover (1.0 → 1.25)
- Drag constraints with elastic (0.1-0.2)
- Spring physics (stiffness: 400, damping: 30, mass: 0.5)
- Smooth scroll with continuous hold detection

### 4. SCREEN CAROUSEL ⭐⭐⭐ (CRITICAL)
**Location:** `components/screens/screen-carousel.tsx` (web reference)

**Description:**
Resizable screen carousel with directional gravity and smooth transitions.

**Key Features:**
1. **Resizable Screens:** Using re-resizable library
2. **Directional Gravity:** 
   - Left resize → push screen right (reveal left neighbor)
   - Right resize → push screen left (reveal right neighbor)
   - Both sides → center (reveal both neighbors)
3. **Smooth Positioning:** Spring animation (stiffness: 300, damping: 30)
4. **Live Resize:** Updates wrapper width during resize for real-time feedback
5. **Circular Wrapping:** First/last screens wrap around for infinite feel

### 5. DRAG-AND-DROP SYSTEM ⭐⭐
**Location:** `components/browser/`, `app/page.tsx` (web reference)

**Description:**
Complex drag-and-drop system using @dnd-kit with visual feedback.

**Features:**
- Sortable tabs and folders
- Drop position indicators (before/after/inside)
- Drag overlay with shadow
- Logo container as drop target
- Workspace organization

**Visual Feedback:**
1. **Dragging:** 50% opacity, cursor-grabbing
2. **Drop Indicators:** 
   - Blue dots (8px diameter) + lines (2px height)
   - Position: before/after/inside
3. **Hover States:** Background color transitions
4. **Logo Container:** Highlight on valid drop

### 6. MACOS-STYLE DOCK ⭐⭐⭐ (NEW - CRITICAL)
**Location:** New component at top of editor

**Description:**
macOS-style dock at the top of the Zed editor to control the screen carousel (tabs).

**Features:**
1. **Dock Icons:** Visual representation of each screen/tab
2. **Hover Effects:** Icon magnification on hover
3. **Active Indicator:** Highlight current screen
4. **Click to Switch:** Click icon to switch to that screen
5. **Drag to Reorder:** Drag icons to reorder screens
6. **Add/Remove:** Plus button to add new screen, X to close

**Animations:**
- **Hover Magnification:** Scale 1.0 → 1.3 with spring
- **Active Glow:** Pulsing glow effect
- **Drag Feedback:** Icon follows cursor, others shift
- **Add Animation:** New icon scales in from 0 → 1
- **Remove Animation:** Icon scales out and fades

---

## Project Structure

```
zed/
├── crates/
│   ├── workspace/          # MODIFY: Replace sidebar
│   │   └── src/
│   │       └── sidebar.rs  # Replace with our sidebar
│   ├── editor/             # MODIFY: Replace tabs
│   │   └── src/
│   │       └── tabs.rs     # Replace with carousel
│   └── animation_demo/     # NEW: Demo and components
│       ├── src/
│       │   ├── lib.rs
│       │   ├── demo_tab.rs
│       │   ├── friday.rs
│       │   ├── hello_glow.rs
│       │   ├── sidebar.rs      # Our sidebar implementation
│       │   ├── carousel.rs     # Screen carousel
│       │   └── macos_dock.rs   # Top dock
│       └── Cargo.toml
└── www/                    # ⭐ NEW: Web code reference (for AI agents)
    ├── components/
    │   ├── friday.tsx
    │   ├── hello-glow.tsx
    │   ├── browser/
    │   │   ├── sidebar-expanded.tsx
    │   │   ├── sidebar-collapsed.tsx
    │   │   └── draggable-tab.tsx
    │   └── screens/
    │       ├── screen-carousel.tsx
    │       └── macos-dock.tsx
    ├── app/
    ├── package.json
    └── README.md
```

---

## Coding Standards

### Rust/GPUI Conventions

1. **Use GPUI's Element Trait:** All custom UI components must implement the `Element` trait
2. **Render Trait for Views:** Use `impl Render for MyView` for components that can be rendered
3. **Spring Physics:** Use analytical spring solutions (not Euler integration) for smooth animations
4. **Frame Rate Target:** All animations must maintain 120 FPS (8.33ms frame budget)
5. **GPU Shaders:** Use custom Metal/Vulkan shaders for complex effects (gradients, blurs)

### Animation Standards

1. **Spring Configurations:**
   - Default: `response: 0.55, damping: 1.0` (critically damped)
   - Snappy: `response: 0.3, damping: 0.82`
   - Bouncy: `response: 0.5, damping: 0.6`
   - Smooth: `response: 0.5, damping: 1.0`

2. **Timing:**
   - Quick transitions: 150-200ms
   - Standard transitions: 300-350ms
   - Slow transitions: 500-750ms

3. **Easing:**
   - Use spring physics for layout changes
   - Use bezier curves for opacity/color transitions
   - Apple default: `cubic-bezier(0.25, 0.1, 0.25, 1.0)`

### File Organization

1. **One component per file:** Each animation component gets its own Rust file
2. **Shared utilities:** Common animation helpers go in `animation_demo/src/utils/`
3. **Shaders:** Custom shaders go in `animation_demo/shaders/`
4. **Tests:** Unit tests in same file, integration tests in `tests/` directory

---

## Reference Documentation

### Key Files to Study

**Zed Architecture:**
- `crates/workspace/src/workspace.rs` - Main workspace structure
- `crates/workspace/src/pane.rs` - Tab/pane system
- `crates/workspace/src/sidebar.rs` - Current sidebar
- `crates/editor/src/editor.rs` - Editor component

**Web Reference (in www/ folder):**
- `www/components/friday.tsx` - Friday border animation
- `www/components/hello-glow.tsx` - Hello glow effect
- `www/components/browser/sidebar-*.tsx` - Sidebar animations
- `www/components/screens/screen-carousel.tsx` - Screen carousel

### External Documentation

- **GPUI Documentation:** Study Zed's GPUI framework for UI rendering
- **ANIMATIONS.md:** Complete animation specifications and implementation guide
- **DX.md:** Developer experience guide with smooth animation patterns

---

## Constraints and Requirements

### Must Follow

1. **No Breaking Changes:** Do not modify Zed's core functionality outside of UI replacements
2. **Performance First:** All animations must maintain 120 FPS on target hardware
3. **GPU Acceleration:** Use Metal/Vulkan shaders for complex effects, not CPU rendering
4. **Memory Efficient:** Minimize allocations during animation frames
5. **Cross-Platform:** Ensure animations work on macOS, Linux, and Windows

### Must Avoid

1. **Blocking Operations:** Never block the main thread during animations
2. **Synchronous I/O:** All file operations must be async
3. **Memory Leaks:** Properly clean up animation resources when components unmount
4. **Cargo Commands:** Never use `cargo build`, `cargo test`, etc. - only use `just run`

---

## Testing Strategy

### Manual Testing

1. **Demo Tab:** Test all animations in the demo tab with live dimension display
2. **Visual Comparison:** Compare GPUI animations side-by-side with web reference
3. **Performance Monitoring:** Use built-in profiler to ensure 120 FPS
4. **Cross-Platform:** Test on macOS, Linux, and Windows

### Automated Testing

1. **Unit Tests:** Test animation math (spring physics, easing functions)
2. **Integration Tests:** Test component interactions (sidebar + carousel)
3. **Performance Tests:** Benchmark animation frame times

---

## Communication Style

When implementing features:

1. **Be Specific:** Reference exact file paths and line numbers
2. **Show Examples:** Provide code snippets from web reference when relevant
3. **Explain Trade-offs:** Discuss performance vs. visual quality decisions
4. **Ask for Clarification:** If animation specs are unclear, ask before implementing
5. **Document Decisions:** Add comments explaining why certain approaches were chosen

---

## Current Task Context

We are using Codex CLI to implement all animations listed in ANIMATIONS.md. The web reference code is in the `www/` folder for easy reference. Start by:

1. Creating the `animation_demo` crate structure
2. Implementing the demo tab to showcase animations
3. Recreating each priority animation in order (Friday Border → Hello Glow → Sidebar → Carousel → Drag-Drop → macOS Dock)
4. Replacing Zed's default sidebar and tab system with our implementations

---

## Additional Resources

- **ANIMATIONS.md:** Complete animation specifications (3183 lines)
- **DX.md:** Developer experience guide with smooth animation patterns (3128 lines)
- **Web Reference:** Complete website code in `www/` folder for AI agents to study

---

**Last Updated:** March 31, 2026  
**Project Status:** Active Development  
**Target Release:** Q2 2026
