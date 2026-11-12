//! The living city of the skyline scene, as Pulse 1.8 has it: your repos as towers (a crane while the CI runs),
//! your datacenter as a district of halls, the car, the robots (your services), the calendar's trains, the candles
//! of what expires, the tips' balloon, the banner plane, the aurora when the Sun storms.

use crate::hub::{get, list, num, text};
use crate::palette::{mix, scale, Pal, C};
use crate::scene::{disc, glow, rect, PlaneMark};
use gpui::{point, px, PathBuilder, Window};
use serde_json::Value;

#[derive(Clone, Copy)]
pub struct Frame {
    pub w: f32,
    pub h: f32,
    pub ox: f32,
    pub oy: f32,
    pub hz: f32,
    pub u: f32,
    pub t: f32,
    /// the weights of the sky: night, twilight, day
    pub sky: [f32; 3],
}

fn level(l: &str, pal: &Pal) -> C {
    match l {
        "ok" => pal.ok,
        "warn" | "restart" => pal.warn,
        "crit" | "down" | "fail" => pal.crit,
        _ => pal.dim,
    }
}

fn label(marks: &mut Vec<PlaneMark>, f: &Frame, x: f32, y: f32, c: C, title: String) {
    marks.push(PlaneMark { c, x: x.clamp(f.w * 0.245, f.w * 0.70), y, title, line: String::new() });
}


/// Your datacenter, as a district in front of the city: a hall for every row (a VM, a container, a host), its racks
/// in the colour of its state; one that is down goes dark and red.
pub fn datacenter(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, marks: &mut Vec<PlaneMark>, private: bool) {
    let groups = list(st, &["datacenter", "groups"]);
    if groups.is_empty() {
        return;
    }
    let y0 = f.oy + f.hz + f.h * 0.018;
    let (hw, hh, gap) = (f.w * 0.017, f.h * 0.052, f.w * 0.0035);
    let total: usize = groups.iter().map(|g| list(g, &["rows"]).len().min(10)).sum();
    let width = total as f32 * (hw + gap) + (groups.len() as f32 - 1.0) * hw * 0.8;
    let mut x = f.ox + f.w / 2.0 - width / 2.0;
    for g in groups {
        let rows = list(g, &["rows"]);
        let gx = x;
        for r in rows.iter().take(10) {
            let lv = text(r, &["level"]);
            let c = level(&lv, pal);
            let down = lv == "crit";
            let body = mix(pal.bg, pal.dim, 0.35);
            rect(win, x, y0, hw, hh, body, 0.96);
            rect(win, x, y0, hw, (f.u * 2.0).max(1.0), mix(c, pal.fg, 0.2), 0.95);
            rect(win, x + hw * 0.78, y0, hw * 0.22, hh, scale(body, 0.7), 0.96);
            for iy in 0..7 {
                for ix in 0..3 {
                    let on = !down && (((f.t * (1.0 + ix as f32 * 0.3) + iy as f32 * 0.7 + x * 0.01).sin()) > -0.5);
                    let cc = if down { pal.crit } else { c };
                    rect(win, x + hw * (0.16 + ix as f32 * 0.22), y0 + hh * (0.14 + iy as f32 * 0.115), hw * 0.12, hh * 0.05, cc, if on { 0.9 } else if down && iy == 3 { 0.9 * (0.5 + 0.5 * (f.t * 6.0).sin()) } else { 0.18 });
                }
            }
            x += hw + gap;
        }
        if get(st, &["datacenter", "labels"]).as_bool().unwrap_or(true) && !private {
            label(marks, f, gx - f.ox, y0 - f.oy + hh + f.h * 0.004, pal.dim, format!("{} · {}", text(g, &["title"]), rows.len()));
        }
        x += hw * 0.8;
    }
}


