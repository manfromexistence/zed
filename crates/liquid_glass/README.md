# Liquid Glass

This crate provides the shared Liquid Glass GPUI primitives for this Zed fork.

It is not exposed as a standalone fullscreen workspace item. Product surfaces should consume the
bounded GPUI primitive directly, as the AI composer does.

## Integration

- GPU primitive: `gpui::Window::paint_liquid_glass`
- Assets: loaded from the root `assets/liquid_glass/` tree

## Backend model

- Windows: DirectX renderer with HLSL shader support
- macOS: Metal renderer with Metal shader support
- Linux and shared GPU path: wgpu renderer with WGSL shader support

The old imgui/winit standalone path has been retired in favor of bounded editor-integrated GPUI
surfaces.
