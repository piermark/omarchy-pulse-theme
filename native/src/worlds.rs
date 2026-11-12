//! Pulse 1.8's two scenes in code, mars and monolith (extras/pulse/web/scenes/*.js), written again in Rust: the same
//! worlds, the same data behind every piece, the same lines when you click. They read what the hub gives a scene
//! (`state.scene`: no location, nothing private), exactly as the JavaScript ones do.

use crate::draw::{c8, Hit, Label, PI, TAU};
use crate::hub::{get, list, num, text};
use crate::life::Frame;
use crate::palette::{mix, Pal, C};
use crate::scene::Rand;
use gpui::Window;
use serde_json::{json, Value};
use std::collections::HashMap;

/// What a scene in code keeps between frames.
pub struct Mem {
    r: Rand,
    stars: Vec<[f32; 4]>,
    far: Vec<f32>,
    mesa: Vec<[f32; 4]>,
    rocks: Vec<[f32; 4]>,
    dust: Vec<[f32; 4]>,
    devil: [f32; 2],
    ridge: Vec<f32>,
    rov: HashMap<String, (f32, f32)>,
    lift: HashMap<String, f64>,
    flash: HashMap<String, f64>,
    go: Vec<String>,
    puffs: Vec<(f32, f32, f64, f32)>,
    pulses: Vec<f64>,
    met: Option<(f32, f32, f64)>,
    orb: HashMap<String, (f32, f32)>,
    rip: Vec<(f32, f32, f64, f32)>,
    prev: Option<Value>,
    said: HashMap<String, f64>,
    seen: f64,
}

impl Default for Mem {
    fn default() -> Self {
        let mut r = Rand::new(1977);
        let four = |r: &mut Rand| [r.next(), r.next(), r.next(), r.next()];
        let stars = (0..180).map(|_| { let v = four(&mut r); [v[0], v[1] * 0.95, 0.3 + v[2] * 0.7, v[3] * 6.28] }).collect();
        let far = (0..=96).map(|i| { let x = i as f32 / 96.0; (0.62 * (-((x - 0.33) / 0.19).powi(2)).exp()).min(0.5) + 0.26 * (-((x - 0.84) / 0.07).powi(2)).exp() + r.next() * 0.018 }).collect();
        let mesa = (0..8).map(|i| { let mx = if i < 4 { 0.02 + i as f32 * 0.1 + r.next() * 0.03 } else { 0.62 + (i - 4) as f32 * 0.1 + r.next() * 0.03 }; [mx, 0.035 + r.next() * 0.05, 0.3 + r.next() * 0.7, r.next()] }).collect();
        let rocks = (0..90).map(|_| { let v = four(&mut r); [v[0], v[1].powf(1.5), 0.5 + v[2], v[3]] }).collect();
        let dust = (0..260).map(|_| { let v = four(&mut r); [v[0], v[1], 0.4 + v[2] * 0.6, v[3]] }).collect();
        let mut hh = 0.5f32;
        let ridge = (0..=64).map(|_| { hh = (hh + (r.next() - 0.5) * 0.22).clamp(0.1, 1.0); hh }).collect();
        let devil = [r.next(), r.next()];
        Mem { r, stars, far, mesa, rocks, dust, devil, ridge, rov: HashMap::new(), lift: HashMap::new(), flash: HashMap::new(), go: Vec::new(), puffs: Vec::new(), pulses: Vec::new(), met: None, orb: HashMap::new(), rip: Vec::new(), prev: None, said: HashMap::new(), seen: 0.0 }
    }
}

/// What a scene sees: the hub's own `state.scene` when it sends one; else the same view made from the whole state
/// (when the scene is forced with --scene), with the same care: no names while the screen is shared.
pub fn view(st: &Value, local_secs: i64) -> Value {
    if get(st, &["scene"]).is_object() {
        return get(st, &["scene"]).clone();
    }
    let private = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
    let now = num(st, &["t"]).unwrap_or(0.0);
    let sod = (now as i64 + local_secs).rem_euclid(86400);
    let mut panels = serde_json::Map::new();
    for w in list(st, &["widgets"]) {
        let rows: Vec<Value> = list(w, &["data", "items"]).iter().map(|it| json!({ "name": if private { String::new() } else { text(it, &["title"]) }, "level": text(it, &["level"]) })).collect();
        if !rows.is_empty() {
            panels.insert(text(w, &["id"]), json!({ "rows": rows }));
        }
    }
    let mut v = st.clone();
    let hide = |a: &Value, keys: &[&str]| -> Value {
        let mut a = a.clone();
        if private {
            for k in keys {
                if let Some(o) = a.as_object_mut() { o.insert(k.to_string(), json!("")); }
            }
        }
        a
    };
    v["agents"] = Value::Array(list(st, &["agents"]).iter().map(|a| hide(a, &["name", "text"])).collect());
    v["git"] = Value::Array(list(st, &["git"]).iter().map(|a| hide(a, &["name", "branch"])).collect());
    v["time"] = json!({ "hour": sod / 3600, "minute": (sod / 60) % 60 });
    let launch_in = num(st, &["space", "launch", "net"]).map(|n| n - now);
    v["space"] = json!({ "sun": num(st, &["night", "sun", "el"]), "kp": num(st, &["space", "kp"]).unwrap_or(0.0), "launchIn": launch_in });
    v["panels"] = Value::Object(panels);
    v["events"] = json!([]);
    v
}

fn rows_of(sv: &Value) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    if let Some(p) = get(sv, &["panels"]).as_object() {
        for (id, w) in p {
            for (i, r) in list(w, &["rows"]).iter().enumerate() {
                let n = text(r, &["name"]);
                out.push((format!("{id}|{}", if n.is_empty() { i.to_string() } else { n }), r.clone()));
            }
        }
    }
    out
}

/// The rows of your panels that say how they are (a status board, services): pods on Mars, cubes on the lake.
fn pods(sv: &Value, max: usize) -> (Vec<(Value, String)>, usize) {
    let (mut rows, mut more) = (Vec::new(), 0);
    if let Some(p) = get(sv, &["panels"]).as_object() {
        for (id, w) in p {
            let rr = list(w, &["rows"]);
            if rr.iter().any(|r| !text(r, &["level"]).is_empty()) {
                for r in rr {
                    if rows.len() < max { rows.push((r.clone(), id.clone())) } else { more += 1 }
                }
            }
        }
    }
    (rows, more)
}

/// What just happened, as each scene's own update() says it: (the effect, its words). "say" is a line of text only.
pub fn update(kind: &str, mem: &mut Mem, sv: &Value) -> Vec<(String, String)> {
    let now = num(sv, &["t"]).unwrap_or(0.0);
    let mut out: Vec<(String, String)> = Vec::new();
    let Some(p) = mem.prev.take() else {
        mem.prev = Some(sv.clone());
        return out;
    };
    let mars = kind == "mars";
    let due = |said: &mut HashMap<String, f64>, key: &str, every: f64| {
        if now - said.get(key).copied().unwrap_or(0.0) < every { return false; }
        said.insert(key.to_string(), now);
        true
    };
    let waiting = |s: &Value| list(s, &["agents"]).iter().filter(|a| text(a, &["state"]) == "waiting").map(|a| text(a, &["name"])).collect::<Vec<_>>();
    let (w0, w1) = (waiting(&p), waiting(sv));
    if w1.len() > w0.len() && due(&mut mem.said, "wait", 30.0) {
        let who = w1.last().cloned().filter(|n| !n.is_empty()).unwrap_or_else(|| if mars { "a rover".into() } else { "an agent".into() });
        out.push(("chime".into(), format!("{who} needs you")));
    }
    if mars {
        let was: HashMap<String, String> = list(&p, &["git"]).iter().enumerate().map(|(i, r)| (key_of(r, i), text(r, &["ci"]))).collect();
        for (i, r) in list(sv, &["git"]).iter().enumerate() {
            let k = key_of(r, i);
            if text(r, &["ci"]) == "ok" && was.get(&k).map(|s| s.as_str()) == Some("fail") && due(&mut mem.said, &format!("green:{k}"), 60.0) {
                mem.go.push(k.clone());
                out.push(("say".into(), format!("{} is green · liftoff", if text(r, &["name"]).is_empty() { "build".to_string() } else { text(r, &["name"]) })));
            }
        }
    } else {
        let fails = |s: &Value| list(s, &["git"]).iter().filter(|r| text(r, &["ci"]) == "fail").count();
        if fails(sv) < fails(&p) && due(&mut mem.said, "green", 60.0) {
            out.push(("launch".into(), "build green".into()));
        }
    }
    let r0: HashMap<String, Value> = rows_of(&p).into_iter().collect();
    for (k, r) in rows_of(sv) {
        let Some(old) = r0.get(&k) else { continue };
        let (l0, l1) = (text(old, &["level"]), text(&r, &["level"]));
        let name = if text(&r, &["name"]).is_empty() { if mars { "a pod".to_string() } else { "a service".to_string() } } else { text(&r, &["name"]) };
        if l1 == "crit" && l0 != "crit" && due(&mut mem.said, "down", 60.0) {
            out.push((if mars { "chime" } else { "lightning" }.into(), format!("{name} is down")));
        }
        if l1 == "ok" && l0 == "crit" && due(&mut mem.said, "up", 60.0) {
            out.push((if mars { "say" } else { "shooting" }.into(), format!("{name} is back")));
        }
    }
    let (k0, k1) = (num(&p, &["space", "kp"]).unwrap_or(0.0), num(sv, &["space", "kp"]).unwrap_or(0.0));
    if k1 >= 6.0 && k0 < 6.0 && due(&mut mem.said, "kp", 3600.0) {
        out.push((if mars { "say" } else { "tip" }.into(), if mars { format!("solar storm · Kp {k1}") } else { format!("aurora tonight · Kp {k1}") }));
    }
    if let (Some(l0), Some(l1)) = (num(&p, &["space", "launchIn"]), num(sv, &["space", "launchIn"])) {
        if l1 <= 0.0 && l0 > 0.0 && due(&mut mem.said, "launch", 600.0) {
            if mars {
                mem.go.push("*".into());
                out.push(("say".into(), "a launch on Earth · one here too".into()));
            } else {
                out.push(("launch".into(), "liftoff".into()));
            }
        }
    }
    if num(&p, &["time", "hour"]) == Some(23.0) && num(sv, &["time", "hour"]) == Some(0.0) && due(&mut mem.said, "midnight", 3600.0) {
        if mars {
            mem.go.push("*".into());
            out.push(("say".into(), "a new sol".into()));
        } else {
            out.push(("fireworks".into(), String::new()));
        }
    }
    mem.prev = Some(sv.clone());
    out
}

