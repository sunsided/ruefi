#![no_std]
#![no_main]
#![allow(unsafe_code)]

mod blitter;
mod rand;
mod uefi_alloc;

use crate::blitter::BackBuffer;
extern crate alloc;
use crate::rand::XorShift64;
use alloc::vec::Vec;
use libm::{cosf, sinf, sqrtf};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::{Key, ScanCode};

mod logo {
    include!(concat!(env!("OUT_DIR"), "/assets_gen.rs"));
}

// Asteroid representation: jagged hexagon with per-vertex radial jitter
struct Asteroid {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
    base_angle: f32,  // orientation for the hexagon
    jitter: [f32; 6], // multiplicative per-vertex radius factors
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().expect("failed to initialize UEFI");
    if run_game().is_err() {
        return Status::ABORTED;
    }
    Status::SUCCESS
}

fn run_game() -> uefi::Result<()> {
    system::with_stdin(|stdin| {
        // Open GOP (scoped, exclusive) inside stdin closure
        let handle = boot::get_handle_for_protocol::<GraphicsOutput>()
            .map_err(|_| uefi::Error::new(Status::ABORTED, ()))?;
        let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(handle)
            .map_err(|_| uefi::Error::new(Status::ABORTED, ()))?;

        let (sw, sh) = gop.current_mode_info().resolution();
        let mut back = BackBuffer::from_gop(&mut gop);

        // RNG seeded from a timing source
        let mut rng = XorShift64::default();

        // Asteroids: prepare a few, away from the center
        let mut asteroids: Vec<Asteroid> = Vec::with_capacity(32);
        {
            let cx = (sw / 2) as f32;
            let cy = (sh / 2) as f32;
            let min_dist = 120.0f32; // keep spawn away from player start
            let max_radius = ((sw.min(sh)) as f32) * 0.08 + 12.0;
            for _ in 0..5 {
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
                // small random velocity, not yet used for movement but stored
                let speed = rng.range_f32(0.5, 2.0);
                let dir = rng.range_f32(0.0, 2.0 * core::f32::consts::PI);
                let vx = -sinf(dir) * speed;
                let vy = cosf(dir) * speed;
                let base_angle = rng.range_f32(0.0, 2.0 * core::f32::consts::PI);
                let mut jitter = [1.0f32; 6];
                for j in 0..6 {
                    jitter[j] = rng.range_f32(0.75, 1.25);
                }
                asteroids.push(Asteroid {
                    x,
                    y,
                    vx,
                    vy,
                    radius,
                    base_angle,
                    jitter,
                });
            }
        }

        // Player state
        let mut px = (sw / 2) as f32;
        let mut py = (sh / 2) as f32;
        let mut angle = 0.0f32; // radians; 0 means facing +Y (downwards on screen)

        // Triangle size (in pixels)
        let tri_h = 24.0f32; // distance from center to nose along forward (Y)
        let tri_w = 18.0f32; // base width

        let mut speed = 0.0f32;
        let thrust = 1.5f32; // pixels per frame when holding Up
        let rot_speed = 0.08f32; // radians per keypress frame

        // Projectiles
        struct Projectile {
            x: f32,
            y: f32,
            vx: f32,
            vy: f32,
        }
        const MAX_PROJECTILES: usize = 100;
        let mut projectiles: Vec<Projectile> = Vec::with_capacity(MAX_PROJECTILES);
        let mut projectile_speed: f32 = 12.0; // configurable speed (pixels/frame)
        let projectile_len: f32 = 5.0; // visible length in pixels

        // Frame counter to optionally skip input polling every other frame
        let mut frame: u64 = 0;

        loop {
            // Poll keyboard only on even frames to reduce overhead
            let poll_input = (frame & 1) == 0;
            // Derive intent for this frame
            let mut rot: i8 = 0;
            let mut thr: i8 = 0;
            let mut fire: bool = false;
            let mut speed_adj: i8 = 0;
            let mut exit = false;
            if poll_input {
                while let Ok(Some(k)) = stdin.read_key() {
                    match k {
                        Key::Special(ScanCode::LEFT) => rot = -1,
                        Key::Special(ScanCode::RIGHT) => rot = 1,
                        Key::Special(ScanCode::UP) => thr = 1,
                        Key::Special(ScanCode::DOWN) => thr = -1,
                        // Space key as printable space
                        Key::Printable(c) if c == ' ' => fire = true,
                        // Adjust projectile speed with [ and ]
                        Key::Printable(c) if c == '[' => {
                            speed_adj = -1;
                        }
                        Key::Printable(c) if c == ']' => {
                            speed_adj = 1;
                        }
                        Key::Special(ScanCode::ESCAPE) => {
                            exit = true;
                        }
                        _ => {}
                    }
                }
            }
            if exit {
                break;
            }

            // Apply rotation and thrust
            angle += (rot as f32) * rot_speed;
            if thr != 0 {
                speed = (thr as f32) * thrust;
            }

            // Integrate movement along forward vector. Coordinate system: X right, Y down (+forward)
            let fx = -sinf(angle);
            let fy = cosf(angle);
            px += fx * speed;
            py += fy * speed;
            // Apply a little friction so motion stops if no thrust
            speed *= 0.85;

            // Keep player on screen (wrap around)
            if px < 0.0 {
                px += sw as f32;
            }
            if py < 0.0 {
                py += sh as f32;
            }
            if px >= sw as f32 {
                px -= sw as f32;
            }
            if py >= sh as f32 {
                py -= sh as f32;
            }

            // Fire projectile if requested and under cap
            if fire && projectiles.len() < MAX_PROJECTILES {
                // Use current forward vector (fx, fy) to compute nose and velocity
                let nose_x = px + fx * tri_h;
                let nose_y = py + fy * tri_h;
                let vx = fx * projectile_speed;
                let vy = fy * projectile_speed;
                projectiles.push(Projectile {
                    x: nose_x,
                    y: nose_y,
                    vx,
                    vy,
                });
            }

            // Apply projectile speed adjustments
            if speed_adj != 0 {
                projectile_speed = (projectile_speed + (speed_adj as f32)).clamp(2.0, 50.0);
            }

            // Update projectiles
            for p in &mut projectiles {
                p.x += p.vx;
                p.y += p.vy;
            }
            // Cull projectiles that left the screen
            let sw_f = sw as f32;
            let sh_f = sh as f32;
            projectiles.retain(|p| p.x >= 0.0 && p.x < sw_f && p.y >= 0.0 && p.y < sh_f);

            // Update asteroids: integrate velocity and wrap around screen edges
            for a in &mut asteroids {
                a.x += a.vx;
                a.y += a.vy;
                if a.x < 0.0 {
                    a.x += sw_f;
                }
                if a.y < 0.0 {
                    a.y += sh_f;
                }
                if a.x >= sw_f {
                    a.x -= sw_f;
                }
                if a.y >= sh_f {
                    a.y -= sh_f;
                }
            }

            // Compute triangle vertices in local space then rotate by angle and translate to (px, py)
            let half_w = 0.5f32 * tri_w;
            // local vertices (X right, Y forward): nose at (0, +tri_h), base at y = -tri_h/2
            let verts = [
                (0.0f32, tri_h),         // nose
                (-half_w, -tri_h * 0.5), // left base
                (half_w, -tri_h * 0.5),  // right base
            ];
            let mut pts = [(0isize, 0isize); 3];
            for (i, (lx, ly)) in verts.iter().enumerate() {
                let x = lx * cosf(angle) - ly * sinf(angle);
                let y = lx * sinf(angle) + ly * cosf(angle);
                pts[i] = ((px + x) as isize, (py + y) as isize);
            }

            // Double-buffered rendering: clear backbuffer, compose scene, then flush
            back.clear_bgr(0, 0, 0);
            back.blit_rgba(logo::LOGO_RGBA, logo::LOGO_WIDTH, logo::LOGO_HEIGHT, 10, 10);
            // Render asteroids as jagged hexagon wireframes
            for a in &asteroids {
                // Precompute cos/sin of base orientation
                let ca = cosf(a.base_angle);
                let sa = sinf(a.base_angle);
                let mut pts_hex = [(0isize, 0isize); 6];
                for i in 0..6 {
                    let t = (i as f32) * (core::f32::consts::PI / 3.0);
                    let rr = a.radius * a.jitter[i];
                    // Local coordinates in our basis (X right, Y forward)
                    let lx = -sinf(t) * rr;
                    let ly = cosf(t) * rr;
                    // Rotate by base_angle and translate to world
                    let x = lx * ca - ly * sa;
                    let y = lx * sa + ly * ca;
                    pts_hex[i] = ((a.x + x) as isize, (a.y + y) as isize);
                }
                for i in 0..6 {
                    let j = (i + 1) % 6;
                    back.draw_line(
                        pts_hex[i].0,
                        pts_hex[i].1,
                        pts_hex[j].0,
                        pts_hex[j].1,
                        200,
                        200,
                        200,
                    );
                }
            }

            // Render projectiles as short lines along their velocity direction
            for p in &projectiles {
                let vlen = sqrtf(p.vx * p.vx + p.vy * p.vy);
                let (tx, ty) = if vlen > 0.0001 {
                    (
                        p.x - (p.vx / vlen) * projectile_len,
                        p.y - (p.vy / vlen) * projectile_len,
                    )
                } else {
                    (p.x, p.y)
                };
                back.draw_line(
                    p.x as isize,
                    p.y as isize,
                    tx as isize,
                    ty as isize,
                    255,
                    255,
                    0,
                );
            }
            back.draw_triangle_wire(
                pts[0].0, pts[0].1, pts[1].0, pts[1].1, pts[2].0, pts[2].1, 255, 255, 255,
            );
            back.flush_to_gop(&mut gop);

            // Simple frame pacing (~60 FPS): stall for ~16 ms to reduce CPU usage and tearing
            boot::stall(16_000);

            // Advance frame counter (wrapping) after each iteration
            frame = frame.wrapping_add(1);
        }

        // On exit, print a message (may or may not be visible depending on GOP/console state)
        system::with_stdout(|out| {
            out.output_string(cstr16!("\r\nExiting game. Bye!\r\n"))
                .ok();
        });

        Ok(())
    })
}
