---
phase: "2"
slug: "core-bar-esc-media-layout-touch-actions"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-27"
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test + selftest subcommands (`--selftest-touch`, `--selftest-uinput`) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --release` |
| **Full suite command** | `cargo test --release && target/release/mtmr --selftest-touch && target/release/mtmr --selftest-uinput` |
| **Estimated runtime** | ~20 s |

---

## Sampling Rate

- **After every task commit:** `cargo test --release`
- **After every plan wave:** full suite
- **Before `/gsd-verify-work`:** full suite green + bar live with real items (user feedback)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 2-01-01 | 01 | 1 | PRE-01 | — | N/A | unit | `cargo test --release` (preset parse tests) | ❌ W0 | ⬜ pending |
| 2-01-02 | 01 | 1 | REND-02 | — | N/A | unit+manual | layout tests + user views --live bar | ❌ | ⬜ pending |
| 2-02-01 | 01 | 2 | INPT-01 | — | N/A | unit | `--selftest-touch` exits 0 with correct tap classification | ❌ | ⬜ pending |
| 2-02-02 | 01 | 2 | INPT-01 | — | N/A | manual | tap on bar triggers item action | ❌ | ⬜ pending |
| 2-03-01 | 01 | 3 | INPT-02/03 | — | N/A | integration | `--selftest-uinput` exits 0 after injecting KEY_ESC | ❌ | ⬜ pending |
| 2-04-01 | 01 | 4 | INPT-04 | — | N/A | integration | SIGTERM during --live → exit 0 | ❌ | ⬜ pending |

---

## Wave 0 Requirements

- [ ] Preset parser + layout unit tests before render integration
- [ ] Touch classifier as pure function (testable without hardware)

---

## Manual-Only Verifications

- Visual: bar shows real items (not color bands) — user feedback while --live runs
- Tap esc button → compositor receives ESC (user observes in a terminal)
