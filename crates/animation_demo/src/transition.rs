use std::time::Duration;

use crate::{AnimatedPoint, AnimatedValue, EasingCurve, SpringConfig};

pub struct AppleTransitions;

impl AppleTransitions {
    pub fn fade_in(opacity: &mut AnimatedValue, duration: Duration) {
        opacity.set(0.0);
        opacity.ease_to(1.0, duration, EasingCurve::AppleDefault);
    }

    pub fn fade_out(opacity: &mut AnimatedValue, duration: Duration) {
        opacity.ease_to(0.0, duration, EasingCurve::EaseOut);
    }

    pub fn scale_in(scale: &mut AnimatedValue, opacity: &mut AnimatedValue) {
        scale.set(0.95);
        opacity.set(0.0);
        scale.spring_to(1.0, SpringConfig::smooth());
        opacity.ease_to(1.0, Duration::from_millis(180), EasingCurve::AppleDefault);
    }

    pub fn sheet_present(
        y: &mut AnimatedValue,
        opacity: &mut AnimatedValue,
        scale: &mut AnimatedValue,
        container_height: f32,
    ) {
        y.set(container_height);
        opacity.set(0.0);
        scale.set(0.95);

        y.spring_to(0.0, SpringConfig::smooth());
        opacity.ease_to(1.0, Duration::from_millis(200), EasingCurve::AppleDefault);
        scale.spring_to(1.0, SpringConfig::snappy());
    }

    pub fn sidebar_expand(width: &mut AnimatedValue) {
        width.spring_to(360.0, SpringConfig::snappy());
    }

    pub fn sidebar_collapse(width: &mut AnimatedValue) {
        width.spring_to(56.0, SpringConfig::snappy());
    }

    pub fn slide_up(offset: &mut AnimatedValue, opacity: &mut AnimatedValue, from: f32) {
        offset.set(from);
        opacity.set(0.0);
        offset.spring_to(0.0, SpringConfig::smooth());
        opacity.ease_to(1.0, Duration::from_millis(180), EasingCurve::AppleDefault);
    }

    pub fn move_point(point: &mut AnimatedPoint, x: f32, y: f32, config: SpringConfig) {
        point.spring_to(x, y, config);
    }
}
