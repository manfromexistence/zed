# Zed Editor + GPUI Animation Recreation Guide

**Date:** March 30, 2026  
**Project:** Recreating Essential Web Animations in Zed Code Editor using Rust + GPUI

---

## Executive Summary

This document analyzes the critical animations from the Friday web application and provides a comprehensive guide for recreating them in the Zed code editor using Rust and GPUI (GPU-accelerated UI framework).

### Project Scope

**We are building a complete UI transformation of Zed:**

1. **Replace Zed's Workspace Sidebar** → Our custom sidebar with all animations and functionality
2. **Transform Zed's Tabs** → Screen carousel system with resizable panels
3. **Add macOS-Style Dock** → Top dock to control screen carousel (tabs)
4. **Demo Tab** → Testing environment for all animations

This is not just animation recreation - we're rebuilding Zed's core UI to match our web application.

### Development Workflow

**Important:** All development will be done directly in a Zed fork:

1. **Side-by-Side Reference:** Copy the original website code into `zed/www/` folder so AI agents can easily learn how animations work
2. **Live Demo Tab:** Create a new untitled tab (like onboarding) that displays all animations with their dimensions
3. **Core UI Replacement:** Replace Zed's existing workspace sidebar and tab system with our implementations
4. **Iterative Development:** Reference web code in `www/` while implementing GPUI equivalents
5. **Visual Validation:** Compare web animations with GPUI implementations in real-time

**CRITICAL - Low-End System Requirements:**

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

### Technology Stack Overview

**Current Web Stack:**
- Next.js 16 + React 19
- Framer Motion 12.36 for animations
- @dnd-kit for drag-and-drop
- CSS keyframes and transitions
- Canvas API for complex effects

**Target Stack:**
- Rust (Edition 2024)
- GPUI (Zed's UI framework)
- Metal/Vulkan for GPU rendering
- Custom shaders for effects

---

## Part 1: Understanding Zed & GPUI

### What is Zed?

Zed is a high-performance code editor built entirely in Rust, designed to render at 120 FPS like a video game. Key characteristics:

- **GPU-Accelerated:** Uses Metal (macOS) and Vulkan (Linux/Windows) for rendering
- **Rust-Native:** Built from scratch without web technologies
- **Collaborative:** Real-time multiplayer editing
- **AI-Powered:** Integrated AI assistance
- **Performance-First:** 8.33ms frame budget (120 FPS target)

### What is GPUI?

GPUI is Zed's custom UI framework - a hybrid immediate/retained mode, GPU-accelerated framework for Rust.

**Core Concepts:**

1. **Element Trait:** Building blocks of UI
   ```rust
   pub trait Element {
       fn layout(&mut self, constraint: SizeConstraint) -> Size;
       fn paint(&mut self, origin: (f32, f32), size: Size, scene: &mut Scene);
   }
   ```

2. **Render Trait:** For views that can be rendered
   ```rust
   impl Render for MyView {
       fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
           div().child("Hello")
       }
   }
   ```

3. **Custom Shaders:** GPU shaders for specific primitives (rectangles, shadows, text, icons)

4. **Scene Graph:** Platform-neutral collection of rendering primitives

**Rendering Pipeline:**
- Constraints flow down (parent → child)
- Sizes flow up (child → parent)
- Paint phase assembles Scene with primitives
- GPU renders at 120 FPS

**Key Advantages:**
- Direct GPU access for maximum performance
- No DOM overhead
- Predictable frame timing
- Custom shader control

---

## Part 2: Core UI Replacements

### Overview

We are replacing Zed's existing UI components with our custom implementations:

| Zed Component | Our Replacement | Purpose |
|---------------|-----------------|---------|
| Workspace Sidebar (right) | Custom Sidebar | Workspaces, folders, tabs, logo container |
| Editor Tabs (top) | Screen Carousel | Resizable panels with directional gravity |
| Tab Bar | macOS-Style Dock | Visual tab control at top of editor |
| (New) Demo Tab | Animation Showcase | Testing environment with dimensions |

### Architecture Changes

**Before (Zed Default):**
```
┌─────────────────────────────────────┐
│ [Tab 1] [Tab 2] [Tab 3]      [+]   │ ← Simple tab bar
├─────────────────────────────────────┤
│ Sidebar │ Editor Content            │
│ (basic) │                           │
└─────────────────────────────────────┘
```

**After (Our Implementation):**
```
┌─────────────────────────────────────┐
│ [🎯][📁][⚙️][🔧][📊]  macOS Dock    │ ← New: macOS-style dock
├─────────────────────────────────────┤
│ Custom  │ ┌──────┐ ┌──────┐        │
│ Sidebar │ │Screen│ │Screen│        │ ← Resizable carousel
│ (ours)  │ │  1   │ │  2   │        │
│         │ └──────┘ └──────┘        │
└─────────────────────────────────────┘
```

---

## Part 3: Essential Animations to Implement

### Priority Animations

We will focus on these critical animations that define the user experience:



#### 1. FRIDAY BORDER EFFECT ⭐⭐⭐ (CRITICAL)
**Location:** `components/friday.tsx`

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

**Web Code Reference:**
```typescript
const BORDER_THICKNESS = 10;
const GLOW_SPREAD = 8;
const GLOW_INTENSITY = 12;
const SLIDE_DURATION_MS = 750;

// Rainbow gradient generation
function generateRainbowGradient(count: number): string {
  const stops = Array.from({ length: count + 1 }, (_, i) => {
    const hue = (i / count) * 360;
    return `hsl(${hue}, 85%, 55%)`;
  });
  return `linear-gradient(90deg, ${stops.join(", ")})`;
}
```

---

#### 2. HELLO GLOW EFFECT ⭐⭐⭐ (CRITICAL)
**Location:** `components/hello-glow.tsx`

**Description:**
Animated rainbow gradient background with dual-layer glow effect.

**Implementation:**
- 25-color HSL gradient (0° to 360°)
- Dual-layer blur (6px + 18px)
- Background-position animation (0% → 200%)
- 6-second animation cycle
- Border radius: 12px

**CSS Structure:**
```css
.hello-glow-background {
  background: linear-gradient(90deg, hsl(0,80%,60%), ..., hsl(360,80%,60%));
  background-size: 200% 100%;
  animation: hello-gradient-shift 6s linear infinite;
  filter: blur(6px);
}

.hello-glow-background::after {
  inset: -15px;
  filter: blur(18px);
  opacity: 0.5;
}

@keyframes hello-gradient-shift {
  0% { background-position: 0% 0%; }
  100% { background-position: 200% 0%; }
}
```

---



#### 3. SIDEBAR ANIMATIONS ⭐⭐⭐ (CRITICAL)
**Location:** `components/browser/sidebar-*.tsx`

**Description:**
Smooth sidebar expand/collapse with content transitions and interactive elements.

**Animations:**
1. **Width Transition:** 56px ↔ 360px (200ms)
2. **Content Fade:** Opacity 0 → 1 (150ms)
3. **Folder Collapse:** Height auto → 0 (200ms)
4. **Workspace Scroll:** Spring animation with scroll buttons
5. **Media Player:** Slide up from bottom (y: 20 → 0)

**Framer Motion Implementation:**
```typescript
<motion.div
  initial={{ opacity: 0, y: 20 }}
  animate={{ opacity: 1, y: 0 }}
  exit={{ opacity: 0 }}
  transition={{ duration: 0.15 }}
>
  {/* Sidebar content */}
</motion.div>

// Folder collapse
<AnimatePresence>
  {!folder.collapsed && (
    <motion.div
      initial={{ height: 0, opacity: 0 }}
      animate={{ height: "auto", opacity: 1 }}
      exit={{ height: 0, opacity: 0 }}
      transition={{ duration: 0.2 }}
    >
      {/* Folder contents */}
    </motion.div>
  )}
</AnimatePresence>
```

**Interactive Elements:**
- Workspace icons with scale on hover (1.0 → 1.25)
- Drag constraints with elastic (0.1-0.2)
- Spring physics (stiffness: 400, damping: 30, mass: 0.5)
- Smooth scroll with continuous hold detection

**Key Features:**
```typescript
// Hover animation
<motion.button
  whileHover={{ scale: 1.25 }}
  whileTap={{ scale: 0.95 }}
  drag
  dragConstraints={{ left: 0, right: 0, top: 0, bottom: 0 }}
  dragElastic={0.2}
  transition={{
    type: "spring",
    stiffness: 400,
    damping: 30,
    mass: 0.5,
  }}
>
```

---



#### 4. SCREEN CAROUSEL ⭐⭐⭐ (CRITICAL)
**Location:** `components/screens/screen-carousel.tsx`

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

**Resize Logic:**
```typescript
// Track which sides are being resized
const resizeSidesRef = useRef<{ left: boolean; right: boolean }>({
  left: false,
  right: false,
});

onResize={(_e, direction, _ref, d) => {
  const newWidth = screen.width + d.width;
  const newHeight = screen.height + d.height;
  
  // Track which sides have been resized
  const dir = direction as string;
  const isResizingLeft = dir.includes("left");
  const isResizingRight = dir.includes("right");
  
  if (isResizingLeft) resizeSidesRef.current.left = true;
  if (isResizingRight) resizeSidesRef.current.right = true;
  
  // Determine overall direction
  if (resizeSidesRef.current.left && resizeSidesRef.current.right) {
    setResizeDirection("both");
  } else if (isResizingLeft && !isResizingRight) {
    setResizeDirection("left");
  } else if (isResizingRight && !isResizingLeft) {
    setResizeDirection("right");
  }
  
  // Apply directional gravity
  const leftEdgeOfActive = safeActiveIndex * (containerSize.width + GAP);
  const availableSpace = containerSize.width - newWidth;
  
  let targetX: number;
  if (resizeSidesRef.current.left && resizeSidesRef.current.right) {
    // Resized from both sides - center to show both adjacent screens
    targetX = -leftEdgeOfActive + availableSpace / 2;
  } else if (isResizingLeft && !isResizingRight) {
    // Resizing from left - push screen to the right
    targetX = -leftEdgeOfActive + availableSpace;
  } else if (isResizingRight && !isResizingLeft) {
    // Resizing from right - push screen to the left
    targetX = -leftEdgeOfActive;
  }
  
  x.set(targetX);
}}
```

**Screen Transitions:**
```typescript
<motion.div
  initial={{ scale: 0.95, opacity: 0 }}
  animate={{ scale: 1, opacity: 1 }}
  className="h-full w-full"
>
  {renderScreen(screen)}
</motion.div>
```

**Spring Animation:**
```typescript
useEffect(() => {
  if (!isDragging && !isResizing && containerSize.width > 0) {
    animate(x, targetX, {
      type: "spring",
      stiffness: 300,
      damping: 30,
    });
  }
}, [safeActiveIndex, containerSize.width, isDragging, isResizing, resizeDirection]);
```

---



#### 5. DRAG-AND-DROP SYSTEM ⭐⭐
**Location:** `components/browser/`, `app/page.tsx`

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

**Implementation:**
```typescript
// Drag detection
const {
  attributes,
  listeners,
  setNodeRef,
  transform,
  transition,
  isDragging,
} = useSortable({
  id: tab.id,
  data: { type: "tab", tab },
});

const style = {
  transform: CSS.Transform.toString(transform),
  transition,
  opacity: isDragging ? 0.5 : 1,
};

// Drop indicators
{overId === tab.id && dropPosition === "before" && (
  <div className="absolute -top-1 left-0 right-0 z-[1000000000000000000000000] flex items-center">
    <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
    <div className="bg-primary h-0.5 flex-1" />
  </div>
)}
```

**Collision Detection:** closestCenter algorithm

---

#### 6. MACOS-STYLE DOCK ⭐⭐⭐ (NEW - CRITICAL)
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

**Visual Design:**
```
┌─────────────────────────────────────────────────┐
│  [📄] [📊] [⚙️] [🔧] [📁] [+]                   │
│   ↑                                              │
│ Active (larger, glowing)                         │
└─────────────────────────────────────────────────┘
```

**Animations:**
- **Hover Magnification:** Scale 1.0 → 1.3 with spring
- **Active Glow:** Pulsing glow effect
- **Drag Feedback:** Icon follows cursor, others shift
- **Add Animation:** New icon scales in from 0 → 1
- **Remove Animation:** Icon scales out and fades

**Implementation Reference:**
```typescript
// Similar to macOS dock behavior
<motion.div
  whileHover={{ scale: 1.3, y: -8 }}
  whileTap={{ scale: 0.95 }}
  transition={{
    type: "spring",
    stiffness: 400,
    damping: 25,
  }}
>
  <DockIcon screen={screen} />
</motion.div>
```

**Integration:**
- Syncs with screen carousel state
- Click icon → carousel animates to that screen
- Drag icon → reorder screens in carousel
- Active icon matches active screen

---

## Part 4: Development Workflow

### Setting Up the Zed Fork

1. **Fork Zed Repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/zed.git
   cd zed
   
   # ⚠️ CRITICAL: Only use 'just run' on low-end systems
   # DO NOT use: cargo build, cargo test, cargo check
   just run
   ```

2. **Create Animation Demo Workspace**
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

**Important:** The `www/` folder contains the complete website code for AI agents (GPT-5.4) to study and learn from. This makes it easy for agents to understand how animations work before implementing them in GPUI.



3. **Understand Zed's Current Architecture**

Before replacing components, study existing code:

```rust
// Study these files in Zed codebase:
// crates/workspace/src/workspace.rs - Main workspace structure
// crates/workspace/src/pane.rs - Tab/pane system
// crates/workspace/src/sidebar.rs - Current sidebar
// crates/editor/src/editor.rs - Editor component
```

**Key Questions:**
- How does Zed's sidebar communicate with workspace?
- How are tabs/panes managed?
- Where is the tab bar rendered?
- How do we inject our dock at the top?

4. **Create Demo Tab (Like Onboarding)**

Create a new tab that displays all animations with dimensions:

```rust
// crates/animation_demo/src/demo_tab.rs

use gpui::*;

pub struct AnimationDemoTab {
    active_demo: DemoType,
}

enum DemoType {
    FridayBorder,
    HelloGlow,
    Sidebar,
    Carousel,
    DragDrop,
}

impl Render for AnimationDemoTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.render_toolbar(cx))
            .child(self.render_demo_area(cx))
            .child(self.render_dimensions_panel(cx))
    }
}

