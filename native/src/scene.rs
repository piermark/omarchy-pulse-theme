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
    /// where each Starlink satellite is drawn: it glides to where it really is
    pub sl: Vec<Option<(f32, f32)>>,
    pub iss: Option<(f32, f32)>,
    pub iss_trail: Vec<(f32, f32)>,
    /// rockets on their way up: (when it left, its name, a Starship?)
    pub liftoffs: Vec<(f64, String, bool)>,
    /// fireworks: (when, x, y, seed, small)
    pub bursts: Vec<(f64, f32, f32, u64, bool)>,
    pub flash: f64,
    pub shooting: Vec<(f64, f32, f32)>,
    /// the words of the last effect, and when
    pub say: Option<(f64, String)>,
    seed: u64,
}

impl World {
    /// An effect from the hub: the same kinds the web page plays.
    pub fn fx(&mut self, kind: &str, text: &str, small: bool, t: f64) {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let mut r = Rand::new(self.seed);
        match kind {
            "launch" => {
                let name = text.split(" · liftoff").next().unwrap_or(text).to_string();
                let big = name.to_lowercase().contains("starship");
                self.liftoffs.push((t, name, big));
            }
            "lightning" => self.flash = t,
            "shooting" => self.shooting.push((t, 0.1 + r.next() * 0.5, 0.04 + r.next() * 0.12)),
            "banner" | "tip" | "chime" | "text" | "flare" => {}
            _ => {
                let n = if small { 2 } else { 6 };
                for i in 0..n {
                    self.bursts.push((t + i as f64 * 0.45, 0.3 + r.next() * 0.4, 0.14 + r.next() * 0.26, (r.next() * 1e9) as u64, small));
                }
            }
        }
        if !text.is_empty() {
            self.say = Some((t, text.chars().take(60).collect()));
        }
    }
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
        World { stars, city, clouds, drops, last_t: 0.0, sl: Vec::new(), iss: None, iss_trail: Vec::new(), liftoffs: Vec::new(), bursts: Vec::new(), flash: 0.0, shooting: Vec::new(), say: None, seed: 42 }
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

pub(crate) fn rect(win: &mut Window, x: f32, y: f32, w: f32, h: f32, c: C, a: f32) {
    if a <= 0.004 || w <= 0.0 || h <= 0.0 {
        return;
    }
    win.paint_quad(PaintQuad::new(Bounds::new(point(px(x), px(y)), size(px(w), px(h)))).background(col(c, a)));
}

pub(crate) fn disc(win: &mut Window, cx: f32, cy: f32, r: f32, c: C, a: f32) {
    if a <= 0.004 || r <= 0.0 {
        return;
    }
    win.paint_quad(PaintQuad::new(Bounds::new(point(px(cx - r), px(cy - r)), size(px(r * 2.0), px(r * 2.0)))).background(col(c, a)).corner_radii(px(r)));
}

/// A soft light: rings of the same colour, fainter as they widen.
pub(crate) fn glow(win: &mut Window, cx: f32, cy: f32, r: f32, c: C, a: f32) {
    for i in 0..7 {
        let k = 1.0 - i as f32 / 7.0;
        disc(win, cx, cy, r * (1.0 - k * 0.86), c, a * 0.2 * (1.2 - k));
    }
}

pub(crate) fn gradient(win: &mut Window, x: f32, y: f32, w: f32, h: f32, top: C, ta: f32, bottom: C, ba: f32) {
    let bg = linear_gradient(180.0, linear_color_stop(col(top, ta), 0.0), linear_color_stop(col(bottom, ba), 1.0));
    win.paint_quad(PaintQuad::new(Bounds::new(point(px(x), px(y)), size(px(w), px(h)))).background(bg));
}

/// What the face on the sphere feels: your most urgent agent decides, as on the web page (faceMood).
fn face_mood(st: &Value) -> &'static str {
    if get(st, &["away"]).as_bool().unwrap_or(false) {
        return "sleep";
    }
    if get(st, &["speaking"]).as_bool().unwrap_or(false) {
        return "talk";
    }
    let rank = |s: &str| match s { "waiting" => 5, "failed" => 4, "working" => 3, "done" => 2, _ => 1 };
    let best = list(st, &["agents"]).iter().filter_map(|a| a.get("state").and_then(Value::as_str)).max_by_key(|s| rank(s));
    match best {
        Some("working") => "think",
        Some("waiting") => "alert",
        Some("done") => "happy",
        Some("failed") => "sad",
        _ => "idle",
    }
}

