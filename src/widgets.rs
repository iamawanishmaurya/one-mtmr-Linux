use std::time::Instant;

/// timeButton: MTMR formatTemplate ("HH:mm") → chrono format.
pub fn time_string(template: &str) -> String {
    let fmt = template
        .replace("HH", "%H")
        .replace("mm", "%M")
        .replace("ss", "%S");
    chrono::Local::now().format(&fmt).to_string()
}

/// battery: capacity + charging glyph from /sys/class/power_supply/BAT0.
pub fn battery_string() -> String {
    let base = "/sys/class/power_supply/BAT0";
    let cap = std::fs::read_to_string(format!("{base}/capacity"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);
    let charging = std::fs::read_to_string(format!("{base}/status"))
        .map(|s| s.trim() == "Charging" || s.trim() == "Full")
        .unwrap_or(false);
    format!("{cap}%{}", if charging { "⚡" } else { "" })
}

/// cpu: /proc/stat aggregated line sampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpuSample {
    pub idle: u64,
    pub total: u64,
}

pub fn cpu_sample() -> Option<CpuSample> {
    let stat = std::fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().next()?;
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|f| f.parse().ok())
        .collect();
    if fields.len() < 5 {
        return None;
    }
    let idle = fields[3] + fields[4]; // idle + iowait
    let total: u64 = fields.iter().sum();
    Some(CpuSample { idle, total })
}

/// CPU busy % between two samples (None until a second sample exists).
pub fn cpu_percent(prev: CpuSample, cur: CpuSample) -> Option<u8> {
    let dt = cur.total.checked_sub(prev.total)?;
    let di = cur.idle.checked_sub(prev.idle)?;
    if dt == 0 {
        return None;
    }
    Some((((dt - di) * 100) / dt) as u8)
}

/// music: current track via playerctl (empty string when nothing plays).
pub fn music_string() -> String {
    std::process::Command::new("playerctl")
        .args(["metadata", "--format", "{{artist}} - {{title}}"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Tracked widget state for the live loop.
pub struct Widgets {
    pub cpu_prev: Option<CpuSample>,
    pub last_music: Option<(Instant, String)>,
}

impl Widgets {
    pub fn new() -> Self {
        Widgets {
            cpu_prev: cpu_sample(),
            last_music: None,
        }
    }

    /// Compute the display title for a widget item kind.
    pub fn render(&mut self, kind: &str, template: Option<&str>, music_ttl_ms: u128) -> String {
        match kind {
            "timeButton" => time_string(template.unwrap_or("HH:mm")),
            "battery" => battery_string(),
            "cpu" => {
                let cur = match cpu_sample() {
                    Some(c) => c,
                    None => return "--".into(),
                };
                let pct = self
                    .cpu_prev
                    .and_then(|p| cpu_percent(p, cur))
                    .map(|p| format!("{p}%"))
                    .unwrap_or_else(|| "--".into());
                self.cpu_prev = Some(cur);
                pct
            }
            "music" => {
                let refresh = std::time::Duration::from_millis(2000);
                match &self.last_music {
                    Some((t, s)) if t.elapsed().as_millis() < music_ttl_ms.max(2000) => s.clone(),
                    _ => {
                        let s = music_string();
                        let _ = refresh;
                        self.last_music = Some((Instant::now(), s.clone()));
                        s
                    }
                }
            }
            _ => String::new(),
        }
    }
}

pub fn is_widget(kind: &str) -> bool {
    matches!(kind, "timeButton" | "battery" | "cpu" | "music")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_formats() {
        // just check it produces the right shape without panicking
        let s = time_string("HH:mm");
        assert_eq!(s.len(), 5);
        assert_eq!(s.as_bytes()[2], b':');
        let s2 = time_string("HH:mm:ss");
        assert_eq!(s2.len(), 8);
    }

    #[test]
    fn cpu_math() {
        let a = CpuSample { idle: 100, total: 200 };
        // +50 busy of +100 total → 50%
        let b = CpuSample { idle: 150, total: 300 };
        assert_eq!(cpu_percent(a, b), Some(50));
        // idle-only delta → 0%
        let c = CpuSample { idle: 250, total: 400 };
        assert_eq!(cpu_percent(b, c), Some(0));
        // full busy → 100%
        let d = CpuSample { idle: 250, total: 500 };
        assert_eq!(cpu_percent(c, d), Some(100));
        // no delta → None
        assert_eq!(cpu_percent(d, d), None);
    }

    #[test]
    fn battery_string_shape() {
        let s = battery_string();
        assert!(s.ends_with('%') || s.contains("⚡"));
    }
}
