//! What never moves, painted once (pulse.js paintStatic and what it calls: paintSkyline, drawCity, paintAir,
//! paintMirror): the sky mixed by the Sun's real height, the floor, the far ridge, a far city in three rows, the
//! skyline of your own city with its landmarks and their lit windows, the air at the horizon, the wet floor that
//! mirrors it all. It is painted in the screen's own pixels, kept as a picture, and painted again only when the
//! light, the theme, the city or the screen change.

use crate::cities_data::{city as city_of, Landmark};
use crate::hub::{get, num, text};
use crate::palette::Pal;
use serde_json::Value;
use tiny_skia::{Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, RadialGradient, Rect, SpreadMode, Stroke, Transform};

type Rgb = [f32; 3];

/// pulse.js rnd(): the same numbers, so the same buildings.
pub struct Rnd(u64);
impl Rnd {
    pub fn new(seed: u64) -> Self {
        Rnd(seed)
    }
    pub fn next(&mut self) -> f32 {
        self.0 = (self.0 * 16807) % 2147483647;
        ((self.0 as f64 - 1.0) / 2147483646.0) as f32
    }
}

/// pulse.js mix(): whole numbers, as `| 0` makes them.
pub fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    [(a[0] + (b[0] - a[0]) * t).trunc(), (a[1] + (b[1] - a[1]) * t).trunc(), (a[2] + (b[2] - a[2]) * t).trunc()]
}
fn col(c: Rgb, a: f32) -> Color {
    Color::from_rgba((c[0] / 255.0).clamp(0.0, 1.0), (c[1] / 255.0).clamp(0.0, 1.0), (c[2] / 255.0).clamp(0.0, 1.0), a.clamp(0.0, 1.0)).unwrap_or(Color::BLACK)
}
fn c255(c: crate::palette::C) -> Rgb {
    [(c[0] * 255.0).round(), (c[1] * 255.0).round(), (c[2] * 255.0).round()]
}

/// pulse.js skyWeights(): how much of the night, the twilight and the day is in the sky, from the Sun's height.
pub fn sky_weights(st: &Value) -> Option<[f32; 3]> {
    if get(st, &["ui", "sun"]).as_bool() == Some(false) {
        return None;
    }
    let el = num(st, &["night", "sun", "el"])? as f32;
    let sm = |a: f32, b: f32, x: f32| { let u = ((x - a) / (b - a)).clamp(0.0, 1.0); u * u * (3.0 - 2.0 * u) };
    let (d, n) = (sm(1.0, 12.0, el), sm(1.0, 9.0, -el));
    Some([n, (1.0 - d - n).max(0.0), d])
}

/// pulse.js sunPhase(): night, dawn, day or dusk from the weather's sunrise and sunset and the local clock.
pub fn phase(st: &Value, local_minutes: i64) -> &'static str {
    let mins = |s: String| -> Option<i64> { let (h, m) = s.split_once(':')?; Some(h.trim().parse::<i64>().ok()? * 60 + m.trim().parse::<i64>().ok()?) };
    let (Some(rise), Some(set)) = (mins(text(st, &["weather", "sunrise"])), mins(text(st, &["weather", "sunset"]))) else { return "night" };
    if (local_minutes - rise).abs() <= 40 { "dawn" } else if (local_minutes - set).abs() <= 40 { "dusk" } else if local_minutes > rise && local_minutes < set { "day" } else { "night" }
}

pub struct Backdrop {
    key: String,
    /// BGRA, w × h
    pub pixels: Vec<u8>,
    pub w: u32,
    pub h: u32,
    /// the tops of your city's landmarks and of the far masts (in the picture's pixels): a red light blinks on each
    pub tops: Vec<(f32, f32)>,
    /// the colour low in the sky, and of the air at the horizon (the mist takes it)
    pub low: Rgb,
    pub air: Rgb,
    pub sky_w: Option<[f32; 3]>,
    /// true when the picture changed at the last `update`
    pub fresh: bool,
}

