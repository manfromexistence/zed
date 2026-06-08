use std::time::{Duration, Instant};

use gpui::{
    Animation, AnimationElement, AnimationExt, AnyElement, App, Element, ElementId,
    GlobalElementId, InspectorElementId, IntoElement, Pixels, Transformation, Window,
    ease_out_quint, percentage, point, px, radians, size,
};

use crate::traits::transformable::Transformable;

/// A subtle transform/opacity target for icon hover microinteractions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconHoverEffect {
    scale: f32,
    rotation_degrees: f32,
    translation: gpui::Point<Pixels>,
    opacity: f32,
    duration: Duration,
    reduced_motion: bool,
}

impl Default for IconHoverEffect {
    fn default() -> Self {
        Self {
            scale: 1.0,
            rotation_degrees: 0.0,
            translation: point(px(0.0), px(0.0)),
            opacity: 1.0,
            duration: Duration::from_millis(150),
            reduced_motion: false,
        }
    }
}

impl IconHoverEffect {
    /// A balanced default: a slight scale-up with a fast ease-out transition.
    pub fn subtle_scale() -> Self {
        Self::default().with_scale(1.06)
    }

    /// A small upward nudge that works well for compact toolbar icons.
    pub fn lift() -> Self {
        Self::default()
            .with_scale(1.04)
            .with_translation(px(0.0), px(-1.0))
    }

    pub fn scale(scale: f32) -> Self {
        Self::default().with_scale(scale)
    }

    pub fn rotate_degrees(degrees: f32) -> Self {
        Self::default().with_rotation_degrees(degrees)
    }

    pub fn translate(x: Pixels, y: Pixels) -> Self {
        Self::default().with_translation(x, y)
    }

    pub fn fade(opacity: f32) -> Self {
        Self::default().with_opacity(opacity)
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_rotation_degrees(mut self, degrees: f32) -> Self {
        self.rotation_degrees = degrees;
        self
    }

    pub fn with_translation(mut self, x: Pixels, y: Pixels) -> Self {
        self.translation = point(x, y);
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration.max(Duration::from_millis(1));
        self
    }

    pub fn reduced_motion(mut self, reduced_motion: bool) -> Self {
        self.reduced_motion = reduced_motion;
        self
    }

    pub fn duration(self) -> Duration {
        self.duration
    }

    pub fn is_reduced_motion(self) -> bool {
        self.reduced_motion
    }

    pub(crate) fn animation(self) -> Animation {
        Animation::new(self.duration).with_easing(ease_out_quint())
    }

    pub(crate) fn opacity_at(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        1.0 + (self.opacity - 1.0) * progress
    }

    pub(crate) fn transformation_at(self, base: Transformation, progress: f32) -> Transformation {
        let progress = progress.clamp(0.0, 1.0);
        let scale = 1.0 + (self.scale - 1.0) * progress;
        let rotation = self.rotation_degrees.to_radians() * progress;
        let translation = point(self.translation.x * progress, self.translation.y * progress);

        base.then(
            Transformation::default()
                .with_scaling(size(scale, scale))
                .with_translation(translation)
                .with_rotation(radians(rotation)),
        )
    }
}

/// A target-driven animation wrapper for hover state changes.
pub struct HoverAnimationElement<E> {
    id: ElementId,
    hovered: bool,
    reduced_motion: bool,
    element: Option<E>,
    animation: Animation,
    animator: Box<dyn Fn(E, f32) -> E + 'static>,
}

impl<E> HoverAnimationElement<E> {
    pub fn map_element(mut self, f: impl FnOnce(E) -> E) -> HoverAnimationElement<E> {
        self.element = self.element.map(f);
        self
    }
}

impl<E: IntoElement + 'static> IntoElement for HoverAnimationElement<E> {
    type Element = HoverAnimationElement<E>;

    fn into_element(self) -> Self::Element {
        self
    }
}

struct HoverAnimationState {
    target_hovered: bool,
    progress: f32,
    from_progress: f32,
    started_at: Option<Instant>,
}

