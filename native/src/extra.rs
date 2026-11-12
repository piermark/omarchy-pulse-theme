//! The rest of Pulse 1.8's world, as pulse.js draws it: eclipses, the seasons and the holidays, thunderstorms and
//! the storms over what is failing, meteor showers, earthquakes, the air, the wind, the garden of your days, your
//! friends' towers, the parcels of your updates, the tower of your workspaces with its ivy, the firewall's wall,
//! the lines you type in `omarchy-pulse term`. Every piece reads the hub's state; nothing here asks the network.

use crate::draw::{c8, Label, PI, TAU};
use crate::hub::{get, list, num, text};
use crate::life::Frame;
use crate::palette::{mix, Pal, C};
use crate::scene::Rand;
use gpui::Window;
use serde_json::Value;
use std::collections::HashMap;

pub struct Bolt {
    t: f64,
    pts: Option<Vec<(f32, f32)>>,
    x: f32,
}

pub struct Mem {
    r: Rand,
    bolts: Vec<Bolt>,
    next_bolt: f64,
    flakes: Vec<[f32; 3]>,
    /// earthquakes: when, magnitude, from where (azimuth), near?
    quakes: Vec<(f64, f32, f32, bool)>,
    tsunamis: Vec<(f64, f32, bool)>,
    gusts: Vec<[f32; 4]>,
    pollen: Vec<[f32; 4]>,
    infra: HashMap<String, (f64, f64, Vec<(f32, f32)>)>,
    lift: f32,
    wifi: HashMap<String, (f32, f32)>,
    truck: HashMap<String, f32>,
    ny_fx: f64,
    flare: f64,
    pub build: f64,
    pub ring: f64,
    ringing: bool,
    /// effects this module asks the world for (fireworks at midnight, a meteor): scene.rs plays them
    pub asks: Vec<&'static str>,
    /// a flash over the whole sky, 0..1, set by a bolt
    pub flash: f32,
    /// the storms over your failing panels: (the panel's id, x, y, how wide, its title), for a click
    pub storms: Vec<(String, f32, f32, f32, String)>,
}

impl Default for Mem {
    fn default() -> Self {
        Mem { r: Rand::new(4711), bolts: Vec::new(), next_bolt: 0.0, flakes: Vec::new(), quakes: Vec::new(), tsunamis: Vec::new(), gusts: Vec::new(), pollen: Vec::new(), infra: HashMap::new(), lift: 0.0, wifi: HashMap::new(), truck: HashMap::new(), ny_fx: 0.0, flare: -1e9, build: -1e9, ring: -1e9, ringing: false, asks: Vec::new(), flash: 0.0, storms: Vec::new() }
    }
}

struct G<'a> {
    win: &'a mut Window,
    ox: f32,
    oy: f32,
}
impl G<'_> {
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: C, a: f32) { crate::scene::rect(self.win, self.ox + x, self.oy + y, w, h, c, a) }
    fn grad(&mut self, x: f32, y: f32, w: f32, h: f32, top: C, ta: f32, bot: C, ba: f32) { if w > 0.0 && h > 0.0 { crate::scene::gradient(self.win, self.ox + x, self.oy + y, w, h, top, ta, bot, ba) } }
    fn glow(&mut self, x: f32, y: f32, r: f32, c: C, a: f32) { if a > 0.01 && r > 0.0 { crate::scene::glow(self.win, self.ox + x, self.oy + y, r, c, a.min(1.0)) } }
    fn disc(&mut self, x: f32, y: f32, r: f32, c: C, a: f32) { crate::scene::disc(self.win, self.ox + x, self.oy + y, r, c, a) }
    fn ell(&mut self, x: f32, y: f32, rx: f32, ry: f32, rot: f32, c: C, a: f32) { crate::draw::arc_fill(self.win, self.ox + x, self.oy + y, rx, ry, rot, 0.0, TAU, c, a) }
    #[allow(clippy::too_many_arguments)]
    fn arc(&mut self, x: f32, y: f32, rx: f32, ry: f32, a0: f32, a1: f32, lw: f32, c: C, a: f32) { crate::draw::arc(self.win, self.ox + x, self.oy + y, rx, ry, a0, a1, lw, c, a) }
    fn poly(&mut self, pts: &[(f32, f32)], c: C, a: f32) { let p: Vec<(f32, f32)> = pts.iter().map(|p| (p.0 + self.ox, p.1 + self.oy)).collect(); crate::draw::poly(self.win, &p, c, a) }
    fn line(&mut self, pts: &[(f32, f32)], lw: f32, c: C, a: f32) { let p: Vec<(f32, f32)> = pts.iter().map(|p| (p.0 + self.ox, p.1 + self.oy)).collect(); crate::draw::line(self.win, &p, lw, c, a) }
    fn frame(&mut self, x: f32, y: f32, w: f32, h: f32, lw: f32, c: C, a: f32) { crate::draw::frame(self.win, self.ox + x, self.oy + y, w, h, lw, c, a) }
}

fn lab(labels: &mut Vec<Label>, x: f32, y: f32, size: f32, c: C, a: f32, text: String, bold: bool) {
    if !text.is_empty() && a > 0.02 {
        labels.push(Label { x, y, size, c, a, text, bold });
    }
}

/// The free band between your panels, as the web page measures it.
pub fn band(f: &Frame) -> (f32, f32) {
    // pulse.js: from the right edge of the left column to the left edge of the right one, 30 px of air each side
    let r = crate::ui::rem(f.w, f.h);
    let b = (27.2 * r + 30.0, f.w - 27.2 * r - 30.0);
    if b.1 - b.0 > f.w * 0.25 { b } else { (f.w * 0.08, f.w * 0.92) }
}

fn private(st: &Value) -> bool {
    get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false)
}

fn bolt_path(r: &mut Rand, x: f32, f: &Frame, to: f32) -> Vec<(f32, f32)> {
    let mut pts = vec![(x + (r.next() - 0.5) * f.w * 0.05, f.h * 0.12)];
    let mut y = f.h * 0.12;
    while y < to {
        y += f.h * (0.03 + r.next() * 0.05);
        let last = pts[pts.len() - 1].0;
        pts.push((last + (r.next() - 0.5) * f.w * 0.03, y.min(to)));
    }
    pts
}

/// An effect of the hub that belongs to this module: an earthquake, a tsunami, a solar flare, a new building, the
/// doorbell, a bolt asked by a rule.
pub fn event(mem: &mut Mem, kind: &str, raw: &Value, t: f64, f: Option<&Frame>) {
    match kind {
        "quake" => mem.quakes.push((t, num(raw, &["mag"]).unwrap_or(5.0) as f32, num(raw, &["az"]).unwrap_or(90.0) as f32, (get(raw, &["near"]).as_bool().unwrap_or(false) || num(raw, &["near"]).unwrap_or(0.0) > 0.0))),
        "tsunami" => mem.tsunamis.push((t, num(raw, &["az"]).unwrap_or(90.0) as f32, (get(raw, &["near"]).as_bool().unwrap_or(false) || num(raw, &["near"]).unwrap_or(0.0) > 0.0))),
        "flare" => mem.flare = t,
        "build" => mem.build = t,
        "doorbell" => mem.ring = t,
        "lightning" => {
            if let Some(f) = f {
                let b = band(f);
                // a bolt asked for one place (a hall of the datacenter that went down) strikes its roof
                let x = num(raw, &["x"]).map(|v| v as f32).unwrap_or_else(|| b.0 + (b.1 - b.0) * (0.2 + mem.r.next() * 0.6));
                let to = num(raw, &["y"]).map(|v| v as f32).unwrap_or(f.hz);
                let pts = bolt_path(&mut mem.r, x, f, to);
                mem.bolts.push(Bolt { t, pts: Some(pts), x });
                mem.flash = 1.0;
            }
        }
        _ => {}
    }
}