fn key_of(r: &Value, i: usize) -> String {
    let n = text(r, &["name"]);
    if n.is_empty() { format!("#{i}") } else { n }
}

fn cl(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// The painter, in the scene's own coordinates (0,0 = the top left of the wallpaper).
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
    fn ell(&mut self, x: f32, y: f32, rx: f32, ry: f32, c: C, a: f32) { crate::draw::ellipse(self.win, self.ox + x, self.oy + y, rx, ry, c, a) }
    #[allow(clippy::too_many_arguments)]
    fn arcf(&mut self, x: f32, y: f32, rx: f32, ry: f32, rot: f32, a0: f32, a1: f32, c: C, a: f32) { crate::draw::arc_fill(self.win, self.ox + x, self.oy + y, rx, ry, rot, a0, a1, c, a) }
    #[allow(clippy::too_many_arguments)]
    fn arc(&mut self, x: f32, y: f32, rx: f32, ry: f32, a0: f32, a1: f32, lw: f32, c: C, a: f32) { crate::draw::arc(self.win, self.ox + x, self.oy + y, rx, ry, a0, a1, lw, c, a) }
    fn poly(&mut self, pts: &[(f32, f32)], c: C, a: f32) { let p: Vec<(f32, f32)> = pts.iter().map(|p| (p.0 + self.ox, p.1 + self.oy)).collect(); crate::draw::poly(self.win, &p, c, a) }
    fn line(&mut self, pts: &[(f32, f32)], lw: f32, c: C, a: f32) { let p: Vec<(f32, f32)> = pts.iter().map(|p| (p.0 + self.ox, p.1 + self.oy)).collect(); crate::draw::line(self.win, &p, lw, c, a) }
}

fn label(labels: &mut Vec<Label>, x: f32, y: f32, size: f32, c: C, a: f32, text: String, bold: bool) {
    if !text.is_empty() && a > 0.02 {
        labels.push(Label { x, y, size, c, a, text, bold });
    }
}

const RUST: C = c8(172, 86, 52);
const DEEP: C = c8(84, 40, 30);
const SAND: C = c8(222, 160, 108);
const HULL: C = c8(236, 230, 220);
const STEEL: C = c8(128, 134, 144);
const GLASS: C = c8(170, 226, 230);
const LAMP: C = c8(255, 232, 196);
const OK: C = c8(124, 227, 139);
const WARN: C = c8(255, 200, 87);
const CRIT: C = c8(255, 92, 80);
const TECH: C = c8(120, 220, 255);
const LEAF: C = c8(96, 190, 112);
const PANEL: C = c8(44, 74, 140);
const WHITE: C = [1.0, 1.0, 1.0];

