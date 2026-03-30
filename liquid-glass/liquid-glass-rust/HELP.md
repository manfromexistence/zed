# Help Needed - Liquid Glass Not Rendering

**Date:** 2026-03-18

**Task Description:**
Port the C++ OpenGL liquid glass effect to Rust using wgpu + WGSL shaders. The liquid glass component should render as a movable distortion effect over a background image, with full ImGui controls matching the C++ version.

---

## Problem

The liquid glass component was **not visible at all** on screen. Only the background image showed. The shader logic appeared to be executing (no crashes), but the liquid glass distortion effect was completely invisible.

---

## Root Cause Analysis

The C++ version works because:

1. **It renders a QUAD at a specific world position** (`m_Position`), not fullscreen
2. **The quad has specific size** (`width x height`, default 3.5 x 3.5)
3. **Vertex shader passes quad center and scale** via `v_MidPoint` and `v_QuadNDC2ScreenNDCScale`
4. **Fragment shader uses these to transform coordinates** from quad-local space to screen space
5. **It samples from a blurred framebuffer**, not the original background

The Rust version failed because:

1. **Renders fullscreen triangle**, not a positioned quad
2. **No concept of quad position/size** - everything is in normalized screen space
3. **No coordinate transformation** from local quad space to screen space
4. **Samples original texture**, not a blurred version
5. **The liquid glass was "everywhere" but invisible** because the distortion was applied to the entire screen

---

## Solution Implemented

The fix uses **bidirectional coordinate transformation** between screen UV space and glass-local [-1,1] space:

### Forward Transform (Screen UV → Glass Local)
```wgsl
let glass_center_uv = u.glass_pos / u.resolution;
let glass_half_uv = u.glass_size / (2.0 * u.resolution);
let glass_local = (uv - glass_center_uv) / glass_half_uv;
// glass_local is now in [-1,1] ONLY inside the glass!
```

### SDF (Superellipse Distance Field)
```wgsl
let d = sd_superellipse(glass_local, u.power_factor, radii);
if (d > 0.0) { return background; } // Outside glass
```

### Distortion (Warp toward center)
```wgsl
let warp = pow(distortion_f(dist), u.f_power);
let distorted_local = glass_local * warp;
```

### Inverse Transform (Glass Local → Screen UV) - THE CRITICAL FIX
```wgsl
let sample_uv = distorted_local * glass_half_uv + glass_center_uv;
// Maps distorted coords back to correct screen position
```

---

## Files Created/Modified

### `liquid-glass-rust/src/shaders/liquid_glass.wgsl`
Complete WGSL shader with proper coordinate transformation:
- `vs_main`: Fullscreen triangle vertex shader
- `fs_main`: Fragment shader with SDF, distortion, noise, glow
- `sd_superellipse`: Signed distance function for superellipse shape
- `distortion_f`: Distortion falloff function
- `rand`: Pseudo-random noise generator
- `glow`: Rim lighting effect

### `liquid-glass-rust/src/ui_state.rs`
UI state struct with all parameters:
- Shape: power_factor, rx, ry, width, height
- Distortion: f_scale, f_sharp, f_power
- Appearance: noise, glow_weight, glow_edge0, glow_edge1, glow_bias
- Blur: blur_radius, blur_samples, blur_iterations
- Interaction: mouse_control, position, pixel_scale

### `liquid-glass-rust/src/main.rs`
Complete host code with:
- wgpu setup with proper surface configuration
- Background texture loading (procedural fallback)
- Uniform buffer management
- ImGui integration with control panel
- Event handling with mouse tracking
- Fullscreen triangle rendering pipeline

---

## How to Build & Run

```bash
cd liquid-glass-rust
cargo build --release
./target/release/liquid-glass-rust.exe
```

---

## Controls

- **Mouse movement**: Repositions the liquid glass (when "Move with mouse" is checked)
- **Space**: Resets all parameters to defaults
- **Esc**: Exits the application
- **ImGui panel**: All parameters adjustable in real-time

---

## Expected Behavior

When working correctly:
- A circular/elliptical liquid glass effect should be visible at the mouse position
- It should distort the background image behind it
- Moving the mouse (with "Move with mouse" enabled) should move the glass
- Adjusting sliders should change the glass appearance in real-time
- The glass should be roughly 3.5x3.5 units in size (adjustable via Width/Height sliders)

---

## Current Status

✅ **RESOLVED** - The liquid glass effect is now visible with proper coordinate transformation.

The key insight was that the original attempts were missing the **inverse transform** step - after distorting coordinates in glass-local space, they must be mapped back to screen UV space to sample the correct background pixels.

---

## Environment Info

- **Language:** Rust 1.94.0 (edition 2024)
- **Graphics API:** wgpu 28.0.0
- **Window:** winit 0.30.13
- **UI:** imgui 0.12.0, imgui-wgpu 0.27.0, imgui-winit-support 0.13.0
- **OS:** Windows
- **Reference:** C++ with OpenGL 4.5, OverEngine framework