/// How far an earthquake moves the whole picture now.
pub fn shake(mem: &mut Mem, t: f64, h: f32) -> (f32, f32) {
    let (mut dx, mut dy) = (0.0f32, 0.0f32);
    mem.quakes.retain(|q| (t - q.0) as f32 <= 1.5 + q.1 * 0.5 + 5.0);
    for (i, q) in mem.quakes.iter().enumerate() {
        let (age, dur) = ((t - q.0) as f32, 1.5 + q.1 * 0.5);
        if age > dur { continue }
        let a = (1.0 - age / dur).max(0.0) * h * 0.0025 * 1.6f32.powf(q.1 - 3.0) * if q.3 { 2.0 } else { 0.7 };
        let ms = (t * 1000.0) as f32;
        dx += (ms * 0.09 + i as f32).sin() * a;
        dy += (ms * 0.13 + i as f32 * 2.0).cos() * a * 0.6;
    }
    (dx.clamp(-h * 0.02, h * 0.02), dy.clamp(-h * 0.012, h * 0.012))
}

/// Over the sky, after the Sun and the Moon: the haze of poor air, a far storm's light, a solar flare.
pub fn sky(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, _labels: &mut Vec<Label>) {
    let mut g = G { win, ox: f.ox, oy: f.oy };
    let (w, h, hz) = (f.w, f.h, f.hz);
    // (the eclipses are drawn with the Sun and the Moon, heavens.rs, as pulse.js does)
    if (t - mem.flare) < 2.5 {
        g.rect(0.0, 0.0, w, hz, pal.warn, 0.18 * (1.0 - ((t - mem.flare) / 2.5) as f32)); // a solar flare
    }
    // the air: a haze over the city when it is poor
    let haze = match text(st, &["air", "level"]).as_str() { "moderate" => 0.07, "poor" => 0.16, "very poor" => 0.24, "extremely poor" => 0.32, _ => 0.0 };
    if haze > 0.0 {
        g.grad(0.0, hz - h * 0.35, w, h * 0.28, c8(150, 135, 110), 0.0, c8(150, 135, 110), haze);
        g.grad(0.0, hz - h * 0.07, w, h * 0.12, c8(150, 135, 110), haze, c8(150, 135, 110), haze * 0.6);
    }
    // thunderstorms: one over you throws bolts on the city; one around you lights the horizon from where it is
    let cells = list(st, &["storms", "cells"]);
    let code = num(st, &["weather", "code"]).unwrap_or(0.0) as i32;
    let here = code >= 95 || cells.iter().any(|c| num(c, &["km"]).unwrap_or(0.0) == 0.0);
    let still = get(st, &["ui", "motion"]).as_str() == Some("reduced");
    if (here || !cells.is_empty()) && t >= mem.next_bolt && !still {
        let lpi = num(st, &["storms", "lpi"]).unwrap_or(0.0) as f32;
        let rate = if here { 9.0 } else { 6.0 } / (1.0 + (lpi / 40.0).min(2.0));
        mem.next_bolt = t + (rate * (0.4 + mem.r.next() * 1.2)) as f64;
        let b = band(f);
        if here {
            let x = b.0 + (b.1 - b.0) * (0.08 + mem.r.next() * 0.84);
            let pts = bolt_path(&mut mem.r, x, f, hz);
            mem.bolts.push(Bolt { t, pts: Some(pts), x });
            mem.flash = 1.0;
            mem.asks.push("thunder");
        } else {
            let c = &cells[((mem.r.next() * cells.len() as f32) as usize).min(cells.len() - 1)];
            let az = num(c, &["az"]).unwrap_or(180.0) as f32;
            let x = if (75.0..=285.0).contains(&az) { w / 2.0 + (az - 180.0) / 90.0 * w * 0.44 } else if az.to_radians().sin() > 0.0 { w * 0.04 } else { w * 0.96 };
            mem.bolts.push(Bolt { t, pts: None, x });
            mem.flash = 0.35;
        }
    }
    for b in mem.bolts.iter().filter(|b| b.pts.is_none()) {
        let age = (t - b.t) as f32;
        if age < 0.6 {
            let a = if age < 0.08 || (0.16..0.22).contains(&age) { 1.0 } else { 0.25 };
            g.glow(b.x, hz - h * 0.08, w * 0.18, pal.fg, 0.5 * a);
        }
    }
}

