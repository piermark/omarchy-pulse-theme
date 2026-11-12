//! On the floor of Pulse 1.8, translated from pulse.js drawCars(): every device of your network is a pair of lights
//! running on the grid, toward you (traffic coming in) or toward the horizon (going out); a new device drives in from
//! the horizon again and again, blinking, with its address over it.

use crate::draw;
use crate::hub::{get, list, num, text};
use crate::palette::{mix, Pal};
use crate::scene::{radial, rect};
use gpui::Window;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct Floor {
    new_at: HashMap<i64, f32>,
    river_t: f32,
}

impl Floor {
    /// pulse.js drawRiver(): the energy of your house as a river across the plaza, blue from the grid, gold from the
    /// Sun, as wide as what the house uses; its current runs to the right, or back when you give energy to the grid.
    #[allow(clippy::too_many_arguments)]
    pub fn river(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, frames: f32, still: bool) {
        let e = get(st, &["energy"]);
        let Some(load) = num(e, &["load"]) else { return };
        let (load, solar, grid) = (load.max(0.0) as f32, num(e, &["solar"]).unwrap_or(0.0).max(0.0) as f32, num(e, &["grid"]).unwrap_or(0.0) as f32);
        let peak = num(e, &["peak"]).filter(|p| *p > 0.0).unwrap_or(6000.0) as f32;
        let f = if load > 0.0 { (solar / load).min(1.0) } else if solar > 0.0 { 1.0 } else { 0.0 };
        let col = mix([80.0 / 255.0, 160.0 / 255.0, 1.0], [1.0, 200.0 / 255.0, 80.0 / 255.0], f);
        let hz = h * 0.745;
        let wide = h * (0.008 + 0.03 * (load / peak).min(1.0));
        let yc = hz + (h - hz) * 0.42;
        let ys = |x: f32, k: f32| yc + (x / w * 6.0 + 1.0).sin() * h * 0.012 + k * wide;
        let mut pts: Vec<(f32, f32)> = Vec::new();
        let mut x = 0.0;
        while x <= w { pts.push((ox + x, oy + ys(x, -0.5))); x += 16.0; }
        let mut x = w;
        while x >= 0.0 { pts.push((ox + x, oy + ys(x, 0.5))); x -= 16.0; }
        draw::poly(win, &pts, col, 0.28);
        if !still {
            self.river_t += (0.0004 + 0.002 * (load / peak).min(1.0)) * if grid < 0.0 { -1.0 } else { 1.0 } * frames;
        }
        let dc = mix(col, pal.fg, 0.4);
        for k in 0..40 {
            let u = (k as f32 / 40.0 + self.river_t).rem_euclid(1.0);
            let px0 = u * w;
            let off = ((k as f32 * 0.618).rem_euclid(1.0) - 0.5) * 0.8;
            rect(win, ox + px0, oy + ys(px0, off), (h / 300.0).max(2.0), (h / 700.0).max(1.0), dc, 0.8);
        }
        if !get(st, &["screencast"]).as_bool().unwrap_or(false) {
            let kw = |v: f32| format!("{:.1} kW", v / 1000.0);
            let s = format!("☀ {} · home {} · grid {}{}", kw(solar), kw(load), if grid < 0.0 { "−" } else { "" }, kw(grid.abs()));
            draw::text(win, cx, font, &s, ox + w * 0.4, oy + ys(w * 0.4, -0.5) - h * 0.008, (h * 0.011).round(), 400.0, col, 0.9, -1);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn cars(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, bass: f32, kind: &str) {
        let lan = list(st, &["lan"]);
        if lan.is_empty() || kind == "calm" || get(st, &["ui", "cars"]).as_bool() == Some(false) {
            return;
        }
        let hz = h * 0.745;
        let cxm = w / 2.0;
        let share = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
        for (i, d) in lan.iter().take(14).enumerate() {
            let id = num(d, &["id"]).map(|v| v as i64).filter(|v| *v != 0).unwrap_or(i as i64);
            let hsh = ((id as f64 * 2654435761.0) % 4294967296.0 / 4294967296.0) as f32;
            let sign = if i % 2 == 1 { 1.0 } else { -1.0 };
            let lane = sign * (1.5 + ((i >> 1) % 8) as f32 + if hsh > 0.5 { 1.0 } else { 0.0 });
            let new = get(d, &["new"]).as_bool().unwrap_or(false);
            let toward = new || id % 3 != 0;
            let sp = 0.03 + hsh * 0.035 + bass * 0.05;
            let mut p = (t * sp + hsh * 7.0).rem_euclid(1.0);
            if !toward {
                p = 1.0 - p;
            }
            if new {
                let at = *self.new_at.entry(id).or_insert(t);
                p = ((t - at) * 0.11).rem_euclid(1.0);
            }
            let pos = |pp: f32| { let f = pp.powf(2.2); (cxm + lane * w * (0.012 + 0.098 * f), hz + (h - hz) * f, f) };
            let a = pos(p);
            let b = pos(if toward { (p - 0.06).max(0.0) } else { (p + 0.06).min(1.0) });
            let f = a.2;
            let lw = w * (0.012 + 0.098 * f);
            let sz = h * (0.0018 + 0.009 * f);
            let c = if new { pal.warn } else if toward { mix(pal.fg, pal.accent, 0.25) } else { pal.accent2 };
            let on = !new || (t * 3.0).rem_euclid(1.0) < 0.6;
            let al = (f * 10.0).min(1.0) * if on { 1.0 } else { 0.35 };
            for o in [-0.22f32, 0.22] {
                let x = a.0 + o * lw;
                let x2 = b.0 + o * w * (0.012 + 0.098 * b.2);
                // its light trail: bright at the car, nothing at the far end (a gradient, in a few steps)
                let n = 2;
                for k in 0..n {
                    let (u0, u1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
                    let seg = [(ox + x + (x2 - x) * u0, oy + a.1 + (b.1 - a.1) * u0), (ox + x + (x2 - x) * u1, oy + a.1 + (b.1 - a.1) * u1)];
                    draw::line(win, &seg, (sz * 0.9).max(0.6), c, 0.55 * al * (1.0 - (u0 + u1) / 2.0));
                }
                draw::hot(win, ox + x, oy + a.1, sz * 3.6, mix(c, pal.fg, 0.3), al);
            }
            if !share && f > 0.12 && f < 0.92 && (new || !text(d, &["name"]).is_empty()) {
                let label = if new { format!("new · {}", text(d, &["ip"])) } else { text(d, &["name"]) };
                let fsz = (h * (0.009 + f * 0.007)).round();
                draw::text(win, cx, font, &label, ox + a.0, oy + a.1 - sz * 3.4, fsz, 600.0, if new { pal.warn } else { pal.fg }, (if new { 0.95 } else { 0.5 }) * ((0.92 - f) * 6.0).min(1.0), 0);
            }
        }
    }
}

/// Points along a canvas path of lines and quadratic curves (moveTo, lineTo, quadraticCurveTo).
struct Pen(Vec<(f32, f32)>);
impl Pen {
    fn to(mut self, x: f32, y: f32) -> Self {
        self.0.push((x, y));
        self
    }
    fn quad(mut self, cx: f32, cy: f32, x: f32, y: f32) -> Self {
        let (x0, y0) = *self.0.last().unwrap_or(&(x, y));
        for k in 1..=10 {
            let u = k as f32 / 10.0;
            self.0.push(((1.0 - u) * (1.0 - u) * x0 + 2.0 * (1.0 - u) * u * cx + u * u * x, (1.0 - u) * (1.0 - u) * y0 + 2.0 * (1.0 - u) * u * cy + u * u * y));
        }
        self
    }
}

/// pulse.js drawCar(): your electric car parked on the plaza, its battery as a light bar along its side, the charger
/// and the energy running through its cable while it charges (the hub reads the car: Tesla and the others).
#[allow(clippy::too_many_arguments)]
pub fn car(win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, h: f32, t: f32, band: (f32, f32), car_age: f32, kind: &str) {
    let car = get(st, &["car"]);
    if !car.is_object() || get(car, &["driving"]).as_bool().unwrap_or(false) || kind == "orbit" || kind == "calm" {
        return;
    }
    let hz = h * 0.745;
    let u = h / 1080.0 * 1.4;
    let (x, y, l, hc) = (ox + band.0 + (band.1 - band.0) * 0.3, oy + hz + h * 0.08, 150.0 * u, 26.0 * u);
    let b = (num(car, &["battery"]).unwrap_or(0.0) as f32).clamp(0.0, 100.0);
    let state = text(car, &["state"]);
    let (ch, full) = (state == "Charging", state == "Complete");
    let col = if b < 20.0 { pal.crit } else if b < 40.0 { pal.warn } else { pal.ok };
    // the body: a generic low sedan
    let body = Pen(vec![(x - l * 0.5, y - hc * 0.35)])
        .quad(x - l * 0.5, y - hc * 0.75, x - l * 0.36, y - hc * 0.8)
        .to(x - l * 0.2, y - hc * 0.85)
        .quad(x - l * 0.05, y - hc * 1.45, x + l * 0.14, y - hc * 1.4)
        .quad(x + l * 0.32, y - hc * 1.3, x + l * 0.44, y - hc * 0.8)
        .quad(x + l * 0.52, y - hc * 0.7, x + l * 0.5, y - hc * 0.3)
        .to(x + l * 0.5, y - hc * 0.18)
        .to(x - l * 0.5, y - hc * 0.18)
        .0;
    draw::poly(win, &body, mix(pal.bg, pal.accent2, 0.18), 1.0);
    let mut closed = body.clone();
    closed.push(body[0]);
    draw::line(win, &closed, (1.4 * u).max(1.0), pal.accent2, 0.85);
    let glass = Pen(vec![(x - l * 0.14, y - hc * 0.86)]).quad(x, y - hc * 1.32, x + l * 0.14, y - hc * 1.3).quad(x + l * 0.28, y - hc * 1.2, x + l * 0.36, y - hc * 0.86).0;
    draw::poly(win, &glass, pal.accent, 0.35);
    for k in [-0.3f32, 0.3] {
        draw::arc_fill(win, x + l * k, y - hc * 0.18, hc * 0.3, hc * 0.3, 0.0, 0.0, std::f32::consts::TAU, pal.bg, 1.0);
        draw::arc(win, x + l * k, y - hc * 0.18, hc * 0.3, hc * 0.3, 0.0, std::f32::consts::TAU, 1.0, pal.fg, 0.6);
    }
    rect(win, x + l * 0.46, y - hc * 0.62, l * 0.04, hc * 0.12, pal.warn, 0.9); // lights
    rect(win, x - l * 0.5, y - hc * 0.62, l * 0.035, hc * 0.12, pal.crit, 0.9);
    // the light bar along the side: the battery
    let (bw, bx, by) = (l * 0.7, x - l * 0.35, y - hc * 0.62);
    rect(win, bx, by, bw, hc * 0.14, pal.bg, 0.8);
    let gl = if ch { 0.75 + 0.25 * (t * 4.0).sin() } else { 1.0 };
    rect(win, bx, by, bw * b / 100.0, hc * 0.14, col, 0.9 * gl);
    if let Some(limit) = num(car, &["limit"]).filter(|v| *v > 0.0) {
        rect(win, bx + bw * limit as f32 / 100.0, by - hc * 0.06, u.max(1.0), hc * 0.26, pal.fg, 0.8);
    }
    if ch || full || state == "Stopped" {
        // the charger and its cable
        let px0 = x - l * 0.72;
        rect(win, px0 - 7.0 * u, y - 46.0 * u, 14.0 * u, 46.0 * u, mix(pal.bg, pal.fg, 0.2), 1.0);
        rect(win, px0 - 4.0 * u, y - 40.0 * u, 8.0 * u, 6.0 * u, if ch { pal.ok } else if full { pal.accent } else { pal.dim }, 0.95);
        let cable = Pen(vec![(px0, y - 28.0 * u)]).quad(px0 + l * 0.12, y + 4.0 * u, x - l * 0.48, y - hc * 0.5).0;
        draw::line(win, &cable, 2.0 * u, pal.fg, 0.5);
        if ch {
            for k in 0..4 {
                // energy flowing into the car
                let q = (t * 0.8 + k as f32 / 4.0).rem_euclid(1.0);
                let qx = px0 + (x - l * 0.48 - px0) * q;
                let qy = (1.0 - q) * (1.0 - q) * (y - 28.0 * u) + 2.0 * (1.0 - q) * q * (y + 4.0 * u) + q * q * (y - hc * 0.5);
                radial(win, qx, qy, 0.0, 6.0 * u, 1.0, pal.ok, &[(0.0, 0.95), (1.0, 0.0)]);
            }
        }
    }
    let fresh = (1.0 - car_age / 5.0).max(0.0);
    if fresh > 0.0 {
        radial(win, x, y - hc, 0.0, l * 0.7, 1.0, col, &[(0.0, 0.35 * fresh), (1.0, 0.0)]);
    }
    if !(get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false)) {
        let range = num(car, &["range"]).map(|r| format!(" · {} {}", r.round(), text(car, &["unit"]))).unwrap_or_default();
        let extra = if ch { format!(" · ⚡{}", num(car, &["power"]).filter(|p| *p > 0.0).map(|p| format!(" {} kW", p.round())).unwrap_or_default()) } else if full { " · charged".to_string() } else { String::new() };
        draw::text(win, cx, font, &format!("{}%{range}{extra}", b.round()), x, y + h * 0.022, (h * 0.012).round(), 600.0, col, 0.95, 0);
    }
}
