# Project State — MTMR-Linux

**Last updated:** 2026-09-26 (initialization)

## Current Position

- **Milestone:** v1 — daily-usable MTMR replacement on T2 Linux
- **Phase:** 2 of 5 — Core Bar VERIFIED PASSED 2026-09-27 (user confirmed live rendering + taps; MTMR preset compat incl. comments/units/icons). Next: Phase 3 planning.
- **Status:** Phase 1 SUCCESS — test pattern rendered on physical Touch Bar (`OK connector=39 mode=60x2008 fb=41`); touch device event13 mapped (ABS_X 0-32767 → 2008px); docs/hardware.md written. Discovered existing `mtmr-linux.service` daemon holding card0 — future unit must Conflicts with it. Rust crate `mtmr` scaffolded.

## Recent Context

- MTMR (macOS original) cloned for reference at `reference/MTMR/` (also cached at /tmp/MTMR)
- Hardware verified: MacBookPro16,2, Touch Bar USB devices present (05ac:8302/8102/8233)
- Key research: appletbdrm DRM card + tiny-dfr as pattern; Rust stack chosen

## Decisions Log

| Date | Decision |
|------|----------|
| 2026-09-26 | Rebuild in Rust for Linux, MTMR repo is reference only |
| 2026-09-26 | Render directly to appletbdrm DRM card; systemd user service replacing tiny-dfr |
| 2026-09-26 | Keep MTMR items.json schema at ~/.config/mtmr/items.json |

## Next Actions

1. `/gsd-plan-phase 1` — plan the hardware spike
2. During Phase 1: identify DRM card, touch input device, backlight nodes on this machine
