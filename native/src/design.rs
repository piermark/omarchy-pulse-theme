//! The engine of the scenes made with the scene editor, and of the community's `.pulse` scenes: Pulse 1.7's
//! scene-design.js, in Rust. A design is plain data (shapes, their colours, their motion, which live data moves them,
//! and the actions that fire when something happens); this draws it. Not yet here: clicks on shapes, and a tint on
//! pictures.

use crate::palette::{col, hex, mix, Pal, C};
use crate::scene::{disc, rect};
use gpui::{point, px, PathBuilder, Window};
use serde_json::Value;
use std::collections::HashMap;

const TAU: f32 = std::f32::consts::TAU;

/// What a scene remembers between frames: animations being played, flashes, what an action showed or hid.
#[derive(Default)]
pub struct Mem {
    pub played: HashMap<String, f64>,
    pub flash: HashMap<String, (String, f64)>,
    pub vis: HashMap<String, bool>,
    every: HashMap<String, i64>,
    last: HashMap<String, f64>,
    prev: Option<Value>,
    /// what a click can find, made new at every frame: x, y, half width, half height, the shape's click text, its id
    pub hits: Vec<(f32, f32, f32, f32, String, String)>,
}

/// A line of text of the design, for the element tree to write (centred on x, y).
pub struct Words {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub bold: bool,
    pub c: C,
    pub a: f32,
    pub text: String,
}

/// A picture of the design, for the element tree to show (centred on x, y).
pub struct Picture {
    pub asset: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub a: f32,
    /// "tint": the picture's shape filled with this colour (logos and icons follow the theme)
    pub tint: Option<C>,
}

/// An effect an action asks Pulse for.
pub struct Ask {
    pub kind: String,
    pub text: String,
}

fn n(v: &Value, k: &str, d: f32) -> f32 {
    v.get(k).and_then(Value::as_f64).map(|x| x as f32).filter(|x| x.is_finite()).unwrap_or(d)
}
fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k).and_then(Value::as_array).map(|a| a.as_slice()).unwrap_or(&[])
}
fn count(st: &Value, state: &str) -> f32 {
    arr(st, "agents").iter().filter(|a| s(a, "state") == state).count() as f32
}
fn commits(st: &Value) -> f32 {
    arr(st, "git").iter().map(|r| n(r, "week", 0.0)).sum()
}

/// One copy of a repeated shape: its own value, its state, its name.
#[derive(Clone, Default)]
pub struct Item {
    v: f32,
    state: String,
    name: String,
    i: usize,
}

/// The live value a name reads, and the range that becomes 0..1 (SOURCES of scene-design.js). `st` is the scene's
/// read-only state (state.scene of the hub).
fn source(src: &str, st: &Value, it: Option<&Item>) -> Option<(Option<f32>, f32, f32)> {
    let sys = |k: &str| st.get("sys").and_then(|y| y.get(k)).and_then(Value::as_f64).map(|x| x as f32);
    let wx = |k: &str| st.get("weather").and_then(|y| y.get(k)).and_then(Value::as_f64).map(|x| x as f32);
    let tm = |k: &str| st.get("time").and_then(|y| y.get(k)).and_then(Value::as_f64).map(|x| x as f32);
    let sp = |k: &str| st.get("space").and_then(|y| y.get(k)).and_then(Value::as_f64).map(|x| x as f32);
    let flag = |k: &str| Some(if st.get(k).map(|v| !v.is_null() && v.as_bool() != Some(false)).unwrap_or(false) { 1.0 } else { 0.0 });
    Some(match src {
        "cpu" => (sys("cpu"), 0.0, 100.0),
        "ram" => (sys("ram"), 0.0, 100.0),
        "gpu" => (sys("gpu"), 0.0, 100.0),
        "temp" => (sys("temp"), 30.0, 95.0),
        "disk" => (sys("disk"), 0.0, 100.0),
        "net" => (Some((sys("rx").unwrap_or(0.0) + sys("tx").unwrap_or(0.0)) / 1e6), 0.0, 10.0),
        "hour" => (Some(tm("hour").unwrap_or(0.0) + tm("minute").unwrap_or(0.0) / 60.0), 0.0, 24.0),
        "minute" => (tm("minute"), 0.0, 60.0),
        "second" => (tm("second"), 0.0, 60.0),
        "weekday" => (tm("weekday"), 0.0, 6.0),
        "outside" => (wx("temp"), -10.0, 35.0),
        "rain" => (wx("rain"), 0.0, 5.0),
        "wind" => (wx("wind"), 0.0, 60.0),
        "cloud" => (wx("cloud"), 0.0, 100.0),
        "daylight" => (wx("day"), 0.0, 1.0),
        "sun" => (sp("sun"), -30.0, 70.0),
        "moon" => (sp("moon"), 0.0, 100.0),
        "planes" => (sp("planes"), 0.0, 10.0),
        "iss" => (sp("iss"), 0.0, 90.0),
        "starlink" => (sp("starlink"), 0.0, 20.0),
        "kp" => (sp("kp"), 0.0, 9.0),
        "launch" => (Some(sp("launchIn").map(|m| m / 60.0).unwrap_or(60.0)), 0.0, 60.0),
        "agents" => (Some(arr(st, "agents").len() as f32), 0.0, 8.0),
        "working" => (Some(count(st, "working")), 0.0, 8.0),
        "waiting" => (Some(count(st, "waiting")), 0.0, 4.0),
        "failed" => (Some(count(st, "failed")), 0.0, 4.0),
        "commits" => (Some(commits(st)), 0.0, 60.0),
        "cifail" => (Some(arr(st, "git").iter().filter(|r| s(r, "ci") == "fail").count() as f32), 0.0, 3.0),
        "music" => (flag("music"), 0.0, 1.0),
        "focus" => (flag("focus"), 0.0, 1.0),
        "meeting" => (flag("meeting"), 0.0, 1.0),
        "away" => (flag("away"), 0.0, 1.0),
        "item" => (Some(it.map(|i| i.v).unwrap_or(0.0)), 0.0, 1.0),
        _ => {
            let id = src.strip_prefix("panel:")?;
            let p = st.get("panels").and_then(|p| p.get(id));
            let v = p.and_then(|p| p.get("value")).and_then(Value::as_f64).map(|x| x as f32);
            let (lo, hi) = match p {
                Some(p) if p.get("min").and_then(Value::as_f64).is_some() && n(p, "max", 0.0) > n(p, "min", 0.0) => (n(p, "min", 0.0), n(p, "max", 100.0)),
                _ => (0.0, 100.0),
            };
            (v, lo, hi)
        }
    })
}

