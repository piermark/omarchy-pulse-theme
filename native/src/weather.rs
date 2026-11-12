//! The real weather over the city, translated from pulse.js draw() and its helpers: rain with depth (drawRainDeep:
//! three layers, rings where the drops land, a veil in heavy rain), as many drops as it really rains (precipCount),
//! snow, hail that bounces, and fog on the horizon.

use crate::backdrop::Rnd;
use crate::draw;
use crate::hub::{get, num};
use crate::palette::{mix, Pal};
use gpui::Window;
use serde_json::Value;

#[derive(Default)]
pub struct Weather {
    drops: Vec<[f32; 3]>,
    size: (f32, f32),
    ripples: Vec<(f32, f32, f64, f32)>,
    seed: u64,
}

/// pulse.js windFrom(): how strong (0..1) and which way on the screen (you look south: a west wind blows left).
pub fn wind(st: &Value) -> (f32, f32) {
    let unit = if get(st, &["weather", "windUnit"]).as_str() == Some("mph") { 1.609 } else { 1.0 };
    let kmh = unit * (num(st, &["weather", "wind"]).unwrap_or(0.0) as f32).max(num(st, &["weather", "gust"]).unwrap_or(0.0) as f32 * 0.8);
    let k = (kmh / 60.0).min(1.0);
    ((num(st, &["weather", "windDir"]).unwrap_or(270.0) as f32).to_radians().sin() * k, k)
}

