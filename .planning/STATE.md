# Project State — MTMR-Linux

**Last updated:** 2026-09-26 (initialization)

## Current Position

- **Milestone:** v1 — daily-usable MTMR replacement on T2 Linux
- **Phase:** 1 of 5 — Hardware Spike (planned, ready for `/gsd-execute-phase 1`)
- **Status:** Phase 1 PLAN.md created 2026-09-26 (research grounded in live hardware probe: card0/USB-1, connector_id 39, mode 60x2008, appletbdrm loaded)

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