impl Default for Backdrop {
    fn default() -> Self {
        Backdrop { key: String::new(), pixels: Vec::new(), w: 0, h: 0, tops: Vec::new(), low: [0.0; 3], air: [0.0; 3], sky_w: None, fresh: false }
    }
}

struct P<'a> {
    pm: &'a mut Pixmap,
}
impl P<'_> {
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgb, a: f32) {
        if w <= 0.0 || h <= 0.0 { return }
        if let Some(r) = Rect::from_xywh(x, y, w, h) {
            let mut p = Paint::default();
            p.set_color(col(c, a));
            p.anti_alias = false;
            self.pm.fill_rect(r, &p, Transform::identity(), None);
        }
    }
    /// A rectangle filled with a gradient along y (stops: place 0..1, colour, alpha) between y0 and y1.
    fn vgrad(&mut self, x: f32, y: f32, w: f32, h: f32, y0: f32, y1: f32, stops: &[(f32, Rgb, f32)]) {
        let st: Vec<GradientStop> = stops.iter().map(|s| GradientStop::new(s.0, col(s.1, s.2))).collect();
        if let (Some(sh), Some(r)) = (LinearGradient::new((0.0, y0).into(), (0.0, y1).into(), st, SpreadMode::Pad, Transform::identity()), Rect::from_xywh(x, y, w.max(0.01), h.max(0.01))) {
            self.pm.fill_rect(r, &Paint { shader: sh, anti_alias: false, ..Default::default() }, Transform::identity(), None);
        }
    }
    fn path(subs: &[Vec<(f32, f32)>], close: bool) -> Option<tiny_skia::Path> {
        let mut pb = PathBuilder::new();
        for s in subs {
            if s.len() < 2 { continue }
            pb.move_to(s[0].0, s[0].1);
            for q in &s[1..] {
                pb.line_to(q.0, q.1);
            }
            if close { pb.close() }
        }
        pb.finish()
    }
    fn fill(&mut self, pts: &[(f32, f32)], c: Rgb, a: f32) {
        if let Some(path) = Self::path(&[pts.to_vec()], true) {
            let mut p = Paint::default();
            p.set_color(col(c, a));
            self.pm.fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);
        }
    }
    fn stroke(&mut self, pts: &[(f32, f32)], lw: f32, c: Rgb, a: f32) {
        if let Some(path) = Self::path(&[pts.to_vec()], false) {
            let mut p = Paint::default();
            p.set_color(col(c, a));
            self.pm.stroke_path(&path, &p, &Stroke { width: lw, ..Default::default() }, Transform::identity(), None);
        }
    }
}

/// Even-odd: is the point inside these outlines?
fn inside(subs: &[Vec<(f32, f32)>], x: f32, y: f32) -> bool {
    let mut n = false;
    for s in subs {
        let mut j = s.len() - 1;
        for i in 0..s.len() {
            let (a, b) = (s[i], s[j]);
            if (a.1 > y) != (b.1 > y) && x < (b.0 - a.0) * (y - a.1) / (b.1 - a.1) + a.0 {
                n = !n;
            }
            j = i;
        }
    }
    n
}

/// What the picture depends on.
pub struct Scene<'a> {
    pub st: &'a Value,
    pub pal: &'a Pal,
    pub kind: &'a str,
    /// the screen in its own pixels
    pub w: u32,
    pub h: u32,
    /// the free band between the side columns, in pixels
    pub band: (f32, f32),
    pub local_minutes: i64,
    pub polished: bool,
    /// a community scene draws its own world (no ridge, no city) unless it asks for the far city
    pub world_off: bool,
}

