use crate::rand::XorShift64;
use libm::{cosf, sinf, sqrtf};

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
    /// Average collision radius computed as mean of inscribed and circumscribed radii
    /// using the per-vertex jittered radii. This approximates the polygon with a circle.
    #[inline]
    pub fn collision_radius(&self) -> f32 {
        let mut min_r = self.radius * self.jitter[0];
        let mut max_r = min_r;
        for i in 1..6 {
            let r = self.radius * self.jitter[i];
            if r < min_r {
                min_r = r;
            }
            if r > max_r {
                max_r = r;
            }
        }
        0.5 * (min_r + max_r)
    }

    /// Mass proportional to area (pi r^2) with unit density -> k*r^2; constant factor cancels in impulse.
    #[inline]
    pub fn mass(&self) -> f32 {
        let r = self.collision_radius();
        r * r
    }

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

    /// Resolve pairwise asteroid collisions in-place using a circle approximation and impulse response.
    /// Operates in a toroidal world of size (sw, sh).
    pub fn resolve_collisions(asteroids: &mut [Asteroid], sw: f32, sh: f32) {
        let e: f32 = 0.9; // restitution
        let percent: f32 = 0.8; // positional correction percentage
        let slop: f32 = 0.01; // penetration allowance
        let half_w = 0.5 * sw;
        let half_h = 0.5 * sh;
        let n = asteroids.len();
        for i in 0..n {
            for j in (i + 1)..n {
                // Shortest delta on torus
                let mut dx = asteroids[j].x - asteroids[i].x;
                if dx > half_w {
                    dx -= sw;
                } else if dx < -half_w {
                    dx += sw;
                }
                let mut dy = asteroids[j].y - asteroids[i].y;
                if dy > half_h {
                    dy -= sh;
                } else if dy < -half_h {
                    dy += sh;
                }

                let r1 = asteroids[i].collision_radius();
                let r2 = asteroids[j].collision_radius();
                let sum_r = r1 + r2;
                let dist2 = dx * dx + dy * dy;
                if dist2 >= sum_r * sum_r {
                    continue;
                }
                let mut dist = sqrtf(dist2);
                // Avoid divide by zero for identical positions
                let (nx, ny) = if dist > 1e-5 {
                    (dx / dist, dy / dist)
                } else {
                    dist = sum_r;
                    (1.0f32, 0.0f32)
                };

                // Relative velocity along normal
                let rvx = asteroids[j].vx - asteroids[i].vx;
                let rvy = asteroids[j].vy - asteroids[i].vy;
                let rel_vel = rvx * nx + rvy * ny;

                // Mass/inverse mass (proportional to r^2)
                let m1 = asteroids[i].mass();
                let m2 = asteroids[j].mass();
                let inv_m1 = 1.0f32 / m1;
                let inv_m2 = 1.0f32 / m2;
                let inv_mass_sum = inv_m1 + inv_m2;

                // Positional correction to resolve overlap
                let penetration = sum_r - dist;
                if penetration > 0.0 {
                    let corr_mag = ((penetration - slop).max(0.0)) * percent / inv_mass_sum;
                    let corr_x = corr_mag * nx;
                    let corr_y = corr_mag * ny;
                    asteroids[i].x -= corr_x * inv_m1;
                    asteroids[i].y -= corr_y * inv_m1;
                    asteroids[j].x += corr_x * inv_m2;
                    asteroids[j].y += corr_y * inv_m2;

                    // Wrap corrected positions
                    if asteroids[i].x < 0.0 {
                        asteroids[i].x += sw;
                    }
                    if asteroids[i].y < 0.0 {
                        asteroids[i].y += sh;
                    }
                    if asteroids[i].x >= sw {
                        asteroids[i].x -= sw;
                    }
                    if asteroids[i].y >= sh {
                        asteroids[i].y -= sh;
                    }
                    if asteroids[j].x < 0.0 {
                        asteroids[j].x += sw;
                    }
                    if asteroids[j].y < 0.0 {
                        asteroids[j].y += sh;
                    }
                    if asteroids[j].x >= sw {
                        asteroids[j].x -= sw;
                    }
                    if asteroids[j].y >= sh {
                        asteroids[j].y -= sh;
                    }
                }

                // Apply impulse only if approaching
                if rel_vel < 0.0 {
                    let j_imp = -(1.0 + e) * rel_vel / inv_mass_sum;
                    let imp_x = j_imp * nx;
                    let imp_y = j_imp * ny;
                    asteroids[i].vx -= imp_x * inv_m1;
                    asteroids[i].vy -= imp_y * inv_m1;
                    asteroids[j].vx += imp_x * inv_m2;
                    asteroids[j].vy += imp_y * inv_m2;
                }
            }
        }
    }
}