/// The robots: your services, walking on the floor. Green and busy, amber when one restarts, red and still when down.
pub fn robots(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, marks: &mut Vec<PlaneMark>, private: bool) {
    let robots = list(st, &["robots"]);
    let n = robots.len().min(8);
    for (i, r) in robots.iter().take(n).enumerate() {
        let health = text(r, &["health"]);
        let c = level(&health, pal);
        let stopped = health == "down" || health == "crit";
        let lane = f.oy + f.hz + f.h * (0.19 + 0.02 * (i % 2) as f32);
        let walk = if stopped { 0.0 } else { (f.t * 0.25 + i as f32 * 1.7).sin() * f.w * 0.018 };
        let x = f.ox + f.w * (0.27 + 0.35 * (i as f32 + 0.5) / n as f32) + walk;
        let s = f.h * 0.022;
        let step = if stopped { 0.0 } else { (f.t * 6.0 + i as f32).sin() };
        rect(win, x - s * 0.28, lane - s * 0.2 + step * s * 0.1, s * 0.14, s * 0.85, pal.dim, 1.0);
        rect(win, x + s * 0.14, lane - s * 0.2 - step * s * 0.1, s * 0.14, s * 0.85, pal.dim, 1.0);
        rect(win, x - s * 0.42, lane - s * 1.15, s * 0.84, s * 1.0, mix(pal.bg, c, 0.45), 1.0);
        rect(win, x - s * 0.42, lane - s * 1.15, s * 0.84, (f.u * 1.5).max(1.0), c, 0.95);
        rect(win, x - s * 0.3, lane - s * 1.75, s * 0.6, s * 0.5, mix(pal.bg, c, 0.6), 1.0);
        disc(win, x - s * 0.12, lane - s * 1.5, s * 0.07, pal.fg, 1.0);
        disc(win, x + s * 0.12, lane - s * 1.5, s * 0.07, pal.fg, 1.0);
        rect(win, x - f.u * 0.7, lane - s * 2.05, (f.u * 1.4).max(1.0), s * 0.3, pal.dim, 1.0);
        glow(win, x, lane - s * 2.05, s * 0.5, c, if health == "restart" { 0.5 + 0.5 * (f.t * 7.0).sin() } else { 0.8 });
        if !private {
            label(marks, f, x - f.ox - s * 1.2, lane - f.oy + s * 0.75, c, text(r, &["name"]).chars().take(14).collect());
        }
    }
}

/// What is about to expire (certificates, domains): a candle each, shorter as the day comes nearer.
pub fn candles(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, marks: &mut Vec<PlaneMark>, private: bool) {
    for (i, e) in list(st, &["expiry"]).iter().take(4).enumerate() {
        let days = e.get("days").and_then(Value::as_f64).unwrap_or(0.0) as f32;
        let c = level(&text(e, &["level"]), pal);
        let x = f.ox + f.w * (0.645 + 0.026 * i as f32);
        let base = f.oy + f.hz + f.h * (0.20 + 0.03 * (i % 2) as f32);
        let hh = f.h * (0.012 + 0.06 * (days / 90.0).clamp(0.0, 1.0));
        rect(win, x - f.u * 4.0, base - hh, f.u * 8.0, hh, mix(pal.fg, c, 0.35), 0.95);
        let flick = 0.75 + 0.25 * (f.t * 9.0 + i as f32 * 2.0).sin();
        glow(win, x, base - hh - f.u * 6.0, f.h * 0.016, pal.warn, flick);
        disc(win, x, base - hh - f.u * 5.0, f.u * 3.0, [1.0, 0.9, 0.6], 1.0);
        if !private {
            label(marks, f, x - f.ox - f.w * 0.012, base - f.oy + f.h * 0.006, c, format!("{days:.0}d"));
        }
    }
}

/// Your calendar: the next events are trains coming in along the horizon; the nearer the hour, the nearer the station.
pub fn trains(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, marks: &mut Vec<PlaneMark>, private: bool) {
    let now = num(st, &["t"]).unwrap_or(0.0);
    for (i, e) in list(st, &["calendar"]).iter().take(2).enumerate() {
        let start = e.get("start").and_then(Value::as_f64).unwrap_or(0.0);
        let mins = ((start - now) / 60.0) as f32;
        if !(-30.0..=180.0).contains(&mins) {
            continue;
        }
        let side = if i == 0 { -1.0 } else { 1.0 };
        let far = (mins / 180.0).clamp(0.0, 1.0); // 1 far away, 0 in the station
        let x = f.ox + f.w / 2.0 + side * f.w * (0.16 + 0.22 * far);
        let y = f.oy + f.hz - f.h * 0.008;
        let c = if mins < 5.0 { pal.warn } else { pal.accent2 };
        for car in 0..5 {
            let cx0 = x + side * car as f32 * f.w * 0.0125;
            rect(win, cx0 - f.w * 0.0055, y - f.h * 0.012, f.w * 0.011, f.h * 0.012, mix(pal.bg, c, 0.45), 1.0);
            rect(win, cx0 - f.w * 0.004, y - f.h * 0.009, f.w * 0.008, f.h * 0.004, c, 0.9);
        }
        glow(win, x - side * f.w * 0.008, y - f.h * 0.006, f.h * 0.012, [1.0, 1.0, 0.9], 0.8);
        if !private {
            let when = if mins <= 0.0 { "now".to_string() } else if mins < 60.0 { format!("in {mins:.0} min") } else { format!("in {:.0}h {:.0}m", (mins / 60.0).floor(), mins % 60.0) };
            label(marks, f, x - f.ox - f.w * 0.02, y - f.oy - f.h * 0.045, c, format!("{} · {when}", text(e, &["title"]).chars().take(18).collect::<String>()));
        }
    }
}

