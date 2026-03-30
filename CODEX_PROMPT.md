# Professional Codex CLI Prompt - Zed Animation Project

**Date:** March 31, 2026  
**Project:** Zed Editor Animation Recreation using Rust + GPUI

---

## 🎯 Master Prompt for Codex CLI

```bash
uto --profile ci
```

Then use this prompt:

---

## Primary Objective

You are tasked with systematically implementing all animations from our web application into the Zed code editor using Rust and GPUI. This is a **large-scale project** that requires careful, phase-by-phase execution with continuous progress tracking.

### Critical Constraints

⚠️ **ABSOLUTE REQUIREMENT - LOW-END DEVICE:**

**YOU MUST ONLY USE THE `just run` COMMAND FOR TESTING.**

- ❌ **NEVER EVER use:** `cargo build`
- ❌ **NEVER EVER use:** `cargo test`
- ❌ **NEVER EVER use:** `cargo check`
- ❌ **NEVER EVER use:** `cargo clippy`
- ❌ **NEVER EVER use:** Any cargo command directly

**WHY:** This is a massive project (like building an operating system). Our low-end device has limited RAM and storage. Using cargo commands will cause out-of-memory errors and system crashes. The `just run` command is specifically optimized for resource-constrained development.

✅ **ONLY ALLOWED COMMAND FOR TESTING:**
```bash
just run
```

If you need to test your changes, you MUST use `just run` and nothing else. This is non-negotiable.

---

## Project Execution Strategy

### Phase-by-Phase Approach

Work through the project in **10 distinct phases** as outlined in `TODO.md`. After completing each phase:

1. ✅ Mark completed tasks in `TODO.md`
2. 📝 Update `CHANGELOG.md` with changes made
3. 🧪 Test using **ONLY** `just run` command
4. 📊 Update progress metrics
5. 🎯 Move to next phase

### Progress Tracking Requirements

After EVERY significant change, you MUST:

1. **Update TODO.md:**
   - Change `[ ]` to `[x]` for completed tasks
   - Update phase status (⏸️ → ⏳ → ✅)
   - Update "Last Updated" timestamp
   - Update completion metrics at bottom

2. **Update CHANGELOG.md:**
   - Add entry under appropriate version/date
   - Use proper commit message format
   - Document what was added/changed/fixed
   - Update statistics section

3. **Test Changes:**
   - Run `just run` to verify implementation
   - Check demo tab for visual accuracy
   - Verify 120 FPS performance maintained
   - Compare with web reference in `www/` folder

---

## Implementation Phases

### Phase 1: Project Setup & Infrastructure (START HERE)

**Goal:** Set up the foundation for all animation work.

**Tasks:**
1. Create `crates/animation_demo/` directory structure
2. Create `Cargo.toml` with dependencies:
   ```toml
   [package]
   name = "animation_demo"
   version = "0.1.0"
   edition = "2021"
   
   [dependencies]
   gpui = { path = "../gpui" }
   smallvec = "1.13"
   ```
3. Create `src/lib.rs` with module declarations
4. Create demo tab component (`src/demo_tab.rs`) similar to Zed's onboarding tab
5. Add dimension display panel to demo tab
6. Set up video/3D rendering preview area in demo tab
7. Verify `just run` works correctly

**Success Criteria:**
- Demo tab opens in Zed
- Dimension panel displays current measurements
- Video/3D preview area is visible
- `just run` executes without errors

**Update After Completion:**
- Mark all Phase 1 tasks as complete in TODO.md
- Add Phase 1 completion entry to CHANGELOG.md
- Update overall progress percentage

---

### Phase 2: Core Animation Components

**Goal:** Build the animation primitives that all other components will use.

**Tasks:**
1. Create `src/spring.rs` - Spring physics system with Apple-style presets
2. Create `src/easing.rs` - Bezier easing curves
3. Create `src/animator.rs` - AnimatedValue, AnimatedPoint, AnimatedColor
4. Create `src/transition.rs` - Pre-built Apple-style transitions
5. Create `src/gesture.rs` - Gesture handling system
6. Test all primitives in demo tab

**Reference:** See DX.md for complete spring physics implementation

**Success Criteria:**
- All animation primitives compile
- Spring physics matches Apple behavior
- Demo tab shows working animation examples
- `just run` executes successfully

---

### Phase 3: Friday Border Effect

**Goal:** Implement the rainbow gradient border animation.

**Tasks:**
1. Create `src/friday.rs`
2. Implement Metal/Vulkan shader for 25-color HSL gradient
3. Add border slide-in animation (750ms)
4. Implement dual glow layers (8px + 12px blur)
5. Add scroll bounce effect with spring physics
6. Test gradient shift animation (0% → 200%)
7. Verify 120 FPS performance
8. Add to demo tab with live preview