impl AnimationDemoTab {
    fn render_dimensions_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .bottom_4()
            .right_4()
            .p_4()
            .bg(rgb(0x1e1e1e))
            .border_1()
            .border_color(rgb(0x3e3e3e))
            .rounded_lg()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xcccccc))
                    .child("Dimensions:")
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(format!("Width: {}px", self.get_current_width()))
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(format!("Height: {}px", self.get_current_height()))
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(format!("Animation: {}", self.get_animation_state()))
            )
    }
}
```

### Side-by-Side Development Process

**Step 1: Copy Web Code to www/ Folder**
- Copy all website code to `zed/www/` folder
- Keep original structure for easy reference
- This allows AI agents (GPT-5.4) to study the code easily
- Add comments linking to GPUI equivalents

**Step 2: Open in Zed**
```bash
# Open both web reference and GPUI implementation
zed www/components/friday.tsx
zed crates/animation_demo/src/friday.rs
```

**Step 3: Implement with Live Preview**
- Write GPUI code in `friday.rs`
- Reference web code in adjacent pane
- Test in demo tab with live dimensions display
- Iterate until behavior matches

**Step 4: Validate**
- Compare visual output
- Check animation timing
- Verify dimensions match
- Test performance (120 FPS target)

---

### Replacing Zed's Core Components

**Step 1: Replace Workspace Sidebar**

```rust
// crates/workspace/src/workspace.rs

// OLD: Zed's default sidebar
// self.left_sidebar = Some(DefaultSidebar::new(cx));

// NEW: Our custom sidebar
use animation_demo::sidebar::CustomSidebar;
self.left_sidebar = Some(CustomSidebar::new(cx));
```

**Step 2: Replace Tab System with Carousel**

```rust
// crates/workspace/src/pane.rs

// OLD: Simple tab bar
// self.render_tab_bar(cx)

// NEW: Screen carousel
use animation_demo::carousel::ScreenCarousel;
self.render_screen_carousel(cx)
```

**Step 3: Add macOS Dock at Top**

```rust
// crates/workspace/src/workspace.rs

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.render_macos_dock(cx))  // NEW: Top dock
            .child(self.render_main_content(cx))
    }
    
    fn render_macos_dock(&self, cx: &mut Context<Self>) -> impl IntoElement {
        use animation_demo::macos_dock::MacOSDock;
        MacOSDock::new(self.screens.clone(), self.active_screen_index)
    }
}
```

**Step 4: Connect Dock to Carousel**

```rust
// Dock click handler
impl MacOSDock {
    fn on_icon_click(&mut self, screen_index: usize, cx: &mut Context<Self>) {
        // Emit event to workspace
        cx.emit(WorkspaceEvent::SwitchToScreen(screen_index));
    }
}

// Workspace handles event
impl Workspace {
    fn handle_switch_screen(&mut self, index: usize, cx: &mut Context<Self>) {
        self.carousel.animate_to_screen(index, cx);
    }
}
```

---

## Part 5: GPUI Implementation Strategies

### Strategy 1: Gradient Animations (Friday, Hello Glow)

**Challenge:** CSS gradient animations with blur

**GPUI Approach:**

1. **Custom Shader for Gradient:**
```rust
// Metal shader for animated gradient
fragment float4 gradient_fragment(
    float4 position [[stage_in]],
    constant GradientUniforms &uniforms [[buffer(0)]]
) {
    float2 uv = position.xy / uniforms.size;
    float offset = uniforms.time * 0.1;
    float hue = fmod(uv.x + offset, 1.0);
    float3 color = hsl_to_rgb(float3(hue * 360.0, 0.85, 0.55));
    return float4(color, 1.0);
}
```

2. **Rust Implementation:**
```rust
struct FridayBorder {
    time: f32,
    phase: BorderPhase,
    border_thickness: f32,
}

enum BorderPhase {
    Idle,
    Entering,
    Active,
    Exiting,
}

impl Element for FridayBorder {
    fn paint(&mut self, origin: Point, size: Size, scene: &mut Scene) {
        self.time += delta_time;
        
        // Push custom shader primitive
        scene.push_custom_shader(CustomShader {
            shader_type: ShaderType::GradientBorder,
            bounds: Bounds { origin, size },
            uniforms: vec![
                ("time", self.time),
                ("border_width", 10.0),
                ("blur_sigma", 8.0),
            ],
        });
    }
}
```



3. **Blur Implementation:**
```rust
// Use separable Gaussian blur (horizontal + vertical passes)
fn gaussian_blur(texture: Texture, sigma: f32) -> Texture {
    let horizontal = blur_pass(texture, vec2(1, 0), sigma);
    blur_pass(horizontal, vec2(0, 1), sigma)
}
```

---

### Strategy 2: Layout Animations (Sidebar, Carousel)

**Challenge:** Smooth layout transitions with spring physics

**GPUI Approach:**

1. **Animated Properties:**
```rust
struct AnimatedValue<T> {
    current: T,
    target: T,
    velocity: T,
    config: SpringConfig,
}

