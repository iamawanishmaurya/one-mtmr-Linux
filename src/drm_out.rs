use anyhow::{anyhow, bail, Context, Result};
use drm::buffer::{Buffer, DrmFourcc};
use drm::control::atomic::AtomicModeReq;
use drm::control::connector::Interface;
use drm::control::dumbbuffer::{DumbBuffer, DumbMapping};
use drm::control::framebuffer;
use drm::control::{AtomicCommitFlags, ClipRect, Device as ControlDevice, Mode, ResourceHandle};
use drm::{ClientCapability, Device as DrmDevice};
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

/// Atomic-commit DRM backend for the Touch Bar (appletbdrm).
/// Ported from the old mtmr-linux project's display.rs (tiny-dfr lineage):
/// appletbdrm does not reliably apply legacy SetCrtc/DIRTYFB updates —
/// only an atomic commit wiring connector CRTC_ID, crtc MODE_ID/ACTIVE and
/// plane FB_ID/SRC_*/CRTC_* properties actually flushes pixels.
pub struct DrmBackend {
    card: Card,
    mode: Mode,
    dbs: [DumbBuffer; 2],
    fbs: [framebuffer::Handle; 2],
    cur: usize,
    bound_plane: Option<drm::control::plane::Handle>,
    master: bool,
}

impl Drop for DrmBackend {
    fn drop(&mut self) {
        if self.master {
            let _ = drm_ffi::auth::release_master(self.card.as_fd());
        }
        for fb in self.fbs {
            let _ = self.card.destroy_framebuffer(fb);
        }
        // dumb buffers are freed by the kernel when the card fd closes
    }
}

fn find_prop_id<T: ResourceHandle>(
    card: &Card,
    handle: T,
    name: &str,
) -> Result<drm::control::property::Handle> {
    let props = card.get_properties(handle)?;
    for id in props.as_props_and_values().0 {
        let info = card.get_property(*id)?;
        if info.name().to_str()? == name {
            return Ok(*id);
        }
    }
    bail!("property {name} not found")
}