/// On the ground of the skyline: the garden of your days and its monuments, your friends' towers, the parcels of
/// your updates, the tower of your workspaces with the ivy of your uptime, the firewall's wall, the tree.
#[allow(clippy::too_many_arguments)]
pub fn ground(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, labels: &mut Vec<Label>, kind: &str) {
    let mut g = G { win, ox: f.ox, oy: f.oy };
    let (w, h, hz, u) = (f.w, f.h, f.hz, f.u);
    let (b0, b1) = band(f);
    let still = get(st, &["ui", "motion"]).as_str() == Some("reduced");
    let tf = if still { 0.0 } else { t as f32 };
    let hide = private(st);
    let (bg, fg, ac, a2) = (pal.bg, pal.fg, pal.accent, pal.accent2);

    // ---- the garden: a small building for each day you worked, the newest at the front; a monument every 7 days in a row
    let recent = list(st, &["growth", "recent"]);
    if !recent.is_empty() && kind != "orbit" {
        let (n, lit, y) = (recent.len(), get(st, &["growth", "lit"]).as_bool().unwrap_or(false), hz + h * 0.035);
        let unit = (w * 0.011).min((b1 - b0) * 0.22 / 24.0);
        let newest = (1.0 - ((t - mem.build) / 6.0) as f32).max(0.0);
        for (i, b) in recent.iter().enumerate() {
            let (right, k) = ((n - 1 - i) % 2 == 1, (n - 1 - i) / 2);
            if k >= 24 { continue }
            let x = if right { b1 - unit * 1.3 * (k as f32 + 1.0) } else { b0 + unit * 1.3 * (k as f32 + 1.0) };
            let fresh = if i == n - 1 { newest } else { 0.0 };
            let rise = if i == n - 1 && newest > 0.0 { (((t - mem.build) / 1.5) as f32).min(1.0) } else { 1.0 };
            let hh = h * (0.03 + (num(b, &["m"]).unwrap_or(0.0) as f32 / 900.0).min(0.1)) * rise;
            g.rect(x - unit / 2.0, y - hh, unit, hh, mix(mix(bg, a2, 0.25), fg, fresh * 0.5), 1.0);
            g.frame(x - unit / 2.0, y - hh, unit, hh, (h / 900.0).max(1.0), a2, 0.5 + fresh * 0.5);
            if lit {
                let mut wy = y - hh + h * 0.008;
                while wy < y - h * 0.006 {
                    let c = if (i * 7 + wy.round() as usize) % 3 > 0 { pal.warn } else { ac };
                    g.rect(x - unit * 0.25, wy, unit * 0.18, h * 0.005, c, 0.55 + fresh * 0.4);
                    g.rect(x + unit * 0.08, wy, unit * 0.18, h * 0.005, c, 0.55 + fresh * 0.4);
                    wy += h * 0.012;
                }
            }
        }
        for mi in 0..(num(st, &["growth", "monuments"]).unwrap_or(0.0) as usize).min(8) {
            let step = unit * 1.3 * ((n as f32 / 2.0).ceil() + 2.0 + (mi / 2) as f32 * 2.0);
            let mx = if mi % 2 == 1 { b1 - step } else { b0 + step };
            let mh = h * (0.16 + 0.02 * mi as f32);
            g.poly(&[(mx - unit * 0.8, y), (mx - unit * 0.4, y - mh * 0.85), (mx, y - mh), (mx + unit * 0.4, y - mh * 0.85), (mx + unit * 0.8, y)], mix(bg, a2, 0.35), 1.0);
            g.line(&[(mx - unit * 0.8, y), (mx - unit * 0.4, y - mh * 0.85), (mx, y - mh), (mx + unit * 0.4, y - mh * 0.85), (mx + unit * 0.8, y)], u, a2, if lit { 0.95 } else { 0.4 });
            if lit { g.glow(mx, y - mh, h * 0.03, a2, 0.9) }
        }
        if newest > 0.0 { g.glow(b0 + unit * 1.3, y - h * 0.05, h * 0.08, a2, 0.5 * newest) }
    }
    let city = !matches!(kind, "calm" | "orbit" | "noir");
    // ---- the tower of your workspaces: a floor each, a lit room for every app, the lift where you are
    if city {
        let tower = list(st, &["tower"]);
        let one = [serde_json::json!({ "id": 1, "active": true, "apps": [] })];
        let floors: &[Value] = if tower.is_empty() { &one } else { tower };
        let nf = floors.len().min(12);
        let fh = (h * 0.036).min(h * 0.22 / nf as f32);
        let (tw, x, y0) = (h * 0.085, b0 + (b1 - b0) * 0.035, hz - 1.0);
        let top = y0 - fh * nf as f32;
        g.rect(x, top, tw, y0 - top, mix(bg, fg, 0.08), 1.0);
        g.frame(x, top, tw, y0 - top, u, ac, 0.55);
        g.rect(x + tw * 0.45, top - h * 0.02, (u * 1.5).max(1.0), h * 0.02, ac, 0.8); // the antenna
        if tf.rem_euclid(1.7) < 0.22 { g.rect(x + tw * 0.45 - u, top - h * 0.022, 3.0 * u, 3.0 * u, pal.crit, 0.95) }
        let mut act = 0.0f32;
        for (i, fl) in floors.iter().take(nf).enumerate() {
            let fy = y0 - fh * (i as f32 + 1.0);
            let active = get(fl, &["active"]).as_bool().unwrap_or(false);
            if active {
                act = fy;
                g.rect(x + 1.0, fy + 1.0, tw - 2.0, fh - 2.0, ac, 0.16);
            }
            lab(labels, x + tw * 0.10, fy + fh * 0.22, fh * 0.42, if active { ac } else { fg }, if active { 1.0 } else { 0.45 }, text(fl, &["id"]), true);
            for (k, app) in list(fl, &["apps"]).iter().take(4).enumerate() {
                let name = app.as_str().unwrap_or("");
                let hsh = name.bytes().fold(0i32, |a, b| a.wrapping_mul(31).wrapping_add(b as i32));
                let c = if hide || name.is_empty() { ac } else { [ac, a2, pal.ok, pal.warn, mix(ac, fg, 0.4)][(hsh.unsigned_abs() % 5) as usize] };
                g.rect(x + tw * (0.24 + k as f32 * 0.17), fy + fh * 0.16, tw * 0.15, fh * 0.68, c, if active { 0.9 } else { 0.55 });
            }
            g.rect(x, fy, tw, u.max(1.0), fg, 0.12);
        }
        if act > 0.0 {
            mem.lift = if mem.lift == 0.0 || (mem.lift - act).abs() > h || still { act } else { mem.lift + (act - mem.lift) * 0.12 };
            g.rect(x + tw - tw * 0.1, mem.lift + fh * 0.18, tw * 0.07, fh * 0.64, fg, 0.85); // the lift goes to the workspace you are on
        }
        // the ivy of your uptime: it climbs a little every hour the PC stays on, all the way up after a week
        if get(st, &["ui", "ivy"]).as_bool().unwrap_or(true) {
            let days = num(st, &["sys", "uptime"]).unwrap_or(0.0) as f32 / 86400.0;
            let k = (days / 7.0).min(1.0);
            let hgt = (y0 - top) * k;
            if hgt >= h * 0.004 {
                let sway = (tf * 0.8).sin() * u;
                let vine = |s: f32| (x + (s * 9.0).sin() * tw * 0.12 + tw * 0.06 + sway * s, y0 - hgt * s);
                let pts: Vec<(f32, f32)> = (0..=20).map(|i| vine(i as f32 / 20.0)).collect();
                g.line(&pts, (1.4 * u).max(1.0), mix(pal.ok, bg, 0.35), 0.9);
                for i in 0..(6.0 + 60.0 * k).round() as usize {
                    let q = (i as f32 * 0.618).rem_euclid(1.0);
                    let p = vine(q);
                    g.ell(p.0 + if i % 2 == 1 { 3.0 } else { -3.0 } * u, p.1, 2.6 * u, 1.5 * u, if i % 2 == 1 { 0.6 } else { -0.6 }, mix(pal.ok, fg, (i % 5) as f32 * 0.06), 0.85);
                }
                // a week without a reboot: it flowers
                let open = if days < 7.0 { 0.0 } else if still { 1.0 } else { (0.25 + (days - 7.0) * 4.0 * 0.75).min(1.0) };
                if open > 0.0 {
                    for i in 0..12 {
                        let qf = 0.08 + (i as f32 * 0.618 + 0.2).rem_euclid(1.0) * 0.9;
                        let p = vine(qf);
                        let (fx, fy, r) = (p.0 + if i % 2 == 1 { 5.0 } else { -5.0 } * u, p.1, (7.0 + (i % 3) as f32 * 1.5) * u);
                        let br = 1.0 + 0.06 * (tf * 1.3 + i as f32).sin();
                        let (pr, d, rot) = (r * 0.48 * (0.45 + 0.55 * open) * br, r * 0.52 * open * br, i as f32 * 1.7 + (tf * 0.5 + i as f32).sin() * 0.15);
                        for p5 in 0..5 {
                            let a = rot + p5 as f32 * TAU / 5.0;
                            g.ell(fx + a.cos() * d, fy + a.sin() * d, pr, pr * 0.72, a, mix(a2, fg, 0.12), 0.95);
                        }
                        g.disc(fx, fy, r * 0.26, pal.warn, 1.0);
                    }
                }
            }
        }
    }
    // ---- your friends' towers: taller with their week, a floor flashes when one of them pushes
    let friends = list(st, &["friends"]);
    if city && !friends.is_empty() {
        let x0 = b1 - (b1 - b0) * 0.035;
        for (i, fr) in friends.iter().take(8).enumerate() {
            let week = num(fr, &["week"]).unwrap_or(0.0) as f32;
            let tw = h * 0.026;
            let (x, hh) = (x0 - i as f32 * tw * 1.6 - tw, h * (0.04 + 0.09 * ((1.0 + week).ln() / 80f32.ln()).min(1.0)));
            let top = hz - hh;
            let fl = num(fr, &["flashAge"]).map(|a| (1.0 - a as f32 / 4.0).max(0.0)).unwrap_or(0.0);
            let floors = ((hh / (h * 0.008)).round() as usize).max(2);
            g.rect(x, top, tw, hh, mix(bg, a2, 0.16), 0.85);
            g.frame(x, top, tw, hh, u, a2, 0.45);
            for k in 0..floors {
                let lit = ((k * 7 + i * 13) % 10) as f32 / 10.0 < (week / 40.0).min(1.0);
                let hot = fl > 0.0 && k == floors - 1;
                if !lit && !hot { continue }
                let fy = hz - (k as f32 + 1.0) * hh / floors as f32 + hh / floors as f32 * 0.3;
                g.rect(x + tw * 0.15, fy, tw * 0.7, (hh / floors as f32 * 0.4).max(1.0), if hot { fg } else { a2 }, if hot { 0.6 + 0.4 * fl } else { 0.55 });
            }
            if fl > 0.0 && !still { g.glow(x + tw / 2.0, top, h * 0.05, a2, 0.5 * fl) }
            let ci = text(fr, &["ci"]);
            if !ci.is_empty() {
                g.rect(x + tw / 2.0 - 1.5 * u, top - 4.0 * u, 3.0 * u, 3.0 * u, if ci == "fail" { pal.crit } else { pal.ok }, if still || (tf * 0.8 + i as f32).rem_euclid(1.6) < 1.2 { 1.0 } else { 0.3 });
            }
            if !hide { lab(labels, x + tw / 2.0, top - 20.0 * u, h * 0.0095, a2, 0.75, text(fr, &["name"]), true) }
        }
    }
    // ---- the parcels of your updates: one each (up to 14), the kernel in red, the AUR ones in the second accent
    if city {
        let (n, aur) = ((num(st, &["updates", "n"]).unwrap_or(0.0) + num(st, &["updates", "aur"]).unwrap_or(0.0)) as usize, num(st, &["updates", "aur"]).unwrap_or(0.0) as usize);
        let news = list(st, &["updates", "news"]);
        let kernel = get(st, &["updates", "kernel"]).as_bool().unwrap_or(false);
        if n > 0 || !news.is_empty() {
            let s = h * 0.016 * (1.0 + (n as f32 / 60.0).min(1.0) * 0.6);
            let (x0, y0, shown) = (b1 - (b1 - b0) * 0.13, hz + h * 0.03, n.min(14));
            for i in 0..shown {
                let row = if i < 5 { 0 } else if i < 9 { 1 } else if i < 12 { 2 } else { 3 };
                let (in_row, idx) = ([5.0, 4.0, 3.0, 2.0][row], (i - [0, 5, 9, 12][row]) as f32);
                let (bx, by) = (x0 - (in_row * s * 1.08) / 2.0 + idx * s * 1.08, y0 - (row as f32 + 1.0) * s * 0.92);
                let c = if i == 0 && kernel { pal.crit } else if i >= shown - aur.min(shown) { a2 } else { mix(pal.warn, bg, 0.25) };
                g.rect(bx, by, s, s * 0.88, mix(bg, c, 0.55), 1.0);
                g.frame(bx, by, s, s * 0.88, u, c, 0.95);
                g.rect(bx + s * 0.45, by, s * 0.1, s * 0.88, c, 0.9); // the tape
            }
            if n > 0 {
                lab(labels, x0, y0 + h * 0.018, (h * 0.012).round(), if kernel { pal.crit } else { fg }, 0.9, format!("{n} {}{}", if n == 1 { "update" } else { "updates" }, if kernel { " · new kernel" } else { "" }), true);
            }
            if !news.is_empty() {
                // Arch news that asks for manual intervention: a road sign with a blinking light
                let (sx, sy) = (x0 + s * 4.2, y0 - s * 3.6);
                g.rect(sx - u, sy, 2.0 * u, y0 - sy, fg, 0.5);
                g.poly(&[(sx, sy - s * 1.1), (sx + s * 0.95, sy + s * 0.5), (sx - s * 0.95, sy + s * 0.5)], pal.warn, 0.95);
                lab(labels, sx, sy - s * 0.45, s * 0.9, bg, 1.0, "!".into(), true);
                if still || tf.rem_euclid(1.2) < 0.7 { g.glow(sx, sy - s * 1.3, s * 0.9, pal.crit, 0.9) }
                if !private(st) {
                    lab(labels, sx, y0 + h * 0.036, (h * 0.011).round(), pal.warn, 0.95, "news: read before you update".into(), true);
                }
            }
        }
    }
    // ---- the firewall: a wall along the horizon, a gate for the VPN, what it blocks breaking on it
    let fw = get(st, &["net", "fw"]);
    if fw.is_object() && (!list(fw, &["events"]).is_empty() || !get(fw, &["vpn"]).is_null()) {
        let wh = h * 0.012;
        g.rect(0.0, hz - wh, w, wh, mix(bg, fg, 0.22), 0.9);
        let mut x = 0.0;
        while x < w {
            g.rect(x, hz - wh - h * 0.006, h * 0.01, h * 0.006, mix(bg, fg, 0.3), 0.9);
            x += h * 0.02;
        }
        if let Some(vpn) = num(fw, &["vpn"]).filter(|v| *v > 0.0) {
            let gw = h * 0.03;
            g.rect(w * 0.5 - gw / 2.0, hz - wh - h * 0.02, gw, wh + h * 0.02, pal.ok, 0.25 + 0.15 * (tf * 2.0).sin());
            lab(labels, w * 0.5, hz - wh - h * 0.04, h * 0.011, pal.ok, 0.9, format!("vpn {vpn:.0}"), false);
        }
        let now = num(st, &["t"]).unwrap_or(0.0);
        for e in list(fw, &["events"]) {
            let ips = text(e, &["src"]) == "ips";
            let (age, dur) = ((now - num(e, &["t"]).unwrap_or(0.0)) as f32, if ips { 2.5 } else { 1.6 });
            if !(0.0..=dur + 0.6).contains(&age) { continue }
            let az = num(e, &["az"]).unwrap_or(180.0) as f32;
            let k = (age / dur).min(1.0);
            let tx = w / 2.0 + ((az - 180.0) / 90.0).clamp(-1.05, 1.05) * w * 0.44;
            let (sx, sy) = (tx + if az < 180.0 { -1.0 } else { 1.0 } * w * 0.05, hz - h * if ips { 0.6 } else { 0.3 });
            let (px, py) = (sx + (tx - sx) * k, sy + (hz - wh - sy) * k);
            if k < 1.0 {
                let c = if ips { pal.crit } else if text(e, &["level"]) == "warn" { pal.warn } else { fg };
                g.line(&[(px - (tx - sx) * 0.08, py - (hz - sy) * 0.08), (px, py)], (h / if ips { 400.0 } else { 900.0 }).max(1.0), c, if ips { 0.9 } else { 0.6 });
                if ips { g.glow(px, py, h * 0.02, pal.warn, 0.9) }
            } else {
                let sp = (age - dur) / 0.6;
                for j in 0..4 {
                    g.rect(tx + (j as f32 * 1.6).cos() * sp * h * 0.015, hz - wh - (j as f32 * 1.6).sin().abs() * sp * h * 0.015, 2.0, 2.0, fg, 0.6 * (1.0 - sp));
                }
            }
        }
    }
    // ---- Christmas: the tree on the floor to the left of the centre, a garland across the sky of the city
    if get(st, &["season", "xmas"]).is_object() || get(st, &["season", "xmas"]).as_bool().unwrap_or(false) {
        let (tx, ty, th) = (w * 0.3, hz + h * 0.1, h * 0.17);
        let green = mix(bg, pal.ok, 0.45);
        for k in 0..3 {
            let kf = k as f32;
            let (tw, top, bot) = (th * (0.5 - kf * 0.12), ty - th * (0.42 + kf * 0.27), ty - th * (kf * 0.27));
            g.poly(&[(tx, top), (tx + tw, bot), (tx - tw, bot)], mix(green, [0.0, 0.0, 0.0], kf * 0.08), 1.0);
        }
        g.rect(tx - th * 0.04, ty, th * 0.08, th * 0.08, mix(bg, c8(90, 50, 30), 0.7), 1.0);
        let bc = [pal.crit, pal.warn, ac, a2];
        for k in 0..14 {
            let by = ty - th * (0.08 + (k as f32 * 0.61).rem_euclid(0.85));
            let half = th * 0.5 * (1.0 - (ty - by) / (th * 1.05)) * 0.8;
            let bx = tx + ((k as f32 * 0.37).rem_euclid(1.0) * 2.0 - 1.0) * half;
            g.disc(bx, by, (h / 380.0).max(1.5), bc[k % 4], if still { 1.0 } else { 0.4 + 0.6 * (tf * 1.5 + k as f32).sin().abs() });
        }
        g.glow(tx, ty - th * 0.97, h * 0.03, pal.warn, 0.9);
        let cols = [pal.crit, pal.ok, pal.warn, ac, a2];
        let y0 = hz - h * 0.19;
        for sp in 0..4 {
            let (x0, x1, n) = (w * sp as f32 / 4.0, w * (sp + 1) as f32 / 4.0, 14);
            let at = |i: usize| { let q = i as f32 / n as f32; (x0 + (x1 - x0) * q, y0 + h * 0.035 * (1.0 - (2.0 * q - 1.0).powi(2))) };
            let pts: Vec<(f32, f32)> = (0..=n).map(at).collect();
            g.line(&pts, (h / 1000.0).max(1.0), mix(bg, fg, 0.25), 0.5);
            for i in 1..n {
                let (x, y) = at(i);
                let on = if still { 1.0 } else { 0.35 + 0.65 * if ((tf * 1.2) as usize + i + sp) % 2 == 1 { 1.0 } else { 0.4 } };
                let c = cols[(i + sp) % cols.len()];
                g.glow(x, y + h * 0.006, h * 0.016, c, 0.55 * on);
                g.disc(x, y + h * 0.006, (h / 330.0).max(1.5), c, 0.95 * on);
            }
        }
    }
    // ---- a storm over what is failing: a cloud for each of your panels in warn or crit, rain and bolts under the critical ones
    if get(st, &["ui", "infraWeather"]).as_bool().unwrap_or(true) {
        mem.storms.clear();
        let bad: Vec<&Value> = list(st, &["widgets"]).iter().filter(|wd| matches!(text(wd, &["level"]).as_str(), "crit" | "warn") && text(wd, &["id"]) != "weather").take(4).collect();
        for (i, wd) in bad.iter().enumerate() {
            let id = text(wd, &["id"]);
            let hsh = id.bytes().fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
            let crit = text(wd, &["level"]) == "crit";
            let (x, y) = (w * (0.3 + 0.4 * ((hsh % 1000) as f32 / 1000.0)), hz - h * (0.2 + 0.035 * (i % 2) as f32));
            let (cc, a, s2) = (if crit { mix(bg, [0.0, 0.0, 0.0], 0.55) } else { mix(bg, fg, 0.18) }, if crit { 0.92 } else { 0.75 }, h * if crit { 0.06 } else { 0.045 });
            for c in [[-1.1, 0.25, 0.75], [0.0, 0.0, 1.0], [1.1, 0.2, 0.8], [0.5, 0.45, 0.7], [-0.5, 0.45, 0.7]] {
                g.ell(x + c[0] * s2, y + c[1] * s2, s2 * c[2] * 1.2, s2 * c[2] * 0.7, 0.0, cc, a);
            }
            if crit {
                for k in 0..14 {
                    let (rx, ry) = (x + (k as f32 / 13.0 - 0.5) * s2 * 2.4, y + s2 * 0.6 + (tf * 0.9 + k as f32 * 0.37).rem_euclid(1.0) * h * 0.12);
                    g.line(&[(rx, ry), (rx - h * 0.004, ry + h * 0.014)], 1.0, fg, 0.25);
                }
                let e = mem.infra.entry(id.clone()).or_insert((t + 2.0 + 4.0 * 0.5, -1e9, Vec::new()));
                if t > e.0 && !still {
                    let mut pts = vec![(x + (mem.r.next() - 0.5) * s2, y + s2 * 0.5)];
                    for _ in 0..7 {
                        let lp = pts[pts.len() - 1];
                        pts.push((lp.0 + (mem.r.next() - 0.5) * h * 0.04, lp.1 + (hz - y) / 7.0));
                    }
                    *e = (t + 3.0 + mem.r.next() as f64 * 6.0, t, pts);
                }
                if t - e.1 < 0.22 {
                    let pts = e.2.clone();
                    g.line(&pts, (h / 500.0).max(1.5), fg, 0.95);
                    g.rect(0.0, 0.0, w, hz, fg, 0.05);
                }
            }
            let title = if text(wd, &["title"]).is_empty() { id.clone() } else { text(wd, &["title"]) };
            mem.storms.push((id.clone(), x, y, s2 * 1.6, if hide { String::new() } else { title.clone() }));
            if !hide {
                lab(labels, x, y + s2 * 0.85, h * 0.012, if crit { pal.crit } else { pal.warn }, 0.9, title.chars().take(24).collect(), false);
            }
        }
    }
}