fn norm(src: &str, st: &Value, it: Option<&Item>, b: &Value) -> f32 {
    let Some((v, mut lo, mut hi)) = source(src, st, it) else { return 0.0 };
    if let (Some(l), Some(h)) = (b.get("lo").and_then(Value::as_f64), b.get("hi").and_then(Value::as_f64)) {
        if h != l {
            lo = l as f32;
            hi = h as f32;
        }
    }
    ((v.unwrap_or(lo) - lo) / if hi - lo == 0.0 { 1.0 } else { hi - lo }).clamp(0.0, 1.0)
}

fn items(by: &str, r: &Value, st: &Value) -> Vec<Option<Item>> {
    let private = st.get("privacy").and_then(Value::as_bool).unwrap_or(false);
    let name = |x: &str| if private { String::new() } else { x.to_string() };
    let mut out: Vec<Option<Item>> = Vec::new();
    match by {
        "agents" => arr(st, "agents").iter().take(24).for_each(|a| {
            let state = s(a, "state");
            out.push(Some(Item { v: match state { "working" => 1.0, "waiting" => 0.6, "done" => 0.3, _ => 0.0 }, state: state.into(), name: name(s(a, "name")), i: 0 }))
        }),
        "repos" => arr(st, "git").iter().take(16).for_each(|g| {
            let state = match s(g, "ci") { "fail" => "failed", "run" => "working", "ok" => "done", _ => "idle" };
            out.push(Some(Item { v: (n(g, "week", 0.0) / 30.0).clamp(0.0, 1.0), state: state.into(), name: name(s(g, "name")), i: 0 }))
        }),
        "cores" => st.get("sys").map(|y| arr(y, "cores")).unwrap_or(&[]).iter().take(32).enumerate().for_each(|(i, c)| {
            let v = c.as_f64().unwrap_or(0.0) as f32;
            out.push(Some(Item { v: (v / 100.0).clamp(0.0, 1.0), state: if v > 85.0 { "failed" } else { "working" }.into(), name: format!("core {i}"), i: 0 }))
        }),
        "planes" => st.get("space").map(|y| arr(y, "list")).unwrap_or(&[]).iter().take(12).for_each(|p| out.push(Some(Item { v: (n(p, "el", 0.0) / 90.0).clamp(0.0, 1.0), state: "working".into(), name: name(s(p, "flight")), i: 0 }))),
        "panel" => {
            let rows = st.get("panels").and_then(|p| p.get(s(r, "panel"))).map(|p| arr(p, "rows")).unwrap_or(&[]);
            let mx = rows.iter().filter_map(|w| w.get("value").and_then(Value::as_f64)).fold(0.0f64, f64::max) as f32;
            rows.iter().take(32).for_each(|w| {
                let lv = s(w, "level");
                let v = match w.get("value").and_then(Value::as_f64) {
                    Some(x) => if mx > 0.0 { (x as f32 / mx).clamp(0.0, 1.0) } else { 0.0 },
                    None => match lv { "ok" => 1.0, "info" => 0.8, "warn" => 0.6, "crit" => 0.3, _ => 0.5 },
                };
                out.push(Some(Item { v, state: if lv.is_empty() { "idle" } else { lv }.into(), name: name(s(w, "name")), i: 0 }))
            })
        }
        "count" => {
            let k = (n(r, "n", 3.0) as usize).clamp(1, 64);
            (0..k).for_each(|i| out.push(Some(Item { v: if k > 1 { i as f32 / (k - 1) as f32 } else { 0.0 }, ..Default::default() })))
        }
        _ => out.push(None),
    }
    for (i, it) in out.iter_mut().enumerate() {
        if let Some(it) = it {
            it.i = i;
        }
    }
    out
}

