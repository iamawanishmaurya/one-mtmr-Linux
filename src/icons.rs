use crate::surface::{Color, Rect, Surface, WHITE};

/// Procedural built-in icons for MTMR built-in item types.
///
/// Real MTMR ships icon assets in its app bundle for built-in types and
/// community presets rely on them implicitly. We draw equivalent icons
/// directly (no binary assets, scales with the 60 px bar).

pub fn icon_for(kind: &str, surf: &mut Surface, r: Rect) -> bool {
    let d = match kind {
        "brightnessUp" => return sun(surf, r, true),
        "brightnessDown" => return sun(surf, r, false),
        "volumeUp" => return speaker(surf, r, 2),
        "volumeDown" => return speaker(surf, r, 1),
        "mute" => return speaker(surf, r, 0),
        "play" => return play_pause(surf, r),
        "previous" => return triangles(surf, r, true),
        "next" => return triangles(surf, r, false),
        "sleep" | "displaySleep" => return moon(surf, r),
        _ => false,
    };
    let _ = d;
    true
}

fn fill_circle(surf: &mut Surface, cx: i32, cy: i32, r: i32, c: Color) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                surf.set_px((cx + dx) as usize, (cy + dy) as usize, c);
            }
        }
    }
}

fn fill_rect_i(surf: &mut Surface, x: i32, y: i32, w: i32, h: i32, c: Color) {
    for yy in y..y + h {
        for xx in x..x + w {
            surf.set_px(xx as usize, yy as usize, c);
        }
    }
}

/// Filled right-pointing triangle from (x, y) spanning w×h.
fn fill_triangle_right(surf: &mut Surface, x: i32, y: i32, w: i32, h: i32, c: Color, flip: bool) {
    for row in 0..h {
        let frac = row as f32 / h as f32;
        let span = (w as f32 * (if flip { frac.max(1.0 - frac) } else { 1.0 - frac })) as i32;
        let xx = if flip { x } else { x + (w - span) };
        fill_rect_i(surf, xx, y + row, span.max(1), 1, c);
    }
}

fn sun(surf: &mut Surface, r: Rect, bright: bool) -> bool {
    let cx = r.x as i32 + r.w as i32 / 2;
    let cy = r.y as i32 + r.h as i32 / 2;
    let core = if bright { 9 } else { 5 };
    fill_circle(surf, cx, cy, core, WHITE);
    // 8 rays
    let len = if bright { 8 } else { 4 };
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::FRAC_PI_4;
        for d in (core + 2)..(core + 2 + len) {
            let px = cx + (d as f32 * a.cos()) as i32;
            let py = cy + (d as f32 * a.sin()) as i32;
            surf.set_px(px as usize, py as usize, WHITE);
        }
    }
    true
}

fn speaker(surf: &mut Surface, r: Rect, waves: i32) -> bool {
    let cx = r.x as i32 + r.w as i32 / 2 - 6;
    let cy = r.y as i32 + r.h as i32 / 2;
    let c = WHITE;
    // box
    fill_rect_i(surf, cx - 10, cy - 5, 7, 10, c);
    // cone (triangle pointing left)
    for row in -10i32..10i32 {
        let t = 1.0 - row.abs() as f32 / 10.0;
        let w = (5.0 * t) as i32;
        fill_rect_i(surf, cx - 3, cy + row, w.max(1), 1, c);
    }
    // waves: arcs approximated by short vertical strokes
    for k in 0..waves {
        let rx = cx + 9 + k * 5;
        let hgt = 6 + k * 4;
        for row in -hgt..=hgt {
            let t = row as f32 / hgt as f32;
            let off = (4.0 * (1.0 - t * t).sqrt()) as i32;
            surf.set_px((rx + off) as usize, (cy + row) as usize, c);
        }
    }
    if waves == 0 {
        // mute X
        for i in -6..6 {
            surf.set_px((cx + 12 + i) as usize, (cy + i) as usize, Color(255, 60, 60));
            surf.set_px((cx + 12 + i) as usize, (cy - i) as usize, Color(255, 60, 60));
        }
    }
    true
}

fn play_pause(surf: &mut Surface, r: Rect) -> bool {
    let cy = r.y as i32 + r.h as i32 / 2;
    let x0 = r.x as i32 + r.w as i32 / 2 - 14;
    fill_triangle_right(surf, x0, cy - 10, 14, 20, WHITE, false);
    fill_rect_i(surf, x0 + 20, cy - 10, 4, 20, WHITE);
    fill_rect_i(surf, x0 + 27, cy - 10, 4, 20, WHITE);
    true
}

fn triangles(surf: &mut Surface, r: Rect, left: bool) -> bool {
    let cy = r.y as i32 + r.h as i32 / 2;
    let x0 = r.x as i32 + r.w as i32 / 2 - 16;
    fill_triangle_right(surf, x0, cy - 10, 12, 20, WHITE, left);
    fill_triangle_right(surf, x0 + 15, cy - 10, 12, 20, WHITE, left);
    if left {
        fill_rect_i(surf, x0 - 6, cy - 10, 3, 20, WHITE);
    } else {
        fill_rect_i(surf, x0 + 29, cy - 10, 3, 20, WHITE);
    }
    true
}

fn moon(surf: &mut Surface, r: Rect) -> bool {
    let cx = r.x as i32 + r.w as i32 / 2;
    let cy = r.y as i32 + r.h as i32 / 2;
    fill_circle(surf, cx, cy, 11, WHITE);
    // bite out with background-colored circle
    fill_circle(surf, cx + 6, cy - 3, 9, Color(10, 10, 10));
    true
}

/// MTMR-style slim slider: thin track line + white round thumb at the value.
pub fn draw_slider(surf: &mut Surface, r: Rect, pct: u8) {
    let cy = r.y as i32 + r.h as i32 / 2;
    let track_h = 6;
    let x0 = r.x as i32 + 2;
    let w = r.w as i32 - 4;
    // track
    fill_rect_i(surf, x0, cy - track_h / 2, w, track_h, Color(90, 90, 90));
    // fill
    let fill_w = (w as u32 * pct.min(100) as u32 / 100) as i32;
    if fill_w > 0 {
        fill_rect_i(surf, x0, cy - track_h / 2, fill_w, track_h, Color(70, 130, 250));
    }
    // thumb
    let kx = (x0 + fill_w).clamp(x0 + 8, x0 + w - 8);
    fill_circle(surf, kx, cy, 13, WHITE);
}