struct SpringConfig {
    stiffness: f32,
    damping: f32,
    mass: f32,
}

impl<T: Lerp> AnimatedValue<T> {
    fn update(&mut self, delta_time: f32) {
        let displacement = self.target - self.current;
        let spring_force = displacement * self.config.stiffness;
        let damping_force = self.velocity * self.config.damping;
        
        let acceleration = (spring_force - damping_force) / self.config.mass;
        self.velocity += acceleration * delta_time;
        self.current += self.velocity * delta_time;
    }
    
    fn is_settled(&self) -> bool {
        (self.current - self.target).abs() < 0.5 && self.velocity.abs() < 0.5
    }
}
```

2. **Sidebar Implementation:**
```rust
struct AnimatedSidebar {
    width: AnimatedValue<f32>,
    expanded: bool,
}

impl AnimatedSidebar {
    fn new() -> Self {
        Self {
            width: AnimatedValue::new(56.0, 56.0, SpringConfig {
                stiffness: 300.0,
                damping: 30.0,
                mass: 1.0,
            }),
            expanded: false,
        }
    }
    
    fn toggle(&mut self) {
        self.expanded = !self.expanded;
        self.width.target = if self.expanded { 360.0 } else { 56.0 };
    }
}

impl Render for AnimatedSidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.width.update(cx.frame_time());
        
        if !self.width.is_settled() {
            cx.request_frame();
        }
        
        div()
            .w(px(self.width.current))
            .h_full()
            .bg(rgb(0x1e1e1e))
            .child(/* sidebar content */)
    }
}
```



3. **Carousel with Directional Gravity:**
```rust
struct ScreenCarousel {
    screens: Vec<Screen>,
    active_index: usize,
    position: AnimatedValue<f32>,
    resize_direction: Option<ResizeDirection>,
}

enum ResizeDirection {
    Left,
    Right,
    Both,
}

impl ScreenCarousel {
    fn update_position(&mut self) {
        let active_screen = &self.screens[self.active_index];
        let container_width = self.container_width;
        let screen_width = active_screen.width.current;
        let available_space = container_width - screen_width;
        
        let target_x = match self.resize_direction {
            Some(ResizeDirection::Both) => {
                // Center to show both neighbors
                -self.active_index as f32 * container_width + available_space / 2.0
            }
            Some(ResizeDirection::Left) => {
                // Push right to reveal left neighbor
                -self.active_index as f32 * container_width + available_space
            }
            Some(ResizeDirection::Right) => {
                // Push left to reveal right neighbor
                -self.active_index as f32 * container_width
            }
            None => {
                // Default: push left
                -self.active_index as f32 * container_width
            }
        };
        
        self.position.target = target_x;
    }
}
```

---

### Strategy 3: Drag-and-Drop System

**Challenge:** Complex interaction with visual feedback

**GPUI Approach:**

1. **Event Handling:**
```rust
struct DraggableTab {
    id: String,
    dragging: bool,
    drag_offset: Point,
}

