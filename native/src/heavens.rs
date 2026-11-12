//! The sky of Pulse 1.8 over you, translated from pulse.js: the faint random stars, the real Sun (and a solar eclipse
//! where it happens), the real bright stars with their constellations joined and named, the planets, the Moon with
//! its phase turned toward the Sun (and a lunar eclipse), meteor showers from their radiant, shooting stars, and the
//! aurora of your network's traffic (a real one in a geomagnetic storm).

use crate::backdrop::Rnd;
use crate::draw;
use crate::hub::{get, list, num, text};
use crate::palette::{mix, Pal, C};
use crate::scene::{radial, sky_xy};
use gpui::Window;
use serde_json::Value;

const TAU: f32 = std::f32::consts::TAU;
const PI: f32 = std::f32::consts::PI;

fn rgb(r: f32, g: f32, b: f32) -> C {
    [r / 255.0, g / 255.0, b / 255.0]
}

/// A shooting star, in fractions of the screen (the hub's effects do not know its size).
struct Shoot {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

#[derive(Default)]
pub struct Heavens {
    stars: Vec<[f32; 4]>,
    shooting: Vec<Shoot>,
    last_meteor: f64,
    seed: u64,
}

impl Heavens {
    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.seed >> 33) % 1_000_000) as f32 / 1_000_000.0
    }

    /// The hub's effects: "shooting" and "iss" each send a shooting star (pulse.js live()).
    pub fn fx(&mut self, kind: &str) {
        match kind {
            "shooting" => {
                let x = self.rand() * 0.6;
                self.shooting.push(Shoot { x, y: 0.05, vx: 0.012, vy: 0.006, life: 1.0 });
            }
            "iss" => self.shooting.push(Shoot { x: 0.1, y: 0.08, vx: 0.01, vy: 0.002, life: 1.0 }),
            _ => {}
        }
    }

    /// pulse.js draw(): the faint random stars (rnd(42), as many and where the web page puts them), then drawSun,
    /// then (after the rays, drawn by the caller) drawRealStars and drawMoon.
    #[allow(clippy::too_many_arguments)]
    pub fn back(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, wall: bool) {
        if self.stars.is_empty() {
            let mut r = Rnd::new(42);
            self.stars = (0..130).map(|_| [r.next(), r.next() * 0.62, r.next(), 0.3 + r.next() * 1.1]).collect();
        }
        let (d, sky, cloud) = crate::sky::darkness(st);
        let star_a = sky * 0.35; // with the real sky, the random ones are only a faint background
        if !wall {
            for s in &self.stars {
                let tw = 0.5 + 0.5 * (t * s[3] * 2.0 + s[2] * 40.0).sin();
                let ss = (h / 1080.0 * if s[2] > 0.9 { 2.4 } else { 1.3 }).max(1.0);
                crate::scene::rect(win, ox + s[0] * w, oy + s[1] * h, ss, ss, pal.fg, (0.15 + 0.6 * tw * s[2]) * star_a);
            }
        }
        if wall {
            return;
        }
        let hz = h * 0.745;
        // ---- drawSun()
        let sun = get(st, &["night", "sun"]);
        if let (Some(az), Some(el)) = (num(sun, &["az"]), num(sun, &["el"])) {
            let (az, el) = (az as f32, el as f32);
            if el >= -1.0 {
                if let Some(p) = sky_xy(az, el, w, hz) {
                    let (r, a, low) = (h * 0.03, (1.0 - cloud).powf(1.5), (1.0 - el / 12.0).max(0.0));
                    if a >= 0.04 {
                        let c = mix(pal.warn, pal.crit, low * 0.5);
                        let (x, y) = (ox + p.0, oy + p.1);
                        radial(win, x, y, 0.0, r * 6.0, 1.0, c, &[(0.0, 0.45 * a), (1.0, 0.0)]);
                        draw::arc_fill(win, x, y, r, r, 0.0, 0.0, TAU, mix(pal.fg, c, 0.4), 0.95 * a);
                        let ec = get(st, &["season", "eclipse"]);
                        let obsc = num(ec, &["obsc"]).unwrap_or(0.0) as f32;
                        if text(ec, &["kind"]) == "solar" && obsc > 0.0 {
                            if obsc > 0.995 {
                                // totality: the corona
                                radial(win, x, y, r, r * 4.5, 1.0, rgb(200.0, 215.0, 255.0), &[(0.0, 0.75 * a), (0.3, 0.22 * a), (1.0, 0.0)]);
                            }
                            let off = (get(ec, &["off"]).get(0).and_then(Value::as_f64).unwrap_or(0.0) as f32, get(ec, &["off"]).get(1).and_then(Value::as_f64).unwrap_or(0.0) as f32);
                            let rm = num(ec, &["rm"]).unwrap_or(1.02) as f32;
                            draw::arc_fill(win, x + off.0 * r, y - off.1 * r, r * rm, r * rm, 0.0, 0.0, TAU, mix(pal.bg, [0.0; 3], 0.6), 1.0);
                            draw::text(win, cx, font, &format!("eclipse · {}%", (obsc * 100.0).round()), x, y + r * 2.6, (h / 62.0).round(), 400.0, pal.fg, 0.8, 0);
                        }
                    }
                }
            }
        }
        let _ = d;
    }

    /// drawRealStars() and drawMoon().
    #[allow(clippy::too_many_arguments)]
    pub fn front(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32) {
        let (d, sky, cloud) = crate::sky::darkness(st);
        let hz = h * 0.745;
        let share = get(st, &["screencast"]).as_bool().unwrap_or(false);
        let night = get(st, &["night"]);
        // ---- drawRealStars()
        if night.is_object() && sky >= 0.03 {
            let stars = list(night, &["stars"]);
            let pts: Vec<Option<(f32, f32)>> = stars.iter().map(|s| match (num(s, &["az"]), num(s, &["el"])) {
                (Some(az), Some(el)) => sky_xy(az as f32, el as f32, w, hz).map(|p| (ox + p.0, oy + p.1)),
                _ => None,
            }).collect();
            let lw = (h / 1100.0).max(1.0);
            for l in list(night, &["lines"]) {
                let (Some(a), Some(b)) = (l.get(0).and_then(Value::as_u64), l.get(1).and_then(Value::as_u64)) else { continue };
                if let (Some(Some(pa)), Some(Some(pb))) = (pts.get(a as usize), pts.get(b as usize)) {
                    if (pa.0 - pb.0).abs() < w * 0.4 {
                        draw::line(win, &[*pa, *pb], lw, pal.accent2, 0.34 * sky);
                    }
                }
            }
            for (i, s) in stars.iter().enumerate() {
                let Some(Some(p)) = pts.get(i) else { continue };
                let mag = num(s, &["m"]).unwrap_or(2.0) as f32;
                let tw = 0.75 + 0.25 * (t * (2.0 + (i % 5) as f32) + i as f32).sin();
                let sz = h / 1080.0 * (5.6 - mag * 1.1).max(2.4);
                let al = (1.3 - mag * 0.18).min(1.0) * sky * tw;
                if mag < 2.6 {
                    radial(win, p.0, p.1, 0.0, sz * 3.0, 1.0, pal.fg, &[(0.0, 0.45 * al), (1.0, 0.0)]);
                }
                draw::arc_fill(win, p.0, p.1, sz / 2.0, sz / 2.0, 0.0, 0.0, TAU, pal.fg, al);
            }
            if sky > 0.5 && !share {
                // the names, faint
                for nm in list(night, &["names"]) {
                    let idx: Vec<usize> = get(nm, &["i"]).as_array().map(|a| a.iter().filter_map(|x| x.as_u64().map(|v| v as usize)).collect()).unwrap_or_default();
                    let q: Vec<(f32, f32)> = idx.iter().filter_map(|i| pts.get(*i).copied().flatten()).collect();
                    if !q.is_empty() && q.len() >= 2.min(idx.len()) {
                        let (xs, ys) = (q.iter().map(|p| p.0).sum::<f32>() / q.len() as f32, q.iter().map(|p| p.1).sum::<f32>() / q.len() as f32);
                        draw::text(win, cx, font, &text(nm, &["n"]), xs, ys + h * 0.03, (h * 0.0105).round(), 600.0, pal.accent2, 0.5 * sky, 0);
                    }
                }
            }
            // the planets: bright, steady, a little coloured
            for pl in list(night, &["planets"]) {
                let (Some(az), Some(el)) = (num(pl, &["az"]), num(pl, &["el"])) else { continue };
                let Some(p) = sky_xy(az as f32, el as f32, w, hz) else { continue };
                let n = text(pl, &["n"]);
                let pc = match n.as_str() { "venus" => rgb(255.0, 250.0, 235.0), "mars" => rgb(255.0, 140.0, 100.0), "jupiter" => rgb(255.0, 236.0, 200.0), "saturn" => rgb(240.0, 215.0, 160.0), _ => pal.fg };
                let pa = sky.max(if n == "venus" { 0.6 * (1.0 - cloud) } else { 0.0 });
                let ps = h / 1080.0 * (6.5 - num(pl, &["m"]).unwrap_or(0.0) as f32 * 0.8).max(4.2);
                let (x, y) = (ox + p.0, oy + p.1);
                radial(win, x, y, 0.0, ps * 4.0, 1.0, pc, &[(0.0, 0.5 * pa), (1.0, 0.0)]);
                draw::arc_fill(win, x, y, ps / 2.0, ps / 2.0, 0.0, 0.0, TAU, pc, pa);
                if !share && pa > 0.3 {
                    draw::text(win, cx, font, &n.to_uppercase(), x + ps * 2.0, y - ps, (h * 0.01).round(), 600.0, pc, 0.55 * pa, -1);
                }
            }
        }
        // ---- drawMoon()
        let m = get(night, &["moon"]);
        if let (Some(az), Some(el)) = (num(m, &["az"]), num(m, &["el"])) {
            let (az, el) = (az as f32, el as f32);
            if let Some(p) = sky_xy(az, el, w, hz).filter(|_| el >= -1.0) {
                let r = h * 0.034;
                let mut a = (0.35 + 0.65 * d.max(0.3)) * (1.0 - 0.75 * cloud);
                let waxing = get(m, &["waxing"]).as_bool().unwrap_or(true);
                let sun = get(night, &["sun"]);
                let sp = match (num(sun, &["az"]), num(sun, &["el"])) {
                    (Some(saz), Some(sel)) => sky_xy(saz as f32, (sel as f32).max(0.1), w, hz),
                    _ => None,
                };
                let mut ang = match sp { Some(s) => (s.1 - p.1).atan2(s.0 - p.0), None => if waxing { 0.0 } else { PI } };
                if num(sun, &["el"]).map(|e| e < 0.0).unwrap_or(false) {
                    ang = (r * 0.3).atan2(if waxing { 1.0 } else { -1.0 } * r); // under the horizon: its side
                }
                let mut k = num(m, &["lit"]).unwrap_or(0.0) as f32;
                let mut lit = mix(pal.fg, rgb(255.0, 244.0, 214.0), 0.5);
                let lec = get(st, &["season", "eclipse"]);
                let (pmag, umag) = (num(lec, &["pmag"]).unwrap_or(0.0) as f32, num(lec, &["umag"]).unwrap_or(0.0) as f32);
                if text(lec, &["kind"]) == "lunar" && pmag > 0.0 {
                    // a lunar eclipse: dimmer in the penumbra, copper red in the umbra
                    lit = mix(lit, rgb(178.0, 64.0, 36.0), (umag * 1.2 + 0.1).clamp(0.0, 1.0));
                    a *= if umag > 0.0 { 1.0 - 0.25 * umag.min(1.0) } else { 1.0 - 0.3 * pmag.min(1.0) };
                    k = k.max(0.97);
                }
                let (x, y) = (ox + p.0, oy + p.1);
                radial(win, x, y, r, r * 4.0, 1.0, lit, &[(0.0, 0.18 * a * k), (1.0, 0.0)]);
                draw::arc_fill(win, x, y, r, r, 0.0, 0.0, TAU, pal.bg, 0.55 * a); // the dark part (earthshine)
                // the lit half on the Sun's side, closed by the terminator: a crescent or gibbous
                let rx = (r * (1.0 - 2.0 * k).abs()).max(0.01);
                let mut pts: Vec<(f32, f32)> = (0..=24).map(|i| { let q = -PI / 2.0 + PI * i as f32 / 24.0; (q.cos() * r, q.sin() * r) }).collect();
                if k < 0.5 {
                    pts.extend((0..=24).map(|i| { let q = PI / 2.0 - PI * i as f32 / 24.0; (q.cos() * rx, q.sin() * r) }));
                } else {
                    pts.extend((0..=24).map(|i| { let q = PI / 2.0 + PI * i as f32 / 24.0; (q.cos() * rx, q.sin() * r) }));
                }
                let (ca, sa) = (ang.cos(), ang.sin());
                let pts: Vec<(f32, f32)> = pts.iter().map(|(px0, py0)| (x + px0 * ca - py0 * sa, y + px0 * sa + py0 * ca)).collect();
                draw::poly(win, &pts, lit, 0.95 * a);
            }
        }
    }

    /// meteors() and drawShooting(): a shower sends its meteors from its radiant; every shooting star leaves a trail.
    #[allow(clippy::too_many_arguments)]
    pub fn shooting(&mut self, win: &mut Window, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, now: f64, frames: f32) {
        let (_, sky, _) = crate::sky::darkness(st);
        let hz = h * 0.745;
        let sh = get(st, &["night", "shower"]);
        if sh.is_object() && sky >= 0.3 {
            let per_min = (num(sh, &["zhr"]).unwrap_or(0.0) as f32 * num(sh, &["strength"]).unwrap_or(1.0) as f32 / 20.0 + 0.5).min(40.0);
            let wait = 60.0 / per_min as f64 * (0.5 + self.rand() as f64);
            if now - self.last_meteor >= wait {
                self.last_meteor = now;
                let el = num(sh, &["el"]).unwrap_or(-1.0) as f32;
                let rp = if el > 0.0 { sky_xy(num(sh, &["az"]).unwrap_or(0.0) as f32, el, w, hz) } else { None };
                let (x0, y0) = (w * (0.15 + self.rand() * 0.7), hz * (0.1 + self.rand() * 0.45));
                let (dx, dy) = match rp { Some(r) => (x0 - r.0, y0 - r.1), None => ((self.rand() - 0.5) * w, h * 0.2) };
                let l = dx.hypot(dy).max(1.0);
                let sp = w * (0.008 + self.rand() * 0.008);
                self.shooting.push(Shoot { x: x0 / w, y: y0 / h, vx: dx / l * sp / w, vy: dy / l * sp / h, life: 1.0 });
            }
        }
        let lw = (h / 600.0).max(1.0);
        for s in self.shooting.iter_mut() {
            s.x += s.vx * frames;
            s.y += s.vy * frames;
            s.life -= 0.02 * frames;
            if s.life <= 0.0 {
                continue;
            }
            let (x, y) = (ox + s.x * w, oy + s.y * h);
            let (tx, ty) = (x - s.vx * w * 12.0, y - s.vy * h * 12.0);
            for k in 0..6 {
                let (u0, u1) = (k as f32 / 6.0, (k + 1) as f32 / 6.0);
                draw::line(win, &[(x + (tx - x) * u0, y + (ty - y) * u0), (x + (tx - x) * u1, y + (ty - y) * u1)], lw, pal.fg, s.life * (1.0 - (u0 + u1) / 2.0));
            }
        }
        self.shooting.retain(|s| s.life > 0.0);
    }

    /// pulse.js draw(): "aurora = network traffic": two veils of light, brighter as more data goes through your
    /// network (or with the music); a third, green one in a geomagnetic storm (Kp 4 and more).
    #[allow(clippy::too_many_arguments)]
    pub fn aurora(win: &mut Window, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, audio: &[f32]) {
        let kp = num(st, &["space", "kp"]).unwrap_or(0.0) as f32;
        let kp_a = if kp >= 4.0 { ((kp - 3.0) / 5.0).min(1.0) } else { 0.0 };
        let net = num(st, &["sys", "rx"]).unwrap_or(0.0) as f32 + num(st, &["sys", "tx"]).unwrap_or(0.0) as f32;
        let mid = if audio.is_empty() { 0.0 } else { let s: Vec<f32> = audio.iter().skip(6).take(10).copied().collect(); if s.is_empty() { 0.0 } else { s.iter().sum::<f32>() / s.len() as f32 / 100.0 } };
        let na = ((1.0 + net).log10() / 7.5).min(1.0).max(mid * 0.9).max(kp_a);
        let bands = if kp_a > 0.0 { 3 } else { 2 };
        let step = 14.0;
        for k in 0..bands {
            let kf = k as f32;
            let c = if k == 2 { pal.ok } else if k == 1 { pal.accent2 } else { pal.accent };
            let base = h * (0.16 + kf * 0.09) - if k == 2 { h * 0.12 } else { 0.0 };
            let top = |x: f32| base + (x * 0.004 + t * (0.25 + kf * 0.1) + kf * 2.0).sin() * h * 0.04 + (x * 0.011 - t * 0.4).sin() * h * 0.015;
            let bot = |x: f32| base + h * (0.07 + 0.05 * na) + (x * 0.003 + t * 0.2 + kf).sin() * h * 0.03;
            // the veil's gradient lies in screen space (base - .05H to base + .14H: 0, .45: strong, 1: 0): each column is
            // that gradient seen through the column's own window between the two curves
            let (g0, g1) = (base - h * 0.05, base + h * 0.14);
            let a = 0.05 + 0.13 * na;
            let g_mid = g0 + (g1 - g0) * 0.45;
            let mut x = 0.0f32;
            while x < w {
                let x1 = (x + step).min(w);
                let (yt, yb) = (top(x).min(top(x1)), bot(x).max(bot(x1)));
                if yb > yt {
                    let mask = gpui::ContentMask { bounds: gpui::Bounds::new(gpui::point(gpui::px(ox + x), gpui::px(oy + yt)), gpui::size(gpui::px(x1 - x), gpui::px(yb - yt))), ..Default::default() };
                    win.with_content_mask(Some(mask), |win| {
                        crate::scene::gradient(win, ox + x, oy + g0, x1 - x, g_mid - g0, c, 0.0, c, a);
                        crate::scene::gradient(win, ox + x, oy + g_mid, x1 - x, g1 - g_mid, c, a, c, 0.0);
                    });
                }
                x = x1;
            }
        }
    }
}
