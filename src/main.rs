#![no_std]
#![no_main]
#![allow(unsafe_code)]

mod asteroid;
mod blitter;
mod projectile;
mod rand;
mod ship;
mod uefi_alloc;

use crate::blitter::BackBuffer;
extern crate alloc;
use crate::asteroid::Asteroid;
use crate::projectile::Projectile;
use crate::rand::XorShift64;
use crate::ship::Ship;
use alloc::vec::Vec;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::{Key, ScanCode};

mod logo {
    include!(concat!(env!("OUT_DIR"), "/assets_gen.rs"));
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
        for _ in 0..5 {
            asteroids.push(Asteroid::random_spawn(&mut rng, sw, sh));
        }

        // Player ship
        let mut ship = Ship::new(sw, sh);

        // Projectiles
        const MAX_PROJECTILES: usize = 100;
        let mut projectiles: Vec<Projectile> = Vec::with_capacity(MAX_PROJECTILES);
        let mut projectile_speed: f32 = 12.0; // configurable speed (pixels/frame)
        let projectile_len: f32 = 5.0; // visible length in pixels

        // Cached screen size as f32
        let sw_f: f32 = sw as f32;
        let sh_f: f32 = sh as f32;

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

            // Update ship physics and wrapping
            ship.update(rot, thr, sw_f, sh_f);

            // Fire projectile if requested and under cap
            if fire && projectiles.len() < MAX_PROJECTILES {
                projectiles.push(Projectile::spawn_from_ship(&ship, projectile_speed));
            }

            // Apply projectile speed adjustments
            if speed_adj != 0 {
                projectile_speed = (projectile_speed + (speed_adj as f32)).clamp(2.0, 50.0);
            }

            // Update projectiles
            for p in &mut projectiles {
                p.update();
            }

            // Cull projectiles that left the screen
            projectiles.retain(|p| p.on_screen(sw_f, sh_f));

            // Update asteroids
            for a in &mut asteroids {
                a.update(sw_f, sh_f);
            }

            // Double-buffered rendering: clear backbuffer, compose scene, then flush
            back.clear_bgr(0, 0, 0);
            back.blit_rgba(logo::LOGO_RGBA, logo::LOGO_WIDTH, logo::LOGO_HEIGHT, 10, 10);

            for a in &asteroids {
                back.draw_asteroid_wrapped(a, sw, sh, 200, 200, 200);
            }

            for p in &projectiles {
                back.draw_projectile(p.x, p.y, p.vx, p.vy, projectile_len, 255, 255, 0);
            }

            back.draw_ship(&ship, 92, 127, 255);

            back.flush_to_gop(&mut gop);

            // Simple frame pacing (~60 FPS): stall for ~16 ms to reduce CPU usage and tearing
            boot::stall(16_000);
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