fn try_open_card(path: &Path, width: u32) -> Result<DrmBackend> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .with_context(|| format!("open {} (rw)", path.display()))?;
    let card = Card(file);
    card.set_client_capability(ClientCapability::UniversalPlanes, true)?;
    card.set_client_capability(ClientCapability::Atomic, true)?;
    drm_ffi::auth::acquire_master(card.as_fd())
        .map_err(|e| anyhow!("acquire DRM master: {e}"))?;

    // Stage: find the Touch Bar connector (USB interface on T2) and its mode
    let res = card
        .resource_handles()
        .map_err(|e| anyhow!("stage=connector: resource_handles: {e}"))?;
    let mut con_info = None;
    for con in res.connectors() {
        let info = card.get_connector(*con, false)?;
        if info.interface() == Interface::USB
            && matches!(info.state(), drm::control::connector::State::Connected)
        {
            con_info = Some(info);
            break;
        }
    }
    let con_info =
        con_info.ok_or_else(|| anyhow!("stage=connector: no connected USB (Touch Bar) connector"))?;
    let mode: Mode = *con_info
        .modes()
        .first()
        .ok_or_else(|| anyhow!("stage=mode: no modes"))?;
    let (_, disp_height) = mode.size();
    if disp_height / 60 < 30 {
        bail!("stage=mode: {disp_height} tall mode does not look like a touchbar");
    }

    // Stage: crtc + plane selection.
    // A previous daemon may have left its framebuffer bound to a plane on this
    // CRTC; if we pick the wrong (overlay) plane our output renders UNDER it.
    // Choose the plane currently bound to our CRTC with a live FB, and mark all
    // other planes bound to this CRTC for disabling in the same commit.
    let crtc_info = card
        .get_crtc(*res.crtcs().first().ok_or_else(|| anyhow!("stage=crtc: none"))?)
        .map_err(|e| anyhow!("stage=crtc: {e}"))?;
    let crtc = crtc_info.handle();
    let planes = card
        .plane_handles()
        .map_err(|e| anyhow!("stage=plane: {e}"))?;
    let mut bound_planes: Vec<drm::control::plane::Handle> = Vec::new();
    for &pl in planes.iter() {
        let props = card.get_properties(pl)?;
        let (handles, values) = props.as_props_and_values();
        let mut on_our_crtc = false;
        for (h, v) in handles.iter().zip(values.iter()) {
            if card.get_property(*h)?.name().to_str()? != "CRTC_ID" {
                continue;
            }
            // as_props_and_values yields raw u64 values; CRTC_ID raw value == handle id
            let crtc_raw: u32 = crtc.into();
            if *v == crtc_raw as u64 {
                on_our_crtc = true;
            }
        }
        if on_our_crtc {
            bound_planes.push(pl);
        }
    }
    let plane = match bound_planes.first() {
        Some(p) => *p,
        None => *planes.first().ok_or_else(|| anyhow!("stage=plane: no planes"))?,
    };
    let planes_to_disable: Vec<_> = bound_planes.iter().filter(|p| **p != plane).collect();

    // Stage: dumb buffer + framebuffer
    let db = card
        .create_dumb_buffer((width, disp_height as u32), DrmFourcc::Xrgb8888, 32)
        .map_err(|e| anyhow!("stage=buffer: create: {e}"))?;
    let db2 = card
        .create_dumb_buffer((width, disp_height as u32), DrmFourcc::Xrgb8888, 32)
        .map_err(|e| anyhow!("stage=buffer: create2: {e}"))?;
    let fb = card
        .add_framebuffer(&db, 24, 32)
        .map_err(|e| anyhow!("stage=buffer: add_framebuffer: {e}"))?;
    let fb2 = card
        .add_framebuffer(&db2, 24, 32)
        .map_err(|e| anyhow!("stage=buffer: add_framebuffer2: {e}"))?;

    // Stage: atomic commit wiring connector→crtc→plane→fb
    let mut req = AtomicModeReq::new();
    req.add_property(
        con_info.handle(),
        find_prop_id(&card, con_info.handle(), "CRTC_ID")?,
        drm::control::property::Value::CRTC(Some(crtc_info.handle())),
    );
    let blob = card
        .create_property_blob(&mode)
        .map_err(|e| anyhow!("stage=atomic: mode blob: {e}"))?;
    req.add_property(
        crtc_info.handle(),
        find_prop_id(&card, crtc_info.handle(), "MODE_ID")?,
        blob,
    );
    req.add_property(
        crtc_info.handle(),
        find_prop_id(&card, crtc_info.handle(), "ACTIVE")?,
        drm::control::property::Value::Boolean(true),
    );
    req.add_property(
        plane,
        find_prop_id(&card, plane, "FB_ID")?,
        drm::control::property::Value::Framebuffer(Some(fb)),
    );
    req.add_property(
        plane,
        find_prop_id(&card, plane, "CRTC_ID")?,
        drm::control::property::Value::CRTC(Some(crtc_info.handle())),
    );
    req.add_property(plane, find_prop_id(&card, plane, "SRC_X")?, drm::control::property::Value::UnsignedRange(0));
    req.add_property(plane, find_prop_id(&card, plane, "SRC_Y")?, drm::control::property::Value::UnsignedRange(0));
    req.add_property(
        plane,
        find_prop_id(&card, plane, "SRC_W")?,
        drm::control::property::Value::UnsignedRange((mode.size().0 as u64) << 16),
    );
    req.add_property(
        plane,
        find_prop_id(&card, plane, "SRC_H")?,
        drm::control::property::Value::UnsignedRange((mode.size().1 as u64) << 16),
    );
    req.add_property(plane, find_prop_id(&card, plane, "CRTC_X")?, drm::control::property::Value::SignedRange(0));
    req.add_property(plane, find_prop_id(&card, plane, "CRTC_Y")?, drm::control::property::Value::SignedRange(0));
    req.add_property(
        plane,
        find_prop_id(&card, plane, "CRTC_W")?,
        drm::control::property::Value::UnsignedRange(mode.size().0 as u64),
    );
    req.add_property(
        plane,
        find_prop_id(&card, plane, "CRTC_H")?,
        drm::control::property::Value::UnsignedRange(mode.size().1 as u64),
    );

    // Disable any competing planes on this CRTC so ours is the visible one
    for &p in &planes_to_disable {
        let ph = *p;
        req.add_property(
            ph,
            find_prop_id(&card, ph, "FB_ID")?,
            drm::control::property::Value::Framebuffer(None),
        );
        req.add_property(
            ph,
            find_prop_id(&card, ph, "CRTC_ID")?,
            drm::control::property::Value::CRTC(None),
        );
    }

    card.atomic_commit(AtomicCommitFlags::ALLOW_MODESET, req)
        .map_err(|e| anyhow!("stage=atomic: commit: {e}"))?;

    Ok(DrmBackend {
        card,
        mode,
        dbs: [db, db2],
        fbs: [fb, fb2],
        cur: 0,
        bound_plane: Some(plane),
        master: true,
    })
}

