<!-- GSD:project-start source:PROJECT.md -->

## Project

**MTMR-Linux**

A native Linux daemon that replicates [MTMR](https://github.com/Toxblh/MTMR) (macOS "My TouchBar. My rules.") for a T2 MacBook Pro running Linux (t2linux). It renders a fully user-customizable Touch Bar UI — buttons, sliders, widgets, shell/app plugin items defined in a JSON preset — directly to the Touch Bar display (via the `appletbdrm` DRM/fb device) and handles touch input, replacing `tiny-dfr` as the Touch Bar renderer.

**Target hardware (verified on this laptop):** MacBookPro16,2 (2020 13", T2 chip, Arch Linux, kernel `7.2.6-arch2-Watanare-T2`). Touch Bar present as USB devices `05ac:8302` (Display), `05ac:8102` (Backlight), `05ac:8233` (iBridge); kernel modules `appletbdrm`, `hid_appletb_bl`, `hid_appletb_kbd`.

**Why:** MTMR only exists for macOS/Swift/AppKit. The user switched the MacBook to Linux and wants the same JSON-presets-driven, plugin-style Touch Bar experience (esc, media keys, volume/brightness sliders, battery/CPU/clock widgets, shell-script buttons) natively on Linux.

**Core Value:** When the user's JSON preset defines the bar, the Touch Bar on their T2 MacBook shows exactly those items, each rendering live data and responding to tap/long-tap — without MTMR, macOS, or Xcode.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->

## Technology Stack

## Touch Bar hardware path on T2 Linux (verified on this machine)

- USB: `05ac:8302` Touch Bar Display, `05ac:8102` Backlight, `05ac:8233` iBridge.
- Kernel drivers (t2linux kernels / upstreaming effort): `appletbdrm` (DRM tiny driver exposing the 2170×60 Touch Bar panel as a DRM card/fbdev), `hid_appletb_bl` (backlight), `hid_appletb_kbd` (keyboard mode switch). Linux 6.17+ mainline enabled x86 MacBook Pro Touch Bars.
- Touch input: hid-multitouch events on the iBridge HID device.
- Reference userspace daemon: **tiny-dfr** (AsahiLinux/tiny-dfr) — Rust, grabs the correct DRM device, renders, and is what the t2linux wiki recommends. Our daemon must claim the same device (systemd conflict or device handshake needed).

## Language choice

- **Rust (recommended)**: tiny-dfr precedent, DRM direct-rendering crates (`drm`, `drm-fourcc`), no runtime deps, systemd-friendly. Touch input via `evdev` crate.
- Alternative: C (libdrm + libevdev) or Python (slow for 60fps-ish refresh; fine for widget refresh only).
- GTK/Qt window approaches are wrong — the bar is a separate panel display, not a window in the compositor.

## Supporting stack

- uinput (Rust `enigo` or raw `uinput` crate) to inject Esc/media/volume keys.
- Audio: `wpctl` (PipeWire) or ALSA mixer; brightness: `/sys/class/backlight` (kernel `apple-ib-bl`).
- Battery/CPU widgets: `/sys/class/power_supply/BAT*`, `/proc/stat`.
- Config: JSON via `serde_json` — mirror MTMR's `items.json` schema.
- Packaging: systemd user service + Arch PKGBUILD.

## Sources

- https://wiki.t2linux.org/guides/postinstall
- https://github.com/AsahiLinux/tiny-dfr
- https://lwn.net/Articles/986150
- https://github.com/Dunedan/mbp-2016-linux (non-T2 reference)

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
