pub fn fill_test_pattern(buf: &mut [u8], w: usize, h: usize) {
    assert_eq!(buf.len(), w * h * 4, "buffer must be XRGB8888 {w}x{h}");
    let bands: [[u8; 3]; 8] = [
        [255, 0, 0],   // red
        [0, 255, 0],   // green
        [0, 0, 255],   // blue
        [255, 255, 0], // yellow
        [255, 0, 255], // magenta
        [0, 255, 255], // cyan
        [255, 255, 255], // white
        [255, 128, 0], // orange
    ];
    for y in 0..h {
        for x in 0..w {
            let (r, g, b) = if x == 0 || x == w - 1 {
                (255u8, 255, 255)
            } else {
                let idx = (x * bands.len()) / w;
                let c = bands[idx];
                (c[0], c[1], c[2])
            };
            let off = (y * w + x) * 4;
            // XRGB8888 little-endian: B, G, R, X
            buf[off] = b;
            buf[off + 1] = g;
            buf[off + 2] = r;
            buf[off + 3] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_dimensions_and_markers() {
        let (w, h) = (60, 2008);
        let mut buf = vec![0u8; w * h * 4];
        fill_test_pattern(&mut buf, w, h);
        // left marker column white
        assert_eq!(buf[0], 255);
        assert_eq!(buf[1], 255);
        assert_eq!(buf[2], 255);
        // right marker column white
        let off = ((h - 1) * w + (w - 1)) * 4;
        assert_eq!(buf[off], 255);
        assert_eq!(buf[off + 2], 255);
        // first band is red: R=255, G=0 at x=1
        let off = 4;
        assert_eq!(buf[off + 2], 255); // R
        assert_eq!(buf[off], 0); // B
    }
}