/// Over everything in the scene: the bolts, the flakes of a cold Christmas, the wind you can see, pollen, an
/// earthquake's waves, a tsunami, the new year, the doorbell, what you type in `omarchy-pulse term`.
pub fn over(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, labels: &mut Vec<Label>) {
    let mut g = G { win, ox: f.ox, oy: f.oy };
    let (w, h, hz, u) = (f.w, f.h, f.hz, f.u);
    let still = get(st, &["ui", "motion"]).as_str() == Some("reduced");
    let tf = if still { 0.0 } else { t as f32 };
    let (fg, ac) = (pal.fg, pal.accent);
    // the bolts over you: a bright core in a wider light, twice in a blink
    mem.bolts.retain(|b| t - b.t <= 0.6);
    for b in &mem.bolts {
        let Some(pts) = &b.pts else { continue };
        let age = (t - b.t) as f32;
        let a = if age < 0.08 || (0.16..0.22).contains(&age) { 1.0 } else { 0.25 };
        for (lw, k) in [(12.0, 0.12), (5.0, 0.3), (2.4, 1.0)] {
            g.line(pts, lw * u, mix(fg, ac, 0.3), k * a);
        }
    }
    if mem.flash > 0.02 {
        g.rect(0.0, 0.0, w, h, fg, 0.10 * mem.flash * if still { 0.3 } else { 1.0 });
        mem.flash *= 0.80;
    }
    // syslog and SNMP traps: a bolt where that device is, right now
    let now = num(st, &["t"]).unwrap_or(0.0);
    for b in list(st, &["bolts"]) {
        let bt = num(b, &["t"]).unwrap_or(0.0);
        let age = (now - bt) as f32;
        if !(0.0..=1.2).contains(&age) { continue }
        let x0 = w * (0.3 + 0.4 * ((bt * 7.0).rem_euclid(1.0) as f32));
        let pts: Vec<(f32, f32)> = (0..=8).map(|k| (x0 + if k > 0 && k < 8 { ((bt * 13.0) as f32 + k as f32 * 7.0).sin() * h * 0.02 } else { 0.0 }, hz * k as f32 / 8.0)).collect();
        g.line(&pts, (h / 400.0).max(2.0), if num(b, &["sev"]).unwrap_or(9.0) <= 2.0 { pal.crit } else { fg }, 1.0 - age / 1.2);
    }
    // it is really cold at Christmas: a few flakes (when it does not already snow or rain)
    let code = num(st, &["weather", "code"]).unwrap_or(0.0) as i32;
    let wet = (51..=67).contains(&code) || (71..=77).contains(&code) || (80..=86).contains(&code) || code >= 95;
    if get(st, &["season", "xmas", "snow"]).as_bool().unwrap_or(false) && !wet {
        if mem.flakes.is_empty() {
            for _ in 0..70 { let v = [mem.r.next(), mem.r.next(), 0.4 + mem.r.next()]; mem.flakes.push(v) }
        }
        for (i, fl) in mem.flakes.iter_mut().enumerate() {
            if !still {
                fl[1] += 0.0009 * fl[2];
                fl[0] += (tf + i as f32).sin() * 0.25 / w;
                if fl[1] > 1.0 { fl[1] = 0.0 }
            }
            g.rect(fl[0] * w, fl[1] * h, 2.0, 2.0, fg, 0.55);
        }
    }
    // strong wind leaves gust lines; a breeze is not drawn
    let unit = if text(st, &["weather", "windUnit"]) == "mph" { 1.609 } else { 1.0 };
    let kmh = unit * (num(st, &["weather", "wind"]).unwrap_or(0.0) as f32).max(num(st, &["weather", "gust"]).unwrap_or(0.0) as f32 * 0.8);
    let wind_k = (kmh / 60.0).min(1.0);
    let wind_x = (num(st, &["weather", "windDir"]).unwrap_or(270.0) as f32).to_radians().sin() * wind_k;
    if wind_k >= 0.4 && wind_x.abs() >= 0.08 && !still {
        let n = (wind_k * 14.0).round() as usize;
        while mem.gusts.len() < n { let v = [mem.r.next() * w, h * (0.25 + mem.r.next() * 0.5), 0.5 + mem.r.next(), mem.r.next() * 6.0]; mem.gusts.push(v) }
        for i in 0..n {
            let ny = h * (0.25 + mem.r.next() * 0.5);
            let gs = &mut mem.gusts[i];
            let len = w * 0.06 * gs[2] * wind_k;
            gs[0] += wind_x * h * 0.02 * gs[2];
            if gs[0] < -len * 2.0 { gs[0] = w + len; gs[1] = ny }
            if gs[0] > w + len * 2.0 { gs[0] = -len; gs[1] = ny }
            let y = gs[1] + (tf * 2.0 + gs[3]).sin() * h * 0.006;
            let sg = wind_x.signum();
            g.line(&[(gs[0], y), (gs[0] - sg * len * 0.5, y - h * 0.004), (gs[0] - sg * len, y)], (h / 900.0).max(1.0), fg, 0.12 * wind_k);
        }
    }
    // pollen floating in the light when there is a lot of it
    let pn = match text(st, &["air", "pollenLevel"]).as_str() { "low" => 10, "moderate" => 25, "high" => 50, "very high" => 80, _ => 0 };
    while mem.pollen.len() < pn { let v = [mem.r.next() * w, hz * (0.3 + mem.r.next() * 0.7), mem.r.next() * 6.0, 0.5 + mem.r.next()]; mem.pollen.push(v) }
    for p in mem.pollen.iter_mut().take(pn) {
        if !still {
            p[0] += (if wind_x == 0.0 { 0.1 } else { wind_x }) * h * 0.0015 * p[3] + (tf + p[2]).sin() * 0.3;
            p[1] += (tf * 0.7 + p[2]).sin() * 0.25;
            if p[0] > w { p[0] = 0.0 }
            if p[0] < 0.0 { p[0] = w }
        }
        g.disc(p[0], p[1], u * 1.4 * p[3], c8(255, 228, 140), 0.55);
    }
    // an earthquake: waves come along the ground from where it happened
    for q in &mem.quakes {
        let age = (t - q.0) as f32;
        let x0 = w / 2.0 + ((q.2 - 180.0) / 90.0).clamp(-1.05, 1.05) * w * 0.44;
        for k in 0..3 {
            let r = (age - k as f32 * 0.35) * w * 0.25;
            if r <= 0.0 { continue }
            g.arc(x0, hz, r, r * 0.16, 0.0, TAU, (h / 400.0).max(1.0), if q.3 { pal.crit } else { pal.warn }, (0.6 - age / 5.0).max(0.0));
        }
    }
    // a tsunami message: the sea at your feet swells, a long wave crosses from where the earthquake was
    mem.tsunamis.retain(|q| t - q.0 <= 14.0);
    for q in &mem.tsunamis {
        let age = (t - q.0) as f32;
        let env = (age / 2.0).min(1.0) * ((14.0 - age) / 3.0).min(1.0);
        let from = if (q.1 - 180.0) / 90.0 < 0.0 { -1.0 } else { 1.0 };
        let crest = if from < 0.0 { -w * 0.2 + age / 10.0 * w * 1.4 } else { w * 1.2 - age / 10.0 * w * 1.4 };
        let base = h - (h - hz) * (0.28 + 0.14 * env * if q.2 { 1.5 } else { 1.0 });
        let mut pts = vec![(0.0, h)];
        let mut x = 0.0;
        while x <= w {
            let d = (x - crest) / (w * 0.12);
            pts.push((x, base - (-d * d).exp() * (h - hz) * 0.5 * env - (x * 0.012 + age * 3.0).sin() * h * 0.004 * env));
            x += 8.0;
        }
        pts.push((w, h));
        g.poly(&pts, mix(if q.2 { pal.crit } else { ac }, pal.bg, 0.5), 0.6 * env);
        g.line(&pts[1..pts.len() - 1], (h / 600.0).max(1.0), fg, 0.55 * env);
        if (age * 2.0) as i32 % 2 == 0 {
            lab(labels, w / 2.0, base - (h - hz) * 0.07, h / 34.0, if q.2 { pal.crit } else { pal.warn }, 0.9 * env, "TSUNAMI".into(), true);
        }
    }
    // the new year: the last hour quietly, the last minute big, then fireworks for a quarter of an hour
    let ny = get(st, &["season", "newYear"]);
    if ny.is_object() {
        let year = text(ny, &["year"]);
        let left = num(ny, &["left"]).unwrap_or(1e9) as f32;
        let (cx, cy) = (w / 2.0, hz - h * 0.16);
        if get(ny, &["party"]).as_bool().unwrap_or(false) {
            if get(st, &["ui", "fireworks"]).as_bool().unwrap_or(true) && t - mem.ny_fx > if still { 4.0 } else { 0.7 } {
                mem.ny_fx = t;
                mem.asks.push("burst");
            }
            lab(labels, cx, cy - h * 0.045, h * 0.07, fg, 0.9, format!("HAPPY {year}"), true);
        } else if left <= 60.0 {
            let fr = left - left.floor();
            lab(labels, cx, cy - h * 0.1, h * 0.16 * if still { 1.0 } else { 1.0 + 0.25 * fr }, fg, 0.5 + 0.5 * fr, format!("{}", left.ceil() as i32), true);
        } else if left <= 3600.0 {
            lab(labels, cx, hz + h * 0.02, h * 0.024, fg, 0.75, format!("{year} in {}:{:02}", (left / 60.0) as i32, (left % 60.0) as i32), false);
        }
    }
    // the doorbell: rings open from the middle of the horizon
    let ring = get(st, &["ring"]).as_bool().unwrap_or(false);
    if ring && !mem.ringing { mem.ring = t }
    mem.ringing = ring;
    let ra = (t - mem.ring) as f32;
    if (0.0..4.0).contains(&ra) {
        for k in 0..3 {
            let r = (ra - k as f32 * 0.4).rem_euclid(1.3) * h * 0.2;
            g.arc(w / 2.0, hz - h * 0.14, r, r, 0.0, TAU, (u * 2.5).max(1.5), pal.warn, (1.0 - r / (h * 0.26)).max(0.0) * (1.0 - ra / 4.0));
        }
    }
    // what you type in omarchy-pulse term
    let term = list(st, &["term"]);
    if !term.is_empty() && !private(st) {
        let lines: Vec<&Value> = term.iter().rev().take(8).rev().collect();
        let (lh, bw) = (h * 0.016, w * 0.34);
        let bh = lines.len() as f32 * lh + h * 0.016;
        let (x, y) = (w / 2.0 - bw / 2.0, h * 0.94 - bh);
        g.rect(x, y, bw, bh, c8(0, 10, 4), 0.78);
        g.frame(x, y, bw, bh, 1.0, c8(60, 255, 140), 0.5);
        for (i, l) in lines.iter().enumerate() {
            lab(labels, w / 2.0, y + h * 0.006 + i as f32 * lh, h * 0.012, c8(90, 255, 150), 0.95, text(l, &["text"]).chars().take(70).collect(), false);
        }
    }
    let _ = PI;
}

