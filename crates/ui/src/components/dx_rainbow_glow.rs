use std::{sync::OnceLock, time::Instant};

use gpui::{
    App, BorderStyle, Bounds, Corners, Edges, Element, ElementId, GlobalElementId, Hsla,
    InspectorElementId, IntoElement, LayoutId, Pixels, Position, Size, Style, Window, fill, hsla,
    linear_color_stop, linear_gradient, point, px, quad, relative, size, transparent_black,
};

const DX_RAINBOW_STOP_COUNT: usize = 17;
const DX_RAINBOW_STOP_LAST_INDEX: usize = DX_RAINBOW_STOP_COUNT - 1;
const DX_RAINBOW_STRIPE_COUNT: usize = 25;
const DX_RAINBOW_CARET_CYCLE_NANOS: u128 = 2_400_000_000;
const DX_RAINBOW_GLOW_STRIPE_CYCLE_NANOS: u128 = 6_000_000_000;
const DX_RAINBOW_GLOW_NEAR_WASH_CYCLE_NANOS: u128 = 7_200_000_000;
const DX_RAINBOW_GLOW_OUTER_WASH_CYCLE_NANOS: u128 = 8_400_000_000;
const DX_RAINBOW_REDUCED_PHASE: f32 = 0.58;
const DX_RAINBOW_STOPS: [Hsla; DX_RAINBOW_STOP_COUNT] = [
    Hsla {
        h: 0.966503268,
        s: 1.,
        l: 0.6,
        a: 1.,
    },
    Hsla {
        h: 0.025773196,
        s: 1.,
        l: 0.619607843,
        a: 1.,
    },
    Hsla {
        h: 0.072916667,
        s: 1.,
        l: 0.592156863,
        a: 1.,
    },
    Hsla {
        h: 0.111413043,
        s: 1.,
        l: 0.639215686,
        a: 1.,
    },
    Hsla {
        h: 0.149888143,
        s: 1.,
        l: 0.707843137,
        a: 1.,
    },
    Hsla {
        h: 0.240165631,
        s: 1.,
        l: 0.684313725,
        a: 1.,
    },
    Hsla {
        h: 0.375502008,
        s: 1.,
        l: 0.674509804,
        a: 1.,
    },
    Hsla {
        h: 0.438888889,
        s: 0.9,
        l: 0.607843137,
        a: 1.,
    },
    Hsla {
        h: 0.512562814,
        s: 1.,
        l: 0.609803922,
        a: 1.,
    },
    Hsla {
        h: 0.559463987,
        s: 1.,
        l: 0.609803922,
        a: 1.,
    },
    Hsla {
        h: 0.626204239,
        s: 1.,
        l: 0.660784314,
        a: 1.,
    },
    Hsla {
        h: 0.685307018,
        s: 1.,
        l: 0.701960784,
        a: 1.,
    },
    Hsla {
        h: 0.738993711,
        s: 1.,
        l: 0.688235294,
        a: 1.,
    },
    Hsla {
        h: 0.793456033,
        s: 1.,
        l: 0.680392157,
        a: 1.,
    },
    Hsla {
        h: 0.880239521,
        s: 1.,
        l: 0.67254902,
        a: 1.,
    },
    Hsla {
        h: 0.940074906,
        s: 1.,
        l: 0.650980392,
        a: 1.,
    },
    Hsla {
        h: 0.966503268,
        s: 1.,
        l: 0.6,
        a: 1.,
    },
];
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DxRainbowDirection {
    Forward,
    Reverse,
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
    let phase = normalize_phase(phase);
    let scaled_phase = phase * DX_RAINBOW_STOP_LAST_INDEX as f32;
    let stop_ix = (scaled_phase.floor() as usize).min(DX_RAINBOW_STOP_LAST_INDEX - 1);
    let mix = scaled_phase - stop_ix as f32;
    let start = DX_RAINBOW_STOPS[stop_ix];
    let end = DX_RAINBOW_STOPS[stop_ix + 1];

    Hsla {
        h: lerp_hue(start.h, end.h, mix),
        s: lerp_unit(start.s, end.s, mix),
        l: lerp_unit(start.l, end.l, mix),
        a: clamp_unit(alpha),
    }
}

fn dx_rainbow_phase_now(
    motion: DxRainbowMotion,
    phase_offset: f32,
    cycle_nanos: u128,
    direction: DxRainbowDirection,
) -> f32 {
    dx_rainbow_phase_at(
        motion,
        phase_offset,
        cycle_nanos,
        direction,
        dx_rainbow_elapsed_nanos(),
    )
}

