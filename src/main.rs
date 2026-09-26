mod drm_out;
mod pattern;

use anyhow::{bail, Result};
use std::path::PathBuf;

const W: usize = 60;
const H: usize = 2008;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--dump-png") => {
            let path = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("usage: mtmr --dump-png <path>"))?;
            let xrgb = drm_out::xrgb8888_bytes(W, H);
            let rgb = drm_out::xrgb8888_to_rgb(&xrgb, W, H);
            drm_out::dump_png(&path, W, H, &rgb)?;
            println!("PNG written: {path} ({W}x{H})");
        }
        Some(flag @ ("--drm" | "--live")) => {
            let _ = flag;
            if flag == "--live" {
                ctrlc::set_handler(|| {
                    println!("mtmr: exiting, releasing Touch Bar");
                    std::process::exit(0);
                })
                .expect("install signal handler");
            }
            drm_out::render_to_card(flag == "--live")?;
        }
        _ => bail!("usage: mtmr [--dump-png <path> | --drm | --live]"),
    }
    Ok(())
}
