# MTMR-Linux

**My TouchBar. My rules. — rebuilt for Linux.**

A Rust daemon that renders a fully customizable Touch Bar UI on T2 MacBook Pros
running Linux (t2linux), driven by MTMR-compatible JSON presets. Esc, media keys,
volume/brightness sliders, clock, battery, CPU and now-playing widgets, shell
command buttons — all from one `items.json`.

Rebuild of [Toxblh/MTMR](https://github.com/Toxblh/MTMR) (macOS) for
MacBookPro16,2-class hardware; original Swift source is vendored under
`reference/MTMR/` as reference only.

## How it works

```
items.json ──▶ mtmr daemon ──▶ appletbdrm DRM card (atomic page flips)
                    ├── evdev touch (tap-up classifier: tap / long-press)
                    ├── /dev/uinput virtual keyboard (esc, media, volume keys)
                    └── widgets: time · battery · CPU · MPRIS music · sliders
```

- Renders to the Touch Bar via `/dev/dri/card0` (connector `USB-1`, mode 60x2008).
  The panel only receives pixels on **framebuffer flips** — the daemon
  double-buffers and page-flips every redraw.
- Touch: "Apple Inc. Touch Bar Display Touchpad" (matched by name; the event
  number changes across boots). X axis 0–32767 → 2008 px.
- Orientation: `MTMR_VFLIP=1` (thickness mirror) is required; do **not** length-flip.
- Never uses legacy SetCrtc — it wedges the iBridge display endpoint
  (only a reboot recovers; see Troubleshooting).

## Install (Arch)

```
makepkg -sf            # or: git clone && cargo build --release
sudo cp target/release/mtmr /usr/bin/mtmr
sudo cp packaging/mtmr.service /etc/systemd/system/
sudo cp packaging/60-mtmr.rules /usr/lib/udev/rules.d/
sudo udevadm control --reload
sudo cp assets/example-items.json /etc/mtmr/items.json
sudo systemctl enable --now mtmr.service
```

Conflicts with `tiny-dfr.service` and old `mtmr-linux.service` automatically.

## Configure

Preset lookup order: `$MTMR_PRESET` → `~/.config/mtmr/items.json` →
`/etc/mtmr/items.json` → `/usr/share/mtmr/items.json` → bundled default.

```json
[
  { "type": "escape", "title": "esc", "width": 64, "align": "left" },
  { "type": "music", "width": 200, "align": "left" },
  { "type": "play", "width": 40, "align": "center" },
  { "type": "timeButton", "formatTemplate": "HH:mm", "width": 80, "align": "center" },
  { "type": "brightness", "width": 100, "align": "right" },
  { "type": "volume", "width": 100, "align": "right" }
]
```

- `width` is in MTMR units (full bar = 1080), scaled to the 2008 px panel.
- `align`: `left` / `center` / `right`.
- Actions: `{"trigger": "singleTap"|"longTap", "action": "keycode", "keycode": 53|{"esc","play","volumeup",…}}`
  or `{"action": "command", "command": "any shell command"}`.
- Built-in types: `escape previous play next volumeUp volumeDown mute
  brightnessUp brightnessDown illuminationUp illuminationDown displaySleep
  exitTouchbar staticButton shellScriptTitledButton timeButton battery cpu music
  brightness volume`.
- Real MTMR community presets load: comments, trailing commas, raw control
  characters, `group…close` containers and base64 icons are all handled;
  Apple-only item types are logged and skipped.

## CLI

```
mtmr --live                 # run the daemon (systemd unit uses this)
mtmr --render-only [file]   # render preset to /tmp/mtmr-bar.png (no hardware)
mtmr --lint <file-or-dir>   # batch preset validation
mtmr --selftest-touch       # tap classifier selftest
mtmr --selftest-uinput      # key injection selftest
sudo mtmr --recover         # rebind appletbdrm when the panel freezes on an old frame
```

## Troubleshooting

| Symptom | Fix |
|---|---|
| Bar frozen on an old frame | `sudo mtmr --recover`, then `sudo systemctl restart mtmr` |
| Bar blank after boot | check `sudo dmesg \| grep appletbdrm`; ensure `mtmr.service` active |
| Taps do nothing | ensure unit has **no `DeviceAllow=`** (it silently blocks `/dev/input`); check `journalctl -u mtmr` for the touch device line |
| Everything upside-down | set `Environment=MTMR_VFLIP=1` in the unit |
| Buttons mirrored | remove any `MTMR_FLIP=1` (length flip is never needed) |
| Volume slider no-op | `wpctl` must reach your PipeWire: `sudo XDG_RUNTIME_DIR=/run/user/$(id -u) wpctl get-volume @DEFAULT_AUDIO_SINK@` |

## Hardware (verified on MacBookPro16,2)

See `docs/hardware.md` for the full probed record: DRM connector, touch axis
ranges, backlight/LED sysfs paths, and the orientation mapping.
