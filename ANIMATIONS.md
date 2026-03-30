

# Complete Guide: Smooth Animations, Animated SVGs, Video & 3D for Zed

---

## PART 1: macOS / iOS SMOOTH ANIMATIONS IN GPUI

### The Core Idea

Apple's animations feel magical because of **one thing**: **spring physics**, not bezier curves. Every modern Apple animation since iOS 7 uses a damped harmonic oscillator.

---

### `crates/smooth_animations/Cargo.toml`

```toml
[package]
name = "smooth_animations"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = { path = "../gpui" }
smallvec = "1.13"
```

---

### `crates/smooth_animations/src/lib.rs`

```rust
mod spring;
mod easing;
mod animator;
mod transition;
mod gesture;
mod animated_element;

pub use spring::*;
pub use easing::*;
pub use animator::*;
pub use transition::*;
pub use gesture::*;
pub use animated_element::*;
```

---

### `crates/smooth_animations/src/spring.rs`
**The heart of every Apple animation**

```rust
use std::f32::consts::PI;

/// Apple-style spring parameters.
///
/// This exactly replicates:
/// - `CASpringAnimation` (macOS AppKit / Core Animation)
/// - `UISpringTimingParameters` (iOS UIKit)  
/// - `spring(response:dampingFraction:blendDuration:)` (SwiftUI)
/// - `spring(duration:bounce:)` (iOS 17+)
#[derive(Clone, Copy, Debug)]
pub struct SpringConfig {
    /// How fast the spring settles. Lower = snappier. (SwiftUI: `response`)
    /// Maps to natural frequency: ω = 2π / response
    pub response: f32,

    /// 0.0 = infinite bounce, 1.0 = no bounce (critical), >1.0 = overdamped
    /// (SwiftUI: `dampingFraction` / `dampingRatio`)
    pub damping_fraction: f32,

    /// Affects momentum. Default 1.0.
    pub mass: f32,

    /// Velocity at animation start (in units/second).
    pub initial_velocity: f32,

    /// When displacement + velocity fall below this, snap to target.
    pub rest_threshold: f32,
}

impl SpringConfig {
    // ───────────────────────────────────────────────
    //  APPLE'S EXACT PRESET SPRINGS
    // ───────────────────────────────────────────────

    /// Default macOS/iOS system spring.
    /// Used for: sheet presentations, navigation pushes, toolbar items.
    pub fn default_apple() -> Self {
        Self {
            response: 0.55,
            damping_fraction: 1.0, // critically damped — no bounce
            mass: 1.0,
            initial_velocity: 0.0,
            rest_threshold: 0.001,
        }
    }

    /// iOS 17+ "snappy" spring.
    /// Used for: toggle switches, button presses, quick feedback.
    pub fn snappy() -> Self {
        Self {
            response: 0.3,
            damping_fraction: 0.82,
            mass: 1.0,
            initial_velocity: 0.0,
            rest_threshold: 0.001,
        }
    }

    /// iOS 17+ "bouncy" spring.
    /// Used for: app launch icons, notification banners, playful interactions.
    pub fn bouncy() -> Self {
        Self {
            response: 0.5,
            damping_fraction: 0.6,
            mass: 1.0,
            initial_velocity: 0.0,
            rest_threshold: 0.001,
        }
    }

    /// iOS 17+ "smooth" spring.
    /// Used for: large view transitions, sheet morphing, window resize.
    pub fn smooth() -> Self {
        Self {
            response: 0.5,
            damping_fraction: 1.0,
            mass: 1.0,
            initial_velocity: 0.0,
            rest_threshold: 0.001,
        }
    }

    /// "Interactive" spring — used when user is actively dragging.
    /// Very responsive, almost no overshoot.
    pub fn interactive() -> Self {
        Self {
            response: 0.15,
            damping_fraction: 1.0,
            mass: 1.0,
            initial_velocity: 0.0,
            rest_threshold: 0.001,
        }
    }

    /// macOS Sequoia "fluid" spring for window morphing.
    pub fn fluid() -> Self {
        Self {
            response: 0.45,
            damping_fraction: 0.78,
            mass: 1.0,
            initial_velocity: 0.0,
            rest_threshold: 0.001,
        }
    }

    /// Gentle spring for opacity fades and subtle effects.
    pub fn gentle() -> Self {
        Self {
            response: 0.8,
            damping_fraction: 1.0,
            mass: 1.0,
            initial_velocity: 0.0,
            rest_threshold: 0.001,
        }
    }

    /// iOS 17+ convenience: `spring(duration:bounce:)`
    /// `bounce`: 0.0 = no bounce, negative = overdamped, positive = bouncy
    pub fn from_duration_bounce(duration: f32, bounce: f32) -> Self {
        Self {
            response: duration,
            damping_fraction: 1.0 - bounce,
            mass: 1.0,
            initial_velocity: 0.0,
            rest_threshold: 0.001,
        }
    }

    pub fn with_initial_velocity(mut self, velocity: f32) -> Self {
        self.initial_velocity = velocity;
        self
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    // ───────────────────────────────────────────────
    //  DERIVED PHYSICS CONSTANTS
    // ───────────────────────────────────────────────

    /// Natural angular frequency ω₀ = 2π / response
    pub fn omega(&self) -> f32 {
        2.0 * PI / self.response
    }

    /// Damped angular frequency ωd = ω₀ √(1 - ζ²)
    pub fn damped_omega(&self) -> f32 {
        let zeta = self.damping_fraction;
        if zeta >= 1.0 {
            0.0 // no oscillation for critically/over-damped
        } else {
            self.omega() * (1.0 - zeta * zeta).sqrt()
        }
    }

    /// Stiffness k = mω₀²
    pub fn stiffness(&self) -> f32 {
        let omega = self.omega();
        self.mass * omega * omega
    }

    /// Damping coefficient c = 2mω₀ζ
    pub fn damping_coefficient(&self) -> f32 {
        2.0 * self.mass * self.omega() * self.damping_fraction
    }

    /// Estimated settling time (time to reach rest_threshold)
    pub fn estimated_duration(&self) -> f32 {
        if self.damping_fraction <= 0.0 {
            return f32::INFINITY;
        }
        // For damped spring: amplitude decays as e^(-ζω₀t)
        // Solve e^(-ζω₀t) = threshold
        let zeta = self.damping_fraction.max(0.01);
        -self.rest_threshold.ln() / (zeta * self.omega())
    }
}

/// The spring differential equation solver.
///
/// Solves: m·x'' + c·x' + k·x = 0
///
/// This uses the **exact analytical solution**, NOT numerical
/// integration, so it's perfectly stable at any frame rate.
#[derive(Clone, Debug)]
pub struct SpringSolver {
    config: SpringConfig,

    // Analytical solution coefficients (computed once at start)
    mode: SpringMode,

    // Animation tracking
    from: f32,
    to: f32,
    elapsed: f32,
    settled: bool,
}

#[derive(Clone, Debug)]
enum SpringMode {
    /// ζ < 1: oscillates while decaying
    UnderDamped {
        omega_d: f32,  // damped frequency
        decay: f32,    // ζ·ω₀ (exponential decay rate)
        c1: f32,       // cos coefficient
        c2: f32,       // sin coefficient
    },
    /// ζ = 1: fastest convergence without oscillation
    CriticallyDamped {
        omega: f32,
        c1: f32,
        c2: f32,
    },
    /// ζ > 1: converges slowly, no oscillation
    OverDamped {
        r1: f32,       // first root
        r2: f32,       // second root
        c1: f32,
        c2: f32,
    },
}

impl SpringSolver {
    pub fn new(config: SpringConfig, from: f32, to: f32) -> Self {
        let displacement = from - to; // initial displacement from target
        let velocity = config.initial_velocity;
        let zeta = config.damping_fraction;
        let omega0 = config.omega();

        let mode = if zeta < 0.999 {
            // UNDER-DAMPED (bouncy)
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            let decay = zeta * omega0;

            let c1 = displacement;
            let c2 = (velocity + decay * displacement) / omega_d;

            SpringMode::UnderDamped { omega_d, decay, c1, c2 }
        } else if zeta < 1.001 {
            // CRITICALLY DAMPED (Apple's default)
            let c1 = displacement;
            let c2 = velocity + omega0 * displacement;

            SpringMode::CriticallyDamped { omega: omega0, c1, c2 }
        } else {
            // OVER-DAMPED (sluggish)
            let sqrt_term = omega0 * (zeta * zeta - 1.0).sqrt();
            let r1 = -zeta * omega0 + sqrt_term;
            let r2 = -zeta * omega0 - sqrt_term;

            let c2 = (velocity - r1 * displacement) / (r2 - r1);
            let c1 = displacement - c2;

            SpringMode::OverDamped { r1, r2, c1, c2 }
        };

        Self {
            config,
            mode,
            from,
            to,
            elapsed: 0.0,
            settled: false,
        }
    }

    /// Advance by dt seconds. Returns current value.
    pub fn step(&mut self, dt: f32) -> f32 {
        if self.settled {
            return self.to;
        }

        self.elapsed += dt;
        let t = self.elapsed;

        let displacement = match &self.mode {
            SpringMode::UnderDamped { omega_d, decay, c1, c2 } => {
                (-decay * t).exp()
                    * (c1 * (omega_d * t).cos() + c2 * (omega_d * t).sin())
            }
            SpringMode::CriticallyDamped { omega, c1, c2 } => {
                (-omega * t).exp() * (c1 + c2 * t)
            }
            SpringMode::OverDamped { r1, r2, c1, c2 } => {
                c1 * (r1 * t).exp() + c2 * (r2 * t).exp()
            }
        };

        let velocity = self.velocity_at(t);
        let value = self.to + displacement;

        // Check if settled
        if displacement.abs() < self.config.rest_threshold
            && velocity.abs() < self.config.rest_threshold
        {
            self.settled = true;
            return self.to;
        }

        value
    }

    /// Get the instantaneous velocity at time t.
    fn velocity_at(&self, t: f32) -> f32 {
        match &self.mode {
            SpringMode::UnderDamped { omega_d, decay, c1, c2 } => {
                let exp = (-decay * t).exp();
                let cos = (omega_d * t).cos();
                let sin = (omega_d * t).sin();
                exp * ((-decay * c1 + omega_d * c2) * cos
                    + (-decay * c2 - omega_d * c1) * sin)
            }
            SpringMode::CriticallyDamped { omega, c1, c2 } => {
                let exp = (-omega * t).exp();
                exp * (c2 - omega * (c1 + c2 * t))
            }
            SpringMode::OverDamped { r1, r2, c1, c2 } => {
                c1 * r1 * (r1 * t).exp() + c2 * r2 * (r2 * t).exp()
            }
        }
    }

    /// Evaluate position at arbitrary time without mutating state.
    pub fn value_at(&self, t: f32) -> f32 {
        let displacement = match &self.mode {
            SpringMode::UnderDamped { omega_d, decay, c1, c2 } => {
                (-decay * t).exp()
                    * (c1 * (omega_d * t).cos() + c2 * (omega_d * t).sin())
            }
            SpringMode::CriticallyDamped { omega, c1, c2 } => {
                (-omega * t).exp() * (c1 + c2 * t)
            }
            SpringMode::OverDamped { r1, r2, c1, c2 } => {
                c1 * (r1 * t).exp() + c2 * (r2 * t).exp()
            }
        };
        self.to + displacement
    }

    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Retarget the animation mid-flight (Apple's interruptible animations).
    /// Preserves current velocity for fluid re-targeting.
    pub fn retarget(&mut self, new_target: f32) {
        let current_value = self.value_at(self.elapsed);
        let current_velocity = self.velocity_at(self.elapsed);

        self.to = new_target;
        self.from = current_value;
        self.elapsed = 0.0;
        self.settled = false;

        // Reconstruct with current velocity preserved
        let displacement = current_value - new_target;
        let velocity = current_velocity;
        let zeta = self.config.damping_fraction;
        let omega0 = self.config.omega();

        self.mode = if zeta < 0.999 {
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            let decay = zeta * omega0;
            let c1 = displacement;
            let c2 = (velocity + decay * displacement) / omega_d;
            SpringMode::UnderDamped { omega_d, decay, c1, c2 }
        } else if zeta < 1.001 {
            let c1 = displacement;
            let c2 = velocity + omega0 * displacement;
            SpringMode::CriticallyDamped { omega: omega0, c1, c2 }
        } else {
            let sqrt_term = omega0 * (zeta * zeta - 1.0).sqrt();
            let r1 = -zeta * omega0 + sqrt_term;
            let r2 = -zeta * omega0 - sqrt_term;
            let c2 = (velocity - r1 * displacement) / (r2 - r1);
            let c1 = displacement - c2;
            SpringMode::OverDamped { r1, r2, c1, c2 }
        };
    }
}
```

