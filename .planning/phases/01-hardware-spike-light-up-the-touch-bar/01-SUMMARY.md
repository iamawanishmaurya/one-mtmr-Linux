# Phase 1 Summary — Hardware Spike (2026-09-26)

## Outcome: SUCCESS

All 4 success criteria met:

1. **Test pattern on physical bar** ✓ — `sudo target/release/mtmr --drm` printed `OK connector=39 mode=60x2008 fb=framebuffer::Handle(41)`; 8-band pattern rendered on the physical Touch Bar (while the stock daemon was briefly stopped, then restored).
2. **Touch coordinates** ✓ — touch device identified as `/dev/input/event13` ("Touch Bar Display Touchpad", INPUT_PROP_DIRECT). X axis 0–32767 across the 2008-px mode axis; mapping documented. (Plan deviation: EVIOCGABS ioctl returns zeros on this device, so ranges were read via `libinput record` — reason recorded in docs/hardware.md.)
3. **Backlight/keyboard-mode docs** ✓ — docs/hardware.md records appletb_backlight (max 2), intel_backlight, white:kbd_backlight LED, hid_appletb_bl/kbd driver paths.
4. **Hardware-free fallback** ✓ — `mtmr --dump-png /tmp/tb.png` writes a 60x2008 PNG; `cargo test` green.

## Hard-won facts (now in docs/hardware.md)

- DRM fd must be O_RDWR for dumb-buffer mmap (RO open → mmap EACCES)
- appletbdrm dumb buffers are page-padded — never assert exact buffer size; use `Buffer::pitch`
- `SetCrtc` needs DRM master (`drm_ffi::auth::acquire_master`)
- An existing daemon (`mtmr-linux.service`, /usr/bin/mtmr-linux, user's earlier project) holds card0/DRM master → EBUSY; our future unit must Conflicts with it

## Artifacts

- Rust crate `mtmr` (src/main.rs, pattern.rs, drm_out.rs), binary renders pattern to bar + PNG fallback
- docs/hardware.md

## Carry-over to Phase 2

- Conflicts list: `mtmr-linux.service` + `tiny-dfr.service`
- SIGTERM restore of keyboard mode + master release
- Touch: scale ABS_X 0–32767 → 0–2008; evdev via `evdev` crate
