# MTMR-Linux

**My TouchBar. My rules. — rebuilt for Linux.**

## What This Is

A native Linux daemon that replicates [MTMR](https://github.com/Toxblh/MTMR) (macOS "My TouchBar. My rules.") for a T2 MacBook Pro running Linux (t2linux). It renders a fully user-customizable Touch Bar UI — buttons, sliders, widgets, shell/app plugin items defined in a JSON preset — directly to the Touch Bar display (via the `appletbdrm` DRM/fb device) and handles touch input, replacing `tiny-dfr` as the Touch Bar renderer.

**Target hardware (verified on this laptop):** MacBookPro16,2 (2020 13", T2 chip, Arch Linux, kernel `7.2.6-arch2-Watanare-T2`). Touch Bar present as USB devices `05ac:8302` (Display), `05ac:8102` (Backlight), `05ac:8233` (iBridge); kernel modules `appletbdrm`, `hid_appletb_bl`, `hid_appletb_kbd`.

**Why:** MTMR only exists for macOS/Swift/AppKit. The user switched the MacBook to Linux and wants the same JSON-presets-driven, plugin-style Touch Bar experience (esc, media keys, volume/brightness sliders, battery/CPU/clock widgets, shell-script buttons) natively on Linux.

## Core Value

When the user's JSON preset defines the bar, the Touch Bar on their T2 MacBook shows exactly those items, each rendering live data and responding to tap/long-tap — without MTMR, macOS, or Xcode.

## Context

- Reference implementation cloned to `/tmp/MTMR` (to be vendored at `reference/MTMR/`): Swift/AppKit app, ~36 Swift files, JSON preset at `~/Library/Application Support/MTMR/items.json` with types `staticButton`, `appleScriptTitledButton`, `shellScriptTitledButton`, `timeButton`, `battery`, `cpu`, `currency`, `weather`, `music`, `dock`, plus system keys (esc, volume, brightness, illumination) and sliders.
- On Linux, macOS-only concepts map as follows: AppleScript → shell scripts/arbitrary commands; system media keys → uinput virtual keyboard; volume/brightness sliders → ALSA/PipeWire (`amixer`/`wpctl`) and `/sys/class/backlight`; battery/CPU → `/sys/class/power_supply`, `/proc/stat`.
- Touch Bar display appears as a DRM card / framebuffer when `appletbdrm` is bound; touch input arrives via hid-multitouch on the iBridge device. `tiny-dfr` (AsahiLinux) is the reference userspace daemon pattern (owns the DRM output, small, systemd service).
- Preset config lives at `~/.config/mtmr/items.json` on Linux (XDG).

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Daemon renders a JSON-defined bar to the Touch Bar DRM device
- [ ] Esc key, media keys, volume/brightness sliders work via uinput/sysfs
- [ ] Shell-script widgets refresh on an interval like MTMR's `shellScriptTitledButton`
- [ ] MTMR preset compatibility for the common item types

### Out of Scope

- AppleScript plugin type — no macOS; equivalent achieved via shell scripts
- Mac-specific plugins (yandexWeather, dnd, darkMode, nightShift, inputsource) — not meaningful on Linux
- Sparkle auto-update framework — replaced by standard Linux packaging (systemd unit + PKGBUILD)
- Touch Bar on non-T2 (2016/2017) Macs — target hardware is this T2 laptop

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rebuild from scratch in Rust/C for Linux; MTMR repo is reference only | Swift/AppKit doesn't port; MTMR's JSON schema is what we keep | — Pending |
| Render directly to the Touch Bar DRM card (`appletbdrm`), tiny-dfr-style | Same proven approach as tiny-dfr; no X11/Wayland dependency | — Pending |
| Keep MTMR's `items.json` preset schema where sensible | User's presets and muscle memory carry over; presets community exists | — Pending |
| Ship as a systemd user service that conflicts with `tiny-dfr` | Replaces the stock renderer cleanly on boot | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-09-26 after initialization*
