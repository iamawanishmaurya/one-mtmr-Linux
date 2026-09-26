---
phase: "1"
slug: "hardware-spike-light-up-the-touch-bar"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-26"
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust std test harness) + `cargo build` gate |
| **Config file** | `Cargo.toml` (created in Wave 0) |
| **Quick run command** | `cargo build --release` |
| **Full suite command** | `cargo test --release && ./target/release/mtmr-spike --dump-png /tmp/tb.png && file /tmp/tb.png` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo build --release`
- **After every plan wave:** Run full suite command
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 1-01-01 | 01 | 0 | REND-03 | — | N/A | build | `cargo build --release` | ❌ W0 | ⬜ pending |
| 1-01-02 | 01 | 1 | REND-03 | — | N/A | integration | `target/release/mtmr-spike --dump-png /tmp/tb.png` then `file /tmp/tb.png` contains "60 x 2008" | ❌ W0 | ⬜ pending |
| 1-02-01 | 01 | 2 | REND-01 | — | N/A | hardware | `target/release/mtmr-spike --drm /dev/dri/card0` exits 0 and connector USB-1 shows enabled | ❌ | ⬜ pending |
| 1-02-02 | 01 | 2 | REND-01 | — | N/A | manual | color bands visible on physical Touch Bar | ❌ | ⬜ pending |
| 1-03-01 | 01 | 3 | REND-01 | — | N/A | manual-probe | `evtest` on recorded event node emits ABS_MT_POSITION_X with max 2008/2170 | ❌ | ⬜ pending |
| 1-03-02 | 01 | 3 | REND-01 | — | N/A | source | `docs/hardware.md` contains connector id 39, mode 60x2008, and a real hid_appletb_bl sysfs path | ✅ W0 after task | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `Cargo.toml` + `src/main.rs` scaffold with `--dump-png` and `--drm` subcommands stubbed
- [ ] PNG dump path testable without hardware

---

## Manual-Only Verifications

- Physical Touch Bar shows the 8 color bands (SC1) — requires eyes on hardware.
- Touch coordinate max-value reading from evtest (SC2) — one-time probe, values recorded into `docs/hardware.md`.
