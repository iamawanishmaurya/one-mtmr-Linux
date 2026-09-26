# Stack Research — MTMR-Linux

## Touch Bar hardware path on T2 Linux (verified on this machine)

- USB: `05ac:8302` Touch Bar Display, `05ac:8102` Backlight, `05ac:8233` iBridge.
- Kernel drivers (t2linux kernels / upstreaming effort): `appletbdrm` (DRM tiny driver exposing the 2170×60 Touch Bar panel as a DRM card/fbdev), `hid_appletb_bl` (backlight), `hid_appletb_kbd` (keyboard mode switch). Linux 6.17+ mainline enabled x86 MacBook Pro Touch Bars.
- Touch input: hid-multitouch events on the iBridge HID device.
- Reference userspace daemon: **tiny-dfr** (AsahiLinux/tiny-dfr) — Rust, grabs the correct DRM device, renders, and is what the t2linux wiki recommends. Our daemon must claim the same device (systemd conflict or device handshake needed).

## Language choice

- **Rust (recommended)**: tiny-dfr precedent, DRM direct-rendering crates (`drm`, `drm-fourcc`), no runtime deps, systemd-friendly. Touch input via `evdev` crate.
- Alternative: C (libdrm + libevdev) or Python (slow for 60fps-ish refresh; fine for widget refresh only).
- GTK/Qt window approaches are wrong — the bar is a separate panel display, not a window in the compositor.

## Supporting stack

- uinput (Rust `enigo` or raw `uinput` crate) to inject Esc/media/volume keys.
- Audio: `wpctl` (PipeWire) or ALSA mixer; brightness: `/sys/class/backlight` (kernel `apple-ib-bl`).
- Battery/CPU widgets: `/sys/class/power_supply/BAT*`, `/proc/stat`.
- Config: JSON via `serde_json` — mirror MTMR's `items.json` schema.
- Packaging: systemd user service + Arch PKGBUILD.

## Sources

- https://wiki.t2linux.org/guides/postinstall
- https://github.com/AsahiLinux/tiny-dfr
- https://lwn.net/Articles/986150
- https://github.com/Dunedan/mbp-2016-linux (non-T2 reference)