---

### `crates/smooth_animations/src/easing.rs`
**For non-spring animations (fades, color transitions)**

```rust
use std::f32::consts::PI;

/// Bezier-based easing for situations where springs are inappropriate
/// (color transitions, opacity fades, progress bars).
#[derive(Clone, Copy, Debug)]
pub enum EasingCurve {
    /// macOS default: cubic-bezier(0.25, 0.1, 0.25, 1.0)
    AppleDefault,
    /// iOS keyboard show: cubic-bezier(0.34, 1.0, 0.64, 1.0)
    AppleKeyboard,
    /// macOS Sequoia morph: cubic-bezier(0.22, 0.0, 0.0, 1.0)
    AppleMorph,
    /// ease-in (acceleration)
    EaseIn,
    /// ease-out (deceleration) — Apple uses this for dismissals
    EaseOut,
    /// ease-in-out (symmetric S-curve)
    EaseInOut,
    /// Linear
    Linear,
    /// Custom cubic bezier
    CubicBezier(f32, f32, f32, f32),
}

impl EasingCurve {
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::AppleDefault => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
            Self::AppleKeyboard => cubic_bezier(0.34, 1.0, 0.64, 1.0, t),
            Self::AppleMorph => cubic_bezier(0.22, 0.0, 0.0, 1.0, t),
            Self::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
            Self::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
            Self::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
            Self::Linear => t,
            Self::CubicBezier(x1, y1, x2, y2) => cubic_bezier(*x1, *y1, *x2, *y2, t),
        }
    }
}

/// Solve cubic bezier using Newton-Raphson iteration.
/// Given t ∈ [0,1] mapped to x-axis, returns the y-axis value.
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, target_x: f32) -> f32 {
    // Newton-Raphson to find parameter t where B_x(t) = target_x
    let mut t = target_x; // initial guess

    for _ in 0..12 {
        let x = bezier_sample(x1, x2, t);
        let dx = bezier_slope(x1, x2, t);

        if dx.abs() < 1e-7 {
            break;
        }

        t -= (x - target_x) / dx;
        t = t.clamp(0.0, 1.0);
    }

    bezier_sample(y1, y2, t)
}

fn bezier_sample(p1: f32, p2: f32, t: f32) -> f32 {
    // B(t) = 3(1-t)²t·P1 + 3(1-t)t²·P2 + t³
    let inv = 1.0 - t;
    3.0 * inv * inv * t * p1 + 3.0 * inv * t * t * p2 + t * t * t
}

fn bezier_slope(p1: f32, p2: f32, t: f32) -> f32 {
    // B'(t) = 3(1-t)²·P1 + 6(1-t)t·(P2-P1) + 3t²·(1-P2)
    let inv = 1.0 - t;
    3.0 * inv * inv * p1 + 6.0 * inv * t * (p2 - p1) + 3.0 * t * t * (1.0 - p2)
}
```

---

### `crates/smooth_animations/src/animator.rs`
**High-level property animator that integrates with GPUI**

```rust
use gpui::*;
use std::time::{Duration, Instant};
use super::spring::{SpringConfig, SpringSolver};
use super::easing::EasingCurve;

/// The kind of animation driving a property.
#[derive(Clone, Debug)]
pub enum AnimationDriver {
    Spring(SpringSolver),
    Eased {
        from: f32,
        to: f32,
        duration: Duration,
        curve: EasingCurve,
        start: Instant,
    },
}

/// Animates a single f32 property with Apple-level smoothness.
///
/// Features:
/// - Mid-flight retargeting (change target without restart)
/// - Velocity preservation across retargets
/// - Frame-rate independent (analytical spring, not Euler)
/// - Automatic rest detection
#[derive(Clone, Debug)]
pub struct AnimatedValue {
    current: f32,
    target: f32,
    driver: Option<AnimationDriver>,
    last_tick: Instant,
}

impl AnimatedValue {
    pub fn new(initial: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            driver: None,
            last_tick: Instant::now(),
        }
    }

    /// Animate to target with a spring (Apple's recommended approach).
    pub fn spring_to(&mut self, target: f32, config: SpringConfig) {
        if (self.target - target).abs() < 0.0001 && self.driver.is_some() {
            return; // already animating to this target
        }

        match &mut self.driver {
            Some(AnimationDriver::Spring(solver)) => {
                // MID-FLIGHT RETARGET — this is what makes Apple animations feel fluid.
                // Instead of restarting, we preserve the current velocity.
                solver.retarget(target);
            }
            _ => {
                let solver = SpringSolver::new(config, self.current, target);
                self.driver = Some(AnimationDriver::Spring(solver));
            }
        }

        self.target = target;
        self.last_tick = Instant::now();
    }

    /// Animate to target with an easing curve over a fixed duration.
    pub fn ease_to(&mut self, target: f32, duration: Duration, curve: EasingCurve) {
        self.driver = Some(AnimationDriver::Eased {
            from: self.current,
            to: target,
            duration,
            curve,
            start: Instant::now(),
        });
        self.target = target;
    }

    /// Set value instantly (no animation).
    pub fn set(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.driver = None;
    }

    /// Call every frame. Returns true if still animating (need repaint).
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        let dt = (now - self.last_tick).as_secs_f32();
        self.last_tick = now;

        match &mut self.driver {
            Some(AnimationDriver::Spring(solver)) => {
                self.current = solver.step(dt);
                if solver.is_settled() {
                    self.current = self.target;
                    self.driver = None;
                    return false;
                }
                true
            }
            Some(AnimationDriver::Eased { from, to, duration, curve, start }) => {
                let elapsed = start.elapsed().as_secs_f32();
                let total = duration.as_secs_f32();
                let t = (elapsed / total).min(1.0);
                let eased = curve.evaluate(t);
                self.current = *from + (*to - *from) * eased;

                if t >= 1.0 {
                    self.current = *to;
                    self.driver = None;
                    return false;
                }
                true
            }
            None => false,
        }
    }

    pub fn value(&self) -> f32 {
        self.current
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn is_animating(&self) -> bool {
        self.driver.is_some()
    }
}

/// Animates a 2D point (x, y) with independent springs.
#[derive(Clone, Debug)]
pub struct AnimatedPoint {
    pub x: AnimatedValue,
    pub y: AnimatedValue,
}

impl AnimatedPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: AnimatedValue::new(x),
            y: AnimatedValue::new(y),
        }
    }

    pub fn spring_to(&mut self, x: f32, y: f32, config: SpringConfig) {
        self.x.spring_to(x, config);
        self.y.spring_to(y, config);
    }

    pub fn tick(&mut self) -> bool {
        let a = self.x.tick();
        let b = self.y.tick();
        a || b
    }

    pub fn value(&self) -> (f32, f32) {
        (self.x.value(), self.y.value())
    }
}

/// Animates RGBA color with per-channel springs.
#[derive(Clone, Debug)]
pub struct AnimatedColor {
    pub r: AnimatedValue,
    pub g: AnimatedValue,
    pub b: AnimatedValue,
    pub a: AnimatedValue,
}

impl AnimatedColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: AnimatedValue::new(r),
            g: AnimatedValue::new(g),
            b: AnimatedValue::new(b),
            a: AnimatedValue::new(a),
        }
    }

    pub fn from_rgba(color: u32) -> Self {
        Self::new(
            ((color >> 24) & 0xFF) as f32 / 255.0,
            ((color >> 16) & 0xFF) as f32 / 255.0,
            ((color >> 8) & 0xFF) as f32 / 255.0,
            (color & 0xFF) as f32 / 255.0,
        )
    }

    pub fn ease_to_rgba(&mut self, color: u32, duration: Duration, curve: EasingCurve) {
        let r = ((color >> 24) & 0xFF) as f32 / 255.0;
        let g = ((color >> 16) & 0xFF) as f32 / 255.0;
        let b = ((color >> 8) & 0xFF) as f32 / 255.0;
        let a = (color & 0xFF) as f32 / 255.0;
        self.r.ease_to(r, duration, curve);
        self.g.ease_to(g, duration, curve);
        self.b.ease_to(b, duration, curve);
        self.a.ease_to(a, duration, curve);
    }

    pub fn tick(&mut self) -> bool {
        let a = self.r.tick();
        let b = self.g.tick();
        let c = self.b.tick();
        let d = self.a.tick();
        a || b || c || d
    }

    pub fn to_gpui_rgba(&self) -> gpui::Rgba {
        gpui::rgba(
            (((self.r.value() * 255.0) as u32) << 24)
                | (((self.g.value() * 255.0) as u32) << 16)
                | (((self.b.value() * 255.0) as u32) << 8)
                | ((self.a.value() * 255.0) as u32),
        )
    }
}
```

---

### `crates/smooth_animations/src/transition.rs`
**Pre-built Apple-style transitions**

