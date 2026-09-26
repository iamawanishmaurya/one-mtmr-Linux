use anyhow::{anyhow, Context, Result};
use std::path::Path;

/// Slider position → percentage (0..=100).
pub fn pos_to_pct(rect_x: usize, rect_w: usize, x: i64) -> u8 {
    if rect_w == 0 {
        return 0;
    }
    let rel = x - rect_x as i64;
    let clamped = rel.clamp(0, rect_w as i64);
    (((clamped * 100) as f32 / rect_w as f32) as u8).min(100)
}

const BACKLIGHT: &str = "/sys/class/backlight/intel_backlight";
/// Old-project lesson: a fully dark panel is a trap — never go below 5%.
pub const BRIGHTNESS_FLOOR_PCT: u64 = 5;

pub fn read_brightness() -> Result<(u64, u64)> {
    let cur: u64 = std::fs::read_to_string(format!("{BACKLIGHT}/brightness"))
        .context("read brightness")?
        .trim()
        .parse()?;
    let max: u64 = std::fs::read_to_string(format!("{BACKLIGHT}/max_brightness"))
        .context("read max_brightness")?
        .trim()
        .parse()?;
    Ok((cur, max))
}

pub fn brightness_pct() -> Result<u8> {
    let (cur, max) = read_brightness()?;
    Ok(((cur * 100) / max.max(1)) as u8)
}

pub fn set_brightness_pct(pct: u8) -> Result<()> {
    let pct = pct.max(BRIGHTNESS_FLOOR_PCT as u8);
    let (_, max) = read_brightness()?;
    let val = (pct as u64 * max) / 100;
    std::fs::write(format!("{BACKLIGHT}/brightness"), format!("{val}"))
        .context("write brightness")?;
    Ok(())
}

/// Keyboard illumination step (max 14660 on this machine) by ±max/10.
pub fn step_illumination(dir: i64) -> Result<()> {
    let base = "/sys/class/leds/:white:kbd_backlight";
    let max: u64 = std::fs::read_to_string(format!("{base}/max_brightness"))
        .context("read kbd max")?
        .trim()
        .parse()?;
    let cur: u64 = std::fs::read_to_string(format!("{base}/brightness"))
        .context("read kbd brightness")?
        .trim()
        .parse()?;
    let step = (max / 10).max(1);
    let next = if dir >= 0 {
        (cur + step).min(max)
    } else {
        cur.saturating_sub(step)
    };
    std::fs::write(format!("{base}/brightness"), format!("{next}")).context("write kbd")?;
    Ok(())
}

/// Find a user runtime dir with a PipeWire socket (daemon runs as root; the
/// compositor session belongs to a real user). Prefers the highest-uid match.
fn pipewire_runtime_dir() -> Option<String> {
    let mut best: Option<(u32, String)> = None;
    for e in std::fs::read_dir("/run/user").ok()?.flatten() {
        let uid: u32 = match e.file_name().to_string_lossy().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let dir = e.path();
        if dir.join("pipewire-0").exists() {
            if best.as_ref().map(|(u, _)| uid > *u).unwrap_or(true) {
                best = Some((uid, dir.to_string_lossy().into_owned()));
            }
        }
    }
    best.map(|(_, d)| d)
}

/// Set volume via wpctl. Returns Err(message) when PipeWire is unreachable
/// (log once; volume keys via uinput still work).
pub fn set_volume_pct(pct: u8) -> Result<()> {
    let mut cmd = std::process::Command::new("wpctl");
    cmd.args([
        "set-volume",
        "@DEFAULT_AUDIO_SINK@",
        &format!("{pct}%"),
        "--limit",
        "1.0",
    ]);
    if let Some(dir) = pipewire_runtime_dir() {
        cmd.env("XDG_RUNTIME_DIR", &dir);
    }
    let out = cmd.output().context("spawn wpctl")?;
    if !out.status.success() {
        return Err(anyhow!(
            "wpctl failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

pub fn volume_pct() -> u8 {
    let mut cmd = std::process::Command::new("wpctl");
    cmd.args(["get-volume", "@DEFAULT_AUDIO_SINK@"]);
    if let Some(dir) = pipewire_runtime_dir() {
        cmd.env("XDG_RUNTIME_DIR", &dir);
    }
    let out = match cmd.output() {
        Ok(o) if o.status.success() => o,
        _ => return 0,
    };
    let s = String::from_utf8_lossy(&out.stdout);
    s.split_whitespace()
        .next()
        .and_then(|v| v.parse::<f32>().ok())
        .map(|v| (v * 100.0) as u8)
        .unwrap_or(0)
}

#[allow(dead_code)]
fn unused(p: &Path) -> bool {
    p.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_math() {
        assert_eq!(pos_to_pct(100, 200, 100), 0);
        assert_eq!(pos_to_pct(100, 200, 200), 50);
        assert_eq!(pos_to_pct(100, 200, 299), 99); // truncates 99.5
        // clamped
        assert_eq!(pos_to_pct(100, 200, 50), 0);
        assert_eq!(pos_to_pct(100, 200, 9999), 100);
        assert_eq!(pos_to_pct(0, 0, 5), 0);
    }

    #[test]
    #[ignore = "requires root + hardware; run: sudo cargo test --release -- --ignored"]
    fn brightness_roundtrip() {
        let orig = brightness_pct().unwrap();
        set_brightness_pct(80).unwrap();
        let mid = brightness_pct().unwrap();
        set_brightness_pct(orig as u8).unwrap();
        assert!((70..=90).contains(&mid), "mid={mid}");
    }

    #[test]
    #[ignore = "requires root + hardware; run: sudo cargo test --release -- --ignored"]
    fn illum_step_math_bound() {
        // verify stepping up is capped (indirect: call is safe, value within max)
        let before = std::fs::read_to_string("/sys/class/leds/:white:kbd_backlight/brightness")
            .unwrap()
            .trim()
            .parse::<u64>()
            .unwrap();
        let _ = step_illumination(1);
        let after = std::fs::read_to_string("/sys/class/leds/:white:kbd_backlight/brightness")
            .unwrap()
            .trim()
            .parse::<u64>()
            .unwrap();
        let max: u64 = std::fs::read_to_string("/sys/class/leds/:white:kbd_backlight/max_brightness")
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert!(after <= max);
        // restore
        std::fs::write("/sys/class/leds/:white:kbd_backlight/brightness", format!("{before}")).unwrap();
    }
}
