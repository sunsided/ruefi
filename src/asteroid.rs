use crate::rand::XorShift64;
use libm::{cosf, sinf};

/// Asteroid representation: jagged hexagon with per-vertex radial jitter
pub struct Asteroid {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub base_angle: f32,  // orientation for the hexagon
    pub jitter: [f32; 6], // multiplicative per-vertex radius factors
}

impl Asteroid {
    /// Create a randomly shaped and placed asteroid, away from the screen center.
    /// Uses the provided RNG; world axes: X right, Y down.
    pub fn random_spawn(rng: &mut XorShift64, sw: usize, sh: usize) -> Self {
        let cx = (sw / 2) as f32;
        let cy = (sh / 2) as f32;
        let min_dist = 120.0f32; // keep spawn away from player start
        let max_radius = ((sw.min(sh)) as f32) * 0.08 + 12.0;

        // pick a position away from the center
        let (x, y) = loop {
            let x = rng.range_f32(0.0, sw as f32);
            let y = rng.range_f32(0.0, sh as f32);
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy >= min_dist * min_dist {
                break (x, y);
            }
        };

        let radius = rng.range_f32(18.0, max_radius);
        // small random velocity
        let speed = rng.range_f32(0.5, 2.0);
        let dir = rng.range_f32(0.0, 2.0 * core::f32::consts::PI);
        let vx = -sinf(dir) * speed;
        let vy = cosf(dir) * speed;
        let base_angle = rng.range_f32(0.0, 2.0 * core::f32::consts::PI);
        let mut jitter = [1.0f32; 6];
        for j in 0..6 {
            jitter[j] = rng.range_f32(0.75, 1.25);
        }

        Self {
            x,
            y,
            vx,
            vy,
            radius,
            base_angle,
            jitter,
        }
    }

    /// Advance asteroid by velocity and wrap around screen bounds (toroidal world).
    #[inline]
    pub fn update(&mut self, sw: f32, sh: f32) {
        self.x += self.vx;
        self.y += self.vy;
        if self.x < 0.0 {
            self.x += sw;
        }
        if self.y < 0.0 {
            self.y += sh;
        }
        if self.x >= sw {
            self.x -= sw;
        }
        if self.y >= sh {
            self.y -= sh;
        }
    }
}