/// A text with its live values filled in: {cpu}, {time}, {panel:ID}, and in a repeated shape {name}, {state}, {item}.
pub fn fill(tpl: &str, st: &Value, it: Option<&Item>) -> String {
    let g = |src: &str| source(src, st, it).and_then(|x| x.0);
    let tm = |k: &str| st.get("time").and_then(|y| y.get(k)).and_then(Value::as_f64).unwrap_or(0.0) as i64;
    let mut out = String::new();
    let mut rest = tpl;
    while let Some(a) = rest.find('{') {
        out.push_str(&rest[..a]);
        let Some(b) = rest[a..].find('}') else { break };
        let key = &rest[a + 1..a + b];
        let val = match key {
            "cpu" | "ram" | "gpu" | "temp" | "agents" | "working" | "waiting" | "commits" | "planes" => Some(format!("{:.0}", g(key).unwrap_or(0.0))),
            "net" => Some(format!("{:.1}", g("net").unwrap_or(0.0))),
            "time" => Some(format!("{:02}:{:02}", tm("hour"), tm("minute"))),
            "hour" => Some(tm("hour").to_string()),
            "date" => Some(format!("{:02}/{:02}", tm("day").max(1), tm("month").max(1))),
            "outside" | "kp" | "moon" => Some(g(key).map(|v| if key == "kp" { format!("{v}") } else { format!("{v:.0}") }).unwrap_or_else(|| "–".into())),
            "name" => Some(it.map(|i| i.name.clone()).unwrap_or_default()),
            "state" => Some(it.map(|i| i.state.clone()).unwrap_or_default()),
            "item" => Some(it.map(|i| format!("{:.0}", i.v * 100.0)).unwrap_or_default()),
            k if k.starts_with("panel:") => {
                let p = st.get("panels").and_then(|p| p.get(&k[6..]));
                Some(match p.and_then(|p| p.get("value")).and_then(Value::as_f64) {
                    Some(v) => format!("{}{}", (v * 10.0).round() / 10.0, p.map(|p| s(p, "unit")).unwrap_or("")),
                    None => "–".into(),
                })
            }
            _ => None,
        };
        match val {
            Some(v) => out.push_str(&v),
            None => out.push_str(&rest[a..=a + b]),
        }
        rest = &rest[a + b + 1..];
    }
    out.push_str(rest);
    out
}

fn theme(name: &str, pal: &Pal, design: &Value) -> C {
    if let Some(c) = name.strip_prefix('#').and_then(|_| hex(name)) {
        return c;
    }
    if let Some(c) = design.get("palette").and_then(|p| p.get(name)).and_then(Value::as_str).and_then(hex) {
        return c;
    }
    match name {
        "bg" => pal.bg,
        "fg" => pal.fg,
        "accent" => pal.accent,
        "accent2" => pal.accent2,
        "ok" => pal.ok,
        "warn" => pal.warn,
        "crit" => pal.crit,
        "dim" => pal.dim,
        _ => [0.53, 0.53, 0.53],
    }
}

struct P {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    alpha: f32,
    rot: f32,
    glow: f32,
    heat: f32,
    speed: f32,
    level: f32,
    count: f32,
    amp: f32,
}

