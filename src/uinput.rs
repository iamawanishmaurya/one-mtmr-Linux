use anyhow::{anyhow, Context, Result};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, EventType, InputEvent, KeyCode};
use std::time::Duration;

const ESC: u16 = 1;
const NEXTSONG: u16 = 163;
const PLAYPAUSE: u16 = 164;
const PREVIOUSSONG: u16 = 165;
const MUTE: u16 = 113;
const VOLUMEDOWN: u16 = 114;
const VOLUMEUP: u16 = 115;

pub fn named_keycode(name: &str) -> Option<u16> {
    Some(match name.to_ascii_lowercase().as_str() {
        "esc" | "escape" => ESC,
        "play" | "playpause" => PLAYPAUSE,
        "next" | "nextsong" => NEXTSONG,
        "previous" | "previoussong" => PREVIOUSSONG,
        "mute" => MUTE,
        "volumedown" => VOLUMEDOWN,
        "volumeup" => VOLUMEUP,
        _ => return None,
    })
}

/// Virtual keyboard via /dev/uinput for esc / media keys.
pub struct KeyInjector {
    dev: VirtualDevice,
}

impl KeyInjector {
    pub fn new() -> Result<Self> {
        let mut keys = AttributeSet::new();
        for c in [
            ESC, NEXTSONG, PLAYPAUSE, PREVIOUSSONG, MUTE, VOLUMEDOWN, VOLUMEUP,
        ] {
            keys.insert(KeyCode(c));
        }
        let dev = VirtualDeviceBuilder::new()
            .context("open /dev/uinput")?
            .name("mtmr keys")
            .with_keys(&keys)?
            .build()?;
        Ok(Self { dev })
    }

    /// Press + release a keycode (visible to the compositor as a real keyboard).
    pub fn inject(&mut self, code: u16) -> Result<()> {
        let syn = InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0);
        self.dev
            .emit(&[InputEvent::new(EventType::KEY.0, code, 1), syn])
            .context("uinput press")?;
        std::thread::sleep(Duration::from_millis(80));
        self.dev
            .emit(&[InputEvent::new(EventType::KEY.0, code, 0), syn])
            .context("uinput release")?;
        Ok(())
    }
}

/// Selftest: create the virtual device and inject KEY_ESC.
pub fn selftest() -> Result<()> {
    let mut inj = KeyInjector::new()?;
    inj.inject(ESC)?;
    println!("SELFTEST-UINPUT OK (KEY_ESC injected via virtual device 'mtmr keys')");
    Ok(())
}

/// Resolve a preset action's keycode field (numeric or named).
pub fn resolve_keycode(k: &crate::preset::Keycode) -> Option<u16> {
    match k {
        crate::preset::Keycode::Code(c) => Some(*c),
        crate::preset::Keycode::Named(n) => named_keycode(n),
    }
}

#[allow(dead_code)]
fn unused(_: u16) {
    let _ = anyhow!(MUTE);
}
