# v1 Requirements — MTMR-Linux

## v1 Requirements

### Rendering Core
- [x] **REND-01**: Daemon detects the Touch Bar DRM card at runtime and displays a test bar on the physical Touch Bar display
- [x] **REND-02**: Daemon renders items from `~/.config/mtmr/items.json` laid out left/center/right with per-item widths
- [x] **REND-03**: Daemon shows an X11 preview window when the Touch Bar DRM card is unavailable (dev/CI fallback)

### Input & System Keys
- [x] **INPT-01**: Tapping an item triggers its action; long-press triggers its secondary action
- [x] **INPT-02**: Escape button sends KEY_ESC via uinput
- [x] **INPT-03**: Media buttons (previous/play/next) send standard media keycodes via uinput
- [x] **INPT-04**: Daemon restores Touch Bar keyboard mode and releases devices cleanly on SIGTERM

### Controls
- [x] **CTRL-01**: Volume slider adjusts system volume via wpctl/PipeWire and works as up/down keys
- [x] **CTRL-02**: Brightness slider adjusts screen brightness via /sys/class/backlight
- [x] **CTRL-03**: Keyboard illumination up/down adjusts the keyboard backlight

### Widgets
- [x] **WIDG-01**: timeButton shows configurable-format clock, refreshed each second/minute, with long-press calendar action
- [x] **WIDG-02**: battery widget shows charge percentage and charging state from /sys/class/power_supply
- [x] **WIDG-03**: cpu widget shows CPU load sampled from /proc/stat at a configurable interval
- [x] **WIDG-04**: music widget shows current track and play/pause state via MPRIS (D-Bus)
- [x] **WIDG-05**: shellScriptTitledButton runs a shell script on refreshInterval and displays its (optionally ANSI-colored) output

### Preset & Config
- [x] **PRE-01**: Preset loads from `~/.config/mtmr/items.json` in MTMR-compatible schema (type, title, width, align, refreshInterval, actions)
- [x] **PRE-02**: Unknown item types are logged and skipped without failing the whole preset
- [x] **PRE-03**: A sensible default preset (esc, media, volume, brightness, clock, battery) ships with the project

### Packaging & Ops
- [ ] **PKG-01**: Installs as a systemd user service with `Conflicts=tiny-dfr.service`
- [ ] **PKG-02**: Daemon exits with a clear error message when no Touch Bar display is found

## v2 Requirements (deferred)

- currency / weather / network / upnext(calendar) widgets
- dock app-switcher widget
- Pomodoro widget
- Multi-finger swipe gestures
- Preset hot-reload
- GUI preset editor (MTMR Designer parity)

## Out of Scope

- AppleScript plugins — no macOS; shell scripts are the equivalent
- Mac-specific plugins (yandexWeather, dnd, darkMode, nightShift, inputsource)
- Sparkle auto-update framework — Linux packaging instead
- Non-T2 (2016/2017) MacBook Touch Bar support

## Traceability (filled by roadmap)

| Requirement | Phase |
|-------------|-------|
| REND-01 | 1 |
| REND-02 | 2 |
| INPT-01 | 2 |
| INPT-02 | 2 |
| INPT-03 | 2 |
| INPT-04 | 2 |
| PRE-01 | 2 |
| PRE-02 | 2 |
| PRE-03 | 2 |
| REND-03 | 1 |
| REND-03 | 1 |

---
*Created: 2026-09-26*