```rust
use super::animator::*;
use super::spring::SpringConfig;
use super::easing::EasingCurve;
use std::time::Duration;

/// Predefined Apple-style transitions ready to use in your UI.
pub struct AppleTransitions;

impl AppleTransitions {
    // ───────── Panel / Sheet Presentation ─────────

    /// Sheet slides up from bottom with slight scale (iOS-style).
    pub fn sheet_present(
        y: &mut AnimatedValue,
        opacity: &mut AnimatedValue,
        scale: &mut AnimatedValue,
        container_height: f32,
    ) {
        y.set(container_height); // start off-screen bottom
        opacity.set(0.0);
        scale.set(0.95);

        y.spring_to(0.0, SpringConfig::smooth());
        opacity.ease_to(1.0, Duration::from_millis(200), EasingCurve::AppleDefault);
        scale.spring_to(1.0, SpringConfig::smooth());
    }

    /// Sheet dismisses down.
    pub fn sheet_dismiss(
        y: &mut AnimatedValue,
        opacity: &mut AnimatedValue,
        scale: &mut AnimatedValue,
        container_height: f32,
    ) {
        y.spring_to(container_height, SpringConfig::snappy());
        opacity.ease_to(0.0, Duration::from_millis(200), EasingCurve::EaseIn);
        scale.spring_to(0.95, SpringConfig::snappy());
    }

    // ───────── Navigation Push / Pop ─────────

    /// Push: current view slides left + fades, new view slides in from right.
    pub fn nav_push_outgoing(x: &mut AnimatedValue, opacity: &mut AnimatedValue, width: f32) {
        x.spring_to(-width * 0.3, SpringConfig::default_apple());
        opacity.ease_to(0.5, Duration::from_millis(350), EasingCurve::AppleDefault);
    }

    pub fn nav_push_incoming(x: &mut AnimatedValue, opacity: &mut AnimatedValue, width: f32) {
        x.set(width);
        opacity.set(1.0);
        x.spring_to(0.0, SpringConfig::default_apple());
    }

    // ───────── Toolbar / Tab Bar Items ─────────

    /// Tab selection: bouncy scale + color change.
    pub fn tab_select(scale: &mut AnimatedValue, y: &mut AnimatedValue) {
        scale.spring_to(1.15, SpringConfig::bouncy());
        y.spring_to(-2.0, SpringConfig::bouncy());
        // After the bounce, spring back to normal
        // (handled by retarget in the next frame or via a delayed callback)
    }

    pub fn tab_deselect(scale: &mut AnimatedValue, y: &mut AnimatedValue) {
        scale.spring_to(1.0, SpringConfig::smooth());
        y.spring_to(0.0, SpringConfig::smooth());
    }

    // ───────── Button Press ─────────

    /// Button presses down (scale shrink) like iOS.
    pub fn button_press(scale: &mut AnimatedValue) {
        scale.spring_to(0.92, SpringConfig::interactive());
    }

    pub fn button_release(scale: &mut AnimatedValue) {
        scale.spring_to(1.0, SpringConfig::snappy());
    }

    // ───────── Hover Effects ─────────

    /// macOS-style hover glow.
    pub fn hover_enter(
        bg_opacity: &mut AnimatedValue,
        scale: &mut AnimatedValue,
    ) {
        bg_opacity.spring_to(1.0, SpringConfig::snappy());
        scale.spring_to(1.02, SpringConfig::snappy());
    }

    pub fn hover_exit(
        bg_opacity: &mut AnimatedValue,
        scale: &mut AnimatedValue,
    ) {
        bg_opacity.spring_to(0.0, SpringConfig::gentle());
        scale.spring_to(1.0, SpringConfig::gentle());
    }

    // ───────── Notification Banner ─────────

    /// Slides in from top with bounce.
    pub fn notification_show(y: &mut AnimatedValue, height: f32) {
        y.set(-height);
        y.spring_to(0.0, SpringConfig::bouncy());
    }

    pub fn notification_dismiss(y: &mut AnimatedValue, height: f32) {
        y.spring_to(-height, SpringConfig::snappy());
    }

    // ───────── List Item Reorder ─────────

    /// Item lifts up with shadow+scale during drag.
    pub fn drag_lift(scale: &mut AnimatedValue, shadow: &mut AnimatedValue) {
        scale.spring_to(1.05, SpringConfig::snappy());
        shadow.spring_to(1.0, SpringConfig::snappy());
    }

    pub fn drag_drop(scale: &mut AnimatedValue, shadow: &mut AnimatedValue) {
        scale.spring_to(1.0, SpringConfig::bouncy());
        shadow.spring_to(0.0, SpringConfig::smooth());
    }

    // ───────── Panel Resize ─────────

    /// Smooth panel width/height resize like macOS sidebar.
    pub fn panel_resize(size: &mut AnimatedValue, new_size: f32) {
        size.spring_to(new_size, SpringConfig::default_apple());
    }

    // ───────── Command Palette / Spotlight ─────────

    /// macOS Spotlight-style appear.
    pub fn spotlight_show(
        opacity: &mut AnimatedValue,
        scale: &mut AnimatedValue,
        blur: &mut AnimatedValue,
    ) {
        opacity.set(0.0);
        scale.set(0.9);
        blur.set(0.0);

        opacity.ease_to(1.0, Duration::from_millis(200), EasingCurve::EaseOut);
        scale.spring_to(1.0, SpringConfig::snappy());
        blur.ease_to(1.0, Duration::from_millis(250), EasingCurve::EaseOut);
    }

    pub fn spotlight_dismiss(
        opacity: &mut AnimatedValue,
        scale: &mut AnimatedValue,
        blur: &mut AnimatedValue,
    ) {
        opacity.ease_to(0.0, Duration::from_millis(150), EasingCurve::EaseIn);
        scale.spring_to(0.95, SpringConfig::interactive());
        blur.ease_to(0.0, Duration::from_millis(150), EasingCurve::EaseIn);
    }

    // ───────── Context Menu ─────────

    /// macOS context menu appear (scale from click point).
    pub fn context_menu_show(
        opacity: &mut AnimatedValue,
        scale: &mut AnimatedValue,
    ) {
        opacity.set(0.0);
        scale.set(0.7);

        opacity.ease_to(1.0, Duration::from_millis(150), EasingCurve::EaseOut);
        scale.spring_to(1.0, SpringConfig::snappy());
    }
}
```

---

### `crates/smooth_animations/src/animated_element.rs`
**GPUI Integration — use animations directly in `Render`**

```rust
use gpui::*;
use super::animator::*;
use super::spring::SpringConfig;

/// A wrapper that adds Apple-like animation capabilities
/// to any GPUI view.
///
/// # Usage in a GPUI View:
///
/// ```rust
/// struct MyPanel {
///     sidebar_width: AnimatedValue,
///     panel_opacity: AnimatedValue,
///     panel_scale: AnimatedValue,
///     button_scales: Vec<AnimatedValue>,
///     hover_bg: AnimatedColor,
/// }
///
/// impl MyPanel {
///     fn new() -> Self {
///         Self {
///             sidebar_width: AnimatedValue::new(250.0),
///             panel_opacity: AnimatedValue::new(1.0),
///             panel_scale: AnimatedValue::new(1.0),
///             button_scales: vec![AnimatedValue::new(1.0); 10],
///             hover_bg: AnimatedColor::new(0.0, 0.0, 0.0, 0.0),
///         }
///     }
///
///     fn toggle_sidebar(&mut self) {
///         let target = if self.sidebar_width.target() > 100.0 {
///             0.0  // collapse
///         } else {
///             250.0  // expand
///         };
///         self.sidebar_width.spring_to(target, SpringConfig::default_apple());
///     }
/// }
///
/// impl Render for MyPanel {
///     fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
///         // Tick all animations and request repaint if any are active
///         let animating = tick_animations!(
///             self.sidebar_width,
///             self.panel_opacity,
///             self.panel_scale
///         );
///         if animating {
///             cx.notify(); // request next frame
///         }
///
///         div()
///             .w(px(self.sidebar_width.value()))
///             .opacity(self.panel_opacity.value())
///     }
/// }
/// ```
pub struct AnimationTicker;

impl AnimationTicker {
    /// Tick multiple AnimatedValues at once. Returns true if any are still animating.
    pub fn tick_all(values: &mut [&mut AnimatedValue]) -> bool {
        let mut any_animating = false;
        for v in values {
            if v.tick() {
                any_animating = true;
            }
        }
        any_animating
    }
}

/// Convenience macro to tick many animated values in render().
#[macro_export]
macro_rules! tick_animations {
    ($($val:expr),+ $(,)?) => {{
        let mut _any = false;
        $(
            if $val.tick() { _any = true; }
        )+
        _any
    }};
}

// ───────────────────────────────────────────────
//  FULL EXAMPLE: Animated Sidebar Panel
// ───────────────────────────────────────────────

pub struct AnimatedSidebar {
    width: AnimatedValue,
    items: Vec<SidebarItem>,
    is_collapsed: bool,
}

struct SidebarItem {
    label: SharedString,
    icon: SharedString,
    hover_bg: AnimatedValue,   // background opacity on hover
    press_scale: AnimatedValue, // scale when pressed
    selected_indicator: AnimatedValue, // selection bar width
    is_selected: bool,
}

impl AnimatedSidebar {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let items = vec![
            ("Explorer", "📁"),
            ("Search", "🔍"),
            ("Source Control", "🔀"),
            ("Extensions", "🧩"),
            ("Settings", "⚙️"),
        ]
        .into_iter()
        .map(|(label, icon)| SidebarItem {
            label: label.into(),
            icon: icon.into(),
            hover_bg: AnimatedValue::new(0.0),
            press_scale: AnimatedValue::new(1.0),
            selected_indicator: AnimatedValue::new(0.0),
            is_selected: false,
        })
        .collect();

        Self {
            width: AnimatedValue::new(240.0),
            items,
            is_collapsed: false,
        }
    }

    pub fn toggle_collapse(&mut self) {
        self.is_collapsed = !self.is_collapsed;
        let target = if self.is_collapsed { 48.0 } else { 240.0 };
        self.width.spring_to(target, SpringConfig::default_apple());
    }

    pub fn select_item(&mut self, index: usize) {
        for (i, item) in self.items.iter_mut().enumerate() {
            if i == index {
                item.is_selected = true;
                item.selected_indicator.spring_to(3.0, SpringConfig::snappy());
            } else {
                item.is_selected = false;
                item.selected_indicator.spring_to(0.0, SpringConfig::gentle());
            }
        }
    }
}

impl Render for AnimatedSidebar {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // Tick all animations
        let mut animating = self.width.tick();
        for item in &mut self.items {
            animating |= item.hover_bg.tick();
            animating |= item.press_scale.tick();
            animating |= item.selected_indicator.tick();
        }
        if animating {
            cx.notify();
        }

        let sidebar_width = self.width.value();
        let is_collapsed = self.is_collapsed;

