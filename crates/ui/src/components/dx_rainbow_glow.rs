use std::{sync::OnceLock, time::Instant};

use gpui::{
    App, BorderStyle, Bounds, Corners, Edges, Element, ElementId, GlobalElementId, Hsla,
    InspectorElementId, IntoElement, LayoutId, Pixels, Position, Size, Style, Window, fill, hsla,
    point, px, quad, relative, size, transparent_black,
};

const DX_RAINBOW_STRIPE_COUNT: usize = 17;
const DX_RAINBOW_CYCLE_SECONDS: f64 = 2.4;
const DX_RAINBOW_REDUCED_PHASE: f32 = 0.58;
const DX_RAINBOW_SATURATION: f32 = 0.86;
const DX_RAINBOW_LIGHTNESS: f32 = 0.62;
static DX_RAINBOW_STARTED_AT: OnceLock<Instant> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DxRainbowMotion {
    Animated,
    Reduced,
}

impl DxRainbowMotion {
    fn is_animated(self) -> bool {
        matches!(self, Self::Animated)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DxRainbowPaintSample {
    phase: f32,
    color: Hsla,
    should_request_animation_frame: bool,
}

impl DxRainbowPaintSample {
    pub fn color(self) -> Hsla {
        self.color
    }

    pub fn should_request_animation_frame(self) -> bool {
        self.should_request_animation_frame
    }
}

#[derive(Clone)]
pub struct DxRainbowGlow {
    id: Option<ElementId>,
    height: Pixels,
    radius: Pixels,
    motion: DxRainbowMotion,
    phase_offset: f32,
}

impl DxRainbowGlow {
    pub fn new() -> Self {
        Self {
            id: None,
            height: px(125.),
            radius: px(12.),
            motion: DxRainbowMotion::Reduced,
            phase_offset: 0.,
        }
    }

    pub fn animated() -> Self {
        Self::new().motion(DxRainbowMotion::Animated)
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height.max(Pixels::ZERO);
        self
    }

    pub fn radius(mut self, radius: Pixels) -> Self {
        self.radius = radius.max(Pixels::ZERO);
        self
    }

    pub fn motion(mut self, motion: DxRainbowMotion) -> Self {
        self.motion = motion;
        self
    }

    pub fn phase_offset(mut self, offset: f32) -> Self {
        self.phase_offset = normalize_phase(offset);
        self
    }
}

impl Default for DxRainbowGlow {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for DxRainbowGlow {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        self.id.clone()
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            position: Position::Relative,
            size: Size {
                width: relative(1.).into(),
                height: self.height.into(),
            },
            ..Default::default()
        };

        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        paint_dx_rainbow_glow(bounds, self.radius, self.motion, self.phase_offset, window);
    }
}

impl IntoElement for DxRainbowGlow {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

fn dx_rainbow_hsla(phase: f32, alpha: f32) -> Hsla {
    hsla(
        normalize_phase(phase),
        DX_RAINBOW_SATURATION,
        DX_RAINBOW_LIGHTNESS,
        clamp_unit(alpha),
    )
}

fn dx_rainbow_phase_now(motion: DxRainbowMotion, phase_offset: f32) -> f32 {
    let base_phase = match motion {
        DxRainbowMotion::Animated => dx_rainbow_animated_phase(),
        DxRainbowMotion::Reduced => DX_RAINBOW_REDUCED_PHASE,
    };

    normalize_phase(base_phase + phase_offset)
}

fn dx_rainbow_animated_phase() -> f32 {
    let elapsed_seconds = DX_RAINBOW_STARTED_AT
        .get_or_init(Instant::now)
        .elapsed()
        .as_secs_f64();
    ((elapsed_seconds % DX_RAINBOW_CYCLE_SECONDS) / DX_RAINBOW_CYCLE_SECONDS) as f32
}

pub fn dx_rainbow_paint_sample(
    motion: DxRainbowMotion,
    phase_offset: f32,
    alpha: f32,
) -> DxRainbowPaintSample {
    let phase = dx_rainbow_phase_now(motion, phase_offset);
    DxRainbowPaintSample {
        phase,
        color: dx_rainbow_hsla(phase, alpha),
        should_request_animation_frame: motion.is_animated(),
    }
}

pub fn paint_dx_rainbow_caret_glow(bounds: Bounds<Pixels>, color: Hsla, window: &mut Window) {
    if bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
        return;
    }

