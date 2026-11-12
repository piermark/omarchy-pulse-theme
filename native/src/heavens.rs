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

/// A rocket of fireworks going up, or a spark of a burst (pulse.js launch() and burst()).
struct Rocket {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    ty: f32,
    col: C,
    col2: C,
    big: bool,
}
struct Spark {
    x: f32,
    y: f32,
    px: f32,
    py: f32,
    vx: f32,
    vy: f32,
    life: f32,
    col: C,
    dec: f32,
    tw: bool,
}

/// The Starlink satellites (where each is drawn, gliding), the ISS and its trail, the fireworks.
#[derive(Default)]
pub struct Overhead {
    sl: Vec<Option<(f32, f32)>>,
    iss: Option<(f32, f32)>,
    iss_trail: Vec<(f32, f32)>,
    rockets: Vec<Rocket>,
    sparks: Vec<Spark>,
    queue: Vec<(f64, f32, f32, bool)>,
    flash: f32,
    seed: u64,
}

impl Overhead {
    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.seed >> 33) % 1_000_000) as f32 / 1_000_000.0
    }

    /// pulse.js fireworks(small, kind): rockets from the floor (four or eleven, a few tenths of a second apart), or
    /// one burst over the core ("burst"). Positions in fractions of the band and of the screen.
    pub fn fireworks(&mut self, now: f64, small: bool, burst: bool) {
        if burst {
            self.queue.push((now, -1.0, -1.0, true));
            return;
        }
        let n = if small { 4 } else { 11 };
        for i in 0..n {
            let (rx, ry) = (self.rand(), 0.08 + self.rand() * 0.2);
            self.queue.push((now + i as f64 * if small { 0.3 } else { 0.28 }, rx, ry, !small || i % 2 == 0));
        }
    }

    /// burst(): a sphere of sparks, even speeds on a ring, a few faster ones.
    fn burst(&mut self, x: f32, y: f32, col: C, col2: C, n: usize, spd: f32) {
        self.flash = self.flash.max(0.5);
        for i in 0..n {
            let a = self.rand() * TAU;
            let v = spd * if self.rand() < 0.8 { 0.85 + self.rand() * 0.15 } else { 0.3 + self.rand() * 0.5 };
            let dec = 0.008 + self.rand() * 0.008;
            let tw = self.rand() < 0.3;
            self.sparks.push(Spark { x, y, px: x, py: y, vx: a.cos() * v, vy: a.sin() * v, life: 1.0, col: if i % 3 != 0 { col } else { col2 }, dec, tw });
        }
    }

    /// A burst somewhere (a service back up, a drone landing, a push on a repo's roof): in screen pixels.
    pub fn burst_at(&mut self, x: f32, y: f32, col: C, fg: C, n: usize, spd: f32) {
        self.burst(x, y, col, fg, n, spd);
    }

    /// drawFireworks(): the rockets and their trails, sparks with gravity, some glittering, the sky lit a moment.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_fireworks(&mut self, win: &mut Window, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, now: f64, band: (f32, f32), frames: f32) {
        let cols = [pal.accent, pal.accent2, pal.ok, pal.warn, pal.fg];
        let due: Vec<(f64, f32, f32, bool)> = self.queue.iter().filter(|q| q.0 <= now).copied().collect();
        self.queue.retain(|q| q.0 > now);
        for (_, rx, ry, big) in due {
            if rx < 0.0 {
                self.burst(ox + w / 2.0, oy + h * 0.745 - h * 0.18, pal.ok, pal.fg, 70, h * 0.006);
                continue;
            }
            let (c1, c2) = (cols[((self.rand() * 5.0) as usize).min(4)], cols[((self.rand() * 5.0) as usize).min(4)]);
            let vx = (self.rand() - 0.5) * w * 0.0008;
            let vy = -h * (0.024 + self.rand() * 0.006);
            self.rockets.push(Rocket { x: ox + band.0 + rx * (band.1 - band.0), y: oy + h * 0.95, vx, vy, ty: oy + h * ry, col: c1, col2: c2, big });
        }
        if self.rockets.is_empty() && self.sparks.is_empty() && self.flash <= 0.02 {
            return;
        }
        let grav = h * 0.00018;
        let mut bursts = Vec::new();
        for r in self.rockets.iter_mut() {
            r.x += r.vx * frames;
            r.y += r.vy * frames;
            r.vy += grav * 0.6 * frames;
            // the rocket and its trail
            let (tx, ty) = (r.x - r.vx * 8.0, r.y - r.vy * 8.0);
            for k in 0..4 {
                let (u0, u1) = (k as f32 / 4.0, (k + 1) as f32 / 4.0);
                draw::line(win, &[(r.x + (tx - r.x) * u0, r.y + (ty - r.y) * u0), (r.x + (tx - r.x) * u1, r.y + (ty - r.y) * u1)], (h / 350.0).max(1.5), mix(pal.fg, r.col, (u0 + u1) / 2.0), 1.0 - (u0 + u1) / 2.0);
            }
            if r.y <= r.ty || r.vy >= 0.0 {
                bursts.push((r.x, r.y, r.col, r.col2, r.big));
            }
        }
        self.rockets.retain(|r| !(r.y <= r.ty || r.vy >= 0.0));
        for (x, y, c1, c2, big) in bursts {
            self.burst(x, y, c1, c2, if big { 140 } else { 80 }, h * if big { 0.0105 } else { 0.0075 });
        }
        let lw = (h / 420.0).max(1.2);
        let mut glitter = Vec::new();
        for _ in 0..self.sparks.len() {
            glitter.push(self.rand() < 0.5);
        }
        for (i, p) in self.sparks.iter_mut().enumerate() {
            p.px = p.x;
            p.py = p.y;
            p.x += p.vx * frames;
            p.y += p.vy * frames;
            let damp = 0.975f32.powf(frames);
            p.vx *= damp;
            p.vy = p.vy * damp + grav * frames;
            p.life -= p.dec * frames;
            if p.life <= 0.0 {
                continue;
            }
            let al = (p.life * 1.4).min(1.0) * if p.tw && p.life < 0.5 { if glitter[i] { 1.0 } else { 0.2 } } else { 1.0 };
            // a short streak from where it was (the canvas strokes from two frames back)
            let (sx, sy) = (p.x - (p.x - p.px) * 2.0, p.y - (p.y - p.py) * 2.0);
            draw::line(win, &[(sx, sy), (p.x, p.y)], lw, mix(p.col, pal.fg, if p.life > 0.8 { 0.6 } else { 0.0 }), al);
        }
        self.sparks.retain(|p| p.life > 0.0);
        if self.flash > 0.02 {
            crate::scene::rect(win, ox, oy, w, h, pal.fg, self.flash * 0.06); // the sky lights up
            self.flash *= 0.85f32.powf(frames);
        }
    }

    /// drawStarlinkFine(): the satellites really in your sky and lit by the Sun, above 10°; they glide to where they
    /// are, a faint thread joins those that fly in a row, each one glints, the first is the brightest, and the tag says
    /// how many and which way they go (over the highest one whose tag is clear of the panels and the sphere).
    #[allow(clippy::too_many_arguments)]
    pub fn starlink(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, frames: f32, boxes: &[[f32; 4]], still: bool) {
        let ss = list(st, &["starlink", "sats"]);
        if ss.is_empty() {
            self.sl.clear();
            return;
        }
        let hz = h * 0.745;
        let u1 = h / 1080.0;
        self.sl.resize(ss.len(), None);
        let mut pts: Vec<(f32, f32, usize)> = Vec::new();
        let ease = 1.0 - (1.0f32 - 0.025).powf(frames.max(0.05));
        for (i, s) in ss.iter().enumerate() {
            let el = num(s, &["el"]).unwrap_or(-90.0) as f32;
            let p = if el >= 10.0 { sky_xy(num(s, &["az"]).unwrap_or(0.0) as f32, el, w, hz) } else { None };
            let Some(p) = p else { self.sl[i] = None; continue };
            let mut q = match self.sl[i] { Some(q) if (q.0 - p.0).abs() <= w * 0.2 && !still => q, _ => p };
            q.0 += (p.0 - q.0) * ease;
            q.1 += (p.1 - q.1) * ease;
            self.sl[i] = Some(q);
            pts.push((q.0, q.1, i));
        }
        if pts.is_empty() {
            return;
        }
        let (_, sky, _) = crate::sky::darkness(st);
        let vis = sky.max(0.5);
        let near = h * 0.09;
        for i in 1..pts.len() {
            let (dx, dy) = (pts[i].0 - pts[i - 1].0, pts[i].1 - pts[i - 1].1);
            if pts[i].2 == pts[i - 1].2 + 1 && dx * dx + dy * dy < near * near {
                draw::line(win, &[(ox + pts[i - 1].0, oy + pts[i - 1].1), (ox + pts[i].0, oy + pts[i].1)], (u1 * 1.2).max(1.0), pal.fg, 0.20 * vis);
            }
        }
        for (i, p) in pts.iter().enumerate() {
            let lead = if i == 0 { 1.5 } else { 1.0 };
            let a = vis * (0.75 + 0.25 * (t * 2.6 + i as f32 * 0.9).sin()) * if i == 0 { 1.0 } else { 0.85 };
            let s = u1 * 3.0 * lead;
            let (x, y) = (ox + p.0, oy + p.1);
            radial(win, x, y, 0.0, s * 3.2, 1.0, pal.fg, &[(0.0, 0.55 * a), (1.0, 0.0)]);
            draw::arc_fill(win, x, y, s * 0.55, s * 0.55, 0.0, 0.0, TAU, pal.fg, a);
            crate::scene::rect(win, x - s * 2.2, y - u1 * 0.5, s * 4.4, u1.max(1.0), pal.fg, a); // a glint
            crate::scene::rect(win, x - u1 * 0.5, y - s * 1.5, u1.max(1.0), s * 3.0, pal.fg, a);
        }
        if get(st, &["screencast"]).as_bool().unwrap_or(false) {
            return;
        }
        let n = pts.len();
        let hd = if n > 1 { (pts[0].0 - pts[n - 1].0, pts[0].1 - pts[n - 1].1) } else { (0.0, 0.0) };
        let arrow = if hd.0.abs() + hd.1.abs() < h * 0.01 {
            ""
        } else if hd.1.abs() > hd.0.abs() * 2.0 {
            if hd.1 < 0.0 { " ↑" } else { " ↓" }
        } else if hd.1 < -hd.0.abs() * 0.4 {
            if hd.0 > 0.0 { " ↗" } else { " ↖" }
        } else if hd.1 > hd.0.abs() * 0.4 {
            if hd.0 > 0.0 { " ↘" } else { " ↙" }
        } else if hd.0 > 0.0 {
            " →"
        } else {
            " ←"
        };
        let (f1, f2) = ((h * 0.0135).round(), (h * 0.0118).round());
        let l1 = format!("STARLINK{}", if n > 3 { " TRAIN" } else { "" });
        let l2 = format!("{n} {}{arrow}", if n == 1 { "satellite" } else { "satellites" });
        let (w1, w2) = (draw::measure(win, font, &l1, f1, 700.0), draw::measure(win, font, &l2, f2, 500.0));
        let (pw, ph) = (w1.max(w2) + h * 0.02, f1 + f2 * 1.45 + h * 0.012);
        let mut sorted = pts.clone();
        sorted.sort_by(|a, b| a.1.total_cmp(&b.1));
        // starlinkLabelAt(): the first satellite whose tag is clear of every panel and of the sphere
        let r = h * 0.17;
        let mut all = boxes.to_vec();
        all.push([w / 2.0 - r, h * 0.745 - r * 2.0, r * 2.0, r * 2.0]);
        let at = sorted.iter().map(|p| (p.0 + h * 0.01, p.1 - h * 0.012)).find(|(x, y)| !all.iter().any(|b| *x < b[0] + b[2] && x + pw > b[0] && *y > b[1] && y - ph < b[1] + b[3]));
        let Some((px0, py0)) = at else { return };
        let (px0, py0) = (ox + px0, oy + py0 - ph);
        crate::scene::rect(win, px0, py0, pw, ph, pal.bg, 0.78);
        crate::scene::rect(win, px0, py0, (h / 500.0).max(2.0), ph, pal.accent, 0.9);
        draw::text(win, cx, font, &l1, px0 + h * 0.011, py0 + h * 0.006 + f1 * 0.85, f1, 700.0, pal.fg, 0.97, -1);
        draw::text(win, cx, font, &l2, px0 + h * 0.011, py0 + h * 0.006 + f1 * 0.85 + f2 * 1.45, f2, 500.0, pal.accent, 0.95, -1);
    }

    /// drawISS(): the station when it really passes over you, its trail, its light (dimmer in the Earth's shadow).
    #[allow(clippy::too_many_arguments)]
    pub fn iss(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, frames: f32) {
        let iss = get(st, &["space", "iss"]);
        let (Some(az), Some(el)) = (num(iss, &["az"]), num(iss, &["el"])) else {
            self.iss = None;
            self.iss_trail.clear();
            return;
        };
        let hz = h * 0.745;
        let tx = w / 2.0 + ((az as f32 - 180.0) / 90.0).clamp(-1.05, 1.05) * w * 0.44;
        let ty = hz - h * 0.05 - (el as f32 / 90.0) * hz * 0.6;
        let k = 1.0 - (1.0f32 - 0.08).powf(frames.max(0.05));
        let p = self.iss.map(|q| (q.0 + (tx - q.0) * k, q.1 + (ty - q.1) * k)).unwrap_or((tx, ty));
        self.iss = Some(p);
        self.iss_trail.push(p);
        if self.iss_trail.len() > 60 {
            self.iss_trail.remove(0);
        }
        let n = self.iss_trail.len();
        for i in 1..n {
            let (a, b) = (self.iss_trail[i - 1], self.iss_trail[i]);
            draw::line(win, &[(ox + a.0, oy + a.1), (ox + b.0, oy + b.1)], (h / 600.0).max(1.0), pal.fg, 0.4 * i as f32 / n as f32);
        }
        let lit = get(iss, &["lit"]).as_bool() != Some(false);
        let (x, y) = (ox + p.0, oy + p.1);
        let c = if lit { pal.fg } else { pal.accent2 };
        radial(win, x, y, 0.0, h * 0.03, 1.0, c, &[(0.0, 0.95), (0.15, 0.5), (1.0, 0.0)]);
        crate::scene::rect(win, x - h / 220.0, y - h / 1400.0, h / 110.0, h / 700.0, pal.fg, 1.0); // the solar wings
        crate::scene::rect(win, x - h / 1400.0, y - h / 450.0, h / 700.0, h / 225.0, pal.fg, 1.0);
        let alt = num(iss, &["alt"]).map(|a| format!("{a}")).unwrap_or_default();
        draw::text(win, cx, font, &format!("ISS · {alt} km · {}°", el.round()), x + h * 0.02, y - h * 0.012, (h * 0.014).round(), 600.0, pal.fg, 0.85, -1);
    }
}
