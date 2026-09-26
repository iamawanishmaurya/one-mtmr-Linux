---
phase: 2
plan: 01
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - src/main.rs
  - src/surface.rs
  - src/preset.rs
  - src/render.rs
  - src/touch.rs
  - src/uinput.rs
  - src/drm_out.rs
  - assets/default-items.json
autonomous: false
requirements: [REND-02, INPT-01, INPT-02, INPT-03, INPT-04, PRE-01]
---

# Plan 01 — Core Bar: Esc, Media, Layout, Touch Actions

**Goal:** `mtmr --live` renders a real item bar from `~/.config/mtmr/items.json` (left/center/right segments), touch taps trigger item actions (long-press for secondary), esc + media keys inject via uinput, and SIGTERM cleans up with exit 0. The bar stays live for user feedback during the rest of development.

## Artifacts this phase produces

- `src/surface.rs` — `Surface { w, h, stride, buf }` with `clear`, `fill_rect`, `hline`; replaces ad-hoc pattern buffer
- `src/preset.rs` — `Item { kind, title, width, align, actions }`, `Action { trigger, keycode | command }`, `load(path) -> Vec<Item>` (serde_json), phase-2 type subset (`escape`, `staticButton`, `previous`, `play`, `next`, `exitTouchbar`); unknown types logged+skipped (PRE-02 behavior starts here)
- `src/render.rs` — `layout(items, bar_w) -> Vec<Rect>` (left/center/right segments, center cluster centered); `draw(surface, items, layout, font)`; `fontdue`-based text centering, system TTF via fc-match fallback scan
- `src/touch.rs` — `TouchState` machine (Down/Motion/Up + 500 ms long-press deadline, 12 px slop) as pure functions: `classify(events) -> TapKind::Single | Long | None` + hit-test `item_at(rects, x)`
- `src/uinput.rs` — `KeyInjector::new()` (VirtualDeviceBuilder with ESC + media keys), `inject(key)`
- `assets/default-items.json` — default preset: esc (left), previous/play/next (center), exitTouchbar (right)
- `--selftest-touch` and `--selftest-uinput` CLI flags; `--live` extended to render items and consume touch

## Tasks

### Task 1 — Preset, surface, layout, text render (Wave 1, type: tracer)

<read_first>
- src/pattern.rs, src/drm_out.rs, src/main.rs (current render path)
- reference/MTMR/MTMR/defaultPreset.json (MTMR schema reference)
- docs/hardware.md (60×2008, stride rules)
</read_first>

