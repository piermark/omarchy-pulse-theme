//! Your AI agents as Pulse 1.8 draws them, translated from pulse.js drawDrones(): a quadcopter each, flying over the
//! city between the panels, its light and its scanning beam in the colour of what it is doing; it lands when its work
//! is done, and its tag (name, what it is doing, for how long, how healthy) follows it.

use crate::draw;
use crate::hub::{get, list, num, text};
use crate::palette::{mix, Pal, C};
use crate::scene::{radial, rect};
use gpui::Window;
use serde_json::Value;
use std::collections::HashMap;

struct Drone {
    ph: f32,
    sp: f32,
    land: f32,
    boom: bool,
}

/// A tag to draw over everything (on the web page it is a line of HTML over the canvas).
pub struct Tag {
    x: f32,
    y: f32,
    parts: Vec<(String, C, f32)>,
    dot: C,
    border: C,
}

#[derive(Default)]
pub struct Drones {
    all: HashMap<String, Drone>,
    seed: u64,
    pub tags: Vec<Tag>,
    /// where a drone has just landed: pulse.js bursts a few sparks there
    pub landed: Vec<(f32, f32, C)>,
}

fn rot(x: f32, y: f32, a: f32) -> (f32, f32) {
    (x * a.cos() - y * a.sin(), x * a.sin() + y * a.cos())
}

