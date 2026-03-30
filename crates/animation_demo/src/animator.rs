use std::time::{Duration, Instant};

use crate::{EasingCurve, SpringConfig, SpringSolver};

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

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn set(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.driver = None;
        self.last_tick = Instant::now();
    }

    pub fn spring_to(&mut self, target: f32, config: SpringConfig) {
        if (self.target - target).abs() < 0.0001 && self.driver.is_some() {
            return;
        }

        match &mut self.driver {
            Some(AnimationDriver::Spring(solver)) => solver.retarget(target),
            _ => {
                self.driver = Some(AnimationDriver::Spring(SpringSolver::new(
                    config,
                    self.current,
                    target,
                )));
            }
        }

        self.target = target;
        self.last_tick = Instant::now();
    }

    pub fn ease_to(&mut self, target: f32, duration: Duration, curve: EasingCurve) {
        self.driver = Some(AnimationDriver::Eased {
            from: self.current,
            to: target,
            duration,
            curve,
            start: Instant::now(),
        });
        self.target = target;
        self.last_tick = Instant::now();
    }

    pub fn stop(&mut self) {
        self.driver = None;
        self.target = self.current;
    }

    pub fn is_animating(&self) -> bool {
        self.driver.is_some()
    }

    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32();
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
            Some(AnimationDriver::Eased {
                from,
                to,
                duration,
                curve,
                start,
            }) => {
                let elapsed = now.duration_since(*start);
                let duration_secs = duration.as_secs_f32().max(0.0001);
                let progress = (elapsed.as_secs_f32() / duration_secs).clamp(0.0, 1.0);
                self.current = lerp(*from, *to, curve.evaluate(progress));
                if progress >= 1.0 {
                    self.current = *to;
                    self.driver = None;
                    return false;
                }
                true
            }
            None => false,
        }
    }
}

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

    pub fn set(&mut self, x: f32, y: f32) {
        self.x.set(x);
        self.y.set(y);
    }

    pub fn spring_to(&mut self, x: f32, y: f32, config: SpringConfig) {
        self.x.spring_to(x, config);
        self.y.spring_to(y, config);
    }

    pub fn ease_to(&mut self, x: f32, y: f32, duration: Duration, curve: EasingCurve) {
        self.x.ease_to(x, duration, curve);
        self.y.ease_to(y, duration, curve);
    }

    pub fn tick(&mut self) -> bool {
        self.x.tick() || self.y.tick()
    }

    pub fn current(&self) -> (f32, f32) {
        (self.x.current(), self.y.current())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgba_u32(color: u32) -> Self {
        Self {
            r: ((color >> 24) & 0xFF) as f32 / 255.0,
            g: ((color >> 16) & 0xFF) as f32 / 255.0,
            b: ((color >> 8) & 0xFF) as f32 / 255.0,
            a: (color & 0xFF) as f32 / 255.0,
        }
    }

    pub fn to_rgba_u32(self) -> u32 {
        ((self.r.clamp(0.0, 1.0) * 255.0).round() as u32) << 24
            | ((self.g.clamp(0.0, 1.0) * 255.0).round() as u32) << 16
            | ((self.b.clamp(0.0, 1.0) * 255.0).round() as u32) << 8
            | (self.a.clamp(0.0, 1.0) * 255.0).round() as u32
    }
}

#[derive(Clone, Debug)]
pub struct AnimatedColor {
    pub r: AnimatedValue,
    pub g: AnimatedValue,
    pub b: AnimatedValue,
    pub a: AnimatedValue,
}

impl AnimatedColor {
    pub fn new(color: Rgba) -> Self {
        Self {
            r: AnimatedValue::new(color.r),
            g: AnimatedValue::new(color.g),
            b: AnimatedValue::new(color.b),
            a: AnimatedValue::new(color.a),
        }
    }

    pub fn from_rgba_u32(color: u32) -> Self {
        Self::new(Rgba::from_rgba_u32(color))
    }

    pub fn spring_to(&mut self, color: Rgba, config: SpringConfig) {
        self.r.spring_to(color.r, config);
        self.g.spring_to(color.g, config);
        self.b.spring_to(color.b, config);
        self.a.spring_to(color.a, config);
    }

    pub fn ease_to(&mut self, color: Rgba, duration: Duration, curve: EasingCurve) {
        self.r.ease_to(color.r, duration, curve);
        self.g.ease_to(color.g, duration, curve);
        self.b.ease_to(color.b, duration, curve);
        self.a.ease_to(color.a, duration, curve);
    }

    pub fn tick(&mut self) -> bool {
        self.r.tick() || self.g.tick() || self.b.tick() || self.a.tick()
    }

    pub fn current(&self) -> Rgba {
        Rgba::new(
            self.r.current(),
            self.g.current(),
            self.b.current(),
            self.a.current(),
        )
    }
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}