impl Weather {
    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.seed >> 33) % 1_000_000) as f32 / 1_000_000.0
    }

    /// The drops of resize(): rnd(42) after the stars and the far blocks, as the web page draws them.
    fn make(&mut self, w: f32, h: f32) {
        let mut r = Rnd::new(42);
        for _ in 0..130 * 4 {
            r.next();
        }
        let mut x = 0.0;
        while x < w {
            let bw = w * (0.012 + r.next() * 0.03);
            r.next();
            x += bw + w * 0.002;
        }
        self.drops = (0..720).map(|_| [r.next() * w, r.next() * h, 0.6 + r.next() * 0.8]).collect();
        self.size = (w, h);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(&mut self, win: &mut Window, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, now: f64, frames: f32, polished: bool, still: bool) {
        if self.size != (w, h) || self.drops.is_empty() {
            self.make(w, h);
        }
        let hz = h * 0.745;
        let code = num(st, &["weather", "code"]).unwrap_or(0.0) as i32;
        let wx = crate::ui::wx_kind(code);
        let (wind_x, wind_k) = wind(st);
        // precipCount()
        let mm = num(st, &["weather", "rain"]).unwrap_or(1.0) as f32;
        let base: f32 = match code { 51 => 0.15, 53 => 0.25, 55 => 0.35, 56 => 0.25, 57 => 0.4, 61 => 0.4, 63 => 0.65, 65 => 1.0, 66 => 0.5, 67 => 0.9, 71 => 0.35, 73 => 0.65, 75 => 1.0, 77 => 0.3, 80 => 0.45, 81 => 0.75, 82 => 1.0, 85 => 0.5, 86 => 0.9, 95 => 0.8, 96 => 0.9, 99 => 1.0, _ => 0.5 };
        let all = if polished { self.drops.len() } else { self.drops.len().min(180) };
        let n = ((all as f32) * base.max(mm / 6.0).min(1.0)).round() as usize;
        let u = h / 1080.0;
        let hail = code == 96 || code == 99;
        let rainy = wx == "rain" || wx == "storm";
        if rainy && polished {
            // drawRainDeep()
            let sl = if wind_x != 0.0 { -wind_x * 0.9 } else { 0.22 };
            let k2 = n as f32 / self.drops.len() as f32;
            if k2 > 0.3 {
                crate::scene::gradient(win, ox, oy, w, hz, pal.fg, 0.0, pal.fg, 0.07 * (k2 - 0.3) / 0.7); // a veil that hides the far city a little
            }
            let c = mix(pal.fg, pal.accent, 0.3);
            for l in 0..3 {
                let (lo, hi) = ([0.6, 0.86, 1.14][l], [0.86, 1.14, 1.41][l]);
                let mut segs: Vec<((f32, f32), (f32, f32))> = Vec::new();
                for i in 0..n {
                    let d = self.drops[i];
                    if d[2] < lo || d[2] >= hi {
                        continue;
                    }
                    let sp = d[2] * d[2];
                    let mut d = d;
                    if !still {
                        let dxw = h * (0.004 + 0.02 * wind_x) * sp;
                        d[1] += h * 0.02 * sp * frames;
                        d[0] += (if wind_x != 0.0 { dxw - h * 0.004 * sp } else { -h * 0.004 * sp }) * frames;
                    }
                    let ground = hz + (h - hz) * ((d[2] - 0.6) / 0.8).max(0.02); // the farther, the nearer the horizon it lands
                    if d[1] > ground || d[0] < -w * 0.1 || d[0] > w * 1.1 {
                        if d[1] > ground && self.ripples.len() < 70 && !still && self.rand() < 0.5 {
                            self.ripples.push((d[0], ground, now, d[2]));
                        }
                        d[1] = -10.0 - self.rand() * h * 0.2;
                        d[0] = self.rand() * w * 1.2 - w * 0.1;
                    }
                    self.drops[i] = d;
                    let len = h * 0.026 * sp + h * 0.008;
                    segs.push(((ox + d[0], oy + d[1]), (ox + d[0] + len * sl, oy + d[1] - len)));
                }
                // each drop a streak: the head brighter, the tail fading (two shapes a layer, not one per drop)
                let lw = ([1.0, 1.5, 2.3][l] * u).max(1.0);
                let al = [0.22, 0.36, 0.55][l];
                let mid = |a: (f32, f32), b: (f32, f32), k: f32| (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k);
                let heads: Vec<_> = segs.iter().map(|(a, b)| (*a, mid(*a, *b, 0.45))).collect();
                let tails: Vec<_> = segs.iter().map(|(a, b)| (mid(*a, *b, 0.45), *b)).collect();
                draw::lines(win, &tails, lw, c, al * 0.55);
                draw::lines(win, &heads, lw, mix(c, pal.fg, 0.35), (al * 1.25).min(1.0));
            }
            // the rings where they land, flat like the floor, and the splash
            self.ripples.retain(|r| (now - r.2) / 0.52 < 1.0);
            // in four steps of age, each one shape: the rings widen and fade, the splash jumps and falls
            for step in 0..4 {
                let (a0, a1) = (step as f32 / 4.0, (step + 1) as f32 / 4.0);
                let mut rings: Vec<((f32, f32), (f32, f32))> = Vec::new();
                let mut splash: Vec<((f32, f32), (f32, f32))> = Vec::new();
                for r in &self.ripples {
                    let a = ((now - r.2) / 0.52) as f32;
                    if a < a0 || a >= a1 {
                        continue;
                    }
                    let rr = h * 0.016 * r.3 * (0.25 + a);
                    let pts = crate::xf::ellipse_pts(ox + r.0, oy + r.1, rr, rr * 0.3, 0.0, std::f32::consts::TAU, 18);
                    rings.extend(pts.windows(2).map(|w2| (w2[0], w2[1])));
                    if a < 0.25 {
                        let y0 = oy + r.1 - h * 0.006 * (1.0 - a * 2.0);
                        splash.push(((ox + r.0, y0), (ox + r.0, y0 + h * 0.005)));
                    }
                }
                let am = (a0 + a1) / 2.0;
                draw::lines(win, &rings, u.max(1.0), pal.fg, 0.6 * (1.0 - am));
                draw::lines(win, &splash, (u * 1.5).max(1.0), pal.fg, 0.5 * (1.0 - am * 4.0).max(0.0));
            }
        } else if rainy {
            let mut plain = Vec::with_capacity(n);
            for i in 0..n {
                let mut d = self.drops[i];
                if !still {
                    let dxw = h * (0.004 + 0.02 * wind_x) * d[2];
                    d[1] += h * 0.022 * d[2] * frames;
                    d[0] += (if wind_x != 0.0 { dxw - h * 0.004 * d[2] } else { -h * 0.004 * d[2] }) * frames;
                }
                if d[1] > h || d[0] < -w * 0.1 || d[0] > w * 1.1 {
                    d[1] = -10.0;
                    d[0] = self.rand() * w * 1.2 - w * 0.1;
                }
                self.drops[i] = d;
                let sl = if wind_x != 0.0 { -wind_x * 0.9 } else { 0.22 };
                plain.push(((ox + d[0], oy + d[1]), (ox + d[0] + h * 0.018 * d[2] * sl, oy + d[1] - h * 0.018 * d[2])));
            }
            draw::lines(win, &plain, 1.0, pal.fg, 0.22);
        } else if wx == "snow" {
            for i in 0..n {
                let mut s = self.drops[i];
                if !still {
                    s[1] += h * 0.0018 * s[2] * frames;
                    s[0] += ((t + i as f32).sin() * 0.3 + wind_x * h * 0.005 * s[2]) * frames;
                }
                if s[1] > h || s[0] < -5.0 || s[0] > w + 5.0 {
                    s[1] = if s[1] > h { -5.0 } else { self.rand() * h };
                    s[0] = if s[0] < -5.0 { w } else if s[0] > w + 5.0 { 0.0 } else { self.rand() * w };
                }
                self.drops[i] = s;
                if polished {
                    let fs = (h / 1080.0 * (1.0 + s[2] * 2.2)).max(1.5);
                    crate::scene::disc(win, ox + s[0], oy + s[1], fs / 2.0, pal.fg, 0.6 * (0.35 + 0.45 * (s[2] - 0.6) / 0.8)); // a round quad: no shape to build
                } else {
                    crate::scene::rect(win, ox + s[0], oy + s[1], 2.0, 2.0, pal.fg, 0.6);
                }
            }
        } else if wx == "fog" {
            // createLinearGradient(0, hz - .15H, 0, hz + .1H): 0 at the top, .08 at 60%, 0 at the bottom
            let (y0, span) = (hz - h * 0.15, h * 0.25);
            crate::scene::gradient(win, ox, oy + y0, w, span * 0.6, pal.fg, 0.0, pal.fg, 0.08);
            crate::scene::gradient(win, ox, oy + y0 + span * 0.6, w, span * 0.4, pal.fg, 0.08, pal.fg, 0.0);
        }
        if hail {
            // white pellets that bounce on the ground
            for i in 0..40 {
                let d = self.drops[i];
                let hy = (d[1] * 3.0 + t * h * 0.9 * d[2]).rem_euclid(h - hz + h * 0.2);
                let yy = if hy < h * 0.2 { hy * 4.0 } else { hz + (hy - h * 0.2) * 0.2 };
                crate::scene::rect(win, ox + (d[0] * 1.3).rem_euclid(w), oy + yy.min(h - 2.0), 2.5, 2.5, pal.fg, 0.85);
            }
        }
        let _ = wind_k; // the gust lines of drawWind() are drawn by extra.rs, with the pollen
    }
}