impl Drones {
    #[allow(clippy::too_many_arguments)]
    pub fn paint(&mut self, win: &mut Window, st: &Value, pal: &Pal, ox: f32, oy: f32, h: f32, t: f32, band: (f32, f32), focus_k: f32, still: bool) {
        self.tags.clear();
        let hz = h * 0.745;
        let agents = if get(st, &["ui", "agentsAs"]).as_str() == Some("robots") { &[][..] } else { list(st, &["agents"]) };
        let (half, mx) = ((band.1 - band.0) / 2.0, (band.0 + band.1) / 2.0);
        let hub_now = num(st, &["t"]).unwrap_or(0.0);
        let mut seen = Vec::new();
        let mut groups: HashMap<String, Vec<(f32, f32, C)>> = HashMap::new();
        for (idx, a) in agents.iter().enumerate() {
            let name = text(a, &["name"]);
            seen.push(name.clone());
            let n = self.all.len();
            let d = self.all.entry(name.clone()).or_insert_with(|| {
                self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let r1 = ((self.seed >> 33) % 1000) as f32 / 1000.0;
                let r2 = ((self.seed >> 13) % 1000) as f32 / 1000.0;
                Drone { ph: n as f32 * 2.4 + r1 * 0.6, sp: 0.7 + r2 * 0.5, land: 0.0, boom: false }
            });
            let state = text(a, &["state"]);
            let down = state == "done" || state == "failed" || (focus_k > 0.5 && state != "waiting");
            d.land = (d.land + if down { 0.012 } else { -0.03 }).clamp(0.0, 1.0);
            let x = mx + half * 0.82 * (t * 0.11 * d.sp + d.ph).sin();
            let y0 = h * (0.5 + 0.035 * (t * 0.23 * d.sp + d.ph * 1.7).sin()) + (idx % 3) as f32 * h * 0.045;
            let y = y0 + (hz - h * 0.04 - y0) * (d.land * d.land);
            let runs = num(a, &["runs"]).unwrap_or(0.0);
            let health = num(a, &["health"]).unwrap_or(1.0) as f32;
            let hk = if runs >= 3.0 { health } else { 1.0 };
            let sick = runs >= 3.0 && health < 0.6;
            let tilt = (t * 0.11 * d.sp + d.ph).cos() * 0.18;
            let s = h * 0.042 * (0.78 + 0.3 * hk);
            let col = match state.as_str() { "working" => pal.accent, "waiting" => pal.warn, "done" => pal.ok, "failed" => pal.crit, "idle" => pal.dim, _ => pal.accent };
            let blink_on = (state != "waiting" || (t * 2.5).rem_euclid(1.0) < 0.55) && !(sick && !still && (t * 7.3 + d.ph).rem_euclid(1.9) < 0.35);
            let group = text(a, &["group"]);
            if !group.is_empty() && (state == "working" || state == "waiting") && d.land < 0.5 {
                groups.entry(group).or_default().push((x, y, col));
            }
            let (gx, gy) = (ox + x, oy + y);
            // the beam: a working drone scans the city
            if (state == "working" || state == "waiting") && d.land < 0.5 {
                let sw = (t * 1.6 + d.ph).sin() * s * 1.4;
                let top_a = if state == "waiting" && !blink_on { 0.08 } else { 0.32 };
                let steps = 4;
                for k in 0..steps {
                    let (u0, u1) = (k as f32 / steps as f32, (k + 1) as f32 / steps as f32);
                    let at = |u: f32, side: f32| {
                        let yy = gy + s * 0.25 + (oy + hz - gy - s * 0.25) * u;
                        (gx + side * (s * 0.25 + (s * 1.65) * u) + sw * u, yy)
                    };
                    let (l0, r0, l1, r1) = (at(u0, -1.0), at(u0, 1.0), at(u1, -1.0), at(u1, 1.0));
                    draw::poly(win, &[l0, r0, r1, l1], col, top_a * (1.0 - (u0 + u1) / 2.0));
                }
            }
            // its glow, in the colour of its state
            radial(win, gx, gy, 0.0, s * 2.6, 1.0, col, &[(0.0, if blink_on { 0.5 } else { 0.15 }), (1.0, 0.0)]);
            // a quadcopter seen slightly from below: four arms, four spinning rotors
            let p = |px0: f32, py0: f32| { let (rx, ry) = rot(px0, py0, tilt); (gx + rx, gy + ry) };
            let arm_c = mix(pal.bg, pal.fg, 0.55);
            for ar in [(-1.25f32, -0.42f32, 0.9f32), (1.25, -0.42, 0.9), (-1.05, 0.22, 1.0), (1.05, 0.22, 1.0)] {
                draw::line(win, &[p(0.0, 0.0), p(ar.0 * s, ar.1 * s)], (s * 0.12).max(1.5), arm_c, 1.0);
                let spin = (0.55 + 0.45 * (t * 37.0 + ar.0 * 3.0).sin().abs()) * s * 0.62 * ar.2;
                let c0 = p(ar.0 * s, ar.1 * s - s * 0.08);
                draw::arc_fill(win, c0.0, c0.1, spin, spin * 0.22, tilt, 0.0, std::f32::consts::TAU, pal.fg, 0.12);
                let ring: Vec<(f32, f32)> = (0..=20).map(|k| { let a2 = k as f32 / 20.0 * std::f32::consts::TAU; let (rx, ry) = rot(a2.cos() * spin, a2.sin() * spin * 0.22, tilt); (c0.0 + rx, c0.1 + ry) }).collect();
                draw::line(win, &ring, (s * 0.05).max(1.0), pal.fg, 0.6);
            }
            draw::arc_fill(win, gx, gy, s * 0.62, s * 0.3, tilt, 0.0, std::f32::consts::TAU, mix(pal.bg, pal.fg, 0.35), 1.0);
            let crew = get(a, &["crew"]).as_bool().unwrap_or(false);
            let body: Vec<(f32, f32)> = (0..=28).map(|k| { let a2 = k as f32 / 28.0 * std::f32::consts::TAU; p(a2.cos() * s * 0.62, a2.sin() * s * 0.3) }).collect();
            draw::line(win, &body, (s * if crew { 0.12 } else { 0.07 }).max(1.0), if crew { pal.accent2 } else { col }, 0.95);
            let eye = p(0.0, s * 0.12);
            draw::arc_fill(win, eye.0, eye.1, s * 0.16, s * 0.16, 0.0, 0.0, std::f32::consts::TAU, col, if blink_on { 1.0 } else { 0.3 });
            if blink_on {
                draw::hot(win, eye.0, eye.1, s * 0.7, col, 0.8); // its eye burns like a lamp
            }
            if (t * 1.3 + d.ph).rem_euclid(1.0) < 0.15 {
                // navigation lights
                let (l, r) = (p(-s * 1.32, -s * 0.5), p(s * 1.18, -s * 0.5));
                rect(win, l.0, l.1, s * 0.14, s * 0.14, pal.crit, 1.0);
                rect(win, r.0, r.1, s * 0.14, s * 0.14, pal.ok, 1.0);
            }
            if d.land > 0.95 && !d.boom {
                d.boom = true;
                self.landed.push((x, y, col));
            }
            if !down {
                d.boom = false;
            }
            // its tag, under it
            let since = num(a, &["since"]).or_else(|| num(a, &["t"])).unwrap_or(hub_now);
            let mins = ((hub_now - since) / 60.0).max(0.0).floor() as i64;
            let mut parts = Vec::new();
            if crew {
                parts.push(("crew ".to_string(), pal.accent2, 1.0));
            }
            parts.push((name.clone(), pal.fg, 1.0));
            let what = text(a, &["text"]);
            if !what.is_empty() && !crew {
                parts.push((format!(" {what}"), pal.fg, 0.6));
            }
            if state == "working" && mins > 0 {
                parts.push((format!(" {mins}m"), pal.accent2, 1.0));
            }
            if state == "waiting" {
                parts.push((" !".to_string(), pal.warn, 1.0));
            }
            if runs >= 3.0 {
                parts.push((format!(" ok {}%", (health * 100.0).round()), if sick { pal.crit } else { pal.ok }, 0.85));
            }
            let border = match state.as_str() { "waiting" => (pal.warn, 1.0), "done" => (pal.ok, 1.0), "failed" => (pal.crit, 1.0), _ => (pal.accent, 0.35) };
            crate::agents::spot(&name, ox + x, oy + y, s * 1.8);
            crate::agents::spot(&name, ox + x, oy + y + s * 0.9 + h * 0.012, s * 1.4); // its tag too
            self.tags.push(Tag { x: ox + x, y: oy + y + s * 0.9, parts, dot: col, border: mix(pal.bg, border.0, border.1) });
        }
        self.all.retain(|k, _| seen.contains(k));
        // 1.6: agents working in the same repo talk: a line of light between them, a packet running along it
        for m in groups.values().filter(|m| m.len() >= 2) {
            for j in 1..m.len() {
                let (a0, a1) = (m[j - 1], m[j]);
                let (cxm, cym) = ((a0.0 + a1.0) / 2.0, a0.1.min(a1.1) - h * 0.04);
                let q = |u: f32| ((1.0 - u) * (1.0 - u) * a0.0 + 2.0 * (1.0 - u) * u * cxm + u * u * a1.0, (1.0 - u) * (1.0 - u) * a0.1 + 2.0 * (1.0 - u) * u * cym + u * u * a1.1);
                // dashed: every other piece of the curve
                for k in 0..24 {
                    if k % 2 == 1 { continue }
                    let (p0, p1) = (q(k as f32 / 24.0), q((k + 1) as f32 / 24.0));
                    draw::line(win, &[(ox + p0.0, oy + p0.1), (ox + p1.0, oy + p1.1)], (h / 700.0).max(1.0), mix(a0.2, a1.2, 0.5), 0.35);
                }
                if !still {
                    for qq in 0..2 {
                        let u = (t * 0.45 + qq as f32 * 0.5 + j as f32 * 0.3).rem_euclid(1.0);
                        let uu = if qq % 2 == 1 { 1.0 - u } else { u };
                        let pp = q(uu);
                        radial(win, ox + pp.0, oy + pp.1, 0.0, h * 0.009, 1.0, pal.fg, &[(0.0, 0.95), (1.0, 0.0)]);
                    }
                }
            }
        }
    }