impl Element for DraggableTab {
    fn handle_event(&mut self, event: &Event, cx: &mut Context) -> EventResult {
        match event {
            Event::MouseDown(e) if self.bounds.contains(e.position) => {
                self.dragging = true;
                self.drag_offset = e.position - self.bounds.origin;
                EventResult::Handled
            }
            Event::MouseMove(e) if self.dragging => {
                let new_position = e.position - self.drag_offset;
                cx.emit(DragEvent::Move { id: self.id, position: new_position });
                EventResult::Handled
            }
            Event::MouseUp(_) if self.dragging => {
                self.dragging = false;
                cx.emit(DragEvent::End { id: self.id });
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }
}
```

2. **Drop Indicators:**
```rust
struct DropIndicator {
    position: DropPosition,
    bounds: Bounds,
}

enum DropPosition {
    Before,
    After,
    Inside,
}

impl Element for DropIndicator {
    fn paint(&mut self, origin: Point, size: Size, scene: &mut Scene) {
        match self.position {
            DropPosition::Before => {
                // Blue dot + line at top
                scene.push_circle(Circle {
                    center: Point::new(origin.x, origin.y),
                    radius: 4.0,
                    color: rgb(0x0000ff),
                });
                scene.push_rect(Rect {
                    bounds: Bounds::new(origin, Size::new(size.width, 2.0)),
                    color: rgb(0x0000ff),
                });
            }
            DropPosition::After => {
                // Blue dot + line at bottom
                scene.push_circle(Circle {
                    center: Point::new(origin.x, origin.y + size.height),
                    radius: 4.0,
                    color: rgb(0x0000ff),
                });
                scene.push_rect(Rect {
                    bounds: Bounds::new(
                        Point::new(origin.x, origin.y + size.height - 2.0),
                        Size::new(size.width, 2.0)
                    ),
                    color: rgb(0x0000ff),
                });
            }
            DropPosition::Inside => {
                // Highlight entire area
                scene.push_rect(Rect {
                    bounds: Bounds::new(origin, size),
                    color: rgba(0x00, 0x00, 0xff, 0.1),
                });
            }
        }
    }
}
```

---



## Part 6: Implementation Roadmap

### Phase 1: Foundation & Architecture Study (Week 1-2)

**Goal:** Set up infrastructure and understand Zed's architecture

1. **Fork Zed and Set Up Workspace**
   - Fork Zed repository
   - Create `animation_demo` crate
   - Copy web reference code to `reference/` folder
   - Set up build system

2. **Study Zed's Architecture**
   - Read `crates/workspace/src/workspace.rs`
   - Understand sidebar implementation
   - Study pane/tab system
   - Map out component hierarchy
   - Identify integration points

3. **Create Demo Tab** (See Part 12 for detailed implementation)
   - Implement untitled tab like onboarding
   - Add toolbar for switching between demos
   - Add dimensions panel (live display)
   - Add performance metrics (FPS counter)
   - Make it default screen on startup

4. **Create Animation Module**
   ```rust
   // crates/animation_demo/src/animation/mod.rs
   pub mod easing;
   pub mod spring;
   pub mod animated_value;
   ```

5. **Implement Core Types**
   - `AnimatedValue<T>` for any animatable property
   - Easing functions (linear, ease-in/out, cubic)
   - Spring physics calculator

**Deliverables:**
- [ ] Zed fork with animation_demo crate
- [ ] Architecture documentation (how Zed's UI works)
- [ ] Demo tab with dimensions display
- [ ] Animation module structure
- [ ] Basic spring physics
- [ ] Simple test animation (fade in/out)

---

### Phase 2: Hello Glow Effect (Week 3)

**Goal:** Implement gradient animation with blur

1. **Gradient Shader**
   - Write Metal shader for HSL gradient
   - Implement time-based animation
   - Add background-position offset

2. **Blur Implementation**
   - Separable Gaussian blur shader
   - Dual-layer blur (6px + 18px)
   - Optimize for performance

3. **Hello Glow Component**
   - Integrate shader into GPUI element
   - Match web dimensions (border-radius, inset)
   - Add to demo tab

**Deliverables:**
- [ ] Gradient shader (Metal)
- [ ] Blur shader
- [ ] Hello Glow component
- [ ] Performance: 120 FPS maintained
- [ ] Visual match with web version

---

### Phase 3: Replace Workspace Sidebar (Week 4-5)

**Goal:** Replace Zed's sidebar with our custom implementation

1. **Build Custom Sidebar Component**
   - Animated width property (56px ↔ 360px)
   - Content fade in/out
   - Spring transition (stiffness: 400, damping: 30)
   - Logo container with drag-drop
   - Workspace switcher
   - Folder system

2. **Integrate with Zed's Workspace**
   - Replace `DefaultSidebar` with `CustomSidebar`
   - Hook into workspace state
   - Handle events (workspace switch, folder toggle)
   - Maintain Zed's existing functionality

3. **Implement All Features**
   - Workspace icon hover (scale 1.0 → 1.25)
   - Drag constraints with elastic
   - Scroll buttons with continuous hold
   - Folder collapse animations
   - Media player slide animation

4. **Test Integration**
   - Verify all Zed features still work
   - Test with multiple workspaces
   - Validate state persistence

**Deliverables:**
- [ ] Custom sidebar component (complete)
- [ ] Integrated into Zed workspace
- [ ] All animations working
- [ ] Zed functionality preserved
- [ ] Performance: 120 FPS maintained

---



### Phase 4: Replace Tab System with Carousel (Week 6-8)

**Goal:** Replace Zed's tabs with screen carousel system

1. **Build Screen Carousel Component**
   - Screen layout with gap
   - Spring-based positioning
   - Active screen highlighting
   - Resize system with drag handles
   - Directional gravity logic
   - Circular wrapping

2. **Replace Zed's Tab Bar**
   - Remove default tab bar rendering
   - Integrate carousel into pane system
   - Map Zed's editors to carousel screens
   - Handle editor switching

3. **Maintain Editor Functionality**
   - File opening → new screen
   - File closing → remove screen
   - Editor state preservation
   - Split view compatibility

4. **Test Integration**
   - Open/close files
   - Switch between editors
   - Resize screens
   - Verify all editor features work

**Deliverables:**
- [ ] Screen carousel component (complete)
- [ ] Integrated into Zed's pane system
- [ ] Replaces default tab bar
- [ ] All editor functionality preserved
- [ ] Directional gravity working
- [ ] Performance: 120 FPS maintained

---

### Phase 5: Add macOS Dock (Week 9-10)

**Goal:** Add top dock to control screen carousel

1. **Build macOS Dock Component**
   - Dock icon rendering
   - Hover magnification (scale 1.0 → 1.3)
   - Active indicator (glow effect)
   - Drag to reorder
   - Add/remove buttons

2. **Integrate with Workspace**
   - Add dock to top of workspace
   - Connect to carousel state
   - Handle icon clicks → switch screens
   - Handle drag → reorder screens
   - Sync active state

3. **Animations**
   - Hover spring animation
   - Active glow pulse
   - Drag feedback
   - Add/remove transitions

4. **Test Integration**
   - Click icons to switch
   - Drag to reorder
   - Add new screens
   - Remove screens
   - Verify sync with carousel

**Deliverables:**
- [ ] macOS dock component (complete)
- [ ] Integrated at top of workspace
- [ ] Connected to carousel
- [ ] All interactions working
- [ ] Animations smooth
- [ ] Performance: 120 FPS maintained

---

### Phase 6: Friday Border Effect (Week 11-12)

**Goal:** Implement rainbow border with scroll animation

1. **Border Rendering**
   - Rainbow gradient shader
   - 10px thickness
   - Clip-path for diagonal corners
   - Dual glow layers

2. **Animation Sequence**
   - Bottom border slides in
   - Right border slides in
   - Left border slides in (with delay)
   - Top border appears
   - All borders visible

3. **Scroll Animation**
   - Smooth scroll down (10% viewport)
   - Spring bounce back to origin
   - Multiple bounce effect
   - Spring physics (stiffness: 0.03, damping: 0.2)

4. **Activation Button**
   - Toggle button
   - State management
   - Smooth transitions

**Deliverables:**
- [ ] Rainbow gradient border shader
- [ ] Glow effects (8px + 12px blur)
- [ ] Slide-in animation sequence
- [ ] Scroll animation with spring physics
- [ ] Activation button
- [ ] Performance: 120 FPS maintained

---

### Phase 7: Drag-and-Drop (Week 13-14)

**Goal:** Full drag-and-drop system

1. **Drag Detection**
   - Mouse event handling
   - Drag threshold (8px)
   - Cursor changes (grab/grabbing)

2. **Visual Feedback**
   - Drag overlay (50% opacity)
   - Drop indicators (dots + lines)
   - Hover highlights

3. **Drop Logic**
   - Collision detection (closestCenter)
   - Position calculation (before/after/inside)
   - State updates
   - Smooth reordering

4. **Tab/Folder Organization**
   - Tab reordering
   - Folder creation
   - Tab → Folder
   - Folder → Tab

**Deliverables:**
- [ ] Drag system with threshold
- [ ] Drop indicators (blue dots + lines)
- [ ] Tab reordering
- [ ] Folder organization
- [ ] Logo container drop target
- [ ] Performance: 120 FPS maintained

---

---

## Part 7: Integration Testing

### Testing Zed Functionality

**Critical Tests:**

1. **Sidebar Integration**
   - [ ] Open/close sidebar
   - [ ] Switch workspaces
   - [ ] Create/delete folders
   - [ ] Drag tabs to folders
   - [ ] All Zed sidebar features work

2. **Carousel Integration**
   - [ ] Open files → new screens
   - [ ] Close files → remove screens
   - [ ] Switch between editors
   - [ ] Editor state preserved
   - [ ] Split view works

3. **Dock Integration**
   - [ ] Click icons → switch screens
   - [ ] Drag icons → reorder
   - [ ] Add screen → opens file picker
   - [ ] Remove screen → closes editor
   - [ ] Sync with carousel

4. **Overall Integration**
   - [ ] All three components work together
   - [ ] No conflicts with Zed features
   - [ ] Performance maintained
   - [ ] State persistence works

---

## Part 8: Testing & Validation

### Visual Validation

**Side-by-Side Comparison:**
1. Open web version in browser
2. Open GPUI demo tab in Zed
3. Compare animations frame-by-frame
4. Verify dimensions match exactly
5. Check timing and easing

**Dimensions Panel:**
```rust
// Display in demo tab
- Width: 360px (target: 360px)
- Height: 800px
- Animation: Expanding (75% complete)
- FPS: 120
- Frame time: 8.2ms
```

### Performance Testing

**Metrics to Track:**
- Frame rate (target: 120 FPS)
- Frame time (target: <8.33ms)
- Memory usage
- GPU utilization

**Profiling:**
```rust
use std::time::Instant;

let start = Instant::now();
// ... animation code ...
let duration = start.elapsed();
if duration.as_millis() > 2 {
    eprintln!("Animation update took {}ms", duration.as_millis());
}
```

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spring_animation() {
        let mut value = AnimatedValue::new(0.0, 100.0, SpringConfig::default());
        
        // Simulate 1 second
        for _ in 0..60 {
            value.update(1.0 / 60.0);
        }
        
        assert!((value.current - 100.0).abs() < 1.0);
    }
}
```

---



## Part 9: Performance Optimization

### Frame Budget

**Target:** 120 FPS = 8.33ms per frame

**Budget Breakdown:**
- Layout: 2ms
- Animation updates: 1ms
- Painting: 2ms
- GPU rendering: 3ms
- Buffer: 0.33ms

### Optimization Strategies

1. **Minimize Allocations**
   - Pre-allocate animation buffers
   - Reuse vectors and data structures
   - Use object pools

2. **Batch GPU Operations**
   - Group similar primitives
   - Minimize draw calls
   - Use instanced rendering

3. **Cull Off-Screen Elements**
   - Skip layout/paint for invisible elements
   - Use viewport culling
   - Lazy evaluation

4. **Cache Expensive Calculations**
   - Pre-compute easing curves
   - Cache gradient textures
   - Memoize layout results

5. **Use GPU for Heavy Work**
   - Blur on GPU, not CPU
   - Gradient generation in shaders
   - All effects on GPU

6. **Throttle Updates**
   - Only update when values change
   - Use dirty flags
   - Request frame only when animating

---

## Part 10: Zed Codebase Deep Dive

### Critical Files to Study

**Workspace Architecture:**
```
crates/workspace/src/
├── workspace.rs          # Main workspace container
├── pane.rs              # Tab/pane management
├── pane_group.rs        # Split view logic
├── sidebar.rs           # Current sidebar (to replace)
├── toolbar.rs           # Top toolbar
└── notifications.rs     # Notification system
```

**GPUI Core:**
```
crates/gpui/src/
├── window.rs            # Window management
├── element.rs           # Element trait
├── scene.rs             # Scene graph
├── platform/            # Platform-specific code
│   ├── mac/            # macOS Metal rendering
│   └── linux/          # Linux Vulkan rendering
└── animation.rs         # Animation utilities (if exists)
```

### Studying Zed's Sidebar

**File:** `crates/workspace/src/sidebar.rs`

```rust
// Current Zed sidebar structure (simplified)
pub struct Sidebar {
    side: Side,
    items: Vec<Box<dyn SidebarItem>>,
    active_item: Option<usize>,
    width: f32,
    open: bool,
}

impl Sidebar {
    pub fn new(side: Side) -> Self {
        Self {
            side,
            items: Vec::new(),
            active_item: None,
            width: 240.0,  // Default width
            open: true,
        }
    }
    
    pub fn toggle(&mut self, cx: &mut ViewContext<Self>) {
        self.open = !self.open;
        cx.notify();
    }
}

impl Render for Sidebar {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .w(px(if self.open { self.width } else { 0.0 }))
            .h_full()
            .bg(cx.theme().colors().panel_background)
            .children(self.items.iter().map(|item| item.render(cx)))
    }
}
```

**Our Replacement Strategy:**
```rust
// crates/animation_demo/src/sidebar.rs

pub struct CustomSidebar {
    // Animated width instead of instant toggle
    width: AnimatedValue<f32>,
    expanded: bool,
    
    // Our custom features
    workspaces: Vec<Workspace>,
    active_workspace: usize,
    folders: Vec<Folder>,
    tabs: Vec<Tab>,
    logos: Vec<Logo>,
    
    // Animation state
    folder_animations: HashMap<String, AnimatedValue<f32>>,
}

impl CustomSidebar {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            width: AnimatedValue::new(56.0, 56.0, SpringConfig {
                stiffness: 400.0,
                damping: 30.0,
                mass: 1.0,
            }),
            expanded: false,
            workspaces: vec![Workspace::default()],
            active_workspace: 0,
            folders: Vec::new(),
            tabs: Vec::new(),
            logos: Vec::new(),
            folder_animations: HashMap::new(),
        }
    }
    
    pub fn toggle(&mut self, cx: &mut ViewContext<Self>) {
        self.expanded = !self.expanded;
        self.width.target = if self.expanded { 360.0 } else { 56.0 };
        cx.notify();
    }
}

impl Render for CustomSidebar {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // Update animations
        self.width.update(cx.frame_time());
        
        // Request next frame if animating
        if !self.width.is_settled() {
            cx.request_frame();
        }
        
        div()
            .w(px(self.width.current))
            .h_full()
            .bg(rgb(0x1e1e1e))
            .child(self.render_header(cx))
            .child(self.render_workspaces(cx))
            .child(self.render_folders(cx))
            .child(self.render_tabs(cx))
    }
}
```

### Studying Zed's Pane System

**File:** `crates/workspace/src/pane.rs`

```rust
// Current Zed pane structure (simplified)
pub struct Pane {
    items: Vec<Box<dyn ItemHandle>>,
    active_item_index: usize,
    tab_bar_visible: bool,
}

impl Pane {
    pub fn render_tab_bar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .h(px(32.0))
            .bg(cx.theme().colors().tab_bar_background)
            .children(
                self.items.iter().enumerate().map(|(i, item)| {
                    self.render_tab(i, item, cx)
                })
            )
    }
    
    fn render_tab(&self, index: usize, item: &Box<dyn ItemHandle>, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let is_active = index == self.active_item_index;
        
        div()
            .px_3()
            .py_2()
            .bg(if is_active {
                cx.theme().colors().tab_active_background
            } else {
                cx.theme().colors().tab_inactive_background
            })
            .child(item.tab_content(cx))
    }
}
```

**Our Carousel Replacement:**
```rust
// crates/animation_demo/src/carousel.rs

pub struct ScreenCarousel {
    screens: Vec<Screen>,
    active_index: usize,
    position: AnimatedValue<f32>,
    container_width: f32,
    gap: f32,
    resize_direction: Option<ResizeDirection>,
}

pub struct Screen {
    id: String,
    item: Box<dyn ItemHandle>,  // Reuse Zed's ItemHandle
    width: AnimatedValue<f32>,
    height: AnimatedValue<f32>,
}

impl ScreenCarousel {
    pub fn from_pane_items(items: Vec<Box<dyn ItemHandle>>, cx: &mut ViewContext<Self>) -> Self {
        let screens = items.into_iter().map(|item| {
            Screen {
                id: item.item_id().to_string(),
                item,
                width: AnimatedValue::new(800.0, 800.0, SpringConfig::default()),
                height: AnimatedValue::new(600.0, 600.0, SpringConfig::default()),
            }
        }).collect();
        
        Self {
            screens,
            active_index: 0,
            position: AnimatedValue::new(0.0, 0.0, SpringConfig {
                stiffness: 300.0,
                damping: 30.0,
                mass: 1.0,
            }),
            container_width: 0.0,
            gap: 8.0,
            resize_direction: None,
        }
    }
    
    pub fn add_item(&mut self, item: Box<dyn ItemHandle>, cx: &mut ViewContext<Self>) {
        let screen = Screen {
            id: item.item_id().to_string(),
            item,
            width: AnimatedValue::new(self.container_width, self.container_width, SpringConfig::default()),
            height: AnimatedValue::new(600.0, 600.0, SpringConfig::default()),
        };
        self.screens.push(screen);
        cx.notify();
    }
    
    pub fn remove_item(&mut self, index: usize, cx: &mut ViewContext<Self>) {
        if index < self.screens.len() {
            self.screens.remove(index);
            if self.active_index >= self.screens.len() && self.active_index > 0 {
                self.active_index -= 1;
            }
            cx.notify();
        }
    }
}

impl Render for ScreenCarousel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // Update all animations
        self.position.update(cx.frame_time());
        for screen in &mut self.screens {
            screen.width.update(cx.frame_time());
            screen.height.update(cx.frame_time());
        }
        
        // Request frame if any animation is active
        if !self.position.is_settled() || 
           self.screens.iter().any(|s| !s.width.is_settled() || !s.height.is_settled()) {
            cx.request_frame();
        }
        
        div()
            .flex()
            .h_full()
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .style(|style| {
                        style.transform = Some(format!("translateX({}px)", self.position.current));
                    })
                    .children(
                        self.screens.iter().enumerate().map(|(i, screen)| {
                            self.render_screen(i, screen, cx)
                        })
                    )
            )
    }
    
    fn render_screen(&self, index: usize, screen: &Screen, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let is_active = index == self.active_index;
        
        div()
            .w(px(screen.width.current))
            .h(px(screen.height.current))
            .mr(px(self.gap))
            .border_2()
            .border_color(if is_active { rgb(0x0000ff) } else { rgb(0x3e3e3e) })
            .rounded_lg()
            .overflow_hidden()
            .child(screen.item.to_any().downcast_ref::<Editor>().unwrap().render(cx))
    }
}
```

### Integrating into Workspace

**File:** `crates/workspace/src/workspace.rs`

```rust
// Current Zed workspace structure (simplified)
pub struct Workspace {
    left_sidebar: Option<View<Sidebar>>,
    right_sidebar: Option<View<Sidebar>>,
    center: View<PaneGroup>,
    // ... other fields
}

impl Workspace {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            left_sidebar: Some(cx.new_view(|cx| Sidebar::new(Side::Left))),
            right_sidebar: Some(cx.new_view(|cx| Sidebar::new(Side::Right))),
            center: cx.new_view(|cx| PaneGroup::new()),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .child(self.left_sidebar.as_ref().map(|s| s.clone()))
            .child(self.center.clone())
            .child(self.right_sidebar.as_ref().map(|s| s.clone()))
    }
}
```

**Our Modified Workspace:**
```rust
// Modify crates/workspace/src/workspace.rs

use animation_demo::sidebar::CustomSidebar;
use animation_demo::carousel::ScreenCarousel;
use animation_demo::macos_dock::MacOSDock;

pub struct Workspace {
    // Replace left sidebar
    left_sidebar: Option<View<CustomSidebar>>,  // Changed type
    right_sidebar: Option<View<Sidebar>>,
    
    // Replace center pane group with carousel
    center: View<ScreenCarousel>,  // Changed type
    
    // Add macOS dock
    top_dock: View<MacOSDock>,  // New field
    
    // ... other fields
}

impl Workspace {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let carousel = cx.new_view(|cx| ScreenCarousel::new(cx));
        let dock = cx.new_view(|cx| MacOSDock::new(cx));
        
        Self {
            left_sidebar: Some(cx.new_view(|cx| CustomSidebar::new(cx))),
            right_sidebar: Some(cx.new_view(|cx| Sidebar::new(Side::Right))),
            center: carousel.clone(),
            top_dock: dock.clone(),
        }
    }
    
    // Connect dock to carousel
    pub fn handle_dock_click(&mut self, screen_index: usize, cx: &mut ViewContext<Self>) {
        self.center.update(cx, |carousel, cx| {
            carousel.switch_to_screen(screen_index, cx);
        });
    }
}

impl Render for Workspace {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.top_dock.clone())  // Add dock at top
            .child(
                div()
                    .flex()
                    .flex_1()
                    .child(self.left_sidebar.as_ref().map(|s| s.clone()))
                    .child(self.center.clone())
                    .child(self.right_sidebar.as_ref().map(|s| s.clone()))
            )
    }
}
```

### GPUI Animation Utilities

**Study these GPUI examples:**

```rust
// From crates/gpui/examples/animation.rs (if exists)

use gpui::*;

// Example: Rotating SVG animation
struct RotatingIcon {
    rotation: f32,
}

impl Render for RotatingIcon {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        self.rotation += 2.0; // Degrees per frame
        if self.rotation >= 360.0 {
            self.rotation -= 360.0;
        }
        
        cx.request_frame(); // Keep animating
        
        svg()
            .path("M10 10 L20 20 L10 30 Z")
            .size(px(40.0))
            .style(|style| {
                style.transform = Some(format!("rotate({}deg)", self.rotation));
            })
    }
}
```

**Create our animation utilities:**

```rust
// crates/animation_demo/src/animation/mod.rs

pub mod easing;
pub mod spring;
pub mod animated_value;

// Re-export commonly used types
pub use animated_value::AnimatedValue;
pub use spring::SpringConfig;
pub use easing::*;

// Utility functions
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

// Trait for types that can be interpolated
pub trait Lerp {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        lerp(*self, *other, t)
    }
}

impl Lerp for Point {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Point {
            x: lerp(self.x, other.x, t),
            y: lerp(self.y, other.y, t),
        }
    }
}

impl Lerp for Size {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Size {
            width: lerp(self.width, other.width, t),
            height: lerp(self.height, other.height, t),
        }
    }
}
```

### Metal Shader Examples

**Study Zed's existing shaders:**

```
crates/gpui/src/platform/mac/shaders/
├── quad.metal           # Rectangle rendering
├── shadow.metal         # Drop shadows
├── text.metal           # Text rendering
└── path.metal           # Path rendering (if exists)
```

**Our custom gradient shader:**

```metal
// crates/animation_demo/shaders/gradient.metal

#include <metal_stdlib>
using namespace metal;

struct GradientUniforms {
    float time;
    float2 size;
    float border_width;
    float blur_sigma;
};

struct VertexOut {
    float4 position [[position]];
    float2 uv;
};

vertex VertexOut gradient_vertex(
    uint vertex_id [[vertex_id]],
    constant float2 *vertices [[buffer(0)]],
    constant GradientUniforms &uniforms [[buffer(1)]]
) {
    VertexOut out;
    float2 pos = vertices[vertex_id];
    out.position = float4(pos, 0.0, 1.0);
    out.uv = (pos + 1.0) * 0.5; // Convert from [-1,1] to [0,1]
    return out;
}

// HSL to RGB conversion
float3 hsl_to_rgb(float3 hsl) {
    float h = hsl.x / 60.0;
    float s = hsl.y;
    float l = hsl.z;
    
    float c = (1.0 - abs(2.0 * l - 1.0)) * s;
    float x = c * (1.0 - abs(fmod(h, 2.0) - 1.0));
    float m = l - c / 2.0;
    
    float3 rgb;
    if (h < 1.0) rgb = float3(c, x, 0);
    else if (h < 2.0) rgb = float3(x, c, 0);
    else if (h < 3.0) rgb = float3(0, c, x);
    else if (h < 4.0) rgb = float3(0, x, c);
    else if (h < 5.0) rgb = float3(x, 0, c);
    else rgb = float3(c, 0, x);
    
    return rgb + m;
}

fragment float4 gradient_fragment(
    VertexOut in [[stage_in]],
    constant GradientUniforms &uniforms [[buffer(0)]]
) {
    // Animate gradient position
    float offset = uniforms.time * 0.1;
    float hue = fmod(in.uv.x + offset, 1.0) * 360.0;
    
    // Convert HSL to RGB
    float3 color = hsl_to_rgb(float3(hue, 0.85, 0.55));
    
    return float4(color, 1.0);
}

// Gaussian blur shader
fragment float4 blur_fragment(
    VertexOut in [[stage_in]],
    texture2d<float> input_texture [[texture(0)]],
    constant GradientUniforms &uniforms [[buffer(0)]]
) {
    constexpr sampler s(mag_filter::linear, min_filter::linear);
    
    float sigma = uniforms.blur_sigma;
    int kernel_size = int(ceil(sigma * 3.0));
    
    float4 color = float4(0.0);
    float total_weight = 0.0;
    
    for (int i = -kernel_size; i <= kernel_size; i++) {
        float2 offset = float2(float(i) / uniforms.size.x, 0.0);
        float weight = exp(-float(i * i) / (2.0 * sigma * sigma));
        color += input_texture.sample(s, in.uv + offset) * weight;
        total_weight += weight;
    }
    
    return color / total_weight;
}
```

### Event Handling in GPUI

**Study Zed's event system:**

```rust
// From crates/gpui/src/window.rs

pub enum Event {
    MouseDown(MouseDownEvent),
    MouseUp(MouseUpEvent),
    MouseMove(MouseMoveEvent),
    KeyDown(KeyDownEvent),
    KeyUp(KeyUpEvent),
    // ... other events
}

pub trait EventHandler {
    fn handle_event(&mut self, event: &Event, cx: &mut Context) -> EventResult;
}

pub enum EventResult {
    Handled,
    Ignored,
}
```

**Our drag-and-drop implementation:**

```rust
// crates/animation_demo/src/drag_drop.rs

use gpui::*;

pub struct DraggableElement {
    id: String,
    bounds: Bounds,
    dragging: bool,
    drag_start: Option<Point>,
    drag_offset: Point,
}

impl DraggableElement {
    pub fn new(id: String) -> Self {
        Self {
            id,
            bounds: Bounds::default(),
            dragging: false,
            drag_start: None,
            drag_offset: Point::zero(),
        }
    }
}

impl Element for DraggableElement {
    fn layout(&mut self, constraint: SizeConstraint, cx: &mut LayoutContext) -> Size {
        // Layout logic
        Size::new(100.0, 100.0)
    }
    
    fn paint(&mut self, origin: Point, size: Size, cx: &mut PaintContext) {
        self.bounds = Bounds::new(origin, size);
        
        // Render element
        cx.paint_quad(Quad {
            bounds: self.bounds,
            background: if self.dragging {
                Some(rgba(0x0000ff, 0.5))
            } else {
                Some(rgb(0x0000ff))
            },
            border_color: rgb(0x000000),
            border_widths: EdgeSizes::all(1.0),
            corner_radii: CornerRadii::all(4.0),
        });
    }
    
    fn handle_event(&mut self, event: &Event, cx: &mut EventContext) -> EventResult {
        match event {
            Event::MouseDown(e) if self.bounds.contains(e.position) => {
                self.dragging = true;
                self.drag_start = Some(e.position);
                self.drag_offset = e.position - self.bounds.origin;
                cx.notify();
                EventResult::Handled
            }
            Event::MouseMove(e) if self.dragging => {
                let new_origin = e.position - self.drag_offset;
                cx.emit(DragEvent::Move {
                    id: self.id.clone(),
                    position: new_origin,
                });
                cx.notify();
                EventResult::Handled
            }
            Event::MouseUp(_) if self.dragging => {
                self.dragging = false;
                self.drag_start = None;
                cx.emit(DragEvent::End {
                    id: self.id.clone(),
                });
                cx.notify();
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }
}

pub enum DragEvent {
    Move { id: String, position: Point },
    End { id: String },
}
```

---

## Part 12: Demo Tab Implementation (Animation Showcase)

### Overview

The Demo Tab is a special tab in Zed (like the onboarding/welcome screen) that serves as a testing and showcase environment for all animations. It should be the default screen when Zed opens without a folder.

**Purpose:**
- Test all animations in isolation
- Display live dimensions and metrics
- Switch between different animation demos
- Compare with web reference side-by-side
- Performance monitoring (FPS, frame time)

**Visual Layout:**
```
┌─────────────────────────────────────────────────────────┐
│ [Friday] [Hello Glow] [Sidebar] [Carousel] [Dock] [All]│ ← Toolbar
├─────────────────────────────────────────────────────────┤
│                                                         │
│                                                         │
│              ANIMATION DEMO AREA                        │
│           (Selected animation renders here)             │
│                                                         │
│                                                         │
│                                    ┌──────────────────┐ │
│                                    │ Dimensions Panel │ │
│                                    │ Width: 360px     │ │
│                                    │ Height: 800px    │ │
│                                    │ FPS: 120         │ │
│                                    │ Frame: 8.2ms     │ │
│                                    │ Animation: 75%   │ │
│                                    └──────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

---

### Creating the Demo Tab

#### Step 1: Study Zed's Welcome Screen

**File to study:** `crates/welcome/src/welcome.rs`

Zed's welcome screen is shown when no folder is open. We'll create a similar tab for our animation demos.

```rust
// Study how Zed creates the welcome screen
// crates/welcome/src/welcome.rs

pub struct WelcomeView {
    // ... fields
}

impl Render for WelcomeView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(/* welcome content */)
    }
}

