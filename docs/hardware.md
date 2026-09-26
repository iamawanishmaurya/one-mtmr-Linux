# MTMR-Linux — Hardware Record (MacBookPro16,2, T2, t2linux)

Probed live on 2026-09-26 (kernel 7.2.6-arch2-Watanare-T2-4-t2).

## Touch Bar display (render target)

| Property | Value |
|---|---|
| DRM card | `/dev/dri/card0` (primary node; **must** open O_RDWR) |
| Connector | `card0-USB-1`, connector_id **39**, interface `USB` |
| Mode | **60x2008** @ 60 Hz (panel 2170×60 rotated 90°; visible width 60 px) |
| Kernel driver | `appletbdrm` (loaded) |
| Backlight module | `hid_appletb_bl` (loaded) |
| Keyboard mode module | `hid_appletb_kbd` (loaded) |

- There is **no fbdev node** for the Touch Bar (`fb0` is i915); render via DRM dumb buffer + legacy `SetCrtc`.
- Dumb buffer allocation is larger than pitch×height (page-aligned); use the buffer's `pitch` and never assert exact size.
- Dumb-buffer mmap requires the fd opened read/write; `File::open` (RO) fails mmap with EACCES.
- Legacy `SetCrtc` requires **DRM master**: call `DRM_IOCTL_SET_MASTER` (`drm_ffi::auth::acquire_master`) first; as non-master it returns EACCES.
- Device contention: an existing system service `mtmr-linux.service` (`/usr/bin/mtmr-linux`, "Touch Bar daemon (MTMR tap model for niri)", runs as `nobody`, `Conflicts=tiny-dfr.service`) holds card0 and DRM master. `acquire_master` returns EBUSY while it runs. Our daemon must `Conflicts=mtmr-linux.service tiny-dfr.service` and take over.

Verified: `mtmr --drm` → `OK connector=39 mode=60x2008 fb=framebuffer::Handle(41)` with the 8-band test pattern on the physical bar (while the stock daemon was stopped).

## Touch input

| Property | Value |
|---|---|
| Device | "Apple Inc. Touch Bar Display Touchpad" |
| Evdev node | **`/dev/input/event13`** (sysfs `input13`, USB `5-6:2.0`, HID `0003:05AC:8302`) |
| Properties | `INPUT_PROP_DIRECT` + `BUTTONPAD` — direct touch surface, no pointer |
| X axis | `ABS_X` (code 0) **min 0 max 32767** — full bar length |
| Y axis | `ABS_Y` (code 1) min 0 max 127 — bar thickness |
| MT axes | `ABS_MT_SLOT` 47, `ABS_MT_POSITION_X` 53, `ABS_MT_POSITION_Y` 54, `ABS_MT_TRACKING_ID` 59 |
| Buttons | `BTN_TOUCH`, `BTN_TOOL_FINGER`, 2/3/4/5-tool taps |

Coordinate mapping: `x_px = ev_x * 2008 / 32767` along the mode's 2008 axis; that axis maps 1:1 to the physical bar's left→right length (orientation to confirm visually during Phase 2 hit-testing). Note: plain `EVIOCGABS` ioctl reports zeros for all axes on this device — read real ranges via `libinput record` or the HID report descriptor.

## Backlight / keyboard illumination

- `/sys/class/backlight/appletb_backlight/` — max_brightness **2** (Touch Bar / keyboard-mode brightness, via `hid_appletb_bl` at `/sys/bus/hid/drivers/hid-appletb-bl/0003:05AC:8102.0007`)
- `/sys/class/backlight/intel_backlight/` — max_brightness 17777 (main display)
- `/sys/class/leds/:white:kbd_backlight/` — keyboard backlight LED

## Phase 2 requirements surfaced by this spike

1. On SIGTERM/exit: release DRM master and restore Touch Bar keyboard mode (`hid_appletb_kbd` governs display vs F-row mode).
2. Systemd unit must `Conflicts=mtmr-linux.service` (existing daemon) — not just tiny-dfr.
3. Touch hit-testing: scale ABS_X 0–32767 → 0–2008 mode axis.
