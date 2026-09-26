# Per-App Touch Bar Capture → Reference Workflow — Implementation Plan

**Purpose:** record what macOS shows on the Touch Bar (DFR display) for **every
app**, then use those recordings on **Linux** as visual references to implement
matching Touch Bar layouts. The two sides are fully decoupled:

```
PHASE A (macOS, one-time bulk pass)          PHASE B/C/D (Linux, ongoing)
──────────────────────────────────           ──────────────────────────────
auto-capture DFR per app ──sync──▶           analyze captures ──▶ implement
while the user simply uses their             per-app pages/pages in react-drm,
Mac normally                                 verify, iterate
```

**Target machine (Linux side, all facts verified):** MacBookPro16,2 (T2),
Arch Linux + niri. Touch Bar stack installed:
- `react-drm-for-touchbar` (fork: iamawanishmaurya/react-drm-for-touchbar) at
  `/home/Astra/react-drm`, running as `react-drm.service` (root, env:
  `DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`,
  `XDG_RUNTIME_DIR=/run/user/1000`, `SUDO_USER=Astra`, `SUDO_UID=1000`,
  `ExecStartPre=/usr/bin/mtmr --recover`)
- Fallback daemon: `mtmr.service` (Rust, this repo) — disabled, kept
- Panel: 2170×60 physical, DRM mode **60×2008** (portrait). Render landscape
  2008×60 and mirror the thickness axis. NEVER use legacy SetCrtc on
  `appletbdrm` (wedges the panel until reboot)
- Touch input: match device by NAME ("Apple Inc. Touch Bar Display Touchpad")
  — event numbers change across boots. ABS_X 0–32767 → 2008 px

---

## PHASE A — Bulk capture on macOS (no Linux involvement)

Goal: walk the user's normal Mac usage and end up with **one or more DFR
screenshots per app**, saved by bundle ID. The user should not have to run
anything per app — the capture tool watches foreground changes automatically.

### A1. Build the auto-capture tool (`dfr-autocap`)

A small macOS command-line daemon (Swift or Python + pyobjc; ~200 lines):

1. Watch the frontmost app via `NSWorkspace.sharedWorkspace` notifications
   (`NSWorkspaceDidActivateApplicationNotification`).
2. On every frontmost change:
   - wait ~1.5 s (bars animate in; capture after settle)
   - capture the DFR display only:
     - preferred: `CGGetDisplaysWithRect` for the 2170×60 display and
       `CGDisplayCreateImage` on it; or
     - shell out to `screencapture -x -o <index-or-file>` targeting the DFR
       display (find index via `CGGetActiveDisplayList` / smallest-height
       display = 60 px)
   - also capture once with **Fn held** (F1–F12 row) per app, saved with an
     `-fn` suffix — macOS shows the function row layer when Fn is pressed
   - debounce: don't re-capture the same app within 10 s; save multiple
     variants (`com.apple.Safari_001.png`, `_002.png`…) when the bar content
     changes (compare hashes of consecutive captures while the app stays
     frontmost — subpages/panels differ)
3. Save to `~/touchbar-captures/<bundle-id>/<bundle-id>_<n>.png` plus a
   `manifest.json` mapping bundle ID → human app name → capture paths.
4. Requirements: run inside the user's GUI session (needs WindowServer
   access — a LaunchAgent, not a system daemon; screen-recording permission
   must be granted in System Settings → Privacy → Screen Recording).

### A2. Capture pass (user's only manual job)

- Install the LaunchAgent, reboot/login
- Use the Mac normally for a day or two — every app that shows a Touch Bar
  gets captured automatically in the background
- Optionally run the Fn-row capture for each app

### A3. Sync to Linux

- Bundle `~/touchbar-captures/` into the repo folder
  `~/touchbar-refs/` (git-ignored if the repo is public — see Legal note)
  via any transfer (USB, git LFS, Syncthing). Format is plain PNG + JSON.

**Acceptance (Phase A):** ≥90 % of the user's regularly-used apps have ≥1
valid 2170×60 capture; `manifest.json` complete; nothing captured from
wrong displays (sanity: image height == 60).

---

## PHASE B — Analyze on Linux

For each app capture (batch job over `~/touchbar-refs/`):

