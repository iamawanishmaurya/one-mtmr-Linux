# Phase 5 Research — Packaging & systemd Integration

Probed (2026-09-27):
- User Astra groups: power, greeter, storage, wheel — NOT in video/input.
- /dev/dri/card0 = root:video 660 → user needs `video` group.
- /dev/input/event* = root:input 660 → user needs `input` group.
- /dev/uinput = root:root 660 (has ACL +) → needs udev rule (TAG+="uaccess" or input group + mode).
- Brightness/LED sysfs writes need group-write udev rules for `video`.

## Design
- **systemd user unit** `mtmr.service` (installed to /usr/lib/systemd/user/): runs `mtmr --live` as the logged-in user; `Conflicts=tiny-dfr.service mtmr-linux.service` (system units also stopped via docs); Wants=graphical-session.target, PartOf=graphical-session.target.
- **udev rules** `60-mtmr.rules`:
  - KERNEL=="uinput", MODE="0660", GROUP="input", TAG+="uaccess"
  - SUBSYSTEM=="backlight", RUN+="/bin/sh -c 'chgrp video /sys/class/backlight/*/brightness && chmod g+w ...'"
  - SUBSYSTEM=="leds", RUN+ similar for kbd backlight.
- Install user into video+input groups (setup step in README + PKGBUILD install script note).
- Volume slider: as user, wpctl works natively (XDG_RUNTIME_DIR present) — slider.rs already handles both.
- Preset resolution gains /usr/share/mtmr/items.json as system fallback (below user config).
- **Stale-panel recovery**: `mtmr --recover` (root, run manually or via ExecStartPre of a oneshot system helper): rebinds appletbdrm (unbind/bind) to re-init the display session when the bar freezes on an old frame. Documented in README troubleshooting.
- **PKGBUILD**: makedepends rust; package: binary, user unit, udev rules, /usr/share/mtmr/items.json default, README.
- Old artifacts to remove: /etc/systemd/system/mtmr-dev.service (disabled+removed), /etc/mtmr/items.json migrated to ~/.config/mtmr/items.json.

## Validation Architecture
- Automated: cargo test; lint; `systemctl --user is-active`; wpctl get-volume roundtrip from user context.
- Manual: bar renders after `loginctl enable-linger`-less login; reboot persistence; volume slider drag changes wpctl volume.
