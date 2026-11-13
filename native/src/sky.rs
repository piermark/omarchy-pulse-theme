//! What crosses the sky of Pulse 1.8, translated from pulse.js: the clouds with a body (cloudSprites, drawClouds) and
//! the planes that are really over you (drawPlanes), each where it is in your sky, moving as it moves, with a tag
//! you can read: the flight, its aircraft, its route, how high, how fast and where it is heading.

use std::f32::consts::{PI, TAU};
use crate::backdrop::{mix as mixi, Rnd};
use crate::draw;
use crate::hub::{get, list, num, text};
use crate::palette::{mix, Pal, C};
use crate::scene::rect;
use gpui::{point, px, size, Bounds, Window};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tiny_skia::{BlendMode, Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, RadialGradient, Rect, SpreadMode, Transform};

type Rgb = [f32; 3];
fn c255(c: C) -> Rgb {
    [(c[0] * 255.0).round(), (c[1] * 255.0).round(), (c[2] * 255.0).round()]
}
fn col(c: Rgb, a: f32) -> Color {
    Color::from_rgba((c[0] / 255.0).clamp(0.0, 1.0), (c[1] / 255.0).clamp(0.0, 1.0), (c[2] / 255.0).clamp(0.0, 1.0), a.clamp(0.0, 1.0)).unwrap_or(Color::BLACK)
}

#[derive(Default)]
pub struct Sky {
    cloud_key: String,
    sprites: Vec<Arc<gpui::RenderImage>>,
    clouds: Vec<[f32; 4]>,
    seed: u64,
    /// each plane by its id: where it is on the screen now, and its contrail
    planes: HashMap<String, (f32, f32, Vec<(f32, f32)>)>,
    /// each plane by its id: the last position the hub gave (azimuth, elevation) and when it came
    fixes: HashMap<String, (f32, f32, f32)>,
    /// pulse-native: on Mars the flights are drawn as flying saucers (the same flights, the same tags)
    pub ufo: bool,
}

/// pulse.js darkness(): how dark the sky is (0 by day, 1 at night), how much of it the clouds leave, the cloud cover.
pub fn darkness(st: &Value) -> (f32, f32, f32) {
    // without the real Sun (no location for the night sky), as pulse.js: day or night from the weather's sunrise
    // and sunset (night only when the hub says so; 0.16 took night whenever the Sun was missing)
    let se = num(st, &["night", "sun", "el"]).map(|v| v as f32).unwrap_or_else(|| match num(st, &["weather", "day"]) {
        Some(d) if d > 0.0 => 20.0,
        Some(_) => -20.0,
        None => 20.0,
    });
    let mut d = if se > 0.0 { 0.0 } else if se > -6.0 { 0.25 * (-se / 6.0) } else if se > -12.0 { 0.25 + 0.75 * ((-se - 6.0) / 6.0) } else { 1.0 };
    let ec = get(st, &["season", "eclipse"]);
    if text(ec, &["kind"]) == "solar" && se > -1.0 {
        d = d.max((num(ec, &["obsc"]).unwrap_or(0.0) as f32).max(0.0).powi(8)); // under a total eclipse the day turns to night
    }
    let cl = match num(st, &["weather", "cloud"]) {
        Some(c) => c as f32 / 100.0,
        None => match crate::ui::wx_kind(num(st, &["weather", "code"]).unwrap_or(0.0) as i32) { "clear" => 0.0, "part" => 0.4, "fog" => 0.7, _ => 0.85 },
    };
    (d, d * (1.0 - 0.85 * cl), cl)
}

