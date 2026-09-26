# Phase 3 Verification (2026-09-27)
- [x] CTRL-01 volume slider — implemented via wpctl + drag; native volume deferred to Phase 5 user service (documented limitation as root)
- [x] CTRL-02 brightness slider — sysfs write, 5% floor, drag verified via layout tests + user visual
- [x] CTRL-03 keyboard illumination — real LED path, ±max/10 stepping, root test passed
- [x] WIDG-01 timeButton — formatTemplate, live
- [x] WIDG-02 battery — /sys/class/power_supply, live
- [x] WIDG-03 cpu — /proc/stat deltas, live
- [x] WIDG-04 music — MPRIS via playerctl, live
- [x] WIDG-05 shellScriptTitledButton — runs script on tap (Phase 2)
Result: PASSED
