# Pitfalls Research — MTMR-Linux

## Hardware/driver pitfalls

1. **`appletbdrm` binding varies by kernel.** Known breakage reports on some kernel versions (e.g. t2linux wiki issue #635 on 6.12.31+). Verify which DRM card exists at runtime; fail with a clear message, don't crash-loop.
2. **tiny-dfr conflict.** t2linux installs/enables tiny-dfr by default. If both run, last modeset wins → flicker. Ship `Conflicts=` and document `systemctl disable --now tiny-dfr`.
3. **Kernel mode / keyboard switching.** The Touch Bar switches between display mode and F-key row via `hid_appletb_kbd`. Wrong assumptions here can leave the user with a dead Fn row if our daemon exits uncleanly — restore state on SIGTERM.
4. **Backlight quirks** — `hid_appletb_bl` node names differ per kernel; detect, don't hardcode.
5. **Touch input mapping** — iBridge multitouch events may need scaling to the 2170-px-wide display; test calibration early (Phase 1 spike).

## Project pitfalls

6. **Scope creep in widgets** — MTMR has ~20 plugin types; ship the top 8 first (esc, media, volume, brightness, shell, time, battery, music) and defer the rest.
7. **Rendering perf** — full 2170×60 redraw per widget tick is cheap (130 KB); don't over-engineer GPU paths.
8. **Preset schema drift** — don't silently drop unknown MTMR item types; log and skip so presets still load.
9. **Permissions** — systemd user service needs uinput/DRM ACLs (uaccess tags or input/video group); most common "doesn't work" report source.
10. **No CI hardware** — keep the X11/SDL preview fallback so every phase is verifiable without the physical bar.
