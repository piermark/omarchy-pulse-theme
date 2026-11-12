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

/// pulse.js drawRays(): a low Sun sends its beams across the sky: nine wedges filled with one radial gradient from
/// the Sun (drawn here as bands, each with the gradient's alpha at its middle).
pub fn rays(win: &mut Window, l: &Light, f: &Frame, still: bool) {
    if !l.sun || l.low < 0.25 {
        return;
    }
    let t = if still { 0.0 } else { f.t };
    let (x, y) = (f.ox + l.x, f.oy + l.y);
    let (len, r0) = (f.h * 0.75, f.h * 0.02);
    let a_at = |r: f32| if r <= r0 { 1.0 } else { (1.0 - (r - r0) / (len - r0)).max(0.0) } * 0.10 * l.a * l.low;
    let bands = 14;
    for i in 0..9 {
        let a0 = -PI * (0.04 + i as f32 * 0.115) + (t * 0.07 + i as f32 * 1.9).sin() * 0.03;
        let wd = 0.035 + ((i * 37) % 5) as f32 * 0.012;
        for k in 0..bands {
            let (ra, rb) = (len * k as f32 / bands as f32, len * (k + 1) as f32 / bands as f32);
            let p = |r: f32, s: f32| (x + (a0 + s * wd).cos() * r, y + (a0 + s * wd).sin() * r);
            draw::poly(win, &[p(ra, -1.0), p(rb, -1.0), p(rb, 1.0), p(ra, 1.0)], l.c, a_at((ra + rb) / 2.0));
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

thread_local! {
    /// pulse.js fogTexture(): 1024×96, made once for a colour (and a strength), kept on the GPU
    static FOG: std::cell::RefCell<Option<(String, std::sync::Arc<gpui::RenderImage>)>> = Default::default();
}

fn fog_texture(win: &mut Window, c: C, k: f32) -> Option<std::sync::Arc<gpui::RenderImage>> {
    let kq = (k.min(1.0) * 24.0).round() / 24.0;
    let key = format!("{:.0},{:.0},{:.0}|{kq}", c[0] * 255.0, c[1] * 255.0, c[2] * 255.0);
    if let Some(i) = FOG.with(|f| f.borrow().as_ref().filter(|x| x.0 == key).map(|x| x.1.clone())) {
        return Some(i);
    }
    use tiny_skia::{Color, GradientStop, Paint, Pixmap, RadialGradient, Rect, SpreadMode, Transform};
    let mut pm = Pixmap::new(1024, 96)?;
    let mut rn = crate::backdrop::Rnd::new(4242);
    let col = |a: f32| Color::from_rgba(c[0].clamp(0.0, 1.0), c[1].clamp(0.0, 1.0), c[2].clamp(0.0, 1.0), a.clamp(0.0, 1.0)).unwrap_or(Color::TRANSPARENT);
    for _ in 0..46 {
        let (px0, py0, r, a) = (rn.next() * 1024.0, 30.0 + rn.next() * 44.0, 40.0 + rn.next() * 90.0, 0.05 + rn.next() * 0.09);
        for kk in -1..=1 {
            // drawn three times, so the picture joins with itself at the sides
            let cx = px0 + kk as f32 * 1024.0;
            let ts = Transform::from_translate(0.0, py0).pre_scale(1.0, 0.32).pre_translate(0.0, -py0);
            let stops = vec![GradientStop::new(0.0, col(a)), GradientStop::new(1.0, col(0.0))];
            if let (Some(sh), Some(rect)) = (RadialGradient::new((cx, py0).into(), (cx, py0).into(), r, stops, SpreadMode::Pad, Transform::identity()), Rect::from_xywh(cx - r, py0 - r, r * 2.0, r * 2.0)) {
                pm.fill_rect(rect, &Paint { shader: sh, anti_alias: false, ..Default::default() }, ts, None);
            }
        }
    }
    let mut px4 = pm.take();
    for p in px4.chunks_exact_mut(4) {
        let a = p[3] as f32 / 255.0;
        if a > 0.0 {
            let (r, g, b) = ((p[0] as f32 / a).min(255.0), (p[1] as f32 / a).min(255.0), (p[2] as f32 / a).min(255.0));
            p[0] = b as u8;
            p[1] = g as u8;
            p[2] = r as u8;
            p[3] = (p[3] as f32 * kq) as u8;
        }
    }
    let im = image::RgbaImage::from_raw(1024, 96, px4)?;
    let img = std::sync::Arc::new(gpui::RenderImage::new(vec![image::Frame::new(im)]));
    if let Some(old) = FOG.with(|f| f.borrow_mut().replace((key, img.clone()))) {
        let _ = win.drop_image(old.1);
    }
    Some(img)
}

/// pulse.js drawMist(): two layers of fog drifting along the horizon, one each way.
pub fn mist(win: &mut Window, f: &Frame, c: C, k: f32, still: bool) {
    if k < 0.03 {
        return;
    }
    let Some(tex) = fog_texture(win, c, k) else { return };
    let (tw, th, mv) = ((f.w * 0.9).max(f.h * 1.6), f.h * 0.12, if still { 0.0 } else { f.t });
    for layer in 0..2 {
        let lf = layer as f32;
        let off = (mv * f.h * if layer == 1 { -0.006 } else { 0.009 } + lf * tw * 0.37).rem_euclid(tw);
        let y = f.oy + f.hz - th * if layer == 1 { 0.78 } else { 0.6 };
        let mut x = -off;
        while x < f.w {
            let at = gpui::Bounds::new(gpui::point(gpui::px(f.ox + x), gpui::px(y)), gpui::size(gpui::px(tw), gpui::px(th)));
            let _ = win.paint_image(at, at, gpui::Corners::default(), 0.0, tex.clone(), 0, false);
            x += tw;
        }
    }
}

/// pulse.js draw(): the floor's grid runs toward you as fast as the CPU works (and with the bass).
#[allow(clippy::too_many_arguments)]
pub fn grid(win: &mut Window, pal: &Pal, f: &Frame, cpu: f32, bass: f32, moving: bool, noir: bool) {
    if noir {
        return;
    }
    let speed = if moving { 0.08 + cpu / 160.0 + bass * 0.9 } else { 0.0 };
    let off = (f.t * speed).rem_euclid(1.0);
    let (hz, lw) = (f.oy + f.hz, (f.h / 1080.0).max(1.0));
    for k in 0..18 {
        let p = (k as f32 + off) / 18.0;
        let yh = hz + (f.h - f.hz) * p.powf(2.2);
        draw::line(win, &[(f.ox, yh), (f.ox + f.w, yh)], lw, pal.accent, 0.05 + 0.45 * p);
    }
    let cx = f.ox + f.w / 2.0;
    for k in -24i32..=24 {
        draw::line(win, &[(cx + k as f32 * f.w * 0.012, hz), (cx + k as f32 * f.w * 0.11, f.oy + f.h)], lw, pal.accent, 0.18 - k.abs() as f32 * 0.005);
    }
}

/// pulse.js drawLightPath(): a low Sun's glow along the horizon, a pool of light long toward you, a path of glints.
pub fn light_path(win: &mut Window, l: &Light, f: &Frame, still: bool) {
    let t = if still { 0.0 } else { f.t };
    let (x, hz) = (f.ox + l.x, f.oy + f.hz);
    let clip = |y0: f32, y1: f32| gpui::ContentMask { bounds: gpui::Bounds::new(gpui::point(gpui::px(f.ox), gpui::px(y0)), gpui::size(gpui::px(f.w), gpui::px((y1 - y0).max(0.0)))), ..Default::default() };
    if l.sun && l.low > 0.05 {
        win.with_content_mask(Some(clip(f.oy, hz)), |win| crate::scene::radial(win, x, hz, 0.0, f.w * 0.30, 0.22, l.c, &[(0.0, 0.34 * l.a * l.low), (1.0, 0.0)]));
    }
    let (wv, hgt) = (f.h * 0.05, (f.h - f.hz) * 0.9);
    let rr = wv * 2.4;
    win.with_content_mask(Some(clip(hz, f.oy + f.h)), |win| crate::scene::radial(win, x, hz, 0.0, rr, hgt / rr, l.c, &[(0.0, 0.55 * l.a), (0.5, 0.18 * l.a), (1.0, 0.0)]));
    for i in 0..9 {
        let u = (i as f32 + 0.5) / 9.0;
        let yy = hz + hgt * 0.75 * u * u + f.h / 400.0;
        let ww = wv * (0.5 + 2.6 * u) * (0.55 + 0.45 * (t * (0.7 + i as f32 * 0.13) + i as f32 * 2.1).sin());
        rect(win, x - ww / 2.0, yy, ww, (f.h / 700.0).max(1.0), l.c, 0.75 * l.a * (1.0 - u * 0.8));
    }
}


/// pulse.js ivyOn2(): ivy on a building of the city, higher the longer the PC stays on.
pub fn ivy(win: &mut Window, pal: &Pal, f: &Frame, x: f32, y0: f32, hgt: f32, seed: f32, still: bool) {
    if hgt < f.h * 0.004 {
        return;
    }
    let u = f.h / 1080.0;
    let n = (hgt / (f.h * 0.006)).round() as usize;
    let sway = if still { 0.0 } else { (f.t * 0.8 + seed).sin() * u * 0.6 };
    for k in 0..n {
        let q = k as f32 / n.max(1) as f32;
        let (yy, xx) = (y0 - hgt * q, x + (q * 11.0 + seed).sin() * 3.0 * u + sway * q);
        draw::arc_fill(win, xx, yy, 2.2 * u, 1.3 * u, if k % 2 == 1 { 0.6 } else { -0.6 }, 0.0, TAU, mix(pal.ok, pal.fg, (k % 4) as f32 * 0.07), 0.8);
    }
}

/// pulse.js draw(): the skyline, a tower for each CPU core either side of the core, as tall as the core's load (it
/// dances with your music; lower in front of a real city; lower in a focus), its windows lit as the RAM fills; in the
/// 1.8 look a building, not a card: its reflection, a side going back toward the horizon (lit when it looks at the
/// Sun or the Moon), the edge that catches the light, a mast, a tank or the air conditioning on the roof, a lit
/// ground floor; the ivy of your uptime.
#[allow(clippy::too_many_arguments)]
pub fn towers(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, lt: Option<Light>, heights: &mut Vec<f32>, audio: &[f32], inner: f32, polish: bool, focus_k: f32, real_city: bool, noir: bool, still: bool) {
    let (h, hz) = (f.h, f.oy + f.hz);
    let cx = f.ox + f.w / 2.0;
    let (bg, ac, a2, black) = (pal.bg, pal.accent, pal.accent2, [0.0, 0.0, 0.0]);
    let cores: Vec<f32> = list(st, &["sys", "cores"]).iter().filter_map(|c| c.as_f64().map(|v| v as f32)).collect();
    let cpu = num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32;
    let ram = num(st, &["sys", "ram"]).unwrap_or(0.0) as f32;
    let music = !audio.is_empty();
    let nc = if noir { 0 } else { cores.len().clamp(8, 32) };
    let side = nc.div_ceil(2).max(1);
    let (b0, b1) = crate::extra::band(f);
    let outer = (cx - f.ox - b0).min(b1 - (cx - f.ox));
    let slot = ((outer - inner) / side as f32).max(4.0);
    let bw = slot * 0.74;
    heights.resize(nc.max(heights.len()), -1.0);
    let (nk, tk) = (f.sky[0], f.sky[1]);
    let uptime = num(st, &["sys", "uptime"]).unwrap_or(0.0) as f32;
    let ivy_on = get(st, &["ui", "ivy"]).as_bool().unwrap_or(true);
    for i in 0..nc {
        let mut load = if cores.is_empty() { cpu } else { cores[i % cores.len()] };
        if music {
            load = (load * 0.4 + audio[(i / 2).min(audio.len() - 1)] * 0.9).min(100.0); // the city dances
        }
        heights[i] = if heights[i] < 0.0 { load } else { heights[i] + (load - heights[i]) * if music { 0.35 } else { 0.06 } };
        let hh = (h * 0.035 + heights[i] / 100.0 * h * 0.2 * (1.0 - focus_k * 0.7) + ((i * 37) % 11) as f32 / 11.0 * h * 0.03) * if real_city { 0.62 } else { 1.0 };
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
                // the front edge on the side of the Sun or the Moon catches its light: a gradient from the edge inward
                let rw = (bw * 0.16).max(1.5);
                let right = f.ox + l.x >= bx + bw / 2.0;
                let (x0, angle) = if right { (bx + bw - rw * 2.4, 270.0) } else { (bx, 90.0) };
                let g = gpui::linear_gradient(angle, gpui::linear_color_stop(crate::palette::col(l.c, 0.8 * l.a), 0.0), gpui::linear_color_stop(crate::palette::col(l.c, 0.0), 1.0));
                win.paint_quad(gpui::PaintQuad::new(gpui::Bounds::new(gpui::point(gpui::px(x0), gpui::px(hz - hh)), gpui::size(gpui::px(rw * 2.4), gpui::px(hh)))).background(g));
            }
            // on the roof: a parapet, and by building a mast, a tank or the air conditioning
            let (rk, ru) = (((i as u64).wrapping_mul(2654435761) % 4294967296) % 5, (h / 700.0).max(1.0));
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
        for ry in 0..rows {
            for cxi in 0..cols {
                let hsh = ((i as i64 * 7919 + ry as i64 * 104729 + cxi as i64 * 1299709) % 1000) as f32 / 1000.0;
                if hsh < ram / 100.0 * 0.75 {
                    let flick = if (f.t * 0.07 + hsh * 13.0).rem_euclid(1.0) > 0.985 { 0.2 } else { 1.0 };
                    let c = if hsh < 0.12 { a2 } else if hsh < 0.5 { ac } else { pal.warn };
                    let a = ((0.35 + hsh * 0.5) * flick * if polish { 0.8 + 0.45 * nk.max(tk * 0.7) } else { 1.0 }).min(1.0);
                    rect(win, bx + cw * 0.7 + cxi as f32 * cw * 1.7, hz - hh + chh * 2.0 + ry as f32 * chh * 2.2, cw, chh, c, a);
                }
            }
        }
        if heights[i] > 90.0 && (f.t * 2.0).rem_euclid(1.0) < 0.5 {
            rect(win, bx + bw / 2.0 - 1.0, hz - hh - h / 90.0, 2.0, 2.0, pal.crit, 1.0);
        }
        if ivy_on && i % 3 == 0 && uptime > 3600.0 {
            // 1.6: the ivy of your uptime
            ivy(win, pal, f, bx + if s2 > 0.0 { 0.0 } else { bw }, hz, hh * (uptime / 86400.0 / 7.0 * (0.6 + (i % 5) as f32 * 0.1)).min(0.85), i as f32, still);
        }
    }
}

/// pulse.js drawGit() and buildSite(): your repositories as towers beside the core, as tall as this week's commits;
/// windows for the days of work, amber ones flickering for what is not committed; a beacon in the colour of the CI;
/// a new floor lowered onto the roof at every commit, scaffolding while there are changes, a tower crane for every
/// agent working on it, fire on the roof and the fire engine while the CI is red.
#[allow(clippy::too_many_arguments)]
pub fn git(win: &mut Window, cx_app: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, f: &Frame, r_core: f32, flash: &std::collections::HashMap<String, f32>, fail_at: &mut std::collections::HashMap<String, f32>, real_city: bool, still: bool, share: bool) -> Vec<(String, f32, f32)> {
    let repos = list(st, &["git"]);
    let mut out = Vec::new();
    if repos.is_empty() {
        return out;
    }
    let (h, hz, t) = (f.h, f.oy + f.hz, f.t);
    let cx = f.ox + f.w / 2.0;
    let (b0, b1) = crate::extra::band(f);
    let n = repos.len().min(8);
    let ex = r_core * 1.2;
    let side_w = [cx - ex - (f.ox + b0), (f.ox + b1) - cx - ex];
    let half = n.div_ceil(2) as f32;
    let max_w = repos.iter().map(|r| num(r, &["week"]).unwrap_or(0.0) as f32).fold(1.0f32, f32::max);
    let agents = list(st, &["agents"]);
    let yel = mix(pal.warn, [1.0, 210.0 / 255.0, 60.0 / 255.0], 0.5);
    for (i, r) in repos.iter().take(n).enumerate() {
        let (right, k) = (i % 2 == 1, (i >> 1) as f32);
        let slot_w = side_w[right as usize] / half;
        let bw = (slot_w * 0.42).min(h * 0.05);
        let x = if right { cx + ex + (k + 0.5) * slot_w } else { cx - ex - (k + 0.5) * slot_w };
        let week = num(r, &["week"]).unwrap_or(0.0) as f32;
        let hh = h * (0.09 + 0.22 * (1.0 + week).ln() / (1.0 + max_w).ln()) * if real_city { 0.85 } else { 1.0 };
        let name = crate::hub::text(r, &["name"]);
        let fl = (1.0 - (t - flash.get(&name).copied().unwrap_or(-99.0)) / 2.5).max(0.0);
        gradient(win, x - bw / 2.0, hz - hh, bw, hh, mix(mix(pal.bg, pal.accent, 0.22), pal.fg, fl * 0.5), 1.0, mix(pal.bg, [0.0; 3], 0.2), 1.0);
        draw::frame(win, x - bw / 2.0, hz - hh, bw, hh, (h / 700.0).max(1.0), pal.accent, 0.55 + fl * 0.45);
        // windows: one row per day of work; uncommitted changes flicker amber
        let dirty_repo = get(r, &["dirty"]).as_bool().unwrap_or(false) || num(r, &["dirty"]).unwrap_or(0.0) > 0.0;
        let rows = ((hh / (h * 0.018)).floor() as i32).min(14);
        for ry in 0..rows {
            for cxi in 0..3 {
                let on = ((i as i32 * 31 + ry * 7 + cxi * 3) % 5) < 3;
                let dirty = dirty_repo && ry < 2 && (t * 2.0 + cxi as f32).rem_euclid(2.0) < 1.0;
                if !on && !dirty {
                    continue;
                }
                rect(win, x - bw / 2.0 + bw * (0.15 + cxi as f32 * 0.27), hz - hh + h * 0.012 + ry as f32 * h * 0.018, bw * 0.16, h * 0.008,
                    if dirty { pal.warn } else { pal.accent }, if dirty { 0.9 } else { 0.35 + 0.4 * fl + 0.2 * ((ry + cxi) % 2) as f32 });
            }
        }
        // the beacon on top: green CI, red failed (blinking), amber running
        let ci = crate::hub::text(r, &["ci"]);
        let bc = match ci.as_str() { "ok" => pal.ok, "fail" => pal.crit, "run" => pal.warn, _ => pal.accent2 };
        let blink = if ci == "fail" || ci == "run" { (t * 2.0).rem_euclid(1.0) < 0.5 } else { true };
        rect(win, x - h / 600.0, hz - hh - h * 0.022, h / 300.0, h * 0.022, bc, if blink { 1.0 } else { 0.25 });
        crate::scene::radial(win, x, hz - hh - h * 0.022, 0.0, h * 0.025, 1.0, bc, &[(0.0, if blink { 0.6 } else { 0.1 }), (1.0, 0.0)]);
        if !name.is_empty() && !share {
            let label = if week > 0.0 { format!("{name} · {week:.0}") } else { name.clone() };
            draw::text(win, cx_app, font, &label, x, hz - hh - h * 0.03, (h * 0.013).round(), 600.0, pal.fg, 0.55 + fl * 0.45, 0);
        }
        out.push((name.clone(), x, hz - hh));
        // ---- buildSite()
        let top = hz - hh;
        let key = if name.is_empty() { format!("#{i}") } else { name.clone() };
        if fl > 0.0 && fl < 1.0 {
            // the new floor, lowered onto the roof
            let drop = fl.powi(2) * h * 0.07;
            rect(win, x - bw / 2.0, top - h * 0.012 - drop, bw, h * 0.012, mix(pal.bg, pal.accent, 0.4), 0.95);
            draw::line(win, &[(x, top - h * 0.012 - drop), (x, top - h * 0.09)], 1.0, yel, 0.8);
        }
        if dirty_repo {
            // scaffolding on the top floors
            let sh = hh.min(h * 0.06);
            let (x0, x1) = (x - bw / 2.0 - bw * 0.12, x + bw / 2.0 + bw * 0.12);
            let lw = (h / 1100.0).max(1.0);
            draw::frame(win, x0, top, x1 - x0, sh, lw, yel, 0.5);
            for kk in 0..3 {
                let yy = top + sh * kk as f32 / 3.0;
                draw::line(win, &[(x0, yy), (x1, yy + sh / 3.0)], lw, yel, 0.5);
                draw::line(win, &[(x1, yy), (x0, yy + sh / 3.0)], lw, yel, 0.5);
            }
        }
        let hay = |a: &Value| format!("{} {}", crate::hub::text(a, &["name"]), crate::hub::text(a, &["text"])).to_lowercase();
        let mut work: Vec<&Value> = agents.iter().filter(|a| crate::hub::text(a, &["state"]) == "working" && !name.is_empty() && hay(a).contains(&name.to_lowercase())).collect();
        if work.is_empty() && i == 0 {
            work = agents.iter().filter(|a| crate::hub::text(a, &["state"]) == "working" && !repos.iter().any(|g2| { let n2 = crate::hub::text(g2, &["name"]); !n2.is_empty() && hay(a).contains(&n2.to_lowercase()) })).collect();
        }
        for (j, a) in work.iter().take(2).enumerate() {
            // a tower crane: the mast beside the building, the jib over it
            let side = if j == 1 { -1.0 } else { 1.0 };
            let (mx, mtop, jib) = (x + side * (bw / 2.0 + bw * 0.35), top - h * 0.07, bw * 1.3);
            let lw = (h / 650.0).max(1.5);
            draw::line(win, &[(mx, hz), (mx, mtop)], lw, yel, 0.9);
            draw::line(win, &[(mx + side * bw * 0.3, mtop), (mx - side * jib, mtop)], lw, yel, 0.9);
            let mut y = hz;
            let lw2 = (h / 1400.0).max(1.0);
            while y > mtop {
                draw::line(win, &[(mx - h * 0.004, y), (mx + h * 0.004, y - h * 0.007)], lw2, yel, 0.9);
                y -= h * 0.014;
            }
            let sw = if still { 0.0 } else { (t * 1.3 + j as f32).sin() * bw * 0.25 };
            let hx = mx - side * jib * 0.7 + sw;
            let hy = mtop + h * 0.03 + if still { 0.0 } else { (t * 0.7 + j as f32).sin() * h * 0.01 };
            draw::line(win, &[(mx - side * jib * 0.7 + sw * 0.2, mtop), (hx, hy)], lw2, yel, 0.9);
            rect(win, hx - h * 0.004, hy, h * 0.008, h * 0.006, yel, 0.95);
            rect(win, mx - h * 0.006, mtop + h * 0.002, h * 0.012, h * 0.01, pal.bg, 0.9); // the cab
            if !share {
                let initial: String = crate::hub::text(a, &["name"]).chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_else(|| "?".into());
                draw::text(win, cx_app, font, &initial, mx, mtop + h * 0.0095, (h * 0.008).round().max(6.0), 700.0, yel, 1.0, 0);
            }
        }
        if ci == "fail" {
            // fire on the roof, and the fire engine on its way
            let since = *fail_at.entry(key.clone()).or_insert(t);
            for fi in 0..6 {
                let fh = h * 0.03 * if still { 0.8 } else { 0.6 + 0.4 * (t * 7.0 + fi as f32 * 1.7).sin().abs() };
                let fx = x - bw / 2.0 + bw * (fi as f32 + 0.5) / 6.0;
                // a flame: quadraticCurveTo through (fx, top - fh * 1.2), filled with a gradient from crit at the roof
                // to transparent warn at top - fh; drawn in slices
                let b = bw / 12.0;
                let half = |dy: f32| b * (1.0 - 4.0 * (dy / (2.4 * fh)).min(0.25)).max(0.0).sqrt();
                let slices = 6;
                for sl in 0..slices {
                    let (d0, d1) = (0.6 * fh * sl as f32 / slices as f32, 0.6 * fh * (sl + 1) as f32 / slices as f32);
                    let k = 1.0 - (d0 + d1) / 2.0 / fh;
                    draw::poly(win, &[(fx - half(d0), top - d0), (fx + half(d0), top - d0), (fx + half(d1), top - d1), (fx - half(d1), top - d1)], mix(pal.warn, pal.crit, k), 0.85 * k);
                }
            }
            let go = ((t - since) / 6.0).min(1.0);
            let side2 = if x > cx { 1.0 } else { -1.0 };
            let (tx, ty, tw) = (x + side2 * (f.w * 0.35 * (1.0 - go) + bw * 0.9), hz + h * 0.03, h * 0.05);
            rect(win, tx - tw / 2.0, ty - h * 0.016, tw, h * 0.016, pal.crit, 0.95);
            rect(win, tx - side2 * tw * 0.35 - h * 0.006, ty - h * 0.024, h * 0.012, h * 0.008, mix(pal.crit, pal.fg, 0.4), 1.0);
            for uu in [-0.3f32, 0.3] {
                draw::arc_fill(win, tx + uu * tw, ty, h * 0.005, h * 0.005, 0.0, 0.0, TAU, pal.bg, 1.0);
            }
            let bl2 = still || (t * 3.0).rem_euclid(1.0) < 0.5;
            rect(win, tx - h * 0.003, ty - h * 0.02, h * 0.006, h * 0.004, if bl2 { [80.0 / 255.0, 140.0 / 255.0, 1.0] } else { pal.crit }, 1.0);
            if go >= 1.0 {
                // the water
                let (sx0, sy0, cx1, cy1, ex1, ey1) = (tx - side2 * tw * 0.4, ty - h * 0.02, (tx + x) / 2.0, top - h * 0.08, x + side2 * bw * 0.2, top - h * 0.005);
                let pts: Vec<(f32, f32)> = (0..=16).map(|q| { let u = q as f32 / 16.0; ((1.0 - u) * (1.0 - u) * sx0 + 2.0 * (1.0 - u) * u * cx1 + u * u * ex1, (1.0 - u) * (1.0 - u) * sy0 + 2.0 * (1.0 - u) * u * cy1 + u * u * ey1) }).collect();
                draw::line(win, &pts, (h / 700.0).max(1.5), [150.0 / 255.0, 200.0 / 255.0, 1.0], 0.55);
            }
        } else {
            fail_at.remove(&key);
        }
    }
    out
}