/// Is the point (x, y), in units of the face's size, on one of the face's lit features? Pulse 1.8's face (drawFace):
/// two tall rounded eyes that blink, look up when it thinks, widen when it needs you; a mouth for every mood.
fn face_lit(x: f32, y: f32, mood: &str, t: f32) -> bool {
    let seg = |ax: f32, ay: f32, bx: f32, by: f32, th: f32| -> bool {
        let (dx, dy) = (bx - ax, by - ay);
        let k = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        ((x - ax - dx * k).powi(2) + (y - ay - dy * k).powi(2)).sqrt() < th
    };
    let arc = |cx: f32, cy: f32, r: f32, a0: f32, a1: f32, th: f32| -> bool {
        let (dx, dy) = (x - cx, y - cy);
        let d = (dx * dx + dy * dy).sqrt();
        let a = dy.atan2(dx).rem_euclid(std::f32::consts::TAU);
        (d - r).abs() < th && a >= a0 && a <= a1
    };
    let pi = std::f32::consts::PI;
    let th = 0.045;
    let (mut eye_w, mut eye_h, gap) = (0.17, 0.30, 0.26);
    let (mut look_x, mut look_y) = (0.0, 0.0);
    if mood == "think" {
        look_x = (t * 1.3).sin() * 0.05;
        look_y = -0.05;
    }
    let blink = if (mood == "idle" || mood == "think") && (t % 4.2) < 0.13 { 0.12 } else { 1.0 };
    if mood == "alert" {
        eye_w *= 1.15;
        eye_h *= 1.2;
    }
    for side in [-1.0f32, 1.0] {
        let (ex, ey) = (side * gap + look_x, -0.08 + look_y);
        let on = match mood {
            "happy" => arc(ex, ey + 0.06, eye_w * 0.9, pi * 1.1, pi * 1.9, th),
            "sleep" => arc(ex, ey - 0.03, eye_w * 0.75, pi * 0.15, pi * 0.85, th),
            "sad" => seg(ex - eye_w * 0.7, ey - eye_w * 0.7, ex + eye_w * 0.7, ey + eye_w * 0.7, th) || seg(ex + eye_w * 0.7, ey - eye_w * 0.7, ex - eye_w * 0.7, ey + eye_w * 0.7, th),
            _ => {
                let hh = eye_h * blink;
                seg(ex, ey - (hh - eye_w).max(0.0) / 2.0, ex, ey + (hh - eye_w).max(0.0) / 2.0, eye_w.min(hh) / 2.0) // a rounded bar
            }
        };
        if on || (mood == "alert" && seg(ex - eye_w * 0.8, ey - eye_h * 0.85, ex + eye_w * 0.6, ey - eye_h, th * 0.8)) {
            return true;
        }
    }
    let my = 0.26;
    match mood {
        "sleep" => arc(0.0, my, 0.04, 0.0, std::f32::consts::TAU, th * 0.7),
        "happy" => arc(0.0, my - 0.08, 0.16, pi * 0.15, pi * 0.85, th),
        "sad" => arc(0.0, my + 0.08, 0.13, pi * 1.2, pi * 1.8, th),
        "alert" => arc(0.0, my, 0.06, 0.0, std::f32::consts::TAU, th * 0.8),
        "talk" => {
            let o = (0.3 + 0.7 * (t * 9.0).sin().abs()) * 0.09;
            (x / 0.1).powi(2) + ((y - my) / o).powi(2) < 1.0
        }
        "think" => {
            if seg(-0.08, my, 0.08, my, th) {
                return true;
            }
            (0..3).any(|i| {
                let on = ((t * 2.2 - i as f32 * 0.33).rem_euclid(1.0)) < 0.5;
                on && ((x - (0.55 + i as f32 * 0.13)).powi(2) + (y - (-0.45 - i as f32 * 0.08)).powi(2)).sqrt() < 0.05
            })
        }
        _ => seg(-0.08, my, 0.08, my, th),
    }
}

