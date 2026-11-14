//! The world behind the panels: the sky at its real hour, the stars, the Moon and the Sun where they are, your
//! clouds moving with your wind, a far city whose windows come on as it gets dark, rain as heavy as it really is,
//! and the planes over you. Everything is drawn on the GPU as quads and paths; nothing here asks the network.

use crate::hub::{get, list, num};
use crate::palette::{col, mix, scale, Pal, C};
use gpui::{linear_color_stop, linear_gradient, point, px, size, Bounds, PaintQuad, Pixels, Window};
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

/// What is made once and kept: the city, the stars, the rain.
pub struct World {
    pub city: Vec<Building>,
    pub last_t: f64,
    /// rockets on their way up: (when it left, its name, a Starship?)
    pub rockets: crate::rockets::Rockets,
    /// fireworks: (when, x, y, seed, small)
    pub bursts: Vec<(f64, f32, f32, u64, bool)>,
    /// the words of the last effect, and when
    pub say: Option<(f64, String)>,
    seed: u64,
    /// scene orbit: where each core's moon is on its ring
    pub rings: Vec<f32>,
    /// the rest of the world (storms, seasons, earthquakes, the garden...) and what it keeps
    pub more: crate::extra::Mem,
    /// the scenes in code (mars, monolith) and what they keep
    pub code: crate::worlds::Mem,
    /// pulse-native: the last level of every service on the status pages, to say when one goes down or comes back
    pub inet_prev: std::collections::HashMap<String, String>,
    /// pulse-native: the card of the agent you clicked, and the answer of its last button
    pub card: crate::agents::Card,
    /// pulse-native 1.1: what the new worlds keep
    pub w2: crate::worlds2::Mem,
    /// pulse-native: Pip, the creature of the wallpaper
    pub pet: crate::pet::Pet,
    pub card_note: std::sync::Arc<std::sync::Mutex<Option<String>>>,
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
    pub sky: crate::sky::Sky,
    pub floor: crate::floor::Floor,
    pub drones: crate::drones::Drones,
    pub focus_k: f32,
    /// the free band between the side columns, where the city stands (pulse.js band)
    pub band: (f32, f32),
    pub heavens: crate::heavens::Heavens,
    pub street: crate::street::Street,
    pub overhead: crate::heavens::Overhead,
    pub pushed: Vec<String>,
    /// something on screen moves fast (see the end of paint)
    pub lively: bool,
    /// the dust in the sphere's light
    pub motes: Vec<[f32; 4]>,
    pub weather: crate::weather::Weather,
    /// the first monitor: the banners and the tips fly there only
    pub primary: bool,
    pub git_flash: std::collections::HashMap<String, f32>,
    pub git_fail: std::collections::HashMap<String, f32>,
    /// where each repo's roof is (pulse.js gitPos): a push bursts there
    pub git_pos: Vec<(String, f32, f32)>,
    /// scene = "noir": what it keeps between frames
    pub noir: crate::modes::Noir,
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
    /// what never moves, painted once and kept as a picture (pulse.js paintStatic)
    pub back: crate::backdrop::Backdrop,
    back_img: Option<std::sync::Arc<gpui::RenderImage>>,
    /// the sphere of Pulse 1.8: its texture, its map, its picture
    pub ball: crate::sphere::Sphere,
    /// the sphere's picture and its reflection as the GPU has them now (dropped when a new frame replaces them)
    ball_img: Option<(std::sync::Arc<gpui::RenderImage>, std::sync::Arc<gpui::RenderImage>)>,
    pub ball_at: f64,
    /// where the face looks, smoothed (pulse.js gaze)
    pub gaze: (f32, f32),
    /// what main.rs knows and the sphere shows: the clock, the date, a text being typed, when you came back, when the
    /// car changed, a camera's and the door's frame
    pub hm: String,
    pub date_short: String,
    pub typing: bool,
    pub woke_at: f64,
    pub car_at: f64,
    pub cam: Option<(String, std::sync::Arc<Vec<u8>>)>,
    pub door: Option<(String, std::sync::Arc<Vec<u8>>)>,
    /// pulse.js alertEnv: 1 at an alert, fading
    pub alert_env: f32,
    /// the desktop of this monitor is empty (the face looks at you only then)
    pub desk_empty: bool,
}

impl World {
    /// An effect from the hub: the same kinds the web page plays.
    pub fn fx(&mut self, kind: &str, text: &str, small: bool, t: f64) {
        self.fx_raw(kind, text, small, t, &Value::Null)
    }

    /// The same, with everything the hub said about it (an earthquake's magnitude and direction).
    pub fn fx_raw(&mut self, kind: &str, text: &str, small: bool, t: f64, raw: &Value) {
        crate::extra::event(&mut self.more, kind, raw, t, self.last.as_ref());
        if matches!(kind, "commit" | "push" | "deploy") {
            self.w2.events.push((kind.to_string(), text.to_string(), t));
            if self.w2.events.len() > 16 {
                self.w2.events.remove(0);
            }
        }
        if kind == "commit" || kind == "push" {
            // pulse.js: the repo's tower flashes and a new floor comes down
            let name = text.split(" · ").next().unwrap_or("").to_string();
            if kind == "push" {
                self.pushed.push(name.clone()); // pulse.js: sparks over that repo's roof
            }
            self.git_flash.insert(name, t as f32);
        }
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        match kind {
            "launch" => {
                let name = text.split(" · liftoff").next().unwrap_or(text).to_string();
                let big = name.to_lowercase().contains("starship");
                self.rockets.launch(t as f32, name, big);
            }
            "lightning" => {}
            "shooting" => self.heavens.fx("shooting"),
            "banner" | "tip" => self.street.fx(kind, text, t, &[]),
            "chime" | "text" | "say" | "flare" | "quake" | "tsunami" | "build" | "doorbell" | "train" | "summary" | "starlink" | "car" | "push" | "commit" | "iss" => {
                if kind == "iss" {
                    self.heavens.fx("iss");
                }
            }
            _ => self.overhead.fireworks(t, small, kind == "burst"),
        }
        if !text.is_empty() && kind != "banner" && kind != "tip" {
            self.say = Some((t, text.chars().take(60).collect()));
        }
    }
}