    /// The tags, over the scene (pulse.css .drone: a dark box, a thin border, a dot in the state's colour).
    pub fn tags(&self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, pal: &Pal, rem: f32) {
        let fs = (0.85 * rem).round();
        let (px0, py0) = (0.55 * rem, 0.15 * rem);
        for tg in &self.tags {
            let dot = 0.45 * rem;
            let widths: Vec<f32> = tg.parts.iter().map(|p| draw::measure(win, font, &p.0, fs, 400.0)).collect();
            let tw = dot + 0.45 * rem + widths.iter().sum::<f32>();
            let (bw, bh) = (tw + px0 * 2.0 + 2.0, fs * 1.25 + py0 * 2.0 + 2.0);
            let x0 = (tg.x - bw / 2.0).round();
            rect(win, x0, tg.y, bw, bh, pal.bg, 0.78);
            draw::frame(win, x0, tg.y, bw, bh, 1.0, tg.border, 1.0);
            let base = tg.y + 1.0 + py0 + fs * 0.95;
            draw::arc_fill(win, x0 + 1.0 + px0 + dot / 2.0, base - fs * 0.32, dot / 2.0, dot / 2.0, 0.0, 0.0, std::f32::consts::TAU, tg.dot, 1.0);
            let mut x = x0 + 1.0 + px0 + dot + 0.45 * rem;
            for (p, wdt) in tg.parts.iter().zip(widths) {
                draw::text(win, cx, font, &p.0, x, base, fs, 400.0, p.1, p.2, -1);
                x += wdt;
            }
        }
    }
}
