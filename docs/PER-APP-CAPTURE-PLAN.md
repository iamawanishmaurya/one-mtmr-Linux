# Per-App Touch Bar Capture → Reference Workflow — Implementation Plan

**Purpose:** a self-contained plan an AI (or developer) can execute to build a
workflow that records what macOS apps show on the Touch Bar (DFR display) and
uses those recordings as visual references to implement matching Touch Bar
layouts on Linux.

**Target machine (all paths/hardware facts verified):** MacBookPro16,2 (T2),
Arch Linux + niri compositor. Linux Touch Bar stack already installed:
- `mtmr` daemon (Rust, this repo) — reference for hardware handling, `--recover`
- `react-drm-for-touchbar` (fork: iamawanishmaurya/react-drm-for-touchbar)
  installed at `/home/Astra/react-drm` — currently the default bar, runs as
  `react-drm.service` (root, with `DBUS_SESSION_BUS_ADDRESS` +
  `SUDO_USER=Astra`/`SUDO_UID=1000` env; `ExecStartPre=/usr/bin/mtmr --recover`)
- Kernel: `appletbdrm` (Touch Bar display, `/dev/dri/card0` today — the minor
  number CHANGES after driver rebinds), `hid_appletb_bl`, `hid_appletb_kbd`
- Touch input: `/proc/bus/input/devices` → "Apple Inc. Touch Bar Display
  Touchpad" (match by NAME; the eventNN number changes). ABS_X 0–32767 → 2008 px
- Panel: 2170×60 physical, DRM mode **60×2008** (portrait). Render landscape
  2008×60 and mirror the thickness axis (VFLIP). Never use legacy SetCrtc.

---

## Goal

Build a repeatable workflow:

```
macOS (boot or Mac)  ──capture──▶  per-app DFR screenshots (PNG)
        │
        ▼ (back on Linux)
analyze layout  ──▶  implement matching Touch Bar page (react-drm layer or
                      mtmr preset) with Linux-native actions
        │
        ▼
verify side-by-side: captured reference vs live bar render
```

Deliverable: a **reference library** (`~/touchbar-refs/<app-id>.png` per app),
a **capture guide** the user follows on macOS, and **per-app Touch Bar pages**
in react-drm for each captured app, visually matching the references.

## Why capture at all

macOS apps render their own Touch Bar UI in compiled code that does not exist
on Linux. Pixel captures are the only faithful record of "what the real bar
looks like per app" — they serve as the visual spec to re-implement with
Linux-native controls. (Interactive behavior can't be replayed; it must be
rebuilt. See "Legal note".)

---

## Phase A — Capture on macOS

**Prerequisite:** boot the MacBook into macOS (this machine dual-boots; keep
the Linux install untouched). macOS version ≥ 10.14. Touch Bar works natively.

1. Install a DFR capture tool. Two options, pick one:
   - **TouchBarScreenshotter** (GitHub, MIT): captures the Touch Bar display
     only, one PNG per invocation. Recommended.
   - `screencapture -x` with the DFR display index (find it via
     `system_profiler SPDisplaysDataType` — it appears as a 2170×60 display).
2. Create the capture script `capture-touchbar.sh` (macOS side):

```sh
#!/bin/zsh
APP_ID="$1"   # e.g. "com.apple.Safari"
OUT="$HOME/touchbar-captures"
mkdir -p "$OUT"
# Bring the target app to front, wait, capture the DFR display
osascript -e "tell application id \"$APP_ID\" to activate"
sleep 1.5
screencapture -x -o "$OUT/${APP_ID//:/_}.png"
# NOTE: -x disables sound; add the DFR display index if screencapture
# grabs the wrong display (see tool docs).
echo "saved $OUT/${APP_ID//:/_}.png"
```

3. Capture at least these apps (the ones the user runs on Linux too):
   `com.apple.Safari`, `com.apple.finder`, `com.apple.iTunes` or Music,
   `com.apple.Terminal`, plus any app the user requests. Also capture the
   **Fn row** (hold Fn, capture) and the **default desktop bar**.
4. Copy all PNGs to the Linux side: `~/touchbar-captures/`.

