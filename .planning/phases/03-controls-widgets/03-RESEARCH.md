# Phase 3 Research — Controls & Widgets

Probed live on this machine (2026-09-27):
- `playerctl` AND `wpctl` both installed — music/volume via shell-outs are viable.
- Keyboard backlight: `/sys/class/leds/white:kbd_backlight/` max_brightness **14660**.
- Battery: `/sys/class/power_supply/BAT0` (capacity 66, status Charging), AC `ADP1` online.
- CPU: /proc/stat aggregated `cpu ` line (8 fields incl. idle+iowait).
- Screen brightness: `/sys/class/backlight/intel_backlight` max 17777.

## Design
- **Widgets as preset types** (MTMR names): `timeButton` (formatTemplate, chrono strftime-style via hand-rolled mapping of HH/mm/ss), `battery` (capacity% + ⚡/🔌 glyph via text), `cpu` (sample /proc/stat at refreshInterval, % busy = 1-(idle+io)/total), `music` via `playerctl metadata --format '{{artist}} - {{title}}'` every refreshInterval (2s min), play/pause toggle on tap (`playerctl play-pause`).
- **Sliders**: new item shapes MTMR doesn't have (macOS used sliders too): `"type":"brightness"` and `"type":"volume"` render as a filled bar; drag (1 finger, inside rect, after slop) sets value:
  - brightness: write /sys/class/backlight/intel_backlight/brightness (root), 5% floor (old-project lesson).
  - volume: `wpctl set-volume @DEFAULT_AUDIO_SINK@ N%` — as a root system service PipeWire may refuse (XDG_RUNTIME_DIR; old project lesson). Auto-detect the real user's runtime dir from /run/user/* (highest-uid with a pipewire-0 socket) and set XDG_RUNTIME_DIR; if wpctl fails, log once and degrade: vol slider becomes a no-op but keys still work. Full fix is the Phase 5 user-service switch.
- **Keyboard illumination**: `illuminationUp/Down` types step /sys/class/leds/white:kbd_backlight/brightness by max/10.
- Refresh: the live loop re-renders every flip; each widget recomputes cheap state at its refreshInterval (default 2s); shell widgets spawn async to avoid blocking the loop (std thread per refresh, result via channel or simple non-blocking check).

## Validation Architecture
- Widgets: unit tests for cpu% math (synthetic /proc/stat lines), battery string, time format; render-only PNG shows live values.
- Sliders: unit test slider value math (position→percent); hardware drag verified by user.
- illum: sysfs write tested directly (value changes then restored).
