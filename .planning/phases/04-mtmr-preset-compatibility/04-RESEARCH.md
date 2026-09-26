# Phase 4 Research — MTMR Preset Compatibility

Empirical sweep: 12 real community presets from Toxblh/MTMR-presets downloaded and analyzed (2026-09-27).

## Type census (counts across sample)
staticButton 113, group 29, close 29, appleScriptTitledButton 19, timeButton 15,
volumeUp 12, volumeDown 12, dock 12, shellScriptTitledButton 10, battery 10,
previous/play/next/escape 9 each, volume 8 (slider), brightnessUp/Down 8,
brightness 8 (slider), mute 7, network 6, pomodoro 5, exitTouchbar 5, weather 4, music 4.

## Parser blockers found (11 of 12 files fail strict JSON)
1. UTF-8 BOM (preset10)
2. // line comments AND /* */ block comments (11 files; block comments can wrap whole items)
3. Trailing commas before } or ] (preset3)
4. Raw control characters inside strings (unescaped \r bytes in AppleScript inline strings) — most common failure
5. README/text preamble before the opening [ (preset7)
6. group/close legacy syntax: {group} starts a container, subsequent items belong to it until {close} — children must be flattened to render

## Fix plan (lenient preprocessor, single state machine pass)
- strip BOM; locate first '[' if preamble text exists
- strip // and /* */ comments (string-aware)
- drop trailing commas
- escape literal control chars (<0x20) inside strings as \uXXXX
- flatten group...close: keep children, inherit group's align/width only when child lacks one; drop group/close markers
- unknown-but-benign types (dock, network, pomodoro, weather, currency, inputsource, dnd, nightShift, yandexWeather, upnext) → logged skip (already works)
- legacy item-level actions on staticButton (action/actionAppleScript/longAction/longExecutablePath/longShellArguments) → ignore gracefully (title/icon still render)

## Validation Architecture
- Unit tests: each lenient-preprocess rule against real-world fixtures (miniature reproductions)
- `--lint <dir-or-file>`: batch mode; prints per-file "OK n-items rendered / m skipped / FAIL reason"; exit 0 iff all parse
- Acceptance: ≥10 of the 12 sample presets lint OK with ≥1 rendered item; MTMR defaultPreset reproduces esc/brightness/media/volume/battery/time