impl Backdrop {
    /// Paints the picture again if anything it depends on changed. Cheap otherwise.
    pub fn update(&mut self, s: &Scene) {
        let (bg, a2, ac, fg, warn) = (c255(s.pal.bg), c255(s.pal.accent2), c255(s.pal.accent), c255(s.pal.fg), c255(s.pal.warn));
        let city = if get(s.st, &["privacy"]).as_bool().unwrap_or(false) { String::new() } else { text(s.st, &["ui", "skyline"]) };
        let ph = phase(s.st, s.local_minutes);
        let sky_w = if s.polished { sky_weights(s.st) } else { None };
        // the light changes slowly: a new picture every fiftieth of the way between night and day
        let q = |v: f32| (v * 50.0).round() as i32;
        let key = format!("{}x{}|{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}|{}|{:?}|{:.0},{:.0}|{}|{}", s.w, s.h, s.kind, city, bg, a2, ac, fg, warn, ph, sky_w.map(|w| [q(w[0]), q(w[1]), q(w[2])]), s.band.0, s.band.1, s.polished, s.world_off);
        self.fresh = false;
        if key == self.key || s.w == 0 || s.h == 0 {
            return;
        }
        let Some(mut pm) = Pixmap::new(s.w, s.h) else { return };
        self.key = key;
        self.fresh = true;
        self.tops.clear();
        self.sky_w = sky_w;
        let (wf, hf) = (s.w as f32, s.h as f32);
        let (hz, cx) = (hf * 0.745, wf / 2.0);
        let black = [0.0, 0.0, 0.0];
        let mut b = P { pm: &mut pm };
        // the sky follows the real sun: night, a warm dawn and dusk, a (still dark enough to read) day
        let pn = [mix(bg, black, 0.35), mix(bg, a2, 0.10), mix(bg, a2, 0.28)];
        let pd = [mix(bg, ac, 0.16), mix(bg, ac, 0.24), mix(mix(bg, fg, 0.22), a2, 0.2)];
        let pt = [mix(bg, [20.0, 10.0, 40.0], 0.4), mix(bg, a2, 0.22), mix(mix(bg, warn, 0.45), a2, 0.35)];
        let pal3 = match ph { "day" => pd, "dawn" | "dusk" => pt, _ => pn };
        let (mut top, mut midc, mut low) = (pal3[0], pal3[1], pal3[2]);
        if let Some(wt) = sky_w {
            // 1.8: no steps: the three skies mixed by the Sun's real height
            let mx3 = |k: usize| -> Rgb { [pn[k][0] * wt[0] + pt[k][0] * wt[1] + pd[k][0] * wt[2], pn[k][1] * wt[0] + pt[k][1] * wt[1] + pd[k][1] * wt[2], pn[k][2] * wt[0] + pt[k][2] * wt[1] + pd[k][2] * wt[2]] };
            (top, midc, low) = (mx3(0), mx3(1), mx3(2));
        }
        self.low = low;
        self.air = low;
        b.vgrad(0.0, 0.0, wf, hz + 1.0, 0.0, hz, &[(0.0, top, 1.0), (0.7, midc, 1.0), (1.0, low, 1.0)]);
        b.vgrad(0.0, hz, wf, hf - hz, hz, hf, &[(0.0, mix(bg, a2, 0.2), 1.0), (1.0, mix(bg, black, 0.5), 1.0)]);
        let (night_k, twi_k) = match sky_w { Some(wt) => (wt[0], wt[1]), None => (if ph == "night" { 1.0 } else { 0.0 }, if ph == "dawn" || ph == "dusk" { 1.0 } else { 0.0 }) };
        if !s.world_off && s.kind != "noir" {
            // the far ridge (resize(): rnd(42), after the stars, the far blocks and the drops have taken theirs)
            let mut r = Rnd::new(42);
            for _ in 0..130 * 4 { r.next(); }
            let mut far: Vec<(f32, f32, f32)> = Vec::new();
            let mut x = 0.0;
            while x < wf {
                let bw = wf * (0.012 + r.next() * 0.03);
                far.push((x, bw, hf * (0.03 + r.next() * 0.12)));
                x += bw + wf * 0.002;
            }
            for _ in 0..720 * 3 { r.next(); }
            let (a1, a2r) = (r.next() * 6.0, r.next() * 6.0);
            let ridge: Vec<f32> = (0..=64).map(|i| { let u = i as f32 / 64.0; 0.5 + 0.28 * (u * 7.1 + a1).sin() + 0.16 * (u * 17.3 + a2r).sin() + 0.08 * (u * 41.0 + a1 * 2.0).sin() }).collect();
            let mut pts: Vec<(f32, f32)> = vec![(0.0, hz)];
            for (i, v) in ridge.iter().enumerate() {
                let u = i as f32 / 64.0;
                pts.push((u * wf, hz - hf * (0.025 + 0.085 * v) * (0.55 + 0.45 * (u - 0.5).abs() * 2.0)));
            }
            pts.push((wf, hz));
            b.fill(&pts, mix(bg, a2, 0.13), 1.0);
            b.stroke(&pts, (hf / 900.0).max(1.0), a2, 0.35);
            let has_city = city_of(&city).is_some();
            let rr = hf * if s.kind == "sphere" { 0.18 } else { 0.15 };
            let mut far_tops: Vec<(f32, f32)> = Vec::new();
            if s.polished {
                // 1.8: a far city that looks like one: three rows of buildings, each paler with the distance, with
                // setbacks, spires, slanted roofs and crowns, a lighter edge on the roof, and windows that come on as
                // the Sun goes down
                let lit = 0.07 + 0.50 * night_k + 0.28 * twi_k;
                let low2 = if has_city { 0.6 } else { 1.0 };
                let (warm, hzc) = (mix(warn, fg, 0.35), mix(low, fg, 0.12));
                for l in 0..3usize {
                    let mut rn = Rnd::new(211 + l as u64 * 53);
                    let (dist, sc) = ([0.62, 0.36, 0.12][l], [0.62, 0.82, 1.0][l] * low2);
                    let body = mix(mix(bg, a2, 0.18), hzc, dist);
                    let (edge, wa) = (mix(body, fg, 0.22), [0.38, 0.62, 0.9][l]);
                    let mut x = -wf * 0.01 * l as f32;
                    while x < wf {
                        let bw = wf * (0.011 + rn.next() * 0.024) * (0.8 + l as f32 * 0.12);
                        let tall = rn.next() < 0.09;
                        let h = hf * (0.028 + rn.next() * 0.10) * sc * if tall { 1.55 } else { 1.0 };
                        let ty = rn.next();
                        let gap = wf * 0.0015 + if rn.next() < 0.15 { wf * 0.012 } else { 0.0 };
                        let x0 = x;
                        x += bw + gap;
                        if s.kind != "sphere" && x0 + bw > cx - rr * 1.1 && x0 < cx + rr * 1.1 {
                            rn.next();
                            continue;
                        }
                        let top_y = hz - h;
                        let mut parts: Vec<[f32; 4]> = vec![[x0, top_y, bw, h]]; // boxes, bottom to top
                        if ty > 0.5 && ty <= 0.72 {
                            parts.push([x0 + bw * 0.18, top_y - h * 0.22, bw * 0.64, h * 0.22]); // a setback
                        } else if ty > 0.72 && ty <= 0.84 {
                            parts.push([x0 + bw * 0.2, top_y - h * 0.18, bw * 0.6, h * 0.18]); // two
                            parts.push([x0 + bw * 0.38, top_y - h * 0.32, bw * 0.24, h * 0.14]);
                        } else if ty > 0.92 {
                            parts.push([x0, top_y - h * 0.05, bw * 0.22, h * 0.05]); // a crown
                            parts.push([x0 + bw * 0.78, top_y - h * 0.05, bw * 0.22, h * 0.05]);
                        }
                        let mut roof_y = top_y;
                        let thin = (hf / 1100.0).max(1.0);
                        if ty > 0.84 && ty <= 0.92 {
                            // a slanted roof
                            let dir = rn.next() < 0.5;
                            let (yl, yr) = (top_y + if dir { 0.0 } else { h * 0.16 }, top_y + if dir { h * 0.16 } else { 0.0 });
                            b.fill(&[(x0, hz), (x0, yl), (x0 + bw, yr), (x0 + bw, hz)], body, 1.0);
                            b.stroke(&[(x0, yl), (x0 + bw, yr)], thin, edge, 0.9);
                        } else {
                            for p in &parts {
                                b.rect(p[0], p[1], p[2], p[3] + 1.0, body, 1.0);
                                b.rect(p[0], p[1], p[2], thin, edge, 0.9);
                                roof_y = roof_y.min(p[1]);
                            }
                        }
                        // one side a little darker: a volume, not a card
                        let st: Vec<GradientStop> = vec![GradientStop::new(0.0, col(black, 0.0)), GradientStop::new(0.62, col(black, 0.0)), GradientStop::new(1.0, col(black, 0.22))];
                        if let (Some(sh), Some(r)) = (LinearGradient::new((x0, 0.0).into(), (x0 + bw, 0.0).into(), st, SpreadMode::Pad, Transform::identity()), Rect::from_xywh(x0, top_y, bw.max(0.01), h.max(0.01))) {
                            b.pm.fill_rect(r, &Paint { shader: sh, anti_alias: false, ..Default::default() }, Transform::identity(), None);
                        }
                        if (tall || ty > 0.72 && ty <= 0.84) && rn.next() < 0.7 {
                            // a mast on the tall ones, with its light
                            let mh = h * (0.12 + rn.next() * 0.2);
                            b.rect(x0 + bw / 2.0 - (hf / 1600.0).max(0.5), roof_y - mh, (hf / 800.0).max(1.0), mh, edge, 0.9);
                            if l > 0 {
                                far_tops.push((x0 + bw / 2.0, roof_y - mh));
                            }
                        }
                        // windows: a grid, some lit; more when it gets dark
                        let ws = (hf / if l == 2 { 520.0 } else { 640.0 }).max(1.0);
                        let (cols, rows) = (((bw - ws * 2.0) / (ws * 2.3)).floor() as i32, ((h - ws * 2.0) / (ws * 2.9)).floor() as i32);
                        let all_lit = rn.next() < 0.06;
                        for ry in 0..rows.max(0) {
                            for cc in 0..cols.max(0) {
                                let hsh = rn.next();
                                if hsh < if all_lit { (lit * 1.8).min(0.9) } else { lit } {
                                    let c2 = (hsh * 97.0).rem_euclid(1.0);
                                    let wc = if c2 < 0.68 { warm } else if c2 < 0.9 { ac } else { a2 };
                                    b.rect(x0 + ws * 1.4 + cc as f32 * ws * 2.3, top_y + ws * 1.6 + ry as f32 * ws * 2.9, ws, ws * 1.5, wc, (0.3 + (hsh * 31.0).rem_euclid(1.0) * 0.6) * wa * (0.55 + 0.45 * night_k.max(twi_k)));
                                }
                            }
                        }
                    }
                }
            } else {
                for f in &far {
                    if s.kind != "sphere" && f.0 + f.1 > cx - rr * 1.1 && f.0 < cx + rr * 1.1 { continue }
                    b.rect(f.0, hz - f.2, f.1, f.2 * if has_city { 0.6 } else { 1.0 }, mix(bg, a2, 0.18), 1.0);
                }
            }
            // your city, in front of the far blocks (pulse.js drawCity)
            if let Some(list) = city_of(&city) {
                let r_core = hf * match s.kind { "sphere" => 0.165, "orbit" => 0.3, _ => 0.14 };
                self.tops = Self::draw_city(&mut b, list, hz, cx, r_core, hf, s.band, bg, a2, ac, fg, warn);
            }
            self.tops.extend(far_tops);
            if s.polished {
                // 1.8: air. The far city fades into the colour of the sky at the horizon, the city lights the low sky
                // at night
                let hzc = mix(low, fg, 0.10 + 0.10 * (1.0 - night_k));
                self.air = hzc;
                b.vgrad(0.0, hz - hf * 0.17, wf, hf * 0.17, hz - hf * 0.17, hz, &[(0.0, hzc, 0.0), (0.75, hzc, 0.20 + 0.10 * twi_k), (1.0, hzc, 0.42 + 0.12 * twi_k)]);
                if night_k > 0.05 {
                    // the glow of a city at night, on the low sky ("lighter", squashed to a third)
                    let st = vec![GradientStop::new(0.0, col(mix(a2, warn, 0.25), 0.16 * night_k)), GradientStop::new(1.0, col(a2, 0.0))];
                    let ts = Transform::from_translate(cx, hz).pre_scale(1.0, 0.34);
                    if let (Some(sh), Some(r)) = (RadialGradient::new((0.0, 0.0).into(), (0.0, 0.0).into(), wf * 0.5, st, SpreadMode::Pad, Transform::identity()), Rect::from_xywh(-wf / 2.0, -wf * 0.5, wf, wf * 0.5)) {
                        b.pm.fill_rect(r, &Paint { shader: sh, blend_mode: tiny_skia::BlendMode::Plus, anti_alias: false, ..Default::default() }, ts, None);
                    }
                }
                // and the same air over the far floor
                b.vgrad(0.0, hz, wf, (hf - hz) * 0.5, hz, hz + (hf - hz) * 0.5, &[(0.0, hzc, 0.22), (1.0, hzc, 0.0)]);
                // a grain breaks the bands of a dark gradient
                let mut r3 = Rnd::new(7);
                let grain: Vec<(u8, u8)> = (0..128 * 128).map(|_| { let v = if r3.next() < 0.5 { 0u8 } else { 255 }; (v, (3.0 + r3.next() * 4.0) as u8) }).collect();
                let (pw, ph2) = (s.w as usize, s.h as usize);
                let data = b.pm.data_mut();
                for y in 0..ph2 {
                    for x in 0..pw {
                        let (v, a) = grain[(y % 128) * 128 + (x % 128)];
                        let (o, af) = ((y * pw + x) * 4, a as f32 / 255.0);
                        for ch in 0..3 {
                            data[o + ch] = (data[o + ch] as f32 * (1.0 - af) + v as f32 * af) as u8;
                        }
                    }
                }
                if s.kind != "orbit" {
                    // the floor is wet: what stands on the horizon is mirrored on it, fading as it comes near
                    let rh = ((hf - hz).min(hf * 0.2)) as usize;
                    if rh >= 8 {
                        let hzi = hz as usize;
                        let data = b.pm.data_mut();
                        for dy in 0..rh {
                            let (src, dst) = (hzi.saturating_sub(dy + 1), hzi + dy);
                            if dst >= ph2 { break }
                            for x in 0..pw {
                                let (so, d_o) = ((src * pw + x) * 4, (dst * pw + x) * 4);
                                for ch in 0..3 {
                                    data[d_o + ch] = (data[d_o + ch] as f32 * 0.58 + data[so + ch] as f32 * 0.42) as u8;
                                }
                            }
                        }
                        let fc = mix(bg, a2, 0.1);
                        b.vgrad(0.0, hz, wf, rh as f32 + 1.0, hz, hz + rh as f32, &[(0.0, fc, 0.0), (1.0, fc, 0.97)]);
                    }
                }
            }
        }
        // RGBA (premultiplied, and opaque here) to the BGRA the GPU takes
        let mut px = pm.take();
        for p4 in px.chunks_exact_mut(4) {
            p4.swap(0, 2);
            p4[3] = 255;
        }
        self.pixels = px;
        self.w = s.w;
        self.h = s.h;
    }