1. Inspect the strip: identify items left→right, groups, widths (macOS units:
   full bar = 1080 units; px × 1080 / 2170), icons vs text
2. Map every macOS action to a Linux equivalent — or mark `skip`:

| macOS action | Linux equivalent |
|---|---|
| web back/forward/reload/home | browser keys (niri-focused browser or Ctrl+[\` keycodes via uinput) |
| play/pause/next/previous | `playerctl play-pause/next/previous` |
| volume/brightness slider | react-drm `audio-slider` / `brightness-slider` layers |
| new tab / close tab | browser key hooks (`useBrowserKeys`) |
| Siri / dictation / DFR-only items | `skip` (no Linux equivalent) |
| app launchers | `launch('command', args)` (drops to user session) |
| system sleep | `systemctl suspend` |

3. Write `~/touchbar-refs/<app-id>.analysis.md` with the mapping table
4. Get user sign-off per app before implementing (cheap: show the capture +
   the proposed table)

---

## PHASE C — Implement per-app pages on Linux

For each signed-off analysis:

1. Create `app/splitted/<app>/page.tsx` in
   `~/react-drm/linux-touchbar-control-center/` — copy the structure of
   `app/splitted/browser/page.tsx` (grouped `<Btn>` components,
   `radiusLeft/right`, `BTN_W` widths taken from the analysis)
2. Wire each control to the Linux action from the analysis table
   (media → playerctl, sliders → existing `audio-slider` /
   `brightness-slider` layers, launches → `launch()` from
   `@/lib/services/launch` which drops to the user session)
3. Register the window-class → page routing in
   `resolveLeftSideLayerByClass(activeClass)` using the Linux app's
   `matchClass` values (Firefox: `firefox`; Dolphin: `dolphin`; …)
4. Icon policy: Material Design SVG paths or react-icons; if the capture
   shows an icon worth copying exactly, trace it as a new SVG path (do not
   copy Apple pixel art into the repo — see Legal note)

**Hard constraints (a previous session verified these the hard way):**
- Never legacy SetCrtc on `appletbdrm` — wedges the panel until reboot
- After killing any Touch Bar daemon, reset the display session before
  starting another (driver unbind/bind of `5-6:2.1` — `mtmr --recover` does
  this; react-drm.service already runs it as ExecStartPre)
- Panel orientation: no length flip (esc at x=0 = physical left); thickness
  axis mirrored (VFLIP)
- `brightnessctl` must be installed or brightness sliders silently no-op
- Root-run service needs `DBUS_SESSION_BUS_ADDRESS`/`XDG_RUNTIME_DIR` or the
  media widget crash-loops the layout

---

## PHASE D — Verify (Linux)

1. `sudo systemctl restart react-drm`; confirm
   `journalctl -u react-drm.service` shows `DRM display ready` without error
   storms
2. Focus each implemented app → bar switches automatically to its page
3. Side-by-side: open the macOS capture next to the live bar; compare
   structure (not pixels — actions differ by design)
4. Tap every control; confirm the Linux action fires
5. Regression check on the default page: volume/brightness sliders, media
   keys, screenshot button still work

---

## Legal note

Captured DFR images are for **personal reference only** — Apple system art
and app UIs are copyrighted; do not push captures or pixel-derived artwork to
a public repository. Re-implemented layouts built from scratch with
MIT/Apache-licensed icons are fine.

## Out of scope / future ideas

- Static pixel replay mode: display a captured PNG as the bar with a manual
  touch hit-map (pixel-perfect look, semi-interactive) — possible mtmr feature
- Automatic layout inference from captures (vision model) — manual analysis
  is currently more reliable
- Non-T2 Macs (2016/2017 `appletb` SPI panel) — different driver, untested

## Acceptance criteria (whole workflow)

- [ ] `dfr-autocap` runs as a LaunchAgent on macOS, capturing per-app DFR
      strips automatically during normal use
- [ ] `~/touchbar-refs/` on Linux contains ≥10 apps with captures + analysis
      files
- [ ] ≥3 apps implemented as react-drm pages with automatic focus switching
- [ ] Every control on each page fires a working Linux action
- [ ] No regressions: sliders, media keys, screenshot button on the default
      page still work