fn keyed(sh: &Value, t: f64, mem: &mut Mem) -> Option<[f32; 5]> {
    let k = sh.get("keys")?;
    let mut fr: Vec<&Value> = arr(k, "frames").iter().collect();
    if fr.is_empty() {
        return None;
    }
    let dur = n(k, "dur", 2.0).max(0.1) as f64;
    let looping = k.get("loop").and_then(Value::as_bool);
    let mut ph;
    if s(k, "start") == "action" {
        let at = *mem.played.get(s(sh, "id"))?;
        ph = (t - at) / dur;
        if ph >= 1.0 {
            if looping == Some(true) {
                ph %= 1.0;
            } else {
                mem.played.remove(s(sh, "id"));
                return None;
            }
        }
    } else {
        ph = if looping == Some(false) { (t / dur).min(1.0) } else { (t / dur) % 1.0 };
    }
    let ph = ph as f32;
    fr.sort_by(|a, b| n(a, "t", 0.0).total_cmp(&n(b, "t", 0.0)));
    let (mut a, mut b) = (fr[0], fr[fr.len() - 1]);
    for i in 0..fr.len() - 1 {
        if ph >= n(fr[i], "t", 0.0) && ph <= n(fr[i + 1], "t", 1.0) {
            a = fr[i];
            b = fr[i + 1];
            break;
        }
    }
    let span = n(b, "t", 1.0) - n(a, "t", 0.0);
    let mut u = if span > 0.0 { ((ph - n(a, "t", 0.0)) / span).clamp(0.0, 1.0) } else { 0.0 };
    if s(k, "ease") == "smooth" {
        u = u * u * (3.0 - 2.0 * u);
    }
    let l = |key: &str, d: f32| n(a, key, d) + (n(b, key, d) - n(a, key, d)) * u;
    Some([l("dx", 0.0), l("dy", 0.0), l("scale", 1.0), l("alpha", 1.0), l("rot", 0.0)])
}

fn resolve(sh: &Value, st: &Value, it: Option<&Item>, t: f64, still: bool, mem: &mut Mem) -> P {
    let anim = sh.get("anim").unwrap_or(&Value::Null);
    let mut p = P { x: n(sh, "x", 0.5), y: n(sh, "y", 0.5), w: n(sh, "w", 0.1), h: n(sh, "h", 0.1), alpha: n(sh, "alpha", 1.0), rot: n(sh, "rot", 0.0), glow: n(sh, "glow", 0.0), heat: 0.0, speed: n(anim, "speed", 1.0), level: n(sh, "level", 0.6), count: n(sh, "count", 40.0), amp: n(sh, "amp", 0.3) };
    for b in arr(sh, "bind") {
        if source(s(b, "src"), st, it).is_none() {
            continue;
        }
        let v = n(b, "from", 0.0) + (n(b, "to", 1.0) - n(b, "from", 0.0)) * norm(s(b, "src"), st, it, b);
        match s(b, "prop") {
            "x" => p.x = v,
            "y" => p.y = v,
            "w" => p.w = v,
            "h" => p.h = v,
            "alpha" => p.alpha = v,
            "rot" => p.rot = v,
            "glow" => p.glow = v,
            "heat" => p.heat = v,
            "speed" => p.speed = v,
            "level" => p.level = v,
            "count" => p.count = v,
            "amp" => p.amp = v,
            _ => {}
        }
    }
    let (tf, ph) = (t as f32, it.map(|i| i.i as f32 * 0.7).unwrap_or(0.0));
    if !still {
        match s(anim, "kind") {
            "pulse" => {
                let k = 1.0 + 0.12 * (tf * TAU * 0.5 * p.speed + ph).sin();
                p.w *= k;
                p.h *= k;
            }
            "spin" => p.rot += tf * 90.0 * p.speed,
            "bob" => p.y += (tf * TAU * 0.4 * p.speed + ph).sin() * 0.012,
            "sway" => p.rot += (tf * TAU * 0.3 * p.speed + ph).sin() * 8.0,
            "blink" => p.alpha *= if (tf * p.speed + ph * 0.1).rem_euclid(1.0) < 0.6 { 1.0 } else { 0.15 },
            "breathe" => p.alpha *= 0.55 + 0.45 * (tf * TAU * 0.25 * p.speed + ph).sin(),
            "drift" => p.x = (p.x + tf * 0.02 * p.speed).rem_euclid(1.2) - 0.1,
            _ => {}
        }
        if let Some(k) = keyed(sh, t, mem) {
            p.x += k[0];
            p.y += k[1];
            p.w *= k[2];
            p.h *= k[2];
            p.alpha *= k[3];
            p.rot += k[4];
        }
    }
    p
}

/// A closed or open line through points given around the shape's centre, turned by `rot` degrees.
fn poly(win: &mut Window, cx: f32, cy: f32, rot: f32, pts: &[(f32, f32)], fill: bool, close: bool, lw: f32, c: C, a: f32) {
    if pts.len() < 2 || a <= 0.004 {
        return;
    }
    let (sn, cs) = rot.to_radians().sin_cos();
    let at = |p: &(f32, f32)| point(px(cx + p.0 * cs - p.1 * sn), px(cy + p.0 * sn + p.1 * cs));
    let mut b = if fill { PathBuilder::fill() } else { PathBuilder::stroke(px(lw)) };
    b.move_to(at(&pts[0]));
    for p in &pts[1..] {
        b.line_to(at(p));
    }
    if close {
        b.close();
    }
    if let Ok(path) = b.build() {
        win.paint_path(path, col(c, a));
    }
}

