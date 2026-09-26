use crate::preset::{Align, Item};
use crate::surface::{Color, Rect, Surface, BG, WHITE};
use anyhow::Result;
use std::path::PathBuf;

/// Compute display rects for items across the 3 MTMR segments on a bar of `bar_w` px.
/// Left items pack from the left edge, right items from the right edge,
/// center items form a centered cluster. 8 px margins / gaps.
pub fn layout(items: &[Item], bar_w: usize) -> Vec<Rect> {
    const MARGIN: usize = 8;
    const GAP: usize = 8;
    let h = 60;
    let mut rects = vec![Rect { x: 0, y: 0, w: 0, h }; items.len()];

    // Widths within each segment; center cluster is centered as a whole.
    let mut left_x = MARGIN;
    let mut right_x = bar_w.saturating_sub(MARGIN);
    let center_count = items.iter().filter(|it| it.align == Align::Center).count();
    // Cluster width includes the internal gaps added during packing.
    let center_w = items
        .iter()
        .filter(|it| it.align == Align::Center)
        .map(|it| it.width)
        .sum::<usize>()
        + center_count.saturating_sub(1) * GAP;

    let mut center_x = bar_w.saturating_sub(center_w) / 2;

    for (i, it) in items.iter().enumerate() {
        let w = it.width.min(bar_w);
        match it.align {
            Align::Left => {
                rects[i] = Rect { x: left_x, y: 0, w, h };
                left_x += w + GAP;
            }
            Align::Center => {
                rects[i] = Rect { x: center_x, y: 0, w, h };
                center_x += w + GAP;
            }
            Align::Right => {
                right_x = right_x.saturating_sub(w);
                rects[i] = Rect { x: right_x, y: 0, w, h };
                right_x = right_x.saturating_sub(GAP);
            }
        }
    }
    rects
}

/// Locate a system font for fontdue rasterization.
pub fn find_font() -> Result<Vec<u8>> {
    let candidates = [
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];
    for c in candidates {
        if PathBuf::from(c).exists() {
            return Ok(std::fs::read(c)?);
        }
    }
    // last resort: first .ttf under /usr/share/fonts
    fn walk(dir: &PathBuf) -> Option<PathBuf> {
        let rd = std::fs::read_dir(dir).ok()?;
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map(|e| e == "ttf").unwrap_or(false) {
                return Some(p);
            }
            if p.is_dir() {
                if let Some(f) = walk(&p) {
                    return Some(f);
                }
            }
        }
        None
    }
    walk(&PathBuf::from("/usr/share/fonts"))
        .ok_or_else(|| anyhow::anyhow!("no ttf font found under /usr/share/fonts"))
        .and_then(|p| Ok(std::fs::read(p)?))
}

/// Draw the bar: dark background + each item's title centered in its rect.
/// slider_pcts overrides the fill percentage for slider kinds.
pub fn draw(
    surf: &mut Surface,
    items: &[Item],
    rects: &[Rect],
    font: &fontdue::Font,
    slider_pcts: &std::collections::HashMap<usize, u8>,
) {
    surf.clear(BG);
    for (idx, (it, r)) in items.iter().zip(rects.iter()).enumerate() {
        match it.kind.as_str() {
            "brightness" | "volume" => {
                let pct = slider_pcts.get(&idx).copied().unwrap_or(50);
                draw_slider(surf, *r, pct);
                continue;
            }
            _ => {}
        }
        if let Some(img) = &it.decoded_image {
            draw_image_centered(surf, *r, img);
        } else if !it.title.is_empty() {
            draw_text_centered(surf, *r, &it.title, font);
        }
        // subtle item outline like MTMR's ShowButtonOutlines
        outline(surf, *r, Color(60, 60, 60));
    }
}

/// Blit an RGBA image centered in the rect, scaled to fit 44px height.
pub fn draw_image_centered(surf: &mut Surface, r: Rect, img: &crate::preset::DecodedImage) {
    if img.w == 0 || img.h == 0 || r.w < 4 || r.h < 4 {
        return;
    }
    let max_h = (r.h - 8).min(44);
    let scale = (max_h as f32 / img.h as f32).min(r.w.saturating_sub(8) as f32 / img.w as f32);
    let (dw, dh) = ((img.w as f32 * scale) as usize, (img.h as f32 * scale) as usize);
    if dw == 0 || dh == 0 {
        return;
    }
    let x0 = r.x + (r.w.saturating_sub(dw)) / 2;
    let y0 = r.y + (r.h.saturating_sub(dh)) / 2;
    for dy in 0..dh {
        for dx in 0..dw {
            let sx = dx * img.w / dw;
            let sy = dy * img.h / dh;
            let off = (sy * img.w + sx) * 4;
            let a = img.rgba[off + 3] as u32;
            if a == 0 {
                continue;
            }
            let px = x0 + dx;
            let py = y0 + dy;
            if px >= surf.w || py >= surf.h {
                continue;
            }
            let dst = (py * surf.stride + px) * 4;
            for c in 0..3 {
                let bg = surf.buf[dst + c] as u32;
                let fg = img.rgba[off + (2 - c)] as u32; // RGBA -> BGR
                surf.buf[dst + c] = ((fg * a + bg * (255 - a)) / 255) as u8;
            }
        }
    }
}

