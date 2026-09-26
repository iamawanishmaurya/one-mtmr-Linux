use anyhow::{anyhow, Context, Result};
use evdev::{Device, EventType};
use evdev::raw_stream::RawDevice;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Instant;

/// Touch events from the Touch Bar touchpad, x already scaled to bar pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ev {
    Down { x: i64 },
    Move { x: i64 },
    Up,
}

/// Classifier outcome.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tap {
    Single,
    Long,
}

pub const LONG_PRESS_MS: u64 = 500;
pub const SLOP_PX: i64 = 12;

/// Tap-up state machine (port of the old project's validated behavior):
/// tap fires on release if it started <500 ms ago and moved <12 px;
/// a finger held >=500 ms fires a long-press once.
#[derive(Debug, Default)]
pub struct Classifier {
    pub down: Option<DownState>,
}

#[derive(Debug, Clone, Copy)]
pub struct DownState {
    pub origin: i64,
    pub t0: u64,
    pub moved: bool,
    pub fired: bool,
}

impl Classifier {
    pub fn feed(&mut self, ev: &Ev, now_ms: u64) -> Option<Tap> {
        match ev {
            Ev::Down { x } => {
                self.down = Some(DownState {
                    origin: *x,
                    t0: now_ms,
                    moved: false,
                    fired: false,
                });
                None
            }
            Ev::Move { x } => {
                if let Some(d) = self.down.as_mut() {
                    if (*x - d.origin).abs() > SLOP_PX {
                        d.moved = true;
                    }
                }
                None
            }
            Ev::Up => {
                let d = self.down.take()?;
                if !d.fired && !d.moved && now_ms.saturating_sub(d.t0) < LONG_PRESS_MS {
                    Some(Tap::Single)
                } else {
                    None
                }
            }
        }
    }

    /// Call periodically; fires the long-press once per held finger.
    pub fn tick(&mut self, now_ms: u64) -> Option<Tap> {
        let d = self.down.as_mut()?;
        if !d.fired && !d.moved && now_ms.saturating_sub(d.t0) >= LONG_PRESS_MS {
            d.fired = true;
            Some(Tap::Long)
        } else {
            None
        }
    }
}

/// Find the Touch Bar digitizer (name match — the event number re-numbers).
pub fn find_device() -> Result<PathBuf> {
    for e in std::fs::read_dir("/dev/input")?.flatten() {
        let p = e.path();
        if !p
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("event") && !n.contains('-'))
            .unwrap_or(false)
        {
            continue;
        }
        if let Ok(dev) = Device::open(&p) {
            if dev
                .name()
                .map(|n| n.contains("Touch Bar Display Touchpad"))
                .unwrap_or(false)
            {
                return Ok(p);
            }
        }
    }
    Err(anyhow!("Touch Bar digitizer not found in /dev/input"))
}

/// Spawn a reader thread; returns the receiver of scaled touch events.
/// `bar_len` = length axis in px, `flip` mirrors the coordinate.
pub fn spawn_reader(bar_len: usize, flip: bool) -> Result<Receiver<Ev>> {
    let path = find_device()?;
    let _probe = Device::open(&path).with_context(|| format!("open {}", path.display()))?;
    let mut dev = RawDevice::open(&path).with_context(|| format!("open {}", path.display()))?;
    eprintln!("mtmr: touch input on {}", path.display());
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let scale = |raw: i32| -> i64 {
            let px = (raw as i64) * (bar_len as i64) / 32768;
            if flip {
                (bar_len as i64 - 1) - px
            } else {
                px
            }
        };
        let mut last_x: i64 = 0;
        loop {
            let evs = match dev.fetch_events() {
                Ok(e) => e,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(_) => break,
            };
            for ev in evs {
                match ev.event_type() {
                    EventType::ABSOLUTE if ev.code() == 0 => {
                        // ABS_X — value arrives only while a finger is down
                        last_x = scale(ev.value());
                    }
                    EventType::KEY if ev.code() == 330 => match ev.value() {
                        1 => {
                            let _ = tx.send(Ev::Down { x: last_x });
                        }
                        _ => {
                            let _ = tx.send(Ev::Up);
                        }
                    },
                    _ => {}
                }
            }
        }
    });
    Ok(rx)
}

/// Selftest: synthetic event script through the classifier.
pub fn selftest() -> Result<()> {
    let mut c = Classifier::default();
    // quick tap -> Single
    assert_eq!(c.feed(&Ev::Down { x: 100 }, 0), None);
    assert_eq!(c.feed(&Ev::Up, 100), Some(Tap::Single));
    // held finger -> Long via tick
    assert_eq!(c.feed(&Ev::Down { x: 100 }, 1000), None);
    assert_eq!(c.tick(1000 + LONG_PRESS_MS), Some(Tap::Long));
    assert_eq!(c.feed(&Ev::Up, 1200), None);
    // drag past slop -> nothing
    assert_eq!(c.feed(&Ev::Down { x: 100 }, 2000), None);
    assert_eq!(c.feed(&Ev::Move { x: 100 + SLOP_PX + 1 }, 2100), None);
    assert_eq!(c.feed(&Ev::Up, 2200), None);
    // slow tap (>= long) -> nothing
    assert_eq!(c.feed(&Ev::Down { x: 50 }, 3000), None);
    assert_eq!(c.feed(&Ev::Up, 3000 + LONG_PRESS_MS + 1), None);
    println!("SELFTEST-TOUCH OK (SINGLE, LONG, NONE-drag, NONE-slow)");
    Ok(())
}

#[allow(dead_code)]
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[allow(dead_code)]
fn unused_instant(t: Instant) -> u64 {
    t.elapsed().as_millis() as u64
}
