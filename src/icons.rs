use std::collections::HashMap;
use std::sync::OnceLock;

/// Built-in icons for MTMR built-in item types.
///
/// Real MTMR ships designed SVG assets in its app bundle; community presets
/// reference them implicitly by item type. We embed Material Design icon
/// paths (Apache 2.0) and rasterize them with resvg for anti-aliased,
/// professional-quality glyphs — no more procedural pixel art.

const BRIGHTNESS_HIGH: &str = "M12 9c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3zm0-6l2.5 2.5h5v5L22 13l-2.5 2.5v5h-5L12 23l-2.5-2.5h-5v-5L2 13l2.5-2.5v-5h5L12 3z";
const BRIGHTNESS_LOW: &str = "M20 8.69V4h-4.69L12 .69 8.69 4H4v4.69L.69 12 4 15.31V20h4.69L12 23.31 15.31 20H20v-4.69L23.31 12 20 8.69zM12 18c-.89 0-1.74-.2-2.5-.55v-11c.76-.35 1.61-.55 2.5-.55 3.31 0 6 2.69 6 6s-2.69 6-6 6z";
const VOLUME_UP: &str = "M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z";
const VOLUME_DOWN: &str = "M18.5 12c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM5 9v6h4l5 5V4L9 9H5z";
const VOLUME_OFF: &str = "M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z";
const PLAY_PAUSE: &str = "M8 5v14l11-7z";
const SKIP_PREVIOUS: &str = "M6 6h2v12H6zm3.5 6l8.5 6V6z";
const SKIP_NEXT: &str = "M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z";
const NIGHT: &str = "M12 3a9 9 0 1 0 9 9c0-.46-.04-.92-.1-1.36a5.389 5.389 0 0 1-4.4 2.26 5.403 5.403 0 0 1-3.14-9.8c-.44-.06-.9-.1-1.36-.1z";

fn svg_body(path: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="{path}"/></svg>"#
    )
}

/// Rasterize one Material path into RGBA pixels (anti-aliased, white fill).
fn rasterize(path: &str, size: u32) -> Option<crate::preset::DecodedImage> {
    let tree = usvg::Tree::from_str(&svg_body(path), &usvg::Options::default()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(size, size)?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(size as f32 / 24.0, size as f32 / 24.0),
        &mut pixmap.as_mut(),
    );
    let data = pixmap.take();
    let mut rgba = Vec::with_capacity(data.len());
    for px in data.chunks_exact(4) {
        // premultiplied RGBA from tiny-skia; recolor to white, keep alpha
        let a = px[3];
        rgba.extend_from_slice(&[255, 255, 255, a]);
    }
    Some(crate::preset::DecodedImage {
        w: size as usize,
        h: size as usize,
        rgba,
    })
}

fn builtin_map() -> &'static HashMap<&'static str, crate::preset::DecodedImage> {
    static MAP: OnceLock<HashMap<&'static str, crate::preset::DecodedImage>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        let pairs: &[(&str, &str)] = &[
            ("brightnessUp", BRIGHTNESS_HIGH),
            ("brightnessDown", BRIGHTNESS_LOW),
            ("volumeUp", VOLUME_UP),
            ("volumeDown", VOLUME_DOWN),
            ("mute", VOLUME_OFF),
            ("play", PLAY_PAUSE),
            ("previous", SKIP_PREVIOUS),
            ("next", SKIP_NEXT),
            ("sleep", NIGHT),
            ("displaySleep", NIGHT),
        ];
        for (kind, path) in pairs {
            if let Some(img) = rasterize(path, 40) {
                m.insert(*kind, img);
            }
        }
        m
    })
}

/// Look up the anti-aliased built-in icon for a type, if any.
pub fn builtin_icon(kind: &str) -> Option<crate::preset::DecodedImage> {
    builtin_map().get(kind).cloned()
}
