---
phase: "3"
slug: controls-widgets
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-27"
---
# Phase 3 — Validation Strategy
| Property | Value |
|---|---|
| Framework | cargo test + live service |
| Quick run | `cargo test --release` |
| Full suite | `cargo test --release && target/release/mtmr --render-only /etc/mtmr/items.json` |
## Per-Task Verification Map
| Task ID | Requirement | Command | Status |
|---|---|---|---|
| 3-01-01 | WIDG-01..04 | `cargo test --release` widget math tests | ⬜ |
| 3-01-02 | WIDG-01..04 | `--render-only` shows live values; user confirms bar | ⬜ |
| 3-02-01 | CTRL-01/02 | slider math unit tests; user drags on hardware | ⬜ |
| 3-02-02 | CTRL-03 | illum sysfs step test; user confirms | ⬜ |
## Manual-only
- Drag feel on physical bar; volume slider vs PipeWire perms.