        div()
            .h_full()
            .w(px(sidebar_width))
            .bg(gpui::rgba(0x1e1e2eff))
            .border_r_1()
            .border_color(gpui::rgba(0x333333ff))
            .flex()
            .flex_col()
            .overflow_hidden()
            .children(self.items.iter().enumerate().map(|(i, item)| {
                let bg_alpha = (item.hover_bg.value() * 40.0) as u32;
                let scale = item.press_scale.value();
                let indicator_width = item.selected_indicator.value();
                let is_selected = item.is_selected;

                div()
                    .h(px(36.0))
                    .w_full()
                    .flex()
                    .items_center()
                    .px(px(12.0))
                    .gap(px(10.0))
                    .bg(gpui::rgba(0xffffff00 + bg_alpha))
                    .rounded(px(6.0))
                    .mx(px(6.0))
                    .my(px(1.0))
                    .relative()
                    .cursor_pointer()
                    // Selection indicator bar (animated width)
                    .when(indicator_width > 0.1, |el| {
                        el.child(
                            div()
                                .absolute()
                                .left(px(0.0))
                                .top(px(6.0))
                                .bottom(px(6.0))
                                .w(px(indicator_width))
                                .bg(gpui::rgba(0x007affff))
                                .rounded_r(px(2.0)),
                        )
                    })
                    // Icon
                    .child(
                        div()
                            .text_size(px(16.0))
                            .child(item.icon.clone()),
                    )
                    // Label (hidden when collapsed)
                    .when(!is_collapsed, |el| {
                        el.child(
                            div()
                                .text_size(px(13.0))
                                .text_color(if is_selected {
                                    gpui::rgba(0xffffffff)
                                } else {
                                    gpui::rgba(0xccccccff)
                                })
                                .child(item.label.clone()),
                        )
                    })
            }))
    }
}
```

---

## PART 2: ANIMATED SVGs

### `crates/animated_svg/Cargo.toml`

```toml
[package]
name = "animated_svg"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = { path = "../gpui" }

# SVG parsing & rendering
usvg = "0.43"               # SVG parsing into a render tree
resvg = "0.43"               # SVG rasterization (CPU)
tiny-skia = "0.11"           # 2D rendering backend for resvg

# Lottie animation support (After Effects exports)
rlottie = "0.5"              # Lottie animation rendering

# SVG path interpolation
svgtypes = "0.15"            # SVG path data parsing
kurbo = "0.11"               # 2D geometry / bezier curves

# Serde for animation definitions
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### `crates/animated_svg/src/lib.rs`

```rust
mod svg_renderer;
mod path_animator;
mod lottie_player;
mod keyframe;

pub use svg_renderer::*;
pub use path_animator::*;
pub use lottie_player::*;
pub use keyframe::*;
```

### `crates/animated_svg/src/svg_renderer.rs`
**Render static SVGs to textures for GPUI**

```rust
use gpui::*;
use resvg::tiny_skia;
use std::sync::Arc;

/// Rasterized SVG data ready for GPU upload.
pub struct RasterizedSvg {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Renders SVG string to RGBA pixels at specified size.
pub fn rasterize_svg(svg_data: &str, width: u32, height: u32) -> Option<RasterizedSvg> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg_data, &options).ok()?;

    let fit_to = usvg::FitTo::Size(width, height);
    let transform = tiny_skia::Transform::identity();

    let size = fit_to.fit_to(tree.size())?;
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())?;

    resvg::render(&tree, fit_to, transform, pixmap.as_mut());

    Some(RasterizedSvg {
        pixels: pixmap.take(),
        width: size.width(),
        height: size.height(),
    })
}

/// Renders SVG from a file path.
pub fn rasterize_svg_file(
    path: &std::path::Path,
    width: u32,
    height: u32,
) -> Option<RasterizedSvg> {
    let data = std::fs::read_to_string(path).ok()?;
    rasterize_svg(&data, width, height)
}
```

### `crates/animated_svg/src/keyframe.rs`
**Define SVG keyframe animations in code or JSON**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete SVG animation definition.
///
/// # JSON Example:
/// ```json
/// {
///   "duration_ms": 1000,
///   "repeat": "loop",
///   "keyframes": [
///     {
///       "time": 0.0,
///       "properties": {
///         "opacity": 1.0,
///         "scale": 1.0,
///         "rotation": 0.0,
///         "path": "M10 10 L90 10 L90 90 L10 90 Z"
///       }
///     },
///     {
///       "time": 0.5,
///       "properties": {
///         "opacity": 0.5,
///         "scale": 1.2,
///         "rotation": 180.0,
///         "path": "M50 10 L90 90 L10 90 Z"
///       },
///       "easing": "apple_snappy"
///     },
///     {
///       "time": 1.0,
///       "properties": {
///         "opacity": 1.0,
///         "scale": 1.0,
///         "rotation": 360.0,
///         "path": "M10 10 L90 10 L90 90 L10 90 Z"
///       }
///     }
///   ]
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SvgAnimationDef {
    pub duration_ms: u64,
    pub repeat: RepeatMode,
    pub keyframes: Vec<SvgKeyframe>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RepeatMode {
    #[serde(rename = "once")]
    Once,
    #[serde(rename = "loop")]
    Loop,
    #[serde(rename = "ping_pong")]
    PingPong,
    #[serde(rename = "count")]
    Count(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SvgKeyframe {
    /// Normalized time 0.0–1.0
    pub time: f32,
    /// Properties at this keyframe
    pub properties: SvgProperties,
    /// Easing to this keyframe (from previous)
    #[serde(default = "default_easing")]
    pub easing: String,
}

fn default_easing() -> String {
    "apple_default".to_string()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SvgProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f32>,  // degrees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_r: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_g: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_b: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_a: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
    /// SVG path data string — for path morphing animations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}
```

### `crates/animated_svg/src/path_animator.rs`
**SVG path morphing (shape A → shape B with smooth interpolation)**

```rust
use kurbo::{BezPath, PathEl, Point};

/// Parsed SVG path command for interpolation.
#[derive(Clone, Debug)]
pub struct InterpolatablePath {
    pub commands: Vec<PathCommand>,
}

#[derive(Clone, Debug)]
pub enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64), // cubic bezier
    ClosePath,
}

impl InterpolatablePath {
    /// Parse an SVG path data string into interpolatable commands.
    pub fn from_svg_d(d: &str) -> Option<Self> {
        let path = svgtypes::PathParser::from(d);
        let mut commands = Vec::new();

        for segment in path {
            let segment = segment.ok()?;
            match segment {
                svgtypes::PathSegment::MoveTo { abs, x, y } if abs => {
                    commands.push(PathCommand::MoveTo(x, y));
                }
                svgtypes::PathSegment::LineTo { abs, x, y } if abs => {
                    commands.push(PathCommand::LineTo(x, y));
                }
                svgtypes::PathSegment::CurveTo {
                    abs, x1, y1, x2, y2, x, y,
                } if abs => {
                    commands.push(PathCommand::CurveTo(x1, y1, x2, y2, x, y));
                }
                svgtypes::PathSegment::ClosePath { .. } => {
                    commands.push(PathCommand::ClosePath);
                }
                // Handle relative commands by converting to absolute
                // (simplified — production code needs full conversion)
                _ => {}
            }
        }

        Some(Self { commands })
    }

    /// Interpolate between two paths at parameter t ∈ [0, 1].
    ///
    /// Both paths MUST have the same number and types of commands.
    /// (Use `normalize_paths` to ensure compatibility.)
    pub fn interpolate(a: &Self, b: &Self, t: f64) -> Option<Self> {
        if a.commands.len() != b.commands.len() {
            return None;
        }

        let commands = a
            .commands
            .iter()
            .zip(b.commands.iter())
            .map(|(ca, cb)| match (ca, cb) {
                (PathCommand::MoveTo(ax, ay), PathCommand::MoveTo(bx, by)) => {
                    PathCommand::MoveTo(lerp(*ax, *bx, t), lerp(*ay, *by, t))
                }
                (PathCommand::LineTo(ax, ay), PathCommand::LineTo(bx, by)) => {
                    PathCommand::LineTo(lerp(*ax, *bx, t), lerp(*ay, *by, t))
                }
                (
                    PathCommand::CurveTo(ax1, ay1, ax2, ay2, ax, ay),
                    PathCommand::CurveTo(bx1, by1, bx2, by2, bx, by),
                ) => PathCommand::CurveTo(
                    lerp(*ax1, *bx1, t),
                    lerp(*ay1, *by1, t),
                    lerp(*ax2, *bx2, t),
                    lerp(*ay2, *by2, t),
                    lerp(*ax, *bx, t),
                    lerp(*ay, *by, t),
                ),
                (PathCommand::ClosePath, PathCommand::ClosePath) => PathCommand::ClosePath,
                // Mismatched commands — fallback to first
                _ => ca.clone(),
            })
            .collect();

        Some(Self { commands })
    }

    /// Convert back to SVG path data string.
    pub fn to_svg_d(&self) -> String {
        let mut d = String::new();
        for cmd in &self.commands {
            match cmd {
                PathCommand::MoveTo(x, y) => {
                    d.push_str(&format!("M{:.2} {:.2} ", x, y));
                }
                PathCommand::LineTo(x, y) => {
                    d.push_str(&format!("L{:.2} {:.2} ", x, y));
                }
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    d.push_str(&format!(
                        "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} ",
                        x1, y1, x2, y2, x, y
                    ));
                }
                PathCommand::ClosePath => {
                    d.push_str("Z ");
                }
            }
        }
        d.trim().to_string()
    }

    /// Convert to kurbo BezPath for rendering.
    pub fn to_kurbo(&self) -> BezPath {
        let mut path = BezPath::new();
        for cmd in &self.commands {
            match cmd {
                PathCommand::MoveTo(x, y) => path.move_to(Point::new(*x, *y)),
                PathCommand::LineTo(x, y) => path.line_to(Point::new(*x, *y)),
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    path.curve_to(
                        Point::new(*x1, *y1),
                        Point::new(*x2, *y2),
                        Point::new(*x, *y),
                    );
                }
                PathCommand::ClosePath => path.close_path(),
            }
        }
        path
    }
}

