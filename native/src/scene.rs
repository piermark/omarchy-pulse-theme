//! The world behind the panels: the sky at its real hour, the stars, the Moon and the Sun where they are, your
//! clouds moving with your wind, a far city whose windows come on as it gets dark, rain as heavy as it really is,
//! and the planes over you. Everything is drawn on the GPU as quads and paths; nothing here asks the network.

use crate::hub::{get, list, num};
use crate::palette::{col, mix, scale, Pal, C};
use gpui::{linear_color_stop, linear_gradient, point, px, size, Bounds, PaintQuad, PathBuilder, Pixels, Window};
use serde_json::Value;

/// The horizon, as a share of the height: the same line as the web page (pulse.js: H * 0.745).
pub const HORIZON: f32 = 0.745;

/// A small repeatable random: the same city and the same stars at every start.
pub struct Rand(u64);
impl Rand {
    pub fn new(seed: u64) -> Self {
        Rand(seed.max(1))
    }
    pub fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX >> 1) as f32
    }
}

pub struct Building {
    pub x: f32, // 0..1 of the width
    pub w: f32,
    pub h: f32, // of the height
    pub layer: usize,
    pub spire: f32,
    pub seed: u64,
}

pub struct Cloud {
    pub x: f32,
    pub y: f32,
    pub s: f32,
    pub puffs: Vec<(f32, f32, f32)>,
}

/// What is made once and kept: the city, the stars, the clouds, the rain.
pub struct World {
    pub stars: Vec<(f32, f32, f32, f32)>,
    pub city: Vec<Building>,
    pub clouds: Vec<Cloud>,
    pub drops: Vec<(f32, f32, f32, f32)>,
    pub last_t: f64,
}

