//! The sphere of Pulse 1.8, translated from pulse.js: a texture of 192×96 units (painted on 384×192 pixels) is drawn at
//! every frame with what the sphere shows (its face, the clock, the weather, the CPU, the news, the music's wave, a
//! countdown, a focus, the next meeting, the car, a camera, who is at the door), and every pixel of the sphere looks
//! up its spot on that texture: the fine LED grid, darker edges, the turn. The same functions, the same numbers:
//! sphereProgram, faceMood, drawFace, paintTex, sphereMap, drawSphere, drawSphereLeds.

use crate::hub::{get, list, num, text};
use crate::palette::{Pal, C};
use serde_json::Value;
use tiny_skia::{Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Rect, Shader, Stroke, Transform};

const TW: f32 = 192.0;
const TH: f32 = 96.0;
const TX: f32 = 2.0;
const PI: f32 = std::f32::consts::PI;
const TAU: f32 = std::f32::consts::TAU;

type Rgb = [f32; 3]; // 0..255, as in pulse.js

fn c255(c: C) -> Rgb {
    [c[0] * 255.0, c[1] * 255.0, c[2] * 255.0]
}
fn mixc(a: Rgb, b: Rgb, k: f32) -> Rgb {
    [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
}
fn color(c: Rgb, a: f32) -> Color {
    Color::from_rgba(c[0].clamp(0.0, 255.0) / 255.0, c[1].clamp(0.0, 255.0) / 255.0, c[2].clamp(0.0, 255.0) / 255.0, a.clamp(0.0, 1.0)).unwrap_or(Color::BLACK)
}

/// What the sphere needs to know at this frame, beside the hub's state.
pub struct Feed<'a> {
    pub st: &'a Value,
    pub pal: &'a Pal,
    /// seconds since the wallpaper started (pulse.js's t)
    pub t: f64,
    /// "10:42", and "THU NOV 12"
    pub hm: &'a str,
    pub date: &'a str,
    pub gaze: (f32, f32),
    /// you move the pointer on an empty desktop
    pub watching: bool,
    /// seconds since you came back (the face wakes up), a large number otherwise
    pub woke: f64,
    /// a text panel is being typed out: the face talks
    pub typing: bool,
    pub alert_env: f32,
    pub audio: &'a [f32],
    pub bass: f32,
    /// seconds since the car changed
    pub car_age: f64,
    pub wind_k: f32,
    pub cam: Option<(&'a str, &'a [u8])>,
    pub door: Option<(&'a str, &'a [u8])>,
}

struct Map {
    d: usize,
    v: Vec<i32>,
    lam: Vec<f32>,
    k: Vec<f32>,
    cos: Vec<f32>,
    a: Vec<u8>,
    /// for every row, the first and the last pixel on the ball (the loop skips the sky around it)
    span: Vec<(usize, usize)>,
}

pub struct Sphere {
    tex: Pixmap,
    font: Option<std::rc::Rc<fontdue::Font>>,
    /// every letter already drawn, with its outline: made once, not every frame
    glyphs: Glyphs,
    map: Option<Map>,
    /// the sphere's picture, D×D, BGRA
    pub out: Vec<u8>,
    /// the same picture upside down at 20%: its reflection on the wet floor
    pub refl: Vec<u8>,
    pub d: usize,
    /// the average colour on screen (0..255): the halo and the light on the plaza take it
    pub avg: Rgb,
    prog_i: usize,
    prog_at: f64,
    pub prog: &'static str,
    led: Vec<(f32, f32)>,
    pics: std::collections::HashMap<&'static str, (String, Pixmap)>,
    /// the screen under the content (its gradient, its stripes, its two lines) as it was last painted, and what it
    /// was painted for: it changes only with the colours and the programme, so it is copied, not painted again
    base: (String, Vec<u8>),
    /// the content's own layer (or, while it is painted, the base under it)
    content: Pixmap,
}

fn font_file() -> Option<Vec<u8>> {
    // the font pulse.js asks for, in bold (the heaviest it has), found as fontconfig finds it
    for want in ["CaskaydiaMono Nerd Font:bold", "Cascadia Mono:bold", "monospace:bold"] {
        if let Ok(o) = std::process::Command::new("fc-match").args(["-f", "%{file}", want]).output() {
            let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !path.is_empty() {
                if let Ok(b) = std::fs::read(&path) {
                    return Some(b);
                }
            }
        }
    }
    None
}

/// pulse.js mainAgent(): the agent that matters most now.
fn main_agent(st: &Value) -> Option<&Value> {
    let order = |s: &str| match s { "waiting" => 5, "failed" => 4, "working" => 3, "done" => 2, "idle" => 1, _ => 0 };
    let mut best: Option<&Value> = None;
    for a in list(st, &["agents"]) {
        if best.map(|b| order(&text(a, &["state"])) > order(&text(b, &["state"]))).unwrap_or(true) {
            best = Some(a);
        }
    }
    best
}

/// pulse.js faceMood().
pub fn face_mood(f: &Feed) -> Option<&'static str> {
    let st = f.st;
    let face = get(st, &["ui", "face"]).as_str().unwrap_or("agents");
    let listen = get(st, &["listen"]);
    match text(listen, &["state"]).as_str() {
        "listening" => return Some("listen"),
        "thinking" => return Some("think"),
        _ => {}
    }
    if get(st, &["speaking"]).as_bool().unwrap_or(false) {
        return Some("talk");
    }
    if get(st, &["away"]).as_bool().unwrap_or(false) && face != "off" {
        return Some("sleep");
    }
    if face != "off" && f.woke < 2.6 {
        return Some("wake");
    }
    if f.typing {
        return Some("talk");
    }
    let Some(a) = main_agent(st) else {
        return if face == "always" || f.watching { Some("idle") } else { None };
    };
    Some(match text(a, &["state"]).as_str() { "working" => "think", "waiting" => "alert", "done" => "happy", "failed" => "sad", _ => "idle" })
}

/// A letter at one size: its coverage, and the same coverage widened all round (the dark outline of txt2).
struct Glyph {
    m: fontdue::Metrics,
    cov: Vec<u8>,
    out: Vec<u8>,
    r: i32,
}
type Glyphs = std::collections::HashMap<(char, u32), std::rc::Rc<Glyph>>;

