#![no_std]
#![no_main]
#![allow(unsafe_code)]

mod blitter;

use crate::blitter::{Surface, blit_rgba_to_gop};
use libm::{cosf, sinf};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::{Color, Key, ScanCode};

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
        let mut surf = Surface::from_gop(&mut gop);

        // Clear background once to black and draw the logo once
        surf.clear(0, 0, 0);
        blit_rgba_to_gop(
            &mut gop,
            logo::LOGO_RGBA,
            logo::LOGO_WIDTH,
            logo::LOGO_HEIGHT,
            10,
            10,
        )
        .map_err(|_| uefi::Error::new(Status::ABORTED, ()))?;

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

        // Keep the previous triangle's vertices for dirty redraw
        let mut prev_pts: Option<[(isize, isize); 3]> = None;
        // Frame counter to optionally skip input polling every other frame
        let mut frame: u64 = 0;

        loop {
            // Poll keyboard only on even frames to reduce overhead
            let poll_input = (frame & 1) == 0;
            // Derive intent for this frame
            let mut rot: i8 = 0;
            let mut thr: i8 = 0;
            let mut exit = false;
            if poll_input {
                while let Ok(Some(k)) = stdin.read_key() {
                    match k {
                        Key::Special(ScanCode::LEFT) => rot = -1,
                        Key::Special(ScanCode::RIGHT) => rot = 1,
                        Key::Special(ScanCode::UP) => thr = 1,
                        Key::Special(ScanCode::DOWN) => thr = -1,
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

            // Dirty redraw: erase previous triangle by drawing it in black
            if let Some(p) = prev_pts {
                surf.draw_triangle_wire(p[0].0, p[0].1, p[1].0, p[1].1, p[2].0, p[2].1, 0, 0, 0);
            }

            // Draw the new triangle in white
            surf.draw_triangle_wire(
                pts[0].0, pts[0].1, pts[1].0, pts[1].1, pts[2].0, pts[2].1, 255, 255, 255,
            );

            // Re-draw the logo so it stays visible if the triangle ran over it
            let _ = blit_rgba_to_gop(
                &mut gop,
                logo::LOGO_RGBA,
                logo::LOGO_WIDTH,
                logo::LOGO_HEIGHT,
                10,
                10,
            );

            // Store current triangle for next frame's erase pass
            prev_pts = Some(pts);

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
