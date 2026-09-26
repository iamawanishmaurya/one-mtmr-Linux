use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Phase-2 subset of the MTMR items.json schema.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default = "default_width")]
    pub width: usize,
    #[serde(default)]
    pub align: Align,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default, rename = "refreshInterval")]
    pub refresh_interval: Option<u64>,
    #[serde(default)]
    pub source: Option<Source>,
    /// Unknown-type items are skipped but keep their raw type for logging.
    #[serde(skip)]
    pub skipped_reason: Option<String>,
}

fn default_width() -> usize {
    100
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(default = "default_trigger")]
    pub trigger: String,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub keycode: Option<Keycode>,
    #[serde(default)]
    pub command: Option<String>,
}

fn default_trigger() -> String {
    "singleTap".into()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keycode {
    Named(String),
    Code(u16),
}

impl<'de> Deserialize<'de> for Keycode {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = Keycode;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("keycode number or name")
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Keycode, E> {
                Ok(Keycode::Code(v as u16))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Keycode, E> {
                Ok(Keycode::Named(v.to_string()))
            }
        }
        d.deserialize_any(V)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Source {
    #[serde(default)]
    pub inline: Option<String>,
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Item kinds with fixed semantics in the Phase 2 bar.
pub const KNOWN_TYPES: &[&str] = &[
    "escape",
    "exitTouchbar",
    "staticButton",
    "previous",
    "play",
    "next",
    "volumeDown",
    "volumeUp",
    "mute",
    "brightnessDown",
    "brightnessUp",
];

pub fn load(path: &Path) -> Result<Vec<Item>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read preset {}", path.display()))?;
    let items: Vec<serde_json::Value> =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let mut out = Vec::new();
    for v in items {
        let kind = v
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("(missing)")
            .to_string();
        if !KNOWN_TYPES.contains(&kind.as_str()) {
            eprintln!("mtmr: skipping unsupported item type '{kind}'");
            continue;
        }
        match serde_json::from_value::<Item>(v) {
            Ok(mut it) => {
                if it.title.is_empty() {
                    it.title = default_title(&it.kind);
                }
                out.push(it);
            }
            Err(e) => eprintln!("mtmr: skipping malformed '{kind}' item: {e}"),
        }
    }
    if out.is_empty() {
        bail!("preset has no usable items");
    }
    Ok(out)
}

fn default_title(kind: &str) -> String {
    match kind {
        "escape" => "esc".into(),
        "previous" => "|<".into(),
        "play" => ">||".into(),
        "next" => ">>".into(),
        "volumeUp" => "+".into(),
        "volumeDown" => "-".into(),
        "mute" => "M".into(),
        "brightnessUp" => "+b".into(),
        "brightnessDown" => "-b".into(),
        "exitTouchbar" => "X".into(),
        other => other.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_subset_and_skips_unknown() {
        let dir = std::env::temp_dir().join("mtmr-preset-test.json");
        std::fs::write(
            &dir,
            r#"[
                {"type":"escape","align":"left"},
                {"type":"weirdUnknownType","title":"x"},
                {"type":"staticButton","title":"hi","width":80,
                 "actions":[{"trigger":"singleTap","action":"keycode","keycode":53}]},
                {"type":"play","align":"center"}
            ]"#,
        )
        .unwrap();
        let items = load(&dir).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].kind, "escape");
        assert_eq!(items[0].title, "esc");
        assert_eq!(items[1].kind, "staticButton");
        assert_eq!(items[1].width, 80);
        assert_eq!(
            items[1].actions[0].keycode,
            Some(Keycode::Code(53))
        );
        assert_eq!(items[2].align, Align::Center);
    }

    #[test]
    fn rejects_empty_presets() {
        let dir = std::env::temp_dir().join("mtmr-preset-empty.json");
        std::fs::write(&dir, r#"[{"type":"mystery"}]"#).unwrap();
        assert!(load(&dir).is_err());
    }
}
