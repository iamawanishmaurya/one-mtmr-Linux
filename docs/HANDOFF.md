# HANDOFF — read this first in a new session

> **START NEW SESSIONS FROM `/home/Astra/opencode/touchbar/react-drm-fork/`** —
> that is the primary working folder (the running bar's fork). All work
> happens there: edit → `./deploy.sh` → commit/push. This mtmr folder is only
> the fallback daemon + planning docs.

If a previous AI session ran out of context or ZCode was restarted, everything
needed to continue Touch Bar work is in this repo and the pointers below.

## Where things live (single source of truth)

Everything is consolidated in `/home/Astra/opencode/touchbar/`:

- `react-drm/` — the RUNNING Touch Bar UI (react-drm-for-touchbar + our fixes).
  Service: `react-drm.service` (root; env DBUS_SESSION_BUS_ADDRESS +
  SUDO_USER/SUDO_UID; ExecStartPre runs `mtmr --recover`).
- `react-drm-fork/` — the GitHub fork (iamawanishmaurya/react-drm-for-touchbar).
  **This is the primary working folder**: edit here, run `./deploy.sh` to copy
  to `react-drm/` and restart. Pushed to master through `71317a7`.
- `mtmr/` — Rust fallback daemon (`mtmr.service`, disabled) + all planning docs.
  Reachable via symlink `/home/Astra/opencode/zcode/mtmr-linux`.

## State of the world (as of 2026-09-28)

- react-drm.service is ACTIVE and is the default bar UI at
  `/home/Astra/opencode/react-drm` (compat symlink).
- mtmr.service + old mtmr-linux.service: disabled fallbacks.
- Passwordless sudo is configured for user `Astra`
  (`/etc/sudoers.d/astra-nopasswd`) — sudo works without a password.
- Known recurring issue: the panel freezes on an old frame after
  suspend/resume or many restarts. Fix: `sudo mtmr --recover && sudo
  systemctl restart react-drm`. `react-drm.service` already runs the recover
  as ExecStartPre. Backlight resets to 0 on driver rebind — the daemon forces
  it to max at startup.
- All the fixes and their causes: `react-drm-fork/TROUBLESHOOTING.md` (11
  documented issues) and `mtmr/docs/TROUBLESHOOTING.md` (19 issues).

## Completed work (never redo)

1. mtmr Rust daemon fully built (5 GSD phases, all verified) — see
   `mtmr/.planning/` for PROJECT/REQUIREMENTS/ROADMAP and phase records.
2. react-drm evaluated, fixed, and installed as the default bar.
3. Snake game (app/snake/page.tsx, gamepad button on the main bar) and Camera
   button (niri screenshot) added to the main bar.
4. Per-app capture→reference workflow plan:
   `mtmr/docs/PER-APP-CAPTURE-PLAN.md` (Phase A runs on macOS; B–D on Linux).

## Pending / next ideas

- Run the per-app capture workflow (`docs/PER-APP-CAPTURE-PLAN.md`)
- Default content for the bar's empty left half (media/clock when no app matches)
- Automatic stale-panel watchdog (currently manual recover or service restart)
- User asked for: nothing else pending at handoff time.

## Gotchas an agent must know (do not rediscover these)

- NEVER legacy SetCrtc on appletbdrm — wedges the iBridge until reboot
- Panel only updates on atomic FB_ID flips; DIRTYFB is a no-op
- Match the touch device by NAME ("Touch Bar Display Touchpad"), never eventN
- Keyboard LED is `/sys/class/leds/:white:kbd_backlight` (leading colon)
- No `DeviceAllow=` in units; service must run as root (DRM master)
- Orientation: no length flip; thickness mirrored (`MTMR_VFLIP=1` in unit)
- After moving/killing daemons: `sudo mtmr --recover` before starting another
- Shell cwd resets between commands to the mtmr symlink path — `cd` within
  the same command when touching other folders