impl HoverAnimationState {
    fn new(hovered: bool) -> Self {
        Self {
            target_hovered: hovered,
            progress: if hovered { 1.0 } else { 0.0 },
            from_progress: if hovered { 1.0 } else { 0.0 },
            started_at: None,
        }
    }
}

impl<E: IntoElement + 'static> Element for HoverAnimationElement<E> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        window.with_element_state(global_id.unwrap(), |state, window| {
            let mut state = state.unwrap_or_else(|| HoverAnimationState::new(self.hovered));
            let target_progress = if self.hovered { 1.0 } else { 0.0 };

            if self.reduced_motion {
                state.target_hovered = self.hovered;
                state.progress = target_progress;
                state.from_progress = target_progress;
                state.started_at = None;
            } else {
                if state.target_hovered != self.hovered {
                    state.target_hovered = self.hovered;
                    state.from_progress = state.progress;
                    state.started_at = Some(Instant::now());
                }

                if let Some(started_at) = state.started_at {
                    let elapsed = started_at.elapsed();
                    let duration = self.animation.duration.max(Duration::from_millis(1));
                    let delta = elapsed.div_duration_f32(duration).min(1.0);
                    let eased_delta = (self.animation.easing)(delta);

                    state.progress =
                        state.from_progress + (target_progress - state.from_progress) * eased_delta;

                    if delta >= 1.0 {
                        state.progress = target_progress;
                        state.started_at = None;
                    } else {
                        window.request_animation_frame();
                    }
                }
            }

            let element = self.element.take().expect("should only be called once");
            let mut element = (self.animator)(element, state.progress).into_any_element();

            ((element.request_layout(window, cx), element), state)
        })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

/// An extension trait for adding common animations to animatable components.
pub trait CommonAnimationExt: AnimationExt {
    /// Render this component as rotating over the given duration.
    ///
    /// NOTE: This method uses the location of the caller to generate an ID for this state.
    ///       If this is not sufficient to identify your state (e.g. you're rendering a list item),
    ///       you can provide a custom ElementID using the `use_keyed_rotate_animation` method.
    #[track_caller]
    fn with_rotate_animation(self, duration: u64) -> AnimationElement<Self>
    where
        Self: Transformable + Sized,
    {
        self.with_keyed_rotate_animation(
            ElementId::CodeLocation(*std::panic::Location::caller()),
            duration,
        )
    }

    /// Render this component as rotating with the given element ID over the given duration.
    fn with_keyed_rotate_animation(
        self,
        id: impl Into<ElementId>,
        duration: u64,
    ) -> AnimationElement<Self>
    where
        Self: Transformable + Sized,
    {
        self.with_animation(
            id,
            Animation::new(Duration::from_secs(duration)).repeat(),
            |component, delta| component.transform(Transformation::rotate(percentage(delta))),
        )
    }

    /// Animate this component between its default and hover transforms.
    fn with_keyed_icon_hover_animation(
        self,
        id: impl Into<ElementId>,
        hovered: bool,
        effect: IconHoverEffect,
    ) -> HoverAnimationElement<Self>
    where
        Self: Transformable + Sized,
    {
        self.with_hover_animation(
            id,
            hovered,
            effect.animation(),
            effect.is_reduced_motion(),
            move |component, progress| {
                component.transform(effect.transformation_at(Transformation::default(), progress))
            },
        )
    }

    /// Animate this component with a caller-provided hover animator and keyed state.
    fn with_hover_animation(
        self,
        id: impl Into<ElementId>,
        hovered: bool,
        animation: Animation,
        reduced_motion: bool,
        animator: impl Fn(Self, f32) -> Self + 'static,
    ) -> HoverAnimationElement<Self>
    where
        Self: Sized,
    {
        HoverAnimationElement {
            id: id.into(),
            hovered,
            reduced_motion,
            element: Some(self),
            animation,
            animator: Box::new(animator),
        }
    }
}

impl<T: AnimationExt> CommonAnimationExt for T {}