fn dx_rainbow_phase_at(
    motion: DxRainbowMotion,
    phase_offset: f32,
    cycle_nanos: u128,
    direction: DxRainbowDirection,
    elapsed_nanos: u128,
) -> f32 {
    let base_phase = match motion {
        DxRainbowMotion::Animated => {
            let phase = dx_rainbow_cycle_phase(elapsed_nanos, cycle_nanos);
            match direction {
                DxRainbowDirection::Forward => phase,
                DxRainbowDirection::Reverse => -phase,
            }
        }
        DxRainbowMotion::Reduced => DX_RAINBOW_REDUCED_PHASE,
    };

    normalize_phase(base_phase + phase_offset)
}

fn dx_rainbow_elapsed_nanos() -> u128 {
    DX_RAINBOW_STARTED_AT
        .get_or_init(Instant::now)
        .elapsed()
        .as_nanos()
}

fn dx_rainbow_cycle_phase(elapsed_nanos: u128, cycle_nanos: u128) -> f32 {
    let cycle_position = elapsed_nanos % cycle_nanos;
    cycle_position as f32 / cycle_nanos as f32
}

pub fn dx_rainbow_paint_sample(
    motion: DxRainbowMotion,
    phase_offset: f32,
    alpha: f32,
) -> DxRainbowPaintSample {
    let phase = dx_rainbow_phase_now(
        motion,
        phase_offset,
        DX_RAINBOW_CARET_CYCLE_NANOS,
        DxRainbowDirection::Forward,
    );
    DxRainbowPaintSample {
        phase,
        color: dx_rainbow_hsla(phase, alpha),
        should_request_animation_frame: motion.is_animated(),
    }
}

fn dx_rainbow_paint_sample_for_cycle(
    motion: DxRainbowMotion,
    phase_offset: f32,
    alpha: f32,
    cycle_nanos: u128,
    direction: DxRainbowDirection,
    elapsed_nanos: u128,
) -> DxRainbowPaintSample {
    let phase = dx_rainbow_phase_at(motion, phase_offset, cycle_nanos, direction, elapsed_nanos);
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

    let elapsed_nanos = dx_rainbow_elapsed_nanos();
    let stripe_sample = dx_rainbow_paint_sample_for_cycle(
        motion,
        phase_offset,
        0.18,
        DX_RAINBOW_GLOW_STRIPE_CYCLE_NANOS,
        DxRainbowDirection::Forward,
        elapsed_nanos,
    );
    if stripe_sample.should_request_animation_frame {
        window.request_animation_frame();
    }

    let radius = clamp_radius(radius, bounds.size);
    let corners = Corners::all(radius);
    let near_wash_phase = dx_rainbow_phase_at(
        motion,
        phase_offset - 0.03,
        DX_RAINBOW_GLOW_NEAR_WASH_CYCLE_NANOS,
        DxRainbowDirection::Forward,
        elapsed_nanos,
    );
    let outer_wash_phase = dx_rainbow_phase_at(
        motion,
        phase_offset - 0.08,
        DX_RAINBOW_GLOW_OUTER_WASH_CYCLE_NANOS,
        DxRainbowDirection::Reverse,
        elapsed_nanos,
    );

    paint_dx_rainbow_wash(
        bounds.dilate(px(28.)),
        radius + px(28.),
        outer_wash_phase,
        0.1,
        window,
    );
    paint_dx_rainbow_wash(
        bounds.dilate(px(10.)),
        radius + px(10.),
        near_wash_phase,
        0.22,
        window,
    );
    window.paint_quad(quad(
        bounds,
        corners,
        stripe_sample.color,
        Edges::default(),
        transparent_black(),
        BorderStyle::default(),
    ));
    paint_dx_rainbow_stripes(bounds, radius, stripe_sample.phase, 1., window);
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
    let phase_step = 1. / DX_RAINBOW_STRIPE_COUNT as f32;

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
        let stripe_phase = phase + ix as f32 * phase_step;
        let next_stripe_phase = stripe_phase + phase_step;
        let corners = stripe_corners(ix, last_ix, clamp_radius(radius, stripe_bounds.size));

        window.paint_quad(quad(
            window.pixel_snap_bounds(stripe_bounds),
            corners,
            linear_gradient(
                90.,
                linear_color_stop(dx_rainbow_hsla(stripe_phase, alpha), 0.),
                linear_color_stop(dx_rainbow_hsla(next_stripe_phase, alpha), 1.),
            ),
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

fn lerp_hue(start: f32, end: f32, mix: f32) -> f32 {
    let delta = (end - start + 0.5).rem_euclid(1.) - 0.5;
    normalize_phase(start + delta * clamp_unit(mix))
}

fn lerp_unit(start: f32, end: f32, mix: f32) -> f32 {
    start + (end - start) * clamp_unit(mix)
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