fn glyph(font: &fontdue::Font, cache: &mut Glyphs, ch: char, px: f32, lw: f32) -> std::rc::Rc<Glyph> {
    let key = (ch, (px * 8.0) as u32);
    if let Some(g) = cache.get(&key) {
        return g.clone();
    }
    let (m, cov) = font.rasterize(ch, px);
    let r = lw.ceil() as i32;
    let (w0, h0) = (m.width as i32, m.height as i32);
    let (w1, h1) = (w0 + 2 * r, h0 + 2 * r);
    let ring: Vec<(i32, i32)> = (0..12).map(|i| { let a = i as f32 / 12.0 * TAU; ((a.cos() * lw).round() as i32, (a.sin() * lw).round() as i32) }).collect();
    // the outline as the canvas strokes it, twelve copies around the letter: their union, kept as one picture
    let mut out = vec![0u8; (w1.max(0) * h1.max(0)) as usize];
    for (dx, dy) in &ring {
        for gy in 0..h0 {
            for gx in 0..w0 {
                let v = cov[(gy * w0 + gx) as usize];
                if v == 0 { continue }
                let o = ((gy + r + dy) * w1 + gx + r + dx) as usize;
                // several layers of 85% black: the same as drawing them one over the other
                let a = out[o] as f32 / 255.0;
                let b = v as f32 / 255.0 * 0.85;
                out[o] = ((a + b - a * b) * 255.0) as u8;
            }
        }
    }
    if cache.len() > 600 {
        cache.clear();
    }
    let g = std::rc::Rc::new(Glyph { m, cov, out, r });
    cache.insert(key, g.clone());
    g
}

/// A painter over the texture, in the texture's units (the canvas calls of pulse.js, one for one).
struct Tex<'a> {
    pm: &'a mut Pixmap,
    font: Option<&'a fontdue::Font>,
    glyphs: &'a mut Glyphs,
    ts: Transform,
    alpha: f32,
}

fn arc_pts(cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32) -> Vec<(f32, f32)> {
    let n = (((a1 - a0).abs() / TAU * 48.0).ceil() as usize).clamp(2, 64);
    (0..=n).map(|i| { let a = a0 + (a1 - a0) * i as f32 / n as f32; (cx + a.cos() * rx, cy + a.sin() * ry) }).collect()
}

impl Tex<'_> {
    fn paint(&self, c: Rgb, a: f32) -> Paint<'static> {
        let mut p = Paint::default();
        p.set_color(color(c, a * self.alpha));
        p.anti_alias = true;
        p
    }
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgb, a: f32) {
        if let Some(r) = Rect::from_xywh(x, y, w.max(0.01), h.max(0.01)) {
            let p = self.paint(c, a);
            self.pm.fill_rect(r, &p, self.ts, None);
        }
    }
    fn path(pts: &[(f32, f32)], close: bool) -> Option<tiny_skia::Path> {
        let mut pb = PathBuilder::new();
        pb.move_to(pts.first()?.0, pts.first()?.1);
        for p in &pts[1..] {
            pb.line_to(p.0, p.1);
        }
        if close {
            pb.close();
        }
        pb.finish()
    }
    fn fill(&mut self, pts: &[(f32, f32)], c: Rgb, a: f32) {
        if let Some(path) = Self::path(pts, true) {
            let p = self.paint(c, a);
            self.pm.fill_path(&path, &p, FillRule::Winding, self.ts, None);
        }
    }
    fn stroke(&mut self, pts: &[(f32, f32)], lw: f32, c: Rgb, a: f32) {
        if let Some(path) = Self::path(pts, false) {
            let p = self.paint(c, a);
            let s = Stroke { width: lw, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Default::default() };
            self.pm.stroke_path(&path, &p, &s, self.ts, None);
        }
    }
    fn circle(&mut self, cx: f32, cy: f32, r: f32, c: Rgb, a: f32) {
        self.fill(&arc_pts(cx, cy, r, r, 0.0, TAU), c, a);
    }

    /// pulse.js txt2(): bold text, centred on x (or from x when `left`), its middle on y, with a dark outline.
    fn text(&mut self, s: &str, x: f32, y: f32, size: f32, c: Rgb, left: bool) -> f32 {
        let Some(font) = self.font else { return 0.0 };
        let px = size * TX;
        let lw = (size / 8.0).max(2.0) * TX / 2.0; // half the stroke's width, in pixels
        let glyphs: Vec<std::rc::Rc<Glyph>> = s.chars().map(|ch| glyph(font, self.glyphs, ch, px, lw)).collect();
        let width: f32 = glyphs.iter().map(|g| g.m.advance_width).sum();
        let lm = font.horizontal_line_metrics(px);
        let (asc, desc) = lm.map(|m| (m.ascent, m.descent)).unwrap_or((px * 0.8, -px * 0.2));
        let base = y * TX + (asc + desc) / 2.0; // textBaseline "middle": the middle of the em box
        let mut pen = if left { x * TX } else { x * TX - width / 2.0 };
        let (w, h) = (self.pm.width() as i32, self.pm.height() as i32);
        let alpha = self.alpha;
        let data = self.pm.data_mut();
        // tiny-skia keeps premultiplied pixels; the texture is opaque, so this is a plain mix
        let mut blit = |gw: i32, gh: i32, cov: &[u8], ox: i32, oy: i32, col: Rgb, k: f32| {
            for gy in 0..gh {
                let py = oy + gy;
                if py < 0 || py >= h { continue }
                let row = (gy * gw) as usize;
                for gx in 0..gw {
                    let v = cov[row + gx as usize];
                    if v == 0 { continue }
                    let pxx = ox + gx;
                    if pxx < 0 || pxx >= w { continue }
                    let a = v as f32 / 255.0 * k * alpha;
                    let o = ((py * w + pxx) * 4) as usize;
                    for ch in 0..3 {
                        data[o + ch] = (data[o + ch] as f32 * (1.0 - a) + col[ch] * a) as u8;
                    }
                }
            }
        };
        // strokeText in rgba(0,0,0,.85), then fillText
        let mut p2 = pen;
        for g in &glyphs {
            let (gx, gy) = ((p2 + g.m.xmin as f32).round() as i32, (base - g.m.height as f32 - g.m.ymin as f32).round() as i32);
            blit(g.m.width as i32 + 2 * g.r, g.m.height as i32 + 2 * g.r, &g.out, gx - g.r, gy - g.r, [0.0, 0.0, 0.0], 1.0);
            p2 += g.m.advance_width;
        }
        for g in &glyphs {
            let (gx, gy) = ((pen + g.m.xmin as f32).round() as i32, (base - g.m.height as f32 - g.m.ymin as f32).round() as i32);
            blit(g.m.width as i32, g.m.height as i32, &g.cov, gx, gy, c, 1.0);
            pen += g.m.advance_width;
        }
        width / TX
    }

    /// pulse.js drawFace(), on the texture.
    #[allow(clippy::too_many_arguments)]
    fn face(&mut self, x: f32, y: f32, s: f32, mood: &str, t: f32, col: Rgb, fixed: bool, gaze: (f32, f32), woke: f32, pal: &Pal) {
        let (mut ec, mut eye_w, mut eye_h, gap) = (col, s * 0.17, s * 0.3, s * 0.26);
        let (mut look_x, mut look_y) = (0.0f32, 0.0f32);
        if mood == "think" {
            look_x = (t * 1.3).sin() * s * 0.05;
            look_y = -s * 0.05;
        }
        if mood != "sleep" && mood != "sad" {
            look_x += gaze.0 * s * 0.15; // at your pointer
            look_y += gaze.1 * s * 0.1;
        }
        let mut blink = if mood == "idle" || mood == "think" { if t.rem_euclid(4.2) < 0.13 { 0.12 } else { 1.0 } } else { 1.0 };
        if mood == "wake" {
            blink = (woke / 1.1).clamp(0.1, 1.0);
        }
        if mood == "alert" {
            eye_w *= 1.15;
            eye_h *= 1.2;
            if !fixed { ec = c255(pal.warn) }
        }
        if mood == "listen" {
            eye_w *= 1.1;
            eye_h *= 1.12;
            look_y -= s * 0.03;
        }
        if mood == "sad" && !fixed {
            ec = c255(pal.crit);
        }
        let lw = (s * 0.07).max(1.5);
        for side in [-1.0f32, 1.0] {
            let (ex, ey) = (x + side * gap + look_x, y - s * 0.08 + look_y);
            match mood {
                "happy" => self.stroke(&arc_pts(ex, ey + s * 0.06, eye_w * 0.9, eye_w * 0.9, PI * 1.1, PI * 1.9), lw, ec, 0.95),
                "sleep" => self.stroke(&arc_pts(ex, ey - s * 0.03, eye_w * 0.75, eye_w * 0.75, PI * 0.15, PI * 0.85), lw, ec, 0.95),
                "sad" => {
                    self.stroke(&[(ex - eye_w * 0.7, ey - eye_w * 0.7), (ex + eye_w * 0.7, ey + eye_w * 0.7)], lw, ec, 0.95);
                    self.stroke(&[(ex + eye_w * 0.7, ey - eye_w * 0.7), (ex - eye_w * 0.7, ey + eye_w * 0.7)], lw, ec, 0.95);
                }
                _ => {
                    // a rounded eye: roundRect(ex - eyeW/2, ey - h/2, eyeW, h, min(eyeW, h)/2)
                    let h = eye_h * blink;
                    let r = eye_w.min(h) / 2.0;
                    let (x0, y0, x1, y1) = (ex - eye_w / 2.0, ey - h / 2.0, ex + eye_w / 2.0, ey + h / 2.0);
                    let mut pts = arc_pts(x1 - r, y0 + r, r, r, -PI / 2.0, 0.0);
                    pts.extend(arc_pts(x1 - r, y1 - r, r, r, 0.0, PI / 2.0));
                    pts.extend(arc_pts(x0 + r, y1 - r, r, r, PI / 2.0, PI));
                    pts.extend(arc_pts(x0 + r, y0 + r, r, r, PI, PI * 1.5));
                    self.fill(&pts, ec, 0.95);
                }
            }
            if mood == "alert" {
                self.stroke(&[(ex - eye_w * 0.8, ey - eye_h * 0.85), (ex + eye_w * 0.6, ey - eye_h * 1.0)], lw, ec, 0.95);
            }
        }
        let my = y + s * 0.26;
        match mood {
            "sleep" => {
                // asleep: a small mouth, and z z z drifting up
                self.stroke(&arc_pts(x, my, s * 0.04, s * 0.04, 0.0, TAU), lw, ec, 0.95);
                for zi in 0..3 {
                    let zp = (t * 0.35 + zi as f32 / 3.0).rem_euclid(1.0);
                    let (zs, zx, zy) = (s * (0.07 + zp * 0.09), x + s * (0.42 + zp * 0.3), y - s * (0.25 + zp * 0.55));
                    self.stroke(&[(zx, zy), (zx + zs, zy), (zx, zy + zs), (zx + zs, zy + zs)], lw, ec, 0.95 * (zp * PI).sin());
                }
            }
            "happy" | "wake" => self.stroke(&arc_pts(x, my - s * 0.08, s * 0.16, s * 0.16, PI * 0.15, PI * 0.85), lw, ec, 0.95),
            "sad" => self.stroke(&arc_pts(x, my + s * 0.08, s * 0.13, s * 0.13, PI * 1.2, PI * 1.8), lw, ec, 0.95),
            "alert" => self.stroke(&arc_pts(x, my, s * 0.06, s * 0.06, 0.0, TAU), lw, ec, 0.95),
            "talk" => {
                let o = (0.3 + 0.7 * (t * 9.0).sin().abs()) * s * 0.09;
                self.fill(&arc_pts(x, my, s * 0.1, o, 0.0, TAU), ec, 0.95);
            }
            "listen" => {
                // listening: a small level meter where the mouth is
                for li in -2..=2 {
                    let lh = s * (0.03 + 0.07 * (t * 7.0 + li as f32 * 1.3).sin().abs());
                    self.rect(x + li as f32 * s * 0.05 - s * 0.015, my - lh / 2.0, s * 0.03, lh, ec, 0.95);
                }
            }
            _ => {
                self.stroke(&[(x - s * 0.08, my), (x + s * 0.08, my)], lw, ec, 0.95);
                if mood == "think" {
                    for i in 0..3 {
                        let on = (t * 2.2 - i as f32 * 0.33).rem_euclid(1.0) < 0.5;
                        self.circle(x + s * (0.55 + i as f32 * 0.13), y - s * 0.45 - i as f32 * s * 0.08, s * 0.035, ec, 0.95 * if on { 1.0 } else { 0.3 });
                    }
                }
            }
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self::new()
    }
}