    let color = clamp_hsla_channels(color);
    let outer = window.pixel_snap_bounds(bounds.dilate(px(5.)));
    let inner = window.pixel_snap_bounds(bounds.dilate(px(2.)));
    window.paint_quad(fill(outer, color.opacity(0.12)));
    window.paint_quad(fill(inner, color.opacity(0.22)));
}

fn clamp_hsla_channels(color: Hsla) -> Hsla {
    hsla(
        normalize_phase(color.h),
        clamp_unit(color.s),
        clamp_unit(color.l),
        clamp_unit(color.a),
    )
}

fn paint_dx_rainbow_glow(
    bounds: Bounds<Pixels>,
    radius: Pixels,
    motion: DxRainbowMotion,
    phase_offset: f32,
    window: &mut Window,
) {
    let bounds = window.pixel_snap_bounds(bounds);
    if bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
        return;
    }

    let sample = dx_rainbow_paint_sample(motion, phase_offset, 0.18);
    if sample.should_request_animation_frame {
        window.request_animation_frame();
    }

    let radius = clamp_radius(radius, bounds.size);
    let corners = Corners::all(radius);

    paint_dx_rainbow_wash(
        bounds.dilate(px(28.)),
        radius + px(28.),
        sample.phase - 0.08,
        0.1,
        window,
    );
    paint_dx_rainbow_wash(
        bounds.dilate(px(10.)),
        radius + px(10.),
        sample.phase - 0.03,
        0.22,
        window,
    );
    window.paint_quad(quad(
        bounds,
        corners,
        sample.color,
        Edges::default(),
        transparent_black(),
        BorderStyle::default(),
    ));
    paint_dx_rainbow_stripes(bounds, radius, sample.phase, 1., window);
}

fn paint_dx_rainbow_wash(
    bounds: Bounds<Pixels>,
    radius: Pixels,
    phase: f32,
    alpha: f32,
    window: &mut Window,
) {
    let bounds = window.pixel_snap_bounds(bounds);
    if bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
        return;
    }

    window.paint_quad(quad(
        bounds,
        Corners::all(clamp_radius(radius, bounds.size)),
        dx_rainbow_hsla(phase, alpha),
        Edges::default(),
        transparent_black(),
        BorderStyle::default(),
    ));
}

fn paint_dx_rainbow_stripes(
    bounds: Bounds<Pixels>,
    radius: Pixels,
    phase: f32,
    alpha: f32,
    window: &mut Window,
) {
    if bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
        return;
    }

    let stripe_width = px(bounds.size.width.as_f32() / DX_RAINBOW_STRIPE_COUNT as f32);
    let last_ix = DX_RAINBOW_STRIPE_COUNT - 1;

    for ix in 0..DX_RAINBOW_STRIPE_COUNT {
        let left = bounds.origin.x + stripe_width * ix;
        let width = if ix == last_ix {
            bounds.right() - left
        } else {
            stripe_width + px(0.5)
        };
        let stripe_bounds = Bounds::new(
            point(left, bounds.origin.y),
            size(width, bounds.size.height),
        );
        let stripe_phase = phase + ix as f32 / last_ix as f32;
        let corners = stripe_corners(ix, last_ix, clamp_radius(radius, stripe_bounds.size));

        window.paint_quad(quad(
            window.pixel_snap_bounds(stripe_bounds),
            corners,
            dx_rainbow_hsla(stripe_phase, alpha),
            Edges::default(),
            transparent_black(),
            BorderStyle::default(),
        ));
    }
}

fn stripe_corners(ix: usize, last_ix: usize, radius: Pixels) -> Corners<Pixels> {
    if ix == 0 {
        Corners {
            top_left: radius,
            bottom_left: radius,
            ..Corners::default()
        }
    } else if ix == last_ix {
        Corners {
            top_right: radius,
            bottom_right: radius,
            ..Corners::default()
        }
    } else {
        Corners::default()
    }
}

fn clamp_radius(radius: Pixels, size: Size<Pixels>) -> Pixels {
    radius
        .max(Pixels::ZERO)
        .min(size.width.min(size.height) * 0.5)
}

fn normalize_phase(phase: f32) -> f32 {
    if phase.is_finite() {
        phase.rem_euclid(1.)
    } else {
        0.
    }
}

fn clamp_unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0., 1.)
    } else {
        0.
    }
}
