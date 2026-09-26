use anyhow::{bail, Context, Result};

/// Fill a DRM dumb buffer with the test pattern. Kept here so the DRM path
/// and any debug paths share one pixel format (XRGB8888, little-endian BGRA bytes).
pub fn xrgb8888_bytes(w: usize, h: usize) -> Vec<u8> {
    let mut buf = vec![0u8; w * h * 4];
    crate::pattern::fill_test_pattern(&mut buf, w, h);
    buf
}

/// Write the RGB contents (3 bytes/px) as a PNG file.
pub fn dump_png(path: &str, w: usize, h: usize, rgb: &[u8]) -> Result<()> {
    let file = std::fs::File::create(path).with_context(|| format!("create {path}"))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header()?;
    writer.write_image_data(rgb)?;
    Ok(())
}

/// Convert XRGB8888 byte buffer to packed RGB for PNG export.
pub fn xrgb8888_to_rgb(xrgb: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(w * h * 3);
    for px in xrgb.chunks_exact(4).take(w * h) {
        rgb.push(px[2]);
        rgb.push(px[1]);
        rgb.push(px[0]);
    }
    rgb
}

/// Fallback stub so `--drm` compiles before Task 2 lands. Replaced by real modeset.
pub fn render_to_card(_card: &std::path::Path) -> Result<()> {
    bail!("DRM output not implemented yet (Task 2)")
}
