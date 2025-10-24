use crate::ship::Ship;

/// Simple projectile moving in a straight line
pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

impl Projectile {
    /// Spawn a projectile from the ship's nose along its current forward vector, scaled by `speed`.
    #[inline]
    pub fn spawn_from_ship(ship: &Ship, speed: f32) -> Self {
        let (nx, ny) = ship.nose();
        let (fx, fy) = ship.forward_vec();
        Projectile {
            x: nx,
            y: ny,
            vx: fx * speed,
            vy: fy * speed,
        }
    }

    /// Advance the projectile by its velocity.
    #[inline]
    pub fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
    }

    /// Check if the projectile is still within the screen bounds.
    #[inline]
    pub const fn on_screen(&self, sw: f32, sh: f32) -> bool {
        self.x >= 0.0 && self.x < sw && self.y >= 0.0 && self.y < sh
    }
}
