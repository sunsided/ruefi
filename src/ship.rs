use libm::{cosf, sinf};

/// Player ship representation and behavior
pub struct Ship {
    pub x: f32,
    pub y: f32,
    pub angle: f32,     // radians; 0 faces +Y (downwards)
    pub speed: f32,     // scalar speed along forward
    pub tri_h: f32,     // distance from center to nose along forward
    pub tri_w: f32,     // base width
    pub thrust: f32,    // acceleration magnitude when thrusting
    pub rot_speed: f32, // radians per input step
}

impl Ship {
    /// Construct a ship centered on screen with default tuning.
    pub fn new(sw: usize, sh: usize) -> Self {
        Ship {
            x: (sw / 2) as f32,
            y: (sh / 2) as f32,
            angle: 0.0,
            speed: 0.0,
            tri_h: 24.0,
            tri_w: 18.0,
            thrust: 1.5,
            rot_speed: 0.08,
        }
    }

    /// Forward unit vector (X right, Y down)
    #[inline]
    pub fn forward_vec(&self) -> (f32, f32) {
        (-sinf(self.angle), cosf(self.angle))
    }

    /// Nose position in world space
    #[inline]
    pub fn nose(&self) -> (f32, f32) {
        let (fx, fy) = self.forward_vec();
        (self.x + fx * self.tri_h, self.y + fy * self.tri_h)
    }

    /// Update rotation, thrust, integrate motion, apply friction, and wrap.
    /// rot: -1, 0, 1; thr: -1, 0, 1
    pub fn update(&mut self, rot: i8, thr: i8, sw: f32, sh: f32) {
        // Apply rotation and thrust intent
        self.angle += (rot as f32) * self.rot_speed;
        if thr != 0 {
            self.speed = (thr as f32) * self.thrust;
        }
        // Integrate along forward
        let (fx, fy) = self.forward_vec();
        self.x += fx * self.speed;
        self.y += fy * self.speed;
        // Friction
        self.speed *= 0.85;
        // Wrap
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
