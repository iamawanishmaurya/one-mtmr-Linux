mod drm_out;
mod pattern;
mod preset;
mod render;
mod surface;

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
    let user = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".config/mtmr/items.json");
    if user.exists() {
        return user;
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
            drm_out::render_to_card(flag == "--live", &items)?;
        }
        _ => bail!("usage: mtmr [--dump-png <path> | --render-only [preset] | --drm | --live]"),
    }
    Ok(())
}
