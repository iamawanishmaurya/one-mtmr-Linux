use anyhow::{anyhow, bail, Context, Result};
use drm::control::connector::{Interface, State};
use drm::buffer::DrmFourcc;
use drm::control::{Device as ControlDevice};
use drm::{buffer::Buffer, ClientCapability, Device as DrmDevice};
use std::os::fd::AsRawFd;
use std::fs::File;
use std::os::fd::{AsFd, BorrowedFd};
use std::path::Path;

struct Card(File);

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl DrmDevice for Card {}
impl ControlDevice for Card {}

/// Render the test pattern to the Touch Bar on the given DRM card.
/// Targets the appletbdrm connector (Interface::USB, connector_id 39), mode 60x2008.
/// With hold=true, re-renders every few seconds and keeps DRM master until SIGINT/SIGTERM.
pub fn render_to_card(card: &Path, hold: bool) -> Result<()> {
    // Dumb-buffer mmap requires the DRM fd to be opened read/write.
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(card)
        .with_context(|| format!("open {} (rw)", card.display()))?;
    let dev = Card(file);

    dev.set_client_capability(ClientCapability::UniversalPlanes, true)
        .context("set UniversalPlanes capability")?;

    // Stage: connector
    let res = dev
        .resource_handles()
        .map_err(|e| anyhow!("stage=connector: resource_handles failed: {e}"))?;
    let mut found = None;
    for handle in res.connectors() {
        let info = dev
            .get_connector(*handle, false)
            .map_err(|e| anyhow!("stage=connector: get_connector: {e}"))?;
        if info.interface() == Interface::USB {
            found = Some(info);
            break;
        }
    }
    let info = found.ok_or_else(|| anyhow!("stage=connector: no USB (Touch Bar) connector found"))?;
    let conn_handle = info.handle();
    let conn_id: u32 = conn_handle.into();
    if !matches!(info.state(), State::Connected) {
        bail!("stage=connector: USB connector {conn_id} not connected");
    }

    // Stage: mode
    let mode: drm::control::Mode = *info
        .modes()
        .first()
        .ok_or_else(|| anyhow!("stage=mode: connector has no modes"))?;
    let (w, h) = mode.size();
    let (w, h) = (w as usize, h as usize);

    // Stage: crtc via current encoder
    let enc_handle = info
        .current_encoder()
        .ok_or_else(|| anyhow!("stage=crtc: connector has no current encoder"))?;
    let enc_info = dev
        .get_encoder(enc_handle)
        .map_err(|e| anyhow!("stage=crtc: get_encoder: {e}"))?;
    let crtc = enc_info
        .crtc()
        .ok_or_else(|| anyhow!("stage=crtc: encoder has no crtc"))?;

    // Stage: dumb buffer + framebuffer
    let mut db = dev
        .create_dumb_buffer((w as u32, h as u32), DrmFourcc::Xrgb8888, 32)
        .map_err(|e| anyhow!("stage=buffer: create_dumb_buffer: {e}"))?;
    let fb = dev
        .add_framebuffer(&db, 24, 32)
        .map_err(|e| anyhow!("stage=buffer: add_framebuffer: {e}"))?;

    // Stage: fill (stride-aware: rows may be padded, keep pattern on first w columns)
    let stride_px = Buffer::pitch(&db) as usize / 4;
    {
        let mut map = dev
            .map_dumb_buffer(&mut db)
            .map_err(|e| anyhow!("stage=buffer: map: {e}"))?;
        crate::pattern::fill_test_pattern_stride(map.as_mut(), w, h, stride_px);
    } // map dropped here; mutable borrow of db ends

    // Stage: master (legacy SetCrtc requires DRM master)
    drm_ffi::auth::acquire_master(dev.as_fd())
        .map_err(|e| anyhow!("stage=master: acquire_master: {e}"))?;

    // Stage: modeset (legacy SetCrtc)
    dev.set_crtc(crtc, Some(fb), (0, 0), &[conn_handle], Some(mode))
        .map_err(|e| anyhow!("stage=modeset: set_crtc: {e}"))?;

    println!("OK connector={conn_id} mode={} fb={fb:?}", mode.name().to_string_lossy());
    if !hold {
        let _ = drm_ffi::auth::release_master(dev.as_fd());
        return Ok(());
    }
    // Live mode: keep master + re-render periodically so the bar survives
    // driver-level resets. Exits on SIGINT/SIGTERM (handler installed by caller).
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let mut map = dev
            .map_dumb_buffer(&mut db)
            .map_err(|e| anyhow!("stage=live: map: {e}"))?;
        crate::pattern::fill_test_pattern_stride(map.as_mut(), w, h, stride_px);
    }
}

/// Fill a test pattern into an XRGB8888 buffer with a row stride wider than the
/// visible width; stride padding columns are left untouched.
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