**Reference:** 
- Web code: `www/components/friday.tsx`
- Specs: ANIMATIONS.md, section "FRIDAY BORDER EFFECT"

**Success Criteria:**
- Border slides in smoothly from bottom/right
- Rainbow gradient animates continuously
- Glow effect visible and smooth
- Maintains 120 FPS
- Matches web reference visually

---

### Phase 4: Hello Glow Effect

**Goal:** Implement the animated rainbow gradient background.

**Tasks:**
1. Create `src/hello_glow.rs`
2. Implement 25-color HSL gradient shader
3. Add dual-layer blur (6px + 18px)
4. Implement background-position animation (6s cycle)
5. Add border radius rendering (12px)
6. Test glow intensity
7. Add to demo tab

**Reference:**
- Web code: `www/components/hello-glow.tsx`
- Specs: ANIMATIONS.md, section "HELLO GLOW EFFECT"

---

### Phase 5: Sidebar Animations

**Goal:** Implement the complete sidebar with all animations.

**Tasks:**
1. Create `src/sidebar.rs`
2. Implement width transition (56px ↔ 360px, 200ms)
3. Add content fade (opacity 0 → 1, 150ms)
4. Implement folder collapse (height auto → 0, 200ms)
5. Add workspace scroll with spring animation
6. Implement media player slide-up (y: 20 → 0)
7. Add hover effects (scale 1.0 → 1.25)
8. Implement drag constraints with elastic (0.1-0.2)
9. Test smooth scroll with continuous hold
10. **Replace Zed's default sidebar** in `crates/workspace/src/sidebar.rs`

**Reference:**
- Web code: `www/components/browser/sidebar-*.tsx`
- Specs: ANIMATIONS.md, section "SIDEBAR ANIMATIONS"

**Success Criteria:**
- Sidebar expands/collapses smoothly
- All interactive elements work
- Replaces Zed's default sidebar
- Maintains 120 FPS

---

### Phase 6: Screen Carousel

**Goal:** Implement the resizable screen carousel with directional gravity.

**Tasks:**
1. Create `src/carousel.rs`
2. Implement resizable screen system
3. Add directional gravity logic:
   - Left resize → push screen right
   - Right resize → push screen left
   - Both sides → center
4. Implement smooth positioning (spring: stiffness 300, damping 30)
5. Add live resize with wrapper width updates
6. Implement circular wrapping
7. Add screen transitions (scale 0.95 → 1, opacity 0 → 1)
8. Test with multiple screens
9. **Replace Zed's tab system** in `crates/editor/src/tabs.rs`

**Reference:**
- Web code: `www/components/screens/screen-carousel.tsx`
- Specs: ANIMATIONS.md, section "SCREEN CAROUSEL"

---

### Phase 7: macOS-Style Dock

**Goal:** Add the top dock for controlling the screen carousel.

**Tasks:**
1. Create `src/macos_dock.rs`
2. Implement dock icon rendering
3. Add hover magnification (scale 1.0 → 1.3 with spring)
4. Implement active indicator with pulsing glow
5. Add click-to-switch functionality
6. Implement drag-to-reorder
7. Add plus button for new screen
8. Add close button (X) with animation
9. Connect dock to carousel state
10. Integrate into workspace top bar

**Reference:**
- Specs: ANIMATIONS.md, section "MACOS-STYLE DOCK"

---

### Phase 8: Drag-and-Drop System

**Goal:** Implement the complete drag-and-drop system.

**Tasks:**
1. Create drag-and-drop event handling
2. Implement sortable tabs
3. Implement sortable folders
4. Add drop position indicators
5. Create drag overlay with shadow
6. Implement logo container as drop target
7. Add all visual feedback
8. Test workspace organization

**Reference:**
- Web code: `www/components/browser/draggable-tab.tsx`
- Specs: ANIMATIONS.md, section "DRAG-AND-DROP SYSTEM"

---

### Phase 9: Video & 3D Rendering

**Goal:** Add video and 3D rendering capabilities to the demo tab.

**Tasks:**
1. Research video player crates (recommend: `gstreamer` or `ffmpeg-next`)
2. Research 3D rendering crates (recommend: `wgpu` or `three-d`)
3. Implement video player component
4. Implement 3D viewer component
5. Add video/3D preview to demo tab
6. Test video playback performance
7. Test 3D rendering performance
8. Ensure 120 FPS for UI while video/3D plays
9. Add controls (play/pause, seek, rotate)

**Success Criteria:**
- Video plays smoothly in demo tab
- 3D models render and rotate
- UI maintains 120 FPS during playback
- Controls are responsive

---

### Phase 10: Integration & Polish

**Goal:** Integrate everything and polish the final product.

**Tasks:**
1. Integrate all components into Zed workspace
2. Test all animations together
3. Performance optimization pass
4. Fix any visual glitches
5. Cross-platform testing (macOS, Linux, Windows)
6. Update documentation
7. Final demo tab polish