/// mars: a colony on Mars, with its own sky and its own colours. The dome is your machine, the rovers your AI
/// agents, the launch pads your repos, the pods the rows of your panels; your weather arrives translated.
pub fn mars(win: &mut Window, sv: &Value, f: &Frame, mem: &mut Mem, t: f64, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, hz, u) = (f.w, f.h, f.hz, f.u);
    let mut g = G { win, ox: f.ox, oy: f.oy };
    let still = get(sv, &["ui", "motion"]).as_str() == Some("reduced");
    let tt = if still { 0.0 } else { t as f32 };
    let now = num(sv, &["t"]).unwrap_or(0.0);
    let cpu = num(sv, &["sys", "cpu"]).unwrap_or(0.0) as f32;
    let mut cores: Vec<f32> = list(sv, &["sys", "cores"]).iter().filter_map(|c| c.as_f64().map(|v| v as f32)).take(32).collect();
    if cores.is_empty() { cores.push(cpu) }
    let temp = num(sv, &["sys", "temp"]).unwrap_or(45.0) as f32;
    let quiet = if get(sv, &["meeting"]).as_bool().unwrap_or(false) || get(sv, &["away"]).as_bool().unwrap_or(false) { 0.55 } else { 1.0 };
    let hour = num(sv, &["time", "hour"]).unwrap_or(12.0) as f32;
    let cx = w / 2.0;
    // ---- the hour is yours: the Sun's real height decides day, the blue twilight, night
    let el = num(sv, &["space", "sun"]).map(|v| v as f32).unwrap_or(if (7.0..19.0).contains(&hour) { 35.0 } else { -30.0 });
    let (wd, wn) = (cl((el + 2.0) / 12.0), cl((-el - 4.0) / 10.0));
    let wt = (1.0 - wd - wn).max(0.0);
    let sky3 = |d: C, tw: C, n: C| -> C { [d[0] * wd + tw[0] * wt + n[0] * wn, d[1] * wd + tw[1] * wt + n[1] * wn, d[2] * wd + tw[2] * wt + n[2] * wn] };
    let (zen, hor) = (sky3(c8(118, 84, 62), c8(40, 46, 70), c8(6, 7, 13)), sky3(c8(216, 162, 112), c8(146, 122, 124), c8(34, 24, 27)));
    let (lr, lg, lb) = (wd + wt * 0.62 + wn * 0.27, wd + wt * 0.59 + wn * 0.29, wd + wt * 0.74 + wn * 0.46);
    let lit = |c: C, k: f32| -> C { [(c[0] * lr * k).min(1.0), (c[1] * lg * k).min(1.0), (c[2] * lb * k).min(1.0)] };
    let dark = 1.0 - wd;
    let ink = mix(HULL, lit(HULL, 1.2), 0.35);

    // ---- the sky
    g.grad(0.0, 0.0, w, hz * 0.7 + 1.0, zen, 1.0, mix(zen, hor, 0.7), 1.0);
    g.grad(0.0, hz * 0.7, w, hz * 0.3 + 1.0, mix(zen, hor, 0.7), 1.0, hor, 1.0);
    if wn > 0.05 {
        for s in &mem.stars {
            let tw = if still { 1.0 } else { 0.75 + 0.25 * (tt * (0.6 + s[2]) + s[3]).sin() };
            let z = (u * (0.7 + s[2] * 1.1)).max(1.0);
            g.rect(s[0] * w, s[1] * hz, z, z, c8(255, 244, 232), wn * s[2] * tw * (0.35 + 0.65 * (1.0 - s[1])));
        }
        let (ex, ey) = (w * 0.618, hz * 0.675); // the Earth: a blue star with a smaller one beside it
        g.glow(ex, ey, h * 0.012, c8(150, 200, 255), 0.8 * wn);
        g.disc(ex, ey, (u * 1.9).max(1.2), c8(210, 232, 255), wn);
        g.rect(ex + u * 7.0, ey - u * 3.0, (u * 1.2).max(1.0), (u * 1.2).max(1.0), c8(220, 220, 215), 0.8 * wn);
        label(labels, ex + u * 30.0, ey - u * 6.0, h * 0.0095, c8(200, 220, 245), 0.5 * wn, "Earth".into(), false);
        hits.push(Hit { x: ex, y: ey, r: h * 0.02, text: "Earth · the star you came from".into() });
        let ph = (tt * 0.0035).rem_euclid(1.0); // Phobos is fast, and rises in the west
        let (px, py) = (w * (0.3 + ph * 0.4), hz * (0.74 - 0.07 * (ph * PI).sin()));
        g.arcf(px, py, u * 5.5, u * 4.2, 0.5, 0.0, TAU, c8(196, 180, 164), wn);
        g.disc(px - u * 1.5, py - u, u * 1.4, c8(120, 108, 98), wn);
        g.disc(w * 0.41, hz * 0.69, (u * 1.6).max(1.2), c8(226, 214, 200), wn); // Deimos
        hits.push(Hit { x: px, y: py, r: h * 0.02, text: "Phobos · crosses the sky in four hours".into() });
        if !still {
            if mem.met.is_none() && mem.r.next() < 0.0015 {
                mem.met = Some((w * (0.3 + mem.r.next() * 0.4), hz * (0.1 + mem.r.next() * 0.3), t));
            }
            if let Some((mx, my, born)) = mem.met {
                let ma = ((t - born) / 0.7) as f32;
                if ma > 1.0 { mem.met = None } else {
                    let (x1, y1) = (mx + ma * h * 0.16, my + ma * h * 0.07);
                    g.line(&[(x1 - h * 0.06, y1 - h * 0.026), (x1, y1)], (u * 1.5).max(1.0), c8(255, 240, 220), 0.9 * wn * (1.0 - ma));
                }
            }
        }
    }
    let kp = num(sv, &["space", "kp"]).unwrap_or(0.0) as f32; // a solar storm: Mars has no shield, the whole night sky glows
    if kp >= 5.0 && wn > 0.3 {
        g.grad(0.0, hz * 0.25, w, hz * 0.75, c8(90, 220, 170), 0.0, c8(90, 220, 170), 0.10 * cl((kp - 4.0) / 4.0) * wn);
    }
    if el > -6.0 {
        // the Sun: small, and blue around it when it is low
        let hr = hour + num(sv, &["time", "minute"]).unwrap_or(0.0) as f32 / 60.0;
        let (sx, sy) = (w * (0.31 + 0.38 * cl((hr - 5.0) / 14.0)), hz - (el.max(-6.0) * 0.01745).sin() * hz * 0.82);
        let low = cl(1.0 - el / 22.0);
        g.glow(sx, sy, h * 0.30, c8(96, 150, 232), 0.55 * low * (1.0 - wn));
        g.glow(sx, sy, h * 0.10, c8(150, 200, 255), 0.5 * low * (1.0 - wn));
        g.glow(sx, sy, h * 0.07, c8(255, 240, 214), 0.65 * (1.0 - low * 0.5));
        g.disc(sx, sy, (h * 0.0085).max(2.0), c8(255, 250, 240), 1.0);
    }
    let cloud = cl(num(sv, &["weather", "cloud"]).unwrap_or(0.0) as f32 / 100.0); // your clouds, as Mars has them: thin streaks of ice
    if cloud > 0.1 {
        for d in mem.dust.iter().take((3.0 + cloud * 7.0).round() as usize) {
            let ccx = ((d[0] + tt * 0.0012 * d[2]).rem_euclid(1.3) - 0.15) * w;
            g.arcf(ccx, hz * (0.12 + d[1] * 0.42), w * (0.07 + d[3] * 0.08), h * (0.004 + d[2] * 0.005), -0.05, 0.0, TAU, mix(hor, WHITE, 0.55), 0.05 + 0.07 * cloud);
        }
    }

    // ---- the land: the far volcano in the haze, the buttes, the plain
    let mut far: Vec<(f32, f32)> = vec![(0.0, hz + 1.0)];
    far.extend((0..=96).map(|i| (w * i as f32 / 96.0, hz - mem.far[i] * h * 0.2)));
    far.push((w, hz + 1.0));
    g.poly(&far, mix(lit(RUST, 0.9), hor, 0.62), 1.0);
    let rim: Vec<(f32, f32)> = (14..=50).map(|i| (w * i as f32 / 96.0, hz - mem.far[i] * h * 0.2)).collect();
    g.line(&rim, u, mix(hor, WHITE, 0.3), 0.25 * (wd + wt));
    for m in &mem.mesa {
        let (x0, mw, mh) = (m[0] * w, m[1] * w, h * (0.018 + 0.04 * m[2]));
        let sl = mw * 0.12;
        g.poly(&[(x0 - sl, hz + 1.0), (x0 + sl * 0.4, hz - mh), (x0 + mw - sl * 0.4, hz - mh * (0.85 + 0.15 * m[3])), (x0 + mw + sl, hz + 1.0)], mix(lit(RUST, 0.72), hor, 0.3), 1.0);
        g.poly(&[(x0 + mw * 0.55, hz + 1.0), (x0 + mw * 0.6, hz - mh * 0.9), (x0 + mw - sl * 0.4, hz - mh * (0.85 + 0.15 * m[3])), (x0 + mw + sl, hz + 1.0)], mix(lit(DEEP, 1.0), hor, 0.3), 0.55);
    }
    let wind = num(sv, &["weather", "wind"]).unwrap_or(0.0) as f32; // your wind raises dust devils, far away
    let n_dev = if still { 0 } else if wind > 35.0 { 2 } else if wind > 14.0 { 1 } else { 0 };
    for i in 0..n_dev {
        let dvx = (mem.devil[i] + tt * 0.004 * if i > 0 { -1.0 } else { 1.0 }).rem_euclid(1.0);
        let (dx0, dh) = (w * (0.28 + 0.44 * dvx), h * (0.07 + 0.03 * i as f32));
        for k in 0..9 {
            let kk = k as f32 / 8.0;
            let wob = (tt * 2.2 + k as f32 * 0.8 + i as f32 * 3.0).sin() * h * 0.006 * kk;
            g.ell(dx0 + wob, hz - kk * dh, h * (0.004 + 0.012 * kk), h * 0.007, mix(lit(SAND, 1.0), hor, 0.4), 0.16 * (1.0 - kk * 0.6));
        }
    }
    g.grad(0.0, hz, w, (h - hz) * 0.3 + 1.0, mix(lit(RUST, 1.05), hor, 0.28), 1.0, lit(RUST, 0.86), 1.0);
    g.grad(0.0, hz + (h - hz) * 0.3, w, (h - hz) * 0.7, lit(RUST, 0.86), 1.0, lit(DEEP, 0.75), 1.0);
    for i in 1..12 {
        // the wind's ripples in the sand
        let (ry, amp) = (hz + (h - hz) * (i as f32 / 12.0).powf(1.7), h * 0.004 * (0.4 + i as f32 / 12.0));
        let pts: Vec<(f32, f32)> = (0..=24).map(|rx| (w * rx as f32 / 24.0, ry + (rx as f32 * 0.9 + i as f32 * 1.7).sin() * amp)).collect();
        g.line(&pts, u, lit(DEEP, 1.0), 0.16);
    }
    for rk in mem.rocks.iter().take(60) {
        let (rd, rxx) = (rk[1], rk[0] * w);
        let (ryy, rs) = (hz + h * 0.012 + rd * (h - hz) * 0.95, h * (0.0025 + 0.011 * rd) * rk[2]);
        if (rxx - cx).abs() < h * 0.2 && rd < 0.12 { continue }
        g.ell(rxx + rs * 0.5, ryy + rs * 0.1, rs * 1.5, rs * 0.3, lit(DEEP, 0.55), 0.35);
        g.poly(&[(rxx - rs, ryy), (rxx - rs * 0.5, ryy - rs * (0.6 + rk[3] * 0.5)), (rxx + rs * 0.5, ryy - rs * 0.75), (rxx + rs, ryy)], lit(DEEP, 1.15), 1.0);
        g.poly(&[(rxx - rs, ryy), (rxx - rs * 0.5, ryy - rs * (0.6 + rk[3] * 0.5)), (rxx - rs * 0.1, ryy)], lit(RUST, 1.2), 0.7);
    }

    // ---- the colony: where things stand
    let (r, y0) = (h * 0.104, hz + h * 0.02);
    let wall_h = r * 0.17;
    let heat = cl((temp - 40.0) / 50.0);
    let core = if heat < 0.5 { mix(TECH, WARN, heat * 2.0) } else { mix(WARN, CRIT, (heat - 0.5) * 2.0) };
    let load = (cpu / 100.0).max(0.08) * quiet;
    let music = get(sv, &["music"]).as_bool().unwrap_or(false) && !still;

    // ---- the launch pads: your repos. Crates for the week's commits, a beacon in the colour of the CI
    let repos: Vec<&Value> = list(sv, &["git"]).iter().take(6).collect();
    for e in list(sv, &["events"]) {
        let et = num(e, &["t"]).unwrap_or(0.0);
        if et <= mem.seen { continue }
        mem.seen = et;
        let k = text(e, &["kind"]);
        if k == "commit" || k == "push" {
            let words = text(e, &["text"]);
            let name = repos.iter().map(|x| text(x, &["name"])).find(|n| !n.is_empty() && words.starts_with(n.as_str())).unwrap_or_else(|| "#0".into());
            mem.flash.insert(name, t);
        }
    }
    for k in std::mem::take(&mut mem.go) {
        if k == "*" { if let Some(r0) = repos.first() { mem.lift.insert(key_of(r0, 0), t); } } else { mem.lift.insert(k, t); }
    }
    for (i, rp) in repos.iter().enumerate() {
        let key = key_of(rp, i);
        let (side, slot) = (if i % 2 == 1 { 1.0 } else { -1.0 }, (i / 2) as f32);
        let (px, py, rh, rw) = (cx + side * (r * 2.45 + slot * r * 0.78), hz + h * 0.007, h * 0.058, h * 0.0105);
        let ci = text(rp, &["ci"]);
        let col = match ci.as_str() { "fail" => CRIT, "run" => WARN, "ok" => OK, _ => TECH };
        let run = if ci == "run" && !still { 0.55 + 0.45 * (tt * 5.0 + i as f32).sin() } else { 1.0 };
        let fk = mem.flash.get(&key).map(|fl| cl(1.0 - ((t - fl) / 2.5) as f32)).unwrap_or(0.0);
        let lt = mem.lift.get(&key).map(|l| (t - l) as f32).unwrap_or(-1.0);
        g.ell(px, py, r * 0.3, h * 0.0045, lit(DEEP, 0.7), 0.9); // the pad
        g.line(&[(px - rw * 2.2, py), (px - rw * 2.2, py - rh * 1.12)], u * 1.4, lit(STEEL, 1.0), 1.0); // the tower
        for a in 0..4 {
            g.line(&[(px - rw * 2.2, py - rh * (0.2 + a as f32 * 0.25)), (px - rw * 1.1, py - rh * (0.3 + a as f32 * 0.25))], u * 0.8, lit(STEEL, 1.0), 1.0);
        }
        g.glow(px - rw * 2.2, py - rh * 1.14, h * 0.011 * (1.0 + fk), col, (0.55 + 0.4 * dark) * run + fk * 0.4);
        g.rect(px - rw * 2.2 - (u * 1.3).max(1.0), py - rh * 1.16, (u * 2.6).max(2.0), (u * 2.6).max(2.0), col, run);
        let away = (0.0..26.0).contains(&lt);
        let (mut ry2, mut alpha) = (py, 1.0f32);
        if away {
            if lt < 8.0 { ry2 = py - h * 0.011 * lt * lt } else if lt < 22.0 { alpha = 0.0 } else { alpha = (lt - 22.0) / 4.0 }
        }
        if alpha > 0.0 && ry2 > -rh {
            if away && lt < 8.0 && !still {
                // its fire, and the dust it lifts
                let fh = rh * (0.7 + 0.25 * (tt * 40.0).sin());
                g.poly(&[(px - rw * 0.45, ry2), (px + rw * 0.45, ry2), (px, ry2 + fh)], c8(255, 190, 90), 0.8);
                g.poly(&[(px - rw * 0.22, ry2), (px + rw * 0.22, ry2), (px, ry2 + fh * 0.5)], c8(255, 250, 230), 0.95);
                g.glow(px, ry2 + rh * 0.1, rh * 1.3, c8(255, 190, 110), 0.6);
                for p2 in 0..7 {
                    let (pa, pr) = (cl(1.0 - lt / 6.0), r * (0.12 + 0.07 * p2 as f32) * (0.5 + lt * 0.35));
                    g.ell(px + (p2 as f32 - 3.0) * r * 0.11 * (1.0 + lt * 0.3), py - h * 0.003, pr, pr * 0.45, mix(lit(SAND, 1.0), WHITE, 0.2), 0.20 * pa);
                }
                g.grad(px - rw * 0.3, ry2 + fh * 0.5, rw * 0.6, (py - ry2 - fh * 0.5).max(0.0), c8(255, 244, 230), 0.5, c8(255, 244, 230), 0.0);
            }
            g.rect(px - rw / 2.0, ry2 - rh * 0.8, rw / 2.0, rh * 0.8, lit(HULL, 0.82), alpha);
            g.rect(px, ry2 - rh * 0.8, rw / 2.0, rh * 0.8, lit(HULL, 1.05), alpha);
            g.poly(&[(px - rw / 2.0, ry2 - rh * 0.8), (px - rw * 0.25, ry2 - rh * 0.96), (px, ry2 - rh * 1.02), (px + rw * 0.25, ry2 - rh * 0.96), (px + rw / 2.0, ry2 - rh * 0.8)], lit(HULL, 1.05), alpha);
            g.rect(px - rw / 2.0, ry2 - rh * 0.5, rw, (rh * 0.05).max(1.0), lit(RUST, 1.1), alpha);
            g.poly(&[(px - rw / 2.0, ry2 - rh * 0.16), (px - rw * 1.05, ry2), (px - rw / 2.0, ry2)], lit(STEEL, 0.7), alpha);
            g.poly(&[(px + rw / 2.0, ry2 - rh * 0.16), (px + rw * 1.05, ry2), (px + rw / 2.0, ry2)], lit(STEEL, 0.7), alpha);
        }
        if ci == "run" && !still {
            g.ell(px + rw * 1.4 + (tt * 1.5 + i as f32).sin() * rw, py - rh * 0.1 - (tt * 0.3 + i as f32).rem_euclid(1.0) * rh * 0.3, rw * 1.1, rw * 0.7, WHITE, 0.18); // it is fuelling
        }
        let week = num(rp, &["week"]).unwrap_or(0.0) as f32;
        let (nb, bs) = (((week / 4.0).ceil().min(8.0) * quiet) as usize, (h * 0.0055).max(2.0)); // crates: the week's commits
        for b in 0..nb {
            g.rect(px + rw * 2.0 + (b % 2) as f32 * (bs + 1.0), py - ((b / 2) as f32 + 1.0) * (bs + 1.0), bs, bs, lit(if b % 3 > 0 { SAND } else { STEEL }, 0.9), 1.0);
        }
        let dirty = num(rp, &["dirty"]).unwrap_or(0.0);
        if dirty > 0.0 { g.rect(px + rw * 2.0, py + u * 3.0, (u * 3.0).max(2.0), (u * 3.0).max(2.0), WARN, 0.95) }
        let name = text(rp, &["name"]);
        label(labels, px, py + h * 0.010, h * 0.0098, ink, 0.7 * quiet, name.chars().take(12).collect(), false);
        hits.push(Hit { x: px, y: py - rh * 0.5, r: h * 0.035, text: format!("{} · {week:.0} commits this week{}{}", if name.is_empty() { "a repo".into() } else { name }, if ci.is_empty() { String::new() } else { format!(" · CI {ci}") }, if dirty > 0.0 { format!(" · {dirty:.0} changed") } else { String::new() }) });
    }

    // ---- the tanks: memory and disk, as full as they are
    let (ram, disk) = (cl(num(sv, &["sys", "ram"]).unwrap_or(0.0) as f32 / 100.0), cl(num(sv, &["sys", "disk"]).unwrap_or(0.0) as f32 / 100.0));
    for (x, frac, col, name) in [(cx - r * 1.72, ram, TECH, "RAM"), (cx - r * 1.38, disk, c8(255, 170, 96), "DISK")] {
        let (tw, th, yb) = (r * 0.24, r * 0.78, y0 + wall_h);
        let fc = if frac > 0.9 { CRIT } else { col };
        g.rect(x - tw / 2.0, yb - th, tw / 2.0, th, lit(HULL, 0.78), 1.0);
        g.rect(x, yb - th, tw / 2.0, th, lit(HULL, 1.02), 1.0);
        g.arcf(x, yb - th, tw / 2.0, tw * 0.28, 0.0, PI, TAU, lit(HULL, 1.02), 1.0);
        g.rect(x - tw * 0.12, yb - th * 0.92, tw * 0.24, th * 0.84, lit(DEEP, 0.6), 1.0);
        g.rect(x - tw * 0.12, yb - th * 0.08 - th * 0.84 * frac, tw * 0.24, th * 0.84 * frac, mix(lit(fc, 0.8), fc, 0.4 + 0.6 * dark), 0.95 * quiet);
        g.glow(x, yb - th * 0.08 - th * 0.42 * frac, tw * 1.4, fc, 0.3 * dark * quiet);
        label(labels, x, yb - th - tw * 0.5 - h * 0.012, h * 0.0085, ink, 0.6, name.into(), false);
        hits.push(Hit { x, y: yb - th / 2.0, r: th * 0.6, text: format!("{name} {:.0}%", frac * 100.0) });
    }

    // ---- the antenna: it talks to orbit as fast as your network
    let net = cl(((num(sv, &["sys", "rx"]).unwrap_or(0.0) + num(sv, &["sys", "tx"]).unwrap_or(0.0)) / 4e6) as f32);
    let (ax, ay, ah) = (cx + r * 1.5, y0 + wall_h, r * 1.05);
    g.line(&[(ax, ay), (ax, ay - ah)], u * 1.6, lit(STEEL, 1.0), 1.0);
    g.line(&[(ax - r * 0.12, ay), (ax, ay - ah * 0.3), (ax + r * 0.12, ay)], u * 1.6, lit(STEEL, 1.0), 1.0);
    g.arcf(ax, ay - ah, r * 0.2, r * 0.2, -0.6, 0.2, PI + 0.2, lit(HULL, 1.0), 1.0);
    g.line(&[(ax, ay - ah), (ax + r * 0.14, ay - ah - r * 0.2)], u * 1.6, lit(STEEL, 1.0), 1.0);
    if !still && mem.r.next() < 0.01 + net * 0.12 && mem.pulses.len() < 8 { mem.pulses.push(t) }
    mem.pulses.retain(|b0| t - b0 <= 1.8);
    for b0 in &mem.pulses {
        let age = ((t - b0) / 1.8) as f32;
        g.arc(ax + r * 0.14, ay - ah - r * 0.2, r * (0.1 + age * 0.9), r * (0.1 + age * 0.9), -1.35, -0.55, u * 1.4, TECH, 0.7 * (1.0 - age) * quiet);
    }
    hits.push(Hit { x: ax, y: ay - ah * 0.7, r: r * 0.4, text: format!("network {:.0}% of 4 MB/s", net * 100.0) });

    // ---- the dome: your machine
    let top = y0 - r * 0.84;
    g.glow(cx, y0 - r * 0.3, r * (1.7 + 0.8 * load), mix(LAMP, core, 0.5), (0.10 + 0.34 * dark) * (0.5 + 0.5 * load) * quiet);
    g.arcf(cx, y0, r, r * 0.84, 0.0, PI, TAU, mix(mix(lit(GLASS, 0.5), zen, 0.5), mix(lit(GLASS, 0.75), LAMP, 0.25 * dark), 0.5), 0.93);
    g.glow(cx, y0 - r * 0.22, r * (0.5 + 0.5 * load) * 0.8, core, (0.35 + 0.5 * load) * quiet); // the reactor's light, inside
    g.rect(cx - r * 0.07, y0 - r * 0.52, r * 0.14, r * 0.52, mix(lit(STEEL, 0.8), core, 0.25), 1.0);
    g.rect(cx - r * 0.035, y0 - r * 0.47, r * 0.07, r * 0.36, core, (0.75 + if still { 0.0 } else { 0.2 * (tt * (1.5 + 4.0 * load)).sin() }) * quiet);
    for i in 0..6 {
        // the greenhouse: it is greener when the machine is calm
        let gx = cx + if i < 3 { -1.0 } else { 1.0 } * r * (0.22 + (i % 3) as f32 * 0.2);
        let ghh = r * (0.1 + 0.07 * ((i * 7) % 3) as f32) * (1.1 - heat * 0.4);
        g.ell(gx, y0 - ghh * 0.5, r * 0.075, ghh * 0.75, mix(lit(LEAF, 0.7), LEAF, 0.35 * dark), 0.95);
        g.rect(gx - r * 0.08, y0 - r * 0.03, r * 0.16, r * 0.03, lit(DEEP, 0.9), 1.0);
    }
    g.arc(cx, y0, r, r * 0.84, PI, TAU, u * 1.3, lit(HULL, 1.1), 0.75); // the glass and its ribs
    for fr in [0.36, 0.72] { g.arc(cx, y0, r * fr, r * 0.84, PI, TAU, u, lit(HULL, 1.1), 0.32) }
    for fr in [0.45f32, 0.78] {
        let (yy2, hw) = (y0 - r * 0.84 * fr, r * (1.0 - fr * fr).sqrt());
        g.arc(cx, yy2, hw, hw * 0.1, 0.0, PI, u, lit(HULL, 1.1), 0.32);
    }
    g.arc(cx - r * 0.12, y0, r * 0.72, r * 0.70, PI * 1.12, PI * 1.42, u * 3.0, WHITE, 0.16 * (wd + wt * 0.6)); // the Sun on the glass
    g.rect(cx - r * 1.05, y0, r * 1.05, wall_h, lit(HULL, 0.8), 1.0); // its wall
    g.rect(cx, y0, r * 1.05, wall_h, lit(HULL, 1.02), 1.0);
    let n = cores.len();
    let cw = r * 1.9 / n as f32;
    for (i, cv) in cores.iter().enumerate() {
        // a light for each core: the busier, the brighter; red above 90%
        let v = cl(cv / 100.0);
        let eq = if music { 0.3 * (tt * (3.0 + i as f32 * 0.7) + i as f32).sin().abs() } else { 0.0 };
        let lv = (v * quiet + eq).min(1.0);
        let c1 = if v > 0.9 { CRIT } else { mix(TECH, core, v) };
        let lx = cx - r * 0.95 + i as f32 * cw;
        g.rect(lx + cw * 0.14, y0 + wall_h * 0.28, cw * 0.72, wall_h * 0.44, mix(lit(DEEP, 0.7), c1, 0.15 + 0.85 * lv), 1.0);
        if lv > 0.4 { g.glow(lx + cw / 2.0, y0 + wall_h * 0.5, cw * 1.6, c1, 0.35 * lv * (0.4 + 0.6 * dark)) }
    }
    g.ell(cx + r * 0.2, y0 + wall_h + h * 0.002, r * 1.25, h * 0.005, lit(DEEP, 0.6), 0.5);
    if let (Some(a), Some(b)) = (num(sv, &["focus", "start"]), num(sv, &["focus", "until"])) {
        if b > a {
            // a focus session: a shield closes over the dome as the time goes
            let fp = cl(((now - a) / (b - a)) as f32);
            g.arc(cx, y0, r * 1.22, r * 1.06, PI, TAU, (u * 2.0).max(1.5), OK, 0.25);
            g.arc(cx, y0, r * 1.22, r * 1.06, PI, PI + fp * PI, (u * 3.5).max(2.0), OK, 0.95);
        }
    }
    hits.push(Hit { x: cx, y: y0 - r * 0.35, r: r * 0.75, text: format!("CPU {cpu:.0}% · {temp:.0}° · RAM {:.0}% · disk {:.0}%", ram * 100.0, disk * 100.0) });
    let _ = top;

    // ---- the pods: the rows of your panels (VMs, containers, hosts), a window each in the colour of its state
    let (rows, more) = pods(sv, 12);
    let py0 = y0 + wall_h + h * 0.056;
    if !rows.is_empty() {
        // the tubes people walk in
        for sd in [-1.0f32, 1.0] {
            let cnt = (((rows.len() as f32 - if sd > 0.0 { 1.0 } else { 0.0 }) / 2.0).ceil()).min(6.0);
            if cnt < 1.0 { continue }
            g.line(&[(cx + sd * r * 0.5, y0 + wall_h), (cx + sd * r * 0.5, py0), (cx + sd * (r * 0.72 + (cnt - 1.0) * r * 0.5), py0)], (u * 2.5).max(1.5), lit(HULL, 0.7), 0.8);
        }
    }
    for (i, (row, id)) in rows.iter().enumerate() {
        let (side, k) = (if i % 2 == 1 { 1.0 } else { -1.0 }, i / 2);
        let (c2, r2, lv) = ((k % 6) as f32, (k / 6) as f32, text(row, &["level"]));
        let (x, y, pw, ph) = (cx + side * (r * 0.72 + c2 * r * 0.5), py0 + r2 * h * 0.04, r * 0.19, r * 0.11);
        let col = match lv.as_str() { "crit" => CRIT, "warn" => WARN, "ok" => OK, _ => TECH };
        let bl = if lv == "crit" && !still { 0.5 + 0.5 * (tt * 6.0 + i as f32).sin() } else { 1.0 };
        g.ell(x + pw * 0.2, y + ph * 1.05, pw * 1.2, ph * 0.3, lit(DEEP, 0.55), 0.45);
        g.ell(x, y, pw, ph, lit(HULL, 0.95), 1.0);
        g.rect(x - pw * 0.8, y, pw * 1.6, ph, lit(HULL, 0.95), 1.0);
        g.rect(x - pw * 0.8, y + ph * 0.55, pw * 1.6, ph * 0.45, lit(HULL, 0.72), 1.0);
        g.rect(x - pw * 0.45, y - ph * 0.3, pw * 0.9, ph * 0.5, mix(lit(col, 0.85), col, 0.4 + 0.6 * dark), bl * quiet);
        g.glow(x, y, pw * (1.3 + if lv == "crit" { 0.8 } else { 0.0 }), col, (0.15 + 0.4 * dark) * bl * quiet);
        let name = text(row, &["name"]);
        hits.push(Hit { x, y, r: pw * 1.1, text: format!("{} · {}", if name.is_empty() { id.clone() } else { name }, if lv.is_empty() { "no state".into() } else { lv }) });
    }

    // ---- the rovers: your AI agents
    let agents: Vec<&Value> = list(sv, &["agents"]).iter().take(10).collect();
    let base = py0 + if rows.is_empty() { 0.0 } else { h * 0.046 };
    mem.puffs.retain(|p| t - p.2 <= 1.6);
    for p in &mem.puffs {
        let age = ((t - p.2) / 1.6) as f32;
        g.ell(p.0, p.1 - age * p.3 * 1.2, p.3 * (0.6 + age * 1.6), p.3 * (0.4 + age), mix(lit(SAND, 1.0), WHITE, 0.12), 0.26 * (1.0 - age));
    }
    for (i, ag) in agents.iter().enumerate() {
        let key = key_of(ag, i);
        let o = mem.rov.entry(key).or_insert((0.33 + ((i as f32 * 0.618 + 0.2) % 1.0) * 0.34, if i % 2 == 1 { 1.0 } else { -1.0 }));
        let lane = (i % 3) as f32;
        let sc = 0.85 + lane * 0.14;
        let (y, s, stt) = (base + lane * h * 0.022 + (i / 3) as f32 * h * 0.006, h * 0.0165 * sc, text(ag, &["state"]));
        let spd = match stt.as_str() { "working" => 0.00055, "idle" => 0.00012, _ => 0.0 };
        if !still {
            o.0 += o.1 * spd * sc;
            if o.0 > 0.69 { o.1 = -1.0 }
            if o.0 < 0.31 { o.1 = 1.0 }
        }
        let (x, d) = (w * o.0, o.1);
        let col = match stt.as_str() { "working" => TECH, "waiting" => WARN, "done" => OK, "failed" => CRIT, _ => mix(HULL, DEEP, 0.4) };
        let bump = if spd > 0.0 && !still { (tt * 9.0 + i as f32).sin() * s * 0.05 } else { 0.0 };
        if stt == "working" && !still && mem.r.next() < 0.25 && mem.puffs.len() < 60 { mem.puffs.push((x - d * s * 1.2, y, t, s * 0.5)) }
        g.ell(x + s * 0.25, y + s * 0.3, s * 1.5, s * 0.22, lit(DEEP, 0.5), 0.5);
        if stt == "working" && dark > 0.3 {
            g.poly(&[(x + d * s, y - s * 0.7), (x + d * s * 5.0, y - s * 1.1), (x + d * s * 5.0, y + s * 0.2), (x + d * s, y - s * 0.4)], LAMP, 0.16 * dark); // its headlights
        }
        let yb = y + bump;
        for wv in -1..=1 { g.disc(x + wv as f32 * s * 0.82, yb, s * 0.32, lit(c8(52, 48, 50), 1.0), 1.0) }
        g.rect(x - s, yb - s * 0.95, s * 2.0, s * 0.55, lit(HULL, 0.95), 1.0);
        g.rect(x - s, yb - s * 0.55, s * 2.0, s * 0.15, lit(HULL, 0.7), 1.0);
        g.poly(&[(x - s * 1.05, yb - s * 1.02), (x + s * 0.6, yb - s * 1.02), (x + s * 0.75, yb - s * 1.2), (x - s * 0.9, yb - s * 1.2)], lit(PANEL, 1.1), 1.0);
        g.line(&[(x + d * s * 0.75, yb - s * 0.95), (x + d * s * 0.75, yb - s * 1.95)], (s * 0.12).max(1.0), lit(STEEL, 1.0), 1.0);
        g.rect(x + d * s * 0.75 - s * 0.22, yb - s * 2.15, s * 0.44, s * 0.26, lit(HULL, 1.0), 1.0);
        let blink = if stt == "waiting" && !still { 0.5 + 0.5 * (tt * 6.0).sin() } else { 1.0 };
        g.rect(x + d * s * 0.85 - s * 0.08, yb - s * 2.1, s * 0.16, s * 0.16, col, blink);
        g.glow(x + d * s * 0.8, y - s * 2.02, s * if stt == "waiting" { 2.6 } else { 1.6 }, col, (0.35 + 0.45 * dark) * blink * quiet);
        if stt == "failed" && !still {
            for sm in 0..3 {
                let sa = (tt * 0.4 + sm as f32 / 3.0).rem_euclid(1.0);
                g.disc(x + (sa * 5.0 + sm as f32).sin() * s * 0.3, y - s * 1.2 - sa * s * 2.6, s * (0.25 + sa * 0.5), lit(c8(70, 64, 64), 1.0), 0.3 * (1.0 - sa));
            }
        }
        if stt == "waiting" { label(labels, x + d * s * 0.8, y - s * 3.6, h * 0.017, WARN, 1.0, "!".into(), true) }
        if stt == "done" {
            g.line(&[(x + d * s * 0.8 - s * 0.4, y - s * 2.75), (x + d * s * 0.8 - s * 0.1, y - s * 2.5), (x + d * s * 0.8 + s * 0.45, y - s * 3.05)], (s * 0.16).max(1.5), OK, 0.95);
        }
        let name = text(ag, &["name"]);
        label(labels, x, y + s * 0.3 + h * 0.004, h * 0.0105, ink, 0.8 * quiet, name.chars().take(16).collect(), false);
        let what = text(ag, &["text"]);
        hits.push(Hit { x, y: y - s, r: s * 2.4, text: format!("{} · {stt}{}", if name.is_empty() { "a rover".into() } else { name }, if what.is_empty() { String::new() } else { format!(" · {}", what.chars().take(40).collect::<String>()) }) });
    }

    // ---- your weather, as Mars would have it: rain is a dust storm, snow is frost falling
    let code = num(sv, &["weather", "code"]).unwrap_or(0.0) as i32;
    let rain = num(sv, &["weather", "rain"]).unwrap_or(0.0) as f32;
    let snow = (71..=77).contains(&code) || code == 85 || code == 86;
    let storm = if snow { 0.0 } else { (cl(rain / 4.0) * 0.8 + if rain > 0.0 { 0.2 } else { 0.0 }).max(if code >= 95 { 1.0 } else { 0.0 }).max(cl((wind - 45.0) / 60.0)) };
    if storm > 0.02 {
        g.rect(0.0, 0.0, w, h, mix(lit(SAND, 1.0), hor, 0.5), 0.34 * storm);
        let nst = if still { 0 } else { (130.0 * storm).round() as usize };
        for dd in mem.dust.iter().take(nst) {
            let dxx = ((dd[0] + tt * (0.25 + 0.3 * dd[2])).rem_euclid(1.1) - 0.05) * w;
            let dyy = (dd[1] * 0.9 + 0.08 + (tt * 0.7 + dd[3] * 6.0).sin() * 0.01) * h;
            let len = h * 0.03 * dd[2];
            g.rect(dxx, dyy, len, (u * 1.2).max(1.0), mix(lit(SAND, 1.15), WHITE, 0.2), 0.3);
        }
    }
    if snow && !still {
        for f2 in mem.dust.iter().take(120) {
            let z2 = (u * (1.0 + f2[2] * 1.4)).max(1.0);
            g.rect((f2[0] + (tt * 0.4 + f2[3] * 6.0).sin() * 0.01).rem_euclid(1.0) * w, (f2[1] + tt * 0.03 * f2[2]).rem_euclid(1.0) * h, z2, z2, c8(235, 240, 250), 0.75);
        }
    }
    // ---- what it is: one line under the dome
    let mut what = format!("{cpu:.0}% · {temp:.0}° · {} cores", cores.len());
    if !agents.is_empty() { what += &format!(" · {} rovers", agents.len()) }
    if !rows.is_empty() { what += &format!(" · {} pods", rows.len() + more) }
    if storm > 0.3 { what += " · dust storm" }
    label(labels, cx, y0 + wall_h + h * 0.014, h * 0.0125, ink, 0.78 * quiet, what, false);
}