/// Draws the design for the time `t`; returns its words and its pictures for the element tree.
#[allow(clippy::too_many_arguments)]
pub fn paint(win: &mut Window, design: &Value, st: &Value, pal: &Pal, mem: &mut Mem, t: f64, ox: f32, oy: f32, w: f32, h: f32) -> (Vec<Words>, Vec<Picture>) {
    let (mut words, mut pics) = (Vec::new(), Vec::new());
    let still = st.get("ui").map(|u| s(u, "motion") == "reduced").unwrap_or(false);
    let u1 = h / 1080.0;
    if let Some(bg) = design.get("background").filter(|b| !s(b, "asset").is_empty()) {
        pics.push(Picture { asset: s(bg, "asset").into(), x: w / 2.0, y: h / 2.0, w, h, a: n(bg, "alpha", 1.0).clamp(0.0, 1.0), tint: None });
    }
    mem.hits.clear();
    {
    }
    for sh in arr(design, "shapes") {
        let id = s(sh, "id").to_string();
        let hidden = sh.get("hidden").and_then(Value::as_bool).unwrap_or(false);
        if mem.vis.get(&id) == Some(&false) || (hidden && mem.vis.get(&id) != Some(&true)) {
            continue;
        }
        let rp = sh.get("repeat").cloned().unwrap_or(Value::Null);
        let list = items(s(&rp, "by"), &rp, st);
        let total = list.len();
        for (i, it) in list.iter().enumerate() {
            let it = it.as_ref();
            let mut p = resolve(sh, st, it, t, still, mem);
            if it.is_some() {
                if rp.get("spread").and_then(Value::as_bool).unwrap_or(false) {
                    let (a, b) = (n(&rp, "from", 0.3), n(&rp, "to", 0.7));
                    p.x = a + (b - a) * if total > 1 { i as f32 / (total - 1) as f32 } else { 0.5 };
                } else {
                    p.x += n(&rp, "dx", 0.05) * i as f32;
                    p.y += n(&rp, "dy", 0.0) * i as f32;
                }
            }
            if p.alpha <= 0.002 {
                continue;
            }
            let mut c = theme(if s(sh, "color").is_empty() { "accent" } else { s(sh, "color") }, pal, design);
            if let Some(it) = it {
                if !it.state.is_empty() && rp.get("stateColors").and_then(Value::as_bool).unwrap_or(false) {
                    c = theme(match it.state.as_str() { "working" => "accent", "waiting" | "warn" => "warn", "done" | "ok" => "ok", "failed" | "crit" => "crit", "info" => "accent2", _ => "dim" }, pal, design);
                }
            }
            if p.heat > 0.0 && !s(sh, "color2").is_empty() {
                c = mix(c, theme(s(sh, "color2"), pal, design), p.heat.clamp(0.0, 1.0));
            }
            if let Some((fc, until)) = mem.flash.get(&id) {
                if t < *until {
                    c = mix(c, theme(fc, pal, design), 0.5 + 0.5 * (((until - t) * 6.0) as f32).sin().abs());
                }
            }
            let (cx, cy, sw, shh, a) = (ox + p.x * w, oy + p.y * h, p.w * w, p.h * h, p.alpha.clamp(0.0, 1.0));
            let fill = sh.get("fill").and_then(Value::as_bool).unwrap_or(true);
            let lw = (n(sh, "line", 2.0) * u1).max(1.0);
            if p.glow > 0.0 && !matches!(s(sh, "type"), "text" | "image") {
                // a halo: the shape's own colour, wider and fainter (the web page uses a blurred shadow)
                let r = sw.max(shh) / 2.0 + p.glow * h * 0.02;
                for k in 0..4 {
                    disc(win, cx, cy, r * (0.55 + k as f32 * 0.15), c, 0.07 * a * p.glow.min(2.0));
                }
            }
            match s(sh, "type") {
                "rect" => {
                    let (hw, hh) = (sw / 2.0, shh / 2.0);
                    let round = n(sh, "round", 0.0).clamp(0.0, 1.0);
                    if p.rot == 0.0 && fill {
                        win.paint_quad(gpui::PaintQuad::new(gpui::Bounds::new(point(px(cx - hw), px(cy - hh)), gpui::size(px(sw), px(shh)))).background(col(c, a)).corner_radii(px(hw.min(hh) * round)));
                    } else {
                        poly(win, cx, cy, p.rot, &[(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)], fill, true, lw, c, a);
                    }
                }
                "ellipse" => {
                    let pts: Vec<(f32, f32)> = (0..48).map(|k| ((k as f32 / 48.0 * TAU).cos() * sw / 2.0, (k as f32 / 48.0 * TAU).sin() * shh / 2.0)).collect();
                    poly(win, cx, cy, p.rot, &pts, fill, true, lw, c, a);
                }
                "line" => poly(win, cx, cy, p.rot, &[(-sw / 2.0, -shh / 2.0), (sw / 2.0, shh / 2.0)], false, false, lw, c, a),
                "star" => {
                    let np = (n(sh, "points", 5.0) as usize).max(3);
                    let inner = n(sh, "inner", 0.45).clamp(0.0, 1.0);
                    let pts: Vec<(f32, f32)> = (0..np * 2).map(|k| {
                        let ang = k as f32 / (np * 2) as f32 * TAU - TAU / 4.0;
                        let rr = if k % 2 == 1 { inner } else { 1.0 };
                        (ang.cos() * sw / 2.0 * rr, ang.sin() * shh / 2.0 * rr)
                    }).collect();
                    poly(win, cx, cy, p.rot, &pts, fill, true, lw, c, a);
                }
                "text" => words.push(Words { x: p.x * w, y: p.y * h, size: (n(sh, "size", 0.04) * h).max(6.0), bold: sh.get("bold").and_then(Value::as_bool).unwrap_or(false), c, a, text: fill_text(s(sh, "text"), st, it) }),
                "ring" => {
                    let r = sw.min(shh) / 2.0;
                    let thick = (r * n(sh, "thick", 0.18).clamp(0.0, 1.0)).max(2.0);
                    let arc = |frac: f32| -> Vec<(f32, f32)> { (0..=64).map(|k| { let ang = -TAU / 4.0 + TAU * frac * k as f32 / 64.0; (ang.cos() * (r - thick / 2.0), ang.sin() * (r - thick / 2.0)) }).collect() };
                    poly(win, cx, cy, p.rot, &arc(1.0), false, false, thick, c, a * 0.18);
                    poly(win, cx, cy, p.rot, &arc(p.level.clamp(0.0, 1.0)), false, false, thick, c, a);
                }
                "bars" => {
                    let series = s(sh, "series");
                    let mut vals: Vec<f32> = if !series.is_empty() && series != "level" { items(series, &Value::Null, st).iter().map(|x| x.as_ref().map(|i| i.v).unwrap_or(0.0)).collect() } else { vec![p.level] };
                    if vals.is_empty() {
                        vals.push(0.0);
                    }
                    let bw = sw / vals.len() as f32;
                    for (k, v) in vals.iter().enumerate() {
                        let bh = shh * v.clamp(0.0, 1.0);
                        rect(win, cx - sw / 2.0 + k as f32 * bw + bw * 0.12, cy + shh / 2.0 - bh, bw * 0.76, bh, c, a);
                    }
                }
                "tower" => {
                    rect(win, cx - sw / 2.0, cy - shh / 2.0, sw, shh, mix(theme("bg", pal, design), c, 0.18), a);
                    poly(win, cx, cy, 0.0, &[(-sw / 2.0, -shh / 2.0), (sw / 2.0, -shh / 2.0), (sw / 2.0, shh / 2.0), (-sw / 2.0, shh / 2.0)], false, true, lw, c, a * 0.8);
                    let cols = ((sw / (h * 0.012)).round() as i32).max(1);
                    let rows = ((shh / (h * 0.018)).round() as i32).max(1);
                    let (cw, ch) = (sw / cols as f32, shh / rows as f32);
                    for ry in 0..rows {
                        for cxi in 0..cols {
                            let hs = ((ry as i64 * 7919 + cxi as i64 * 104729 + it.map(|i| i.i as i64 * 31).unwrap_or(0)) % 997) as f32 / 997.0;
                            if hs < p.level.clamp(0.0, 1.0) {
                                rect(win, cx - sw / 2.0 + cxi as f32 * cw + cw * 0.25, cy - shh / 2.0 + ry as f32 * ch + ch * 0.25, cw * 0.5, ch * 0.45, c, a * (0.5 + hs * 0.5));
                            }
                        }
                    }
                }
                "wave" => {
                    let amp = shh / 2.0 * p.amp.clamp(0.0, 1.0);
                    let f = n(sh, "freq", 3.0).max(0.5);
                    let ph = if still { 0.0 } else { t as f32 * TAU * 0.3 * p.speed };
                    let pts: Vec<(f32, f32)> = (0..=48).map(|k| { let uu = k as f32 / 48.0; (-sw / 2.0 + uu * sw, (uu * f * TAU + ph).sin() * amp) }).collect();
                    poly(win, cx, cy, p.rot, &pts, false, false, lw, c, a);
                }
                "particles" => {
                    let np = (p.count.round() as i64).clamp(0, 400);
                    let dir = if s(sh, "dir").is_empty() { "down" } else { s(sh, "dir") };
                    let seed = (id.len().max(1) as i64) * 977;
                    let sz = (n(sh, "dot", 2.5) * u1).max(1.0);
                    let tf = t as f32;
                    for q in 0..np {
                        let r1 = ((q * 16807 + seed) % 2147) as f32 / 2147.0;
                        let r2 = ((q * 48271 + seed * 3) % 3001) as f32 / 3001.0;
                        let r3 = ((q * 69621 + seed) % 1009) as f32 / 1009.0;
                        let tt = if still { r2 } else { (tf * 0.15 * p.speed * (0.5 + r3) + r2).rem_euclid(1.0) };
                        let (mut fx, mut fy) = (match dir { "right" => tt, "left" => 1.0 - tt, _ => r1 }, match dir { "down" => tt, "up" => 1.0 - tt, _ => r1 });
                        let mut aa = a;
                        if dir == "fireflies" {
                            fx = r1 + if still { 0.0 } else { (tf * (0.3 + r3) + q as f32).sin() * 0.03 };
                            fy = r2 + if still { 0.0 } else { (tf * (0.25 + r1) + q as f32).cos() * 0.04 };
                            if !still {
                                aa *= 0.4 + 0.6 * (tf * (1.0 + r3) + q as f32).sin().abs();
                            }
                        }
                        rect(win, cx - sw / 2.0 + fx * sw - sz / 2.0, cy - shh / 2.0 + fy * shh - sz / 2.0, sz, sz * if dir == "down" || dir == "up" { 2.2 } else { 1.0 }, c, aa);
                    }
                }
                "image" => pics.push(Picture { asset: s(sh, "asset").into(), x: p.x * w, y: p.y * h, w: sw, h: shh, a, tint: if sh.get("tint").and_then(Value::as_bool).unwrap_or(false) { Some(c) } else { None } }),
                _ => {}
            }
            let click = s(sh, "click");
            mem.hits.push((p.x * w, p.y * h, sw.max(h * 0.02) / 2.0, shh.max(h * 0.02) / 2.0, if click.is_empty() { String::new() } else { fill_text(click, st, it) }, id.clone()));
        }
    }
    (words, pics)
}

