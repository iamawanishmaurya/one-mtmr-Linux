mod drm_out;
mod pattern;
mod preset;
mod render;
mod surface;
mod touch;
mod uinput;

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
    render::draw(&mut surf, items, &rects, &font);
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
            let items = preset::load(&preset)?;
            let surf = draw_bar(&items)?; // landscape: 2008 long x 60 thick
            let out = "/tmp/mtmr-bar.png";
            drm_out::dump_png(out, surf.w, surf.h, &surf.to_rgb())?;
            println!("{} items; PNG written: {out} ({}x{})", items.len(), surf.w, surf.h);
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
    let font = load_font()?;
    let rects = render::layout(&items, H);

    // orientation: MTMR_FLIP=1 mirrors the bar (set after visual check)
    let flip = std::env::var("MTMR_FLIP").map(|v| v == "1").unwrap_or(false);
    let mut backend = drm_out::DrmBackend::open(W as u32)?;

    let mut land = surface::Surface::new(H, W, H);
    let mut redraw = |land: &mut surface::Surface, hl: Option<usize>| {
        render::draw(land, &items, &rects, &font);
        if let Some(i) = hl {
            land.invert_rect(rects[i]);
        }
    };
    redraw(&mut land, None);
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
            redraw(&mut land, None);
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
                redraw(&mut land, highlighted);
                backend.present(&land)?;
            }
            Ok(touch::Ev::Move { x }) => {
                let n = now(t0);
                let _ = classifier.feed(&touch::Ev::Move { x }, n);
                if highlighted.is_some() {
                    if let Some(d) = classifier.down {
                        if d.moved {
                            highlighted = None;
                            redraw(&mut land, None);
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
                    redraw(&mut land, None);
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

        // periodic flip so the panel keeps receiving frames
        if last_flip.elapsed() >= std::time::Duration::from_secs(2) {
            last_flip = std::time::Instant::now();
            redraw(&mut land, highlighted);
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
    if let Some(cmd) = &action.command {
        match std::process::Command::new("sh").arg("-c").arg(cmd).spawn() {
            Ok(_) => {}
            Err(e) => eprintln!("mtmr: command failed: {e}"),
        }
    }
    Dispatch::Handled
}