/// The tips' hot-air balloon: every so often it drifts across with one of Omarchy's shortcuts on its basket.
pub fn balloon(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, marks: &mut Vec<PlaneMark>) {
    let tips = list(st, &["tips"]);
    let every = num(st, &["ui", "tipsEvery"]).unwrap_or(40.0).max(20.0) as f32;
    if tips.is_empty() {
        return;
    }
    let cycle = every + 26.0;
    let k = (f.t % cycle) / 26.0;
    if k > 1.0 {
        return;
    }
    let tip = tips[((f.t / cycle) as usize) % tips.len()].as_str().unwrap_or("").to_string();
    let x = f.ox + f.w * (0.26 + 0.44 * k);
    let y = f.oy + f.hz * (0.72 - 0.10 * (k * 3.14).sin()) + (f.t * 0.8).sin() * f.h * 0.004;
    let r = f.h * 0.038;
    for (i, c) in [pal.accent2, pal.accent, pal.accent2, pal.accent, pal.accent2].iter().enumerate() {
        let wv = r * 2.0 * (1.0 - (i as f32 - 2.0).abs() * 0.3);
        rect(win, x - r + (i as f32) * r * 0.4, y - wv / 2.0, r * 0.4, wv, mix(*c, pal.bg, 0.25), 0.95);
    }
    rect(win, x - r * 0.28, y + r * 1.55, r * 0.56, r * 0.36, mix(pal.warn, pal.bg, 0.4), 1.0);
    let mut ropes = PathBuilder::stroke(px((f.u * 1.0).max(1.0)));
    ropes.move_to(point(px(x - r * 0.7), px(y + r * 0.8)));
    ropes.line_to(point(px(x - r * 0.28), px(y + r * 1.55)));
    ropes.move_to(point(px(x + r * 0.7), px(y + r * 0.8)));
    ropes.line_to(point(px(x + r * 0.28), px(y + r * 1.55)));
    if let Ok(p) = ropes.build() {
        win.paint_path(p, crate::palette::col(pal.fg, 0.6));
    }
    label(marks, f, x - f.ox + r * 1.2, y - f.oy - r * 0.4, pal.accent, tip);
}

/// The banner plane: now and then a small plane tows your words across the sky.
pub fn banner(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, marks: &mut Vec<PlaneMark>) {
    let words = text(st, &["ui", "banner"]);
    let every = num(st, &["ui", "bannerEvery"]).unwrap_or(45.0).max(20.0) as f32 * 4.0;
    if words.is_empty() {
        return;
    }
    let k = (f.t % (every + 30.0)) / 30.0;
    if k > 1.0 {
        return;
    }
    let x = f.ox + f.w * (1.05 - 1.3 * k);
    let y = f.oy + f.hz * 0.50 + (f.t * 0.6).sin() * f.h * 0.006;
    let s = f.h * 0.012;
    rect(win, x - s * 1.4, y - s * 0.2, s * 2.8, s * 0.4, pal.fg, 0.95);
    rect(win, x - s * 0.2, y - s * 1.0, s * 0.5, s * 2.0, pal.fg, 0.85);
    rect(win, x + s * 1.2, y - s * 0.6, s * 0.25, s * 0.6, pal.fg, 0.9);
    let mut rope = PathBuilder::stroke(px((f.u * 1.0).max(1.0)));
    rope.move_to(point(px(x + s * 1.4), px(y)));
    rope.line_to(point(px(x + s * 4.0), px(y)));
    if let Ok(p) = rope.build() {
        win.paint_path(p, crate::palette::col(pal.fg, 0.6));
    }
    let bx = x - f.ox + s * 4.0;
    if bx > f.w * 0.245 && bx < f.w * 0.70 {
        marks.push(PlaneMark { c: pal.accent2, x: bx, y: y - f.oy - s * 1.1, title: words.chars().take(48).collect(), line: String::new() });
    }
}

/// Everything of the living city, in the order it stands: far to near.
pub fn paint(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, marks: &mut Vec<PlaneMark>) {
    let private = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
    trains(win, st, pal, f, marks, private);
    datacenter(win, st, pal, f, marks, private);
    robots(win, st, pal, f, marks, private);
    candles(win, st, pal, f, marks, private);
    let up = num(st, &["updates", "n"]).unwrap_or(0.0);
    if up > 0.0 {
        let kernel = if get(st, &["updates", "kernel"]).as_bool().unwrap_or(false) { " · new kernel" } else { "" };
        label(marks, f, f.w * 0.645, f.hz + f.h * 0.035, pal.warn, format!("{up:.0} updates{kernel}"));
    }
}