// Register as a serializable item
impl Item for WelcomeView {
    type Event = ();
    
    fn tab_content(&self, _cx: &WindowContext) -> AnyElement {
        Label::new("Welcome").into_any_element()
    }
}
```

#### Step 2: Create Animation Demo Tab

**File:** `crates/animation_demo/src/demo_tab.rs`

```rust
use gpui::*;
use std::time::Instant;

pub struct AnimationDemoTab {
    /// Currently selected demo
    active_demo: DemoType,
    
    /// Animation instances
    friday_border: Option<FridayBorder>,
    hello_glow: Option<HelloGlow>,
    sidebar_demo: Option<SidebarDemo>,
    carousel_demo: Option<CarouselDemo>,
    dock_demo: Option<DockDemo>,
    
    /// Performance metrics
    fps_counter: FpsCounter,
    frame_times: Vec<f32>,
    
    /// Dimensions tracking
    current_width: f32,
    current_height: f32,
    animation_progress: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DemoType {
    FridayBorder,
    HelloGlow,
    Sidebar,
    Carousel,
    Dock,
    All,  // Show all animations at once
}

impl AnimationDemoTab {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            active_demo: DemoType::All,
            friday_border: Some(FridayBorder::new(cx)),
            hello_glow: Some(HelloGlow::new(cx)),
            sidebar_demo: Some(SidebarDemo::new(cx)),
            carousel_demo: Some(CarouselDemo::new(cx)),
            dock_demo: Some(DockDemo::new(cx)),
            fps_counter: FpsCounter::new(),
            frame_times: Vec::with_capacity(120),
            current_width: 0.0,
            current_height: 0.0,
            animation_progress: 0.0,
        }
    }
    
    fn switch_demo(&mut self, demo: DemoType, cx: &mut ViewContext<Self>) {
        self.active_demo = demo;
        
        // Reset animations when switching
        match demo {
            DemoType::FridayBorder => {
                if let Some(friday) = &mut self.friday_border {
                    friday.reset(cx);
                }
            }
            DemoType::HelloGlow => {
                if let Some(glow) = &mut self.hello_glow {
                    glow.reset(cx);
                }
            }
            DemoType::Sidebar => {
                if let Some(sidebar) = &mut self.sidebar_demo {
                    sidebar.reset(cx);
                }
            }
            DemoType::Carousel => {
                if let Some(carousel) = &mut self.carousel_demo {
                    carousel.reset(cx);
                }
            }
            DemoType::Dock => {
                if let Some(dock) = &mut self.dock_demo {
                    dock.reset(cx);
                }
            }
            DemoType::All => {
                // Reset all
                if let Some(friday) = &mut self.friday_border {
                    friday.reset(cx);
                }
                if let Some(glow) = &mut self.hello_glow {
                    glow.reset(cx);
                }
                if let Some(sidebar) = &mut self.sidebar_demo {
                    sidebar.reset(cx);
                }
                if let Some(carousel) = &mut self.carousel_demo {
                    carousel.reset(cx);
                }
                if let Some(dock) = &mut self.dock_demo {
                    dock.reset(cx);
                }
            }
        }
        
        cx.notify();
    }
}