fn fill_text(tpl: &str, st: &Value, it: Option<&Item>) -> String {
    fill(tpl, st, it)
}

// ------------------------------------------------------------------ actions: when something happens, do something

fn happened(prev: Option<&Value>, st: &Value) -> Vec<&'static str> {
    let Some(p) = prev else { return vec!["start"] };
    let mut out = Vec::new();
    let ag = |v: &Value| -> HashMap<String, String> { arr(v, "agents").iter().map(|a| (format!("{}|{}", s(a, "name"), n(a, "since", 0.0)), s(a, "state").to_string())).collect() };
    let (a0, a1) = (ag(p), ag(st));
    for (k, v) in &a1 {
        if !a0.contains_key(k) {
            out.push("agent-new");
        }
        if a0.get(k) != Some(v) {
            match v.as_str() {
                "waiting" => out.push("agent-waiting"),
                "done" => out.push("agent-done"),
                "failed" => out.push("agent-failed"),
                _ => {}
            }
        }
    }
    let ci = |v: &Value| -> HashMap<String, String> { arr(v, "git").iter().enumerate().map(|(i, r)| (if s(r, "name").is_empty() { i.to_string() } else { s(r, "name").to_string() }, s(r, "ci").to_string())).collect() };
    let (c0, c1) = (ci(p), ci(st));
    for (k, v) in &c1 {
        if c0.get(k) != Some(v) {
            if v == "fail" {
                out.push("ci-fail");
            }
            if v == "ok" && c0.get(k).map(String::as_str) == Some("fail") {
                out.push("ci-ok");
            }
        }
    }
    let val = |v: &Value, src: &str| source(src, v, None).and_then(|x| x.0).unwrap_or(0.0);
    if commits(st) > commits(p) {
        out.push("commit");
    }
    if val(st, "hour").floor() != val(p, "hour").floor() {
        out.push("hour");
    }
    if val(st, "planes") > val(p, "planes") {
        out.push("plane");
    }
    if val(st, "iss") > 10.0 && val(p, "iss") <= 10.0 {
        out.push("iss");
    }
    let li = |v: &Value| v.get("space").and_then(|x| x.get("launchIn")).and_then(Value::as_f64);
    if li(st).map(|x| x <= 0.0) == Some(true) && li(p).map(|x| x <= 0.0) != Some(true) {
        out.push("launch");
    }
    if val(st, "rain") > 0.0 && val(p, "rain") <= 0.0 {
        out.push("rain");
    }
    for (name, ev) in [("focus", "focus"), ("music", "music")] {
        if val(st, name) > 0.0 && val(p, name) <= 0.0 {
            out.push(ev);
        }
    }
    if val(st, "away") <= 0.0 && val(p, "away") > 0.0 {
        out.push("back");
    }
    let rows = |v: &Value| -> HashMap<String, String> {
        let mut m = HashMap::new();
        if let Some(ps) = v.get("panels").and_then(Value::as_object) {
            for (id, pn) in ps {
                for (i, r) in arr(pn, "rows").iter().enumerate() {
                    m.insert(format!("{id}|{}", if s(r, "name").is_empty() { i.to_string() } else { s(r, "name").to_string() }), s(r, "level").to_string());
                }
            }
        }
        m
    };
    let (r0, r1) = (rows(p), rows(st));
    for (k, v) in &r1 {
        let Some(b) = r0.get(k) else { continue };
        if v == "crit" && b != "crit" {
            out.push("row-crit");
        }
        if v == "ok" && (b == "crit" || b == "warn") {
            out.push("row-ok");
        }
    }
    out
}

