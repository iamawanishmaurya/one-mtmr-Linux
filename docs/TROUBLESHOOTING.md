# MTMR-Linux — Troubleshooting & Battle Scars

Every problem we actually hit while building this, with the fix that worked.
Ordered roughly by the rendering pipeline: kernel → DRM → daemon → touch → config.

## Kernel / hardware

### 1. Legacy `SetCrtc` wedges the iBridge display endpoint (WORST BUG)
- **Symptom:** modeset ioctl returns success, but the panel never updates — and
  from then on *every* appletbdrm message times out (`dmesg: Failed to send
  message (-110)`) until **reboot**. USB rebind/re-auth does NOT recover it.
- **Fix:** never use legacy SetCrtc. Only atomic commits (see #2).
- **Diagnosis chain:** `dmesg | grep appletbdrm` → unbind/bind attempt → if the
  probe itself fails with -110, reboot.

### 2. `DIRTYFB` is a no-op on appletbdrm — panel only updates on FB_ID flip
- **Symptom:** kernel scans out our framebuffer (`udevadm`/probe shows CRTC fb =
  our fb) yet the panel shows an old frame forever.
- **Fix:** double-buffer (two dumb buffers/framebuffers) and atomically flip
  `FB_ID` every redraw — the flip is what makes the driver transmit pixels
  over USB.

### 3. Stale panel after many daemon restarts ("shows the old version")
- **Symptom:** daemon runs, commits succeed, no kernel errors — but the panel
  keeps showing a previous frame.
- **Fix:** rebind the driver: `sudo mtmr --recover` (unbind + bind of
  `5-6:2.1`). The service runs this automatically as `ExecStartPre`, so every
  start gets a fresh display session.

### 4. Backlight resets to 0 after driver rebind (bar "blank")
- **Symptom:** everything works, panel is pitch dark; frames transmit fine.
- **Fix:** the daemon writes max brightness to
  `/sys/class/backlight/appletb_backlight/brightness` at startup. (A dark
  panel also silently eats taps — old-project lesson.)

### 5. Wrong plane → output renders UNDER a stale framebuffer
- **Symptom:** atomic commit succeeds but the old daemon's frame stays visible.
- **Cause:** a dead daemon's fb stays bound to the CRTC's primary plane;
  `plane_handles()[0]` may be an overlay that renders under it.
- **Fix:** scan plane properties, commit onto the plane whose `CRTC_ID` equals
  our CRTC, and disable competing planes in the same commit.

### 6. Panel orientation: thickness axis is inverted
- **Symptom:** text upside-down on the physical bar.
- **Fix:** `MTMR_VFLIP=1` (mirror the 60px axis). Length axis needs **no**
  flip (esc = physical left); a length mirror puts esc on the right.

## DRM programming (rust `drm` crate)

### 7. mmap of dumb buffer fails EACCES
- The DRM fd must be opened **O_RDWR** (`File::open` read-only is not enough).

### 8. `SetCrtc`/atomic commit fails EACCES → need DRM master
- Call `drm_ffi::auth::acquire_master` before committing; release on exit.
- `EBUSY` means another daemon holds master (check `fuser /dev/dri/card0`).

### 9. Dumb buffer size ≠ pitch × height
- appletbdrm page-pads allocations; always use the buffer's `pitch` and never
  assert exact size.

### 10. User services cannot get DRM master
- Unprivileged `acquire_master` returns EACCES on this kernel — that's why
  MTMR-Linux ships as a **system** service (same choice as t2linux's tiny-dfr).
- Also: `SupplementaryGroups=` fails in user units whose manager predates a
  group change; and `uaccess` ACLs don't apply on non-`seat0` seats
  (the Touch Bar is on `seat-touchbar`).

## Daemon bugs found in testing

### 11. No `DeviceAllow=` in the systemd unit — ever
- Whitelisting devices silently blocks `/dev/input`: display works, taps don't.
  (Inherited from the old project; kept in every unit here.)

### 12. Slider drags did nothing — `Move` events never sent
- The touch reader updated its x-position but only forwarded Down/Up.
- Fix: send `Ev::Move` on every `ABS_X` while `BTN_TOUCH` is held.

### 13. Slider stayed inverted (white) after releasing the drag
- The press highlight was only cleared when a *tap* fired; a drag release
  (`Up` after movement) left the item inverted.
- Fix: clear the highlight on every release, then decide tap/dispatch.

### 14. Stale binary trap
- Restarting the service without reinstalling the built binary shows old
  behavior. Fix: `./deploy.sh` (build → install → restart) is the only
  deployment path; never just restart.

### 15. EVIOCGABS reads zeros for the Touch Bar touchpad axes
- The ioctl returns min=max=0 for this device. Read real ranges (ABS_X
  0–32767) via `libinput record` or the HID report descriptor instead.

## Config / preset gotchas

### 16. Community presets are not valid JSON
- They contain BOMs, text preambles, `//` and `/* */` comments, trailing
  commas, and raw control characters in strings. `src/lenient.rs::sanitize`
  handles all of it; `mtmr --lint` batch-validates.

### 17. Built-in icons are implicit in MTMR presets
- Presets rely on MTMR's bundled icons for built-in types (suns, speakers,
  play glyph). A compatible renderer must draw its own — `src/icons.rs`.
  Also: sliders must look like macOS sliders (slim track + round thumb),
  not full-height rectangles.

### 18. Old-project lessons still apply
- TOML `[Timings]`-style header re-scoping (we use JSON, so safe), battery
  `capacity` over `charge_now` (drift), `pactl`/`wpctl` need
  `XDG_RUNTIME_DIR` when running as root (auto-detected in slider.rs),
  brightness 5% floor (a dark panel is a trap).
