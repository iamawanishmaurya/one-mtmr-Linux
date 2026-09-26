mod drm_out;
mod lenient;
mod pattern;
mod preset;
mod render;
mod slider;
mod surface;
mod touch;
mod uinput;
mod widgets;

use anyhow::{bail, Result};
use std::path::PathBuf;

const W: usize = 60;
const H: usize = 2008;

fn load_font() -> Result<fontdue::Font> {
    let data = render::find_font()?;
    fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
        .map_err(|e| anyhow::anyhow!("font load: {e:?}"))
}

/// Build the bar surface in landscape coords (2008 long x 60 thick).
/// The DRM buffer is 60x2008 (length on the y axis); drm_out rotates on blit.
fn draw_bar(items: &[preset::Item]) -> Result<surface::Surface> {
    let font = load_font()?;
    let mut surf = surface::Surface::new(H, W, H);
    let rects = render::layout(items, H);
    render::draw(&mut surf, items, &rects, &font, &Default::default());
    Ok(surf)
}

fn default_preset_path() -> PathBuf {
    // 1. $MTMR_PRESET  2. ~/.config/mtmr/items.json  3. /etc/mtmr/items.json  4. bundled default
    if let Ok(p) = std::env::var("MTMR_PRESET") {
        return PathBuf::from(p);
    }
    let user = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".config/mtmr/items.json");
    if user.exists() {
        return user;
    }
    let system = PathBuf::from("/etc/mtmr/items.json");
    if system.exists() {
        return system;
    }
    let shared = PathBuf::from("/usr/share/mtmr/items.json");
    if shared.exists() {
        return shared;
    }
    PathBuf::from("assets/default-items.json")
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--dump-png") => {
            let path = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("usage: mtmr --dump-png <path>"))?;
            let mut surf = surface::Surface::new(W, H, W);
            pattern::fill_test_pattern_stride(&mut surf.buf, W, H, W);
            drm_out::dump_png(&path, W, H, &surf.to_rgb())?;
            println!("PNG written: {path} ({W}x{H})");
        }
        Some("--render-only") => {
            let preset = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_preset_path);
            let mut items = preset::load(&preset)?;
            // fill live widget values so previews are realistic
            let mut widgets = widgets::Widgets::new();
            for it in items.iter_mut() {
                if widgets::is_widget(&it.kind) {
                    it.title = widgets.render(&it.kind, it.format_template.as_deref(), 4000);
                }
            }
            let surf = draw_bar(&items)?; // landscape: 2008 long x 60 thick
            let out = "/tmp/mtmr-bar.png";
            drm_out::dump_png(out, surf.w, surf.h, &surf.to_rgb())?;
            println!("{} items; PNG written: {out} ({}x{})", items.len(), surf.w, surf.h);
        }
        Some("--recover") => {
            // Rebind appletbdrm to re-init a stale display session (panel frozen
            // on an old frame despite successful commits). Requires root.
            use anyhow::Context as _;
            let unbind = "/sys/bus/usb/drivers/appletbdrm/unbind";
            let bind = "/sys/bus/usb/drivers/appletbdrm/bind";
            std::fs::write(unbind, "5-6:2.1").context("unbind appletbdrm")?;
            std::thread::sleep(std::time::Duration::from_millis(1500));
            std::fs::write(bind, "5-6:2.1").context("bind appletbdrm")?;
            println!("mtmr: appletbdrm rebound — display session re-initialized");
        }
        Some("--lint") => {
            let path = args
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| anyhow::anyhow!("usage: mtmr --lint <file-or-dir>"))?;
            let mut files: Vec<PathBuf> = Vec::new();
            if path.is_dir() {
                let mut entries: Vec<_> = std::fs::read_dir(&path)?
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
                    .collect();
                entries.sort();
                files = entries;
            } else {
                files.push(path);
            }
            let mut fails = 0;
            for f in &files {
                match preset::load(f) {
                    Ok(items) => {
                        let skipped = skipped_count(f);
                        println!(
                            "{}: OK {} rendered, {} skipped",
                            f.display(),
                            items.len(),
                            skipped
                        );
                    }
                    Err(e) => {
                        fails += 1;
                        println!("{}: FAIL {}", f.display(), e);
                    }
                }
            }
            if fails > 0 {
                bail!("{fails} preset(s) failed");
            }
            println!("lint: all {} presets OK", files.len());
        }
        Some("--selftest-touch") => touch::selftest()?,
        Some("--selftest-uinput") => uinput::selftest()?,
        Some(flag @ ("--drm" | "--live")) => {
            if flag == "--live" {
                ctrlc::set_handler(|| {
                    println!("mtmr: exiting, releasing Touch Bar");
                    std::process::exit(0);
                })
                .expect("install signal handler");
            }
            let preset = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_preset_path);
            let items = preset::load(&preset)?;
            if flag == "--drm" {
                drm_out::render_to_card(false, &items)?;
            } else {
                live_loop(items)?;
            }
        }
        _ => bail!("usage: mtmr [--dump-png <path> | --render-only [preset] | --drm | --live | --selftest-touch | --selftest-uinput]"),
    }
    Ok(())
}

