# Roadmap — MTMR-Linux

## Phases

### Phase 1: Hardware Spike — Light Up the Touch Bar
**Goal:** Prove we can render pixels to the physical Touch Bar on this MacBookPro16,2 and receive touch input from it.
**Mode:** mvp
**Success Criteria**:
1. Daemon finds the Touch Bar DRM card (`appletbdrm`) and displays a colored test pattern on the physical bar
2. Touch events from the iBridge device are logged with correct x-coordinates matching the 2170px bar
3. Backlight and keyboard-mode kernel interfaces are identified and documented in `docs/hardware.md`
4. An X11 preview fallback renders the same test pattern when the bar is absent

### Phase 2: Core Bar — Esc, Media, Layout, Touch Actions
**Goal:** A usable minimal bar: JSON preset layout, tap/long-press actions, esc and media keys via uinput.
**Mode:** mvp
**Success Criteria**:
1. Bar renders items from `~/.config/mtmr/items.json` with left/center/right alignment and per-item widths
2. Tapping a staticButton runs its action; long-press runs the secondary action
3. Esc button sends KEY_ESC; previous/play/next buttons send media keycodes visible to `wev`/apps
4. SIGTERM restores Touch Bar keyboard mode and exits cleanly

### Phase 3: Controls & Widgets
**Goal:** Daily-driver controls and live widgets (tiny-dfr parity and beyond).
**Mode:** mvp
**Success Criteria**:
1. Volume and brightness sliders change system volume / screen brightness
2. Keyboard illumination buttons work
3. Time, battery, CPU, and MPRIS music widgets display live-updating values at their configured intervals
4. shellScriptTitledButton renders ANSI-colored script output on its refreshInterval

### Phase 4: MTMR Preset Compatibility
**Goal:** Existing MTMR presets load and behave sensibly.
**Mode:** mvp
**Success Criteria**:
1. The default preset ships and loads; a sample preset from MTMR-presets loads without errors
2. Unknown item types are logged and skipped, remaining items still render
3. MTMR's `defaultPreset.json`-equivalent behavior reproduces (esc, media, volume, brightness, clock, battery)

### Phase 5: Packaging & systemd Integration
**Goal:** Install-and-go on this laptop: replace tiny-dfr cleanly.
**Mode:** mvp
**Success Criteria**:
1. `Conflicts=tiny-dfr.service` user unit starts on login and renders the bar
2. Clear error message when no Touch Bar display is present (exit, not crash-loop)
3. PKGBUILD/README install instructions reproduce a working setup from a clean checkout

## Requirements Traceability

| Requirement | Phase |
|-------------|-------|
| REND-01 | 1 |
| REND-02 | 2 |
| REND-03 | 1 |
| INPT-01 | 2 |
| INPT-02 | 2 |
| INPT-03 | 2 |
| INPT-04 | 2 |
| CTRL-01 | 3 |
| CTRL-02 | 3 |
| CTRL-03 | 3 |
| WIDG-01 | 3 |
| WIDG-02 | 3 |
| WIDG-03 | 3 |
| WIDG-04 | 3 |
| WIDG-05 | 3 |
| PRE-01 | 4 |
| PRE-02 | 4 |
| PRE-03 | 4 |
| PKG-01 | 5 |
| PKG-02 | 5 |

All 20 v1 requirements mapped ✓