**Quality rules:** capture at native resolution (2170×60); do not scale or
crop; keep the file named by macOS bundle ID.

## Phase B — Analyze on Linux

1. For each capture, produce a layout description:
   - Which buttons/groups appear, their order and approximate widths
     (widths in macOS "units": full bar = 1080 units; measure px×1080/2170)
   - What each control likely maps to on Linux (e.g. Safari reload →
     `niri msg`-focused browser reload keys; Siri → drop)
   - Which parts are impossible (Apple-only: Siri, dictation) — mark `skip`
2. Write the result as a table into
   `~/touchbar-captures/<app-id>.analysis.md`, e.g.:

| # | item | units | macOS action | Linux action |
|---|------|-------|--------------|--------------|
| 1 | back | 64 | web back | browser keys hook |

3. Get user sign-off on each analysis before implementing.

## Phase C — Implement per-app pages (react-drm)

The control center already routes per focused window:

- `app/splitted/layout.tsx` resolves the left layer from the active window
  class (`resolveLeftSideLayerByClass(activeClass)`), per-app page components
  live in `app/splitted/<app>/page.tsx` (see existing `browser/`, `dolphin/`)
- Add a new page per analyzed app: copy the structure of
  `app/splitted/browser/page.tsx` (groups of `<Btn>` with `radiusLeft/right`),
  wire actions to Linux equivalents:
  - media: `playerctl play-pause/next/previous`
  - volume/brightness: existing slider layers (`go('audio-slider')`,
    `go('brightness-slider')`)
  - app launch: `launch('command', args)` from `@/lib/services/launch`
    (drops into the user session automatically)
  - screenshots: `niri msg action screenshot` with NIRI_SOCKET auto-detect
    (see repo commit history for the working pattern)
- Register the class → page mapping in the routes resolution
  (`resolveLeftSideLayerByClass`) with the macOS app's likely Linux
  `matchClass` values (e.g. Firefox: `firefox`, `navigator`)

**Design constraints (hard-won — do not violate):**
- Render surface is landscape 2008×60; icons 32–40 px; use Material Design SVG
  paths via the existing `src/icons.rs`-style approach, or react-icons in the
  control center. Text via Noto Sans.
- The physical panel applies NO length flip: esc at x=0 renders on the LEFT.
- Never use legacy SetCrtc on `appletbdrm` (wedges the panel; only a reboot
  recovers). react-drm's own atomic/Cairo path is safe.
- After killing any Touch Bar daemon, reset the display session before
  starting another:
  `echo "5-6:2.1" | sudo tee /sys/bus/usb/drivers/appletbdrm/unbind && sleep 1
   && echo "5-6:2.1" | sudo tee /sys/bus/usb/drivers/appletbdrm/bind`

## Phase D — Verification

1. Build + restart: `sudo systemctl restart react-drm`
2. Focus each target app on the main display; confirm the bar switches pages
3. Side-by-side check: open `~/touchbar-captures/<app>.png` next to the live
   bar; compare structure (not pixel-exact — Linux actions differ)
4. Tap every implemented control and confirm its Linux action fires
5. Check `journalctl -u react-drm.service` for error storms

## Legal note

Captures are for **personal reference only**. Apple system assets and app
UIs are copyrighted — do not commit captured DFR PNGs or pixel-derived
artwork to a public repo. Re-implemented layouts built from scratch (with
MIT/Apache-licensed icons) are fine.

## Out of scope / future

- Static pixel replay mode (show a captured PNG as the bar with hit-map
  overlays) — possible follow-up in mtmr if "pixel-perfect look" is required
- Automatic layout inference from screenshots (vision model) — currently
  manual analysis is more reliable
- Non-T2 Macs (2016/2017 `appletb` SPI panel) — different driver, untested

## Acceptance criteria (whole workflow)

- [ ] ≥3 apps captured, analyzed, and implemented as react-drm pages
- [ ] Focusing each app on Linux switches the bar to its page automatically
- [ ] Every control on each page fires a working Linux action
- [ ] Side-by-side visual comparison documented in `~/touchbar-captures/`
- [ ] No regressions: volume/brightness sliders, media keys, and the
      screenshot button still work on the default page
