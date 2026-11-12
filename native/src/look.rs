//! The look of Pulse 1.8 ("76. The look" in the guide), as pulse.js paints it: one light (the real Sun by day, the
//! real Moon by night) that the towers catch and that lies on the floor as a path of glints; a tower for each CPU
//! core with a side, a roof and a reflection; the far city mirrored on the wet floor; air and mist over the
//! horizon; the beams of a low Sun; rings where the rain lands. `polish = false` in [pulse] turns it off.

use crate::draw::{self, PI, TAU};
use crate::hub::{get, list, num};
use crate::life::Frame;
use crate::palette::{mix, Pal, C};
use crate::scene::{gradient, rect, sky_xy, Building};
use gpui::Window;
use serde_json::Value;

/// The light of the scene: where it is, its colour, how strong, and how low when it is the Sun.
#[derive(Clone, Copy)]
pub struct Light {
    pub x: f32,
    pub y: f32,
    pub c: C,
    pub a: f32,
    pub sun: bool,
    pub low: f32,
}

pub fn on(st: &Value) -> bool {
    get(st, &["ui", "polish"]).as_bool().unwrap_or(true)
}

pub fn light(st: &Value, pal: &Pal, f: &Frame) -> Option<Light> {
    let cloud = (num(st, &["weather", "cloud"]).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
    if let (Some(az), Some(el)) = (num(st, &["night", "sun", "az"]), num(st, &["night", "sun", "el"])) {
        if el > -1.0 {
            if let Some(p) = sky_xy(az as f32, el as f32, f.w, f.hz) {
                let (a, low) = ((1.0 - cloud).powf(1.5), (1.0 - el as f32 / 12.0).max(0.0));
                if a > 0.04 {
                    return Some(Light { x: p.0, y: p.1, c: mix(pal.warn, pal.crit, low * 0.5), a: a * (0.55 + 0.45 * low), sun: true, low });
                }
            }
        }
    }
    if let (Some(az), Some(el)) = (num(st, &["night", "moon", "az"]), num(st, &["night", "moon", "el"])) {
        if el > -1.0 {
            if let Some(p) = sky_xy(az as f32, el as f32, f.w, f.hz) {
                let lit = num(st, &["night", "moon", "lit"]).unwrap_or(0.0) as f32;
                let am = (0.35 + 0.65 * f.sky[0].max(0.3)) * (1.0 - 0.75 * cloud) * (0.25 + 0.75 * lit);
                if am > 0.08 {
                    return Some(Light { x: p.0, y: p.1, c: mix(pal.fg, [1.0, 0.957, 0.839], 0.5), a: am * 0.7, sun: false, low: 0.0 });
                }
            }
        }
    }
    None
}

/// A low Sun sends its beams across the sky.
pub fn rays(win: &mut Window, l: &Light, f: &Frame, still: bool) {
    if !l.sun || l.low < 0.25 {
        return;
    }
    let t = if still { 0.0 } else { f.t };
    let (x, y) = (f.ox + l.x, f.oy + l.y);
    for i in 0..9 {
        let a0 = -PI * (0.04 + i as f32 * 0.115) + (t * 0.07 + i as f32 * 1.9).sin() * 0.03;
        let wd = 0.035 + ((i * 37) % 5) as f32 * 0.012;
        // a beam fades with the distance: three lengths, each fainter
        for (len, k) in [(f.h * 0.28, 0.05), (f.h * 0.5, 0.035), (f.h * 0.75, 0.02)] {
            draw::poly(win, &[(x, y), (x + (a0 - wd).cos() * len, y + (a0 - wd).sin() * len), (x + (a0 + wd).cos() * len, y + (a0 + wd).sin() * len)], l.c, k * l.a * l.low);
        }
    }
}

/// Air: the far city fades into the colour of the sky at the horizon, the city lights the low sky at night, and the
/// same air lies over the far floor.
pub fn air(win: &mut Window, pal: &Pal, f: &Frame, low: C) -> C {
    let (nk, tk) = (f.sky[0], f.sky[1]);
    let hzc = mix(low, pal.fg, 0.10 + 0.10 * (1.0 - nk));
    gradient(win, f.ox, f.oy + f.hz - f.h * 0.17, f.w, f.h * 0.1275, hzc, 0.0, hzc, 0.20 + 0.10 * tk);
    gradient(win, f.ox, f.oy + f.hz - f.h * 0.0425, f.w, f.h * 0.0425 + 1.0, hzc, 0.20 + 0.10 * tk, hzc, 0.42 + 0.12 * tk);
    if nk > 0.05 {
        let c = mix(pal.accent2, pal.warn, 0.25);
        for (k, a) in [(1.0, 0.03), (0.72, 0.04), (0.46, 0.045), (0.24, 0.045)] {
            draw::arc_fill(win, f.ox + f.w / 2.0, f.oy + f.hz, f.w * 0.5 * k, f.w * 0.17 * k, 0.0, PI, TAU, c, a * nk);
        }
    }
    gradient(win, f.ox, f.oy + f.hz, f.w, (f.h - f.hz) * 0.5, hzc, 0.22, hzc, 0.0);
    hzc
}

/// The floor is wet: the far city is mirrored on it, fading as it comes near.
pub fn mirror(win: &mut Window, city: &[Building], pal: &Pal, f: &Frame, low: C) {
    let rh = (f.h - f.hz).min(f.h * 0.2);
    if rh < 8.0 {
        return;
    }
    for b in city {
        let depth = b.layer as f32 / 2.0;
        let body = mix(mix(pal.bg, low, 0.70 - depth * 0.30), pal.fg, 0.02 + depth * 0.02);
        let bh = (b.h * f.h).min(rh);
        gradient(win, f.ox + b.x * f.w, f.oy + f.hz, b.w * f.w, bh, body, 0.42, body, 0.42 * (1.0 - bh / rh).max(0.0));
    }
    let fc = mix(pal.bg, pal.accent2, 0.1);
    gradient(win, f.ox, f.oy + f.hz, f.w, rh + 1.0, fc, 0.0, fc, 0.97);
}

/// Mist drifting between the rows of the far city: thicker at dawn, at night and in fog.
pub fn mist(win: &mut Window, f: &Frame, c: C, k: f32, still: bool) {
    if k < 0.03 {
        return;
    }
    let t = if still { 0.0 } else { f.t };
    let th = f.h * 0.12;
    for layer in 0..2 {
        let lf = layer as f32;
        let speed = if layer == 1 { -0.006 } else { 0.009 };
        let y = f.oy + f.hz - th * if layer == 1 { 0.39 } else { 0.30 };
        for i in 0..7 {
            let fi = i as f32;
            let x = ((fi * 0.171 + lf * 0.37 + t * speed * f.h / f.w).rem_euclid(1.2) - 0.1) * f.w;
            let (rx, ry) = (f.w * (0.10 + 0.05 * ((fi * 1.7 + lf).sin() * 0.5 + 0.5)), th * (0.22 + 0.10 * ((fi * 2.3).cos() * 0.5 + 0.5)));
            draw::ellipse(win, f.ox + x, y + (fi * 1.3).sin() * th * 0.12, rx, ry, c, 0.07 * k.min(1.0));
            draw::ellipse(win, f.ox + x, y + (fi * 1.3).sin() * th * 0.12, rx * 0.6, ry * 0.6, c, 0.06 * k.min(1.0));
        }
    }
}

/// The light on the floor: a low Sun's glow along the horizon, a pool long toward you, a path of glints.
pub fn light_path(win: &mut Window, l: &Light, f: &Frame, still: bool) {
    let t = if still { 0.0 } else { f.t };
    let (x, hz) = (f.ox + l.x, f.oy + f.hz);
    if l.sun && l.low > 0.05 {
        for (k, a) in [(1.0, 0.07), (0.6, 0.10), (0.3, 0.12)] {
            draw::arc_fill(win, x, hz, f.w * 0.30 * k, f.w * 0.066 * k, 0.0, PI, TAU, l.c, a * l.a * l.low);
        }
    }
    let (wv, hgt) = (f.h * 0.05, (f.h - f.hz) * 0.9);
    for (k, a) in [(1.0, 0.06), (0.62, 0.10), (0.32, 0.20)] {
        draw::arc_fill(win, x, hz, wv * 2.4 * k, hgt * k, 0.0, 0.0, PI, l.c, a * l.a);
    }
    for i in 0..9 {
        let u = (i as f32 + 0.5) / 9.0;
        let yy = hz + hgt * 0.75 * u * u + f.h / 400.0;
        let ww = wv * (0.5 + 2.6 * u) * (0.55 + 0.45 * (t * (0.7 + i as f32 * 0.13) + i as f32 * 2.1).sin());
        rect(win, x - ww / 2.0, yy, ww, (f.h / 700.0).max(1.0), l.c, 0.75 * l.a * (1.0 - u * 0.8));
    }
}

/// Where the rain lands a ring opens on the wet floor.
pub fn rain_rings(win: &mut Window, f: &Frame, pal: &Pal, mm: f32, still: bool) {
    if still {
        return;
    }
    let n = (6.0 + mm.min(6.0) * 5.0) as usize;
    for i in 0..n {
        let ph = f.t * 1.3 + i as f32 * 0.37;
        let (age, turn) = (ph.fract(), ph.floor());
        let hsh = |k: f32| ((turn * 12.9898 + i as f32 * 78.233 + k).sin() * 43758.547).fract().abs();
        let d = hsh(1.0).powf(1.5);
        let (x, y) = (f.ox + hsh(2.0) * f.w, f.oy + f.hz + f.h * 0.01 + d * (f.h - f.hz) * 0.9);
        let r = age * f.h * (0.006 + 0.022 * d);
        draw::arc(win, x, y, r, r * 0.26, 0.0, TAU, 1.0, pal.fg, 0.30 * (1.0 - age) * (0.4 + 0.6 * d));
    }
}

/// The skyline: a tower for each CPU core, as tall as its load (it dances with your music), its windows lit as the
/// RAM fills; a building, not a card: a side going back toward the horizon, a roof with a mast, a tank or the air
/// conditioning, a lit ground floor, its reflection on the wet floor. `skip` slots belong to your repos.
#[allow(clippy::too_many_arguments)]
pub fn towers(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, lt: Option<Light>, heights: &mut Vec<f32>, audio: &[f32], skip: usize, inner: f32, polish: bool) {
    let (h, hz) = (f.h, f.oy + f.hz);
    let cx = f.ox + f.w / 2.0;
    let (bg, ac, a2, black) = (pal.bg, pal.accent, pal.accent2, [0.0, 0.0, 0.0]);
    let cores: Vec<f32> = list(st, &["sys", "cores"]).iter().filter_map(|c| c.as_f64().map(|v| v as f32)).collect();
    let cpu = num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32;
    let ram = num(st, &["sys", "ram"]).unwrap_or(0.0) as f32;
    let music = get(st, &["music"]).as_bool().unwrap_or(false) && !audio.is_empty();
    let focus = get(st, &["focus"]).is_object();
    let nc = cores.len().clamp(8, 32);
    let side = nc.div_ceil(2).max(1);
    let (b0, b1) = crate::extra::band(f);
    let outer = (f.w / 2.0 - b0).min(b1 - f.w / 2.0);
    let slot = ((outer - inner) / side as f32).max(4.0);
    let bw = slot * 0.74;
    heights.resize(nc.max(heights.len()), -1.0);
    let (nk, tk) = (f.sky[0], f.sky[1]);
    for i in 0..nc {
        let mut load = if cores.is_empty() { cpu } else { cores[i % cores.len()] };
        if music {
            load = (load * 0.4 + audio[(i / 2).min(audio.len() - 1)] * 100.0 * 0.9).min(100.0); // the city dances
        }
        heights[i] = if heights[i] < 0.0 { load } else { heights[i] + (load - heights[i]) * if music { 0.35 } else { 0.06 } };
        if i < skip {
            continue; // a repo's tower stands here
        }
        let hh = h * 0.035 + heights[i] / 100.0 * h * 0.2 * if focus { 0.3 } else { 1.0 } + ((i * 37) % 11) as f32 / 11.0 * h * 0.03;
        let (s2, idx) = (if i % 2 == 1 { 1.0 } else { -1.0 }, (i / 2) as f32);
        let bx = if s2 > 0.0 { cx + inner + idx * slot } else { cx - inner - idx * slot - bw };
        gradient(win, bx, hz - hh, bw, hh, mix(bg, ac, 0.16), 1.0, mix(bg, black, 0.3), 1.0);
        if polish {
            gradient(win, bx, hz, bw, hh * 0.75, mix(bg, ac, 0.34), 0.62, mix(bg, ac, 0.34), 0.0); // the tower on the wet floor
            let in_x = if s2 > 0.0 { bx } else { bx + bw };
            let (dw, dir) = ((slot * 0.24).min(bw * 0.34), -s2);
            let dy = hh * (dw / dw.max((in_x - cx).abs()) * 0.9 + 0.05).min(0.2);
            let lit_side = lt.map(|l| (f.ox + l.x < in_x) == (dir < 0.0)).unwrap_or(false);
            let side_c = match (lit_side, lt) {
                (true, Some(l)) => mix(mix(bg, ac, 0.10), l.c, 0.30 * l.a),
                _ => mix(bg, black, 0.42),
            };
            draw::poly(win, &[(in_x, hz - hh), (in_x + dir * dw, hz - hh + dy), (in_x + dir * dw, hz), (in_x, hz)], side_c, 1.0);
            draw::line(win, &[(in_x, hz - hh), (in_x + dir * dw, hz - hh + dy)], (h / 1200.0).max(1.0), ac, 0.28);
            if let Some(l) = lt {
                // the front edge on the side of the Sun or the Moon catches its light
                let rw = (bw * 0.16).max(1.5) * 2.4;
                let right = f.ox + l.x >= bx + bw / 2.0;
                let steps = 4;
                for k in 0..steps {
                    let sw = rw / steps as f32;
                    let x = if right { bx + bw - sw * (k + 1) as f32 } else { bx + sw * k as f32 };
                    rect(win, x, hz - hh, sw, hh, l.c, 0.8 * l.a * (1.0 - (k as f32 + 0.5) / steps as f32));
                }
            }
            // on the roof: by building a mast, a tank or the air conditioning
            let (rk, ru) = ((i as u32).wrapping_mul(2654435761) % 5, (h / 700.0).max(1.0));
            let rc = mix(bg, ac, 0.34);
            match rk {
                0 => {
                    rect(win, bx + bw * 0.5 - ru / 2.0, hz - hh - hh * 0.16 - h / 90.0, ru, hh * 0.16 + h / 90.0, rc, 1.0);
                    rect(win, bx + bw * 0.5 - ru * 2.0, hz - hh - hh * 0.07, ru * 4.0, ru, rc, 1.0);
                }
                1 => {
                    rect(win, bx + bw * 0.16, hz - hh - h / 110.0, bw * 0.3, h / 110.0, rc, 1.0);
                    rect(win, bx + bw * 0.2, hz - hh - h / 110.0 - ru * 2.0, bw * 0.22, ru * 2.0, rc, 1.0);
                }
                2 => {
                    rect(win, bx + bw * 0.12, hz - hh - h / 190.0, bw * 0.2, h / 190.0, rc, 1.0);
                    rect(win, bx + bw * 0.42, hz - hh - h / 190.0, bw * 0.2, h / 190.0, rc, 1.0);
                    rect(win, bx + bw * 0.72, hz - hh - h / 150.0, bw * 0.14, h / 150.0, rc, 1.0);
                }
                3 => {
                    rect(win, bx + bw * 0.25, hz - hh - h / 70.0, bw * 0.5, h / 70.0, rc, 1.0);
                    rect(win, bx + bw * 0.25, hz - hh - h / 70.0, bw * 0.5, ru, ac, 0.6);
                }
                _ => {}
            }
            rect(win, bx + bw * 0.12, hz - (h / 260.0).max(2.0), bw * 0.76, (h / 520.0).max(1.5), mix(pal.warn, pal.fg, 0.3), 0.25 + 0.5 * nk.max(tk)); // the lit ground floor
        }
        rect(win, bx, hz - hh, bw, (h / 600.0).max(1.0), if heights[i] > 90.0 { pal.crit } else { ac }, 0.85);
        // the windows: more of them lit as the RAM fills
        let (cw, chh) = ((bw / 5.0).max(2.0), (h / 140.0).max(2.0));
        let (cols, rows) = ((((bw - cw) / (cw * 1.7)).floor() as i32).max(1), ((hh - chh * 2.0) / (chh * 2.2)).floor() as i32);
        for ry in 0..rows.min(60) {
            for cxi in 0..cols.min(8) {
                let hsh = ((i as i64 * 7919 + ry as i64 * 104729 + cxi as i64 * 1299709) % 1000) as f32 / 1000.0;
                if hsh < ram / 100.0 * 0.75 {
                    let flick = if (f.t * 0.07 + hsh * 13.0).fract() > 0.985 { 0.2 } else { 1.0 };
                    let c = if hsh < 0.12 { a2 } else if hsh < 0.5 { ac } else { pal.warn };
                    let a = ((0.35 + hsh * 0.5) * flick * if polish { 0.8 + 0.45 * nk.max(tk * 0.7) } else { 1.0 }).min(1.0);
                    rect(win, bx + cw * 0.7 + cxi as f32 * cw * 1.7, hz - hh + chh * 2.0 + ry as f32 * chh * 2.2, cw, chh, c, a);
                }
            }
        }
    }
}
