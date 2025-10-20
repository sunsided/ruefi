use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

///  Minimal blitter for sending RGBA8 to GOP framebuffer
pub fn blit_rgba_to_gop(
    gop: &mut GraphicsOutput,
    rgba: &[u8],
    w: usize,
    h: usize,
    dst_x: usize,
    dst_y: usize,
) -> Result<(), ()> {
    let info = gop.current_mode_info();
    let stride = info.stride(); // pixels per scanline
    let fmt = info.pixel_format(); // Rgb | Bgr | Bitmask | BltOnly

    let mut fb = gop.frame_buffer();
    let fb_ptr = fb.as_mut_ptr();
    let bpp = 4usize;
    let pitch_bytes = stride * bpp;

    let (sw, sh) = gop.current_mode_info().resolution();
    let w = core::cmp::min(w, sw.saturating_sub(dst_x));
    let h = core::cmp::min(h, sh.saturating_sub(dst_y));

    for row in 0..h {
        let dst_off = (dst_y + row) * pitch_bytes + dst_x * bpp;
        let dst_row = unsafe { fb_ptr.add(dst_off) };
        let src_row = &rgba[row * (w * 4)..][..(w * 4)];

        match fmt {
            PixelFormat::Bgr => {
                for x in 0..w {
                    let r = src_row[x * 4 + 0];
                    let g = src_row[x * 4 + 1];
                    let b = src_row[x * 4 + 2];
                    let a = src_row[x * 4 + 3];
                    if a == 0 {
                        continue;
                    }

                    unsafe {
                        let p = dst_row.add(x * 4);
                        *p.add(0) = b;
                        *p.add(1) = g;
                        *p.add(2) = r;
                        *p.add(3) = 0;
                    }
                }
            }
            PixelFormat::Rgb => {
                for x in 0..w {
                    let r = src_row[x * 4 + 0];
                    let g = src_row[x * 4 + 1];
                    let b = src_row[x * 4 + 2];
                    let a = src_row[x * 4 + 3];
                    if a == 0 {
                        continue;
                    }

                    unsafe {
                        let p = dst_row.add(x * 4);
                        *p.add(0) = r;
                        *p.add(1) = g;
                        *p.add(2) = b;
                        *p.add(3) = 0;
                    }
                }
            }
            _ => return Err(()),
        }
    }
    Ok(())
}
