---
phase: 1
plan: 01
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - src/main.rs
  - src/drm_out.rs
  - src/pattern.rs
  - docs/hardware.md
autonomous: false
requirements: [REND-01, REND-03]
---

# Plan 01 — Hardware Spike: Light Up the Touch Bar

**Goal:** A minimal Rust binary (`mtmr-spike`) that renders an 8-band color test pattern to the Touch Bar via `/dev/dri/card0` (`appletbdrm`, connector USB-1, mode 60x2008), with a PNG-dump fallback for hardware-free verification, plus `docs/hardware.md` recording the probed touch input node and backlight sysfs paths.

## Artifacts this phase produces

- Crate `mtmr` (binary `mtmr-spike` in Phase 1; renamed `mtmr` daemon in Phase 2)
- `src/main.rs` — CLI: `--dump-png <path>` (no hardware needed), `--drm [card]` (real bar)
- `src/pattern.rs` — `fill_test_pattern(buffer: &mut [u8], w: usize, h: usize)` — 8 vertical color bands (RGB) along the 2008 axis, white column at x=0 and x=59 as orientation markers
- `src/drm_out.rs` — `render_to_card(card: &Path) -> Result<()>`: open card0, find connector name "USB-1" (connector_id 39), mode 60x2008, create dumb buffer XRGB8888, map, fill via pattern, legacy modeset (SetCrtc)
- `docs/hardware.md` — probed hardware record

## Tasks

### Task 1 — Scaffold crate + PNG fallback (Wave 1, type: tracer)

<read_first>
- `.planning/phases/01-hardware-spike-light-up-the-touch-bar/01-RESEARCH.md` (mode 60x2008, band layout)
- `reference/MTMR/MTMR/defaultPreset.json` (context only)
</read_first>

<action>
Run `rustup default stable`. Create a binary crate `mtmr` at repo root: `cargo init --name mtmr`. Add dependencies in `[dependencies]`: `drm = "0.14"` (or latest), `drm-fourcc = "2"`, `png = "0.17"`, `anyhow = "1"`. Implement `src/pattern.rs::fill_test_pattern` writing 8 equal RGB bands (red, green, blue, yellow, magenta, cyan, white, orange) as columns along the width-2008 axis of a 60-wide buffer in XRGB8888 little-endian byte order (B,G,R,X per pixel), with columns x=0 and x=59 forced white. Implement `--dump-png <path>`: build the 60x2008 RGB buffer, fill it, write PNG via the `png` crate. `cargo build --release` must succeed.
</action>

<acceptance_criteria>
- `Cargo.toml` exists with `name = "mtmr"` and the four dependencies
- `cargo build --release` exits 0
- `./target/release/mtmr --dump-png /tmp/tb.png` exits 0 and `file /tmp/tb.png` reports a 60 x 2008 PNG
</acceptance_criteria>

<verify>
  <automated>cargo build --release 2>&1 | tail -1 && ./target/release/mtmr --dump-png /tmp/tb.png && file /tmp/tb.png | grep -o '60 x 2008'</automated>
  <fails_when>non-zero exit from cargo, missing /tmp/tb.png, or grep finds no "60 x 2008"</fails_when>
</verify>

### Task 2 — DRM modeset to the physical bar (Wave 2)

<read_first>
- `src/main.rs`, `src/pattern.rs` (from Task 1)
- tiny-dfr DRM output reference: https://github.com/AsahiLinux/tiny-dfr (pattern only, no vendoring)
</read_first>

<action>
Implement `src/drm_out.rs::render_to_card`: open `/dev/dri/card0` (arg-overridable), use the `drm` crate Client to get resources; select the connector whose name is "USB-1" (fallback: connector_id 39); resolve its encoder/crtc; pick mode 60x2008@60; create a dumb framebuffer XRGB8888 60x2008, map it, call `fill_test_pattern`, commit a legacy SetCrtc. On success print `OK connector=<id> mode=<name> fb=<id>`. On any failure print a one-line diagnostic naming the stage (open/connector/mode/buffer/modeset) and exit 1 — never panic. `--drm [card]` CLI flag routes to this path. Also run it with the Touch Bar in F-key mode via `hid_appletb_kbd` if needed is NOT required this phase; document whatever mode-switching was needed in docs/hardware.md.
</action>

