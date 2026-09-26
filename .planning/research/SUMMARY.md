# Project Research Summary — MTMR-Linux

## Key Findings

**Stack:** Rust daemon (`serde_json`, `drm`/`drm-fourcc`, `evdev`, `uinput`, `tiny-skia`), rendering directly to the Touch Bar DRM card exposed by `appletbdrm` on this t2linux MacBookPro16,2. Shipped as a systemd user service that conflicts with tiny-dfr. X11/SDL preview fallback for hardware-free development.

**Table Stakes:** esc key, media keys, volume/brightness (keys + sliders), static & shell-script buttons, time/battery/cpu/music widgets, MTMR-compatible `~/.config/mtmr/items.json` preset schema.

**Watch Out For:** kernel-dependent `appletbdrm` binding; tiny-dfr device contention; restoring Touch Bar keyboard mode on exit; uinput/DRM permissions for user services; scope creep across MTMR's ~20 plugin types.

## Implications for Roadmap

- Phase 1 must be a hardware spike: confirm DRM card, touch device, backlight nodes on this machine, and get a trivial colored bar rendered for real before building features.
- Build in vertical slices: render + touch + esc/media first (parity with tiny-dfr), then widget ecosystem, then MTMR preset compatibility and packaging.
- Every phase is verifiable via the preview fallback even when away from hardware.

## Sources

- t2linux wiki post-install: https://wiki.t2linux.org/guides/postinstall
- tiny-dfr: https://github.com/AsahiLinux/tiny-dfr
- LWN Touch Bar support: https://lwn.net/Articles/986150
- MTMR: https://github.com/Toxblh/MTMR (cloned reference)
- MTMR presets: https://github.com/Toxblh/MTMR-presets
