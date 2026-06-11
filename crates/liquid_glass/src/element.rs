use std::sync::Arc;

use gpui::{
    AnyElement, Bounds, IntoElement, Pixels, Point, RenderImage, Window, canvas, point, px, size,
};
use ui::prelude::*;

const STANDALONE_VIEWPORT_WIDTH: f32 = 1024.0;
const STANDALONE_VIEWPORT_HEIGHT: f32 = 768.0;
const MAX_GLASS_AXIS_PX: f32 = 4096.0;

#[derive(Clone, Debug)]
pub struct LiquidGlassStyle {
    pub power_factor: f32,
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub f_power: f32,
    pub noise: f32,
    pub glow_weight: f32,
    pub glow_edge0: f32,
    pub glow_edge1: f32,
    pub glow_bias: f32,
    pub chromatic_aberration: f32,
    pub aberration_samples: u32,
    pub blur_radius: f32,
    pub blur_iterations: u32,
    pub blur_downscale: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiquidGlassGeometry {
    pub width: f32,
    pub height: f32,
    pub pixel_scale: f32,
    pub position: [f32; 2],
    pub mouse_control: bool,
}

impl LiquidGlassGeometry {
    pub fn new(
        width: f32,
        height: f32,
        pixel_scale: f32,
        position: [f32; 2],
        mouse_control: bool,
    ) -> Self {
        Self {
            width,
            height,
            pixel_scale,
            position,
            mouse_control,
        }
    }

    pub fn glass_bounds(
        &self,
        source_bounds: Bounds<Pixels>,
        mouse_position: Point<Pixels>,
    ) -> Bounds<Pixels> {
        let width = finite_or_default(self.width, 3.5);
        let height = finite_or_default(self.height, 3.5);
        let pixel_scale = finite_or_default(self.pixel_scale, 100.0).abs().max(1.0);
        let width = px((width * pixel_scale).abs().clamp(1.0, MAX_GLASS_AXIS_PX));
        let height = px((height * pixel_scale).abs().clamp(1.0, MAX_GLASS_AXIS_PX));
        let glass_size = size(width, height);

        let center = if self.mouse_control && source_bounds.contains(&mouse_position) {
            mouse_position
        } else {
            self.position_center(source_bounds)
        };

        Bounds::centered_at(center, glass_size)
    }

    fn position_center(&self, source_bounds: Bounds<Pixels>) -> Point<Pixels> {
        let x_ratio = normalized_axis(self.position[0], STANDALONE_VIEWPORT_WIDTH);
        let y_ratio = normalized_axis(self.position[1], STANDALONE_VIEWPORT_HEIGHT);

        point(
            source_bounds.origin.x + px(source_bounds.size.width.as_f32() * x_ratio),
            source_bounds.origin.y + px(source_bounds.size.height.as_f32() * y_ratio),
        )
    }
}

impl LiquidGlassStyle {
    pub fn paint(
        &self,
        window: &mut Window,
        source_bounds: Bounds<Pixels>,
        glass_bounds: Bounds<Pixels>,
        source_image: Arc<RenderImage>,
    ) {
        if !has_drawable_area(source_bounds) || !has_drawable_area(glass_bounds) {
            return;
        }

        if let Err(error) = window.paint_liquid_glass(
            source_bounds,
            source_image,
            gpui::LiquidGlassParams {
                glass_bounds,
                use_backdrop: true,
                power_factor: self.power_factor,
                a: self.a,
                b: self.b,
                c: self.c,
                d: self.d,
                f_power: self.f_power,
                noise: self.noise,
                glow_weight: self.glow_weight,
                glow_edge0: self.glow_edge0,
                glow_edge1: self.glow_edge1,
                glow_bias: self.glow_bias,
                chromatic_aberration: self.chromatic_aberration,
                aberration_samples: self.aberration_samples,
                blur_radius: self.blur_radius,
                blur_iterations: self.blur_iterations,
                blur_downscale: self.blur_downscale,
            },
            0,
        ) {
            log::debug!("failed to paint Liquid Glass layer: {error}");
        }
    }
}

pub fn liquid_glass_layer(
    source_image: Arc<RenderImage>,
    glass_bounds: Bounds<Pixels>,
    style: LiquidGlassStyle,
) -> AnyElement {
    canvas(
        move |bounds, _, _| bounds,
        move |bounds, _, window, _cx| {
            style.paint(window, bounds, glass_bounds, source_image.clone());
        },
    )
    .into_any_element()
}

pub fn bounded_liquid_glass_layer(
    source_image: Arc<RenderImage>,
    style: LiquidGlassStyle,
) -> AnyElement {
    canvas(
        move |bounds, _, _| bounds,
        move |bounds, _, window, _cx| {
            style.paint(window, bounds, bounds, source_image.clone());
        },
    )
    .absolute()
    .inset_0()
    .size_full()
    .into_any_element()
}

pub fn liquid_glass_layer_with_geometry(
    source_image: Arc<RenderImage>,
    geometry: LiquidGlassGeometry,
    style: LiquidGlassStyle,
) -> AnyElement {
    canvas(
        move |bounds, _, _| bounds,
        move |bounds, _, window, _cx| {
            style.paint(
                window,
                bounds,
                geometry.glass_bounds(bounds, window.mouse_position()),
                source_image.clone(),
            );
        },
    )
    .absolute()
    .inset_0()
    .size_full()
    .into_any_element()
}

pub fn paint_liquid_glass_layer(
    window: &mut Window,
    source_bounds: Bounds<Pixels>,
    glass_bounds: Bounds<Pixels>,
    source_image: Arc<RenderImage>,
    style: &LiquidGlassStyle,
) {
    style.paint(window, source_bounds, glass_bounds, source_image);
}

fn finite_or_default(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn normalized_axis(position: f32, viewport_axis: f32) -> f32 {
    (finite_or_default(position, viewport_axis * 0.5) / viewport_axis).clamp(0.0, 1.0)
}

fn has_drawable_area(bounds: Bounds<Pixels>) -> bool {
    let width = bounds.size.width.as_f32();
    let height = bounds.size.height.as_f32();

    width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0
}
