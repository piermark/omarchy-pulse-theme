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
    /// scene orbit: where each core's moon is on its ring
    pub rings: Vec<f32>,
    /// the rest of the world (storms, seasons, earthquakes, the garden...) and what it keeps
    pub more: crate::extra::Mem,
    /// the scenes in code (mars, monolith) and what they keep
    pub code: crate::worlds::Mem,
    /// words the scene wants written, and what a click can find: made new at every frame
    pub labels: Vec<crate::draw::Label>,
    pub hits: Vec<crate::draw::Hit>,
    /// the sphere (or the dome) on the screen: centre and radius, for what main.rs writes on it
    pub centre: Option<(f32, f32, f32)>,
    /// where the pointer is, in the wallpaper's pixels, and when it last moved (the face looks at you)
    pub pointer: Option<(f32, f32, f64)>,
    /// your own picture is the background ([pulse] wallpaper, mode "live"): only Pulse's live things are drawn
    pub wall: bool,
    /// a camera panel exists: the sphere shows it in its turn
    pub has_cam: bool,
    /// [pulse] secondary = "simple" on a second monitor: the sky, the city and the centre, nothing else
    pub simple: bool,
    /// the last frame's measures, for an effect that needs a place
    pub last: Option<crate::life::Frame>,
    /// your heartbeats on the line of the horizon: when each one started
    pub beats: Vec<f64>,
    /// how tall each core's tower is now (it follows the load smoothly)
    pub heights: Vec<f32>,
    /// the music's bars, from the hub's live channel (empty when nothing plays)
    pub audio: Vec<f32>,
    /// the local time's offset in seconds, for a scene's clock
    pub tz: i64,
    /// what the sphere shows now: its face, or one of its programmes (the clock, the weather, the CPU, the news...)
    pub prog: &'static str,
    /// when the last alert came (the sphere turns to it)
    pub alert_at: f64,
    /// what a scene in code sees, made once for each state of the hub
    view: Option<(f64, Value)>,
    /// effects the scenes asked for that make a sound: main.rs plays them
    pub sounds: Vec<&'static str>,
}

impl World {
    /// An effect from the hub: the same kinds the web page plays.
    pub fn fx(&mut self, kind: &str, text: &str, small: bool, t: f64) {
        self.fx_raw(kind, text, small, t, &Value::Null)
    }