---

## Continuous Requirements

### After Every Code Change:

1. **Update TODO.md:**
   ```markdown
   - [x] Completed task description
   ```
   Update phase status and metrics

2. **Update CHANGELOG.md:**
   ```markdown
   ### [0.1.0] - 2026-03-31
   
   #### Added
   - Implemented Friday border effect with rainbow gradient shader
   - Added dual glow layers (8px + 12px blur)
   
   #### Changed
   - Optimized spring physics for better performance
   
   #### Fixed
   - Fixed gradient animation stuttering at 120 FPS
   ```

3. **Test with `just run`:**
   ```bash
   just run
   ```
   **NEVER use cargo commands!**

4. **Verify in Demo Tab:**
   - Check visual accuracy against web reference
   - Verify dimensions are correct
   - Confirm 120 FPS maintained
   - Test all interactive elements

---

## Video & 3D Rendering Requirements

### Video Player Implementation

**Recommended Crate:** `gstreamer` or `ffmpeg-next`

**Requirements:**
- Support common formats (MP4, WebM, MOV)
- Hardware acceleration when available
- Smooth playback at 30/60 FPS
- UI maintains 120 FPS during playback
- Controls: play, pause, seek, volume

**Integration:**
- Render in demo tab preview area
- Show video dimensions and FPS
- Display playback controls
- Allow file selection

### 3D Viewer Implementation

**Recommended Crate:** `wgpu` or `three-d`

**Requirements:**
- Support common formats (OBJ, GLTF, FBX)
- Real-time rotation and zoom
- Lighting and materials
- UI maintains 120 FPS during rendering
- Controls: rotate, zoom, pan

**Integration:**
- Render in demo tab preview area
- Show model info (vertices, faces)
- Display rotation controls
- Allow file selection

---

## Quality Standards

### Performance
- ✅ Maintain 120 FPS (8.33ms frame budget)
- ✅ GPU-accelerated rendering only
- ✅ Minimal CPU usage during animations
- ✅ No memory leaks
- ✅ Works on low-end devices

### Visual Accuracy
- ✅ Matches web reference exactly
- ✅ Correct timing and easing
- ✅ Proper spring physics behavior
- ✅ Smooth transitions
- ✅ No visual glitches

### Code Quality
- ✅ Clean, readable Rust code
- ✅ Proper error handling
- ✅ Documented functions and types
- ✅ Follows GPUI patterns
- ✅ No unsafe code unless necessary

---

## Communication Protocol

### When Starting a Phase:
```
Starting Phase X: [Phase Name]
- Reading specifications from ANIMATIONS.md
- Reviewing web reference code in www/
- Planning implementation approach
```

### When Completing a Task:
```
✅ Completed: [Task Description]
- Implementation details
- Testing results (using `just run`)
- Performance metrics
- Updated TODO.md and CHANGELOG.md
```

### When Encountering Issues:
```
⚠️ Issue Encountered: [Issue Description]
- What was attempted
- Error message or behavior
- Proposed solution
- Need clarification on: [specific question]
```

### When Completing a Phase:
```
🎉 Phase X Complete: [Phase Name]
- All tasks completed
- TODO.md updated
- CHANGELOG.md updated
- Testing passed with `just run`
- Ready for Phase X+1
```

---

## Final Reminders

### CRITICAL - Testing Command
**ONLY USE THIS COMMAND:**
```bash
just run
```

**NEVER USE THESE:**
```bash
cargo build    # ❌ Will crash system
cargo test     # ❌ Will crash system
cargo check    # ❌ Will crash system
cargo clippy   # ❌ Will crash system
```

### Documentation Updates
- Update TODO.md after every task
- Update CHANGELOG.md after every significant change
- Keep progress metrics current
- Document all decisions and trade-offs

### Reference Materials
- AGENTS.md: Project instructions and standards
- ANIMATIONS.md: Complete animation specifications
- DX.md: Smooth animation implementation patterns
- TODO.md: Task tracking and progress
- CHANGELOG.md: Change history
- www/: Web reference code for comparison

---

## Success Criteria for Project Completion

- [ ] All 10 phases completed
- [ ] All animations implemented and working
- [ ] Video and 3D rendering functional
- [ ] Demo tab shows all features
- [ ] Zed's sidebar replaced with custom implementation
- [ ] Zed's tabs replaced with screen carousel
- [ ] macOS dock integrated at top
- [ ] All animations maintain 120 FPS
- [ ] Works on low-end devices
- [ ] TODO.md shows 100% completion
- [ ] CHANGELOG.md fully documented
- [ ] Cross-platform tested (macOS, Linux, Windows)

---

**Start with Phase 1 and work systematically through each phase. Update documentation continuously. Test only with `just run`. Good luck!**
