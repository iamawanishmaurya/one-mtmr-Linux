# Phase 3 UAT — Controls & Widgets (2026-09-27)

## Automated (all green)
- cargo test: 10 passed (cpu% math, battery string, time formats, slider math incl. edges/clamps)
- Root-gated hardware tests: brightness roundtrip + illum step bound — 2 passed
- --render-only renders full layout with widgets/sliders

## Manual (user, live hardware)
- Orientation settled: esc left, text upright with MTMR_VFLIP=1 (see docs/hardware.md)
- Widgets live on bar (clock/cpu/battery/music): CONFIRMED
- Sliders + illum buttons rendered: CONFIRMED ("good it work")

## Issues found & fixed
1. Right-cluster packed in reverse config order — fixed to config order = visual order (003903e)
2. Right-segment width omitted inter-item gaps → X clipped off bar — fixed (4c865b2)
3. Long track titles spilled into neighbors — in-rect text clipping (e77c1e7)
4. kbd LED path was wrong: actual /sys/class/leds/:white:kbd_backlight (184b754)
5. Panel display session goes stale after many daemon restarts — recovery: appletbdrm unbind/rebind; orientation knobs MTMR_FLIP/VFLIP added (e23fb36, c08f39b)
