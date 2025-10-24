use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

extern crate alloc;
use alloc::vec::Vec;

/// Software back buffer in system memory with the same drawing API
pub struct BackBuffer {
    pub width: usize,
    pub height: usize,
    buf: Vec<u8>, // width * height * 4 in GOP-native pixel order
    // Cached GOP framebuffer information to avoid per-frame queries
    dst_ptr: *mut u8,
    dst_pitch: usize, // bytes per scanline in GOP framebuffer
    // Shuffle function: takes RGB input and returns packed u32 in target pixel order (little-endian)
    shuffle: fn(r: u8, g: u8, b: u8) -> u32,
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

        // Choose shuffle function based on GOP pixel format. Input is RGB.
        let shuffle: fn(u8, u8, u8) -> u32 = match fmt {
            PixelFormat::Rgb => Self::pack_rgb as fn(u8, u8, u8) -> u32,
            PixelFormat::Bgr => Self::pack_bgr as fn(u8, u8, u8) -> u32,
            _ => Self::pack_rgb as fn(u8, u8, u8) -> u32,
        };

        let len = width * height * bpp;
        let mut buf = Vec::with_capacity(len);
        unsafe {
            buf.set_len(len);
        }
        BackBuffer {
            width,
            height,
            buf,
            dst_ptr,
            dst_pitch,
            shuffle,
        }
    }

    #[inline(always)]
    const fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
        // little-endian bytes: [r, g, b, 0]
        (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
    }

    #[inline(always)]
    const fn pack_bgr(r: u8, g: u8, b: u8) -> u32 {
        // little-endian bytes: [b, g, r, 0]
        (b as u32) | ((g as u32) << 8) | ((r as u32) << 16)
    }

    #[inline]
    pub fn clear_bgr(&mut self, r: u8, g: u8, b: u8) {
        self.clear_rgb(b, g, r)
    }

    pub fn clear_rgb(&mut self, r: u8, g: u8, b: u8) {
        // Fill line by line using shuffle to pack RGB into target order
        let packed = (self.shuffle)(r, g, b).to_le_bytes();
        for y in 0..self.height {
            let mut p = y * self.width * 4;
            for _x in 0..self.width {
                self.buf[p..p + 4].copy_from_slice(&packed);
                p += 4;
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
        let p = y * self.width * 4 + x * 4;
        let packed = (self.shuffle)(r, g, b).to_le_bytes();
        self.buf[p..p + 4].copy_from_slice(&packed);
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
        // Single path: put_pixel_rgb already adapts using the shuffle
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
                let packed = (self.shuffle)(r, g, b).to_le_bytes();
                self.buf[p..p + 4].copy_from_slice(&packed);
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