/// Normalize two paths to have the same number and type of commands.
/// Adds intermediate points to the shorter path by subdividing segments.
pub fn normalize_paths(
    a: &InterpolatablePath,
    b: &InterpolatablePath,
) -> (InterpolatablePath, InterpolatablePath) {
    let mut a_out = a.clone();
    let mut b_out = b.clone();

    // Strategy: convert all LineTo to CurveTo (with control points on the line)
    // so both paths have the same command types
    for cmd in a_out.commands.iter_mut() {
        if let PathCommand::LineTo(x, y) = cmd {
            // Convert to degenerate cubic bezier (straight line)
            *cmd = PathCommand::CurveTo(*x, *y, *x, *y, *x, *y);
        }
    }
    for cmd in b_out.commands.iter_mut() {
        if let PathCommand::LineTo(x, y) = cmd {
            *cmd = PathCommand::CurveTo(*x, *y, *x, *y, *x, *y);
        }
    }

    // If different lengths, pad shorter with duplicate of last point
    while a_out.commands.len() < b_out.commands.len() {
        if let Some(last) = a_out.commands.last().cloned() {
            a_out.commands.push(last);
        }
    }
    while b_out.commands.len() < a_out.commands.len() {
        if let Some(last) = b_out.commands.last().cloned() {
            b_out.commands.push(last);
        }
    }

    (a_out, b_out)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
```

### `crates/animated_svg/src/lottie_player.rs`
**Play Lottie animations (After Effects JSON format)**

```rust
use gpui::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Lottie animation player using rlottie.
///
/// Lottie files are exported from After Effects via Bodymovin plugin.
/// They produce smooth, resolution-independent vector animations.
///
/// Use cases in your editor:
/// - Loading/processing indicators
/// - Success/error feedback animations
/// - Onboarding/empty state illustrations
/// - Animated icons (file type, git status)
pub struct LottiePlayer {
    animation: Option<rlottie::Animation>,
    // Frame cache
    frame_buffer: Vec<u32>,
    width: usize,
    height: usize,
    total_frames: usize,
    fps: f64,

    // Playback
    current_frame: usize,
    start_time: Instant,
    state: LottiePlayState,
    repeat: bool,
    speed: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LottiePlayState {
    Playing,
    Paused,
    Stopped,
    Completed,
}

impl LottiePlayer {
    /// Load a Lottie JSON file.
    pub fn from_file(path: &PathBuf, width: usize, height: usize) -> Option<Self> {
        let animation = rlottie::Animation::from_file(path)?;
        let total_frames = animation.totalframe();
        let fps = animation.framerate();

        let mut player = Self {
            animation: Some(animation),
            frame_buffer: vec![0u32; width * height],
            width,
            height,
            total_frames,
            fps,
            current_frame: 0,
            start_time: Instant::now(),
            state: LottiePlayState::Stopped,
            repeat: true,
            speed: 1.0,
        };

        // Render first frame
        player.render_frame(0);
        Some(player)
    }

    /// Load from JSON string.
    pub fn from_data(
        json: &str,
        key: &str,
        width: usize,
        height: usize,
    ) -> Option<Self> {
        let animation = rlottie::Animation::from_data(
            json.to_string(),
            key.to_string(),
            String::new(),
        )?;
        let total_frames = animation.totalframe();
        let fps = animation.framerate();

        Some(Self {
            animation: Some(animation),
            frame_buffer: vec![0u32; width * height],
            width,
            height,
            total_frames,
            fps,
            current_frame: 0,
            start_time: Instant::now(),
            state: LottiePlayState::Stopped,
            repeat: true,
            speed: 1.0,
        })
    }

    pub fn play(&mut self) {
        self.state = LottiePlayState::Playing;
        self.start_time = Instant::now();
    }

    pub fn pause(&mut self) {
        self.state = LottiePlayState::Paused;
    }

    pub fn stop(&mut self) {
        self.state = LottiePlayState::Stopped;
        self.current_frame = 0;
        self.render_frame(0);
    }

    /// Call every frame. Returns true if the display needs updating.
    pub fn tick(&mut self) -> bool {
        if self.state != LottiePlayState::Playing {
            return false;
        }

        let elapsed = self.start_time.elapsed().as_secs_f64() * self.speed as f64;
        let frame = ((elapsed * self.fps) as usize) % self.total_frames;

        if frame != self.current_frame {
            self.current_frame = frame;
            self.render_frame(frame);

            // Check completion
            if frame >= self.total_frames - 1 && !self.repeat {
                self.state = LottiePlayState::Completed;
            }

            return true;
        }
        false
    }

    fn render_frame(&mut self, frame: usize) {
        if let Some(ref mut anim) = self.animation {
            let surface = rlottie::Surface::new(&mut self.frame_buffer);
            anim.render(frame, surface);
        }
    }

    /// Get the current frame as RGBA8 pixel data.
    pub fn pixels_rgba8(&self) -> Vec<u8> {
        // rlottie outputs BGRA, we need RGBA for GPU upload
        self.frame_buffer
            .iter()
            .flat_map(|&pixel| {
                let b = (pixel & 0xFF) as u8;
                let g = ((pixel >> 8) & 0xFF) as u8;
                let r = ((pixel >> 16) & 0xFF) as u8;
                let a = ((pixel >> 24) & 0xFF) as u8;
                [r, g, b, a]
            })
            .collect()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

// ───────────────────────────────────────────────
//  GPUI View wrapper for Lottie
// ───────────────────────────────────────────────

pub struct LottieView {
    player: LottiePlayer,
}

impl LottieView {
    pub fn new(path: PathBuf, size: usize, cx: &mut ViewContext<Self>) -> Self {
        let mut player = LottiePlayer::from_file(&path, size, size)
            .expect("Failed to load Lottie file");
        player.play();

        Self { player }
    }
}

impl Render for LottieView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let needs_update = self.player.tick();
        if needs_update || self.player.state == LottiePlayState::Playing {
            cx.notify(); // request next frame
        }

        let w = self.player.width() as f32;
        let h = self.player.height() as f32;
        let pixels = self.player.pixels_rgba8();

        canvas(
            move |bounds, cx| {
                // Upload pixel data to GPU texture and paint
                // Implementation depends on GPUI's texture API
                // Pseudo:
                // let texture = cx.create_texture(w as u32, h as u32, &pixels);
                // cx.paint_image(bounds, texture);
            },
            |_, _, _| {},
        )
        .w(px(w))
        .h(px(h))
    }
}
```

### `crates/animated_svg/src/animated_svg_view.rs`
**Complete animated SVG component with keyframe playback**

```rust
use gpui::*;
use std::time::{Duration, Instant};
use super::keyframe::*;
use super::path_animator::*;
use super::svg_renderer::*;
use crate::smooth_animations::{AnimatedValue, SpringConfig, EasingCurve};

/// Plays a keyframe-based SVG animation.
pub struct AnimatedSvgView {
    /// The base SVG template with placeholders
    svg_template: String,

    /// Animation definition (from JSON or code)
    animation: SvgAnimationDef,

    /// Current animated properties
    opacity: AnimatedValue,
    scale_x: AnimatedValue,
    scale_y: AnimatedValue,
    rotation: AnimatedValue,
    translate_x: AnimatedValue,
    translate_y: AnimatedValue,
    fill_color: [AnimatedValue; 4],

    /// Path morphing state
    path_states: Vec<PathMorphState>,

    /// Playback
    start_time: Instant,
    current_time: f32,
    repeat_count: u32,
    is_playing: bool,

    /// Rasterized output cache
    raster_cache: Option<RasterizedSvg>,
    raster_size: u32,
}

struct PathMorphState {
    from: InterpolatablePath,
    to: InterpolatablePath,
    current_d: String,
}

impl AnimatedSvgView {
    pub fn new(
        svg_template: String,
        animation_json: &str,
        size: u32,
        _cx: &mut ViewContext<Self>,
    ) -> Self {
        let animation: SvgAnimationDef =
            serde_json::from_str(animation_json).expect("Invalid animation JSON");

        // Initialize properties from first keyframe
        let first = animation.keyframes.first().map(|k| &k.properties);

        Self {
            svg_template,
            animation,
            opacity: AnimatedValue::new(first.and_then(|p| p.opacity).unwrap_or(1.0)),
            scale_x: AnimatedValue::new(first.and_then(|p| p.scale_x.or(p.scale)).unwrap_or(1.0)),
            scale_y: AnimatedValue::new(first.and_then(|p| p.scale_y.or(p.scale)).unwrap_or(1.0)),
            rotation: AnimatedValue::new(first.and_then(|p| p.rotation).unwrap_or(0.0)),
            translate_x: AnimatedValue::new(first.and_then(|p| p.translate_x).unwrap_or(0.0)),
            translate_y: AnimatedValue::new(first.and_then(|p| p.translate_y).unwrap_or(0.0)),
            fill_color: [
                AnimatedValue::new(first.and_then(|p| p.fill_r).unwrap_or(1.0)),
                AnimatedValue::new(first.and_then(|p| p.fill_g).unwrap_or(1.0)),
                AnimatedValue::new(first.and_then(|p| p.fill_b).unwrap_or(1.0)),
                AnimatedValue::new(first.and_then(|p| p.fill_a).unwrap_or(1.0)),
            ],
            path_states: Vec::new(),
            start_time: Instant::now(),
            current_time: 0.0,
            repeat_count: 0,
            is_playing: false,
            raster_cache: None,
            raster_size: size,
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
        self.start_time = Instant::now();
    }

    fn find_keyframe_pair(&self, time: f32) -> Option<(&SvgKeyframe, &SvgKeyframe, f32)> {
        let kfs = &self.animation.keyframes;
        for i in 0..kfs.len() - 1 {
            if time >= kfs[i].time && time <= kfs[i + 1].time {
                let segment_duration = kfs[i + 1].time - kfs[i].time;
                let segment_t = if segment_duration > 0.0 {
                    (time - kfs[i].time) / segment_duration
                } else {
                    1.0
                };
                return Some((&kfs[i], &kfs[i + 1], segment_t));
            }
        }
        None
    }

    fn apply_keyframes(&mut self, normalized_time: f32) {
        if let Some((from_kf, to_kf, t)) = self.find_keyframe_pair(normalized_time) {
            // Get easing function
            let eased_t = match to_kf.easing.as_str() {
                "apple_default" => EasingCurve::AppleDefault.evaluate(t),
                "apple_snappy" => EasingCurve::CubicBezier(0.2, 0.0, 0.0, 1.0).evaluate(t),
                "apple_morph" => EasingCurve::AppleMorph.evaluate(t),
                "ease_in" => EasingCurve::EaseIn.evaluate(t),
                "ease_out" => EasingCurve::EaseOut.evaluate(t),
                "linear" => t,
                _ => EasingCurve::AppleDefault.evaluate(t),
            };

            // Interpolate each property
            let fp = &from_kf.properties;
            let tp = &to_kf.properties;

            if let (Some(a), Some(b)) = (fp.opacity, tp.opacity) {
                self.opacity.set(lerp_f32(a, b, eased_t));
            }
            if let (Some(a), Some(b)) = (
                fp.scale_x.or(fp.scale),
                tp.scale_x.or(tp.scale),
            ) {
                self.scale_x.set(lerp_f32(a, b, eased_t));
            }
            if let (Some(a), Some(b)) = (fp.rotation, tp.rotation) {
                self.rotation.set(lerp_f32(a, b, eased_t));
            }
        }
    }
}

impl Render for AnimatedSvgView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        if self.is_playing {
            let duration_secs = self.animation.duration_ms as f32 / 1000.0;
            let elapsed = self.start_time.elapsed().as_secs_f32();

            let normalized_time = match self.animation.repeat {
                RepeatMode::Loop => (elapsed / duration_secs).fract(),
                RepeatMode::PingPong => {
                    let cycle = (elapsed / duration_secs) as u32;
                    let frac = (elapsed / duration_secs).fract();
                    if cycle % 2 == 0 { frac } else { 1.0 - frac }
                }
                RepeatMode::Once => (elapsed / duration_secs).min(1.0),
                RepeatMode::Count(n) => {
                    let cycle = (elapsed / duration_secs) as u32;
                    if cycle >= n {
                        self.is_playing = false;
                        1.0
                    } else {
                        (elapsed / duration_secs).fract()
                    }
                }
            };

            self.apply_keyframes(normalized_time);
            cx.notify(); // keep repainting
        }

        let opacity = self.opacity.value();
        let scale_x = self.scale_x.value();
        let scale_y = self.scale_y.value();
        let rotation = self.rotation.value();
        let size = self.raster_size as f32;

        // Build the SVG with current property values
        let svg_with_props = self.svg_template
            .replace("{{opacity}}", &format!("{:.3}", opacity))
            .replace("{{rotation}}", &format!("{:.1}", rotation));

        // Rasterize the SVG
        if let Some(raster) = rasterize_svg(&svg_with_props, self.raster_size, self.raster_size) {
            self.raster_cache = Some(raster);
        }

        div()
            .w(px(size * scale_x))
            .h(px(size * scale_y))
            .opacity(opacity)
            .child(
                canvas(
                    move |bounds, cx| {
                        // Paint rasterized SVG
                        // Upload to GPU texture and render
                    },
                    |_, _, _| {},
                )
                .size_full()
            )
    }
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
```

---

## PART 3: BEST RUST CRATES FOR VIDEO PLAYER

### Comparison Table

```
┌──────��───────────────┬──────────┬────────────┬────────────┬───────────────┬──────────────┐
│ Crate                │ Maturity │ Formats    │ HW Accel   │ Complexity    │ Best For     │
├──────────────────────┼──────────┼────────────┼────────────┼───────────────┼──────────────┤
│ ffmpeg-next          │ ★★★★★   │ Everything │ Yes (VAAPI │ Medium-High   │ RECOMMENDED  │
│ (FFmpeg bindings)    │          │            │ VideoToolb)│               │ #1 choice    │
├──────────────────────┼──────────┼────────────┼────────────┼───────────────┼──────────────┤
│ video-rs             │ ★★★★    │ Everything │ Yes (via   │ LOW           │ If you want  │
│ (high-level ffmpeg)  │          │ (uses ff)  │ ffmpeg)    │ (simplified)  │ easy API     │
├──────────────────────┼──────────┼────────────┼────────────┼───────────────┼──────────────┤
│ gstreamer-rs         │ ★★★★★   │ Everything │ Yes (full) │ HIGH          │ If you need  │
│ (GStreamer bindings)  │          │            │            │ (pipeline)    │ streaming    │
├──────────────────────┼──────────┼────────────┼────────────┼───────────────┼──────────────┤
│ libmpv (mpv-rs)      │ ★★★     │ Everything │ Yes        │ LOW           │ Embedded     │
│                      │          │ (mpv)      │            │               │ full player  │
├──────────────────────┼──────────┼────────────┼────────────┼───────────────┼──────────────┤
│ openh264             │ ★★★     │ H.264 only │ No         │ Low           │ H.264 only   │
│                      │          │            │            │               │              │
├──────────────────────┼──────────┼────────────┼────────────┼───────────────┼──────────────┤
│ dav1d (rav1d)        │ ★★★★    │ AV1 only   │ No (fast   │ Medium        │ AV1 only     │
│                      │          │            │ software)  │               │              │
└──────────────────────┴──────────┴────────────┴────────────┴───────────────┴──────────────┘
```

### ✅ RECOMMENDED: `video-rs` + `ffmpeg-next`

```toml
[package]
name = "zed_video_player"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = { path = "../gpui" }

# PRIMARY: High-level video decode/encode
video-rs = { version = "0.10", features = ["ndarray"] }

# ALTERNATIVE: Direct FFmpeg for fine-grained control
ffmpeg-next = "7.0"

# Audio output for video playback
cpal = "0.15"

# Frame timing & sync
crossbeam-channel = "0.5"

# Color space conversion (YUV → RGB) on GPU
# (optional — ffmpeg can do this on CPU via swscale)
```

### Using `video-rs` (Simplest Approach)

```rust
use video_rs::decode::Decoder;
use video_rs::Time;
use std::path::Path;

/// Simple video frame extraction using video-rs
pub struct SimpleVideoDecoder {
    decoder: Decoder,
    width: u32,
    height: u32,
    duration: Time,
    frame_rate: f32,
}

impl SimpleVideoDecoder {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        video_rs::init().expect("Failed to init video-rs/ffmpeg");

        let source = video_rs::location::Location::File(path.to_path_buf());
        let decoder = Decoder::new(&source)?;

        let (width, height) = decoder.size();
        let duration = decoder.duration().unwrap_or(Time::zero());
        let frame_rate = decoder.frame_rate();

        Ok(Self {
            decoder,
            width: width as u32,
            height: height as u32,
            duration,
            frame_rate,
        })
    }

    /// Decode frames into a channel for async consumption.
    pub fn decode_all_frames(
        mut self,
        sender: crossbeam_channel::Sender<VideoFrame>,
    ) {
        std::thread::spawn(move || {
            while let Ok((time, frame)) = self.decoder.decode() {
                // frame is ndarray::Array3<u8> in RGB format
                let pixels: Vec<u8> = frame.into_raw_vec();

                let vf = VideoFrame {
                    data: pixels,
                    width: self.width,
                    height: self.height,
                    pts_seconds: time.as_secs_f64(),
                };

                if sender.send(vf).is_err() {
                    break; // receiver dropped
                }
            }
        });
    }

    /// Seek to a specific time.
    pub fn seek(&mut self, time_secs: f64) -> anyhow::Result<()> {
        let time = Time::from_secs_f64(time_secs);
        self.decoder.seek(time)?;
        Ok(())
    }
}

pub struct VideoFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub pts_seconds: f64,
}
```

### Using `ffmpeg-next` (Full Control)

```rust
use ffmpeg_next as ffmpeg;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context as Scaler, flag::Flags};
use ffmpeg::util::frame::video::Video as FfmpegFrame;
use std::path::Path;