/// Draw a slider: dark track, white fill proportional to pct, outline.
pub fn draw_slider(surf: &mut Surface, r: Rect, pct: u8) {
    surf.fill_rect(r, Color(30, 30, 30));
    let fill_w = (r.w * pct.min(100) as usize) / 100;
    if fill_w > 0 {
        surf.fill_rect(
            Rect { x: r.x, y: r.y, w: fill_w, h: r.h },
            Color(220, 220, 220),
        );
    }
    outline(surf, r, Color(80, 80, 80));
}

fn outline(surf: &mut Surface, r: Rect, c: Color) {
    if r.w < 2 || r.h < 2 {
        return;
    }
    for x in r.x..r.x + r.w {
        surf.set_px(x, r.y, c);
        surf.set_px(x, r.y + r.h - 1, c);
    }
    for y in r.y..r.y + r.h {
        surf.set_px(r.x, y, c);
        surf.set_px(r.x + r.w - 1, y, c);
    }
}

pub fn draw_text_centered(surf: &mut Surface, r: Rect, text: &str, font: &fontdue::Font) {
    let size = 26.0;
    // measure
    let mut width = 0.0;
    let mut glyphs = Vec::new();
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, size);
        width += metrics.advance_width;
        glyphs.push((metrics, bitmap));
    }
    let ascent = size * 0.8;
    let mut pen_x = if width <= r.w as f32 {
        r.x as f32 + (r.w as f32 - width) / 2.0
    } else {
        r.x as f32 + 4.0 // left-align + clip long text inside the rect
    };
    let clip_x0 = r.x as i32;
    let clip_x1 = (r.x + r.w) as i32;
    let baseline = r.y as i32 + ((r.h as f32 + ascent) / 2.0) as i32;
    for (m, bitmap) in glyphs {
        let gx = pen_x as i32 + m.xmin as i32;
        let gy = baseline - m.height as i32 - m.ymin as i32;
        for (pi, alpha) in bitmap.iter().enumerate() {
            if *alpha == 0 {
                continue;
            }
            let px = gx + (pi % m.width) as i32;
            let py = gy + (pi / m.width) as i32;
            if px < clip_x0 || px >= clip_x1 || py < 0 {
                continue;
            }
            // alpha blend onto background
            let a = (*alpha as u32) / 255;
            let off = (py as usize * surf.stride + px as usize) * 4;
            if px < surf.w as i32 && py < surf.h as i32 {
                let b = &mut surf.buf[off..off + 3];
                b[0] = (BG.2 as u32 * (255 - a) + WHITE.2 as u32 * a) as u8 / 1;
                b[1] = (BG.1 as u32 * (255 - a) + WHITE.1 as u32 * a) as u8 / 1;
                b[2] = (BG.0 as u32 * (255 - a) + WHITE.0 as u32 * a) as u8 / 1;
            }
        }
        pen_x += m.advance_width;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::{Action, Item};

    fn item(kind: &str, title: &str, width: usize, align: Align) -> Item {
        Item {
            kind: kind.into(),
            title: title.into(),
            width,
            align,
            actions: vec![Action {
                trigger: "singleTap".into(),
                action: None,
                keycode: None,
                command: None,
            }],
            refresh_interval: None,
            source: None,
            skipped_reason: None,
            image: None,
            decoded_image: None,
            format_template: None,
        }
    }

    #[test]
    fn left_center_right_layout() {
        let items = vec![
            item("escape", "esc", 100, Align::Left),
            item("play", ">||", 100, Align::Center),
            item("exitTouchbar", "X", 100, Align::Right),
        ];
        let rects = layout(&items, 2008);
        assert_eq!(rects[0].x, 8); // left margin
        // centered cluster: single 100px item -> x = (2008-100)/2 = 954
        assert_eq!(rects[1].x, 954);
        // right item flush against right margin: 2008-8-100 = 1900
        assert_eq!(rects[2].x, 1900);
    }

    #[test]
    fn center_cluster_is_symmetric() {
        let items = vec![
            item("previous", "a", 100, Align::Center),
            item("play", "b", 100, Align::Center),
            item("next", "c", 100, Align::Center),
        ];
        let rects = layout(&items, 2008);
        // cluster width = 3*100 + 2*8 = 316; x0 = (2008-316)/2 = 846
        assert_eq!(rects[0].x, 846);
        assert_eq!(rects[1].x, 954);
        assert_eq!(rects[2].x, 1062);
        // symmetric: distance of cluster left edge from left == right edge from right
        assert_eq!(rects[0].x, 2008 - (rects[2].x + rects[2].w));
    }

    #[test]
    fn width_clamped_to_bar() {
        let items = vec![item("staticButton", "huge", 9999, Align::Left)];
        let rects = layout(&items, 2008);
        assert_eq!(rects[0].w, 2008);
    }
}