impl Render for AnimationDemoTab {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // Update FPS counter
        self.fps_counter.update(cx.frame_time());
        
        // Request next frame for continuous animation
        cx.request_frame();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .child(self.render_toolbar(cx))
            .child(self.render_demo_area(cx))
            .child(self.render_dimensions_panel(cx))
    }
}

impl AnimationDemoTab {
    fn render_toolbar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .h(px(48.0))
            .px_4()
            .items_center()
            .gap_2()
            .bg(rgb(0x252525))
            .border_b_1()
            .border_color(rgb(0x3e3e3e))
            .child(self.render_demo_button("Friday Border", DemoType::FridayBorder, cx))
            .child(self.render_demo_button("Hello Glow", DemoType::HelloGlow, cx))
            .child(self.render_demo_button("Sidebar", DemoType::Sidebar, cx))
            .child(self.render_demo_button("Carousel", DemoType::Carousel, cx))
            .child(self.render_demo_button("Dock", DemoType::Dock, cx))
            .child(self.render_demo_button("All", DemoType::All, cx))
    }
    
    fn render_demo_button(
        &self,
        label: &str,
        demo_type: DemoType,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let is_active = self.active_demo == demo_type;
        
        div()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(if is_active {
                rgb(0x0078d4)
            } else {
                rgb(0x2d2d2d)
            })
            .hover(|style| style.bg(if is_active {
                rgb(0x0086f0)
            } else {
                rgb(0x3e3e3e)
            }))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _event, cx| {
                this.switch_demo(demo_type, cx);
            }))
            .child(
                div()
                    .text_sm()
                    .text_color(if is_active {
                        rgb(0xffffff)
                    } else {
                        rgb(0xcccccc)
                    })
                    .child(label)
            )
    }
    
    fn render_demo_area(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .p_8()
            .child(match self.active_demo {
                DemoType::FridayBorder => {
                    self.friday_border.as_mut().map(|f| f.render(cx))
                }
                DemoType::HelloGlow => {
                    self.hello_glow.as_mut().map(|g| g.render(cx))
                }
                DemoType::Sidebar => {
                    self.sidebar_demo.as_mut().map(|s| s.render(cx))
                }
                DemoType::Carousel => {
                    self.carousel_demo.as_mut().map(|c| c.render(cx))
                }
                DemoType::Dock => {
                    self.dock_demo.as_mut().map(|d| d.render(cx))
                }
                DemoType::All => {
                    Some(self.render_all_demos(cx))
                }
            })
    }
    
    fn render_all_demos(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_8()
            .size_full()
            .child(
                div()
                    .flex()
                    .gap_8()
                    .child(
                        div()
                            .flex_1()
                            .h(px(300.0))
                            .child(self.friday_border.as_mut().map(|f| f.render(cx)))
                    )
                    .child(
                        div()
                            .flex_1()
                            .h(px(300.0))
                            .child(self.hello_glow.as_mut().map(|g| g.render(cx)))
                    )
            )
            .child(
                div()
                    .flex()
                    .gap_8()
                    .child(
                        div()
                            .flex_1()
                            .h(px(400.0))
                            .child(self.sidebar_demo.as_mut().map(|s| s.render(cx)))
                    )
                    .child(
                        div()
                            .flex_1()
                            .h(px(400.0))
                            .child(self.carousel_demo.as_mut().map(|c| c.render(cx)))
                    )
            )
            .child(
                div()
                    .flex()
                    .justify_center()
                    .h(px(80.0))
                    .child(self.dock_demo.as_mut().map(|d| d.render(cx)))
            )
    }
    
    fn render_dimensions_panel(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .absolute()
            .bottom_4()
            .right_4()
            .p_4()
            .min_w(px(200.0))
            .bg(rgba(0x1e, 0x1e, 0x1e, 0.95))
            .border_1()
            .border_color(rgb(0x3e3e3e))
            .rounded_lg()
            .shadow_lg()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .child("Performance Metrics")
                    )
                    .child(self.render_metric("FPS", &format!("{:.1}", self.fps_counter.fps())))
                    .child(self.render_metric("Frame Time", &format!("{:.2}ms", self.fps_counter.frame_time())))
                    .child(self.render_metric("Width", &format!("{:.0}px", self.current_width)))
                    .child(self.render_metric("Height", &format!("{:.0}px", self.current_height)))
                    .child(self.render_metric("Animation", &format!("{:.0}%", self.animation_progress * 100.0)))
                    .child(self.render_metric("Active Demo", &format!("{:?}", self.active_demo)))
            )
    }
    
    fn render_metric(&self, label: &str, value: &str) -> impl IntoElement {
        div()
            .flex()
            .justify_between()
            .gap_4()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(label)
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xcccccc))
                    .font_weight(FontWeight::MEDIUM)
                    .child(value)
            )
    }
}