    /// pulse.js drawCity(): each landmark where the table says, filled with a gradient (or drawn as a line), with
    /// lit windows only inside its outline. Returns the tops that get a red light.
    #[allow(clippy::too_many_arguments)]
    fn draw_city(b: &mut P, list: &[Landmark], hz: f32, cx: f32, r_core: f32, hf: f32, band: (f32, f32), bg: Rgb, a2: Rgb, ac: Rgb, fg: Rgb, warn: Rgb) -> Vec<(f32, f32)> {
        let mut tops = Vec::new();
        let ex = r_core * 1.25;
        let (left, right) = ((band.0, cx - ex), (cx + ex, band.1));
        let body = mix(bg, a2, 0.3);
        let mut r2 = Rnd::new(97);
        for l in list {
            let u = l.x.abs();
            let x = if l.x < 0.0 { left.1 - u * (left.1 - left.0) } else { right.0 + u * (right.1 - right.0) };
            let (h, w) = (l.h * hf, l.w * hf);
            let subs: Vec<Vec<(f32, f32)>> = l.paths.iter().map(|s| s.iter().map(|q| (x + q.0 * hf, hz + q.1 * hf)).collect()).collect();
            // a soft glow behind it, so the outline reads against the sky (the canvas shadow: a wide faint line here)
            if let Some(path) = P::path(&subs, false) {
                for (k, a) in [(hf / 60.0, 0.10), (hf / 110.0, 0.16)] {
                    let mut p = Paint::default();
                    p.set_color(col(a2, a));
                    b.pm.stroke_path(&path, &p, &Stroke { width: k * 2.0, line_join: tiny_skia::LineJoin::Round, ..Default::default() }, Transform::identity(), None);
                }
            }
            if l.stroked {
                if let Some(path) = P::path(&subs, false) {
                    let mut p = Paint::default();
                    p.set_color(col(mix(body, fg, 0.45), 1.0));
                    b.pm.stroke_path(&path, &p, &Stroke { width: (hf / 420.0).max(1.2), ..Default::default() }, Transform::identity(), None);
                }
            } else {
                if let Some(path) = P::path(&subs, true) {
                    let st = vec![GradientStop::new(0.0, col(mix(body, a2, 0.35), 1.0)), GradientStop::new(1.0, col(mix(body, [0.0; 3], 0.3), 1.0))];
                    if let Some(sh) = LinearGradient::new((0.0, hz - h).into(), (0.0, hz).into(), st, SpreadMode::Pad, Transform::identity()) {
                        b.pm.fill_path(&path, &Paint { shader: sh, anti_alias: true, ..Default::default() }, FillRule::EvenOdd, Transform::identity(), None);
                    }
                    let mut p = Paint::default();
                    p.set_color(col(mix(a2, fg, 0.35), 0.75));
                    b.pm.stroke_path(&path, &p, &Stroke { width: (hf / 800.0).max(1.0), ..Default::default() }, Transform::identity(), None);
                }
                // lit windows, only inside the outline
                let nwin = if l.nat { 0 } else if l.flat { 30 } else { (12.0 + 90.0 * w * h / (hf * hf) * 10.0).round() as usize };
                for _ in 0..nwin {
                    let (px, py, ws) = (x + (r2.next() - 0.5) * w, hz - r2.next() * h * 0.9, (hf / 500.0).max(1.0));
                    if inside(&subs, px, py) {
                        let wc = if r2.next() < 0.3 { warn } else { ac };
                        b.rect(px, py, ws, ws * 1.4, wc, 0.35 + r2.next() * 0.45);
                    }
                }
            }
            if l.top {
                tops.push((x, hz - h));
            }
        }
        tops
    }
}