impl Sphere {
    pub fn new() -> Self {
        let font = font_file().and_then(|b| fontdue::Font::from_bytes(b, fontdue::FontSettings::default()).ok()).map(std::rc::Rc::new);
        // pulse.js ledGrid()
        let mut led = Vec::new();
        for i in 0..46 {
            let phi = -PI / 2.0 + (i as f32 + 0.5) / 46.0 * PI;
            let n = ((phi.cos() * 116.0).round() as usize).max(6);
            for j in 0..n {
                led.push((phi, j as f32 / n as f32 * TAU));
            }
        }
        Sphere { tex: Pixmap::new((TW * TX) as u32, (TH * TX) as u32).expect("the sphere's texture"), font, glyphs: Default::default(), map: None, out: Vec::new(), refl: Vec::new(), d: 0, avg: [0.0; 3], prog_i: 0, prog_at: 0.0, prog: "face", led, pics: Default::default(), base: Default::default(), content: Pixmap::new((TW * TX) as u32, (TH * TX) as u32).expect("the sphere's content") }
    }

    /// pulse.js sphereProgram(): what is urgent first, then the face, the music, and the programmes in turn.
    fn program(&mut self, f: &Feed) -> &'static str {
        let st = f.st;
        let now = num(st, &["t"]).unwrap_or(0.0);
        let face = get(st, &["ui", "face"]).as_str().unwrap_or("agents");
        // a fixed programme (PULSE_PROG=weather), for trying one out: window.PULSE_PROG on the web page
        if let Ok(fixed) = std::env::var("PULSE_PROG") {
            for p in ["clock", "weather", "cpu", "news", "music", "alert", "launch", "focus", "face", "meeting", "car", "door", "camera"] {
                if p == fixed {
                    return p;
                }
            }
        }
        if f.alert_env > 0.25 {
            return "alert";
        }
        if get(st, &["ring"]).as_bool().unwrap_or(false) && f.door.is_some() {
            return "door";
        }
        if let Some(start) = list(st, &["calendar"]).first().and_then(|c| num(c, &["start"])) {
            if start - now > 0.0 && start - now <= 60.0 {
                return "meeting"; // the last minute before a meeting: its countdown
            }
        }
        if get(st, &["car"]).is_object() && f.car_age < 10.0 {
            return "car"; // the car changed: its battery on the sphere
        }
        if !get(st, &["listen"]).is_null() || get(st, &["speaking"]).as_bool().unwrap_or(false) {
            return "face";
        }
        let l = get(st, &["space", "launch"]);
        if let Some(net) = num(l, &["net"]) {
            if text(l, &["status"]) == "Go" && net - now > 0.0 && net - now < 3600.0 {
                return "launch";
            }
        }
        if get(st, &["away"]).as_bool().unwrap_or(false) && face != "off" {
            return "face";
        }
        if get(st, &["focus"]).is_object() {
            return "focus";
        }
        if face_mood(f).is_some() {
            return "face";
        }
        if !f.audio.is_empty() {
            return "music";
        }
        let mut progs = vec!["clock", "weather", "cpu", "news"];
        if f.cam.is_some() {
            progs.push("camera");
        }
        if f.t - self.prog_at > 11.0 {
            self.prog_at = f.t;
            self.prog_i = (self.prog_i + 1) % progs.len();
        }
        progs[self.prog_i % progs.len()]
    }

    /// A camera's (or the door's) frame, decoded once for each new one.
    fn picture(&mut self, slot: &'static str, src: Option<(&str, &[u8])>, w: u32, h: u32) -> bool {
        let Some((key, bytes)) = src else {
            self.pics.remove(slot);
            return false;
        };
        if self.pics.get(slot).map(|p| p.0 != key).unwrap_or(true) {
            let Ok(img) = image::load_from_memory(bytes) else { return false };
            let rgba = img.resize_exact(w, h, image::imageops::FilterType::Triangle).to_rgba8();
            let Some(mut pm) = Pixmap::new(w, h) else { return false };
            pm.data_mut().copy_from_slice(rgba.as_raw());
            self.pics.insert(slot, (key.to_string(), pm));
        }
        true
    }

    /// drawImage with "lighter": the picture's light is added to the texture, `times` over.
    fn add_picture(&mut self, slot: &'static str, x: f32, y: f32, times: f32) {
        let Some((_, pic)) = self.pics.get(slot) else { return };
        let (tw, th) = (self.tex.width() as i32, self.tex.height() as i32);
        let (pw, ph) = (pic.width() as i32, pic.height() as i32);
        let (x0, y0) = ((x * TX) as i32, (y * TX) as i32);
        let src = pic.data().to_vec();
        let data = self.tex.data_mut();
        for py in 0..ph {
            let ty = y0 + py;
            if ty < 0 || ty >= th { continue }
            for px in 0..pw {
                let txx = x0 + px;
                if txx < 0 || txx >= tw { continue }
                let (o, s) = (((ty * tw + txx) * 4) as usize, ((py * pw + px) * 4) as usize);
                for ch in 0..3 {
                    data[o + ch] = (data[o + ch] as f32 + src[s + ch] as f32 * times).min(255.0) as u8;
                }
            }
        }
    }

    /// pulse.js paintTex().
    fn paint_tex(&mut self, prog: &'static str, f: &Feed, front: f32, led_style: bool) {
        let (st, pal) = (f.st, f.pal);
        let t = f.t as f32;
        let (ac, a2, fg) = (c255(pal.accent), c255(pal.accent2), c255(pal.fg));
        let (warn, crit, ok) = (c255(pal.warn), c255(pal.crit), c255(pal.ok));
        let mood = face_mood(f);
        let hot = if prog == "alert" { Some(crit) } else if prog == "face" && mood == Some("alert") { Some(warn) } else { None };
        let now = num(st, &["t"]).unwrap_or(0.0);
        // pictures are decoded before the painter borrows the texture
        let has_cam = prog == "camera" && self.picture("cam", f.cam, (TH * 1.8 * TX) as u32, (TH * 0.76 * TX) as u32);
        let has_door = prog == "door" && self.picture("door", f.door, (TH * 1.9 * TX) as u32, (TH * 0.88 * TX) as u32);
        let font = self.font.clone();
        // the sphere is a screen: a glowing base that turns with it, content on top. The base depends only on the
        // colours, the programme and an alert, so it is painted once and copied after that
        let base_key = format!("{led_style}{prog}{hot:?}{ac:?}{a2:?}{fg:?}");
        if self.base.0 == base_key && self.base.1.len() == self.tex.data().len() {
            self.tex.data_mut().copy_from_slice(&self.base.1);
        }
        let mut c = Tex { pm: &mut self.tex, font: font.as_deref(), glyphs: &mut self.glyphs, ts: Transform::from_scale(TX, TX), alpha: 1.0 };
        if self.base.0 != base_key || self.base.1.len() != c.pm.data().len() {
            c.rect(0.0, 0.0, TW, TH, [0.0; 3], 1.0);
            if led_style {
                // the LED look: a dark screen, only what is drawn lights up
                if let Some(h) = hot {
                    c.rect(0.0, 0.0, TW, TH, h, if prog == "alert" { 0.75 } else { 0.18 });
                }
                c.rect(0.0, TH * 0.06, TW, 1.0, ac, 0.22);
                c.rect(0.0, TH * 0.93, TW, 1.0, ac, 0.22);
            } else {
                let stops = vec![
                    tiny_skia::GradientStop::new(0.0, color(hot.unwrap_or(a2), 0.95)),
                    tiny_skia::GradientStop::new(0.5, color(hot.unwrap_or(mixc(a2, ac, 0.5)), 0.8)),
                    tiny_skia::GradientStop::new(1.0, color(hot.unwrap_or(ac), 0.95)),
                ];
                if let (Some(sh), Some(r)) = (tiny_skia::LinearGradient::new((0.0, 0.0).into(), (0.0, TH).into(), stops, tiny_skia::SpreadMode::Pad, Transform::identity()), Rect::from_xywh(0.0, 0.0, TW, TH)) {
                    let p = Paint { shader: sh, anti_alias: false, ..Default::default() };
                    c.pm.fill_rect(r, &p, c.ts, None);
                }
                let dark = if prog == "face" || prog == "alert" { 0.08 } else { 0.16 };
                let mut i = 0.0;
                while i < TW {
                    c.rect(i, 0.0, 3.0, TH, [0.0; 3], dark); // faint stripes fixed to the sphere: you see it turn
                    i += 24.0;
                }
                c.rect(0.0, TH * 0.06, TW, 1.0, fg, 0.5);
                c.rect(0.0, TH * 0.93, TW, 1.0, fg, 0.5);
                if matches!(prog, "music" | "camera" | "door") {
                    c.rect(0.0, 0.0, TW, TH, [0.0; 3], 0.7);
                }
            }
            self.base = (base_key, c.pm.data().to_vec());
        }
        // the content is painted still, in the middle of its own layer, and laid on the base shifted to the front
        // with sub-texel precision (the glyphs and pictures land on whole pixels: painted where the front is, they
        // stepped a texel at a time while the sphere turned smoothly under them)
        let _ = c;
        std::mem::swap(&mut self.tex, &mut self.content); // the base waits in `content`
        self.tex.fill(tiny_skia::Color::TRANSPARENT);
        let mut c = Tex { pm: &mut self.tex, font: font.as_deref(), glyphs: &mut self.glyphs, ts: Transform::from_scale(TX, TX), alpha: 1.0 };
        let shift = front + (t * 0.25).sin() * TW * 0.04 - TW / 2.0; // content gently sways around the front
        let fx = TW / 2.0;
        let at = [fx, fx - TW, fx + TW];
        match prog {
            "clock" => {
                for x in at {
                    c.text(f.hm, x, TH * 0.47, 21.0, fg, false);
                    c.text(f.date, x, TH * 0.74, 9.0, fg, false);
                }
            }
            "weather" => {
                let w = get(st, &["weather"]);
                let has = get(w, &["temp"]).is_number();
                let wk = if has { crate::ui::wx_kind(num(w, &["code"]).unwrap_or(0.0) as i32) } else { "clear" };
                let night = has && num(w, &["day"]).map(|d| d == 0.0).unwrap_or(get(w, &["day"]).as_bool() == Some(false));
                let wind_line = has && f.wind_k >= 0.5 && ((now * 1000.0 / 3000.0) as i64) % 2 == 1;
                for x in at {
                    // the real weather: sun or moon, clouds, rain, snow, a storm, fog
                    let (ix, iy) = (x - 24.0, TH * 0.45);
                    let cloud = |c: &mut Tex, cx2: f32, cy2: f32, r: f32, col: Rgb| {
                        c.circle(cx2 - r * 0.55, cy2, r * 0.6, col, 1.0);
                        c.circle(cx2 + r * 0.25, cy2 - r * 0.35, r * 0.75, col, 1.0);
                        c.circle(cx2 + r * 0.8, cy2 + r * 0.05, r * 0.55, col, 1.0);
                        c.rect(cx2 - r * 0.55, cy2, r * 1.35, r * 0.6, col, 1.0);
                    };
                    let sky = |c: &mut Tex, cx2: f32, cy2: f32, r: f32| {
                        if night {
                            // a crescent moon
                            let mut pts = arc_pts(cx2, cy2, r, r, PI * 0.5, PI * 1.5);
                            pts.extend(arc_pts(cx2, cy2, r * 0.45, r, PI * 1.5, PI * 0.5));
                            c.fill(&pts, fg, 1.0);
                        } else {
                            c.circle(cx2, cy2, r, warn, 1.0);
                        }
                    };
                    match wk {
                        "clear" => sky(&mut c, ix, iy, 11.0),
                        "part" => {
                            sky(&mut c, ix - 5.0, iy - 5.0, 8.0);
                            cloud(&mut c, ix + 2.0, iy + 3.0, 9.0, fg);
                        }
                        _ => {
                            cloud(&mut c, ix, iy - 2.0, 10.0, if wk == "storm" { mixc(fg, [80.0, 80.0, 90.0], 0.45) } else { fg });
                            let dc = if wk == "snow" { fg } else { ac };
                            if wk == "rain" || wk == "storm" {
                                for d in -1..=1 { c.rect(ix + d as f32 * 6.0 - 1.0, iy + 9.0, 2.4, 6.0, dc, 1.0) }
                            }
                            if wk == "snow" {
                                for d in -1..=1 { c.circle(ix + d as f32 * 6.0, iy + 12.0, 2.0, dc, 1.0) }
                            }
                            if wk == "storm" {
                                c.fill(&[(ix + 1.0, iy + 6.0), (ix - 4.0, iy + 14.0), (ix, iy + 14.0), (ix - 3.0, iy + 21.0), (ix + 5.0, iy + 11.0), (ix + 1.0, iy + 11.0)], warn, 1.0);
                            }
                            if wk == "fog" {
                                for k in 0..3 { c.rect(ix - 12.0, iy + 9.0 + k as f32 * 4.0, 24.0, 2.0, fg, 0.9) }
                            }
                        }
                    }
                    if f.wind_k >= 0.5 {
                        // strong wind: three gust lines across the icon
                        for wi in 0..3 {
                            let (wy, wl) = (iy - 10.0 + wi as f32 * 7.0, 16.0 - wi as f32 * 3.0);
                            let mut pts = vec![(ix - 16.0, wy), (ix - 16.0 + wl, wy)];
                            pts.extend(arc_pts(ix - 16.0 + wl, wy - 3.0, 3.0, 3.0, PI / 2.0, -PI * 0.9));
                            c.stroke(&pts, 2.0, fg, 0.95);
                        }
                    }
                    c.text(&if has { format!("{:.0}°", num(w, &["temp"]).unwrap_or(0.0)) } else { "--".into() }, x + 16.0, TH * 0.46, 30.0, fg, false);
                    let line = if wind_line { format!("WIND {:.0} {}", num(w, &["wind"]).unwrap_or(0.0).max(num(w, &["gust"]).unwrap_or(0.0)), text(w, &["windUnit"]).to_uppercase()) } else if has { text(w, &["city"]).to_uppercase() } else { String::new() };
                    c.text(&line, x, TH * 0.78, 11.0, fg, false);
                }
            }
            "cpu" => {
                // the last minutes of CPU as bars, scaled to the busiest moment (at least 25%), so a quiet PC is not a black band
                let hcpu: Vec<f32> = list(st, &["hist", "cpu"]).iter().filter_map(|v| v.as_f64().map(|x| x as f32)).collect();
                let top = hcpu.iter().cloned().fold(25.0f32, f32::max);
                c.rect(0.0, TH * 0.42, TW, TH * 0.48, [0.0; 3], 0.18);
                for i in 0..TW as usize {
                    let v = if hcpu.is_empty() { 0.0 } else { hcpu[((i as f32 / TW * hcpu.len() as f32) as usize).min(hcpu.len() - 1)] };
                    let hh = (TH * 0.02).max(v / top * TH * 0.42);
                    c.rect(i as f32, TH * 0.88 - hh, 1.0, hh, if v > 80.0 { crit } else if v > 50.0 { warn } else { ac }, 0.95);
                }
                let cpu = num(st, &["sys", "cpu"]).unwrap_or(0.0);
                for x in at {
                    c.text(&format!("CPU {cpu:.0}%"), x, TH * 0.3, 22.0, fg, false);
                }
            }
            "news" => {
                let nw = list(st, &["widgets"]).iter().find(|w| matches!(text(w, &["id"]).as_str(), "news" | "ai"));
                let news = nw.map(|w| { let first = list(w, &["data", "items"]).first().map(|i| text(i, &["title"])).unwrap_or_default(); if first.is_empty() { text(w, &["data", "text"]).lines().next().unwrap_or("").to_string() } else { first } }).unwrap_or_default();
                let txt = format!("{}   ◆   ", if news.is_empty() || get(st, &["privacy"]).as_bool().unwrap_or(false) { "omarchy pulse".to_string() } else { news });
                // measure once (off the texture), then run it across
                let w2 = { c.alpha = 0.0; let w = c.text(&txt, 0.0, -100.0, 24.0, fg, true).max(1.0); c.alpha = 1.0; w };
                let off = (t * 26.0).rem_euclid(w2);
                let mut x2 = -off;
                while x2 < TW + w2 {
                    c.text(&txt, x2, TH * 0.5, 24.0, fg, true);
                    x2 += w2;
                }
            }
            "music" => {
                let nb = f.audio.len().max(1);
                let mut i = 0.0;
                while i < TW {
                    let k = (((i / TW) * 2.0 - 1.0).abs() * (nb - 1) as f32) as usize;
                    let v2 = f.audio.get(k).copied().unwrap_or(0.0) / 100.0;
                    let hh2 = v2 * TH * 0.42;
                    c.rect(i, TH / 2.0 - hh2, 3.0, hh2 * 2.0, mixc(ac, a2, i / TW), 1.0);
                    i += 4.0;
                }
            }
            "alert" => {
                let pulse = 0.55 + 0.45 * (t * 8.0).sin();
                c.rect(0.0, 0.0, TW, TH, crit, 0.55 * pulse + 0.2);
                for x in at {
                    c.text("!", x, TH * 0.52, 56.0, fg, false);
                }
            }
            "launch" => {
                // the countdown of the next launch
                let l = get(st, &["space", "launch"]);
                let tl = (num(l, &["net"]).unwrap_or(now) - now).max(0.0);
                let rocket: String = text(l, &["rocket"]).to_uppercase().chars().take(12).collect();
                for x in at {
                    c.text("T-MINUS", x, TH * 0.2, 9.0, fg, false);
                    c.text(&format!("{:02}:{:02}", (tl / 60.0) as i64, (tl % 60.0) as i64), x, TH * 0.5, 26.0, fg, false);
                    c.text(&rocket, x, TH * 0.76, 9.0, fg, false);
                }
            }
            "focus" => {
                let fo = get(st, &["focus"]);
                let (a, b) = (num(fo, &["start"]).unwrap_or(now), num(fo, &["until"]).unwrap_or(now));
                let (left, tot) = ((b - now).max(0.0), (b - a).max(1.0));
                c.rect(0.0, TH * 0.86, TW * (1.0 - left / tot) as f32, TH * 0.06, ac, 0.85);
                let label: String = text(fo, &["label"]).to_uppercase().chars().take(14).collect();
                for x in at {
                    c.text(&format!("{:02}:{:02}", (left / 60.0) as i64, (left % 60.0) as i64), x, TH * 0.44, 32.0, fg, false);
                    c.text(&format!("FOCUS · {label}"), x, TH * 0.72, 11.0, fg, false);
                }
            }
            "face" => {
                if let Some(m) = mood {
                    for x in at {
                        if led_style {
                            c.face(x, TH * 0.5, TH * 0.6, m, t, warn, false, f.gaze, f.woke as f32, pal);
                        } else {
                            c.face(x, TH * 0.5, TH * 0.6, m, t, [8.0, 8.0, 14.0], true, f.gaze, f.woke as f32, pal);
                        }
                    }
                }
            }
            "meeting" => {
                let m = list(st, &["calendar"]).first().cloned().unwrap_or(Value::Null);
                let tm = (num(&m, &["start"]).unwrap_or(now) - now).max(0.0);
                let title: String = if get(st, &["privacy"]).as_bool().unwrap_or(false) { String::new() } else { text(&m, &["title"]).to_uppercase().chars().take(12).collect() };
                for x in at {
                    c.text("MEETING IN", x, TH * 0.2, 9.0, warn, false);
                    c.text(&format!("0:{:02}", (tm % 60.0) as i64), x, TH * 0.5, 26.0, fg, false);
                    c.text(&title, x, TH * 0.76, 9.0, fg, false);
                }
            }
            "car" => {
                let car = get(st, &["car"]);
                let cb = num(car, &["battery"]).unwrap_or(0.0).round();
                let ccol = if cb < 20.0 { crit } else if cb < 40.0 { warn } else { ok };
                c.rect(0.0, TH * 0.62, TW, TH * 0.1, [0.0; 3], 0.5);
                c.rect(0.0, TH * 0.62, TW * cb as f32 / 100.0, TH * 0.1, ccol, 1.0);
                let state = match text(car, &["state"]).as_str() { "Complete" => "CHARGED", "Charging" => "CHARGING", _ => "CAR" };
                let range = num(car, &["range"]).map(|r| format!("{r:.0} {}", text(car, &["unit"]).to_uppercase())).unwrap_or_default();
                for x in at {
                    c.text(state, x, TH * 0.2, 9.0, ccol, false);
                    c.text(&format!("{cb:.0}%"), x, TH * 0.45, 26.0, fg, false);
                    c.text(&range, x, TH * 0.84, 10.0, fg, false);
                }
            }
            _ => {}
        }
        if has_door {
            // the doorbell: who is at the door, on the whole sphere
            for x in at { self.add_picture("door", x - TH * 0.95, TH * 0.06, 1.0) }
            let font = self.font.clone();
            let mut c = Tex { pm: &mut self.tex, font: font.as_deref(), glyphs: &mut self.glyphs, ts: Transform::from_scale(TX, TX), alpha: 1.0 };
            for x in at { c.text("DOOR", x, TH * 0.9, 9.0, warn, false); }
        } else if has_cam {
            // night cameras are dark: twice as bright on the sphere
            for x in at { self.add_picture("cam", x - TH * 0.9, TH * 0.12, 2.0) }
        }
        self.lay_content(shift);
    }

    /// The content layer (in `tex`) over the base (in `content`), moved `shift` texels to the right, between two
    /// pixels where it falls between them; the result ends in `tex`.
    fn lay_content(&mut self, shift: f32) {
        let (w, h) = (self.tex.width() as usize, self.tex.height() as usize);
        let s = (shift * TX).rem_euclid(w as f32);
        let (i, fr) = (s.floor() as usize, s - s.floor());
        let (k1, k0) = ((fr * 256.0) as u32, 256 - (fr * 256.0) as u32);
        let src = self.tex.data();
        let dst = self.content.data_mut();
        for y in 0..h {
            let row = y * w;
            for x in 0..w {
                // content at x - s: between x - i (weight 1 - fr) and x - i - 1 (weight fr)
                let a = (x + w - i) % w;
                let b = (a + w - 1) % w;
                let (pa, pb) = ((row + a) * 4, (row + b) * 4);
                let ca = (src[pa + 3] as u32 * k0 + src[pb + 3] as u32 * k1) >> 8;
                if ca == 0 {
                    continue;
                }
                let o = (row + x) * 4;
                let keep = 255 - ca.min(255);
                for ch in 0..3 {
                    let cv = (src[pa + ch] as u32 * k0 + src[pb + ch] as u32 * k1) >> 8;
                    dst[o + ch] = (cv + dst[o + ch] as u32 * keep / 255).min(255) as u8;
                }
                dst[o + 3] = (ca + dst[o + 3] as u32 * keep / 255).min(255) as u8;
            }
        }
        std::mem::swap(&mut self.tex, &mut self.content);
    }

    /// pulse.js sphereMap(): for every pixel of a D×D sphere, its row on the texture, its longitude, how lit it is.
    fn make_map(d: usize) -> Map {
        let (r, n) = (d as f32 / 2.0, d * d);
        let rows = ((d as f32 / 2.6).round()).max(40.0);
        let mut m = Map { d, v: vec![-1; n], lam: vec![0.0; n], k: vec![0.0; n], cos: vec![0.0; n], a: vec![0; n], span: vec![(0, 0); d] };
        for y in 0..d {
            for x in 0..d {
                let i = y * d + x;
                let (nx, ny) = ((x as f32 + 0.5 - r) / r, (y as f32 + 0.5 - r) / r);
                let r2 = nx * nx + ny * ny;
                if r2 >= 1.0 {
                    continue;
                }
                let nz = (1.0 - r2).sqrt();
                let phi = (-ny).asin();
                let v_row = (0.5 - phi / PI) * TH * TX;
                m.v[i] = (v_row as i32).clamp(0, (TH * TX) as i32 - 1);
                m.lam[i] = nx.atan2(nz) / TAU; // in turns
                let sp = &mut m.span[y];
                if sp.1 == 0 {
                    sp.0 = x;
                }
                sp.1 = x + 1;
                m.cos[i] = phi.cos();
                let row_f = ((0.5 - phi / PI) * rows).rem_euclid(1.0); // between two rows of LEDs: darker
                let grid = 0.62 + 0.38 * (row_f * PI).sin().powf(0.6);
                let limb = 0.5 + 0.5 * nz.powf(0.45); // the curve: a little dimmer toward the edge
                let rim = 1.0 + 0.25 * (1.0 - nz).powi(3); // light gathers at the rim, like on glass
                m.k[i] = grid * limb * rim;
                let rr = r2.sqrt(); // the edge fades into the sky: no outline
                m.a[i] = if rr > 0.93 { ((1.0 - rr) / 0.07 * 255.0).clamp(0.0, 255.0) as u8 } else { 255 };
            }
        }
        m
    }

    /// pulse.js drawSphere(), up to the picture of the sphere itself (`out`, D×D, BGRA) and the colour on screen.
    /// Returns the turn of the sphere (for whoever draws around it).
    /// pulse.js drawNoirCity(): the big screen of the noir scene shows the sphere's texture as seen from the front
    /// (its middle three fifths) as a giant LED billboard: BGRA, straight alpha, and its size.
    pub fn board(&mut self, f: &Feed) -> (Vec<u8>, u32, u32) {
        self.prog = self.program(f);
        self.paint_tex(self.prog, f, TW / 2.0, false);
        let (tw, th) = (self.tex.width() as usize, self.tex.height() as usize);
        let x0 = ((TW / 2.0 - TW * 0.3) * TX) as usize;
        let cw = ((TW * 0.6 * TX) as usize).min(tw - x0);
        let data = self.tex.data();
        // pulse-native: a giant LED billboard, as over a Tokyo crossing: every LED a round dot of light (one for
        // two texels), brighter in the middle, a dark gap between them; the colours pushed a little, as LEDs are
        let (lc, lr, cell) = (cw / 2, th / 2, 5usize);
        let (iw, ih) = (lc * cell, lr * cell);
        let mut out = vec![0u8; iw * ih * 4];
        let dot: Vec<f32> = (0..cell * cell)
            .map(|k| {
                let (dx, dy) = ((k % cell) as f32 + 0.5 - cell as f32 / 2.0, (k / cell) as f32 + 0.5 - cell as f32 / 2.0);
                let d = (dx * dx + dy * dy).sqrt() / (cell as f32 * 0.5);
                (1.15 - d * d * 0.95).clamp(0.08, 1.0)
            })
            .collect();
        for r in 0..lr {
            for c in 0..lc {
                // the LED's colour: the two-by-two texels it stands for
                let mut acc = [0f32; 4];
                for (dy, dx) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                    let i = ((r * 2 + dy) * tw + x0 + c * 2 + dx) * 4;
                    for ch in 0..4 {
                        acc[ch] += data[i + ch] as f32 / 4.0;
                    }
                }
                let a = acc[3].max(1.0);
                let (rr, gg, bb) = (acc[0] * 255.0 / a, acc[1] * 255.0 / a, acc[2] * 255.0 / a);
                let lum = (rr + gg + bb) / 3.0;
                let sat = |v: f32| (lum + (v - lum) * 1.25).clamp(0.0, 255.0) * 1.12;
                let (rr, gg, bb) = (sat(rr), sat(gg), sat(bb));
                for k in 0..cell * cell {
                    let (px, py) = (c * cell + k % cell, r * cell + k / cell);
                    let o = (py * iw + px) * 4;
                    let m = dot[k];
                    out[o] = (bb * m).min(255.0) as u8;
                    out[o + 1] = (gg * m).min(255.0) as u8;
                    out[o + 2] = (rr * m).min(255.0) as u8;
                    out[o + 3] = (acc[3].max(200.0)) as u8;
                }
            }
        }
        (out, iw as u32, ih as u32)
    }

    pub fn frame(&mut self, f: &Feed, radius: f32) -> f32 {
        let d = ((radius * 2.0).round() as usize).max(16);
        if self.map.as_ref().map(|m| m.d != d).unwrap_or(true) {
            self.map = Some(Self::make_map(d));
            self.d = d;
        }
        // the buffers go to the GPU every frame (taken, not copied): new ones of the right size
        if self.out.len() != d * d * 4 {
            self.out = vec![0; d * d * 4];
        }
        if self.refl.len() != d * d * 4 {
            self.refl = vec![0; d * d * 4];
        }
        let t = f.t as f32;
        let music = !f.audio.is_empty();
        let rot = t * 0.2 + if music { f.bass * 0.35 } else { 0.0 };
        let front = rot.rem_euclid(TAU) / TAU * TW; // what faces you
        let led_style = get(f.st, &["ui", "sphere"]).as_str() == Some("led");
        self.prog = self.program(f);
        crate::perf::time("sphere-tex", || self.paint_tex(self.prog, f, front, led_style));
        let twp = (TW * TX) as usize;
        let thp = (TH * TX) as usize;
        let td = self.tex.data();
        let bg = c255(f.pal.bg);
        if led_style {
            // pulse.js drawSphereLeds(): a dark ball, a dot for each LED in the colour of its spot on the texture
            let r = d as f32 / 2.0;
            let body = mixc(bg, [0.0; 3], 0.4);
            for y in 0..d {
                for x in 0..d {
                    let o = (y * d + x) * 4;
                    let (nx, ny) = ((x as f32 + 0.5 - r) / r, (y as f32 + 0.5 - r) / r);
                    let inside = nx * nx + ny * ny < 1.0;
                    self.out[o] = body[2] as u8;
                    self.out[o + 1] = body[1] as u8;
                    self.out[o + 2] = body[0] as u8;
                    self.out[o + 3] = if inside { 255 } else { 0 };
                }
            }
            let (dot, lift, dim) = ((r * 0.03).max(1.0), 1.0 + 0.5 * f.bass, c255(f.pal.dim));
            for (phi, lam) in &self.led {
                let cp = phi.cos();
                let z = cp * lam.cos();
                if z < 0.02 {
                    continue;
                }
                let uu = ((lam + rot) / TAU).rem_euclid(1.0);
                let (sx, sy) = (r + cp * lam.sin() * r, r - phi.sin() * r);
                let o = (((((0.5 - phi / PI) * thp as f32) as usize).min(thp - 1)) * twp + ((uu * twp as f32) as usize).min(twp - 1)) * 4;
                let (cr, cg, cb) = (td[o] as f32, td[o + 1] as f32, td[o + 2] as f32);
                let (lum, shade, sz) = ((cr + cg + cb) / 765.0, 0.35 + 0.65 * z, dot * (0.55 + 0.45 * z));
                let (col, a) = if lum < 0.05 { (dim, 0.22 * shade) } else { ([cr * shade * lift, cg * shade * lift, cb * shade * lift], (0.4 + lum).min(1.0)) };
                let (x0, y0, x1, y1) = (((sx - sz / 2.0).round() as i32).max(0), ((sy - sz / 2.0).round() as i32).max(0), ((sx + sz / 2.0).round() as i32).min(d as i32), ((sy + sz / 2.0).round() as i32).min(d as i32));
                for py in y0..y1.max(y0 + 1).min(d as i32) {
                    for px in x0..x1.max(x0 + 1).min(d as i32) {
                        let q = (py as usize * d + px as usize) * 4;
                        if self.out[q + 3] == 0 { continue }
                        self.out[q] = (self.out[q] as f32 * (1.0 - a) + col[2].min(255.0) * a) as u8;
                        self.out[q + 1] = (self.out[q + 1] as f32 * (1.0 - a) + col[1].min(255.0) * a) as u8;
                        self.out[q + 2] = (self.out[q + 2] as f32 * (1.0 - a) + col[0].min(255.0) * a) as u8;
                    }
                }
            }
            for y in 0..d {
                let (src, dst) = (y * d * 4, (d - 1 - y) * d * 4);
                for x in 0..d {
                    let (a, b) = (src + x * 4, dst + x * 4);
                    self.refl[b..b + 3].copy_from_slice(&self.out[a..a + 3].to_vec());
                    self.refl[b + 3] = (self.out[a + 3] as f32 * 0.2) as u8;
                }
            }
            self.avg = c255(f.pal.accent2);
            return rot;
        }
        let m = self.map.as_ref().expect("the sphere's map");
        let lift = 1.0 + 0.45 * f.bass;
        let cols = twp as f32 / 2.0; // LED columns around the sphere
        let (bg_r, bg_g, bg_b) = (bg[0] * 0.6, bg[1] * 0.6, bg[2] * 0.6); // dark LEDs take the night's colour
        let (mut sr, mut sg, mut sb, mut cnt) = (0.0f32, 0.0f32, 0.0f32, 0u32);
        // 0.7 + 0.3 * sqrt(sin(x * PI)), looked up instead of computed for every pixel
        let shade: [f32; 256] = std::array::from_fn(|i| 0.7 + 0.3 * ((i as f32 + 0.5) / 256.0 * PI).sin().sqrt());
        // the picture and, in the same pass, its reflection (upside down, at 20%) for the wet floor
        let twf = twp as f32;
        let rot_t = rot / TAU;
        // the reflection shows only its upper part (the floor's clip): the rows of the ball that end up there
        let refl_from = d * 3 / 10;
        for y in 0..d {
            let (x0, x1) = m.span[y];
            if x1 <= x0 {
                continue; // a row of sky: the buffers are already clear
            }
            let row0 = y * d;
            let ry = (d - 1 - y) * d;
            let out_row = &mut self.out[(row0 + x0) * 4..(row0 + x1) * 4];
            let (v_r, lam_r, cos_r, k_r, a_r) = (&m.v[row0 + x0..row0 + x1], &m.lam[row0 + x0..row0 + x1], &m.cos[row0 + x0..row0 + x1], &m.k[row0 + x0..row0 + x1], &m.a[row0 + x0..row0 + x1]);
            let mut refl_row = if y >= refl_from { Some(&mut self.refl[(ry + x0) * 4..(ry + x1) * 4]) } else { None };
            for j in 0..(x1 - x0) {
                let v = v_r[j];
                if v < 0 {
                    continue;
                }
                let q = lam_r[j] + rot_t;
                let uu = q - q.floor();
                // between two texels: mixed by how far, so the content glides around the sphere instead of
                // stepping a texel at a time (pulse.js takes the nearest one)
                let uf = uu * twf;
                let u0 = (uf as usize).min(twp - 1);
                let u1 = if u0 + 1 >= twp { 0 } else { u0 + 1 };
                let fu = uf - u0 as f32;
                let row = v as usize * twp;
                let (t0, t1) = ((row + u0) * 4, (row + u1) * 4);
                let cq = uu * cols * cos_r[j];
                let col_f = cq - cq.floor(); // between two columns: darker
                let k = k_r[j] * shade[((col_f * 255.0) as usize).min(255)] * lift;
                let (p0, p1) = (&td[t0..t0 + 3], &td[t1..t1 + 3]);
                let r = p0[0] as f32 + (p1[0] as f32 - p0[0] as f32) * fu;
                let g = p0[1] as f32 + (p1[1] as f32 - p0[1] as f32) * fu;
                let b = p0[2] as f32 + (p1[2] as f32 - p0[2] as f32) * fu;
                let a = a_r[j];
                let px4 = [(b * k + bg_b).min(255.0) as u8, (g * k + bg_g).min(255.0) as u8, (r * k + bg_r).min(255.0) as u8, a];
                out_row[j * 4..j * 4 + 4].copy_from_slice(&px4);
                if let Some(rr) = refl_row.as_deref_mut() {
                    rr[j * 4..j * 4 + 4].copy_from_slice(&[px4[0], px4[1], px4[2], (a as u32 * 51 / 255) as u8]);
                }
                if (row0 + x0 + j) & 15 == 0 {
                    sr += r;
                    sg += g;
                    sb += b;
                    cnt += 1;
                }
            }
        }
        if cnt > 0 {
            self.avg = [sr / cnt as f32, sg / cnt as f32, sb / cnt as f32];
        }
        rot
    }
}

#[allow(dead_code)]
fn unused(_: Shader) {}