// FPS Counter utility
struct FpsCounter {
    frame_count: u32,
    last_update: Instant,
    current_fps: f32,
    last_frame_time: f32,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            frame_count: 0,
            last_update: Instant::now(),
            current_fps: 0.0,
            last_frame_time: 0.0,
        }
    }
    
    fn update(&mut self, delta_time: f32) {
        self.frame_count += 1;
        self.last_frame_time = delta_time * 1000.0; // Convert to ms
        
        let elapsed = self.last_update.elapsed().as_secs_f32();
        if elapsed >= 1.0 {
            self.current_fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.last_update = Instant::now();
        }
    }
    
    fn fps(&self) -> f32 {
        self.current_fps
    }
    
    fn frame_time(&self) -> f32 {
        self.last_frame_time
    }
}

// Implement Item trait to make it a tab
impl Item for AnimationDemoTab {
    type Event = ();
    
    fn tab_content(&self, _cx: &WindowContext) -> AnyElement {
        Label::new("Animation Demo").into_any_element()
    }
    
    fn tab_icon(&self, _cx: &WindowContext) -> Option<Icon> {
        Some(Icon::Play)
    }
    
    fn tab_tooltip_text(&self, _cx: &WindowContext) -> Option<SharedString> {
        Some("Animation Demo - Test all animations".into())
    }
}

// Make it serializable so it persists across sessions
impl SerializableItem for AnimationDemoTab {
    fn serialized_item_kind() -> &'static str {
        "AnimationDemoTab"
    }
    
    fn deserialize(
        _project: Model<Project>,
        _workspace: WeakView<Workspace>,
        _item_id: workspace::ItemId,
        cx: &mut ViewContext<Pane>,
    ) -> Task<Result<View<Self>>> {
        Task::ready(Ok(cx.new_view(|cx| Self::new(cx))))
    }
    
    fn serialize(
        &mut self,
        _workspace: &mut Workspace,
        _cx: &mut ViewContext<Self>,
    ) -> Option<Task<Result<()>>> {
        None
    }
}
```

---

### Making Demo Tab the Default Screen

#### Step 1: Register the Demo Tab

**File:** `crates/animation_demo/src/lib.rs`

```rust
use gpui::*;

mod demo_tab;
mod friday;
mod hello_glow;
mod sidebar;
mod carousel;
mod macos_dock;
mod animation;

pub use demo_tab::AnimationDemoTab;

pub fn init(cx: &mut AppContext) {
    // Register the demo tab as a serializable item
    workspace::register_serializable_item::<AnimationDemoTab>(cx);
}
```

#### Step 2: Modify Workspace to Show Demo Tab by Default

**File:** `crates/workspace/src/workspace.rs`

```rust
// Add to workspace initialization
impl Workspace {
    pub fn new(
        workspace_id: WorkspaceId,
        project: Model<Project>,
        app_state: Arc<AppState>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        // ... existing initialization ...
        
        // If no folder is open, show animation demo tab
        if project.read(cx).visible_worktrees(cx).count() == 0 {
            cx.spawn(|workspace, mut cx| async move {
                workspace.update(&mut cx, |workspace, cx| {
                    let demo_tab = cx.new_view(|cx| AnimationDemoTab::new(cx));
                    workspace.add_item_to_active_pane(Box::new(demo_tab), cx);
                })
            }).detach();
        }
        
        // ... rest of initialization ...
    }
}
```

#### Step 3: Add Command to Open Demo Tab

**File:** `crates/animation_demo/src/demo_tab.rs`

```rust
// Add action for opening demo tab
actions!(animation_demo, [OpenAnimationDemo]);

impl AnimationDemoTab {
    pub fn register(cx: &mut AppContext) {
        cx.observe_new_views(|workspace: &mut Workspace, cx| {
            workspace.register_action(|workspace, _: &OpenAnimationDemo, cx| {
                let demo_tab = cx.new_view(|cx| AnimationDemoTab::new(cx));
                workspace.add_item_to_active_pane(Box::new(demo_tab), cx);
            });
        }).detach();
    }
}
```

#### Step 4: Add Keybinding

**File:** `assets/keymaps/default.json`

```json
{
  "context": "Workspace",
  "bindings": {
    "cmd-shift-a": "animation_demo::OpenAnimationDemo"
  }
}
```

---

### Live Dimensions Display

The dimensions panel shows real-time metrics for the active animation:

```rust
impl AnimationDemoTab {
    fn update_dimensions(&mut self, cx: &mut ViewContext<Self>) {
        match self.active_demo {
            DemoType::FridayBorder => {
                if let Some(friday) = &self.friday_border {
                    self.current_width = friday.width();
                    self.current_height = friday.height();
                    self.animation_progress = friday.animation_progress();
                }
            }
            DemoType::HelloGlow => {
                if let Some(glow) = &self.hello_glow {
                    self.current_width = glow.width();
                    self.current_height = glow.height();
                    self.animation_progress = glow.animation_progress();
                }
            }
            DemoType::Sidebar => {
                if let Some(sidebar) = &self.sidebar_demo {
                    self.current_width = sidebar.width();
                    self.current_height = sidebar.height();
                    self.animation_progress = sidebar.animation_progress();
                }
            }
            DemoType::Carousel => {
                if let Some(carousel) = &self.carousel_demo {
                    self.current_width = carousel.width();
                    self.current_height = carousel.height();
                    self.animation_progress = carousel.animation_progress();
                }
            }
            DemoType::Dock => {
                if let Some(dock) = &self.dock_demo {
                    self.current_width = dock.width();
                    self.current_height = dock.height();
                    self.animation_progress = dock.animation_progress();
                }
            }
            DemoType::All => {
                // Show aggregate metrics
                self.current_width = 0.0;
                self.current_height = 0.0;
                self.animation_progress = 0.0;
            }
        }
    }
}
```

---

### Animation Controls

Add interactive controls for each animation:

```rust
impl AnimationDemoTab {
    fn render_animation_controls(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .absolute()
            .bottom_4()
            .left_4()
            .p_4()
            .bg(rgba(0x1e, 0x1e, 0x1e, 0.95))
            .border_1()
            .border_color(rgb(0x3e3e3e))
            .rounded_lg()
            .shadow_lg()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(self.render_control_button("Play", Icon::Play, cx))
                    .child(self.render_control_button("Pause", Icon::Pause, cx))
                    .child(self.render_control_button("Reset", Icon::RotateCcw, cx))
                    .child(self.render_control_button("Slow", Icon::Gauge, cx))
            )
    }
    
    fn render_control_button(
        &self,
        label: &str,
        icon: Icon,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        div()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(rgb(0x2d2d2d))
            .hover(|style| style.bg(rgb(0x3e3e3e)))
            .cursor_pointer()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Icon::new(icon).size(IconSize::Small))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xcccccc))
                            .child(label)
                    )
            )
    }
}
```

---

### Integration with Zed's Startup

**File:** `crates/zed/src/main.rs`

```rust
fn main() {
    // ... existing initialization ...
    
    // Initialize animation demo
    animation_demo::init(&mut cx);
    
    // Register demo tab
    AnimationDemoTab::register(&mut cx);
    
    // ... rest of initialization ...
}
```

---

### Testing the Demo Tab

**Manual Testing Checklist:**

1. **Startup Test**
   - [ ] Launch Zed without opening a folder
   - [ ] Demo tab appears automatically
   - [ ] "All" view is selected by default
   - [ ] All 5 animations are visible

2. **Toolbar Test**
   - [ ] Click each button (Friday, Hello Glow, Sidebar, Carousel, Dock, All)
   - [ ] Correct animation displays
   - [ ] Active button is highlighted
   - [ ] Smooth transitions between demos

3. **Dimensions Panel Test**
   - [ ] FPS counter updates (should show ~120 FPS)
   - [ ] Frame time shows <8.33ms
   - [ ] Width/height update when resizing
   - [ ] Animation progress shows 0-100%
   - [ ] Active demo name is correct

4. **Animation Controls Test**
   - [ ] Play button starts animation
   - [ ] Pause button stops animation
   - [ ] Reset button restarts from beginning
   - [ ] Slow button reduces speed (0.5x)

5. **Performance Test**
   - [ ] All animations maintain 120 FPS
   - [ ] No frame drops during transitions
   - [ ] Memory usage is stable
   - [ ] CPU usage is reasonable

6. **Keybinding Test**
   - [ ] Cmd+Shift+A opens demo tab
   - [ ] Can open multiple demo tabs
   - [ ] Each tab is independent

---

### Advanced Features

#### Side-by-Side Comparison

Add a split view to compare web version with GPUI implementation:

```rust
impl AnimationDemoTab {
    fn render_comparison_mode(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .child(
                div()
                    .flex_1()
                    .border_r_1()
                    .border_color(rgb(0x3e3e3e))
                    .child(
                        div()
                            .p_2()
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .child("Web Reference")
                    )
                    .child(self.render_web_reference(cx))
            )
            .child(
                div()
                    .flex_1()
                    .child(
                        div()
                            .p_2()
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .child("GPUI Implementation")
                    )
                    .child(self.render_demo_area(cx))
            )
    }
    