/// The Phase-2 live loop: render the preset, consume touch, dispatch actions.
fn live_loop(items: Vec<preset::Item>) -> Result<()> {
    // backlight on: the panel can be dark after a driver rebind (looks like a
    // dead bar), and a dark panel + dim-guard silently eats taps (old lesson)
    if let Ok(max) = std::fs::read_to_string("/sys/class/backlight/appletb_backlight/max_brightness") {
        let _ = std::fs::write(
            "/sys/class/backlight/appletb_backlight/brightness",
            max.trim(),
        );
    }
    let font = load_font()?;
    let rects = render::layout(&items, H);
    let mut mutated = items.clone();

    // orientation: MTMR_FLIP=1 mirrors the bar (set after visual check)
    let flip = std::env::var("MTMR_FLIP").map(|v| v == "1").unwrap_or(false);
    let mut backend = drm_out::DrmBackend::open(W as u32)?;
    let mut widgets = widgets::Widgets::new();
    // slider state per item index, seeded from current hardware
    let mut slider_pcts: std::collections::HashMap<usize, u8> = std::collections::HashMap::new();
    for (i, it) in items.iter().enumerate() {
        let pct = match it.kind.as_str() {
            "brightness" => slider::brightness_pct().unwrap_or(50),
            "volume" => slider::volume_pct(),
            _ => continue,
        };
        slider_pcts.insert(i, pct);
    }
    let mut last_volume_err: Option<String> = None;

    let mut land = surface::Surface::new(H, W, H);
    fn redraw(
        land: &mut surface::Surface,
        items: &[preset::Item],
        rects: &[surface::Rect],
        font: &fontdue::Font,
        slider_pcts: &std::collections::HashMap<usize, u8>,
        hl: Option<usize>,
    ) {
        render::draw(land, items, rects, font, slider_pcts);
        if let Some(i) = hl {
            land.invert_rect(rects[i]);
        }
    }
    macro_rules! redraw { ($hl:expr) => { redraw(&mut land, &mutated, &rects, &font, &slider_pcts, $hl) } }
    redraw!(None);
    backend.present(&land)?;
    backend.present(&land)?; // both buffers hold the same frame

    let rx = touch::spawn_reader(H, flip)?;
    let mut injector = uinput::KeyInjector::new().ok();
    let mut classifier = touch::Classifier::default();
    let mut highlighted: Option<usize> = None;
    let mut last_flip = std::time::Instant::now();
    let t0 = std::time::Instant::now();
    let mut now = |t0: std::time::Instant| t0.elapsed().as_millis() as u64;

    loop {
        // long-press deadline
        let n = now(t0);
        if let Some(tap) = classifier.tick(n) {
            if let Some(d) = classifier.down {
                let _ = d; // long-press has no distinct action in the default preset yet
            }
            let _ = tap;
            redraw!(None);
            backend.present(&land)?;
            highlighted = None;
        }

        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(touch::Ev::Down { x }) => {
                let n = now(t0);
                let _ = classifier.feed(&touch::Ev::Down { x }, n);
                let idx = rects
                    .iter()
                    .position(|r| x >= r.x as i64 && x < (r.x + r.w) as i64);
                highlighted = idx;
                redraw!(highlighted);
                backend.present(&land)?;
            }
            Ok(touch::Ev::Move { x }) => {
                let n = now(t0);
                let _ = classifier.feed(&touch::Ev::Move { x }, n);
                // slider drag: finger started inside a slider rect
                if let Some(i) = highlighted {
                    if items[i].kind == "brightness" || items[i].kind == "volume" {
                        let r = rects[i];
                        if (x as usize) >= r.x && (x as usize) < r.x + r.w {
                            let pct = slider::pos_to_pct(r.x, r.w, x);
                            slider_pcts.insert(i, pct);
                            let kind = items[i].kind.clone();
                            if kind == "brightness" {
                                let _ = slider::set_brightness_pct(pct);
                            } else {
                                match slider::set_volume_pct(pct) {
                                    Ok(()) => last_volume_err = None,
                                    Err(e) => {
                                        if last_volume_err.as_deref() != Some(e.to_string().as_str()) {
                                            eprintln!("mtmr: {e}");
                                            last_volume_err = Some(e.to_string());
                                        }
                                    }
                                }
                            }
                            redraw!(highlighted);
                            backend.present(&land)?;
                        }
                        continue;
                    }
                }
                if highlighted.is_some() {
                    if let Some(d) = classifier.down {
                        if d.moved {
                            highlighted = None;
                            redraw!(None);
                            backend.present(&land)?;
                        }
                    }
                }
            }
            Ok(touch::Ev::Up) => {
                let n = now(t0);
                let tap = classifier.feed(&touch::Ev::Up, n);
                if let Some(tap) = tap {
                    let idx = highlighted;
                    highlighted = None;
                    redraw!(None);
                    backend.present(&land)?;
                    if let Some(i) = idx {
                        let trigger = match tap {
                            touch::Tap::Single => "singleTap",
                            touch::Tap::Long => "longTap",
                        };
                        if dispatch(&items[i], trigger, &mut injector) == Dispatch::Exit {
                            println!("mtmr: exitTouchbar");
                            return Ok(());
                        }
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                bail!("touch reader died")
            }
        }

        // widget refresh + periodic flip
        if last_flip.elapsed() >= std::time::Duration::from_secs(1) {
            last_flip = std::time::Instant::now();
            let mut changed = false;
            for (i, it) in mutated.iter_mut().enumerate() {
                if widgets::is_widget(&it.kind) {
                    let val = widgets.render(&it.kind, it.format_template.as_deref(), 4000);
                    if val != it.title {
                        it.title = val;
                    }
                }
            }
            redraw!(highlighted);
            backend.present(&land)?;
        }
    }
}

#[derive(PartialEq)]
enum Dispatch {
    Handled,
    Exit,
}

fn builtin_keycode(kind: &str) -> Option<u16> {
    Some(match kind {
        "escape" => 1,
        "previous" => 165,
        "play" => 164,
        "next" => 163,
        "volumeUp" => 115,
        "volumeDown" => 114,
        "mute" => 113,
        "brightnessUp" => 225,
        "brightnessDown" => 224,
        "displaySleep" => 142,
        _ => return None,
    })
}

fn dispatch(
    item: &preset::Item,
    trigger: &str,
    injector: &mut Option<uinput::KeyInjector>,
) -> Dispatch {
    if item.kind == "exitTouchbar" {
        return Dispatch::Exit;
    }
    if item.kind == "illuminationUp" {
        let _ = slider::step_illumination(1);
        return Dispatch::Handled;
    }
    if item.kind == "illuminationDown" {
        let _ = slider::step_illumination(-1);
        return Dispatch::Handled;
    }
    if item.actions.is_empty() {
        if let Some(code) = builtin_keycode(&item.kind) {
            if let Some(inj) = injector {
                let _ = inj.inject(code);
            }
            return Dispatch::Handled;
        }
        if item.kind == "shellScriptTitledButton" {
            if let Some(src) = &item.source {
                if let Some(inline) = &src.inline {
                    let _ = std::process::Command::new("sh").arg("-c").arg(inline).spawn();
                }
            }
            return Dispatch::Handled;
        }
    }
    let action = item
        .actions
        .iter()
        .find(|a| a.trigger == trigger || (trigger == "longTap" && a.trigger == "longTap"))
        .or_else(|| item.actions.first());
    let Some(action) = action else {
        return Dispatch::Handled;
    };
    if let Some(k) = &action.keycode {
        if let Some(code) = uinput::resolve_keycode(k) {
            match injector {
                Some(inj) => {
                    if let Err(e) = inj.inject(code) {
                        eprintln!("mtmr: key inject failed: {e}");
                    }
                }
                None => eprintln!("mtmr: no uinput device (permission?)"),
            }
            return Dispatch::Handled;
        }
    }
    if item.kind == "music" && item.actions.is_empty() {
        let _ = std::process::Command::new("playerctl").arg("play-pause").spawn();
        return Dispatch::Handled;
    }
    if let Some(cmd) = &action.command {
        match std::process::Command::new("sh").arg("-c").arg(cmd).spawn() {
            Ok(_) => {}
            Err(e) => eprintln!("mtmr: command failed: {e}"),
        }
    }
    Dispatch::Handled
}

/// Count skipped items by re-reading and diffing (lint nicety, not exact).
fn skipped_count(path: &PathBuf) -> usize {
    let raw = match std::fs::read_to_string(path) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    match serde_json::from_str::<Vec<serde_json::Value>>(&crate::lenient::sanitize(&raw)) {
        Ok(all) => all.len(),
        Err(_) => 0,
    }
}
