use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringConfig {
    pub response: f32,
    pub damping_fraction: f32,
    pub mass: f32,
    pub initial_velocity: f32,
    pub rest_threshold: f32,
}

impl SpringConfig {
    pub const fn new(
        response: f32,
        damping_fraction: f32,
        mass: f32,
        initial_velocity: f32,
        rest_threshold: f32,
    ) -> Self {
        Self {
            response,
            damping_fraction,
            mass,
            initial_velocity,
            rest_threshold,
        }
    }

    pub const fn default_apple() -> Self {
        Self::new(0.55, 1.0, 1.0, 0.0, 0.001)
    }

    pub const fn snappy() -> Self {
        Self::new(0.3, 0.82, 1.0, 0.0, 0.001)
    }

    pub const fn bouncy() -> Self {
        Self::new(0.5, 0.6, 1.0, 0.0, 0.001)
    }

    pub const fn smooth() -> Self {
        Self::new(0.5, 1.0, 1.0, 0.0, 0.001)
    }

    pub const fn interactive() -> Self {
        Self::new(0.15, 1.0, 1.0, 0.0, 0.001)
    }

    pub const fn fluid() -> Self {
        Self::new(0.45, 0.78, 1.0, 0.0, 0.001)
    }

    pub const fn gentle() -> Self {
        Self::new(0.8, 1.0, 1.0, 0.0, 0.001)
    }

    pub fn from_duration_bounce(duration: f32, bounce: f32) -> Self {
        Self::new(duration, (1.0 - bounce).max(0.01), 1.0, 0.0, 0.001)
    }

    pub fn omega(self) -> f32 {
        ((2.0 * PI) / self.response.max(0.0001)) / self.mass.max(0.0001).sqrt()
    }

    pub fn settling_duration(self) -> f32 {
        let zeta = self.damping_fraction.max(0.01);
        -self.rest_threshold.ln() / (zeta * self.omega())
    }
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self::default_apple()
    }
}

#[derive(Clone, Debug)]
pub struct SpringSolver {
    config: SpringConfig,
    mode: SpringMode,
    from: f32,
    to: f32,
    elapsed: f32,
    settled: bool,
}

#[derive(Clone, Debug)]
enum SpringMode {
    UnderDamped {
        omega_d: f32,
        decay: f32,
        c1: f32,
        c2: f32,
    },
    CriticallyDamped {
        omega: f32,
        c1: f32,
        c2: f32,
    },
    OverDamped {
        r1: f32,
        r2: f32,
        c1: f32,
        c2: f32,
    },
}

impl SpringSolver {
    pub fn new(config: SpringConfig, from: f32, to: f32) -> Self {
        let mode = Self::mode_for(config, from, to, config.initial_velocity);
        Self {
            config,
            mode,
            from,
            to,
            elapsed: 0.0,
            settled: false,
        }
    }

    pub fn config(&self) -> SpringConfig {
        self.config
    }

    pub fn from(&self) -> f32 {
        self.from
    }

    pub fn to(&self) -> f32 {
        self.to
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn step(&mut self, delta_time: f32) -> f32 {
        if self.settled {
            return self.to;
        }

        self.elapsed += delta_time.max(0.0);
        let value = self.value_at(self.elapsed);
        let velocity = self.velocity_at(self.elapsed);
        let displacement = (value - self.to).abs();

        if displacement <= self.config.rest_threshold
            && velocity.abs() <= self.config.rest_threshold * 8.0
        {
            self.settled = true;
            self.elapsed = self.config.settling_duration();
            return self.to;
        }

        value
    }

    pub fn value_at(&self, t: f32) -> f32 {
        self.to + self.displacement_at(t)
    }

    pub fn velocity_at(&self, t: f32) -> f32 {
        match &self.mode {
            SpringMode::UnderDamped {
                omega_d,
                decay,
                c1,
                c2,
            } => {
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

    pub fn is_settled(&self) -> bool {
        self.settled
    }

    pub fn retarget(&mut self, to: f32) {
        let current = self.value_at(self.elapsed);
        let velocity = self.velocity_at(self.elapsed);
        self.from = current;
        self.to = to;
        self.elapsed = 0.0;
        self.settled = false;
        self.mode = Self::mode_for(self.config, current, to, velocity);
    }

    fn displacement_at(&self, t: f32) -> f32 {
        match &self.mode {
            SpringMode::UnderDamped {
                omega_d,
                decay,
                c1,
                c2,
            } => (-decay * t).exp() * (c1 * (omega_d * t).cos() + c2 * (omega_d * t).sin()),
            SpringMode::CriticallyDamped { omega, c1, c2 } => {
                (-omega * t).exp() * (c1 + c2 * t)
            }
            SpringMode::OverDamped { r1, r2, c1, c2 } => {
                c1 * (r1 * t).exp() + c2 * (r2 * t).exp()
            }
        }
    }

    fn mode_for(config: SpringConfig, from: f32, to: f32, velocity: f32) -> SpringMode {
        let displacement = from - to;
        let zeta = config.damping_fraction;
        let omega0 = config.omega();

        if zeta < 0.999 {
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            let decay = zeta * omega0;
            let c1 = displacement;
            let c2 = (velocity + decay * displacement) / omega_d;
            SpringMode::UnderDamped {
                omega_d,
                decay,
                c1,
                c2,
            }
        } else if zeta < 1.001 {
            let c1 = displacement;
            let c2 = velocity + omega0 * displacement;
            SpringMode::CriticallyDamped {
                omega: omega0,
                c1,
                c2,
            }
        } else {
            let sqrt_term = omega0 * (zeta * zeta - 1.0).sqrt();
            let r1 = -zeta * omega0 + sqrt_term;
            let r2 = -zeta * omega0 - sqrt_term;
            let c2 = (velocity - r1 * displacement) / (r2 - r1);
            let c1 = displacement - c2;
            SpringMode::OverDamped { r1, r2, c1, c2 }
        }
    }
}