    /// The same, with everything the hub said about it (an earthquake's magnitude and direction).
    pub fn fx_raw(&mut self, kind: &str, text: &str, small: bool, t: f64, raw: &Value) {
        crate::extra::event(&mut self.more, kind, raw, t, self.last.as_ref());
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let mut r = Rand::new(self.seed);
        match kind {
            "launch" => {
                let name = text.split(" · liftoff").next().unwrap_or(text).to_string();
                let big = name.to_lowercase().contains("starship");
                self.liftoffs.push((t, name, big));
            }
            "lightning" => {}
            "shooting" => self.shooting.push((t, 0.1 + r.next() * 0.5, 0.04 + r.next() * 0.12)),
            "banner" | "tip" | "chime" | "text" | "say" | "flare" | "quake" | "tsunami" | "build" | "doorbell" | "train" | "summary" | "starlink" | "car" | "push" | "commit" | "iss" => {
                if kind == "iss" {
                    self.shooting.push((t, 0.1, 0.08));
                }
            }
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
        World { stars, city, clouds, drops, last_t: 0.0, sl: Vec::new(), iss: None, iss_trail: Vec::new(), liftoffs: Vec::new(), bursts: Vec::new(), flash: 0.0, shooting: Vec::new(), say: None, seed: 42, rings: Vec::new(), more: Default::default(), code: Default::default(), labels: Vec::new(), hits: Vec::new(), centre: None, pointer: None, wall: false, has_cam: false, simple: false, last: None, beats: Vec::new(), heights: Vec::new(), audio: Vec::new(), tz: 0, prog: "face", alert_at: -1e9, view: None, sounds: Vec::new() }
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
pub(crate) fn face_mood(st: &Value) -> &'static str {
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
pub(crate) fn face_lit(x: f32, y: f32, mood: &str, t: f32) -> bool {
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
fn led_sphere(win: &mut Window, cx: f32, cy: f32, r: f32, st: &Value, pal: &Pal, t: f32, u: f32, gaze: (f32, f32), prog: &str, audio: &[f32]) {
    let mood = face_mood(st);
    let cores: Vec<f32> = list(st, &["sys", "cores"]).iter().filter_map(|c| c.as_f64().map(|v| (v as f32 / 100.0).clamp(0.02, 1.0))).take(32).collect();
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
            let lit = match prog {
                // the face looks where the pointer is
                "face" => face_lit((x - gaze.0 * 0.10) / s, (y - gaze.1 * 0.07) / s, mood, t),
                // a bar for each core, or for each band of the music
                "cpu" | "music" => {
                    let bars: Vec<f32> = if prog == "music" && !audio.is_empty() { audio.iter().map(|v| v.clamp(0.03, 1.0)).collect() } else if prog == "music" { (0..16).map(|i| 0.2 + 0.6 * (t * (2.0 + i as f32 * 0.4) + i as f32).sin().abs()).collect() } else { cores.clone() };
                    let n = bars.len().max(1);
                    let fx = (x + 0.72) / 1.44;
                    (0.0..1.0).contains(&fx) && (fx * n as f32).fract() < 0.72 && y < 0.42 && y > 0.42 - bars[((fx * n as f32) as usize).min(n - 1)] * 0.62
                }
                "alert" => ((x * x + y * y).sqrt() * 6.0 - t * 4.0).rem_euclid(2.0) < 0.5,
                // the other programmes are words (main.rs writes them): a thin frame of light round them
                _ => (y.abs() - 0.50).abs() < 0.022 && x.abs() < 0.62,
            };
            let face_c = if prog == "alert" { pal.crit } else if matches!(prog, "face") { face_c } else { mix(pal.accent, pal.accent2, (y + 0.5).clamp(0.0, 1.0)) };
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

/// What the sphere shows now, as the web page chooses it (progNow): what is urgent first, then its face while your
/// agents work or you are near, the music, and otherwise its programmes in turn, eleven seconds each.
pub fn sphere_prog(st: &Value, t: f64, watching: bool, alert: bool, has_cam: bool) -> &'static str {
    let now = num(st, &["t"]).unwrap_or(0.0);
    let face = get(st, &["ui", "face"]).as_str().unwrap_or("agents").to_string();
    if alert {
        return "alert";
    }
    if let Some(start) = list(st, &["calendar"]).first().and_then(|c| num(c, &["start"])) {
        if start - now > 0.0 && start - now <= 60.0 {
            return "meeting";
        }
    }
    if !get(st, &["listen"]).is_null() || get(st, &["speaking"]).as_bool().unwrap_or(false) {
        return "face";
    }
    if let Some(net) = num(st, &["space", "launch", "net"]) {
        if get(st, &["space", "launch", "status"]).as_str() == Some("Go") && net - now > 0.0 && net - now < 3600.0 {
            return "launch";
        }
    }
    if get(st, &["away"]).as_bool().unwrap_or(false) && face != "off" {
        return "face";
    }
    if get(st, &["focus"]).is_object() {
        return "focus";
    }
    if face != "off" && (!list(st, &["agents"]).is_empty() || face == "always" || watching) {
        return "face";
    }
    if get(st, &["music"]).as_bool().unwrap_or(false) {
        return "music";
    }
    let mut progs = vec!["clock", "weather", "cpu", "news"];
    if get(st, &["car", "battery"]).is_number() && get(st, &["ui", "cars"]).as_bool().unwrap_or(true) {
        progs.push("car");
    }
    if has_cam && !get(st, &["privacy"]).as_bool().unwrap_or(false) {
        progs.push("camera");
    }
    progs[((t / 11.0) as usize) % progs.len()]
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
pub fn paint(bounds: Bounds<Pixels>, world: &mut World, st: &Value, pal: &Pal, t: f64, kind: &str, bare: bool, win: &mut Window) -> Vec<PlaneMark> {
    let sphere = kind == "sphere";
    let (orbit, calm, hacker, noir) = (kind == "orbit", kind == "calm", kind == "hacker", kind == "noir");
    let (mars, monolith, pond) = (kind == "mars", kind == "monolith", kind == "pond");
    let wall = world.wall;
    let simple = world.simple;
    let bare = bare || orbit || noir || mars || monolith || pond || wall; // no near towers, no dome, no city life: the scene draws its own centre
    let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let (ox, oy) = (f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let (qx, qy) = crate::extra::shake(&mut world.more, t, h); // an earthquake shakes the whole picture
    let (ox, oy) = (ox + qx, oy + qy);
    world.labels.clear();
    world.hits.clear();
    world.centre = None;
    let hz = h * HORIZON;
    let u = h / 1080.0;
    let dt = if world.last_t > 0.0 { ((t - world.last_t) as f32).clamp(0.0, 0.25) } else { 0.0 };
    world.last_t = t;
    // [pulse] motion = "reduced": the world stands still (the data still moves it)
    let still = get(st, &["ui", "motion"]).as_str() == Some("reduced");
    let tf = if still { 0.0 } else { t as f32 };
    let dt = if still { 0.0 } else { dt };
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
    if !wall {
        gradient(win, ox, oy, w, hz * 0.7 + 1.0, top, 1.0, midc, 1.0);
        gradient(win, ox, oy + hz * 0.7, w, hz * 0.3 + 1.0, midc, 1.0, low, 1.0);
    }

    if noir {
        crate::modes::noir_sky(win, &crate::life::Frame { w, h, ox, oy, hz, u, t: tf, sky: [wn, wt, wd] });
    }
    // ---- stars, fewer behind the clouds
    let starry = if wall { 0.0 } else { wn * (1.0 - cloud * 0.85) };
    if starry > 0.03 {
        for s in &world.stars {
            let tw = 0.72 + 0.28 * (tf * (0.5 + s.2) + s.3).sin();
            let z = (u * (0.8 + s.2 * 1.3)).max(1.0);
            rect(win, ox + s.0 * w, oy + s.1 * hz * 0.96, z, z, pal.fg, starry * s.2 * tw * (1.0 - s.1 * 0.6));
        }
    }

    // ---- the real sky over you: the bright stars where they are, their constellations joined and named, the planets
    if starry > 0.05 {
        let stars = list(st, &["night", "stars"]);
        let pos: Vec<Option<(f32, f32)>> = stars.iter().map(|s0| match (s0.get("az").and_then(Value::as_f64), s0.get("el").and_then(Value::as_f64)) {
            (Some(az), Some(el)) => sky_xy(az as f32, el as f32, w, hz),
            _ => None,
        }).collect();
        let mut lines = PathBuilder::stroke(px((u * 1.0).max(1.0)));
        let mut any = false;
        for l in list(st, &["night", "lines"]) {
            let (Some(a), Some(b)) = (l.get(0).and_then(Value::as_u64), l.get(1).and_then(Value::as_u64)) else { continue };
            if let (Some(Some(pa)), Some(Some(pb))) = (pos.get(a as usize), pos.get(b as usize)) {
                lines.move_to(point(px(ox + pa.0), px(oy + pa.1)));
                lines.line_to(point(px(ox + pb.0), px(oy + pb.1)));
                any = true;
            }
        }
        if any {
            if let Ok(path) = lines.build() {
                win.paint_path(path, col(pal.accent, 0.22 * starry));
            }
        }
        for (i, s0) in stars.iter().enumerate() {
            let Some(Some(p)) = pos.get(i) else { continue };
            let mag = s0.get("m").and_then(Value::as_f64).unwrap_or(2.0) as f32;
            let br = (1.0 - (mag + 1.0) / 5.0).clamp(0.25, 1.0);
            let tw = 0.8 + 0.2 * (tf * 1.7 + i as f32).sin();
            glow(win, ox + p.0, oy + p.1, u * 7.0 * br, [1.0, 0.98, 0.92], 0.5 * starry * br);
            disc(win, ox + p.0, oy + p.1, (u * 1.9 * br).max(1.0), [1.0, 0.98, 0.92], starry * tw);
        }
        for nm in list(st, &["night", "names"]) {
            let idx: Vec<usize> = nm.get("i").and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_u64().map(|v| v as usize)).collect()).unwrap_or_default();
            let pts: Vec<(f32, f32)> = idx.iter().filter_map(|i| pos.get(*i).copied().flatten()).collect();
            if pts.len() >= 2 {
                let (cx0, cy0) = (pts.iter().map(|p| p.0).sum::<f32>() / pts.len() as f32, pts.iter().map(|p| p.1).sum::<f32>() / pts.len() as f32);
                if cy0 < h * 0.32 && cx0 > w * 0.34 && cx0 < w * 0.66 {
                    continue; // never over the clock
                }
                marks.push(PlaneMark { c: mix(pal.accent, pal.bg, 0.35), x: cx0.clamp(w * 0.245, w * 0.70), y: cy0 + h * 0.012, title: nm.get("n").and_then(Value::as_str).unwrap_or("").to_string(), line: String::new() });
            }
        }
    }
    for pl in list(st, &["night", "planets"]).iter().filter(|_| !wall) {
        let (Some(az), Some(el)) = (pl.get("az").and_then(Value::as_f64), pl.get("el").and_then(Value::as_f64)) else { continue };
        let Some(p) = sky_xy(az as f32, el as f32, w, hz) else { continue };
        let name = pl.get("n").and_then(Value::as_str).unwrap_or("");
        let c = match name { "mars" => [1.0, 0.55, 0.4], "venus" => [1.0, 1.0, 0.9], "jupiter" => [1.0, 0.9, 0.75], "saturn" => [0.95, 0.85, 0.6], _ => [0.9, 0.9, 1.0] };
        let a = (wn + wt * 0.6) * (1.0 - cloud * 0.7);
        glow(win, ox + p.0, oy + p.1, u * 10.0, c, 0.6 * a);
        disc(win, ox + p.0, oy + p.1, (u * 2.6).max(1.2), c, a);
        if name == "saturn" {
            rect(win, ox + p.0 - u * 6.0, oy + p.1 - u * 0.5, u * 12.0, (u * 1.0).max(1.0), c, 0.8 * a);
        }
        if a > 0.2 {
            marks.push(PlaneMark { c, x: (p.0 + u * 10.0).clamp(w * 0.245, w * 0.70), y: p.1 - u * 8.0, title: name.to_string(), line: String::new() });
        }
    }

    // ---- the Moon and the Sun, where they really are
    let sky_on = !wall;
    if let (true, Some(az), Some(el)) = (sky_on, num(st, &["night", "moon", "az"]), num(st, &["night", "moon", "el"])) {
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
    if let (true, Some(az), Some(el)) = (sky_on && get(st, &["ui", "sun"]).as_bool().unwrap_or(true), num(st, &["night", "sun", "az"]), num(st, &["night", "sun", "el"])) {
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
    world.last = Some(frame);
    let still_l = get(st, &["ui", "motion"]).as_str() == Some("reduced");
    let polish = crate::look::on(st) && !wall && !noir && !mars;
    let lt = if polish { crate::look::light(st, pal, &frame) } else { None };
    if let (Some(l), true) = (&lt, sky_on) {
        crate::look::rays(win, l, &frame, still_l);
    }
    if sky_on {
        crate::extra::sky(win, st, pal, &frame, &mut world.more, t, &mut world.labels);
        crate::life::aurora(win, st, pal, &frame);
    }
    // ---- your clouds, moving with your wind: slowly when there is almost none, but never standing still
    let toward = (wind_dir + 180.0).to_radians(); // the wind comes from windDir; east is on the left
    let mut vx = -toward.sin() * (wind / 60.0);
    let floor = 0.012;
    if vx.abs() < floor {
        vx = if vx < 0.0 { -floor } else { floor };
    }
    let shown = if wall { 0 } else { ((cloud * 12.0).round() as usize).min(world.clouds.len()) };
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
    for b in world.city.iter().filter(|_| !noir && !wall && !mars) {
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

    if !noir && !wall {
        // ---- the ground: pulse.js's floor, and the grid that runs to the horizon
        gradient(win, ox, oy + hz, w, h - hz, mix(bg, a2, 0.2), 1.0, mix(bg, black, 0.5), 1.0);
        gradient(win, ox, oy + hz, w, h * 0.06, mix(pal.accent, a2, 0.5), 0.10 + 0.12 * (wn + wt * 0.6), bg, 0.0);
        if polish {
            // 1.8: the air over the far city, mist between its rows, the wet floor that mirrors it
            let hzc = crate::look::air(win, pal, &frame, low);
            let fog = if code == 45 || code == 48 { 1.0 } else { 0.0 };
            crate::look::mist(win, &frame, hzc, (0.25 + 0.35 * wn + 0.5 * wt + fog).min(1.0) * 0.9, still_l);
            if !orbit {
                crate::look::mirror(win, &world.city, pal, &frame, low);
            }
        }
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
    }
    if let (Some(l), false) = (&lt, wall || noir) {
        crate::look::light_path(win, l, &frame, still_l); // its light on the floor
    }
    // ---- the near city: a tower for each CPU core either side of the centre
    let cpu = (num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
    let hide = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
    let taken = if bare || calm || simple { 0 } else { crate::life::repos(win, st, pal, &frame, &mut marks, hide) };
    if !bare && !calm {
        let r_centre = if sphere { h * 0.19 * 1.08 } else { h * 0.135 * 1.18 };
        crate::look::towers(win, st, pal, &frame, lt, &mut world.heights, &world.audio, taken, r_centre.max(w * 0.085), polish);
    }
    let (mut rr, mut dcx, mut dcy) = (0.0f32, ox + w / 2.0, oy + hz);
    if !bare {
        // ---- the centre: the dome of the skyline, or the whole sphere of Pulse turning over the horizon
        rr = if sphere { h * 0.19 } else { h * 0.135 };
        dcy = if sphere { oy + hz - rr * 1.12 } else { oy + hz };
        dcx = ox + w / 2.0;
        let arc = if sphere { std::f32::consts::TAU } else { std::f32::consts::PI };
        if sphere {
            glow(win, dcx, dcy, rr * 1.9, a2, 0.35);
            disc(win, dcx, dcy, rr, mix(bg, a2, 0.10), 0.55);
        } else {
            gradient(win, dcx - rr, dcy - rr, rr * 2.0, rr, a2, 0.0, a2, 0.10);
        }
        if sphere {
            let eyes = get(st, &["ui", "eyes"]).as_bool().unwrap_or(true);
            let gaze = match world.pointer {
                Some((px0, py0, at)) if eyes && t - at < 15.0 => (((px0 - (dcx - ox)) / (h * 0.45)).clamp(-1.0, 1.0), ((py0 - (dcy - oy)) / (h * 0.45)).clamp(-1.0, 1.0)),
                _ => (0.0, 0.0),
            };
            let watching = matches!(world.pointer, Some((_, _, at)) if eyes && t - at < 6.0);
            world.prog = sphere_prog(st, t, watching, t - world.alert_at < 8.0, world.has_cam);
            led_sphere(win, dcx, dcy, rr, st, pal, tf, u, gaze, world.prog, &world.audio);
            // a ball, not a disc: the light it catches up on the left, and its edge turning away into the dark
            glow(win, dcx - rr * 0.36, dcy - rr * 0.42, rr * 0.55, [1.0, 1.0, 1.0], 0.10);
            for (k, a) in [(0.985, 0.42), (0.95, 0.26), (0.91, 0.14)] {
                crate::draw::arc(win, dcx, dcy, rr * k, rr * k, 0.0, std::f32::consts::TAU, rr * 0.05, pal.bg, a);
            }
            crate::draw::arc(win, dcx, dcy, rr * 0.99, rr * 0.99, -2.6, -1.2, (u * 2.0).max(1.5), mix(pal.fg, a2, 0.4), 0.55);
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
        rr = if sphere { rr * 1.05 } else { rr };
        // (the drones below fly over its top either way: dcy - rr is the top of both)
    }
    // ---- your AI agents: a drone each over the dome, its light in the colour of what it is doing
    let agents = list(st, &["agents"]);
    let n_ag = if bare || simple { 0 } else { agents.len().min(6) };
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

    if !bare && !calm && !simple {
        crate::life::paint(win, st, pal, &frame, &mut marks);
    }
    if !bare && !simple {
        crate::extra::ground(win, st, pal, &frame, &mut world.more, t, &mut world.labels, kind);
        crate::extra::net(win, st, pal, &frame, &mut world.more, t, &mut world.labels);
    }
    if !bare {
        world.centre = Some((dcx - ox, dcy - oy, rr));
    }
    if orbit {
        crate::modes::orbit(win, st, pal, &frame, &mut world.rings, dt, face_mood(st));
    }
    if hacker {
        crate::modes::radar(win, st, pal, &frame, &mut marks, hide);
    }
    if noir {
        crate::modes::noir_city(win, st, pal, &frame, &mut marks);
    }
    crate::life::balloon(win, st, pal, &frame, &mut marks);
    crate::life::banner(win, st, pal, &frame, &mut marks);
    // the line of the horizon: the machine's pulse, livelier with the CPU (noir has its street instead)
    let _ = cpu;
    let cpu = (num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
    // your heart, when you connect one; else the CPU: every beat is a spike that runs along the line
    let bpm = num(st, &["heart"]).filter(|v| *v > 20.0).map(|v| v as f32).unwrap_or(46.0 + cpu * 95.0);
    if world.beats.last().map(|b| (t - b) as f32 > 60.0 / bpm).unwrap_or(true) && !still_l {
        world.beats.push(t);
    }
    world.beats.retain(|b| t - b < 2.6);
    let gauss = |x: f32, sd: f32| (-(x * x) / (2.0 * sd * sd)).exp();
    let ecg = |x: f32| -0.12 * gauss(x + 0.075, 0.018) + gauss(x, 0.011) - 0.32 * gauss(x - 0.024, 0.012) + 0.18 * gauss(x - 0.12, 0.035);
    let beats: Vec<f32> = world.beats.iter().map(|b| ((t - b) / 2.4) as f32).collect();
    let mut wave = PathBuilder::stroke(px((u * 2.0).max(1.5)));
    for i in 0..=(if wall || mars { 0 } else { 320 }) {
        let fx = i as f32 / 320.0;
        let near = (-((fx - 0.5) * 3.2).powi(2)).exp();
        let spike: f32 = beats.iter().map(|pos| ecg((fx - pos) * 0.9) * (1.0 - pos).max(0.0)).sum();
        let y = oy + hz + (fx * 38.0 - tf * (1.0 + cpu * 3.0)).sin() * h * 0.006 * (0.3 + cpu) * (1.0 - near * 0.8) - spike * h * 0.03 * (0.6 + cpu * 0.7) * (1.0 - near * 0.7);
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
    if wet && !snow && polish {
        crate::look::rain_rings(win, &frame, pal, rain, still_l);
    }
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
    for p in list(st, &["sky"]).iter().take(if simple { 0 } else { 6 }) {
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
    if mars || monolith || pond {
        // a scene in code: its own world over Pulse's sky, reading what the hub gives a scene
        let hub_t = num(st, &["t"]).unwrap_or(0.0);
        if world.view.as_ref().map(|v| v.0 != hub_t).unwrap_or(true) {
            let sv = crate::worlds::view(st, world.tz);
            let ups = if pond { Vec::new() } else { crate::worlds::update(kind, &mut world.code, &sv) };
            for (k, words) in ups {
                if k == "chime" {
                    world.sounds.push("chime");
                }
                world.fx(&k, &words, false, t);
            }
            world.view = Some((hub_t, sv));
        }
        let view = world.view.take();
        if let Some((_, sv)) = &view {
            if mars {
                crate::worlds::mars(win, sv, &frame, &mut world.code, t, &mut world.labels, &mut world.hits);
            } else if pond {
                crate::worlds::pond(win, sv, pal, &frame, &mut world.code, t, &mut world.hits);
            } else {
                crate::worlds::monolith(win, sv, pal, &frame, &mut world.code, t, &mut world.labels, &mut world.hits);
            }
        }
        world.view = view;
    }
    crate::extra::over(win, st, pal, &frame, &mut world.more, t, &mut world.labels);
    for ask in std::mem::take(&mut world.more.asks) {
        match ask {
            "shooting" => world.fx("shooting", "", false, t),
            "thunder" => world.sounds.push("thunder"),
            "burst" => world.fx("fireworks", "", true, t),
            _ => {}
        }
    }
    world.bursts.retain(|b| t - b.0 < 2.6);
    if get(st, &["away"]).as_bool().unwrap_or(false) {
        rect(win, ox, oy, w, h, pal.bg, 0.45); // you are away: the city sleeps
    }
    marks
}
