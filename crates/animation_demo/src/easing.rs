#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EasingCurve {
    AppleDefault,
    AppleKeyboard,
    AppleMorph,
    EaseIn,
    EaseOut,
    EaseInOut,
    Linear,
    CubicBezier(f32, f32, f32, f32),
}

impl EasingCurve {
    pub fn evaluate(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::AppleDefault => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
            Self::AppleKeyboard => cubic_bezier(0.34, 1.0, 0.64, 1.0, t),
            Self::AppleMorph => cubic_bezier(0.22, 0.0, 0.0, 1.0, t),
            Self::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
            Self::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
            Self::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
            Self::Linear => t,
            Self::CubicBezier(x1, y1, x2, y2) => cubic_bezier(x1, y1, x2, y2, t),
        }
    }
}

impl Default for EasingCurve {
    fn default() -> Self {
        Self::AppleDefault
    }
}

fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, target_x: f32) -> f32 {
    let mut t = target_x;

    for _ in 0..12 {
        let x = bezier_sample(x1, x2, t);
        let dx = bezier_slope(x1, x2, t);

        if dx.abs() < 1e-7 {
            break;
        }

        t = (t - (x - target_x) / dx).clamp(0.0, 1.0);
    }

    bezier_sample(y1, y2, t)
}

fn bezier_sample(p1: f32, p2: f32, t: f32) -> f32 {
    let inv = 1.0 - t;
    3.0 * inv * inv * t * p1 + 3.0 * inv * t * t * p2 + t * t * t
}

fn bezier_slope(p1: f32, p2: f32, t: f32) -> f32 {
    let inv = 1.0 - t;
    3.0 * inv * inv * p1 + 6.0 * inv * t * (p2 - p1) + 3.0 * t * t * (1.0 - p2)
}
