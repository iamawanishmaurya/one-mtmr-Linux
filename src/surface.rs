/// RGB drawing surface backed by an XRGB8888 byte buffer (little-endian BGRA bytes).
#[derive(Debug, Clone)]
pub struct Surface {
    pub w: usize,
    pub h: usize,
    pub stride: usize, // row pitch in pixels (>= w)
    pub buf: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

pub const WHITE: Color = Color(255, 255, 255);
pub const BG: Color = Color(10, 10, 10);

impl Surface {
    pub fn new(w: usize, h: usize, stride: usize) -> Surface {
        assert!(stride >= w);
        Surface {
            w,
            h,
            stride,
            buf: vec![0; stride * h * 4],
        }
    }

    pub fn clear(&mut self, c: Color) {
        for y in 0..self.h {
            for x in 0..self.w {
                self.set_px(x, y, c);
            }
        }
    }

    pub fn fill_rect(&mut self, r: Rect, c: Color) {
        for y in r.y..(r.y + r.h).min(self.h) {
            for x in r.x..(r.x + r.w).min(self.w) {
                self.set_px(x, y, c);
            }
        }
    }

    /// Invert pixels in a rect (for press highlight).
    pub fn invert_rect(&mut self, r: Rect) {
        for y in r.y..(r.y + r.h).min(self.h) {
            for x in r.x..(r.x + r.w).min(self.w) {
                let off = (y * self.stride + x) * 4;
                self.buf[off] = 255 - self.buf[off];
                self.buf[off + 1] = 255 - self.buf[off + 1];
                self.buf[off + 2] = 255 - self.buf[off + 2];
            }
        }
    }

    pub fn set_px(&mut self, x: usize, y: usize, c: Color) {
        if x >= self.w || y >= self.h {
            return;
        }
        let off = (y * self.stride + x) * 4;
        self.buf[off] = c.2; // B
        self.buf[off + 1] = c.1; // G
        self.buf[off + 2] = c.0; // R
    }

    /// Get packed RGB bytes (3/px) for PNG export.
    pub fn to_rgb(&self) -> Vec<u8> {
        let mut rgb = Vec::with_capacity(self.w * self.h * 3);
        for y in 0..self.h {
            for x in 0..self.w {
                let off = (y * self.stride + x) * 4;
                rgb.push(self.buf[off + 2]);
                rgb.push(self.buf[off + 1]);
                rgb.push(self.buf[off]);
            }
        }
        rgb
    }
}