/// monolith: a crystal the size of a tower floats over a mirror lake: your machine. A band for each CPU core, two
/// rings for the memory and the disk, shards for your AI agents, pylons for your repos, cubes for your services.
#[allow(clippy::too_many_arguments)]
pub fn monolith(win: &mut Window, sv: &Value, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, hz, u) = (f.w, f.h, f.hz, f.u);
    let mut g = G { win, ox: f.ox, oy: f.oy };
    let (bg, fg, ac, a2, ok, warn, crit) = (pal.bg, pal.fg, pal.accent, pal.accent2, pal.ok, pal.warn, pal.crit);
    let black = [0.0, 0.0, 0.0];
    let still = get(sv, &["ui", "motion"]).as_str() == Some("reduced");
    let tt = if still { 0.0 } else { t as f32 };
    let now = num(sv, &["t"]).unwrap_or(0.0);
    let cpu = num(sv, &["sys", "cpu"]).unwrap_or(0.0) as f32;
    let mut cores: Vec<f32> = list(sv, &["sys", "cores"]).iter().filter_map(|c| c.as_f64().map(|v| v as f32)).take(32).collect();
    if cores.is_empty() { cores.push(cpu) }
    let temp = num(sv, &["sys", "temp"]).unwrap_or(45.0) as f32;
    let quiet = if get(sv, &["meeting"]).as_bool().unwrap_or(false) || get(sv, &["away"]).as_bool().unwrap_or(false) { 0.55 } else { 1.0 };
    let night_k = num(sv, &["space", "sun"]).map(|s| cl((6.0 - s as f32) / 14.0)).unwrap_or(1.0);
    let cx = w / 2.0;

    // ---- the aurora, when the Kp says there is one
    let kp = num(sv, &["space", "kp"]).unwrap_or(0.0) as f32;
    if kp >= 4.0 && night_k > 0.3 {
        for ab in 0..3 {
            let abf = ab as f32;
            let mut pts: Vec<(f32, f32)> = (0..=40).map(|ax| (w * (0.22 + 0.56 * ax as f32 / 40.0), hz * (0.2 + abf * 0.07) + (ax as f32 * 0.35 + tt * 0.25 + abf * 2.0).sin() * h * 0.03)).collect();
            pts.extend((0..=40).rev().map(|ax| (w * (0.22 + 0.56 * ax as f32 / 40.0), hz * (0.2 + abf * 0.07) + h * 0.1 + (ax as f32 * 0.3 + tt * 0.2 + abf).sin() * h * 0.02)));
            g.poly(&pts, if ab == 1 { a2 } else { ok }, 0.05 * ((kp - 3.0) / 4.0).min(1.0) * night_k);
        }
    }

    // ---- the shore: a far ridge, and the lake
    let mut ridge: Vec<(f32, f32)> = vec![(0.0, hz)];
    ridge.extend((0..=64).map(|i| (w * i as f32 / 64.0, hz - h * 0.012 - mem.ridge[i] * h * 0.035 * (1.0 - 0.8 * (-((i as f32 - 32.0) / 9.0).powi(2)).exp()))));
    ridge.push((w, hz));
    g.poly(&ridge, mix(bg, a2, 0.14), 0.96);
    g.grad(0.0, hz, w, (h - hz) * 0.35 + 1.0, mix(bg, ac, 0.20), 0.97, mix(bg, a2, 0.08), 0.98);
    g.grad(0.0, hz + (h - hz) * 0.35, w, (h - hz) * 0.65, mix(bg, a2, 0.08), 0.98, mix(bg, black, 0.55), 1.0);
    g.rect(0.0, hz - u, w, (u * 1.5).max(1.0), ac, 0.55); // the water line
    for i in 1..14 {
        g.rect(0.0, hz + (h - hz) * (i as f32 / 14.0).powf(1.8), w, u.max(1.0), fg, 0.05); // long glints, in perspective
    }

    // ---- the monolith: where it floats, how it breathes
    let (mh, mw, bob) = (h * 0.235, h * 0.07, (tt * 0.5).sin() * h * 0.005);
    let top = hz - h * 0.035 - mh + bob;
    let mid = top + mh / 2.0;
    let heat = cl((temp - 40.0) / 50.0);
    let core = if heat < 0.5 { mix(ac, warn, heat * 2.0) } else { mix(warn, crit, (heat - 0.5) * 2.0) };
    let load = (cpu / 100.0).max(0.08) * quiet;
    let music = get(sv, &["music"]).as_bool().unwrap_or(false) && !still;
    let mono = |g: &mut G, alpha: f32, flip: bool| {
        // drawn twice: itself, and its image in the water
        let fy = |y: f32| if flip { 2.0 * hz - y } else { y };
        let (n, cap) = (cores.len(), mw * 0.75);
        let bh = (mh - cap * 2.0) / n as f32;
        g.poly(&[(cx, fy(top)), (cx + mw / 2.0, fy(top + cap)), (cx, fy(top + cap))], mix(bg, ac, 0.30), alpha);
        g.poly(&[(cx, fy(top)), (cx - mw / 2.0, fy(top + cap)), (cx, fy(top + cap))], mix(bg, ac, 0.16), alpha);
        g.poly(&[(cx, fy(top + mh)), (cx + mw / 2.0, fy(top + mh - cap)), (cx, fy(top + mh - cap))], mix(bg, ac, 0.24), alpha);
        g.poly(&[(cx, fy(top + mh)), (cx - mw / 2.0, fy(top + mh - cap)), (cx, fy(top + mh - cap))], mix(bg, ac, 0.12), alpha);
        for (k, cv) in cores.iter().enumerate() {
            // a band for each core, from the top: the busier, the brighter
            let v = cl(cv / 100.0);
            let eq = if music { 0.25 * (tt * (3.0 + k as f32 * 0.7) + k as f32).sin().abs() } else { 0.0 };
            let lv = (v * quiet + eq).min(1.0);
            let c1 = if v > 0.9 { crit } else { mix(ac, core, v) };
            let ya = top + cap + k as f32 * bh;
            let bhh = bh - u.max(1.0);
            let y0 = if flip { fy(ya) - bhh } else { ya };
            g.rect(cx - mw / 2.0, y0, mw / 2.0, bhh, mix(mix(bg, ac, 0.10), c1, 0.12 + 0.55 * lv), alpha); // the shaded side
            g.rect(cx, y0, mw / 2.0, bhh, mix(mix(bg, ac, 0.20), c1, 0.20 + 0.75 * lv), alpha); // the lit side
            if lv > 0.25 { g.rect(cx + mw * 0.06, y0 + bh * 0.25, mw * 0.36 * lv, (bh * 0.14).max(1.0), fg, 0.25 * lv * alpha) }
        }
        let (ea, eb) = (fy(top + cap * 0.3), fy(top + mh - cap * 0.3));
        g.rect(cx - (u * 0.75).max(0.5), ea.min(eb), (u * 1.5).max(1.0), (eb - ea).abs(), fg, 0.5 * alpha); // its edge, toward you
    };
    g.glow(cx, hz + h * 0.012, h * (0.10 + 0.22 * load), core, 0.30 + 0.5 * load); // the light it drops on the water
    mono(&mut g, 0.30, true);
    g.poly(&[(cx - u * 2.0, top + mh), (cx + u * 2.0, top + mh), (cx + mw * 0.3, hz), (cx - mw * 0.3, hz)], core, 0.28 * load + 0.03); // the beam under it

    // ---- the pylons: your repos on the shore, a beam as tall as the week's commits, in the colour of its CI
    let repos: Vec<&Value> = list(sv, &["git"]).iter().take(10).collect();
    for e in list(sv, &["events"]) {
        let et = num(e, &["t"]).unwrap_or(0.0);
        if et <= mem.seen { continue }
        mem.seen = et;
        let k = text(e, &["kind"]);
        if k == "commit" || k == "push" {
            let words = text(e, &["text"]);
            let name = repos.iter().map(|x| text(x, &["name"])).find(|n| !n.is_empty() && words.starts_with(n.as_str())).unwrap_or_else(|| "#0".into());
            mem.flash.insert(name, t);
        }
    }
    for (i, rp) in repos.iter().enumerate() {
        let (side, slot) = (if i % 2 == 1 { 1.0 } else { -1.0 }, (i / 2) as f32);
        let (px, ph, py) = (cx + side * (w * 0.085 + slot * w * 0.032), h * 0.05, hz);
        let ci = text(rp, &["ci"]);
        let col = match ci.as_str() { "fail" => crit, "run" => warn, "ok" => ok, _ => ac };
        let week = num(rp, &["week"]).unwrap_or(0.0) as f32;
        let bhh = h * (0.03 + 0.22 * (week / 30.0).min(1.0)) * quiet;
        let fk = mem.flash.get(&key_of(rp, i)).map(|fl| cl(1.0 - ((t - fl) / 2.5) as f32)).unwrap_or(0.0);
        let run = if ci == "run" && !still { 0.6 + 0.4 * (tt * 5.0 + i as f32).sin() } else { 1.0 };
        g.poly(&[(px - u * 5.0, py), (px - u * 1.5, py - ph), (px + u * 1.5, py - ph), (px + u * 5.0, py)], mix(bg, fg, 0.28), 1.0);
        let bw = (u * (2.0 + 4.0 * fk)).max(2.0);
        g.grad(px - bw / 2.0, py - ph - bhh * (1.0 + fk * 0.6), bw, bhh * (1.0 + fk * 0.6), col, 0.0, col, (0.55 + 0.45 * fk) * run);
        g.glow(px, py - ph, h * 0.012 * (1.0 + fk), col, 0.9 * run);
        let dirty = num(rp, &["dirty"]).unwrap_or(0.0);
        if dirty > 0.0 { g.rect(px + u * 5.0, py - ph * 0.5, (u * 3.0).max(2.0), (u * 3.0).max(2.0), warn, 0.9) }
        g.rect(px - u, py, (u * 2.0).max(1.0), bhh * 0.5, col, 0.16); // its beam on the water
        let name = text(rp, &["name"]);
        hits.push(Hit { x: px, y: py - ph, r: h * 0.03, text: format!("{} · {week:.0} commits this week{}{}", if name.is_empty() { "a repo".into() } else { name }, if ci.is_empty() { String::new() } else { format!(" · CI {ci}") }, if dirty > 0.0 { format!(" · {dirty:.0} changed") } else { String::new() }) });
    }

    // ---- the swarm: the rows of your panels (VMs, containers, hosts), a cube each, in the colour of its state
    let (rows, _) = pods(sv, 28);
    for (i, (row, id)) in rows.iter().enumerate() {
        let (side, k) = (if i % 2 == 1 { 1.0 } else { -1.0 }, i / 2);
        let (col2, row2, lv) = ((k % 7) as f32, (k / 7) as f32, text(row, &["level"]));
        let (x, y, s) = (cx + side * (w * 0.07 + col2 * h * 0.03), hz + h * 0.078 + row2 * h * 0.034 + (tt * 0.9 + i as f32).sin() * h * 0.003, h * 0.011);
        let col = match lv.as_str() { "crit" => crit, "warn" => warn, "ok" => ok, _ => ac };
        let bl = if lv == "crit" && !still { 0.5 + 0.5 * (tt * 6.0 + i as f32).sin() } else { 1.0 };
        g.rect(x - s, y - s, s * 2.0, s * 2.0, mix(bg, col, 0.55), 0.95 * bl);
        g.poly(&[(x - s, y - s), (x - s * 0.4, y - s * 1.6), (x + s * 1.6, y - s * 1.6), (x + s, y - s)], mix(bg, col, 0.85), 0.95 * bl);
        g.poly(&[(x + s, y - s), (x + s * 1.6, y - s * 1.6), (x + s * 1.6, y + s * 0.4), (x + s, y + s)], mix(bg, col, 0.30), 0.95 * bl);
        g.glow(x, y + s * 2.2, s * 2.2, col, 0.25 * bl);
        let name = text(row, &["name"]);
        hits.push(Hit { x, y, r: s * 2.0, text: format!("{} · {}", if name.is_empty() { id.clone() } else { name }, if lv.is_empty() { "no state".into() } else { lv }) });
    }

    // ---- the lake moves with the network; rain is Pulse's, the rings are ours
    let net = (((num(sv, &["sys", "rx"]).unwrap_or(0.0) + num(sv, &["sys", "tx"]).unwrap_or(0.0)) / 4e6) as f32).min(1.0);
    if !still && mem.r.next() < 0.02 + net * 0.25 {
        let (rx, ry) = (mem.r.next(), mem.r.next());
        mem.rip.push((cx + (rx - 0.5) * w * 0.4, hz + ry.powf(1.6) * (h - hz) * 0.8, t, 0.4 + net));
    }
    mem.rip.retain(|r| t - r.2 <= 2.5);
    for r in &mem.rip {
        let age = (t - r.2) as f32;
        let d = (r.1 - hz) / (h - hz);
        let rr = age * h * 0.05 * r.3 * (0.3 + d);
        g.arc(r.0, r.1, rr, rr * 0.22, 0.0, TAU, u, ac, 0.28 * (1.0 - age / 2.5));
    }

    // ---- the rings: memory and disk, turning; behind the monolith first, in front after
    let ring = |g: &mut G, frac: f32, rx: f32, col: C, spd: f32, front_part: bool| {
        let (ry, a0, y) = (rx * 0.2, tt * spd, mid + bob * 0.5);
        let lw = (u * 2.4).max(1.5);
        if front_part { g.arc(cx, y, rx, ry, 0.0, PI, lw, col, 0.18) } else { g.arc(cx, y, rx, ry, PI, TAU, lw, col, 0.18) }
        let a1 = a0 + frac * TAU; // the lit arc, cut into the half we draw now
        for s in 0..24 {
            let (p0, p1) = (a0 + (a1 - a0) * s as f32 / 24.0, a0 + (a1 - a0) * (s + 1) as f32 / 24.0);
            if (((p0 + p1) / 2.0).sin() > 0.0) == front_part { g.arc(cx, y, rx, ry, p0, p1, lw, col, 0.95) }
        }
    };
    let (ram, disk) = (num(sv, &["sys", "ram"]).unwrap_or(0.0) as f32 / 100.0, num(sv, &["sys", "disk"]).unwrap_or(0.0) as f32 / 100.0);
    ring(&mut g, ram, mw * 1.25, a2, 0.25, false);
    ring(&mut g, disk, mw * 1.7, ac, -0.16, false);

    // ---- the shards behind it, the monolith, the shards in front
    let agents: Vec<&Value> = list(sv, &["agents"]).iter().take(12).collect();
    let mut shards: Vec<(usize, f32, f32, f32, f32, f32)> = Vec::new(); // index, x, y, z, angle, rx
    for (i, ag) in agents.iter().enumerate() {
        let state = text(ag, &["state"]);
        let o = mem.orb.entry(key_of(ag, i)).or_insert((i as f32 * 2.1, 1.0));
        let spd = match state.as_str() { "working" => 0.9, "waiting" => 0.12, "done" => 0.3, "failed" => 0.0, "idle" => 0.18, _ => 0.2 };
        if !still { o.0 += spd * 0.012 }
        let want = match state.as_str() { "waiting" => 1.5, "failed" => 1.25, _ => 1.0 };
        o.1 += (want - o.1) * 0.03;
        let rx = mw * (2.3 + (i % 3) as f32 * 0.45) * o.1;
        let (ry, z, x) = (rx * 0.24, o.0.sin(), cx + o.0.cos() * rx);
        let y = if state == "failed" { hz + h * 0.02 + (i % 3) as f32 * h * 0.01 } else { mid + z * ry + (tt * 1.3 + i as f32).sin() * h * 0.004 };
        shards.push((i, x, y, if state == "failed" { 1.0 } else { z }, o.0, rx));
    }
    let shard = |g: &mut G, s: &(usize, f32, f32, f32, f32, f32), labels: &mut Vec<Label>, hits: &mut Vec<Hit>| {
        let ag = agents[s.0];
        let state = text(ag, &["state"]);
        let col = match state.as_str() { "working" => ac, "waiting" => warn, "done" => ok, "failed" => crit, _ => mix(fg, bg, 0.45) };
        let zz = (s.3 + 1.0) / 2.0;
        let sz = h * 0.012 * (0.8 + 0.35 * zz) * if state == "waiting" { 1.5 + if still { 0.0 } else { 0.25 * (tt * 5.0).sin() } } else { 1.0 };
        let dim = (0.55 + 0.45 * zz) * quiet;
        if state == "working" && !still { g.arc(cx, mid, s.5, s.5 * 0.24, s.4 - 0.5, s.4, (u * 1.5).max(1.0), col, 0.35 * dim) } // a trail along its orbit
        g.glow(s.1, s.2, sz * 3.0, col, 0.5 * dim);
        let (sn, cs) = if state == "failed" { 0.9f32.sin_cos() } else { (0.0, 1.0) };
        let p = |x: f32, y: f32| (s.1 + x * cs - y * sn, s.2 + x * sn + y * cs);
        g.poly(&[p(0.0, -sz * 1.6), p(sz * 0.8, 0.0), p(0.0, sz * 1.6)], mix(bg, col, 0.9), dim);
        g.poly(&[p(0.0, -sz * 1.6), p(-sz * 0.8, 0.0), p(0.0, sz * 1.6)], mix(bg, col, 0.55), dim);
        if state == "waiting" { label(labels, s.1, s.2 - sz * 3.4, h * 0.016, warn, 1.0, "!".into(), true) }
        let name = text(ag, &["name"]);
        if s.3 > -0.2 { label(labels, s.1, s.2 + sz * 2.4, h * 0.0105, fg, 0.6 * dim, name.chars().take(16).collect(), false) }
        let what = text(ag, &["text"]);
        hits.push(Hit { x: s.1, y: s.2, r: sz * 3.0, text: format!("{} · {state}{}", if name.is_empty() { "an agent".into() } else { name }, if what.is_empty() { String::new() } else { format!(" · {}", what.chars().take(40).collect::<String>()) }) });
    };
    for s in shards.iter().filter(|s| s.3 <= 0.0) { shard(&mut g, s, labels, hits) }
    g.glow(cx, mid, mh * (0.55 + 0.25 * load), core, (0.16 + 0.3 * load) * (0.7 + 0.3 * night_k)); // its heart's light
    mono(&mut g, 1.0, false);
    g.glow(cx, mid, mw * 0.9, core, 0.35 * load + if still { 0.0 } else { 0.08 * (tt * 2.2).sin() });
    ring(&mut g, ram, mw * 1.25, a2, 0.25, true);
    ring(&mut g, disk, mw * 1.7, ac, -0.16, true);
    for s in shards.iter().filter(|s| s.3 > 0.0) { shard(&mut g, s, labels, hits) }

    // ---- a focus session: a shield closes around it as the time goes
    if let (Some(a), Some(b)) = (num(sv, &["focus", "start"]), num(sv, &["focus", "until"])) {
        if b > a {
            let fp = cl(((now - a) / (b - a)) as f32);
            g.arc(cx, mid, mw * 2.1, mh * 0.62, 0.0, TAU, (u * 2.0).max(1.5), ok, 0.25);
            g.arc(cx, mid, mw * 2.1, mh * 0.62, -PI / 2.0, -PI / 2.0 + fp * TAU, (u * 3.5).max(2.0), ok, 0.95);
        }
    }
    // ---- motes of light rising off the water: more of them, the busier the machine
    let nd = if still { 0 } else { (110.0 * (0.35 + 0.65 * load)) as usize };
    for d in mem.dust.iter().take(nd) {
        let dy = (d[1] + tt * 0.012 * d[2]).rem_euclid(1.0);
        let dx = cx + (d[0] - 0.5) * w * 0.5 + (tt * 0.3 + d[3] * 6.28).sin() * h * 0.01;
        let sz2 = (u * (0.8 + d[2] * 1.4)).max(1.0);
        g.rect(dx, hz + h * 0.02 - dy * h * 0.42, sz2, sz2, mix(fg, core, 0.5), 0.55 * (dy * PI).sin() * (0.3 + 0.5 * d[2]));
    }
    // ---- what it is: one line under it
    let mut what = format!("{cpu:.0}% · {temp:.0}° · {} cores", cores.len());
    if !agents.is_empty() { what += &format!(" · {} agents", agents.len()) }
    if !rows.is_empty() { what += &format!(" · {} services", rows.len()) }
    label(labels, cx, hz + h * 0.022, h * 0.0125, fg, 0.55 * quiet, what, false);
    hits.push(Hit { x: cx, y: mid, r: mh * 0.5, text: format!("CPU {cpu:.0}% · {temp:.0}° · RAM {:.0}% · disk {:.0}%", ram * 100.0, disk * 100.0) });
}
