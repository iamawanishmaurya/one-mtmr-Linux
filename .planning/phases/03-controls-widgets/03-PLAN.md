---
phase: 3
plan: 01
wave: 1
depends_on: []
files_modified:
  - src/widgets.rs
  - src/slider.rs
  - src/preset.rs
  - src/render.rs
  - src/main.rs
  - assets/example-items.json
autonomous: false
requirements: [CTRL-01, CTRL-02, CTRL-03, WIDG-01, WIDG-02, WIDG-03, WIDG-04, WIDG-05]
---
# Plan 01 — Controls & Widgets

**Goal:** Live widgets (clock, battery, CPU, MPRIS music) and draggable brightness/volume sliders on the bar; keyboard illumination stepping.

## Artifacts this phase produces
- `src/widgets.rs` — `tick_widget(kind, state) -> String` for timeButton/battery/cpu/music (+ `/proc/stat` cpu delta math, battery glyph, playerctl shell-out)
- `src/slider.rs` — slider item state (0..100), `pos_to_pct`, sysfs brightness writer with 5% floor, wpctl volume setter with /run/user auto-detect, kbd-illum stepper
- preset: new known types `timeButton` (formatTemplate), `battery`, `cpu`, `music`, `brightness` (slider), `volume` (slider), `illuminationUp/Down` actions
- render: sliders drawn as track + fill; widgets drawn as text
- live loop: per-item refreshInterval re-evaluation before each flip

## Tasks
### Task 1 — Widgets (Wave 1)
<read_first>src/preset.rs src/render.rs src/main.rs docs/hardware.md</read_first>
<action>
Add chrono. Implement widgets.rs: time string from formatTemplate (HH/mm/ss tokens), battery from /sys/class/power_supply/BAT0 (capacity, status), cpu% from /proc/stat delta (idle+iowait), music via `playerctl metadata --format "{{artist}} - {{title}}"` (empty if none). In load(), accept types timeButton/battery/cpu/music (known). Live loop: before each flip (every 1s now), recompute widget titles at their refreshInterval (default 2s) and redraw. Static action wiring: music tap → playerctl play-pause via command action in default config.
</action>
<acceptance_criteria>
- cargo test: cpu% math on synthetic samples (0%, 100%, 33% cases), battery string, HH:mm formatting
- --render-only shows live clock/battery/cpu values in PNG
- Physical bar shows live-updating widgets (user)
</acceptance_criteria>
<verify><automated>cargo test --release 2>&1 | tail -2</automated><fails_when>test failures</fails_when></verify>

### Task 2 — Sliders + illumination (Wave 2)
<read_first>src/touch.rs src/main.rs docs/hardware.md</read_first>
<action>
slider.rs: brightness write with floor (max 17777, floor 5%), volume via wpctl with XDG_RUNTIME_DIR auto-detect (scan /run/user/* for pipewire-0 socket; on failure log once), illum step ±max/10. preset: slider types brightness/volume (width default 200 units), render as track (gray outline) + fill (white, pct). Drag handling in live loop: on Move inside slider rect while finger down and past slop → set pct from x, throttle writes to ≥150ms apart, redraw immediately. illuminationUp/Down items: builtin step action (extends builtin_keycode path to sysfs).
</action>
<acceptance_criteria>
- unit tests: pos_to_pct edges/mid; brightness floor honored (5%); illum step math
- user drags brightness slider → screen brightness changes; volume slider → volume changes (or logged degradation)
- illumination buttons step keyboard backlight (user)
</acceptance_criteria>
<verify><automated>cargo test --release 2>&1 | tail -2</automated><fails_when>test failures</fails_when></verify>