impl World {
    pub fn new() -> Self {
        let mut r = Rand::new(1808);
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
        World { city, last_t: 0.0, rockets: Default::default(), bursts: Vec::new(), say: None, seed: 42, rings: Vec::new(), more: Default::default(), code: Default::default(), inet_prev: Default::default(), card: Default::default(), w2: Default::default(), pet: Default::default(), card_note: Default::default(), labels: Vec::new(), hits: Vec::new(), centre: None, pointer: None, wall: false, has_cam: false, simple: false, last: None, beats: Vec::new(), heights: Vec::new(), audio: Vec::new(), tz: 0, prog: "face", alert_at: -1e9, view: None, sounds: Vec::new(), back: Default::default(), back_img: None, ball: Default::default(), ball_img: None, ball_at: -1e9, gaze: (0.0, 0.0), hm: String::new(), date_short: String::new(), typing: false, woke_at: -1e9, car_at: -1e9, cam: None, door: None, alert_env: 0.0, desk_empty: true, sky: Default::default(), floor: Default::default(), drones: Default::default(), focus_k: 0.0, band: (0.0, 0.0), heavens: Default::default(), street: Default::default(), overhead: Default::default(), pushed: Vec::new(), lively: true, motes: Vec::new(), weather: Default::default(), primary: true, git_flash: Default::default(), git_fail: Default::default(), git_pos: Vec::new(), noir: Default::default() }
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

/// A canvas radial gradient of one colour (createRadialGradient(cx, cy, r0, cx, cy, r1) with these stops of alpha),
/// drawn as discs one inside the other, each adding what is missing to reach the alpha of its ring. `sy` squashes
/// it (a pool of light on the floor).
#[allow(clippy::too_many_arguments)]
pub(crate) fn radial(win: &mut Window, cx: f32, cy: f32, r0: f32, r1: f32, sy: f32, c: C, stops: &[(f32, f32)]) {
    crate::draw::soft(win, cx, cy, r0, r1, sy, c, stops);
}

pub(crate) fn gradient(win: &mut Window, x: f32, y: f32, w: f32, h: f32, top: C, ta: f32, bottom: C, ba: f32) {
    let bg = linear_gradient(180.0, linear_color_stop(col(top, ta), 0.0), linear_color_stop(col(bottom, ba), 1.0));
    win.paint_quad(PaintQuad::new(Bounds::new(point(px(x), px(y)), size(px(w), px(h)))).background(bg));
}

/// What the face on the sphere feels: your most urgent agent decides, as on the web page (faceMood).
/// pulse.js faceMood(): what the face in the core shows, or no face.
fn core_mood(st: &Value, world: &World, t: f64, w: f32, h: f32) -> Option<&'static str> {
    let listen = crate::hub::text(st, &["listen", "state"]);
    if listen == "listening" {
        return Some("listen");
    }
    if listen == "thinking" {
        return Some("think");
    }
    if get(st, &["speaking"]).as_bool().unwrap_or(false) {
        return Some("talk");
    }
    let face = get(st, &["ui", "face"]).as_str().unwrap_or("agents");
    if get(st, &["away"]).as_bool().unwrap_or(false) && face != "off" {
        return Some("sleep");
    }
    if face != "off" && t - world.woke_at < 2.6 {
        return Some("wake");
    }
    if world.typing {
        return Some("talk");
    }
    if list(st, &["agents"]).is_empty() {
        let watching = matches!(world.pointer, Some((x, y, at)) if t - at < 6.0 && world.desk_empty && x >= 0.0 && y >= 0.0 && x <= w && y <= h);
        return if face == "always" || watching { Some("idle") } else { None };
    }
    Some(face_mood(st))
}

/// The share of a focus session still to go (pulse.js: the ring around the core).
fn focus_left(st: &Value) -> Option<f32> {
    let f = get(st, &["focus"]);
    let (start, until) = (num(f, &["start"])?, num(f, &["until"])?);
    let now = num(st, &["t"]).unwrap_or(start);
    Some((((until - now).max(0.0)) / (until - start).max(1.0)) as f32)
}

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




/// A plane on the screen: where, and what to write beside it (main.rs writes the text).
pub struct PlaneMark {
    pub x: f32,
    pub y: f32,
    pub c: C,
    pub title: String,
    pub line: String,
}

/// Paints the world for the time `t` (seconds) and returns the planes' places for their tags.
pub fn paint(bounds: Bounds<Pixels>, world: &mut World, st: &Value, pal: &Pal, t: f64, kind: &str, bare: bool, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, ui_boxes: &[[f32; 4]]) -> Vec<PlaneMark> {
    let mut lap = crate::perf::Lap::new();
    let paths0 = crate::draw::PATHS.with(|n| n.replace(0));
    if crate::perf::on() {
        crate::perf::record("paths×1ms", paths0 as f64 / 1000.0); // shapes the last frame built, in thousandths
    }
    let sphere = kind == "sphere";
    let (orbit, calm, hacker, noir) = (kind == "orbit", kind == "calm", kind == "hacker", kind == "noir");
    let (mars, monolith, pond) = (kind == "mars", kind == "monolith", kind == "pond");
    let own = crate::worlds2::is_world(kind); // pulse-native 1.1: station, abyss, racks, moonbase draw all of it
    let wall = world.wall;
    let simple = world.simple;
    let bare = bare || orbit || mars || monolith || pond || wall; // (noir keeps pulse.js's git, cars, robots: its own centre is the board) // no near towers, no dome, no city life: the scene draws its own centre
    let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let (ox, oy) = (f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let (qx, qy) = crate::extra::shake(&mut world.more, t, h); // an earthquake shakes the whole picture
    let (ox, oy) = (ox + qx, oy + qy);
    world.labels.clear();
    world.hits.clear();
    crate::agents::clear_spots();
    crate::agents::set_boxes(ui_boxes);
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
    let code = num(st, &["weather", "code"]).unwrap_or(0.0) as i32;
    let wind = num(st, &["weather", "wind"]).unwrap_or(0.0) as f32;
    let wind_dir = num(st, &["weather", "windDir"]).unwrap_or(270.0) as f32;

    let marks = Vec::new();
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
    // what never moves comes from one picture, painted as pulse.js paints it and only when something changes
    let use_back = !wall && !mars && !pond && !own && kind != "city"; // (noir too: its megacity is in the picture)
    if use_back {
        let sf = win.scale_factor().clamp(1.0, 3.0); // the screen's own pixels, also at a scale over 2
        let rem = crate::ui::rem(w, h);
        let edge = (27.2 * rem + 30.0) * sf;
        let band = if (w * sf - 2.0 * edge) > w * sf * 0.25 { (edge, w * sf - edge) } else { (w * sf * 0.08, w * sf * 0.92) };
        let local_minutes = ((num(st, &["t"]).unwrap_or(0.0) as i64 + world.tz).rem_euclid(86400)) / 60;
        world.back.update(&crate::backdrop::Scene { st, pal, kind, w: (w * sf).round() as u32, h: (h * sf).round() as u32, band, local_minutes, polished: crate::look::on(st), world_off: monolith });
        if world.back.fresh || world.back_img.is_none() {
            let b = &world.back;
            if let Some(im) = image::RgbaImage::from_raw(b.w, b.h, b.pixels.clone()) {
                if let Some(old) = world.back_img.replace(std::sync::Arc::new(gpui::RenderImage::new(vec![image::Frame::new(im)]))) {
                    let _ = win.drop_image(old);
                }
            }
        }
        if let Some(im) = &world.back_img {
            let at = Bounds::new(point(px(ox), px(oy)), size(px(w), px(h)));
            let _ = win.paint_image(at, at, gpui::Corners::default(), 0.0, im.clone(), 0, false);
        }
        // the red lights on top of your city's landmarks
        if tf.rem_euclid(1.7) < 0.22 && !monolith {
            for tp in &world.back.tops {
                rect(win, ox + tp.0 / sf - h / 700.0, oy + tp.1 / sf - h / 300.0, h / 350.0, h / 350.0, pal.crit, 0.95);
            }
        }
    } else if !wall {
        gradient(win, ox, oy, w, hz * 0.7 + 1.0, top, 1.0, midc, 1.0);
        gradient(win, ox, oy + hz * 0.7, w, hz * 0.3 + 1.0, midc, 1.0, low, 1.0);
    }

    lap.mark("backdrop");
    // ---- pulse.js draw(): the faint stars, the real Sun (the rays and the real stars and the Moon follow)
    let sky_on = !wall;
    if !own {
        world.heavens.back(win, cx, font, st, pal, ox, oy, w, h, tf, wall);
    }

    lap.mark("heavens-back");
    let frame = crate::life::Frame { w, h, ox, oy, hz, u, t: tf, sky: [wn, wt, wd] };
    world.last = Some(frame);
    if own {
        // one of the new worlds: it draws its whole picture; then what goes over every scene
        crate::worlds2::paint(kind, win, st, pal, &frame, &mut world.w2, t, world.tz, &mut world.labels, &mut world.hits);
        for (k, words) in crate::status::changes(&mut world.inet_prev, st) {
            world.fx(&k, &words, false, t);
        }
        world.overhead.draw_fireworks(win, pal, ox, oy, w, h, t, (ox + w * 0.2, ox + w * 0.8), dt * 20.0);
        world.band = (ox + w * 0.22, ox + w * 0.78);
        if !simple {
            let hub_now = num(st, &["t"]).unwrap_or(0.0);
            let hour = ((hub_now as i64 + world.tz).rem_euclid(86400)) / 3600;
            let (pointer, band) = (world.pointer, world.band);
            world.pet.paint(win, cx, font, st, pal, ox, oy, w, h, t, band, hour, pointer, kind != "racks", get(st, &["ui", "motion"]).as_str() == Some("reduced"));
        }
        if let Some(n) = world.card_note.lock().ok().and_then(|mut g| g.take()) {
            world.card.note = Some((n, t));
        }
        world.card.paint(win, cx, font, st, pal, ox, oy, w, h, t);
        let falling = false;
        world.lively = !world.audio.is_empty() || falling || world.overhead.busy() || world.card.is_open() || world.pet.busy() || t - world.alert_at < 8.0;
        crate::draw::vignette(win, ox, oy, w, h);
        return Vec::new();
    }
    let still_l = get(st, &["ui", "motion"]).as_str() == Some("reduced");
    let polish = crate::look::on(st) && !wall && !noir && !mars;
    let lt = if polish { crate::look::light(st, pal, &frame) } else { None };
    if let (Some(l), true) = (&lt, sky_on) {
        crate::look::rays(win, l, &frame, still_l);
    }
    if sky_on {
        world.heavens.front(win, cx, font, st, pal, ox, oy, w, h, tf);
        world.overhead.starlink(win, cx, font, st, pal, ox, oy, w, h, tf, dt * 20.0, ui_boxes, still_l);
    }
    if sky_on {
        crate::extra::sky(win, st, pal, &frame, &mut world.more, t, &mut world.labels);
    }
    let vx = -(wind_dir + 180.0).to_radians().sin() * (wind / 60.0); // the wind comes from windDir; east is on the left
    lap.mark("rays-front-starlink");
    // ---- pulse.js drawClouds(): clouds with a body, lit from the side of the Sun or the Moon, drifting with your wind
    if sky_on && !wall {
        let light = lt.as_ref().map(|l| (l.c, l.sun, l.x - ox));
        world.sky.clouds(win, st, pal, ox, oy, w, h, light, dt * 20.0);
    }
    if sky_on {
        // aurora = network traffic, then the shooting stars (meteor showers from their radiant)
        crate::heavens::Heavens::aurora(win, st, pal, ox, oy, w, h, tf, &world.audio);
        world.heavens.shooting(win, st, pal, ox, oy, w, h, t, dt * 20.0);
    }

    lap.mark("clouds-aurora");
    // ---- the far city: three rows, paler with the distance; windows come on as the Sun goes down
    let lit = 0.07 + 0.50 * wn + 0.28 * wt;
    let warm = mix(pal.warn, pal.fg, 0.35);
    for b in world.city.iter().filter(|_| !noir && !wall && !mars && !use_back) {
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
        if !use_back {
            gradient(win, ox, oy + hz, w, h - hz, mix(bg, a2, 0.2), 1.0, mix(bg, black, 0.5), 1.0);
            gradient(win, ox, oy + hz, w, h * 0.06, mix(pal.accent, a2, 0.5), 0.10 + 0.12 * (wn + wt * 0.6), bg, 0.0);
        }
        if polish {
            // 1.8: the air over the far city, mist between its rows, the wet floor that mirrors it
            if !use_back {
                crate::look::air(win, pal, &frame, low);
            }
            if !orbit && !use_back {
                crate::look::mirror(win, &world.city, pal, &frame, low);
            }
        }
    }
    lap.mark("ground");
    // ---- the near city: a tower for each CPU core either side of the centre
    let cpu = (num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
    let hide = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
    let real_city = !hide && crate::cities::city(&crate::hub::text(st, &["ui", "skyline"])).is_some();
    let r_core = h * if orbit { 0.15 } else if sphere { 0.165 } else { 0.135 };
    if !bare && !orbit {
        // pulse.js drawGit(): your repos beside the core (not in calm), then the towers of the cores in front of them
        world.git_pos = if calm { Vec::new() } else { crate::look::git(win, cx, font, st, pal, &frame, r_core, &world.git_flash, &mut world.git_fail, real_city, still_l, hide) };
        for name in std::mem::take(&mut world.pushed) {
            if let Some(p) = world.git_pos.iter().find(|p| p.0 == name) {
                world.overhead.burst_at(p.1, p.2, pal.accent, pal.fg, 70, h * 0.006);
            }
        }
        let inner = r_core * if sphere { 1.08 } else { 1.18 };
        let want = if get(st, &["focus"]).is_object() { 1.0 } else { 0.0 };
        world.focus_k += (want - world.focus_k) * if still_l { 1.0 } else { (0.02 * dt * 20.0).min(1.0) };
        crate::look::towers(win, st, pal, &frame, lt, &mut world.heights, &world.audio, inner, polish, world.focus_k, real_city, noir, still_l);
    }
    lap.mark("git-towers");
    let (mut rr, mut dcx, mut dcy) = (0.0f32, ox + w / 2.0, oy + hz);
    if !bare && !noir {
        // ---- the centre: the dome of the skyline, or the whole sphere of Pulse turning over the horizon
        // pulse.js: R = H * (sphere ? .165 : .135), the sphere sits just over the horizon (cy = hz - R * .97)
        rr = if sphere { h * 0.165 } else { h * 0.135 };
        dcy = if sphere { oy + hz - rr * 0.97 } else { oy + hz };
        dcx = ox + w / 2.0;
        if sphere {
            // the eyes: towards the pointer while it moves, then slowly back to the middle (pulse.js draw())
            let eyes = get(st, &["ui", "eyes"]).as_bool().unwrap_or(true);
            let (mut gx, mut gy) = (0.0f32, 0.0f32);
            if let (true, Some((px0, py0, at))) = (eyes, world.pointer) {
                if t - at < 15.0 {
                    gx = (px0 - w / 2.0) / (h * 0.45);
                    gy = (py0 - (hz - h * 0.16)) / (h * 0.45);
                    let gl = (gx * gx + gy * gy).sqrt();
                    if gl > 1.0 {
                        gx /= gl;
                        gy /= gl;
                    }
                }
            }
            world.gaze.0 += (gx - world.gaze.0) * 0.18;
            world.gaze.1 += (gy - world.gaze.1) * 0.18;
            let face_cfg = get(st, &["ui", "face"]).as_str().unwrap_or("agents").to_string();
            let watching = matches!(world.pointer, Some((px0, py0, at)) if eyes && face_cfg != "off" && world.desk_empty && t - at < 6.0 && px0 >= 0.0 && py0 >= 0.0 && px0 <= w && py0 <= h);
            world.alert_env *= 0.975;
            let music = !world.audio.is_empty();
            let bass = if music { world.audio.iter().take(4).sum::<f32>() / 4.0 / 100.0 } else { 0.0 };
            let hm = world.hm.clone();
            let date = world.date_short.clone();
            let (cam, door) = (world.cam.clone(), world.door.clone());
            let audio = world.audio.clone();
            let feed = crate::sphere::Feed {
                st, pal, t, hm: &hm, date: &date, gaze: world.gaze, watching, woke: t - world.woke_at, typing: world.typing, alert_env: world.alert_env,
                audio: &audio, bass, car_age: t - world.car_at,
                wind_k: (if get(st, &["weather", "windUnit"]).as_str() == Some("mph") { 1.609 } else { 1.0 } * (num(st, &["weather", "wind"]).unwrap_or(0.0) as f32).max(num(st, &["weather", "gust"]).unwrap_or(0.0) as f32 * 0.8) / 60.0).min(1.0),
                cam: cam.as_ref().map(|c| (c.0.as_str(), c.1.as_slice())), door: door.as_ref().map(|c| (c.0.as_str(), c.1.as_slice())),
            };
            // the sphere's picture is made in the screen's own pixels (a HiDPI screen gets two or three times as many)
            let px_d = ((rr * win.scale_factor().clamp(1.0, 3.0) * 2.0).round() as usize).max(16);
            // a new picture every frame: the LEDs move as smoothly as the rest (a picture every other frame made them
            // step out of time with the scene)
            let fresh = world.ball_img.is_none() || world.ball.d != px_d || t != world.ball_at;
            if fresh {
                world.ball_at = t;
                crate::perf::time("sphere", || world.ball.frame(&feed, rr * win.scale_factor().clamp(1.0, 3.0)));
            }
            world.prog = world.ball.prog;
            let avg = { let a = world.ball.avg; [a[0] / 255.0, a[1] / 255.0, a[2] / 255.0] };
            let hzy = oy + hz;
            if get(st, &["ui", "sphere"]).as_str() == Some("led") {
                // the LED look: a halo in the second accent, the ball, a thin outline
                radial(win, dcx, dcy, rr * 0.8, rr * 1.9, 1.0, if world.alert_env > 0.25 { pal.crit } else { a2 }, &[(0.0, 0.28 + 0.25 * bass), (1.0, 0.0)]);
            } else {
                // light on the plaza and the city, in the colour on screen
                radial(win, dcx, hzy, rr * 0.3, rr * 3.2, 1.0, avg, &[(0.0, 0.38), (1.0, 0.0)]);
                // the glow it throws into the wet night air, wide and soft
                let hk = 0.34 + 0.18 * bass;
                radial(win, dcx, dcy, rr * 0.75, rr * 2.3, 1.0, avg, &[(0.0, hk), (0.35, hk * 0.45), (0.7, hk * 0.12), (1.0, 0.0)]);
                // where it touches the plaza: a pool of its own light
                radial(win, dcx, hzy, 0.0, rr * 1.1, 0.16, avg, &[(0.0, 0.5), (1.0, 0.0)]);
                if polish {
                    let beat_env = world.beats.last().map(|b| (-((t - b) as f32) / 0.16).exp()).unwrap_or(0.0);
                    let wet = if matches!(crate::ui::wx_kind(code), "rain" | "storm" | "fog") { 1.0 } else { 0.0 };
                    crate::core::aura(win, dcx, dcy, rr, avg, tf, bass, beat_env, wet, &mut world.motes, dt);
                }
            }
            // the sphere's picture, and its reflection on the wet floor (the same picture upside down, at 20%)
            let d = world.ball.d;
            if fresh && d > 0 && world.ball.out.len() == d * d * 4 && world.ball.refl.len() == d * d * 4 {
                let frame = |bytes: Vec<u8>| image::RgbaImage::from_raw(d as u32, d as u32, bytes).map(|im| std::sync::Arc::new(gpui::RenderImage::new(vec![image::Frame::new(im)])));
                // the buffers go to the GPU as they are (no copy); the sphere makes new ones next frame
                let (out, refl) = (std::mem::take(&mut world.ball.out), std::mem::take(&mut world.ball.refl));
                if let (Some(a), Some(b)) = (frame(out), frame(refl)) {
                    if let Some((oa, ob)) = world.ball_img.replace((a, b)) {
                        let _ = win.drop_image(oa);
                        let _ = win.drop_image(ob);
                    }
                }
            }
            if let Some((a, b)) = world.ball_img.clone() {
                {
                    let at = |x: f32, y: f32, ww: f32, hh: f32| Bounds::new(point(px(x), px(y)), size(px(ww), px(hh)));
                    let _ = win.paint_image(at(dcx - rr, dcy - rr, rr * 2.0, rr * 2.0), at(dcx - rr, dcy - rr, rr * 2.0, rr * 2.0), gpui::Corners::default(), 0.0, a, 0, false);
                    let ry = 2.0 * hzy + rr * 0.08 - (dcy + rr);
                    let clip = at(dcx - rr * 1.2, hzy, rr * 2.4, rr * 1.3);
                    win.with_content_mask(Some(gpui::ContentMask { bounds: clip, ..Default::default() }), |win| {
                        let _ = win.paint_image(at(dcx - rr, ry, rr * 2.0, rr * 2.0), at(dcx - rr, ry, rr * 2.0, rr * 2.0), gpui::Corners::default(), 0.0, b, 0, false);
                    });
                }
            }
            if get(st, &["ui", "sphere"]).as_str() == Some("led") {
                crate::draw::arc(win, dcx, dcy, rr, rr, 0.0, std::f32::consts::TAU, (h / 700.0).max(1.0), pal.accent, 0.3);
            } else {
                // a thin rim of light on the outline, blending the curve into the halo
                crate::draw::arc(win, dcx, dcy, rr * 1.0, rr * 1.0, 0.0, std::f32::consts::TAU, rr * 0.05, mix(avg, pal.fg, 0.3), 0.16);
                crate::draw::arc(win, dcx, dcy, rr * 0.96, rr * 0.96, 0.0, std::f32::consts::TAU, rr * 0.04, avg, 0.08);
            }
        } else {
            // pulse.js draw(): the wireframe core of skyline, calm and hacker, and the agent's face in it
            let mood = core_mood(st, world, t, w, h);
            let cores: Vec<f32> = list(st, &["sys", "cores"]).iter().filter_map(|c| c.as_f64().map(|v| v as f32)).collect();
            let beat_env = world.beats.last().map(|b| (-((t - b) as f32) / 0.16).exp()).unwrap_or(0.0);
            let focus_left = focus_left(st);
            let (gaze, woke) = (world.gaze, (t - world.woke_at) as f32);
            crate::core::paint(win, pal, kind, ox, oy, w, h, tf, dt, cpu * 100.0, &cores, beat_env, Some((pal.crit, world.alert_env)), mood, gaze, woke, focus_left, &mut world.rings);
        }
        rr = if sphere { rr * 1.05 } else { rr };
        // (the drones below fly over its top either way: dcy - rr is the top of both)
    }
    lap.mark("sphere-all");
    // ---- pulse.js drawDrones(): your AI agents, a quadcopter each over the city between the panels (in every scene of
    // Pulse's own, orbit and noir too; not over a scene in code or your own wallpaper)
    if !simple && !(mars || monolith || pond || wall) && !(bare && !orbit && !noir) {
        let r = crate::ui::rem(w, h);
        let mut band = (27.2 * r + 30.0, w - 27.2 * r - 30.0);
        if band.1 - band.0 <= w * 0.25 {
            band = (w * 0.08, w * 0.92);
        }
        world.drones.paint(win, st, pal, ox, oy, h, tf, band, world.focus_k, still_l);
        world.band = band;
        for (x, y, _) in std::mem::take(&mut world.drones.landed) {
            world.seed = world.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            world.bursts.push((t, x / w, y / h, world.seed >> 20, true));
        }
    }
    lap.mark("drones");
    // pulse.js draw(): the mist over the horizon, then the floor's grid running toward you, then the light on it
    if polish && !noir && !orbit && !wall {
        let a = world.back.air;
        let fog = if code == 45 || code == 48 { 0.5 } else { 0.0 };
        crate::look::mist(win, &frame, [a[0] / 255.0, a[1] / 255.0, a[2] / 255.0], 0.28 + 0.4 * wn + 0.45 * wt + fog, still_l);
    }
    if !wall {
        let bass = if world.audio.is_empty() { 0.0 } else { world.audio.iter().take(4).sum::<f32>() / 4.0 / 100.0 };
        let cpu_pc = num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32;
        crate::look::grid(win, pal, &frame, cpu_pc, bass, !(calm || still_l || get(st, &["paused"]).as_bool().unwrap_or(false)), noir);
    }
    if let (Some(l), false) = (&lt, wall || noir) {
        crate::look::light_path(win, l, &frame, still_l); // its light on the floor
    }

    lap.mark("mist-grid-light");
    if !bare && !simple {
        crate::extra::ground(win, st, pal, &frame, &mut world.more, t, &mut world.labels, kind);
        crate::extra::net(win, st, pal, &frame, &mut world.more, t, &mut world.labels);
        lap.mark("fs-ground-net");
        // pulse.js drawCars(): your network's traffic, lights running in and out along the grid
        if !simple {
            let bass = if world.audio.is_empty() { 0.0 } else { world.audio.iter().take(4).sum::<f32>() / 4.0 / 100.0 };
            world.floor.river(win, cx, font, st, pal, ox, oy, w, h, dt * 20.0, still_l);
            lap.mark("fs-river");
            world.floor.cars(win, cx, font, st, pal, ox, oy, w, h, tf, bass, kind);
            lap.mark("fs-cars");
            crate::floor::car(win, cx, font, st, pal, ox, oy, h, tf, world.band, (t - world.car_at) as f32, kind);
            lap.mark("fs-car");
        }
        if !simple && !calm && !orbit {
            // pulse.js: the datacenter (not in noir), the trains, the robots, the candles
            let share = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
            let band = world.band;
            if !noir { world.street.datacenter(win, cx, font, st, pal, ox, oy, w, h, tf, t, band, wn.max(wt), still_l); }
            lap.mark("fs-dc");
            for (x, y, words) in std::mem::take(&mut world.street.zaps) {
                crate::extra::event(&mut world.more, "lightning", &serde_json::json!({ "x": x - ox, "y": y - oy }), t, world.last.as_ref());
                world.say = Some((t, words));
                world.sounds.push("thunder");
            }
            for (x, y) in std::mem::take(&mut world.street.sparks) {
                world.seed = world.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                world.bursts.push((t, (x - ox) / w, (y - oy) / h, world.seed >> 20, true));
            }
            world.street.trains(win, cx, font, st, pal, ox, oy, h, tf, band, share);
            lap.mark("fs-trains");
            world.street.robots(win, cx, font, st, pal, ox, oy, h, tf, band, dt * 20.0, share);
            lap.mark("fs-robots");
            world.street.candles(win, cx, font, st, pal, ox, oy, w, h, tf, still_l, share);
        }
    }
    if !bare {
        world.centre = Some((dcx - ox, dcy - oy, rr));
    }
    if orbit {
        let mood = core_mood(st, world, t, w, h);
        let cores: Vec<f32> = list(st, &["sys", "cores"]).iter().filter_map(|c| c.as_f64().map(|v| v as f32)).collect();
        let beat_env = world.beats.last().map(|b| (-((t - b) as f32) / 0.16).exp()).unwrap_or(0.0);
        let cpu_o = num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32;
        crate::core::paint(win, pal, kind, ox, oy, w, h, tf, dt, cpu_o, &cores, beat_env, Some((pal.crit, world.alert_env)), mood, world.gaze, (t - world.woke_at) as f32, focus_left(st), &mut world.rings);
    }
    if hacker {
        crate::core::radar(win, cx, font, st, pal, ox, oy, w, h, tf, still_l, hide);
    }
    lap.mark("floor-street");
    // the line of the horizon: the machine's pulse, livelier with the CPU (noir has its street instead)
    let _ = cpu;
    let cpu = (num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
    // your heart, when you connect one; else the CPU: every beat is a spike that runs along the line
    let bpm = num(st, &["heart"]).filter(|v| *v > 20.0).map(|v| v as f32).unwrap_or(46.0 + cpu * 95.0);
    if world.beats.last().map(|b| (t - b) as f32 > 60.0 / bpm).unwrap_or(true) && !still_l {
        world.beats.push(t);
    }
    world.beats.retain(|b| t - b < 2.6);
    world.beats.retain(|b| t - b < 1.8);
    // pulse.js draw(): the horizon is your pulse: every beat leaves the core and runs to both edges, and the music's
    // bars ride on it; three strokes, wide and faint to thin and bright, in the accent (the alert's colour in an alert)
    let gauss = |x: f32, sd: f32| (-(x * x) / (2.0 * sd * sd)).exp();
    let ecg = |x: f32| -0.12 * gauss(x + 0.075, 0.018) + gauss(x, 0.011) - 0.32 * gauss(x - 0.024, 0.012) + 0.18 * gauss(x - 0.12, 0.035);
    if !(wall || mars) {
        let amp = 0.6 + cpu * 100.0 / 140.0;
        let ages: Vec<f32> = world.beats.iter().map(|b| (t - b) as f32).collect();
        let audio = world.audio.clone();
        let mut ys = Vec::with_capacity((w / 3.0) as usize + 2);
        let mut x = 0.0f32;
        while x <= w {
            let dx = (x - w / 2.0).abs() / w;
            let mut yv = 0.0;
            for age in &ages {
                let fade = (1.0 - age / 1.8).max(0.0);
                let u = dx - age * 0.32;
                if fade > 0.0 && u > -0.2 && u < 0.3 {
                    // (further away a beat is flat: e^-(0.2²/2·0.035²) is nothing)
                    yv += ecg(u) * amp * fade;
                }
            }
            if !audio.is_empty() {
                let bi = ((dx * 2.0 * audio.len() as f32) as usize).min(audio.len() - 1);
                yv += audio[bi] / 100.0 * 0.5 * (1.0 - dx);
            }
            ys.push((ox + x, oy + hz - 1.0 - yv * h * 0.06));
            x += 3.0;
        }
        let lc = if world.alert_env > 0.05 { mix(pal.accent, pal.crit, (world.alert_env * 1.5).min(1.0)) } else { pal.accent };
        for (lw, a) in [(7.0f32, 0.06f32), (3.5, 0.16), (1.6, 0.95)] {
            crate::draw::line(win, &ys, lw * h / 1080.0 * 1.6, lc, a);
        }
    }

    lap.mark("horizon");
    // ---- the real weather: rain with depth and its rings, snow, hail, fog (pulse.js draw())
    {
        world.weather.paint(win, st, pal, ox, oy, w, h, tf, t, dt * 20.0, polish, still_l);
    }
    lap.mark("weather");
    // ---- pulse.js drawISS(), drawFireworks(): the station, then the fireworks (and every burst of sparks)
    if sky_on {
        world.overhead.iss(win, cx, font, st, pal, ox, oy, w, h, dt * 20.0);
    }
    for b in std::mem::take(&mut world.bursts) {
        // a burst asked by the scene (a drone landing, a service back up): sparks there
        world.overhead.burst_at(ox + b.1 * w, oy + b.2 * h, pal.ok, pal.fg, if b.4 { 40 } else { 70 }, h * if b.4 { 0.004 } else { 0.006 });
    }
    world.overhead.draw_fireworks(win, pal, ox, oy, w, h, t, world.band, dt * 20.0);
    // ---- pulse.js drawLiftoffsFine(): a real launch
    let share_l = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
    world.rockets.paint(win, cx, font, pal, ox, oy, w, h, tf, world.band, [wn, wt, wd], still_l, share_l);

    if noir && !wall {
        // pulse.js drawNoirCity(): the megacity's lights, its big screen, its signs, the cars in the air, the rain
        let sf = win.scale_factor().clamp(1.0, 3.0);
        let fresh_board = if world.noir.board.is_none() || t != world.ball_at {
            world.ball_at = t;
            let (cam, door) = (world.cam.clone(), world.door.clone());
            let (hm, date, audio) = (world.hm.clone(), world.date_short.clone(), world.audio.clone());
            let feed = crate::sphere::Feed {
                st, pal, t, hm: &hm, date: &date, gaze: world.gaze, watching: false, woke: t - world.woke_at, typing: world.typing, alert_env: world.alert_env,
                audio: &audio, bass: 0.0, car_age: t - world.car_at, wind_k: 0.0,
                cam: cam.as_ref().map(|c| (c.0.as_str(), c.1.as_slice())), door: door.as_ref().map(|c| (c.0.as_str(), c.1.as_slice())),
            };
            Some(world.ball.board(&feed))
        } else {
            None
        };
        world.prog = world.ball.prog;
        crate::modes::noir_city(win, cx, font, pal, &frame, &world.back, sf, fresh_board, &mut world.noir, still_l, dt * 20.0);
    }
    lap.mark("iss-fw-rocket");
    // ---- pulse.js drawPlanes(): the planes really over you, gliding where they are going, each with its tag (not on
    // Mars: its own sky covers them, and orbiters and the scout fly there instead)
    if !simple {
        let frames = dt * 20.0;
        if !mars { world.sky.planes(win, cx, font, st, pal, ox, oy, w, h, tf, frames, ui_boxes, polish); }
        // drawBanners(), drawBalloons()
        let (primary, desk) = (world.primary, world.desk_empty);
        world.street.banners(win, cx, font, st, pal, ox, oy, w, h, t, primary, desk);
        world.street.balloons(win, cx, font, st, pal, ox, oy, w, h, t, primary, desk, vx, still_l);
    }
    lap.mark("planes-banners");
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
                if !simple {
                    // pulse-native: the flights over you, flying over Mars as saucers (same tags)
                    world.sky.ufo = true;
                    world.sky.planes(win, cx, font, st, pal, ox, oy, w, h, tf, dt * 20.0, ui_boxes, polish);
                    world.sky.ufo = false;
                }
            } else if pond {
                crate::worlds::pond(win, sv, pal, &frame, &mut world.code, t, &mut world.hits);
            } else {
                crate::worlds::monolith(win, sv, pal, &frame, &mut world.code, t, &mut world.labels, &mut world.hits);
            }
        }
        world.view = view;
    }
    // pulse-native: a service on the status pages went down or came back (the datacenter's hall strikes too)
    for (k, words) in crate::status::changes(&mut world.inet_prev, st) {
        world.fx(&k, &words, false, t);
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
    lap.mark("code-over");
    // what moves fast enough to need every frame: the music, falling rain or snow, fireworks, a launch, a shooting
    // star, the pointer the face follows, a text typing. Without any of it the scene moves slowly and 12 frames a
    // second are enough (main.rs)
    let falling = matches!(crate::ui::wx_kind(code), "rain" | "storm" | "snow");
    let pointing = world.pointer.map(|p| t - p.2 < 3.0).unwrap_or(false);
    world.lively = !world.audio.is_empty() || falling || world.overhead.busy() || world.rockets.busy() || world.heavens.busy() || pointing || world.typing || t - world.alert_at < 8.0;
    if !wall && !simple {
        // pulse-native: Pip walks on the floor
        let hub_now = num(st, &["t"]).unwrap_or(0.0);
        let hour = ((hub_now as i64 + world.tz).rem_euclid(86400)) / 3600;
        let (pointer, band) = (world.pointer, world.band);
        world.pet.paint(win, cx, font, st, pal, ox, oy, w, h, t, band, hour, pointer, mars, still_l);
        if world.pet.busy() {
            world.lively = true;
        }
    }
    crate::draw::vignette(win, ox, oy, w, h); // pulse.css #vig: the edges of the picture a little darker
    // the drones' tags, over the scene (on the web page they are HTML over the canvas); hidden in privacy
    if !(get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false)) {
        world.drones.tags(win, cx, font, pal, crate::ui::rem(w, h));
    }
    // pulse-native: the card of the agent you clicked, over everything
    if let Some(n) = world.card_note.lock().ok().and_then(|mut g| g.take()) {
        world.card.note = Some((n, t));
    }
    world.card.paint(win, cx, font, st, pal, ox, oy, w, h, t);
    if world.card.is_open() {
        world.lively = true;
    }
    if get(st, &["away"]).as_bool().unwrap_or(false) {
        rect(win, ox, oy, w, h, pal.bg, 0.45); // you are away: the city sleeps
    }
    marks
}
