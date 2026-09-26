---
phase: 4
plan: 01
wave: 1
depends_on: []
files_modified:
  - src/lenient.rs
  - src/preset.rs
  - src/main.rs
autonomous: false
requirements: [PRE-01, PRE-02, PRE-03]
---
# Plan 01 — Lenient preset loading + compatibility sweep

**Goal:** Real community MTMR presets load without errors: lenient preprocessing + group flattening + batch `--lint` proving it.

## Artifacts this phase produces
- `src/lenient.rs` — `sanitize(raw) -> String` (BOM, preamble, comments, trailing commas, control chars)
- preset.rs: group/close flattening; skip-list extended (dock, network, pomodoro, weather, currency, inputsource, dnd, nightShift, yandexWeather, upnext, spotify?)
- main.rs: `--lint <path>` batch mode
- fixtures in tests reproducing each real-world quirk

## Tasks
### Task 1 — Lenient preprocessor (Wave 1)
<read_first>src/preset.rs (strip_comments)</read_first>
<action>
Create src/lenient.rs::sanitize with string-aware state machine: BOM strip, preamble skip to first '[', // and /* */ comment removal, trailing-comma removal (between value and }/], outside strings), control-char escaping inside strings. Unit tests per rule using miniature fixtures modeled on the 12 sampled presets (incl. AppleScript inline with raw \r bytes, block-commented items, BOM, README preamble, trailing commas).
</action>
<acceptance_criteria>
- all lenient unit tests pass
- sanitize leaves valid JSON untouched (idempotence test)
</acceptance_criteria>
<verify><automated>cargo test --release 2>&1 | tail -2</automated><fails_when>test failures</fails_when></verify>

### Task 2 — group/close flattening + skip-list (Wave 2)
<read_first>src/preset.rs</read_first>
<action>
In load(): after sanitize, walk values; on type "group" set pending group attrs (align/width defaults) and continue; on "close" clear pending; children inherit group align when missing. Extend skip list: dock, network, pomodoro, weather, currency, inputsource, dnd, nightShift, yandexWeather, upnext, spotify, flush? — log each once per load. Accept legacy staticButton item-level action fields silently (serde ignores unknown keys already).
</action>
<acceptance_criteria>
- group fixture flattens: children render with group align; close consumed
- skip list logs once per type per load
</acceptance_criteria>
<verify><automated>cargo test --release 2>&1 | tail -2</automated><fails_when>test failures</fails_when></verify>

### Task 3 — --lint batch sweep (Wave 3)
<read_first>src/main.rs, .planning/phases/04-.../04-RESEARCH.md (downloaded sample in /tmp/presets)</read_first>
<action>
Add `--lint <path>`: file or dir; for each preset print "<file>: OK <rendered> rendered, <skipped> skipped" or "<file>: FAIL <reason>"; exit 0 iff all OK (dir mode: count FAILs). Run against the 12 downloaded community presets (re-download to /tmp/presets if absent). Fix any remaining parse failures in lenient.rs. Record results in 04-SUMMARY.md. Ship the best-rendering Linux-adapted community preset? No — keep current daily preset as bundled default (PRE-03 already satisfied).
</action>
<acceptance_criteria>
- ≥10/12 sample presets lint OK with ≥1 rendered item; FAILs (if any) documented with reason
- MTMR defaultPreset (reference/MTMR/MTMR/defaultPreset.json) lints OK and renders esc/brightness/media/volume/battery/time
</acceptance_criteria>
<verify><automated>./target/release/mtmr --lint /tmp/presets 2>&1 | tail -13</automated><fails_when>exit non-zero or any FAIL lines beyond documented ones</fails_when></verify>
