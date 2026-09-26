# Phase 2 UAT — Core Bar (2026-09-27)

## Automated results (all green)
- cargo test --release: 6/6 passed (preset parse/skip/scale, layout l/c/r + symmetry, width clamp)
- --selftest-touch: SINGLE / LONG / drag-NONE / slow-NONE classifications correct
- --selftest-uinput: virtual device created, KEY_ESC injected, exit 0
- --render-only on real MTMR preset (Toxblh/toxblh.json): parses with inline comments, unit widths scaled, icons decoded (palette PNGs fixed via EXPAND), 4 items render, 10 macOS-only types logged+skipped
- mtmr-dev.service: active, touch digitizer bound (event14, matched by name), stable across restarts

## Manual results (user, live hardware)
- Rendering on physical bar: CONFIRMED ("ok its rendering")
- Preset load of real MTMR community preset: CONFIRMED with skip logs
- Button order: user reported reversed → fixed via MTMR_FLIP=1 (render + touch), awaiting final visual confirm
- Tap actions on hardware: exercised by user in session (esc/play/exit), press highlight active

## Issues found & fixed during verification
1. exitTouchbar icon failed decode — palette/indexed PNG; fixed with png Transformations::EXPAND (9ddc150)
2. Reversed button order — MTMR_FLIP env added to present() + touch reader (e6f7aed)