impl Sky {
    /// pulse.js cloudSprites(): four clouds, each painted once as a picture: round masses lit from the side of the Sun
    /// or the Moon, darker underneath, a flat base. Made at twice the web page's size, with smaller puffs along the
    /// top (the cauliflower of a real cumulus), the edge toward the light catching it, and at night the city's light
    /// on their bellies. `alpha` is how solid they are now (it is part of the picture).
    #[allow(clippy::too_many_arguments)]
    fn sprites(&mut self, win: &mut Window, base: Rgb, light: Option<(Rgb, bool, bool)>, storm: bool, fg: Rgb, alpha: f32, under: (Rgb, f32)) {
        let lc = match light { Some((c, sun, _)) => mixi(base, c, if sun { 0.75 } else { 0.45 }), None => mixi(base, fg, 0.25) };
        let sh = mixi(base, [0.0; 3], if storm { 0.5 } else { 0.3 });
        let ls: f32 = match light { Some((_, _, left)) => if left { -1.0 } else { 1.0 }, None => 0.0 };
        let uq = (under.1 * 10.0).round() / 10.0;
        let key = format!("{base:?}|{lc:?}|{ls}|{storm}|{:.2}|{:?}|{uq}", alpha, under.0);
        if key == self.cloud_key && !self.sprites.is_empty() {
            return;
        }
        self.cloud_key = key;
        for old in self.sprites.drain(..) {
            let _ = win.drop_image(old);
        }
        const S: f32 = 2.0; // the web page's 320×120, at twice the pixels
        let ts = Transform::from_scale(S, S);
        let puff = |pm: &mut Pixmap, x: f32, y: f32, r: f32, inner: Rgb, mid: Rgb, outer: Rgb, a0: f32, a1: f32| {
            let stops = vec![GradientStop::new(0.0, col(inner, a0)), GradientStop::new(0.55, col(mid, a1)), GradientStop::new(1.0, col(outer, 0.0))];
            if let (Some(shd), Some(path)) = (RadialGradient::new((x + ls * r * 0.35, y - r * 0.4).into(), (x, y).into(), r, stops, SpreadMode::Pad, Transform::identity()), PathBuilder::from_circle(x, y, r)) {
                pm.fill_path(&path, &Paint { shader: shd, anti_alias: true, ..Default::default() }, FillRule::Winding, ts, None);
            }
        };
        for v in 0..4u64 {
            let Some(mut pm) = Pixmap::new((320.0 * S) as u32, (120.0 * S) as u32) else { return };
            let mut rn = Rnd::new(900 + v * 17);
            let n = 9 + v * 2;
            let mut tops = Vec::new();
            for i in 0..n {
                // pulse.js: the masses along an arch
                let u = i as f32 / (n - 1) as f32;
                let px0 = 40.0 + u * 240.0 + (rn.next() - 0.5) * 24.0;
                let env = (std::f32::consts::PI * (0.08 + 0.84 * u)).sin();
                let r = 16.0 + env * (20.0 + rn.next() * 14.0);
                let py0 = 78.0 - env * (10.0 + rn.next() * 16.0);
                puff(&mut pm, px0, py0, r, lc, base, sh, 0.95, 0.85);
                tops.push((px0, py0, r));
            }
            // the cauliflower: smaller, brighter puffs on the upper edge of every mass
            let mut rd = Rnd::new(7000 + v * 31);
            for &(px0, py0, r) in &tops {
                for _ in 0..3 {
                    let a = -std::f32::consts::PI * (0.15 + rd.next() * 0.7);
                    let (x, y) = (px0 + a.cos() * r * 0.62, py0 + a.sin() * r * 0.62);
                    puff(&mut pm, x, y, r * (0.32 + rd.next() * 0.2), mixi(lc, fg, 0.12), lc, base, 0.7, 0.45);
                }
            }
            // the edge toward the light catches it
            if ls != 0.0 {
                let (lx, ly) = (160.0 + ls * 150.0, 20.0);
                let stops = vec![GradientStop::new(0.0, col(mixi(lc, fg, 0.3), 0.45)), GradientStop::new(1.0, col(lc, 0.0))];
                if let (Some(shd), Some(rc)) = (RadialGradient::new((lx, ly).into(), (lx, ly).into(), 170.0, stops, SpreadMode::Pad, Transform::identity()), Rect::from_xywh(0.0, 0.0, 320.0, 120.0)) {
                    pm.fill_rect(rc, &Paint { shader: shd, blend_mode: BlendMode::SourceAtop, anti_alias: false, ..Default::default() }, ts, None);
                }
            }
            // at night the city lights their bellies from below
            if under.1 > 0.02 {
                let g = vec![GradientStop::new(0.0, col(under.0, 0.0)), GradientStop::new(1.0, col(under.0, under.1))];
                if let (Some(shd), Some(rc)) = (LinearGradient::new((0.0, 50.0).into(), (0.0, 100.0).into(), g, SpreadMode::Pad, Transform::identity()), Rect::from_xywh(0.0, 40.0, 320.0, 80.0)) {
                    pm.fill_rect(rc, &Paint { shader: shd, blend_mode: BlendMode::SourceAtop, anti_alias: false, ..Default::default() }, ts, None);
                }
            }
            // a flat, darker base (pulse.js)
            let fl = vec![GradientStop::new(0.0, col(sh, 0.0)), GradientStop::new(0.5, col(sh, 0.5)), GradientStop::new(1.0, col(sh, 0.0))];
            if let (Some(shd), Some(r)) = (LinearGradient::new((0.0, 70.0).into(), (0.0, 104.0).into(), fl, SpreadMode::Pad, Transform::identity()), Rect::from_xywh(0.0, 70.0, 320.0, 40.0)) {
                pm.fill_rect(r, &Paint { shader: shd, blend_mode: BlendMode::SourceAtop, anti_alias: false, ..Default::default() }, ts, None);
            }
            let cut = vec![GradientStop::new(0.0, Color::from_rgba8(0, 0, 0, 0)), GradientStop::new(1.0, Color::from_rgba8(0, 0, 0, 255))];
            if let (Some(shd), Some(r)) = (LinearGradient::new((0.0, 88.0).into(), (0.0, 112.0).into(), cut, SpreadMode::Pad, Transform::identity()), Rect::from_xywh(0.0, 88.0, 320.0, 32.0)) {
                pm.fill_rect(r, &Paint { shader: shd, blend_mode: BlendMode::DestinationOut, anti_alias: false, ..Default::default() }, ts, None);
            }
            // premultiplied RGBA to the straight BGRA the GPU takes, with the clouds' alpha in it
            let (pw, ph) = (pm.width(), pm.height());
            let mut px4 = pm.take();
            for p in px4.chunks_exact_mut(4) {
                let a = p[3] as f32 / 255.0;
                if a > 0.0 {
                    let (r, g, b) = ((p[0] as f32 / a).min(255.0), (p[1] as f32 / a).min(255.0), (p[2] as f32 / a).min(255.0));
                    p[0] = b as u8;
                    p[1] = g as u8;
                    p[2] = r as u8;
                }
                p[3] = (p[3] as f32 * alpha.clamp(0.0, 1.0)) as u8;
            }
            if let Some(im) = image::RgbaImage::from_raw(pw, ph, px4) {
                self.sprites.push(Arc::new(gpui::RenderImage::new(vec![image::Frame::new(im)])));
            }
        }
    }

