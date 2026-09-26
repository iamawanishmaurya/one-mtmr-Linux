# Features Research — MTMR-Linux

Feature parity target: MTMR's built-in button types, mapped to Linux equivalents.

## Table stakes (users expect from MTMR)

| MTMR feature | Linux equivalent |
|---|---|
| `escape` key button | uinput KEY_ESC |
| `previous` / `play` / `next` media keys | uinput KEY_PREVIOUSSONG/PLAYPAUSE/NEXTSONG |
| `volumeUp/Down`, `mute` | wpctl/amixer + uinput, slider variant |
| `brightnessUp/Down`, sliders | `/sys/class/backlight` |
| `illuminationUp/Down` (kbd backlight) | hid_appletb_bl sysfs |
| `staticButton` with custom action | run shell command |
| `shellScriptTitledButton` | spawn shell script, refreshInterval, ANSI colors |
| `timeButton` (clock, long-tap calendar) | chrono/strftime |
| `battery` | /sys/class/power_supply (level, charging, time estimate) |
| `cpu` | /proc/stat sampling |
| `music` (now playing) | MPRIS over D-Bus (playerctl equivalent) |
| `exitTouchbar` | restore keyboard-mode/exit daemon |

## Differentiators (v2 candidates)

- `currency`, `weather` widgets (HTTP APIs)
- `dock` (running-app switcher) — via D-Bus/compositor APIs
- Pomodoro timer, network status, calendar "up next"
- Volume/brightness finger-gesture sliding (touch input drag)
- Preset hot-reload on file change
- GUI preset editor (MTMR Designer parity)

## Research notes

- MTMR presets community: github.com/Toxblh/MTMR-presets — compatibility with this schema maximizes reuse.
- ANSI-color shell output parity is cheap to implement and popular in MTMR presets.
- Must-have for daily usability: esc + media + volume — these are what tiny-dfr provides today; we must at least match it before users switch.