<action>
Add deps: `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `fontdue = "0.9"`. Create `src/surface.rs` (`Surface::new(w,h,stride)`, `clear(color)`, `fill_rect`, pixel accessor `px(x,y,&mut)` writing B,G,R,X order). Move fill_test_pattern logic to use Surface. `src/preset.rs`: serde structs per research Item/Action; `load()` returns `anyhow::Result<Vec<Item>>`, logging (tracing or eprintln) and skipping unknown `type` values. `src/render.rs`: `layout` segment math — left items packed from x=8, right items packed from right edge minus 8, center cluster centered on 1004; item gap 8 px; `Item.width` px (default 100). Text: fontdue rasterize `title` at ~28 px height, centered in rect, white; dark background (10,10,10). Ship `assets/default-items.json` per research. `--render-only [preset]` CLI flag: parse + layout + draw + dump PNG (no DRM) for testability. `--live` now renders the preset instead of the color bands.
</action>

<acceptance_criteria>
- `cargo test --release` includes preset-parse tests (valid subset parses; unknown type skipped; defaults applied) and layout tests (left pack order, right alignment to bar edge, center cluster symmetric)
- `target/release/mtmr --render-only assets/default-items.json` exits 0 and writes `/tmp/mtmr-bar.png` (60x2008)
- With the old daemon disabled, `sudo target/release/mtmr --live` shows esc/prev/play/next/exit items instead of color bands (user visual confirmation)
</acceptance_criteria>

<verify>
  <automated>cargo test --release 2>&1 | tail -2 && ./target/release/mtmr --render-only assets/default-items.json && file /tmp/mtmr-bar.png | grep -o '60 x 2008'</automated>
  <fails_when>cargo test reports failures, --render-only exits non-zero, or PNG is missing/wrong size</fails_when>
</verify>

### Task 2 — Touch input + tap classification (Wave 2)

<read_first>
- docs/hardware.md (event13, ABS_X 0–32767, EVIOCGABS-zero caveat)
- src/preset.rs, src/render.rs (rects for hit-testing)
</read_first>

<action>
Add dep `evdev = "0.13"`. `src/touch.rs`: spawn a reader thread opening the device found by scanning `/dev/input/event*` for name "Touch Bar Display Touchpad" (fail with clear error if absent); forward EV_KEY:BTN_TOUCH/EV_ABS:ABS_X(+MT_SLOT handling: use ABS_X for slot 0 simplicity in Phase 2) as `TouchState` over `std::sync::mpsc`. Classifier pure functions: `on_down(x,y,t0)`, `on_move(x,y)`, `on_up(t)` with LONG_PRESS_MS=500, SLOP_PX=12 (bar-space x after scaling). `--live` main loop: poll mpsc with 100 ms timeout, redraw on state change (press highlight: invert item rect), resolve tap → hit-test → dispatch action (Task 3's injector; commands via std::process::Command). `--selftest-touch` feeds a synthetic event script (down at x=100, up at t=100 ms; down, wait 600 ms; drag beyond slop) through the classifier and prints classifications, exit 0 iff all three match Single/Long/None.
</action>

<acceptance_criteria>
- `target/release/mtmr --selftest-touch` exits 0 printing SINGLE, LONG, NONE
- Tapping the esc region on the physical bar dispatches the escape action; long-press does the long-press action (user feedback)
- The bar re-renders with a visible highlight while a finger is down
</acceptance_criteria>

<verify>
  <automated>./target/release/mtmr --selftest-touch; echo "exit=$?"</automated>
  <fails_when>exit code non-zero or classifications missing</fails_when>
</verify>

### Task 3 — uinput key injection + preset wiring (Wave 3)

<read_first>
- src/touch.rs, src/preset.rs (Action shape)
- docs/hardware.md (uinput permission note)
</read_first>

<action>
`src/uinput.rs`: `KeyInjector::new()` builds a virtual device (evdev::uinput::VirtualDeviceBuilder, name "mtmr keys") with keys KEY_ESC(1), KEY_PREVIOUSSONG(165), KEY_PLAYPAUSE(164), KEY_NEXTSONG(163), KEY_VOLUMEUP(115), KEY_VOLUMEDOWN(114), KEY_MUTE(113); `inject(code)` press+SYN, 80 ms, release+SYN. Preset Action: `{"trigger":"singleTap","action":"keycode","keycode":53}` or `"keycode":"esc"` (name map), and `{"action":"command","command":"..."}`. Wire into --live dispatch. `--selftest-uinput`: create injector, inject KEY_ESC, drop, exit 0. Convert the Phase-1 --live hold loop into the main event loop (render + touch + signals); keep 5 s re-render.
</action>

<acceptance_criteria>
- `target/release/mtmr --selftest-uinput` exits 0; ESC is observable in a text console/`wev` when run live (user confirms)
- Default preset esc button injects KEY_ESC; prev/play/next inject media keys; exitTouchbar action terminates the daemon
</acceptance_criteria>

<verify>
  <automated>sudo ./target/release/mtmr --selftest-uinput; echo "exit=$?"</automated>
  <fails_when>exit code non-zero or device creation errors</fails_when>
</verify>

### Task 4 — Signal cleanup + live handoff (Wave 4)

<read_first>
- src/main.rs, src/drm_out.rs (master acquire/release)
</read_first>

<action>
SIGTERM/SIGINT handler (ctrlc already a dep): exit 0 after releasing DRM master (extend render_to_card to expose release; or run loop inside and return). Extend --live: on SIGHUP re-read preset and re-render (document in README). Add a `mtmr` section to README.md: dev workflow (sudo target/release/mtmr --live), preset location (~/.config/mtmr/items.json preferred over assets), signal behavior. Kill the leftover Phase-1 background --live process and restart with the new build so the bar stays live for the user.
</action>

<acceptance_criteria>
- `sudo timeout -s TERM 3 target/release/mtmr --live` exits 0 (no panic, no coredump)
- README.md contains `## mtmr` with preset path and signals
- The physical bar shows the default preset items (not color bands) at the end of this task
</acceptance_criteria>

<verify>
  <automated>sudo timeout -s TERM 3 ./target/release/mtmr --live; echo "exit=$?"</automated>
  <fails_when>exit code other than 0 (124 means TERM was not handled; 101 panic)</fails_when>
</verify>

## must_haves

truths:
- "`--render-only` produces a 60x2008 PNG of the laid-out preset without DRM (explicit)"
- "Unknown item types are logged and skipped; remaining items render (explicit)"
- "Touch tap-up classifier: single/long/none with 500 ms deadline and 12 px slop (explicit, unit-tested)"
- "ESC + media keys inject as real keycodes via /dev/uinput (explicit)"
- "SIGTERM during --live exits 0 and releases DRM master (explicit)"
- "Physical bar shows JSON-defined items with tap actions while --live runs (backstop: needs eyes + finger)"
  verification: backstop

prohibitions:
- statement: "Never open /dev/dri/card1 and never grab /dev/input devices exclusively"
  status: resolved
  verification: "code review: only card0 default path; no EVIOCGRAB call"
- statement: "Never block the render loop on touch reads (reader thread + channel only)"
  status: resolved
  verification: "code review: mpsc recv_timeout in loop"

## Risks

- fontdue glyph metrics vs 60 px row height — clamp to surface, test with default preset strings
- MT slot handling simplified to ABS_X — multi-finger drags may misattribute; acceptable Phase 2 (taps only), flagged for Phase 3 sliders
- Root-run for dev (sudo) matches Phase 1; user-service with input group is Phase 5 — do not attempt early
