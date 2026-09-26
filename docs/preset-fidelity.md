# Preset fidelity — known gap & fix (2026-09-27)

## Issue (user-reported)

Bar rendered from the mrcsmxms community preset "matched nothing" compared to the
real MTMR screenshot (mrcsmxms/pic.png):

1. **Sliders rendered as full-height solid gray rectangles.** Real MTMR draws
   sliders in the macOS `NSSlider` style: a slim track line with a round white
   thumb knob at the current value.
2. **Built-in item types rendered as text fallbacks** (`-b`, `+b`, `-`, `+`).
   Real MTMR ships icon assets inside its app bundle for every built-in type
   (sun for brightness, speaker for volume, play glyph, …) and community presets
   rely on those implicit icons.

## Root cause

- MTMR is a macOS app: sliders are AppKit controls and icons are bundled asset
  files. A Linux rebuild has neither, and our Phase-2/3 renderer drew the
  simplest possible placeholders.
- Community presets never reference those icons explicitly — they are implied
  by the item `type` — so any compatible renderer must provide its own icon set.

## Fix (this repo)

1. **`draw_slider`** rewritten: 6 px slim track centered vertically, blue fill
   line, and a white circular thumb (r=13 px) at the value position.
2. **`src/icons.rs`**: procedural (font-independent) icon drawing — filled
   circles, triangles, rounded rects — rendered directly onto the bar surface:
   - `brightnessUp` sun with rays / `brightnessDown` dim sun
   - `volumeUp` speaker with waves / `volumeDown` speaker / `mute` muted speaker
   - `play` play-pause glyph, `previous` / `next` double-triangles
   - `sleep` crescent moon, `displaySleep` moon+Z
   Built-in types draw their icon when the preset gives no explicit `title`
   or `image`; text remains the fallback.
3. Icons are generated in code (no binary assets) so the PKGBUILD stays
   source-only and every icon scales with the 60 px bar height.

Result: the mrcsmxms preset now renders visually close to the macOS original.