    fn render_web_reference(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // Embed web view or screenshot for comparison
        div()
            .flex_1()
            .bg(rgb(0x000000))
            .child("Web reference would go here")
    }
}
```

#### Recording Mode

Add ability to record animations for documentation:

```rust
impl AnimationDemoTab {
    fn start_recording(&mut self, cx: &mut ViewContext<Self>) {
        // Capture frames for GIF/video export
        self.recording = true;
        self.recorded_frames.clear();
        cx.notify();
    }
    
    fn stop_recording(&mut self, cx: &mut ViewContext<Self>) {
        self.recording = false;
        // Export frames to GIF
        self.export_recording(cx);
    }
}
```

---

### Summary

The Demo Tab provides:

1. ✅ **Default Screen** - Shows automatically when no folder is open
2. ✅ **Animation Showcase** - All 5 animations in one place
3. ✅ **Live Metrics** - FPS, frame time, dimensions, progress
4. ✅ **Interactive Controls** - Play, pause, reset, slow motion
5. ✅ **Easy Switching** - Toolbar to switch between demos
6. ✅ **Performance Monitoring** - Real-time performance tracking
7. ✅ **Comparison Mode** - Side-by-side with web reference
8. ✅ **Keybinding** - Cmd+Shift+A to open anytime

This makes development and testing much easier, allowing you to see all animations in action and verify they match the web implementation.

---

## Part 11: Resources & References

### Official Documentation

- [Zed Editor](https://zed.dev/)
- [GPUI Documentation](https://www.gpui.rs/)
- [Zed GitHub](https://github.com/zed-industries/zed)
- [GPUI Blog: Rendering at 120 FPS](https://zed.dev/blog/videogame)
- [GPUI Blog: Ownership and Data Flow](https://zed.dev/blog/gpui-ownership)

### Zed Codebase Study Guide

**Essential Files:**
1. `crates/workspace/src/workspace.rs` - Main workspace container
2. `crates/workspace/src/pane.rs` - Tab/pane management
3. `crates/workspace/src/sidebar.rs` - Sidebar implementation
4. `crates/gpui/src/window.rs` - Window and event handling
5. `crates/gpui/src/element.rs` - Element trait
6. `crates/gpui/src/scene.rs` - Scene graph
7. `crates/gpui/examples/` - GPUI examples

**GPUI Examples to Study:**
- `animation.rs` - Animation examples
- `shadow.rs` - Shadow rendering
- `image.rs` - Image loading
- `text_wrapper.rs` - Text wrapping
- `uniform_list.rs` - Optimized lists

### Key Concepts

**From "Rendering at 120 FPS" Blog:**
- Custom shaders for specific primitives
- Signed Distance Functions (SDF) for shapes
- Glyph atlas for text rendering
- Element trait for layout/paint
- Scene graph architecture

**From GPUI README:**
- Hybrid immediate/retained mode
- Entity system for state management
- Context types (App, Window, Test)
- Action system for keyboard shortcuts

### Rust Resources

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust Edition 2024](https://doc.rust-lang.org/edition-guide/)
- [Metal Shading Language](https://developer.apple.com/metal/Metal-Shading-Language-Specification.pdf)

### Animation Theory

- [Easing Functions](https://easings.net/)
- [Spring Physics](https://www.ryanjuckett.com/damped-springs/)
- [Signed Distance Functions](https://iquilezles.org/articles/distfunctions/)

---

## Part 11: Summary

### Core UI Replacements

We are replacing Zed's core UI with our custom implementations:

1. ✅ **Workspace Sidebar** - Replace `crates/workspace/src/sidebar.rs` with our animated sidebar
2. ✅ **Tab System** - Replace tab bar in `crates/workspace/src/pane.rs` with screen carousel
3. ✅ **macOS Dock** - Add top dock to `crates/workspace/src/workspace.rs`
4. ✅ **Demo Tab** - Testing environment for all animations

### Essential Animations

We are implementing these 6 critical animation systems:

1. ✅ **Friday Border Effect** - Rainbow gradient borders with scroll animation
2. ✅ **Hello Glow Effect** - Animated gradient background with dual blur
3. ✅ **Sidebar Animations** - Expand/collapse, folders, workspace switcher
4. ✅ **Screen Carousel** - Resizable panels with directional gravity
5. ✅ **macOS Dock** - Top dock with hover magnification and controls
6. ✅ **Drag-and-Drop System** - Tab/folder organization with visual feedback

### Development Approach

**Key Workflow:**
1. Fork Zed and study architecture (`workspace.rs`, `pane.rs`, `sidebar.rs`)
2. Copy web code to `reference/` folder in Zed fork
3. Create `animation_demo` crate with all components
4. Create demo tab (like onboarding) with dimensions display
5. Implement GPUI versions side-by-side with web reference
6. Replace Zed's components with our implementations
7. Test integration thoroughly
8. Iterate until visual and performance match

**Integration Points:**
- `crates/workspace/src/workspace.rs` - Add dock, replace sidebar
- `crates/workspace/src/pane.rs` - Replace tab bar with carousel
- `crates/workspace/src/sidebar.rs` - Replace with CustomSidebar
- `crates/animation_demo/` - All our custom components

**Success Criteria:**
- Visual match with web version
- 120 FPS performance
- Smooth spring physics
- Accurate dimensions
- Responsive interactions
- All Zed features still work

### Timeline

- **Weeks 1-2:** Foundation + Architecture Study + Demo Tab
- **Week 3:** Hello Glow Effect
- **Weeks 4-5:** Replace Workspace Sidebar
- **Weeks 6-8:** Replace Tab System with Carousel
- **Weeks 9-10:** Add macOS Dock
- **Weeks 11-12:** Friday Border Effect
- **Weeks 13-14:** Drag-and-Drop System

**Total:** 14 weeks for complete implementation

### Key Files to Modify

**Zed Core (Modify):**
```
crates/workspace/src/
├── workspace.rs     # Add dock, replace sidebar
├── pane.rs          # Replace tab bar
└── sidebar.rs       # Replace entirely
```

**Our Code (Create):**
```
crates/animation_demo/src/
├── lib.rs
├── demo_tab.rs
├── sidebar.rs       # CustomSidebar
├── carousel.rs      # ScreenCarousel
├── macos_dock.rs    # MacOSDock
├── friday.rs        # Friday border effect
├── hello_glow.rs    # Hello glow effect
├── drag_drop.rs     # Drag-and-drop system
└── animation/
    ├── mod.rs
    ├── animated_value.rs
    ├── spring.rs
    └── easing.rs
```

**Shaders (Create):**
```
crates/animation_demo/shaders/
├── gradient.metal   # Rainbow gradient
└── blur.metal       # Gaussian blur
```

**Reference (Copy):**
```
reference/friday-web/
├── components/
│   ├── friday.tsx
│   ├── hello-glow.tsx
│   ├── browser/
│   └── screens/
└── README.md
```

---

**Document Version:** 3.0  
**Last Updated:** March 30, 2026  
**Focus:** Complete UI replacement in Zed fork with detailed code references