/// Called with every new state: runs the design's actions and returns the effects to ask Pulse for.
pub fn react(design: &Value, st: &Value, mem: &mut Mem, t: f64) -> Vec<Ask> {
    react_to(design, st, mem, t, None)
}

/// The topmost shape under the pointer: its click text and its id.
pub fn click(mem: &Mem, x: f32, y: f32) -> Option<(String, String)> {
    mem.hits.iter().rev().find(|h| (x - h.0).abs() <= h.2 && (y - h.1).abs() <= h.3).map(|h| (h.4.clone(), h.5.clone()))
}

/// The same with a click on a shape: only the actions that wait for that click fire.
pub fn react_to(design: &Value, st: &Value, mem: &mut Mem, t: f64, clicked: Option<&str>) -> Vec<Ask> {
    let ev = if clicked.is_some() { Vec::new() } else { happened(mem.prev.as_ref(), st) };
    let mut out = Vec::new();
    for ac in arr(design, "actions").iter().take(40) {
        if ac.get("off").and_then(Value::as_bool).unwrap_or(false) {
            continue;
        }
        let id = s(ac, "id").to_string();
        let w = ac.get("when").unwrap_or(&Value::Null);
        let fire = if let Some(shape_id) = clicked {
            s(w, "type") == "event" && s(w, "event") == "click" && (s(w, "shape").is_empty() || s(w, "shape") == shape_id)
        } else if s(w, "type") == "event" {
            if s(w, "event") == "every" {
                let k = (t / n(w, "secs", 60.0).max(5.0) as f64).floor() as i64;
                let was = mem.every.insert(id.clone(), k).unwrap_or(0);
                k > was
            } else {
                ev.contains(&s(w, "event"))
            }
        } else {
            match (mem.prev.as_ref(), source(s(w, "src"), st, None)) {
                (Some(p), Some((Some(v), _, _))) => match source(s(w, "src"), p, None).and_then(|x| x.0) {
                    Some(v0) => {
                        let x = n(w, "value", 0.0);
                        if s(w, "type") == "below" { v < x && v0 >= x } else { v > x && v0 <= x }
                    }
                    None => false,
                },
                _ => false,
            }
        };
        if !fire {
            continue;
        }
        let cool = n(ac, "cooldown", 10.0).max(1.0) as f64;
        if mem.last.get(&id).map(|l| t - l < cool).unwrap_or(false) {
            continue;
        }
        mem.last.insert(id, t);
        for d in arr(ac, "do").iter().take(8) {
            let shape = s(d, "shape").to_string();
            match s(d, "type") {
                "fx" => out.push(Ask { kind: s(d, "fx").to_string(), text: fill(&s(d, "text").chars().take(60).collect::<String>(), st, None) }),
                "play" if !shape.is_empty() => {
                    mem.played.insert(shape, t);
                }
                "flash" if !shape.is_empty() => {
                    mem.flash.insert(shape, (if s(d, "color").is_empty() { "warn".into() } else { s(d, "color").into() }, t + n(d, "secs", 3.0).clamp(0.3, 30.0) as f64));
                }
                "show" if !shape.is_empty() => {
                    mem.vis.insert(shape, true);
                }
                "hide" if !shape.is_empty() => {
                    mem.vis.insert(shape, false);
                }
                "toggle" if !shape.is_empty() => {
                    let hidden = arr(design, "shapes").iter().find(|x| s(x, "id") == shape).and_then(|x| x.get("hidden")).and_then(Value::as_bool).unwrap_or(false);
                    let now = mem.vis.get(&shape).copied().unwrap_or(!hidden);
                    mem.vis.insert(shape, !now);
                }
                _ => {}
            }
        }
    }
    if clicked.is_none() {
        mem.prev = Some(st.clone());
    }
    out
}