/// Pulse Net, the rest of your network as part of the city (read-only): a building for each switch with a window
/// for each port and roads between neighbours, a tower for each host with a floor for each VM, an antenna for each
/// access point with its people, a truck for each backup job, robots with red eyes for intruders.
pub fn net(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, labels: &mut Vec<Label>) {
    let pnet = get(st, &["net"]);
    if !pnet.is_object() {
        return;
    }
    let mut g = G { win, ox: f.ox, oy: f.oy };
    let (w, h, hz) = (f.w, f.h, f.hz);
    let floor = h - hz;
    let still = get(st, &["ui", "motion"]).as_str() == Some("reduced");
    let tf = t as f32;
    let hide = private(st);
    let now = num(st, &["t"]).unwrap_or(0.0);
    let (bg, fg, ac, a2) = (pal.bg, pal.fg, pal.accent, pal.accent2);
    let mut pos: HashMap<String, [f32; 4]> = HashMap::new();
    let row_y = hz + floor * 0.2;
    let sws = list(pnet, &["switches"]);
    if !sws.is_empty() {
        let span = w * 0.25;
        let bwid = (w * 0.085).min(span / sws.len() as f32 * 0.82);
        for (i, s) in sws.iter().enumerate() {
            let ports = list(s, &["ports"]);
            let (x, cols) = (w * 0.26 + (i as f32 + 0.5) * span / sws.len() as f32, 12usize);
            let (rows, cw) = (ports.len().div_ceil(cols).max(1), bwid / cols as f32);
            let bh = rows as f32 * cw * 2.0 + h * 0.016;
            let top = row_y - bh;
            let up = get(s, &["up"]).as_bool().unwrap_or(true);
            g.rect(x - bwid / 2.0, top, bwid, bh, mix(bg, ac, 0.14), 0.95);
            g.frame(x - bwid / 2.0, top, bwid, bh, (h / 800.0).max(1.0), if up { ac } else { pal.crit }, if up { 0.6 } else if (tf * 2.0).fract() < 0.5 { 1.0 } else { 0.3 });
            for (k, p) in ports.iter().enumerate() {
                let (px0, py0) = (x - bwid / 2.0 + (k % cols) as f32 * cw + cw * 0.2, top + h * 0.008 + (k / cols) as f32 * cw * 2.0);
                let (c, a) = if get(p, &["err"]).as_bool().unwrap_or(false) { (pal.crit, if still || (tf * 3.0).fract() < 0.6 { 1.0 } else { 0.3 }) } else if get(p, &["up"]).as_bool().unwrap_or(false) { (if get(p, &["jam"]).as_bool().unwrap_or(false) { pal.warn } else { ac }, 0.35 + 0.65 * num(p, &["util"]).unwrap_or(0.0) as f32) } else { (bg, 0.9) };
                g.rect(px0, py0, cw * 0.6, cw * 1.3, c, a);
            }
            let name = text(s, &["name"]);
            if !name.is_empty() && !hide { lab(labels, x, top - h * 0.02, h * 0.011, fg, 0.7, name.clone(), false) }
            pos.insert(if name.is_empty() { format!("#{i}") } else { name }, [x, top, bwid, row_y]);
        }
        for (i, s) in sws.iter().enumerate() {
            // the roads between neighbours (LLDP/CDP); a busy uplink is a traffic jam
            let name = text(s, &["name"]);
            let Some(a) = pos.get(&if name.is_empty() { format!("#{i}") } else { name }).copied() else { continue };
            for p in list(s, &["ports"]) {
                let Some(b) = pos.get(&text(p, &["peer"])).copied() else { continue };
                if b[0] == a[0] { continue }
                let (y, jam) = (row_y + h * 0.012, get(p, &["jam"]).as_bool().unwrap_or(false));
                g.line(&[(a[0], y), (b[0], y)], (h / 600.0).max(1.0), if jam { pal.warn } else { ac }, 0.35);
                let (n, sp) = (if jam { 10 } else { 3 }, if jam { 0.05 } else { 0.4 + num(p, &["util"]).unwrap_or(0.0) as f32 });
                for k in 0..n {
                    let q = if still { k as f32 / n as f32 } else { (tf * sp + k as f32 / n as f32).fract() };
                    g.rect(a[0] + (b[0] - a[0]) * q, y - 1.5, 3.0, 3.0, if jam { pal.warn } else { fg }, 0.85);
                }
            }
        }
    }
    // the datacenter: a tower for each host, a floor for each VM; a migration is a lift; maintenance has scaffolding
    let hosts = list(pnet, &["dc", "hosts"]);
    if !hosts.is_empty() {
        let dspan = w * 0.2;
        let (tw, fh) = ((w * 0.03).min(dspan / hosts.len() as f32 * 0.6), h * 0.007);
        for (i, ho) in hosts.iter().enumerate() {
            let vms = list(ho, &["vms"]);
            let (x, n) = (w * 0.54 + (i as f32 + 0.5) * dspan / hosts.len() as f32, vms.len().min(24));
            let top = row_y - n as f32 * fh - h * 0.01;
            g.rect(x - tw / 2.0, top, tw, row_y - top, mix(bg, a2, 0.15), 0.95);
            for (k, v) in vms.iter().take(n).enumerate() {
                let up = get(v, &["up"]).as_bool().unwrap_or(false);
                g.rect(x - tw * 0.4, row_y - (k as f32 + 1.0) * fh - h * 0.004, tw * 0.8, fh * 0.7, if up { a2 } else { mix(bg, fg, 0.15) }, if up { 0.75 } else { 0.6 });
            }
            g.frame(x - tw / 2.0, top, tw, row_y - top, (h / 900.0).max(1.0), if get(ho, &["up"]).as_bool().unwrap_or(true) { a2 } else { pal.crit }, 0.7);
            if get(ho, &["maint"]).as_bool().unwrap_or(false) {
                for k2 in 0..4 {
                    let yy = top + (row_y - top) * k2 as f32 / 4.0;
                    g.line(&[(x - tw * 0.7, yy), (x + tw * 0.7, yy + (row_y - top) / 4.0)], 1.0, pal.warn, 0.6);
                }
            }
            let name = text(ho, &["name"]);
            if !name.is_empty() && !hide { lab(labels, x, top - h * 0.018, h * 0.01, fg, 0.65, name.clone(), false) }
            pos.insert(if name.is_empty() { format!("h{i}") } else { name }, [x, top, tw, row_y]);
        }
        for m in list(pnet, &["dc", "moves"]) {
            let (Some(a), Some(b)) = (pos.get(&text(m, &["from"])), pos.get(&text(m, &["to"]))) else { continue };
            let age = (now - num(m, &["t"]).unwrap_or(0.0)) as f32;
            if age > 6.0 { continue }
            let q = (age / 4.0).clamp(0.0, 1.0);
            g.rect(a[0] + (b[0] - a[0]) * q - tw * 0.4, a[1].min(b[1]) - h * 0.03 * (q * PI).sin(), tw * 0.8, fh, pal.warn, 0.95);
        }
    }
    // Wi-Fi: an antenna for each access point, a person for each client, nearer with a stronger signal
    let aps = list(pnet, &["wifi", "aps"]);
    if !aps.is_empty() {
        let wy = hz + floor * 0.52;
        let mut ap_x: HashMap<String, f32> = HashMap::new();
        for (i, a) in aps.iter().enumerate() {
            let x = w * 0.32 + (i as f32 + 0.5) * w * 0.36 / aps.len() as f32;
            let name = a.as_str().unwrap_or("").to_string();
            g.line(&[(x, wy), (x, wy - h * 0.035)], (h / 700.0).max(1.5), ac, 0.8);
            for r in 0..3 {
                let rr = (if still { r as f32 / 3.0 } else { (tf * 0.5 + r as f32 / 3.0).fract() }) * h * 0.05;
                g.arc(x, wy - h * 0.035, rr, rr, PI * 1.15, PI * 1.85, 1.0, ac, 0.4 * (1.0 - rr / (h * 0.05)));
            }
            if !name.is_empty() && !hide { lab(labels, x, wy + h * 0.006, h * 0.01, fg, 0.6, name.clone(), false) }
            ap_x.insert(name, x);
        }
        for c in list(pnet, &["wifi", "clients"]) {
            let Some(ax) = ap_x.get(&text(c, &["ap"])) else { continue };
            let id = text(c, &["id"]);
            let hsh = u32::from_str_radix(id.get(0..6).unwrap_or("0"), 16).unwrap_or(0);
            let (ang, sig) = (((hsh % 360) as f32).to_radians(), num(c, &["signal"]).unwrap_or(-60.0) as f32);
            let d = w * (0.012 + 0.05 * ((-sig - 35.0) / 55.0).clamp(0.0, 1.0));
            let (tx, ty) = (ax + ang.cos() * d, wy - h * 0.005 + ang.sin() * d * 0.25);
            let p = mem.wifi.entry(id).or_insert((tx, ty));
            let k = if still { 1.0 } else { 0.03 }; // roaming: it walks to its new antenna
            p.0 += (tx - p.0) * k;
            p.1 += (ty - p.1) * k;
            g.rect(p.0 - 1.0, p.1 - h * 0.012, 2.0, h * 0.01, fg, 0.75);
            g.disc(p.0, p.1 - h * 0.015, h * 0.003, fg, 0.75);
        }
    }
    // backups: each job is a truck. Running: it drives from the city to the vault. Done: it drives back home and parks
    // with a green light. Failed or late: it stops halfway, hazard lights and smoke, until the next good run.
    let bks: Vec<&Value> = list(pnet, &["backups"]).iter().take(4).collect();
    if !bks.is_empty() {
        let by = hz + floor * 0.6;
        let (b0, b1) = band(f);
        let (x0, x1) = (b0 + (b1 - b0) * 0.12, b1 - (b1 - b0) * 0.12);
        g.rect(x0 - h * 0.05, by + h * 0.004, x1 - x0 + h * 0.1, h * 0.01 + bks.len() as f32 * h * 0.022, mix(bg, fg, 0.12), 0.9); // the road
        for (k, (dx0, name)) in [(x0, "BACKUP"), (x1, "VAULT")].into_iter().enumerate() {
            let (dw, dh) = (h * 0.05, h * 0.035);
            let dx = dx0 + if k == 1 { h * 0.012 } else { -h * 0.062 };
            let c = if k == 1 { a2 } else { ac };
            g.rect(dx, by - dh, dw, dh, mix(bg, fg, 0.2), 1.0);
            g.frame(dx, by - dh, dw, dh, 1.0, c, 0.8);
            if !hide { lab(labels, dx + dw / 2.0, by - dh - h * 0.016, h * 0.009, c, 0.9, name.into(), true) }
        }
        for (i, b) in bks.iter().enumerate() {
            let (lane, l2) = (by + h * 0.016 + i as f32 * h * 0.022, h * 0.05);
            let (state, name) = (text(b, &["state"]), text(b, &["name"]));
            let ago = num(b, &["t"]).map(|bt| (now - bt) as f32).unwrap_or(1e9);
            let (mut x, mut dir) = (x0, 1.0f32);
            let bad = state == "failed" || state == "late";
            if state == "running" {
                let tk = mem.truck.entry(name.clone()).or_insert(0.0);
                if !still { *tk += 0.0012 }
                if *tk > 1.0 { *tk = 0.0 }
                x = x0 + (x1 - x0) * if still { 0.5 } else { *tk };
            } else if state == "ok" {
                let back = (ago / 900.0).min(1.0);
                x = x1 + (x0 - x1) * if still { 1.0 } else { back };
                dir = if back < 1.0 { -1.0 } else { 1.0 };
                mem.truck.remove(&name);
            } else if bad {
                x = x0 + (x1 - x0) * 0.5;
            }
            let c = if bad { pal.crit } else if state == "ok" { pal.ok } else { ac };
            let at = |k: f32| x + dir * l2 * k;
            let box_x = at(-0.5).min(at(0.18));
            g.rect(box_x, lane - l2 * 0.42, l2 * 0.68, l2 * 0.36, mix(bg, fg, 0.55), 1.0); // the box
            g.rect(at(-0.46).min(at(0.14)), lane - l2 * 0.3, l2 * 0.6, l2 * 0.08, c, 0.95); // its stripe: the state
            g.rect(at(0.2).min(at(0.46)), lane - l2 * 0.3, l2 * 0.26, l2 * 0.24, mix(bg, fg, 0.4), 1.0); // the cab
            g.rect(at(0.32).min(at(0.43)), lane - l2 * 0.27, l2 * 0.11, l2 * 0.1, ac, 0.5); // its window
            for k in [-0.35, 0.05, 0.34] { g.disc(at(k), lane - l2 * 0.05, l2 * 0.07, bg, 1.0) }
            if state == "running" || dir < 0.0 { g.rect(at(0.45).min(at(0.48)), lane - l2 * 0.12, l2 * 0.03, l2 * 0.04, pal.warn, 0.9) } // headlights
            if bad {
                let on = still || (tf * 2.0).fract() < 0.5;
                g.rect(x - l2 * 0.52, lane - l2 * 0.16, l2 * 0.04, l2 * 0.04, pal.warn, if on { 1.0 } else { 0.15 });
                g.rect(x + l2 * 0.44, lane - l2 * 0.16, l2 * 0.04, l2 * 0.04, pal.warn, if on { 1.0 } else { 0.15 });
                if !still {
                    for sm in 0..5 {
                        let q = (tf * 0.25 + sm as f32 / 5.0).fract();
                        g.disc(x + l2 * 0.3 + (q * 6.0 + sm as f32).sin() * l2 * 0.12 + q * l2 * 0.09, lane - l2 * 0.35 - q * h * 0.05, l2 * (0.08 + q * 0.18), mix(fg, bg, 0.4), 0.28 * (1.0 - q));
                    }
                }
            }
            if !hide && !name.is_empty() {
                labels.push(Label { x: x + l2 * 0.55 + h * 0.06, y: lane - l2 * 0.34, size: h * 0.01, c, a: 0.85, text: format!("{name} · {state}"), bold: false });
            }
        }
    }
    // intruders: robots with red eyes walking to the server they hit, bigger when more serious
    for (i, a) in list(pnet, &["intruders"]).iter().enumerate() {
        let tgt = pos.get(&text(a, &["target"])).copied().unwrap_or([w * 0.5, 0.0, 0.0, hz + floor * 0.2]);
        let q = ((now - num(a, &["t"]).unwrap_or(now)).max(0.0) as f32 / 40.0).min(1.0);
        let sx = if i % 2 == 1 { w * 0.24 } else { w * 0.76 };
        let (x, y, s2) = (sx + (tgt[0] - sx) * if still { 1.0 } else { q }, tgt[3] + h * 0.02, h * if text(a, &["level"]) == "crit" { 0.03 } else { 0.02 });
        let body = mix(bg, fg, 0.4);
        g.rect(x - s2 * 0.3, y - s2, s2 * 0.6, s2 * 0.7, body, 0.95);
        g.rect(x - s2 * 0.22, y - s2 * 1.35, s2 * 0.44, s2 * 0.33, body, 0.95);
        g.rect(x - s2 * 0.15, y - s2 * 1.25, s2 * 0.1, s2 * 0.08, pal.crit, 1.0);
        g.rect(x + s2 * 0.05, y - s2 * 1.25, s2 * 0.1, s2 * 0.08, pal.crit, 1.0);
        let stp = if still { 0.0 } else { (tf * 8.0 + i as f32).sin() * s2 * 0.1 };
        g.rect(x - s2 * 0.2 + stp, y - s2 * 0.3, s2 * 0.12, s2 * 0.3, body, 0.95);
        g.rect(x + s2 * 0.08 - stp, y - s2 * 0.3, s2 * 0.12, s2 * 0.3, body, 0.95);
    }
}