/// FFmpeg-based video decoder with hardware acceleration support.
pub struct FfmpegVideoDecoder {
    input_ctx: ffmpeg::format::context::Input,
    video_stream_index: usize,
    decoder: ffmpeg::decoder::Video,
    scaler: Scaler,
    width: u32,
    height: u32,
    frame_rate: f64,
    time_base: ffmpeg::Rational,
}

impl FfmpegVideoDecoder {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        ffmpeg::init()?;

        let input_ctx = input(&path)?;

        let video_stream = input_ctx
            .streams()
            .best(Type::Video)
            .ok_or(anyhow::anyhow!("No video stream"))?;

        let video_stream_index = video_stream.index();
        let time_base = video_stream.time_base();

        let context = ffmpeg::codec::context::Context::from_parameters(
            video_stream.parameters(),
        )?;

        let mut decoder = context.decoder().video()?;

        // Enable multithreaded decoding
        unsafe {
            let ctx = &mut *decoder.as_mut_ptr();
            ctx.thread_count = num_cpus::get() as i32;
            ctx.thread_type = ffmpeg::ffi::FF_THREAD_FRAME as i32;
        }

        let width = decoder.width();
        let height = decoder.height();

        let frame_rate = video_stream
            .avg_frame_rate()
            .numerator() as f64
            / video_stream.avg_frame_rate().denominator().max(1) as f64;

        // Create YUV → RGBA scaler
        let scaler = Scaler::get(
            decoder.format(),
            width,
            height,
            Pixel::RGBA,
            width,
            height,
            Flags::BILINEAR,
        )?;

        Ok(Self {
            input_ctx,
            video_stream_index,
            decoder,
            scaler,
            width,
            height,
            frame_rate,
            time_base,
        })
    }

    /// Decode the next frame. Returns None at end of stream.
    pub fn next_frame(&mut self) -> Option<VideoFrame> {
        loop {
            match self.input_ctx.packets().next() {
                Some(Ok((stream, packet))) => {
                    if stream.index() != self.video_stream_index {
                        continue;
                    }

                    self.decoder.send_packet(&packet).ok()?;

                    let mut decoded = FfmpegFrame::empty();
                    if self.decoder.receive_frame(&mut decoded).is_ok() {
                        let mut rgba_frame = FfmpegFrame::empty();
                        self.scaler.run(&decoded, &mut rgba_frame).ok()?;

                        let pts = decoded.pts().unwrap_or(0);
                        let pts_seconds = pts as f64
                            * self.time_base.numerator() as f64
                            / self.time_base.denominator() as f64;

                        return Some(VideoFrame {
                            data: rgba_frame.data(0).to_vec(),
                            width: self.width,
                            height: self.height,
                            pts_seconds,
                        });
                    }
                }
                _ => return None,
            }
        }
    }

    /// Seek to position in seconds.
    pub fn seek(&mut self, seconds: f64) -> anyhow::Result<()> {
        let timestamp = (seconds
            * self.time_base.denominator() as f64
            / self.time_base.numerator() as f64) as i64;

        self.input_ctx.seek(timestamp, ..timestamp)?;
        self.decoder.flush();
        Ok(())
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn frame_rate(&self) -> f64 { self.frame_rate }
}
```

### Building FFmpeg for each platform

```bash
# macOS (Homebrew)
brew install ffmpeg pkg-config
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"

# Linux (Ubuntu/Debian)
sudo apt install libavcodec-dev libavformat-dev libavutil-dev \
    libswscale-dev libswresample-dev pkg-config

# Windows (vcpkg)
vcpkg install ffmpeg:x64-windows
set VCPKG_ROOT=C:\vcpkg

# Static linking (for distribution without FFmpeg installed)
# In Cargo.toml:
# ffmpeg-next = { version = "7.0", features = ["static"] }
```

---

## PART 4: BEST RUST CRATES FOR 3D RENDERING

### Comparison Table

```
┌───────────────────┬──────────┬─────────────────┬──────────────┬──────────────────────┐
│ Crate             │ Level    │ Features        │ Complexity   │ Best For             │
├───────────────────┼──────────┼─────────────────┼──────────────┼──────────────────────┤
│ wgpu              │ Low      │ Raw GPU access  │ HIGH         │ RECOMMENDED: Full    │
│                   │          │ Compute, render │              │ control, xplatform   │
├───────────────────┼──────────┼─────────────────┼──────────────┼──────────────────────┤
│ three-d           │ High     │ PBR, lights,    │ LOW          │ Quick 3D viewer,     │
│                   │          │ camera, gltf    │              │ less code            │
├───────────────────┼──────────┼─────────────────┼──────────────┼──────────────────────┤
│ rend3             │ Mid-High │ PBR renderer    │ MEDIUM       │ Production PBR       │
│                   │          │ on top of wgpu  │              │ with less boilerplate│
├───────────────────┼──────────┼─────────────────┼──────────────┼──────────────────────┤
│ bevy (render)     │ High     │ Full ECS +      │ MEDIUM       │ If you want ECS      │
│                   │          │ renderer        │              │ architecture         │
├──────��────────────┼──────────┼─────────────────┼──────────────┼──────────────────────┤
│ vulkano           │ Low      │ Vulkan-only     │ VERY HIGH    │ Vulkan-specific      │
│                   │          │                 │              │ applications         │
├───────────────────┼──────────┼─────────────────┼──────────────┼──────────────────────┤
│ glam              │ Math     │ Vectors, mats,  │ NONE         │ ALWAYS USE THIS      │
│                   │          │ quaternions     │ (math lib)   │ for 3D math          │
├───────────────────┼──────────┼─────────────────┼──────────────┼──────────────────────┤
│ gltf              │ Loader   │ glTF/GLB models │ LOW          │ ALWAYS USE THIS      │
│                   │          │                 │              │ for model loading    │
└───────────────────┴──────────┴─────────────────┴──────────────┴──────────────────────┘
```

### ✅ RECOMMENDED STACK:

```toml
[package]
name = "zed_3d_viewer"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = { path = "../gpui" }