    /// pulse.js drawClouds(): as many as the cover says, drifting with your wind.
    #[allow(clippy::too_many_arguments)]
    pub fn clouds(&mut self, win: &mut Window, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, light: Option<(C, bool, f32)>, frames: f32) {
        let (d, _, cov) = darkness(st);
        if cov < 0.08 {
            return;
        }
        let n = (4.0 + cov * 14.0).round() as usize;
        while self.clouds.len() < n {
            // Math.random() on the web page: any place will do
            self.seed = (self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) | 1;
            let mut r = Rnd::new(self.seed % 2147483646 + 1);
            self.clouds.push([r.next() * w * 1.2 - w * 0.1, h * (0.06 + r.next() * 0.42), 0.6 + r.next() * 0.9, r.next() * 10.0]);
        }
        let storm = crate::ui::wx_kind(num(st, &["weather", "code"]).unwrap_or(0.0) as i32) == "storm";
        let (bg, fg) = (c255(pal.bg), c255(pal.fg));
        let base = if storm { mixi(bg, [40.0, 40.0, 52.0], 0.5) } else { mixi(bg, fg, if d > 0.5 { 0.12 } else { 0.55 }) };
        // pulse.js windFrom(): from the west (270) -> negative -> to the left
        let unit = if get(st, &["weather", "windUnit"]).as_str() == Some("mph") { 1.609 } else { 1.0 };
        let kmh = unit * (num(st, &["weather", "wind"]).unwrap_or(0.0) as f32).max(num(st, &["weather", "gust"]).unwrap_or(0.0) as f32 * 0.8);
        let wind_x = (num(st, &["weather", "windDir"]).unwrap_or(270.0) as f32).to_radians().sin() * (kmh / 60.0).min(1.0);
        let a = (if storm { 0.55 } else { 0.22 + 0.25 * cov }) * (cov * 1.4).min(1.0);
        let under = (mixi(c255(pal.accent2), c255(pal.warn), 0.25), 0.35 * d);
        self.sprites(win, base, light.map(|l| (c255(l.0), l.1, l.2 < w / 2.0)), storm, fg, (a * 1.9).min(1.0), under);
        for c in self.clouds.iter_mut().take(n) {
            let (cw, ch) = (w * 0.14 * c[2], h * 0.045 * c[2]);
            // the web page moves them a step a frame at 20 frames a second: the same speed here at any frame rate. A
            // breath of wind against the drift would stop them: they never stand still
            let mut step = (if wind_x == 0.0 { -0.05 } else { wind_x }) * h * 0.0012 * c[2] + 0.02 * h / 1080.0;
            let least = h * 0.00006 * c[2];
            if step.abs() < least {
                step = if step < 0.0 { -least } else { least };
            }
            c[0] += step * frames;
            if c[0] > w + cw {
                c[0] = -cw;
            }
            if c[0] < -cw * 1.2 {
                c[0] = w + cw;
            }
            if let Some(im) = self.sprites.get(((c[3] * 7.0) as usize) % 4) {
                let at = Bounds::new(point(px(ox + c[0] - cw * 0.72), px(oy + c[1] - ch * 1.5)), size(px(cw * 1.44), px(ch * 2.6)));
                let _ = win.paint_image(at, at, gpui::Corners::default(), 0.0, im.clone(), 0, false);
            }
        }
    }

