# Architecture Research — MTMR-Linux

## Shape

Single Rust daemon (`mtmr`) + JSON preset (`~/.config/mtmr/items.json`), run as systemd user service.

```
┌──────────┐  items.json   ┌───────────────────────────────┐
│ ~/.config│──────────────▶│ mtmr daemon                   │
└──────────┘               │  preset parser → item model   │
                           │  widget loop (per-item timers)│
                           │  layout renderer → RGB buffer │
                           │  DRM output (appletbdrm card) │
                           │  evdev touch input → hit test │
                           │  uinput → esc/media/volume    │
                           └───────────────────────────────┘
```

## Key components

1. **Preset parser** — serde_json → `Vec<Item>`; per-item: type, width, align, refreshInterval, action, title/image.
2. **Renderer** — simple software rasterizer (or `tiny-skia`) drawing text/icons onto a 2170×60 RGB surface; item layout left/center/right like MTMR.
3. **DRM output** — enumerate DRM cards, find the Touch Bar panel (`appletbdrm` / connector name), modeset, mmap dumb buffer, blit surface. Reference: tiny-dfr's output code.
4. **Input** — evdev on the iBridge multitouch device; translate touch x → item hit-testing; tap = action, long-tap = secondary action (MTMR convention).
5. **Key injection** — /dev/uinput for Esc, media keys; nothing requires accessibility hacks on Linux (input via uinput group perms).
6. **Widget providers** — battery, cpu, time, mpris-music, shell-script; each ticks on its own refreshInterval, produces title/emoji/image + optional ANSI color.

## Integration decisions

- Conflict with `tiny-dfr`: ship unit with `Conflicts=tiny-dfr.service` and docs to disable it; both grab the same DRM card.
- If `appletbdrm` isn't bound, fall back to an offscreen SDL/X11 preview window — huge for development and CI testing without hardware.
- Config hot-reload optional; SIGHUP minimum.

## Pitfalls to respect

- DRM device selection must be robust: Touch Bar may be /dev/dri/card0 or card1.
- Keyboard backlight slider needs `hid_appletb_bl`; brightness needs /sys/class/backlight — verify node names on this machine during Phase 1 spike.
- uinput requires udev permission (input group) for user services.