impl World {
    pub fn new() -> Self {
        let mut r = Rand::new(1808);
        let stars = (0..260).map(|_| (r.next(), r.next().powf(1.3), 0.25 + r.next() * 0.75, r.next() * 6.28)).collect();
        let mut city = Vec::new();
        for layer in 0..3usize {
            let mut x = -0.02;
            while x < 1.02 {
                let w = (0.012 + r.next() * 0.022) * (1.0 + layer as f32 * 0.25);
                let tall = if r.next() > 0.86 { 1.9 } else { 1.0 };
                let h = (0.02 + r.next() * 0.05) * (0.7 + layer as f32 * 0.35) * tall;
                city.push(Building { x, w, h, layer, spire: if r.next() > 0.8 { 0.2 + r.next() * 0.5 } else { 0.0 }, seed: (r.next() * 1e9) as u64 });
                x += w + r.next() * 0.004;
            }
        }
        let clouds = (0..12)
            .map(|_| {
                let n = 4 + (r.next() * 4.0) as usize;
                Cloud {
                    x: r.next() * 1.3 - 0.15,
                    y: 0.06 + r.next() * 0.36,
                    s: 0.6 + r.next() * 0.9,
                    puffs: (0..n).map(|i| ((i as f32 - n as f32 / 2.0) * 0.55 + r.next() * 0.3, r.next() * 0.35 - 0.17, 0.6 + r.next() * 0.6)).collect(),
                }
            })
            .collect();
        let drops = (0..900).map(|_| (r.next(), r.next(), 0.5 + r.next() * 0.5, r.next())).collect();
        World { stars, city, clouds, drops, last_t: 0.0 }
    }
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let u = ((x - a) / (b - a)).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// [night, twilight, day] from the Sun's real elevation: the same curve as pulse.js (skyWeights).
pub fn sky_weights(st: &Value) -> [f32; 3] {
    match num(st, &["night", "sun", "el"]) {
        Some(el) => {
            let el = el as f32;
            let d = smooth(1.0, 12.0, el);
            let n = smooth(1.0, 9.0, -el);
            [n, (1.0 - d - n).max(0.0), d]
        }
        None => {
            let day = num(st, &["weather", "day"]).unwrap_or(0.0) > 0.0;
            if day { [0.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0] }
        }
    }
}

/// Where a point of the sky is on the screen. The wallpaper looks south: east on the left, west on the right;
/// what is behind you (north) shows only when it is almost overhead. As pulse.js (skyXY).
pub fn sky_xy(az: f32, el: f32, w: f32, hz: f32) -> Option<(f32, f32)> {
    if el < -1.0 {
        return None;
    }
    let front = (75.0..=285.0).contains(&az);
    if !front && el < 62.0 {
        return None;
    }
    let x = if front { w / 2.0 + (az - 180.0) / 90.0 * w * 0.44 } else { w / 2.0 - az.to_radians().sin() * w * 0.44 * (90.0 - el) / 28.0 };
    Some((x, hz * 0.93 - (el.max(0.0) / 90.0).sqrt() * hz * 0.85))
}

fn rect(win: &mut Window, x: f32, y: f32, w: f32, h: f32, c: C, a: f32) {
    if a <= 0.004 || w <= 0.0 || h <= 0.0 {
        return;
    }
    win.paint_quad(PaintQuad::new(Bounds::new(point(px(x), px(y)), size(px(w), px(h)))).background(col(c, a)));
}

fn disc(win: &mut Window, cx: f32, cy: f32, r: f32, c: C, a: f32) {
    if a <= 0.004 || r <= 0.0 {
        return;
    }
    win.paint_quad(PaintQuad::new(Bounds::new(point(px(cx - r), px(cy - r)), size(px(r * 2.0), px(r * 2.0)))).background(col(c, a)).corner_radii(px(r)));
}

/// A soft light: rings of the same colour, fainter as they widen.
fn glow(win: &mut Window, cx: f32, cy: f32, r: f32, c: C, a: f32) {
    for i in 0..7 {
        let k = 1.0 - i as f32 / 7.0;
        disc(win, cx, cy, r * (1.0 - k * 0.86), c, a * 0.2 * (1.2 - k));
    }
}

fn gradient(win: &mut Window, x: f32, y: f32, w: f32, h: f32, top: C, ta: f32, bottom: C, ba: f32) {
    let bg = linear_gradient(180.0, linear_color_stop(col(top, ta), 0.0), linear_color_stop(col(bottom, ba), 1.0));
    win.paint_quad(PaintQuad::new(Bounds::new(point(px(x), px(y)), size(px(w), px(h)))).background(bg));
}

/// A plane on the screen: where, and what to write beside it (main.rs writes the text).
pub struct PlaneMark {
    pub x: f32,
    pub y: f32,
    pub c: C,
    pub title: String,
    pub line: String,
}

/// Paints the world for the time `t` (seconds) and returns the planes' places for their tags.
pub fn paint(bounds: Bounds<Pixels>, world: &mut World, st: &Value, pal: &Pal, t: f64, win: &mut Window) -> Vec<PlaneMark> {
    let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let (ox, oy) = (f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let hz = h * HORIZON;
    let u = h / 1080.0;
    let dt = if world.last_t > 0.0 { ((t - world.last_t) as f32).clamp(0.0, 0.25) } else { 0.0 };
    world.last_t = t;
    let tf = t as f32;
    let [wn, wt, wd] = sky_weights(st);
    let cloud = (num(st, &["weather", "cloud"]).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
    let rain = num(st, &["weather", "rain"]).unwrap_or(0.0) as f32;
    let code = num(st, &["weather", "code"]).unwrap_or(0.0) as i32;
    let wind = num(st, &["weather", "wind"]).unwrap_or(0.0) as f32;
    let wind_dir = num(st, &["weather", "windDir"]).unwrap_or(270.0) as f32;

    let mut marks = Vec::new();
    // ---- the sky: pulse.js's own three skies (night, twilight, day: PN, PT, PD), mixed by the Sun's real height
    let (bg, a2) = (pal.bg, pal.accent2);
    let black = [0.0, 0.0, 0.0];
    let pn = [mix(bg, black, 0.35), mix(bg, a2, 0.10), mix(bg, a2, 0.28)];
    let pt = [mix(bg, [0.078, 0.039, 0.157], 0.4), mix(bg, a2, 0.22), mix(mix(bg, pal.warn, 0.45), a2, 0.35)];
    let pd = [mix(bg, pal.accent, 0.16), mix(bg, pal.accent, 0.24), mix(mix(bg, pal.fg, 0.22), a2, 0.2)];
    let grey = mix(pal.dim, pal.bg, 0.55);
    let mx = |k: usize| -> C { [pn[k][0] * wn + pt[k][0] * wt + pd[k][0] * wd, pn[k][1] * wn + pt[k][1] * wt + pd[k][1] * wd, pn[k][2] * wn + pt[k][2] * wt + pd[k][2] * wd] };
    let over = cloud * 0.35; // an overcast sky is greyer
    let (top, midc, low) = (mix(mx(0), grey, over * 0.6), mix(mx(1), grey, over), mix(mx(2), grey, over * 0.8));
    gradient(win, ox, oy, w, hz * 0.7 + 1.0, top, 1.0, midc, 1.0);
    gradient(win, ox, oy + hz * 0.7, w, hz * 0.3 + 1.0, midc, 1.0, low, 1.0);

    // ---- stars, fewer behind the clouds
    let starry = wn * (1.0 - cloud * 0.85);
    if starry > 0.03 {
        for s in &world.stars {
            let tw = 0.72 + 0.28 * (tf * (0.5 + s.2) + s.3).sin();
            let z = (u * (0.8 + s.2 * 1.3)).max(1.0);
            rect(win, ox + s.0 * w, oy + s.1 * hz * 0.96, z, z, pal.fg, starry * s.2 * tw * (1.0 - s.1 * 0.6));
        }
    }

    // ---- the Moon and the Sun, where they really are
    if let (Some(az), Some(el)) = (num(st, &["night", "moon", "az"]), num(st, &["night", "moon", "el"])) {
        if let Some((x, y)) = sky_xy(az as f32, el as f32, w, hz) {
            let lit = num(st, &["night", "moon", "lit"]).unwrap_or(1.0) as f32;
            let waxing = get(st, &["night", "moon", "waxing"]).as_bool().unwrap_or(true);
            let r = h * 0.022;
            let a = (0.35 + 0.65 * wn) * (1.0 - cloud * 0.6);
            glow(win, ox + x, oy + y, r * 4.5, pal.fg, 0.5 * a * lit.max(0.15));
            disc(win, ox + x, oy + y, r, mix(pal.fg, [1.0, 1.0, 1.0], 0.4), a);
            // the unlit part: a disc of the sky's colour, slid to the side the Moon is dark on
            let shade = (1.0 - lit).clamp(0.0, 1.0);
            if shade > 0.03 {
                let off = r * 2.0 * (1.0 - shade) * if waxing { -1.0 } else { 1.0 };
                disc(win, ox + x + off, oy + y, r * 1.02, top, a.min(1.0));
            }
        }
    }
    if let (Some(az), Some(el)) = (num(st, &["night", "sun", "az"]), num(st, &["night", "sun", "el"])) {
        if el > -4.0 {
            if let Some((x, y)) = sky_xy(az as f32, el as f32, w, hz) {
                let low_sun = 1.0 - smooth(0.0, 25.0, el as f32);
                let c = mix([1.0, 0.97, 0.88], pal.warn, low_sun * 0.8);
                let a = (1.0 - cloud * 0.7) * (0.35 + 0.65 * (wd + wt));
                glow(win, ox + x, oy + y, h * 0.16, c, 0.55 * a);
                disc(win, ox + x, oy + y, h * 0.018, c, a);
            }
        }
    }

    // ---- your clouds, moving with your wind: slowly when there is almost none, but never standing still
    let toward = (wind_dir + 180.0).to_radians(); // the wind comes from windDir; east is on the left
    let mut vx = -toward.sin() * (wind / 60.0);
    let floor = 0.012;
    if vx.abs() < floor {
        vx = if vx < 0.0 { -floor } else { floor };
    }
    let shown = ((cloud * 12.0).round() as usize).min(world.clouds.len());
    let dark_c = mix(mix(pal.bg, pal.dim, 0.5), top, 0.35);
    let light_c = mix(dark_c, pal.fg, 0.10 + 0.25 * wd);
    for c in world.clouds.iter_mut().take(shown) {
        c.x += vx * dt * 0.05 * (1.2 - c.y);
        if c.x > 1.25 {
            c.x = -0.25;
        } else if c.x < -0.25 {
            c.x = 1.25;
        }
        let r = h * 0.05 * c.s;
        let (cx0, cy0) = (ox + c.x * w, oy + c.y * hz * 0.8);
        for p in &c.puffs {
            // a puff is a few discs, each wider and fainter: a soft edge without a blur
            for k in 0..4 {
                let g = 1.0 + k as f32 * 0.22;
                disc(win, cx0 + p.0 * r * 1.5, cy0 + p.1 * r + r * 0.12, r * p.2 * g, dark_c, 0.16 * (0.4 + 0.6 * cloud));
            }
            disc(win, cx0 + p.0 * r * 1.5, cy0 + p.1 * r - r * 0.18, r * p.2 * 0.7, light_c, 0.10);
        }
    }

    // ---- the far city: three rows, paler with the distance; windows come on as the Sun goes down
    let lit = 0.07 + 0.50 * wn + 0.28 * wt;
    let warm = mix(pal.warn, pal.fg, 0.35);
    for b in &world.city {
        let depth = b.layer as f32 / 2.0; // 0 far, 1 near
        let body = mix(mix(pal.bg, low, 0.70 - depth * 0.30), pal.fg, 0.02 + depth * 0.02);
        let (bx, bw, bh) = (ox + b.x * w, b.w * w, b.h * h);
        let by = oy + hz - bh;
        rect(win, bx, by, bw, bh + 1.0, body, 1.0);
        rect(win, bx, by, bw, (u * 1.2).max(1.0), mix(body, pal.fg, 0.22 * (wd + wt * 0.5) + 0.05), 1.0); // the roof's edge
        rect(win, bx + bw * 0.72, by, bw * 0.28, bh, scale(body, 0.82), 1.0); // the side in the shade
        if b.spire > 0.0 {
            rect(win, bx + bw * 0.48, by - bh * b.spire, (u * 1.4).max(1.0), bh * b.spire, body, 1.0);
        }
        if b.layer > 0 {
            let mut r = Rand::new(b.seed);
            let (cw, ch) = ((u * (2.2 + depth)).max(1.5), (u * (2.6 + depth)).max(1.5));
            let (cols, rows) = (((bw - cw) / (cw * 2.1)) as i32, ((bh - ch * 2.0) / (ch * 2.3)) as i32);
            for iy in 0..rows.min(40) {
                for ix in 0..cols.min(12) {
                    let k = r.next();
                    if k < lit {
                        let c = if r.next() > 0.82 { pal.accent } else { warm };
                        rect(win, bx + cw + ix as f32 * cw * 2.1, by + ch * 1.5 + iy as f32 * ch * 2.3, cw, ch, c, (0.35 + 0.5 * depth) * (0.5 + 0.5 * wn + 0.3 * wt));
                    }
                }
            }
        }
    }

    // ---- the ground: pulse.js's floor, and the grid that runs to the horizon
    gradient(win, ox, oy + hz, w, h - hz, mix(bg, a2, 0.2), 1.0, mix(bg, black, 0.5), 1.0);
    gradient(win, ox, oy + hz, w, h * 0.06, mix(pal.accent, a2, 0.5), 0.10 + 0.12 * (wn + wt * 0.6), bg, 0.0);
    let mut grid = PathBuilder::stroke(px((u * 1.0).max(1.0)));
    for i in 1..=9 {
        let y = oy + hz + (h - hz) * (i as f32 / 9.0).powf(2.1);
        grid.move_to(point(px(ox), px(y)));
        grid.line_to(point(px(ox + w), px(y)));
    }
    for i in -12..=12 {
        let fx = i as f32 / 12.0;
        grid.move_to(point(px(ox + w / 2.0 + fx * w * 0.12), px(oy + hz)));
        grid.line_to(point(px(ox + w / 2.0 + fx * w * 1.25), px(oy + h)));
    }
    if let Ok(path) = grid.build() {
        win.paint_path(path, col(a2, 0.20));
    }
    // ---- the near city: neon towers either side of the dome, their windows lit as the CPU works
    let cpu = (num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
    let mut r = Rand::new(77);
    for i in 0..16 {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let k = (i / 2) as f32;
        let x = ox + w / 2.0 + side * (w * 0.092 + k * w * 0.026 + r.next() * w * 0.006);
        let bw = w * (0.011 + r.next() * 0.008);
        let bh = h * (0.08 + r.next() * 0.17) * (1.0 - k * 0.05);
        let body = mix(mix(bg, pal.accent, 0.20), black, 0.30);
        let by = oy + hz - bh;
        rect(win, x - bw / 2.0, by, bw, bh, body, 1.0);
        rect(win, x + bw * 0.2, by, bw * 0.3, bh, scale(body, 0.75), 1.0);
        rect(win, x - bw / 2.0, by, bw, (u * 1.6).max(1.0), pal.accent, 0.9);
        rect(win, x - bw / 2.0, by, (u * 1.2).max(1.0), bh, pal.accent, 0.35);
        let (cw, ch) = (bw * 0.16, (u * 5.0).max(2.0));
        let rows = ((bh - ch * 2.0) / (ch * 2.2)) as i32;
        for iy in 0..rows.min(40) {
            for ix in 0..3 {
                let on = r.next() < 0.30 + 0.55 * cpu;
                let c = if r.next() > 0.8 { a2 } else { pal.accent };
                if on {
                    rect(win, x - bw * 0.36 + ix as f32 * bw * 0.26, by + ch * 1.6 + iy as f32 * ch * 2.2, cw, ch, c, 0.55 + 0.35 * (wn + wt));
                }
            }
        }
        if i % 3 == 0 {
            rect(win, x, by - bh * 0.14, (u * 1.4).max(1.0), bh * 0.14, a2, 0.9);
            let blink = if ((tf * 0.8 + i as f32 * 0.37).fract()) < 0.5 { 1.0 } else { 0.25 };
            glow(win, x, by - bh * 0.14, h * 0.012, pal.ok, blink);
        }
    }
    // ---- the dome at the centre: the sphere of Pulse, as a wire of meridians and parallels
    let rr = h * 0.135;
    let (dcx, dcy) = (ox + w / 2.0, oy + hz);
    gradient(win, dcx - rr, dcy - rr, rr * 2.0, rr, a2, 0.0, a2, 0.10);
    let mut dome = PathBuilder::stroke(px((u * 1.4).max(1.0)));
    for m in 0..=8 {
        let kx = (m as f32 / 8.0 * std::f32::consts::PI + tf * 0.08).cos(); // the meridians turn slowly
        for i in 0..=40 {
            let a = i as f32 / 40.0 * std::f32::consts::PI;
            let p = point(px(dcx + a.cos() * rr * kx), px(dcy - a.sin() * rr));
            if i == 0 { dome.move_to(p) } else { dome.line_to(p) }
        }
    }
    for l in 1..5 {
        let a = l as f32 / 5.0 * std::f32::consts::FRAC_PI_2;
        dome.move_to(point(px(dcx - a.cos() * rr), px(dcy - a.sin() * rr)));
        dome.line_to(point(px(dcx + a.cos() * rr), px(dcy - a.sin() * rr)));
    }
    if let Ok(path) = dome.build() {
        win.paint_path(path, col(a2, 0.55));
    }
    let mut rim = PathBuilder::stroke(px((u * 2.2).max(1.5)));
    for i in 0..=60 {
        let a = i as f32 / 60.0 * std::f32::consts::PI;
        let p = point(px(dcx + a.cos() * rr), px(dcy - a.sin() * rr));
        if i == 0 { rim.move_to(p) } else { rim.line_to(p) }
    }
    if let Ok(path) = rim.build() {
        win.paint_path(path, col(pal.accent, 0.9));
    }
    // ---- your AI agents: a drone each over the dome, its light in the colour of what it is doing
    let agents = list(st, &["agents"]);
    let n_ag = agents.len().min(6);
    for (i, a) in agents.iter().take(n_ag).enumerate() {
        let state = a.get("state").and_then(Value::as_str).unwrap_or("");
        let c = match state { "working" => pal.accent, "waiting" => pal.warn, "done" => pal.ok, "failed" => pal.crit, _ => pal.dim };
        let fx = i as f32 - (n_ag as f32 - 1.0) / 2.0;
        let x = dcx + fx * w * 0.075;
        let y = dcy - rr - h * (0.05 + 0.045 * ((i % 2) as f32)) + (tf * 1.3 + i as f32 * 1.7).sin() * h * 0.006;
        let ds = h * 0.018;
        let spin = (tf * 18.0 + i as f32).sin().abs();
        for sx in [-1.0f32, 1.0] {
            rect(win, x + sx * ds * 1.5 - ds * (0.3 + 0.6 * spin), y - ds * 0.55, ds * (0.6 + 1.2 * spin), (u * 1.6).max(1.0), pal.fg, 0.7);
            rect(win, x + sx * ds * 1.5 - u, y - ds * 0.5, (u * 2.0).max(1.0), ds * 0.5, pal.dim, 1.0);
        }
        rect(win, x - ds * 1.5, y - ds * 0.05, ds * 3.0, (u * 2.0).max(1.0), pal.dim, 1.0);
        disc(win, x, y + ds * 0.1, ds * 0.62, mix(bg, c, 0.35), 1.0);
        disc(win, x, y + ds * 0.1, ds * 0.3, c, 1.0);
        let pulse = if state == "waiting" { 0.5 + 0.5 * (tf * 6.0).sin() } else { 0.7 };
        glow(win, x, y + ds * 0.1, ds * 2.2, c, pulse);
        let name = a.get("name").and_then(Value::as_str).unwrap_or("agent").to_string();
        let what = a.get("text").and_then(Value::as_str).unwrap_or("").chars().take(28).collect::<String>();
        marks.push(PlaneMark { c, x: (x - w * 0.05).clamp(w * 0.245, w * 0.62), y: y + ds * 1.2, title: format!("● {name}"), line: if what.is_empty() { state.to_string() } else { format!("{state} · {what}") } });
    }

    // the line of the horizon: the machine's pulse, livelier with the CPU
    let _ = cpu;
    let cpu = (num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
    let mut wave = PathBuilder::stroke(px((u * 2.0).max(1.5)));
    for i in 0..=160 {
        let fx = i as f32 / 160.0;
        let near = (-((fx - 0.5) * 3.2).powi(2)).exp();
        let y = oy + hz + (fx * 38.0 - tf * (1.0 + cpu * 3.0)).sin() * h * 0.006 * (0.3 + cpu) * (1.0 - near * 0.8);
        if i == 0 {
            wave.move_to(point(px(ox), px(y)));
        } else {
            wave.line_to(point(px(ox + fx * w), px(y)));
        }
    }
    if let Ok(path) = wave.build() {
        win.paint_path(path, col(pal.accent, 0.85));
    }

    // ---- rain as heavy as it really is; far drops short and faint, near ones long and bright
    let wet = (51..=67).contains(&code) || (80..=82).contains(&code) || code >= 95 || rain > 0.0;
    let snow = (71..=77).contains(&code) || code == 85 || code == 86;
    if wet && !snow {
        let n = ((80.0 + rain * 220.0) as usize).min(world.drops.len());
        let slant = (0.28 + vx * 4.0).clamp(-0.6, 0.6); // pulse.js leans its rain; the wind leans it more
        let mut far = PathBuilder::stroke(px((u * 1.0).max(1.0)));
        let mut near_p = PathBuilder::stroke(px((u * 1.8).max(1.2)));
        for d in world.drops.iter().take(n) {
            let near = d.2 > 0.78;
            let len = h * if near { 0.05 + 0.04 * d.3 } else { 0.014 + 0.016 * d.3 };
            let reach = if near { 1.05 } else { HORIZON + 0.02 };
            let y = ((d.1 + tf * if near { 1.5 } else { 0.7 + d.2 * 0.5 }) % 1.0) * h * reach;
            let x = ((d.0 * 1.3 - 0.15) * w + slant * (h * reach - y)).rem_euclid(w * 1.1);
            let b = if near { &mut near_p } else { &mut far };
            b.move_to(point(px(ox + x), px(oy + y)));
            b.line_to(point(px(ox + x - slant * len), px(oy + y + len)));
        }
        let drop_c = mix(pal.fg, pal.accent, 0.35);
        if let Ok(path) = far.build() {
            win.paint_path(path, col(drop_c, 0.22));
        }
        if let Ok(path) = near_p.build() {
            win.paint_path(path, col(drop_c, 0.38));
        }
    } else if snow {
        for d in world.drops.iter().take(260) {
            let y = ((d.1 + tf * 0.045 * (0.5 + d.2)) % 1.0) * h;
            let x = ((d.0 + (tf * 0.4 + d.3 * 6.28).sin() * 0.008).rem_euclid(1.0)) * w;
            disc(win, ox + x, oy + y, (u * (1.0 + d.2 * 1.6)).max(1.0), [0.95, 0.97, 1.0], 0.55 + 0.3 * d.2);
        }
    }

    // ---- the planes over you: where they really are, their lights, a short trail
    let privacy = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
    for p in list(st, &["sky"]).iter().take(6) {
        let (Some(az), Some(el)) = (p.get("az").and_then(Value::as_f64), p.get("el").and_then(Value::as_f64)) else { continue };
        let Some((x, y)) = sky_xy(az as f32, el as f32, w, hz) else { continue };
        let track = p.get("track").and_then(Value::as_f64).unwrap_or(90.0) as f32;
        let dir = if (track - 180.0).rem_euclid(360.0) < 180.0 { 1.0 } else { -1.0 }; // heading west: to the right
        let s = h * 0.016;
        let mut trail = PathBuilder::stroke(px((u * 1.4).max(1.0)));
        trail.move_to(point(px(ox + x - dir * s * 7.0), px(oy + y)));
        trail.line_to(point(px(ox + x - dir * s), px(oy + y)));
        if let Ok(path) = trail.build() {
            win.paint_path(path, col(pal.fg, 0.22));
        }
        rect(win, ox + x - s, oy + y - s * 0.16, s * 2.0, s * 0.32, pal.fg, 0.95);
        rect(win, ox + x - s * 0.25, oy + y - s * 0.75, s * 0.5, s * 1.5, pal.fg, 0.8);
        let blink = if (tf * 1.4).fract() < 0.12 { 1.0 } else { 0.0 };
        disc(win, ox + x - s * 0.1, oy + y - s * 0.75, s * 0.18, [1.0, 0.25, 0.25], 0.95);
        disc(win, ox + x - s * 0.1, oy + y + s * 0.75, s * 0.18, [0.3, 1.0, 0.45], 0.95);
        disc(win, ox + x + dir * s, oy + y, s * 0.22, [1.0, 1.0, 1.0], blink);
        if !privacy {
            let g = |k: &str| p.get(k).and_then(Value::as_str).unwrap_or("").to_string();
            let (flight, kind, from, to) = (g("flight"), g("type"), g("from"), g("to"));
            let ft = p.get("ft").and_then(Value::as_f64).unwrap_or(0.0);
            let kt = p.get("kt").and_then(Value::as_f64).unwrap_or(0.0);
            let mut title = if flight.is_empty() { "plane".to_string() } else { flight };
            if !kind.is_empty() {
                title = format!("{title} · {kind}");
            }
            let route = if !from.is_empty() && !to.is_empty() { format!("{from} → {to} · ") } else { String::new() };
            marks.push(PlaneMark { c: pal.accent, x: (x + s * 2.2).clamp(w * 0.245, w * 0.62), y: y - s * 2.2, title, line: format!("{route}{} ft · {} kt", (ft as i64 / 100) * 100, kt as i64) });
        }
    }
    marks
}
