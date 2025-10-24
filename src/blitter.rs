use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

/// Minimal blitter for sending RGBA8 to GOP framebuffer
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

/// Simple immediate-mode drawing helpers for GOP
pub struct Surface {
    fb_ptr: *mut u8,
    stride: usize, // pixels per scanline
    fmt: PixelFormat,
    pub width: usize,
    pub height: usize,
}

impl Surface {
    pub fn from_gop(gop: &mut GraphicsOutput) -> Self {
        let info = gop.current_mode_info();
        let (width, height) = info.resolution();
        let stride = info.stride();
        let fmt = info.pixel_format();
        let mut fb = gop.frame_buffer();
        let fb_ptr = fb.as_mut_ptr();
        Surface {
            fb_ptr,
            stride,
            fmt,
            width,
            height,
        }
    }

    #[inline]
    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        let pitch_bytes = self.stride * 4;
        for y in 0..self.height {
            let row = unsafe { self.fb_ptr.add(y * pitch_bytes) };
            for x in 0..self.width {
                unsafe {
                    let p = row.add(x * 4);
                    match self.fmt {
                        PixelFormat::Bgr => {
                            *p.add(0) = b;
                            *p.add(1) = g;
                            *p.add(2) = r;
                            *p.add(3) = 0;
                        }
                        PixelFormat::Rgb => {
                            *p.add(0) = r;
                            *p.add(1) = g;
                            *p.add(2) = b;
                            *p.add(3) = 0;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    #[inline]
    pub fn put_pixel(&mut self, x: isize, y: isize, r: u8, g: u8, b: u8) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return;
        }
        let pitch_bytes = self.stride * 4;
        unsafe {
            let p = self.fb_ptr.add(y * pitch_bytes + x * 4);
            match self.fmt {
                PixelFormat::Bgr => {
                    *p.add(0) = b;
                    *p.add(1) = g;
                    *p.add(2) = r;
                    *p.add(3) = 0;
                }
                PixelFormat::Rgb => {
                    *p.add(0) = r;
                    *p.add(1) = g;
                    *p.add(2) = b;
                    *p.add(3) = 0;
                }
                _ => {}
            }
        }
    }

    pub fn draw_line(
        &mut self,
        mut x0: isize,
        mut y0: isize,
        x1: isize,
        y1: isize,
        r: u8,
        g: u8,
        b: u8,
    ) {
        // Bresenham's line algorithm
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.put_pixel(x0, y0, r, g, b);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn draw_triangle_wire(
        &mut self,
        x0: isize,
        y0: isize,
        x1: isize,
        y1: isize,
        x2: isize,
        y2: isize,
        r: u8,
        g: u8,
        b: u8,
    ) {
        self.draw_line(x0, y0, x1, y1, r, g, b);
        self.draw_line(x1, y1, x2, y2, r, g, b);
        self.draw_line(x2, y2, x0, y0, r, g, b);
    }
}