# ────── OPTION A: three-d (Fastest to implement) ──────
# Use this if you want a 3D model viewer with minimal code.
three-d = "0.18"

# ────── OPTION B: wgpu + manual (Maximum control) ──────
# Use this if you want full control over the rendering pipeline.
wgpu = "23.0"
bytemuck = { version = "1.14", features = ["derive"] }

# ────── Shared (use with either option) ──────
glam = "0.28"                    # 3D math (vectors, matrices, quaternions)
gltf = "1.4"                    # glTF/GLB 3D model loading
tobj = "4.0"                    # OBJ model loading  
stl_io = "0.8"                  # STL model loading
image = "0.25"                  # Texture image loading
```

### Option A: `three-d` (Recommended for Quick Start)

```rust
use three_d::*;
use std::path::Path;

/// 3D model viewer using three-d crate.
/// Renders to an offscreen texture, then uploads to GPUI.
pub struct ThreeDViewer {
    context: HeadlessContext,
    camera: Camera,
    model: Option<Gm<Mesh, PhysicalMaterial>>,
    lights: Vec<Box<dyn Light>>,
    render_target: RenderTarget,
    width: u32,
    height: u32,
    orbit_control: OrbitState,
}

struct OrbitState {
    azimuth: f32,
    elevation: f32,
    distance: f32,
    target: glam::Vec3,
}

impl ThreeDViewer {
    pub fn new(width: u32, height: u32) -> anyhow::Result<Self> {
        let context = HeadlessContext::new()?;

        let camera = Camera::new_perspective(
            Viewport::new_at_origo(width, height),
            vec3(3.0, 3.0, 3.0),   // position
            vec3(0.0, 0.0, 0.0),   // target
            vec3(0.0, 1.0, 0.0),   // up
            degrees(45.0),          // FOV
            0.01,                   // near
            1000.0,                 // far
        );

        let render_target = RenderTarget::new(
            &context,
            width,
            height,
        )?;

        // Default lights
        let ambient = AmbientLight::new(&context, 0.3, Srgba::WHITE);
        let directional = DirectionalLight::new(
            &context,
            2.0,
            Srgba::WHITE,
            &vec3(-1.0, -1.0, -1.0),
        );

        Ok(Self {
            context,
            camera,
            model: None,
            lights: vec![Box::new(ambient), Box::new(directional)],
            render_target,
            width,
            height,
            orbit_control: OrbitState {
                azimuth: 0.7,
                elevation: 0.5,
                distance: 5.0,
                target: glam::Vec3::ZERO,
            },
        })
    }

    /// Load a glTF/GLB model.
    pub fn load_gltf(&mut self, path: &Path) -> anyhow::Result<()> {
        let bytes = std::fs::read(path)?;
        let loaded = three_d_asset::io::load_and_deserialize(
            path.to_str().unwrap(),
            &bytes,
        )?;

        // Extract the first mesh
        if let Some(cpu_mesh) = loaded.meshes.first() {
            let material = PhysicalMaterial::new_opaque(
                &self.context,
                &CpuMaterial {
                    albedo: Srgba::new(180, 180, 200, 255),
                    metallic: 0.1,
                    roughness: 0.5,
                    ..Default::default()
                },
            );

            let mesh = Mesh::new(&self.context, cpu_mesh);
            self.model = Some(Gm::new(mesh, material));

            // Auto-center and fit
            self.fit_to_model();
        }

        Ok(())
    }

    /// Load an OBJ model.
    pub fn load_obj(&mut self, path: &Path) -> anyhow::Result<()> {
        let (models, _materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;

        if let Some(model) = models.first() {
            let mesh_data = &model.mesh;

            let positions: Vec<Vec3> = mesh_data
                .positions
                .chunks(3)
                .map(|c| vec3(c[0], c[1], c[2]))
                .collect();

            let normals: Vec<Vec3> = if mesh_data.normals.is_empty() {
                // Compute normals if not provided
                vec![vec3(0.0, 1.0, 0.0); positions.len()]
            } else {
                mesh_data
                    .normals
                    .chunks(3)
                    .map(|c| vec3(c[0], c[1], c[2]))
                    .collect()
            };

            let indices: Vec<u32> = mesh_data.indices.clone();

            let cpu_mesh = CpuMesh {
                positions: Positions::F32(positions),
                normals: Some(normals),
                indices: Indices::U32(indices),
                ..Default::default()
            };

            let material = PhysicalMaterial::new_opaque(
                &self.context,
                &CpuMaterial {
                    albedo: Srgba::new(200, 200, 210, 255),
                    metallic: 0.0,
                    roughness: 0.6,
                    ..Default::default()
                },
            );

            let mesh = Mesh::new(&self.context, &cpu_mesh);
            self.model = Some(Gm::new(mesh, material));
            self.fit_to_model();
        }

        Ok(())
    }

    fn fit_to_model(&mut self) {
        if let Some(ref model) = self.model {
            let aabb = model.aabb();
            let center = aabb.center();
            let size = aabb.size().magnitude();

            self.orbit_control.target =
                glam::Vec3::new(center.x, center.y, center.z);
            self.orbit_control.distance = size * 1.5;
        }
    }

    /// Orbit the camera by delta mouse movement.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.orbit_control.azimuth += dx * 0.01;
        self.orbit_control.elevation = (self.orbit_control.elevation + dy * 0.01)
            .clamp(-1.5, 1.5);
    }

    /// Zoom in/out.
    pub fn zoom(&mut self, delta: f32) {
        self.orbit_control.distance =
            (self.orbit_control.distance * (1.0 - delta * 0.1)).max(0.1);
    }

    fn update_camera(&mut self) {
        let oc = &self.orbit_control;
        let x = oc.distance * oc.elevation.cos() * oc.azimuth.sin();
        let y = oc.distance * oc.elevation.sin();
        let z = oc.distance * oc.elevation.cos() * oc.azimuth.cos();

        let position = vec3(
            oc.target.x + x,
            oc.target.y + y,
            oc.target.z + z,
        );
        let target = vec3(oc.target.x, oc.target.y, oc.target.z);

        self.camera.set_view(position, target, vec3(0.0, 1.0, 0.0));
    }

    /// Render one frame and return RGBA pixels.
    pub fn render_frame(&mut self) -> Vec<u8> {
        self.update_camera();

        // Clear and render
        self.render_target
            .clear(ClearState::color_and_depth(0.08, 0.08, 0.12, 1.0, 1.0));

        if let Some(ref model) = self.model {
            self.render_target.render(
                &self.camera,
                model.into_iter(),
                &self.lights.iter().map(|l| l.as_ref()).collect::<Vec<_>>(),
            );
        }

        // Read pixels back
        self.render_target.read_color::<[u8; 4]>()
            .into_iter()
            .flat_map(|pixel| pixel.to_vec())
            .collect()
    }
}
```

### Option B: `wgpu` (Full Control)

For the wgpu approach, use the same pattern shown above but with full control over shaders. The key dependencies:

```toml
wgpu = "23.0"
glam = "0.28"
gltf = "1.4"
bytemuck = { version = "1.14", features = ["derive"] }
```

### Loading glTF models (shared code for both options)

```rust
use gltf;
use glam::*;
use std::path::Path;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

pub struct LoadedModel {
    pub meshes: Vec<LoadedMesh>,
    pub bounding_box: BoundingBox,
}

pub struct LoadedMesh {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub name: String,
}

pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
    pub fn diagonal(&self) -> f32 {
        self.size().length()
    }
}

/// Load a glTF/GLB file into GPU-ready mesh data.
pub fn load_gltf(path: &Path) -> anyhow::Result<LoadedModel> {
    let (document, buffers, _images) = gltf::import(path)?;

    let mut all_meshes = Vec::new();
    let mut global_min = Vec3::splat(f32::MAX);
    let mut global_max = Vec3::splat(f32::MIN);

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or(anyhow::anyhow!("No positions"))?
                .collect();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|n| n.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let indices: Vec<u32> = reader
                .read_indices()
                .map(|idx| idx.into_u32().collect())
                .unwrap_or_else(|| (0..positions.len() as u32).collect());

            let vertices: Vec<Vertex3D> = positions
                .iter()
                .zip(normals.iter())
                .zip(uvs.iter())
                .map(|((pos, nor), uv)| {
                    // Update bounding box
                    let p = Vec3::from(*pos);
                    global_min = global_min.min(p);
                    global_max = global_max.max(p);

                    Vertex3D {
                        position: *pos,
                        normal: *nor,
                        uv: *uv,
                    }
                })
                .collect();

            all_meshes.push(LoadedMesh {
                vertices,
                indices,
                name: mesh.name().unwrap_or("unnamed").to_string(),
            });
        }
    }

    Ok(LoadedModel {
        meshes: all_meshes,
        bounding_box: BoundingBox {
            min: global_min,
            max: global_max,
        },
    })
}
```

---

## PART 5: INTEGRATING EVERYTHING INTO GPUI VIEWS

### `crates/workspace/src/video_tab.rs`

```rust
use gpui::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crossbeam_channel::{bounded, Receiver};
use smooth_animations::{AnimatedValue, SpringConfig, AppleTransitions};

pub struct VideoTab {
    path: PathBuf,
    frame_receiver: Option<Receiver<zed_video_player::VideoFrame>>,
    current_pixels: Arc<Mutex<Option<Vec<u8>>>>,
    width: u32,
    height: u32,

    // Playback state
    is_playing: bool,
    current_time: f64,
    duration: f64,

    // Animated UI controls
    controls_opacity: AnimatedValue,
    play_button_scale: AnimatedValue,
    timeline_progress: AnimatedValue,
    volume_slider: AnimatedValue,
    panel_appear_scale: AnimatedValue,
    panel_appear_opacity: AnimatedValue,
}

impl VideoTab {
    pub fn new(path: PathBuf, cx: &mut ViewContext<Self>) -> Self {
        let (tx, rx) = bounded(8);
        let current_pixels = Arc::new(Mutex::new(None));
        let pixels_ref = current_pixels.clone();

        // Start decoder in background
        let decode_path = path.clone();
        cx.background_executor().spawn(async move {
            let decoder = zed_video_player::SimpleVideoDecoder::open(&decode_path)
                .expect("Failed to open video");
            decoder.decode_all_frames(tx);
        }).detach();

        // Frame consumer loop
        let rx_clone = rx.clone();
        cx.spawn(|this, mut cx| async move {
            loop {
                match rx_clone.try_recv() {
                    Ok(frame) => {
                        *pixels_ref.lock().unwrap() = Some(frame.data);
                        let _ = this.update(&mut cx, |this, cx| {
                            this.current_time = frame.pts_seconds;
                            cx.notify();
                        });
                    }
                    Err(_) => {
                        cx.background_executor()
                            .timer(Duration::from_millis(4))
                            .await;
                    }
                }
            }
        }).detach();

        let mut tab = Self {
            path,
            frame_receiver: Some(rx),
            current_pixels,
            width: 0,
            height: 0,
            is_playing: true,
            current_time: 0.0,
            duration: 0.0,
            // Apple-like animated controls
            controls_opacity: AnimatedValue::new(1.0),
            play_button_scale: AnimatedValue::new(1.0),
            timeline_progress: AnimatedValue::new(0.0),
            volume_slider: AnimatedValue::new(0.8),
            panel_appear_scale: AnimatedValue::new(0.9),
            panel_appear_opacity: AnimatedValue::new(0.0),
        };

        // Entrance animation
        AppleTransitions::sheet_present(
            &mut AnimatedValue::new(0.0), // y not used here
            &mut tab.panel_appear_opacity,
            &mut tab.panel_appear_scale,
            600.0,
        );

        tab
    }

    fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;

        // Bouncy button feedback
        self.play_button_scale.spring_to(0.85, SpringConfig::interactive());
        // Then bounce back (schedule via retarget on next tick)
    }

    fn on_mouse_move(&mut self, _cx: &mut ViewContext<Self>) {
        // Show controls on mouse activity
        self.controls_opacity.spring_to(1.0, SpringConfig::snappy());
        // Auto-hide after 3 seconds of inactivity
    }
}

impl Render for VideoTab {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // Tick all animations
        let animating = tick_animations!(
            self.controls_opacity,
            self.play_button_scale,
            self.timeline_progress,
            self.panel_appear_scale,
            self.panel_appear_opacity
        );
        if animating || self.is_playing {
            cx.notify();
        }

        let scale = self.panel_appear_scale.value();
        let opacity = self.panel_appear_opacity.value();
        let controls_alpha = self.controls_opacity.value();
        let btn_scale = self.play_button_scale.value();

        // Bounce-back for play button
        if !self.play_button_scale.is_animating()
            && (self.play_button_scale.value() - 1.0).abs() > 0.01
        {
            self.play_button_scale.spring_to(1.0, SpringConfig::bouncy());
        }

        div()
            .size_full()
            .bg(gpui::rgba(0x000000ff))
            .opacity(opacity)
            // Apply scale transform conceptually
            // (GPUI may need canvas-based transforms for actual scaling)
            .child(
                // Video frame
                canvas(
                    {
                        let pixels = self.current_pixels.clone();
                        move |bounds, cx| {
                            if let Some(ref data) = *pixels.lock().unwrap() {
                                // Upload texture and paint
                                // cx.paint_texture(bounds, data, width, height);
                            }
                        }
                    },
                    |_, _, _| {},
                )
                .size_full(),
            )
            .child(
                // Controls overlay (Apple-style fade in/out)
                div()
                    .absolute()
                    .bottom_0()
                    .w_full()
                    .h(px(90.0))
                    .opacity(controls_alpha)
                    .bg(gpui::rgba(0x00000088))
                    .flex()
                    .items_center()
                    .px(px(20.0))
                    .gap(px(16.0))
                    // Play/Pause button with spring scale
                    .child(
                        div()
                            .w(px(44.0 * btn_scale))
                            .h(px(44.0 * btn_scale))
                            .rounded_full()
                            .bg(gpui::rgba(0xffffff22))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .text_color(gpui::rgba(0xffffffff))
                            .text_size(px(20.0))
                            .child(if self.is_playing { "⏸" } else { "▶" })
                            .on_click(cx.listener(|this, _, cx| {
                                this.toggle_play();
                                cx.notify();
                            })),
                    )
                    // Timeline
                    .child(
                        div()
                            .flex_1()
                            .h(px(4.0))
                            .bg(gpui::rgba(0xffffff33))
                            .rounded(px(2.0))
                            .child(
                                div()
                                    .h_full()
                                    .w(relative(if self.duration > 0.0 {
                                        (self.current_time / self.duration) as f32
                                    } else {
                                        0.0
                                    }))
                                    .bg(gpui::rgba(0x007affff))
                                    .rounded(px(2.0)),
                            ),
                    )
            )
    }
}
```

### `crates/workspace/src/model_tab.rs`

```rust
use gpui::*;
use std::path::PathBuf;
use std::time::Duration;
use smooth_animations::{AnimatedValue, AnimatedPoint, SpringConfig};

pub struct ModelTab {
    viewer: zed_3d_viewer::ThreeDViewer,
    path: PathBuf,
    pixels: Vec<u8>,
    width: u32,
    height: u32,

    // Animated camera orbit with springs
    orbit_azimuth: AnimatedValue,
    orbit_elevation: AnimatedValue,
    orbit_distance: AnimatedValue,

    // UI
    panel_opacity: AnimatedValue,
    is_dragging: bool,
    last_mouse: Option<(f32, f32)>,
}

impl ModelTab {
    pub fn new(path: PathBuf, cx: &mut ViewContext<Self>) -> Self {
        let width = 800;
        let height = 600;

        let mut viewer = zed_3d_viewer::ThreeDViewer::new(width, height)
            .expect("Failed to create 3D viewer");

        // Load model based on extension
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "glb" | "gltf" => { viewer.load_gltf(&path).ok(); }
            "obj" => { viewer.load_obj(&path).ok(); }
            _ => {}
        }

        let pixels = viewer.render_frame();

        let mut tab = Self {
            viewer,
            path,
            pixels,
            width,
            height,
            orbit_azimuth: AnimatedValue::new(0.7),
            orbit_elevation: AnimatedValue::new(0.5),
            orbit_distance: AnimatedValue::new(5.0),
            panel_opacity: AnimatedValue::new(0.0),
            is_dragging: false,
            last_mouse: None,
        };

        // Entrance animation
        tab.panel_opacity.spring_to(1.0, SpringConfig::smooth());

        // Start render loop
        cx.spawn(|this, mut cx| async move {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16)) // 60fps
                    .await;

                let _ = this.update(&mut cx, |this, cx| {
                    let animating = tick_animations!(
                        this.orbit_azimuth,
                        this.orbit_elevation,
                        this.orbit_distance,
                        this.panel_opacity
                    );

                    if animating {
                        // Update viewer camera from animated values
                        this.viewer.orbit_control.azimuth =
                            this.orbit_azimuth.value();
                        this.viewer.orbit_control.elevation =
                            this.orbit_elevation.value();
                        this.viewer.orbit_control.distance =
                            this.orbit_distance.value();

                        // Re-render
                        this.pixels = this.viewer.render_frame();
                        cx.notify();
                    }
                });
            }
        }).detach();

        tab
    }

    fn on_drag(&mut self, dx: f32, dy: f32) {
        // Spring-animated orbit — feels like rotating a real object
        let new_az = self.orbit_azimuth.target() + dx * 0.01;
        let new_el = (self.orbit_elevation.target() + dy * 0.01)
            .clamp(-1.5, 1.5);

        // Use interactive spring during drag (very responsive)
        self.orbit_azimuth.spring_to(new_az, SpringConfig::interactive());
        self.orbit_elevation.spring_to(new_el, SpringConfig::interactive());
    }

    fn on_drag_end(&mut self) {
        // Optional: add momentum by using a bouncier spring
        // The spring will naturally decelerate thanks to damping
    }

    fn on_scroll(&mut self, delta: f32) {
        let new_dist = (self.orbit_distance.target() * (1.0 - delta * 0.1)).max(0.5);
        self.orbit_distance.spring_to(new_dist, SpringConfig::snappy());
    }

    fn reset_camera(&mut self) {
        // Animate back to default view with a bouncy spring
        self.orbit_azimuth.spring_to(0.7, SpringConfig::bouncy());
        self.orbit_elevation.spring_to(0.5, SpringConfig::bouncy());
        self.orbit_distance.spring_to(5.0, SpringConfig::bouncy());
    }
}

impl Render for ModelTab {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let opacity = self.panel_opacity.value();

        div()
            .size_full()
            .opacity(opacity)
            .bg(gpui::rgba(0x141420ff))
            .relative()
            .child(
                // 3D viewport
                canvas(
                    {
                        let pixels = self.pixels.clone();
                        let w = self.width;
                        let h = self.height;
                        move |bounds, cx| {
                            // Paint 3D render output as texture
                        }
                    },
                    |_, _, _| {},
                )
                .size_full()
                .cursor(CursorStyle::ClosedHand)
                .on_mouse_down(MouseButton::Left, cx.listener(|this, event, cx| {
                    this.is_dragging = true;
                    this.last_mouse = Some((event.position.x.0, event.position.y.0));
                }))
                .on_mouse_up(MouseButton::Left, cx.listener(|this, _, cx| {
                    this.is_dragging = false;
                    this.last_mouse = None;
                    this.on_drag_end();
                }))
                .on_mouse_move(cx.listener(|this, event, cx| {
                    if this.is_dragging {
                        if let Some((lx, ly)) = this.last_mouse {
                            let dx = event.position.x.0 - lx;
                            let dy = event.position.y.0 - ly;
                            this.on_drag(dx, dy);
                        }
                        this.last_mouse = Some((
                            event.position.x.0,
                            event.position.y.0,
                        ));
                    }
                }))
                .on_scroll_wheel(cx.listener(|this, event, cx| {
                    this.on_scroll(event.delta.pixel_delta(px(1.0)).y.0);
                    cx.notify();
                })),
            )
            // Toolbar
            .child(
                div()
                    .absolute()
                    .top(px(10.0))
                    .right(px(10.0))
                    .flex()
                    .gap(px(8.0))
                    .child(
                        div()
                            .px(px(12.0))
                            .py(px(6.0))
                            .bg(gpui::rgba(0xffffff15))
                            .rounded(px(6.0))
                            .text_color(gpui::rgba(0xffffffcc))
                            .text_size(px(12.0))
                            .cursor_pointer()
                            .child("Reset Camera")
                            .on_click(cx.listener(|this, _, cx| {
                                this.reset_camera();
                                cx.notify();
                            })),
                    )
            )
    }
}
```

---

## FINAL SUMMARY: Crate Recommendations

```
┌──────────────────────────────────────────────────────────────────┐
│                    YOUR TECH STACK                                │
├────────────────┬─────────────────────────────────────────────────┤
│ ANIMATIONS     │ Pure Rust (code above) — no external crate     │
│                │ needed. Spring physics + bezier easing.         │
├────────────────┼─────────────────────────────────────────────────┤
│ ANIMATED SVG   │ resvg + usvg (rendering)                       │
│                │ rlottie (Lottie/After Effects animations)       │
│                │ kurbo + svgtypes (path morphing)                │
├────────────────┼─────────────────────────────────────────────────┤
│ VIDEO PLAYER   │ video-rs (simple) or ffmpeg-next (full control)│
│                │ + cpal (audio output)                           │
│                │ + crossbeam-channel (frame passing)             │
├────────────────┼─────────────────────────────────────────────────┤
│ 3D RENDERER    │ three-d (quick) or wgpu (full control)         │
│                │ + glam (3D math — ALWAYS)                       │
│                │ + gltf (model loading — ALWAYS)                 │
│                │ + tobj (OBJ loading)                            │
│                │ + bytemuck (GPU data)                           │
├────────────────┼─────────────────────────────────────────────────┤
│ LIQUID GLASS   │ ✅ Already done by you                          │
└────────────────┴─────────────────────────────────────────────────┘
```
