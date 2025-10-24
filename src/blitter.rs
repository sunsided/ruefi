use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

extern crate alloc;
use alloc::vec::Vec;
use core::arch::x86_64::_mm256_mask_sll_epi32;

/// Software back buffer in system memory with the same drawing API
pub struct BackBuffer {
    pub width: usize,
    pub height: usize,
    fmt: PixelFormat,
    buf: Vec<u8>, // width * height * 4 in GOP-native pixel order
    // Cached GOP framebuffer information to avoid per-frame queries
    dst_ptr: *mut u8,
    dst_pitch: usize, // bytes per scanline in GOP framebuffer
}

impl BackBuffer {
    pub fn from_gop(gop: &mut GraphicsOutput) -> Self {
        let info = gop.current_mode_info();
        let (width, height) = info.resolution();
        let fmt = info.pixel_format();
        let bpp = 4usize;
        let dst_pitch = info.stride() * bpp;
        // Get and cache the raw framebuffer pointer
        let mut fb = gop.frame_buffer();
        let dst_ptr = fb.as_mut_ptr();

        let len = width * height * bpp;
        let mut buf = Vec::with_capacity(len);
        unsafe {
            buf.set_len(len);
        }
        BackBuffer {
            width,
            height,
            fmt,
            buf,
            dst_ptr,
            dst_pitch,
        }
    }

    #[inline]
    pub fn clear_bgr(&mut self, r: u8, g: u8, b: u8) {
        self.clear_rgb(b, g, r)
    }

    pub fn clear_rgb(&mut self, r: u8, g: u8, b: u8) {
        // Fill line by line in correct pixel order
        for y in 0..self.height {
            let row_off = y * self.width * 4;
            for x in 0..self.width {
                let p = row_off + x * 4;
                self.buf[p + 0] = r;
                self.buf[p + 1] = g;
                self.buf[p + 2] = b;
                self.buf[p + 3] = 0;
            }
        }
    }

    #[inline]
    pub fn put_pixel_bgr(&mut self, x: isize, y: isize, r: u8, g: u8, b: u8) {
        self.put_pixel_rgb(x, y, b, g, r);
    }

    #[inline]
    pub fn put_pixel_rgb(&mut self, x: isize, y: isize, r: u8, g: u8, b: u8) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return;
        }
        let p = y * self.width * 4 + x * 4;
        self.buf[p + 0] = r;
        self.buf[p + 1] = g;
        self.buf[p + 2] = b;
        self.buf[p + 3] = 0;
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
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        match self.fmt {
            PixelFormat::Rgb => loop {
                self.put_pixel_rgb(x0, y0, r, g, b);
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
            },
            PixelFormat::Bgr => loop {
                self.put_pixel_bgr(x0, y0, r, g, b);
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
            },
            PixelFormat::Bitmask => return,
            PixelFormat::BltOnly => return,
        };
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

    /// Blit an RGBA image into the backbuffer (alpha==0 treated as transparent)
    pub fn blit_rgba(&mut self, rgba: &[u8], w: usize, h: usize, dst_x: usize, dst_y: usize) {
        let w = core::cmp::min(w, self.width.saturating_sub(dst_x));
        let h = core::cmp::min(h, self.height.saturating_sub(dst_y));
        for row in 0..h {
            let src_row = &rgba[row * (w * 4)..][..(w * 4)];
            let dst_off = (dst_y + row) * self.width * 4 + dst_x * 4;
            for x in 0..w {
                let r = src_row[x * 4 + 0];
                let g = src_row[x * 4 + 1];
                let b = src_row[x * 4 + 2];
                let a = src_row[x * 4 + 3];
                if a == 0 {
                    continue;
                }
                let p = dst_off + x * 4;
                match self.fmt {
                    PixelFormat::Bgr => {
                        self.buf[p + 0] = b;
                        self.buf[p + 1] = g;
                        self.buf[p + 2] = r;
                        self.buf[p + 3] = 0;
                    }
                    PixelFormat::Rgb => {
                        self.buf[p + 0] = r;
                        self.buf[p + 1] = g;
                        self.buf[p + 2] = b;
                        self.buf[p + 3] = 0;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Copy the backbuffer to the GOP framebuffer
    #[inline]
    pub fn flush_to_gop(&self, _gop: &mut GraphicsOutput) {
        let bpp = 4usize;
        let src_pitch = self.width * bpp;
        let dst_ptr = self.dst_ptr;
        let dst_pitch = self.dst_pitch;

        unsafe {
            if src_pitch == dst_pitch {
                // Fast path: contiguous copy of the whole buffer
                core::ptr::copy_nonoverlapping(self.buf.as_ptr(), dst_ptr, self.height * src_pitch);
            } else {
                // Fallback: copy row by row when GOP stride differs from width
                for y in 0..self.height {
                    let src_row = self.buf.as_ptr().add(y * src_pitch);
                    let dst_row = dst_ptr.add(y * dst_pitch);
                    core::ptr::copy_nonoverlapping(src_row, dst_row, src_pitch);
                }
            }
        }
    }
}