impl DrmBackend {
    /// Open the first card hosting a Touch Bar panel.
    pub fn open(width: u32) -> Result<DrmBackend> {
        let mut errors = Vec::new();
        let mut entries: Vec<_> = std::fs::read_dir("/dev/dri/")?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if !(name.starts_with("card") && name.len() <= 5) {
                continue; // skip renderD* and by-path
            }
            match try_open_card(&entry.path(), width) {
                Ok(b) => return Ok(b),
                Err(e) => errors.push(format!("{}: {e}", entry.path().display())),
            }
        }
        bail!("no Touch Bar found:\n{}", errors.join("\n"))
    }

    pub fn mode_size(&self) -> (usize, usize) {
        let (w, h) = self.mode.size();
        (w as usize, h as usize)
    }

    pub fn stride_px(&self) -> usize {
        Buffer::pitch(&self.dbs[0]) as usize / 4
    }

    fn fb_prop(&self, plane: drm::control::plane::Handle, name: &str) -> Result<drm::control::property::Handle> {
        find_prop_id(&self.card, plane, name)
    }

    pub fn map(&mut self) -> Result<DumbMapping<'_>> {
        self.card
            .map_dumb_buffer(&mut self.dbs[self.cur])
            .map_err(|e| anyhow!("map: {e}"))
    }

    /// Push the just-drawn back buffer to the panel.
    ///
    /// appletbdrm transmits a frame to the panel only when the plane's FB_ID
    /// changes (a flip); DIRTYFB is a no-op on this driver. We therefore
    /// double-buffer and atomically flip FB_ID on every redraw.
    pub fn flip(&mut self) -> Result<()> {
        let plane = self
            .bound_plane
            .ok_or_else(|| anyhow!("no bound plane"))?;
        let next = 1 - self.cur;
        let mut req = AtomicModeReq::new();
        req.add_property(
            plane,
            self.fb_prop(plane, "FB_ID")?,
            drm::control::property::Value::Framebuffer(Some(self.fbs[next])),
        );
        self.card
            .atomic_commit(AtomicCommitFlags::empty(), req)
            .map_err(|e| anyhow!("flip: {e}"))?;
        self.cur = next;
        Ok(())
    }

    /// Blit a landscape surface (x = bar length, y = thickness) into the back
    /// buffer and page-flip it to the panel. MTMR_FLIP=1 mirrors the length
    /// axis (fixes reversed button order on this panel's orientation).
    pub fn present(&mut self, land: &crate::surface::Surface) -> Result<()> {
        let (w, h) = self.mode_size();
        let stride = self.stride_px();
        let flip = std::env::var("MTMR_FLIP").map(|v| v == "1").unwrap_or(false);
        {
            let mut map = self.map()?;
            let buf = map.as_mut();
            // land: x = length (0..2008), y = thickness (0..60)
            // drm:  x = thickness, y = length
            for l in 0..h {
                for t in 0..w {
                    let (src, dst) = if flip {
                        ((t * land.stride + l) * 4, ((h - 1 - l) * stride + t) * 4)
                    } else {
                        ((t * land.stride + l) * 4, (l * stride + t) * 4)
                    };
                    buf[dst] = land.buf[src];
                    buf[dst + 1] = land.buf[src + 1];
                    buf[dst + 2] = land.buf[src + 2];
                }
            }
        }
        self.flip()
    }
}

/// Render the given preset items to the Touch Bar via atomic page flips.
/// With hold=true, re-renders periodically and keeps DRM master until SIGINT/SIGTERM.
pub fn render_to_card(hold: bool, items: &[crate::preset::Item]) -> Result<()> {
    let mut backend = DrmBackend::open(60)?;
    let (w, h) = backend.mode_size();
    println!("OK mode={w}x{h} (atomic backend)");
    let font = crate::render::find_font()?;
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default())
        .map_err(|e| anyhow!("font load: {e:?}"))?;
    let rects = crate::render::layout(items, h);
    let mut land = crate::surface::Surface::new(h, w, h);
    crate::render::draw(&mut land, items, &rects, &font, &Default::default());
    backend.present(&land)?;
    backend.present(&land)?; // both buffers hold the same frame
    if !hold {
        return Ok(());
    }
    loop {
        std::thread::sleep(std::time::Duration::from_secs(2));
        backend.present(&land)?;
    }
}

/// Fill a tightly-packed XRGB8888 buffer with the test pattern (PNG fallback path).
pub fn xrgb8888_bytes(w: usize, h: usize) -> Vec<u8> {
    let mut buf = vec![0u8; w * h * 4];
    crate::pattern::fill_test_pattern(&mut buf, w, h);
    buf
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