/// The LED sphere, after the Sphere of Las Vegas: a dark ball of thousands of LEDs, and only what is drawn lights up.
/// Its face follows your AI agents; the CPU runs round its lower rows; a halo in its own colour, and its light on
/// the floor.
#[allow(clippy::too_many_arguments)]
fn led_sphere(win: &mut Window, cx: f32, cy: f32, r: f32, st: &Value, pal: &Pal, t: f32, u: f32) {
    let mood = face_mood(st);
    let face_c = match mood { "alert" => pal.warn, "sad" => pal.crit, _ => mix(pal.warn, [1.0, 0.95, 0.4], 0.35) };
    let cpu = (num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
    glow(win, cx, cy, r * 1.8, face_c, 0.22);
    glow(win, cx, cy + r * 1.16, r * 1.3, face_c, 0.18); // its light on the wet floor
    disc(win, cx, cy, r * 1.012, mix(pal.accent, pal.bg, 0.4), 0.9);
    disc(win, cx, cy, r, crate::palette::scale(pal.bg, 0.45), 1.0);
    let rows = 46;
    let dark = mix(pal.bg, pal.accent2, 0.30);
    let pi = std::f32::consts::PI;
    for row in 0..rows {
        let phi = (row as f32 + 0.5) / rows as f32 * pi - pi / 2.0; // latitude, bottom to top
        let n = ((rows as f32 * 2.0) * phi.cos()).round().max(4.0) as i32; // LEDs on the half we see... all round: 2n
        for k in 0..n {
            let lam = (k as f32 + 0.5) / n as f32 * pi - pi / 2.0; // the front half
            let (x, y) = (phi.cos() * lam.sin(), -phi.sin());
            let facing = phi.cos() * lam.cos(); // 1 at the centre, 0 at the edge: the curve
            let s = 0.86; // the face's size, of the radius
            let lit = face_lit(x / s, y / s, mood, t);
            // the CPU as a ring of light running round the lower rows
            let band = y > 0.62 && y < 0.80 && ((lam / pi + 0.5 + t * 0.06).rem_euclid(1.0)) < cpu;
            let (c, a) = if lit { (face_c, 1.0) } else if band { (pal.accent, 0.85) } else { (dark, 0.55) };
            let lr = (r * pi / rows as f32 * 0.34 * (0.55 + 0.45 * facing)).max(u * 0.8);
            disc(win, cx + x * r, cy + y * r, lr, c, a * (0.30 + 0.70 * facing));
            if lit && facing > 0.5 {
                disc(win, cx + x * r, cy + y * r, lr * 2.1, face_c, 0.16);
            }
        }
    }
    // the glass: a soft light from above
    gradient(win, cx - r * 0.55, cy - r * 0.92, r * 1.1, r * 0.5, [1.0, 1.0, 1.0], 0.05, [1.0, 1.0, 1.0], 0.0);
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
pub fn paint(bounds: Bounds<Pixels>, world: &mut World, st: &Value, pal: &Pal, t: f64, sphere: bool, win: &mut Window) -> Vec<PlaneMark> {
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

    let frame = crate::life::Frame { w, h, ox, oy, hz, u, t: tf, sky: [wn, wt, wd] };
    crate::life::aurora(win, st, pal, &frame);
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
    let hide = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
    let taken = if sphere { 0 } else { crate::life::repos(win, st, pal, &frame, &mut marks, hide) };
    let mut r = Rand::new(77);
    for i in 0..(if sphere { 0 } else { 16 }) {
        if i < taken {
            for _ in 0..3 { r.next(); }
            continue; // a repo's tower stands here
        }
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
        gradient(win, x - bw / 2.0, oy + hz + u * 2.0, bw, bh * 0.55, pal.accent, 0.13, pal.accent, 0.0);
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
    // ---- the centre: the dome of the skyline, or the whole sphere of Pulse turning over the horizon
    let rr = if sphere { h * 0.19 } else { h * 0.135 };
    let (dcx, dcy) = (ox + w / 2.0, if sphere { oy + hz - rr * 1.12 } else { oy + hz });
    let arc = if sphere { std::f32::consts::TAU } else { std::f32::consts::PI };
    if sphere {
        glow(win, dcx, dcy, rr * 1.9, a2, 0.35);
        disc(win, dcx, dcy, rr, mix(bg, a2, 0.10), 0.55);
    } else {
        gradient(win, dcx - rr, dcy - rr, rr * 2.0, rr, a2, 0.0, a2, 0.10);
    }
    if sphere {
        led_sphere(win, dcx, dcy, rr, st, pal, tf, u);
    } else {
        let mut dome = PathBuilder::stroke(px((u * 1.4).max(1.0)));
        for m in 0..=8 {
            let kx = (m as f32 / 8.0 * std::f32::consts::PI + tf * if sphere { 0.25 } else { 0.08 }).cos(); // the meridians turn
            for i in 0..=48 {
                let a = i as f32 / 48.0 * arc;
                let p = point(px(dcx + a.cos() * rr * kx), px(dcy - a.sin() * rr));
                if i == 0 { dome.move_to(p) } else { dome.line_to(p) }
            }
        }
        let lats: Vec<f32> = if sphere { vec![-0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75] } else { vec![0.2, 0.4, 0.6, 0.8] };
        for l in lats {
            let a = l * std::f32::consts::FRAC_PI_2;
            dome.move_to(point(px(dcx - a.cos() * rr), px(dcy - a.sin() * rr)));
            dome.line_to(point(px(dcx + a.cos() * rr), px(dcy - a.sin() * rr)));
        }
        if let Ok(path) = dome.build() {
            win.paint_path(path, col(a2, 0.55));
        }
        let mut rim = PathBuilder::stroke(px((u * 2.2).max(1.5)));
        for i in 0..=72 {
            let a = i as f32 / 72.0 * arc;
            let p = point(px(dcx + a.cos() * rr), px(dcy - a.sin() * rr));
            if i == 0 { rim.move_to(p) } else { rim.line_to(p) }
        }
        if let Ok(path) = rim.build() {
            win.paint_path(path, col(pal.accent, 0.9));
        }
    }
    let rr = if sphere { rr * 1.05 } else { rr };
    // (the drones below fly over its top either way: dcy - rr is the top of both)
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
        let hide = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
        if !hide {
        marks.push(PlaneMark { c, x: (x - w * 0.05).clamp(w * 0.245, w * 0.62), y: y + ds * 1.2, title: format!("● {name}"), line: if what.is_empty() { state.to_string() } else { format!("{state} · {what}") } });
        }
    }

    if !sphere {
        crate::life::paint(win, st, pal, &frame, &mut marks);
    }
    crate::life::balloon(win, st, pal, &frame, &mut marks);
    crate::life::banner(win, st, pal, &frame, &mut marks);
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

    let privacy = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
    // ---- the ISS when it is over you: a bright point with its panels, and where it has just been
    match (num(st, &["space", "iss", "az"]), num(st, &["space", "iss", "el"])) {
        (Some(az), Some(el)) => {
            let tx = w / 2.0 + ((az as f32 - 180.0) / 90.0).clamp(-1.05, 1.05) * w * 0.44;
            let ty = hz - h * 0.05 - (el as f32 / 90.0) * hz * 0.6;
            let p = world.iss.map(|q| (q.0 + (tx - q.0) * 0.08, q.1 + (ty - q.1) * 0.08)).unwrap_or((tx, ty));
            world.iss = Some(p);
            world.iss_trail.push(p);
            if world.iss_trail.len() > 60 {
                world.iss_trail.remove(0);
            }
            let mut tr = PathBuilder::stroke(px((u * 1.5).max(1.0)));
            for (i, q) in world.iss_trail.iter().enumerate() {
                let pt = point(px(ox + q.0), px(oy + q.1));
                if i == 0 { tr.move_to(pt) } else { tr.line_to(pt) }
            }
            if world.iss_trail.len() > 2 {
                if let Ok(path) = tr.build() {
                    win.paint_path(path, col(pal.fg, 0.3));
                }
            }
            let s = h * 0.01;
            glow(win, ox + p.0, oy + p.1, s * 3.0, [1.0, 1.0, 0.9], 0.9);
            rect(win, ox + p.0 - s * 1.6, oy + p.1 - s * 0.25, s * 3.2, s * 0.5, pal.accent, 0.9);
            disc(win, ox + p.0, oy + p.1, s * 0.5, [1.0, 1.0, 1.0], 1.0);
            let alt = num(st, &["space", "iss", "alt"]).unwrap_or(420.0);
            marks.push(PlaneMark { c: pal.warn, x: (p.0 + s * 2.5).clamp(w * 0.245, w * 0.62), y: p.1 - s * 3.0, title: "ISS".into(), line: format!("{alt:.0} km · {el:.0}° up") });
        }
        _ => {
            world.iss = None;
            world.iss_trail.clear();
        }
    }
    // ---- the Starlink train really in your sky: it glides, a thread joins the row, each one glints
    let sats = list(st, &["starlink", "sats"]);
    world.sl.resize(sats.len(), None);
    let mut pts: Vec<(f32, f32)> = Vec::new();
    for (i, sat) in sats.iter().enumerate() {
        let (Some(az), Some(el)) = (sat.get("az").and_then(Value::as_f64), sat.get("el").and_then(Value::as_f64)) else { world.sl[i] = None; continue };
        if el < 10.0 {
            world.sl[i] = None;
            continue;
        }
        let Some(p) = sky_xy(az as f32, el as f32, w, hz) else { world.sl[i] = None; continue };
        let q = match world.sl[i] {
            Some(q) if (q.0 - p.0).abs() < w * 0.2 => (q.0 + (p.0 - q.0) * 0.025, q.1 + (p.1 - q.1) * 0.025),
            _ => p,
        };
        world.sl[i] = Some(q);
        pts.push(q);
    }
    if !pts.is_empty() {
        let mut thread = PathBuilder::stroke(px((u * 1.0).max(1.0)));
        for (i, q) in pts.iter().enumerate() {
            let pt = point(px(ox + q.0), px(oy + q.1));
            if i == 0 { thread.move_to(pt) } else { thread.line_to(pt) }
        }
        if pts.len() > 1 {
            if let Ok(path) = thread.build() {
                win.paint_path(path, col(pal.fg, 0.22));
            }
        }
        for (i, q) in pts.iter().enumerate() {
            let glint = 0.6 + 0.4 * (tf * 3.0 - i as f32 * 0.7).sin().max(0.0);
            let s = h * if i == 0 { 0.0042 } else { 0.003 };
            glow(win, ox + q.0, oy + q.1, s * 3.5, [1.0, 1.0, 1.0], 0.5 * glint);
            disc(win, ox + q.0, oy + q.1, s, [1.0, 1.0, 1.0], glint);
        }
        let head = pts[0];
        marks.push(PlaneMark { c: pal.fg, x: (head.0 + h * 0.02).clamp(w * 0.245, w * 0.62), y: head.1 - h * 0.05, title: "STARLINK TRAIN".into(), line: format!("{} satellites", pts.len()) });
    }
    // ---- shooting stars, fireworks and lightning: the hub's effects
    world.shooting.retain(|s0| t - s0.0 < 1.2);
    for s0 in &world.shooting {
        let k = ((t - s0.0) / 1.2) as f32;
        let (x, y) = (ox + (s0.1 + k * 0.22) * w, oy + (s0.2 + k * 0.07) * h);
        let mut p = PathBuilder::stroke(px((u * 2.0).max(1.2)));
        p.move_to(point(px(x - w * 0.05), px(y - h * 0.016)));
        p.line_to(point(px(x), px(y)));
        if let Ok(path) = p.build() {
            win.paint_path(path, col([1.0, 0.97, 0.9], 0.9 * (1.0 - k)));
        }
    }
    world.bursts.retain(|b| t - b.0 < 2.6);
    for b in &world.bursts {
        let age = (t - b.0) as f32;
        if age < 0.0 {
            continue;
        }
        let k = age / 2.6;
        let mut r = Rand::new(b.3);
        let c = [pal.accent, pal.accent2, pal.warn, pal.ok][(b.3 % 4) as usize];
        let (bx, by) = (ox + b.1 * w, oy + b.2 * h);
        let reach = h * if b.4 { 0.07 } else { 0.15 } * (1.0 - (1.0 - k).powi(3));
        for _ in 0..if b.4 { 40 } else { 90 } {
            let (a, sp) = (r.next() * 6.283, 0.35 + r.next() * 0.65);
            let (x, y) = (bx + a.cos() * reach * sp, by + a.sin() * reach * sp + k * k * h * 0.05);
            disc(win, x, y, (u * (2.6 - 1.8 * k)).max(1.0), mix(c, [1.0, 1.0, 1.0], (1.0 - k) * 0.6), (1.0 - k).powf(1.5));
        }
        glow(win, bx, by, h * 0.12 * (1.0 - k), c, 0.5 * (1.0 - k * 3.0).max(0.0));
    }
    let fl = (t - world.flash) as f32;
    if world.flash > 0.0 && fl < 0.5 {
        rect(win, ox, oy, w, h, [1.0, 1.0, 1.0], 0.35 * (1.0 - fl / 0.5) * if (fl * 20.0) as i32 % 2 == 0 { 1.0 } else { 0.4 });
    }
    // ---- a real launch: a rocket you can recognise climbs from your horizon (Pulse 1.8's liftoff)
    world.liftoffs.retain(|l| t - l.0 < 26.0);
    for (li, l) in world.liftoffs.iter().enumerate() {
        let age = (t - l.0) as f32;
        let big = l.2;
        let lx = ox + w * (0.66 + 0.05 * li as f32);
        let rh = h * if big { 0.075 } else { 0.058 };
        let rw = rh * if big { 0.13 } else { 0.085 };
        let climb = h * 0.0052 * age * age + h * 0.004 * age;
        let ry = oy + hz - climb; // the base of the rocket
        let lean = (age - 6.0).max(0.0) * rw * 0.5;
        let x = lx - lean;
        // pad smoke, rolling outwards
        if age < 9.0 {
            for i in 0..9 {
                let k = (age / 9.0).min(1.0);
                let sx = (i as f32 - 4.0) * rh * 0.22 * (1.0 + age * 0.35);
                disc(win, lx + sx, oy + hz - rh * 0.05, rh * (0.16 + 0.05 * ((i * 7) % 3) as f32) * (1.0 + age * 0.3), mix(pal.fg, bg, 0.35), 0.22 * (1.0 - k));
            }
        }
        // the trail: smoke that widens and fades behind it
        for i in 0..22 {
            let back = i as f32 / 22.0;
            let ty = ry + back * climb.min(h * 0.5);
            if ty > oy + hz {
                break;
            }
            disc(win, x + lean * back + (tf * 2.0 + i as f32).sin() * rw * back * 2.0, ty, rw * (0.7 + back * 3.2), mix(pal.fg, mix(a2, pal.warn, 0.5), wt * 0.6), 0.26 * (1.0 - back));
        }
        // the flame: a white heart, then warm, flickering
        let fh = rh * (0.9 + 0.3 * (tf * 40.0).sin());
        gradient(win, x - rw * 0.45, ry, rw * 0.9, fh, [1.0, 0.98, 0.9], 0.95, pal.warn, 0.0);
        glow(win, x, ry + rh * 0.1, rh * 1.4, pal.warn, 0.7);
        let sep = age > 9.5;
        // the second stage (and the whole rocket before the separation)
        let hull = if big { [0.78, 0.80, 0.84] } else { [0.95, 0.95, 0.96] };
        let (body_y, body_h) = if sep { (ry - rh * 0.42, rh * 0.42) } else { (ry - rh, rh) };
        rect(win, x - rw / 2.0, body_y, rw, body_h, hull, 1.0);
        rect(win, x, body_y, rw / 2.0, body_h, scale(hull, 0.82), 1.0);
        disc(win, x, body_y, rw / 2.0, hull, 1.0); // the fairing
        if !big && !sep {
            rect(win, x - rw / 2.0, ry - rh * 0.6, rw, rh * 0.07, [0.1, 0.1, 0.12], 1.0); // the black interstage
            rect(win, x - rw * 0.9, ry - rh * 0.1, rw * 0.4, rh * 0.1, [0.15, 0.15, 0.17], 1.0);
            rect(win, x + rw * 0.5, ry - rh * 0.1, rw * 0.4, rh * 0.1, [0.15, 0.15, 0.17], 1.0);
        }
        if sep {
            // the booster comes off and falls back
            let fa = age - 9.5;
            let by = ry + rh * 0.1 + fa * fa * h * 0.004;
            if by < oy + hz {
                rect(win, x + fa * rw * 1.2 - rw / 2.0, by, rw, rh * 0.58, scale(hull, 0.9), 0.95);
            }
        }
        let phase = if age < 5.0 { "liftoff" } else if age < 9.5 { "max Q" } else { "stage separation" };
        if !privacy {
            marks.push(PlaneMark { c: pal.warn, x: (x - ox + rh * 0.5).clamp(w * 0.245, w * 0.70), y: (ry - oy - rh * 1.2).max(h * 0.03), title: l.1.to_uppercase(), line: format!("T+00:{:02} · {phase}", age as i32) });
        }
    }

    // ---- the planes over you: where they really are, their lights, a short trail
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
        let (px0, py0) = (ox + x, oy + y);
        let mut body = PathBuilder::fill();
        let pt = |fx: f32, fy: f32| point(px(px0 + dir * fx * s), px(py0 + fy * s));
        body.move_to(pt(1.25, 0.0)); // the nose
        body.line_to(pt(0.95, -0.13));
        body.line_to(pt(0.25, -0.15));
        body.line_to(pt(-0.35, -1.05)); // a wing, swept back
        body.line_to(pt(-0.62, -1.05));
        body.line_to(pt(-0.28, -0.15));
        body.line_to(pt(-0.95, -0.12));
        body.line_to(pt(-1.2, -0.45)); // the tail
        body.line_to(pt(-1.35, -0.45));
        body.line_to(pt(-1.2, 0.0));
        body.line_to(pt(-1.35, 0.45));
        body.line_to(pt(-1.2, 0.45));
        body.line_to(pt(-0.95, 0.12));
        body.line_to(pt(-0.28, 0.15));
        body.line_to(pt(-0.62, 1.05));
        body.line_to(pt(-0.35, 1.05));
        body.line_to(pt(0.25, 0.15));
        body.line_to(pt(0.95, 0.13));
        body.close();
        if let Ok(path) = body.build() {
            win.paint_path(path, col(mix(pal.fg, [1.0, 1.0, 1.0], 0.4), 0.96));
        }
        let blink = if (tf * 1.4).fract() < 0.12 { 1.0 } else { 0.0 };
        glow(win, ox + x - dir * s * 0.48, oy + y - s * 1.05, s * 0.5, [1.0, 0.25, 0.25], 0.95);
        glow(win, ox + x - dir * s * 0.48, oy + y + s * 1.05, s * 0.5, [0.3, 1.0, 0.45], 0.95);
        glow(win, ox + x - dir * s * 1.2, oy + y, s * 0.9, [1.0, 1.0, 1.0], blink);
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
            let tx = if y < h * 0.27 && x > w * 0.33 && x < w * 0.64 { w * 0.645 } else { (x + s * 2.2).clamp(w * 0.245, w * 0.62) };
            marks.push(PlaneMark { c: pal.accent, x: tx, y: (y - s * 1.2).max(h * 0.02), title, line: format!("{route}{} ft · {} kt", (ft as i64 / 100) * 100, kt as i64) });
        }
    }
    marks
}
