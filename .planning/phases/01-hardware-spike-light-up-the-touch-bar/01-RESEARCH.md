# Phase 1 Research — Hardware Spike

## Ground truth probed on this machine (2026-09-26)

- Touch Bar DRM card: **`/dev/dri/card0`**; connector **`card0-USB-1`**, status `connected`, `enabled`, connector_id `39`, mode **`60x2008`** (the 2170×60 panel in portrait orientation — rendering must rotate 90° or allocate 60×2008 and write columns accordingly).
- Kernel modules loaded: `appletbdrm`, `hid_appletb_bl`, `hid_appletb_kbd`, `hid_apple`.
- `/sys/class/graphics/fb0` belongs to **i915** (main display) — there is NO fbdev node for the Touch Bar; we must use DRM directly.
- `card1` is i915 (main GPU). Never target card1.
- No Rust toolchain installed (`rustup` present, no default toolchain) — task 1 installs `stable`.
- tiny-dfr not found running/installed as a user unit on this machine (no conflict right now, but still guard).

## Rendering approach (Phase 1 test pattern)

Use the Rust `drm` + `drm-fourcc` crates against `/dev/dri/card0`:

1. Open card0, load client capabilities (`UniversalPlaness`, maybe `Atomic`).
2. Get resources; find connector id 39 / name "USB-1"; get its encoder/crtc and mode `60x2008` @ 60Hz.
3. Create a dumb buffer `60×2008` XRGB8888, map it, fill a test pattern (vertical color bands), attach as fb, modeset.
4. Optionally run with `DRM_IOCTL_MODE_SETPLANE`/legacy SetCrtc — legacy API is fine for one plane.

Rotation: the native buffer is 60 wide × 2008 tall. A "vertical band" pattern along the 2008 axis maps to horizontal segments of the physical bar. Pattern: 8 color bands of 251 px + margins; also embed a left/right edge marker (bright white at x=0 and x=59 columns) to verify orientation by looking at the bar's physical left/right ends.

Alternative (dev-only): X11 preview window drawing the same pattern (success criterion 4) — plain `x11rb` or even a PNG dump; PNG dump is dependency-free and composable with the X11 window later. Decision: Phase 1 uses a PNG dump (testable, CI-safe) + real DRM path.

## Touch input (probe-only this phase)

- Touch input device: iBridge HID multitouch — enumerate `/dev/input/event*` with `libinput record`/`evtest` naming or `by-path` under `usb5-6:2.0-0003:05AC:8302` (input13). This phase only *identifies and logs* the event node; full evdev handling is Phase 2.

## Backlight / keyboard mode (document-only this phase)

- `hid_appletb_bl` exposes the keyboard-mode/backlight controls; locate sysfs nodes under `/sys/bus/hid/drivers/hid-appletb*` and record paths in `docs/hardware.md`.
- Keyboard mode switching (display vs F-row) is governed by `hid_appletb_kbd`; document the exit-restore requirement found during probing.

## Validation Architecture

Signal-to-criterion mapping for this phase:

- **SC1 (test pattern on physical bar):** automated — probe script runs the DRM modeset and asserts exit 0 + the framebuffer is the active fb; human confirms bands visible on the bar.
- **SC2 (touch coordinates):** automated — `evtest`/`libinput record` on the identified event node emits ABS_MT_POSITION_X events whose max matches 2008 (portrait) or 2170 (rotated); recorded in docs.
- **SC3 (backlight/keyboard-mode docs):** automated — `docs/hardware.md` exists containing connector id 39, mode 60x2008, event node path, and at least one `hid_appletb_bl` sysfs path that exists on disk.
- **SC4 (X11/PNG preview fallback):** automated — preview binary/command writes `test-pattern.png` 60×2008; verified by `file` output.

## Sources

- Probed system state (commands above, this machine).
- https://github.com/AsahiLinux/tiny-dfr (DRM output pattern reference)
- Rust crates: `drm`, `drm-fourcc`, `drm-ffi` (docs.rs).
