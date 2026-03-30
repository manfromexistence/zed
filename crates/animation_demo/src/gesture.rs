use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureAxis {
    Horizontal,
    Vertical,
    Free,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GesturePhase {
    Idle,
    Pressed,
    Dragging,
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GesturePoint {
    pub x: f32,
    pub y: f32,
}

impl GesturePoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragConstraint {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub elasticity: f32,
}

impl DragConstraint {
    pub fn clamp(&self, point: GesturePoint) -> GesturePoint {
        GesturePoint {
            x: elastic_clamp(point.x, self.min_x, self.max_x, self.elasticity),
            y: elastic_clamp(point.y, self.min_y, self.max_y, self.elasticity),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DragGesture {
    axis: GestureAxis,
    phase: GesturePhase,
    anchor: GesturePoint,
    current: GesturePoint,
    delta: GesturePoint,
    velocity: GesturePoint,
    started_at: Instant,
    last_update: Instant,
}

impl DragGesture {
    pub fn new(axis: GestureAxis) -> Self {
        let now = Instant::now();
        Self {
            axis,
            phase: GesturePhase::Idle,
            anchor: GesturePoint::new(0.0, 0.0),
            current: GesturePoint::new(0.0, 0.0),
            delta: GesturePoint::new(0.0, 0.0),
            velocity: GesturePoint::new(0.0, 0.0),
            started_at: now,
            last_update: now,
        }
    }

    pub fn begin(&mut self, point: GesturePoint) {
        let now = Instant::now();
        self.phase = GesturePhase::Pressed;
        self.anchor = point;
        self.current = point;
        self.delta = GesturePoint::new(0.0, 0.0);
        self.velocity = GesturePoint::new(0.0, 0.0);
        self.started_at = now;
        self.last_update = now;
    }

    pub fn update(&mut self, point: GesturePoint, constraint: Option<DragConstraint>) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32().max(0.0001);
        let constrained = constraint.map(|c| c.clamp(point)).unwrap_or(point);
        let raw_delta = GesturePoint::new(
            constrained.x - self.anchor.x,
            constrained.y - self.anchor.y,
        );

        self.phase = GesturePhase::Dragging;
        self.velocity = GesturePoint::new(
            (constrained.x - self.current.x) / dt,
            (constrained.y - self.current.y) / dt,
        );
        self.current = constrained;
        self.delta = match self.axis {
            GestureAxis::Horizontal => GesturePoint::new(raw_delta.x, 0.0),
            GestureAxis::Vertical => GesturePoint::new(0.0, raw_delta.y),
            GestureAxis::Free => raw_delta,
        };
        self.last_update = now;
    }

    pub fn end(&mut self) {
        self.phase = GesturePhase::Released;
    }

    pub fn reset(&mut self) {
        self.phase = GesturePhase::Idle;
        self.delta = GesturePoint::new(0.0, 0.0);
        self.velocity = GesturePoint::new(0.0, 0.0);
    }

    pub fn phase(&self) -> GesturePhase {
        self.phase
    }

    pub fn delta(&self) -> GesturePoint {
        self.delta
    }

    pub fn velocity(&self) -> GesturePoint {
        self.velocity
    }

    pub fn duration(&self) -> Duration {
        self.last_update.duration_since(self.started_at)
    }
}

#[derive(Clone, Debug)]
pub struct HoldGesture {
    threshold: Duration,
    pressed_at: Option<Instant>,
}

impl HoldGesture {
    pub fn new(threshold: Duration) -> Self {
        Self {
            threshold,
            pressed_at: None,
        }
    }

    pub fn press(&mut self) {
        self.pressed_at = Some(Instant::now());
    }

    pub fn release(&mut self) {
        self.pressed_at = None;
    }

    pub fn is_active(&self) -> bool {
        self.pressed_at
            .is_some_and(|pressed_at| pressed_at.elapsed() >= self.threshold)
    }
}

fn elastic_clamp(value: f32, min: f32, max: f32, elasticity: f32) -> f32 {
    if value < min {
        min + (value - min) * elasticity
    } else if value > max {
        max + (value - max) * elasticity
    } else {
        value
    }
}
