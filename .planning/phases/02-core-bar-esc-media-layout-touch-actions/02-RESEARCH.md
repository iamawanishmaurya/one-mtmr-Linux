# Phase 2 Research — Core Bar: Esc, Media, Layout, Touch Actions

Inherits Phase 1 hardware facts (docs/hardware.md) and old-project validated patterns (/home/Astra/opencode/mtmr-linux docs, summarized in memory + below).

## Item model & layout (REND-02)

- Parse `~/.config/mtmr/items.json` — Phase 2 subset of MTMR schema: `type` (`staticButton`, `escape`, `previous`, `play`, `next`, `exitTouchbar`), `title`, `width`, `align` (left/center/right), `actions` array (`{trigger, action, keycode|command}`) — MTMR uses `action: "shellScript"`/`"keycode"`; Phase 2 supports `keycode` (numeric or X11-style name) and `command` (shell).
- Layout: mode axis is 2008 px wide (60 px tall). Three segments: left / center (centered cluster) / right. Within a segment, items stack by `width` (default 100 px MTMR-ish, or proportional). Bar height 60; text centered.
- Text rendering: `fontdue` crate rasterizing a system TTF (`fc-match` found `/usr/share/fonts/noto/NotoSans-Regular.ttf` at runtime; fallback scan `/usr/share/fonts/**/{NotoSans-Regular,DejaVuSans}.ttf`). No Cairo/pango (keep deps light; old project's Cairo stack was its biggest dependency weight).
- Render buffer stays 60×2008 XRGB8888 with pitch stride (Phase 1 pattern.rs refactored into a proper `surface.rs` with `clear`, `fill_rect`, `draw_text`).

## Touch input (INPT-01)

- `evdev` crate on `/dev/input/event13` (path from docs/hardware.md; enumerate at runtime by device name "Touch Bar Display Touchpad" + ABS_X present, don't hardcode).
- Coordinate scale: `x_px = ABS_X * 2008 / 32767`, y within 0..127 → row center.
- **Tap-up state machine (validated in old project, port the logic):**
  - Down: record origin, arm long-press deadline (500 ms).
  - Motion > 12 px slop → cancel tap, it's a drag (ignored Phase 2).
  - Long-press deadline fires while finger down → long-press action, mark consumed.
  - Up before deadline & within slop → single-tap action.
- Non-blocking: evdev async via `EventStream` (tokio) — but avoid tokio; use a polling thread + channel, or `epoll`. Decision: dedicated reader thread sending `TouchState` messages over `std::sync::mpsc` to the render loop; simplest and dep-free.

## Key injection (INPT-02/03)

- `/dev/uinput` via `evdev::uinput::VirtualDeviceBuilder` with keys: KEY_ESC, KEY_PREVIOUSSONG, KEY_PLAYPAUSE, KEY_NEXTSONG, KEY_VOLUMEUP/DOWN/MUTE (volume used Phase 3).
- Emit: EV_KEY press, EV_SYN SYN_REPORT, sleep 80 ms, release, SYN_REPORT. Session compositor (niri) sees them as a real keyboard.
- Permissions: run as user in `input` group during dev (currently root via sudo — fine for now; user-service + groups lands in Phase 5 packaging).

## Exit/cleanup (INPT-04)

- SIGTERM/SIGINT handler: break loop, `release_master`, exit 0. uinput VirtualDevice drops on process exit (kernel removes it). Keyboard-mode note: `hid_appletb_kbd` is inert on this machine (validated in old project) — nothing to restore kernel-side beyond releasing master; document this in the handler.

## Live-reload of preset

- Phase 2: re-read JSON on SIGHUP only (hot-reload via inotify is a v2 feature).

## Validation Architecture

- SC (REND-02): automated — `cargo test` asserts layout math (segment widths, centering) and text raster produces non-empty glyphs; `--dump-png` after preset render shows items (manual visual check on bar via --live).
- SC (INPT-01): semi-automated — `mtmr --selftest-touch` replays a synthetic evdev event sequence through the classifier unit test (pure functions, no hardware); hardware tap confirmed manually.
- SC (INPT-02/03): automated — `mtmr --selftest-uinput` creates the virtual device, injects KEY_ESC, exits 0; user-visible in `wev` or compositor.
- SC (INPT-04): automated — start `--live`, SIGTERM, assert exit 0 and `journal`-free (no panic); master released (subsequent acquire succeeds).

## Sources

- docs/hardware.md (Phase 1 probe)
- Old project src/touch.rs tap-up machine + TROUBLESHOOTING §7/§15/§17 (validated behavior, ported not copied — different deps)
- evdev/fontdue crate docs (docs.rs)