    /// pulse.js drawPlanes(): you look south, east on the left. Each plane where it really is, a contrail behind it,
    /// red and green on its wings, a white strobe, and its tag clear of the panels and of the other tags.
    #[allow(clippy::too_many_arguments)]
    pub fn planes(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, frames: f32, ui_boxes: &[[f32; 4]], polished: bool) {
        let hz = h * 0.745;
        let planes = list(st, &["sky"]);
        let share = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
        let (mut seen, mut boxes, mut shown): (Vec<String>, Vec<[f32; 4]>, usize) = (Vec::new(), Vec::new(), 0);
        let sphere_box = { let r = h * 0.17; [w / 2.0 - r, h * 0.745 - r * 2.0, r * 2.0, r * 2.0] };
        let hits = |bx: &[f32; 4], lst: &[[f32; 4]]| lst.iter().any(|b| !(bx[0] > b[0] + b[2] || bx[0] + bx[2] < b[0] || bx[1] > b[1] + b[3] || bx[1] + bx[3] < b[1]));
        let ease = 1.0 - (1.0f32 - 0.03).powf(frames.max(0.05));
        let mut fixed: Vec<String> = Vec::new();
        for (i, p) in planes.iter().enumerate() {
            let (az0, el0) = (num(p, &["az"]).unwrap_or(0.0) as f32, num(p, &["el"]).unwrap_or(0.0) as f32);
            // the hub hears the planes every 15 seconds: between two fixes each one goes on flying, at its own speed
            // and on its own heading, from where it was (so it glides as in the real sky instead of standing still)
            let pid = text(p, &["id"]);
            fixed.push(pid.clone());
            let fix = self.fixes.entry(pid).or_insert((az0, el0, t));
            if fix.0 != az0 || fix.1 != el0 {
                *fix = (az0, el0, t);
            }
            let since = (t - fix.2).clamp(0.0, 40.0);
            let (az, el) = {
                let hkm = num(p, &["ft"]).unwrap_or(0.0) as f32 * 0.0003048;
                let kms = num(p, &["kt"]).unwrap_or(0.0) as f32 * 1.852 / 3600.0;
                let trk = num(p, &["track"]).unwrap_or(0.0) as f32;
                if hkm > 0.05 && kms > 0.0 && el0 < 89.5 && since > 0.0 {
                    let g = hkm / el0.max(0.5).to_radians().tan();
                    let (mut x, mut y) = (g * az0.to_radians().sin(), g * az0.to_radians().cos());
                    x += kms * since * trk.to_radians().sin();
                    y += kms * since * trk.to_radians().cos();
                    let g2 = (x * x + y * y).sqrt().max(1e-3);
                    (x.atan2(y).to_degrees().rem_euclid(360.0), hkm.atan2(g2).to_degrees())
                } else {
                    (az0, el0)
                }
            };
            // a plane behind you (north) is not in this sky, unless it is almost overhead: then it flies near the top
            let (front, over) = ((75.0..=285.0).contains(&az), el >= 62.0);
            if !front && !over {
                continue;
            }
            shown += 1;
            if shown > 6 {
                break;
            }
            let id = { let s = text(p, &["id"]); if s.is_empty() { format!("#{i}") } else { s } };
            seen.push(id.clone());
            let mut tx = if front { w / 2.0 + (az - 180.0) / 90.0 * w * 0.44 } else { w / 2.0 - az.to_radians().sin() * w * 0.44 * (90.0 - el) / 28.0 };
            // the sky above the towers: most planes are 10-40° up, so the low part of the sky gets more room
            let ty = hz * 0.93 - (el.max(0.0) / 90.0).sqrt() * hz * 0.85;
            // a plane where a panel is would fly unseen under it: it is drawn just beside the panel, toward the middle
            let m = h * 0.04;
            for _ in 0..4 {
                let Some(b) = ui_boxes.iter().find(|b| tx > b[0] - m && tx < b[0] + b[2] + m && ty > b[1] - m && ty < b[1] + b[3] + m) else { break };
                tx = if b[0] + b[2] / 2.0 > w / 2.0 { b[0] - m } else { b[0] + b[2] + m };
            }
            let q = self.planes.entry(id).or_insert((tx, ty, Vec::new()));
            q.0 += (tx - q.0) * ease;
            q.1 += (ty - q.1) * ease;
            let s2 = h * (0.009 + el / 90.0 * 0.008) * if polished { 1.25 } else { 1.0 };
            let track = num(p, &["track"]).unwrap_or(0.0) as f32;
            let mut dir: f32 = if track.to_radians().sin() > 0.0 { -1.0 } else { 1.0 }; // heading east flies to the left
            // where it came from: a contrail that fades behind it
            // a jump (a new fix far from where it was going, a panel it is moved beside): its contrail starts again
            // instead of a line across the sky
            if q.2.last().map(|l| (l.0 - q.0).abs() + (l.1 - q.1).abs() > s2 * 6.0).unwrap_or(false) {
                q.2.clear();
            }
            if q.2.last().map(|l| (l.0 - q.0).abs() + (l.1 - q.1).abs() > 2.0).unwrap_or(true) {
                q.2.push((q.0, q.1));
                if q.2.len() > 70 {
                    q.2.remove(0);
                }
            }
            let n = q.2.len();
            if self.ufo {
                // a saucer leaves no contrail: a faint shimmer behind it instead
                for k in (1..n).step_by(4).rev().take(8) {
                    let kf = k as f32 / n as f32;
                    draw::hot(win, ox + q.2[k].0, oy + q.2[k].1, s2 * (0.6 + kf), [0.55, 1.0, 0.85], 0.10 * kf);
                }
            }
            // in seven pieces, each one shape (pulse.js strokes 70: as wide and as bright, fading behind)
            for bin in 0..if self.ufo { 0 } else { 7 } {
                let (k0, k1) = ((bin * n / 7).max(1), ((bin + 1) * n / 7).min(n));
                if k1 <= k0 {
                    continue;
                }
                let kf = (k0 + k1) as f32 / 2.0 / n as f32;
                let segs: Vec<_> = (k0..k1)
                    .filter(|&k| (q.2[k].0 - q.2[k - 1].0).abs() + (q.2[k].1 - q.2[k - 1].1).abs() <= s2 * 6.0)
                    .map(|k| ((ox + q.2[k - 1].0, oy + q.2[k - 1].1), (ox + q.2[k].0, oy + q.2[k].1)))
                    .collect();
                draw::lines(win, &segs, (s2 * 0.35 * kf).max(1.0), pal.fg, 0.28 * kf);
            }
            // where it really moves on the screen wins over its heading (a turn, a plane almost overhead)
            if n > 8 {
                let mdx = q.0 - q.2[n - 9].0;
                if mdx.abs() > s2 * 0.6 {
                    dir = if mdx > 0.0 { 1.0 } else { -1.0 };
                }
            }
            let (fx, fy, sx) = (ox + q.0, oy + q.1, dir);
            if self.ufo {
                // pulse-native: a flying saucer, seen a little from below: it wobbles, its rim lights run round
                let wob = (t * 1.7 + i as f32).sin();
                let (fy, r) = (fy + wob * s2 * 0.5, s2 * 2.4);
                let tilt = wob * 0.08 + sx * 0.06;
                draw::hot(win, fx, fy, r * 1.6, [0.45, 1.0, 0.8], 0.18);
                draw::arc_fill(win, fx, fy - r * 0.18, r * 0.42, r * 0.36, tilt, PI, TAU, [0.55, 0.95, 1.0], 0.75); // the glass dome
                draw::arc_fill(win, fx - r * 0.12, fy - r * 0.32, r * 0.12, r * 0.07, tilt, 0.0, TAU, [1.0, 1.0, 1.0], 0.5);
                draw::arc_fill(win, fx, fy, r, r * 0.26, tilt, 0.0, TAU, [0.62, 0.66, 0.72], 0.97); // the hull
                draw::arc_fill(win, fx, fy - r * 0.05, r * 0.9, r * 0.12, tilt, PI, TAU, [0.86, 0.9, 0.95], 0.6); // its shine
                draw::arc_fill(win, fx, fy + r * 0.12, r * 0.55, r * 0.13, tilt, 0.0, PI, [0.25, 0.27, 0.32], 0.95); // the belly
                for k in 0..7 {
                    let a = k as f32 / 7.0 * TAU + t * 2.4;
                    if a.sin() < -0.2 {
                        continue; // behind the hull
                    }
                    let (lx, ly) = (fx + a.cos() * r * 0.82, fy + a.sin() * r * 0.17 + a.cos() * tilt * r);
                    let c = [[1.0, 0.35, 0.35], [0.4, 1.0, 0.6], [0.4, 0.75, 1.0], [1.0, 0.85, 0.3]][(k + (t * 3.0) as usize) % 4];
                    draw::hot(win, lx, ly, s2 * 0.9, c, 0.95);
                }
                if (t * 0.21 + i as f32 * 0.37).rem_euclid(1.0) < 0.12 {
                    // now and then a beam down to the sand
                    draw::poly(win, &[(fx - r * 0.3, fy + r * 0.2), (fx + r * 0.3, fy + r * 0.2), (fx + r * 0.9, fy + r * 4.5), (fx - r * 0.9, fy + r * 4.5)], [0.55, 1.0, 0.85], 0.12);
                }
            } else {
            // the plane seen from below: fuselage, swept wings, tail, the nose where it goes
            let body = mix(pal.fg, pal.bg, 0.25);
            draw::poly(win, &[(fx + sx * s2 * 2.6, fy), (fx - sx * s2 * 2.2, fy - s2 * 0.28), (fx - sx * s2 * 2.2, fy + s2 * 0.28)], body, 0.95);
            draw::poly(win, &[(fx + sx * s2 * 0.5, fy), (fx - sx * s2 * 0.6, fy - s2 * 2.0), (fx - sx * s2 * 0.2, fy), (fx - sx * s2 * 0.6, fy + s2 * 2.0)], body, 0.95);
            draw::poly(win, &[(fx - sx * s2 * 1.6, fy), (fx - sx * s2 * 2.2, fy - s2 * 0.9), (fx - sx * s2 * 2.0, fy), (fx - sx * s2 * 2.2, fy + s2 * 0.9)], body, 0.95);
            // red on its left wing, green on its right, as seen from below
            for (side, c) in [(-1.0f32, pal.crit), (1.0, pal.ok)] {
                let (lx, ly) = (fx - sx * s2 * 0.6, fy + if side < 0.0 { 1.0 } else { -1.0 } * s2 * 2.0 * sx);
                draw::hot(win, lx, ly, s2 * 1.3, c, 1.0);
            }
            if (t + i as f32 * 0.37).rem_euclid(1.2) < 0.08 {
                draw::hot(win, fx, fy, s2 * 2.6, pal.fg, 0.9); // the white strobe, a short flash
            }
            }
            let flight = text(p, &["flight"]);
            if share || flight.is_empty() {
                continue;
            }
            // 1.8: a tag you can read: the flight, its route, how high and how fast
            let (f1, f2, pad) = ((h * 0.0148).round(), (h * 0.0122).round(), h * 0.008);
            let card = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"][((track.rem_euclid(360.0) / 45.0).round() as usize) % 8];
            let (from, to, fc, tc) = (text(p, &["from"]), text(p, &["to"]), text(p, &["fromCity"]), text(p, &["toCity"]));
            let l2 = if !from.is_empty() && !to.is_empty() { format!("{from} → {to}{}", if !fc.is_empty() && !tc.is_empty() { format!("   {fc} → {tc}") } else { String::new() }) } else { String::new() };
            let ft = (num(p, &["ft"]).unwrap_or(0.0) / 100.0).round() as i64 * 100;
            let ft_s = { let s = ft.to_string(); let mut o = String::new(); for (k, ch) in s.chars().enumerate() { if k > 0 && (s.len() - k) % 3 == 0 { o.push(',') } o.push(ch) } o };
            let kt = num(p, &["kt"]).filter(|v| *v > 0.0).map(|v| format!(" · {v:.0} kt")).unwrap_or_default();
            let l3 = format!("{ft_s} ft{kt} · {card}");
            let ty_s = text(p, &["type"]);
            let w1 = draw::measure(win, font, &flight, f1, 700.0);
            let wt = if ty_s.is_empty() { 0.0 } else { draw::measure(win, font, &format!("  {ty_s}"), f2, 500.0) };
            let (w2, w3) = (draw::measure(win, font, &l2, f2, 500.0), draw::measure(win, font, &l3, f2, 500.0));
            let lw = (w1 + wt).max(w2).max(w3) + pad * 2.0 + h * 0.004;
            let lh = pad * 1.6 + f1 + if l2.is_empty() { 0.0 } else { f2 * 1.45 } + f2 * 1.45;
            let (qx, qy) = (q.0, q.1);
            let cands = [[qx + s2 * 2.4, qy - s2 * 2.6 - h * 0.013, lw, lh], [qx - s2 * 2.4 - lw, qy - s2 * 2.6 - h * 0.013, lw, lh], [qx + s2 * 2.4, qy + s2 * 2.2, lw, lh], [qx - s2 * 2.4 - lw, qy + s2 * 2.2, lw, lh]];
            // the best place: clear of other labels and of the panels; else clear of other labels only
            let mut all = ui_boxes.to_vec();
            all.push(sphere_box);
            let Some(lb) = cands.iter().find(|c| !hits(c, &boxes) && !hits(c, &all)).or_else(|| cands.iter().find(|c| !hits(c, &boxes))).copied() else { continue };
            boxes.push(lb);
            let (ax, ay) = (if lb[0] > qx { lb[0] } else { lb[0] + lw }, lb[1] + lh / 2.0); // a thin line from the plane to its tag
            draw::line(win, &[(ox + qx + if ax > qx { 1.0 } else { -1.0 } * s2 * 1.2, oy + qy), (ox + ax, oy + ay)], (h / 1300.0).max(1.0), pal.fg, 0.35);
            rect(win, ox + lb[0], oy + lb[1], lw, lh, pal.bg, 0.78);
            rect(win, ox + lb[0], oy + lb[1], (h / 500.0).max(2.0), lh, pal.accent, 0.9);
            let tx0 = ox + lb[0] + pad + h * 0.004;
            let mut ty0 = oy + lb[1] + pad * 0.8 + f1 * 0.82;
            draw::text(win, cx, font, &flight, tx0, ty0, f1, 700.0, pal.fg, 0.97, -1);
            if !ty_s.is_empty() {
                draw::text(win, cx, font, &format!("  {ty_s}"), tx0 + w1, ty0, f2, 500.0, pal.fg, 0.6, -1);
            }
            if !l2.is_empty() {
                ty0 += f2 * 1.45;
                draw::text(win, cx, font, &l2, tx0, ty0, f2, 500.0, pal.accent2, 0.98, -1);
            }
            ty0 += f2 * 1.45;
            draw::text(win, cx, font, &l3, tx0, ty0, f2, 500.0, pal.fg, 0.72, -1);
        }
        self.planes.retain(|k, _| seen.contains(k));
        self.fixes.retain(|k, _| fixed.contains(k));
    }
}
