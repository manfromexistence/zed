use gpui::{Hsla, Pixels, Point, hsla, point, px};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
enum TypingEffectPreset {
    Particles,
    Fireworks,
    Flames,
    Magic,
}

const PRESETS: [TypingEffectPreset; 4] = [
    TypingEffectPreset::Particles,
    TypingEffectPreset::Fireworks,
    TypingEffectPreset::Flames,
    TypingEffectPreset::Magic,
];

#[derive(Clone, Debug)]
struct TypingParticle {
    origin: Point<Pixels>,
    velocity_x: f32,
    velocity_y: f32,
    gravity: f32,
    radius: f32,
    spawned_at: Instant,
    lifetime: Duration,
    hue: f32,
    saturation: f32,
    lightness: f32,
    alpha: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct TypingParticleLayout {
    pub origin: Point<Pixels>,
    pub size: Pixels,
    pub color: Hsla,
}

#[derive(Default)]
pub(crate) struct TypingEffectsState {
    particles: Vec<TypingParticle>,
    pending_bursts: usize,
    next_preset_ix: usize,
    burst_serial: u64,
}

impl TypingEffectsState {
    pub fn queue_burst(&mut self) {
        self.pending_bursts = self.pending_bursts.saturating_add(1);
    }

    pub fn has_pending_bursts(&self) -> bool {
        self.pending_bursts > 0
    }

    pub fn spawn_pending(&mut self, origin: Point<Pixels>, now: Instant) {
        while self.pending_bursts > 0 {
            let preset = PRESETS[self.next_preset_ix % PRESETS.len()];
            self.next_preset_ix = (self.next_preset_ix + 1) % PRESETS.len();
            self.burst_serial = self.burst_serial.wrapping_add(1);
            self.spawn_burst(origin, preset, now, self.burst_serial);
            self.pending_bursts -= 1;
        }
    }

    pub fn layout_particles(&mut self, now: Instant) -> Vec<TypingParticleLayout> {
        self.particles
            .retain(|particle| particle.spawned_at + particle.lifetime > now);

        self.particles
            .iter()
            .filter_map(|particle| particle.layout(now))
            .collect()
    }

    fn spawn_burst(
        &mut self,
        origin: Point<Pixels>,
        preset: TypingEffectPreset,
        now: Instant,
        serial: u64,
    ) {
        let particle_count = match preset {
            TypingEffectPreset::Particles => 14,
            TypingEffectPreset::Fireworks => 20,
            TypingEffectPreset::Flames => 16,
            TypingEffectPreset::Magic => 15,
        };

        self.particles.reserve(particle_count);

        for ix in 0..particle_count {
            let normalized_ix = ix as f32 / particle_count as f32;
            let angular_seed = ((serial as usize * 31 + ix * 17) % 360) as f32 / 360.0;
            let velocity_seed = ((serial as usize * 19 + ix * 29 + 11) % 100) as f32 / 100.0;
            let radius_seed = ((serial as usize * 13 + ix * 23 + 7) % 100) as f32 / 100.0;

            let particle = match preset {
                TypingEffectPreset::Particles => {
                    let angle = angular_seed * std::f32::consts::TAU;
                    TypingParticle {
                        origin,
                        velocity_x: angle.cos() * (28.0 + velocity_seed * 30.0),
                        velocity_y: angle.sin() * (24.0 + velocity_seed * 34.0),
                        gravity: 10.0,
                        radius: 2.8 + radius_seed * 3.1,
                        spawned_at: now,
                        lifetime: Duration::from_millis(420 + (velocity_seed * 240.0) as u64),
                        hue: (angular_seed + normalized_ix * 0.18) % 1.0,
                        saturation: 0.88,
                        lightness: 0.68,
                        alpha: 0.98,
                    }
                }
                TypingEffectPreset::Fireworks => {
                    let angle = normalized_ix * std::f32::consts::TAU;
                    TypingParticle {
                        origin,
                        velocity_x: angle.cos() * (36.0 + velocity_seed * 42.0),
                        velocity_y: angle.sin() * (36.0 + velocity_seed * 42.0),
                        gravity: 14.0,
                        radius: 3.0 + radius_seed * 2.8,
                        spawned_at: now,
                        lifetime: Duration::from_millis(560 + (velocity_seed * 300.0) as u64),
                        hue: (0.08 + normalized_ix * 0.75 + angular_seed * 0.1) % 1.0,
                        saturation: 0.92,
                        lightness: 0.68,
                        alpha: 0.96,
                    }
                }
                TypingEffectPreset::Flames => {
                    let sway = (normalized_ix - 0.5) * 30.0;
                    TypingParticle {
                        origin,
                        velocity_x: sway,
                        velocity_y: -(30.0 + velocity_seed * 32.0),
                        gravity: -4.0,
                        radius: 3.2 + radius_seed * 3.3,
                        spawned_at: now,
                        lifetime: Duration::from_millis(480 + (velocity_seed * 240.0) as u64),
                        hue: 0.03 + radius_seed * 0.09,
                        saturation: 0.96,
                        lightness: 0.60 + velocity_seed * 0.1,
                        alpha: 0.94,
                    }
                }
                TypingEffectPreset::Magic => {
                    let angle = angular_seed * std::f32::consts::TAU;
                    let hue = if ix % 2 == 0 {
                        0.55 + radius_seed * 0.1
                    } else {
                        0.74 + radius_seed * 0.08
                    };
                    TypingParticle {
                        origin,
                        velocity_x: angle.cos() * (18.0 + velocity_seed * 22.0),
                        velocity_y: angle.sin() * (16.0 + velocity_seed * 24.0) - 12.0,
                        gravity: 4.0,
                        radius: 2.6 + radius_seed * 2.6,
                        spawned_at: now,
                        lifetime: Duration::from_millis(620 + (velocity_seed * 320.0) as u64),
                        hue,
                        saturation: 0.86,
                        lightness: 0.74,
                        alpha: 0.93,
                    }
                }
            };

            self.particles.push(particle);
        }
    }
}

impl TypingParticle {
    fn layout(&self, now: Instant) -> Option<TypingParticleLayout> {
        let elapsed = now.saturating_duration_since(self.spawned_at);
        if elapsed >= self.lifetime {
            return None;
        }

        let age_secs = elapsed.as_secs_f32();
        let progress = age_secs / self.lifetime.as_secs_f32();
        let fade = (1.0 - progress).powf(1.45);
        let x = self.origin.x + px(self.velocity_x * age_secs);
        let y = self.origin.y
            + px(self.velocity_y * age_secs + 0.5 * self.gravity * age_secs * age_secs);
        let size = px(self.radius * (1.08 - progress * 0.22).max(0.78));

        Some(TypingParticleLayout {
            origin: point(x - size / 2.0, y - size / 2.0),
            size,
            color: hsla(
                self.hue % 1.0,
                self.saturation,
                self.lightness,
                self.alpha * fade,
            ),
        })
    }
}