<acceptance_criteria>
- `src/drm_out.rs` contains a function `render_to_card` taking a card path
- `./target/release/mtmr --drm` exits 0 on this machine and prints a line matching `^OK connector=39`
- `/sys/class/drm/card0-USB-1/enabled` remains `enabled` after run
</acceptance_criteria>

<verify>
  <automated>./target/release/mtmr --drm; echo "exit=$?"</automated>
  <fails_when>exit code non-zero, or stdout lacks a line starting with "OK connector="</fails_when>
</verify>

### Task 3 — Probe touch input + backlight, write docs/hardware.md (Wave 3)

<read_first>
- `/sys/class/drm/card0-USB-1/modes` (60x2008)
- `lsusb` output line for 05ac:8302
</read_first>

<action>
Identify the iBridge multitouch evdev node: iterate `/dev/input/event*` (use `cat /proc/bus/input/devices`), find the device under USB `5-6:2.0` / HID `0003:05AC:8302`; run `evtest --query <node> EV_ABS ABS_MT_POSITION_X` (or read `event*/id` + sysfs `capabilities/abs` and `device/caption`) to record the X-axis max (expect 2008 or 2170). Locate at least one real backlight/backlight-adjacent sysfs path under `/sys/bus/hid/drivers/` matching `hid-appletb`. Write `docs/hardware.md` recording: connector USB-1 / connector_id 39 / mode 60x2008 / card0; the touch event node path and X-axis max; the hid_appletb_bl sysfs path(s); which mode-switch steps (if any) were needed for Task 2's modeset; and the SIGTERM restore note for Phase 2.
</action>

<acceptance_criteria>
- `docs/hardware.md` exists and contains the strings `card0`, `60x2008`, `USB-1`, `39`, `event`, and a `/sys/bus/hid/drivers/` path that exists on disk
- The recorded event node path exists in `/dev/input/`
</acceptance_criteria>

<verify>
  <automated>grep -q '60x2008' docs/hardware.md && grep -q 'USB-1' docs/hardware.md && evtest --query "$(grep -oE '/dev/input/event[0-9]+' docs/hardware.md | head -1)" EV_ABS ABS_MT_POSITION_X 2>&1 | head -2</automated>
  <fails_when>grep finds nothing, the extracted event node path is missing from /dev/input, or evtest cannot read the axis</fails_when>
</verify>

## must_haves

truths:
- "`cargo build --release` on this repo produces `target/release/mtmr` (explicit)"
- "`./target/release/mtmr --dump-png /tmp/tb.png` writes a 60x2008 PNG (explicit)"
- "`./target/release/mtmr --drm` performs a modeset on /dev/dri/card0 connector USB-1 and exits 0 (explicit)"
- "Physical Touch Bar displays the 8 color bands while --drm runs (backstop: requires eyes on hardware)"
  verification: backstop
- "Touch event node identified with ABS_MT_POSITION_X max recorded in docs/hardware.md (backstop: probe output recorded, not rerunnable in CI)"
  verification: backstop
- "docs/hardware.md records card0 / USB-1 / connector_id 39 / mode 60x2008 / event node / hid_appletb_bl path (explicit)"

prohibitions:
- statement: "Never open /dev/dri/card1 (i915 main display) for rendering"
  status: resolved
  verification: "code review: card path defaults to /dev/dri/card0 and no card1 literal in src/"

## Risks

- `drm` crate API version drift vs snippet-level docs — pin exact versions in Cargo.toml; legacy modeset path is stable.
- `appletbdrm` may require the panel already enabled (it is: `enabled` = connected on probe) — if modeset fails, capture stage-level diagnostics before adjusting.
- Rust toolchain download requires network (~200 MB) — first task does it.
