//! pulse-native 1.1: four new worlds, each one linked to as much of your real, digital world as it can hold.
//!
//!   station   an orbital station over the real Earth (day and night, aurora, the real ISS, Starlink, launches)
//!   abyss     an undersea base (the network is the cables on the sea floor, the agents are submarines)
//!   racks     a server room (every core a rack, every service a server, the agents are technicians)
//!   moonbase  a base on the Moon, with the Earth in its real phase in the sky
//!
//! Everything in them is something of yours: a click on any of it says what. The repos are modules or domes, CI is
//! their light, a commit docks a cargo ship, the deploy builds a module, the updates are supplies, the agents work
//! outside (click one: its card), the services down on the internet black out, your line is the link to Earth.
//! They read the hub's whole state (as Pulse's own scenes do), and hide names while you share the screen.

use crate::draw::{c8, Hit, Label, PI, TAU};
use crate::hub::{get, list, num, text};
use crate::life::Frame;
use crate::palette::{mix, Pal, C};
use crate::scene::Rand;
use crate::worlds::{label, G};
use gpui::Window;
use serde_json::Value;
use std::collections::HashMap;

pub const KINDS: [&str; 4] = ["station", "abyss", "racks", "moonbase"];

pub fn is_world(kind: &str) -> bool {
    KINDS.contains(&kind)
}

/// What these worlds keep between frames.
pub struct Mem {
    snow: Vec<[f32; 4]>,
    fish: Vec<[f32; 4]>,
    /// the newest commit seen (git[].last), and when a ship came for it
    last_commit: f64,
    dock: f64,
    /// a launch seen lifting off (local clock)
    launch: f64,
    lin_prev: Option<f32>,
    /// where each agent floats (x, y) and its phase
    agents: HashMap<String, (f32, f32, f32)>,
    /// the sonar's last ping
    ping: f64,
    /// the effects the hub sent (commit, push, deploy...): (kind, words, when)
    pub events: Vec<(String, String, f64)>,
}

impl Default for Mem {
    fn default() -> Self {
        let mut r = Rand::new(2049);
        let four = |r: &mut Rand| [r.next(), r.next(), r.next(), r.next()];
        let snow = (0..140).map(|_| four(&mut r)).collect();
        let fish = (0..60).map(|_| four(&mut r)).collect();
        Mem { snow, fish, last_commit: 0.0, dock: -1e9, launch: -1e9, lin_prev: None, agents: HashMap::new(), ping: -1e9, events: Vec::new() }
    }
}

/// The real world, read once per frame.
struct D {
    private: bool,
    cpu: f32,
    cores: Vec<f32>,
    temp: f32,
    ram: f32,
    disk: f32,
    rx: f32,
    tx: f32,
    sun: f32,
    moon_lit: f32,
    moon_waxing: bool,
    kp: f32,
    iss: f32,
    starlink: usize,
    planes: usize,
    launch_in: Option<f32>,
    mission: String,
    repos: Vec<(String, f32, String, f32)>,
    agents: Vec<(String, String, String)>,
    solar: Option<f32>,
    load: Option<f32>,
    updates: i64,
    kernel: bool,
    alerts: usize,
    down: Vec<(String, String)>,
    line_ms: Option<f32>,
    offline: bool,
    lan: Vec<(String, bool)>,
    deploy: Option<(String, f32)>,
    away: bool,
    meeting: bool,
    music: bool,
    weather_code: i32,
    cloud: f32,
}

fn read(st: &Value, local_secs: i64) -> D {
    let private = get(st, &["privacy"]).as_bool().unwrap_or(false) || get(st, &["screencast"]).as_bool().unwrap_or(false);
    let nm = |s: String| if private { String::new() } else { s };
    let now = num(st, &["t"]).unwrap_or(0.0);
    let sod = (now as i64 + local_secs).rem_euclid(86400) as f32;
    let deploy = list(st, &["widgets"]).iter().find(|w| text(w, &["type"]) == "gauge" && text(w, &["id"]).contains("deploy")).map(|w| (text(w, &["data", "note"]), num(w, &["data", "value"]).unwrap_or(0.0) as f32));
    let inet = get(st, &["internet"]);
    D {
        private,
        cpu: num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32,
        cores: list(st, &["sys", "cores"]).iter().filter_map(|c| c.as_f64().map(|v| v as f32)).take(32).collect(),
        temp: num(st, &["sys", "temp"]).unwrap_or(45.0) as f32,
        ram: num(st, &["sys", "ram"]).unwrap_or(0.0) as f32,
        disk: num(st, &["sys", "disk"]).unwrap_or(0.0) as f32,
        rx: num(st, &["sys", "rx"]).unwrap_or(0.0) as f32,
        tx: num(st, &["sys", "tx"]).unwrap_or(0.0) as f32,
        sun: num(st, &["night", "sun", "el"]).map(|v| v as f32).unwrap_or(if (7.0..19.0).contains(&(sod / 3600.0)) { 30.0 } else { -30.0 }),
        moon_lit: num(st, &["night", "moon", "lit"]).unwrap_or(0.5) as f32,
        moon_waxing: get(st, &["night", "moon", "waxing"]).as_bool().unwrap_or(true),
        kp: num(st, &["space", "kp"]).unwrap_or(0.0) as f32,
        iss: num(st, &["space", "iss", "el"]).unwrap_or(-90.0) as f32,
        starlink: list(st, &["starlink", "sats"]).iter().filter(|s| num(s, &["el"]).unwrap_or(0.0) > 0.0).count(),
        planes: list(st, &["sky", "planes"]).len(),
        launch_in: num(st, &["space", "launch", "net"]).map(|n| (n - now) as f32),
        mission: text(st, &["space", "launch", "mission"]),
        repos: list(st, &["git"]).iter().map(|r| (nm(text(r, &["name"])), num(r, &["week"]).unwrap_or(0.0) as f32, text(r, &["ci"]), num(r, &["dirty"]).unwrap_or(0.0) as f32)).take(10).collect(),
        agents: list(st, &["agents"]).iter().map(|a| (text(a, &["name"]), text(a, &["state"]), nm(text(a, &["text"])))).take(10).collect(),
        solar: num(st, &["energy", "solar"]).map(|v| v as f32),
        load: num(st, &["energy", "load"]).map(|v| v as f32),
        updates: num(st, &["updates", "n"]).unwrap_or(0.0) as i64 + num(st, &["updates", "aur"]).unwrap_or(0.0) as i64,
        kernel: get(st, &["updates", "kernel"]).as_bool().unwrap_or(false),
        alerts: list(st, &["alerts"]).len(),
        down: list(inet, &["services"]).iter().filter(|r| matches!(text(r, &["level"]).as_str(), "warn" | "crit")).map(|r| (text(r, &["name"]), text(r, &["level"]))).collect(),
        line_ms: num(inet, &["line", "out"]).map(|v| v as f32),
        offline: text(inet, &["line", "level"]) == "crit",
        lan: list(st, &["lan"]).iter().map(|d| (nm(text(d, &["name"])), get(d, &["new"]).as_bool().unwrap_or(false))).take(16).collect(),
        deploy,
        away: get(st, &["away"]).as_bool().unwrap_or(false),
        meeting: get(st, &["meeting"]).as_bool().unwrap_or(false),
        music: get(st, &["music", "status"]).as_str() == Some("Playing") || list(st, &["widgets"]).iter().any(|w| text(w, &["id"]) == "media" && text(w, &["data", "status"]) == "Playing"),
        weather_code: num(st, &["weather", "code"]).unwrap_or(0.0) as i32,
        cloud: num(st, &["weather", "cloud"]).unwrap_or(30.0) as f32,
    }
}

fn cl(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}


fn bytes(v: f32) -> String {
    if v >= 1e6 { format!("{:.1} MB/s", v / 1e6) } else if v >= 1e3 { format!("{:.0} KB/s", v / 1e3) } else { format!("{v:.0} B/s") }
}

fn agent_col(state: &str, pal: &Pal) -> C {
    match state {
        "waiting" => pal.warn,
        "done" => pal.ok,
        "failed" => pal.crit,
        "idle" => pal.dim,
        _ => pal.accent,
    }
}

fn agent_words(a: &(String, String, String)) -> String {
    format!("{} · {}{}", if a.0.is_empty() { "an agent".to_string() } else { a.0.clone() }, a.1, if a.2.is_empty() { String::new() } else { format!(" · {}", a.2.chars().take(40).collect::<String>()) })
}

/// The events since the last frame: a new commit (git[].last grew) or the hub's commit/push effect.
fn commit_came(d_last: f64, mem: &mut Mem, t: f64) -> bool {
    let mut came = false;
    if d_last > mem.last_commit {
        if mem.last_commit > 0.0 {
            came = true;
        }
        mem.last_commit = d_last;
    }
    let evs: Vec<_> = mem.events.drain(..).collect();
    for (k, _, _) in evs {
        if k == "commit" || k == "push" {
            came = true;
        }
    }
    if came && t - mem.dock > 30.0 {
        mem.dock = t;
        return true;
    }
    false
}

/// The real launch: the moment it lifted off is kept (the hub moves on to the next one at once).
fn launch_age(d: &D, mem: &mut Mem, t: f64) -> f32 {
    if let Some(v) = d.launch_in {
        let crossed = mem.lin_prev.map(|p| p > 0.0 && v <= 0.0).unwrap_or(false);
        if (crossed || (v <= 0.0 && v > -60.0)) && t - mem.launch > 600.0 {
            mem.launch = t + v.min(0.0) as f64;
        }
    }
    mem.lin_prev = d.launch_in;
    (t - mem.launch) as f32
}

fn countdown(v: Option<f32>) -> String {
    match v {
        Some(v) if v > 0.0 => {
            let s = v as i64;
            if s >= 86400 { format!("next launch in {} days", s / 86400) } else if s >= 3600 { format!("next launch T−{}h{:02}", s / 3600, (s / 60) % 60) } else { format!("next launch T−{:02}:{:02}", s / 60, s % 60) }
        }
        _ => String::new(),
    }
}

/// The places the painted picture gives back (logical pixels, from the picture's corner).
pub type Anchors = HashMap<&'static str, Vec<(f32, f32, f32)>>;

fn at(an: &Anchors, k: &str, i: usize) -> Option<(f32, f32, f32)> {
    an.get(k).and_then(|v| v.get(i)).copied()
}

/// What the painted picture of `kind` depends on now (see plate.rs): rounded, so it is painted again only when
/// the light, your sky, your repos or the turning Earth change enough to be seen.
pub fn plate_ask(kind: &str, st: &Value, pal: &Pal, w: u32, h: u32, sf: f32, local_secs: i64) -> crate::plate::Ask {
    let d = read(st, local_secs);
    let now = num(st, &["t"]).unwrap_or(0.0) as i64;
    let (sun, cloud) = ((d.sun / 3.0).round() as i32 * 3, (d.cloud / 10.0).round() as i32 * 10);
    let repos = list(st, &["git"]).iter().take(8).map(|r| (String::new(), text(r, &["ci"]), (num(r, &["week"]).unwrap_or(0.0) / 5.0).round() as i32 * 5)).collect();
    let accent = [(pal.accent[0] * 255.0) as u8, (pal.accent[1] * 255.0) as u8, (pal.accent[2] * 255.0) as u8];
    crate::plate::Ask {
        kind: kind.to_string(),
        w,
        h,
        sf,
        sun: if kind == "racks" { 0 } else { sun },
        cloud: if matches!(kind, "station" | "abyss" | "moonbase") { cloud } else { 0 },
        moon: if kind == "moonbase" { (d.moon_lit * 20.0).round() as i32 * 5 } else { 0 },
        waxing: d.moon_waxing,
        // the Earth under the station turns a step every five minutes; from the Moon, every half hour
        turn: match kind { "station" => now / 300, "moonbase" => now / 1800, _ => 0 },
        repos,
        accent,
        hot: if kind == "racks" { ((d.temp - 40.0) / 10.0).round() as i32 * 20 } else { 0 },
        cores: if kind == "racks" { d.cores.len().max(2) } else { 0 },
    }
}

/// Draws what moves in the world `kind`, over its painted picture.
#[allow(clippy::too_many_arguments)]
pub fn paint(kind: &str, win: &mut Window, st: &Value, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, local_secs: i64, an: &Anchors, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let d = read(st, local_secs);
    let newest = list(st, &["git"]).iter().filter_map(|r| num(r, &["last"])).fold(0.0f64, f64::max);
    let docked = commit_came(newest, mem, t);
    let still = get(st, &["ui", "motion"]).as_str() == Some("reduced");
    let tt = if still { 0.0 } else { t as f32 };
    let mut g = G { win, ox: f.ox, oy: f.oy };
    match kind {
        "station" => station(&mut g, &d, pal, f, mem, t, tt, still, docked, an, labels, hits),
        "abyss" => abyss(&mut g, &d, pal, f, mem, t, tt, still, docked, an, labels, hits),
        "racks" => racks(&mut g, &d, pal, f, mem, t, tt, still, docked, an, labels, hits),
        _ => moonbase(&mut g, &d, pal, f, mem, t, tt, still, docked, an, labels, hits),
    }
    if d.away {
        g.rect(0.0, 0.0, f.w, f.h, [0.0; 3], 0.35); // you are away: the lights go down
    }
}

/// The repos' labels and their clicks, at the places the picture put them; a red CI blinks there.
#[allow(clippy::too_many_arguments)]
fn repo_marks(g: &mut G, d: &D, pal: &Pal, an: &Anchors, key: &str, below: f32, h: f32, tt: f32, still: bool, labels: &mut Vec<Label>, hits: &mut Vec<Hit>, what: &str) {
    for (i, (name, week, ci, dirty)) in d.repos.iter().enumerate() {
        let Some((x, y, s)) = at(an, key, i) else { continue };
        if ci == "fail" && !still {
            g.glow(x, y, s * 1.4, pal.crit, 0.35 + 0.3 * (tt * 4.0).sin());
        }
        if ci == "run" && !still {
            g.glow(x, y, s * 1.1, pal.warn, 0.2 + 0.2 * (tt * 2.0).sin());
        }
        if *dirty > 0.0 {
            g.glow(x + s * 0.8, y - s * 0.5, h * 0.008, pal.warn, 0.7);
        }
        label(labels, x, y + s * below, h * 0.0105, pal.fg, 0.8, name.chars().take(14).collect(), false);
        hits.push(Hit { x, y, r: s.max(h * 0.02), text: format!("{} · {what} · {week:.0} commits this week{}{}", if name.is_empty() { "a repo".into() } else { name.clone() }, if ci.is_empty() { String::new() } else { format!(" · CI {ci}") }, if *dirty > 0.0 { format!(" · {dirty:.0} changed") } else { String::new() }) });
    }
}

// ============================================================================================ station

#[allow(clippy::too_many_arguments)]
fn station(g: &mut G, d: &D, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, tt: f32, still: bool, _docked: bool, an: &Anchors, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, u) = (f.w, f.h, f.u);
    let limb = at(an, "limb", 0).map(|a| a.1).unwrap_or(h * 0.4);
    let (sx, sy, half) = at(an, "truss", 0).unwrap_or((w * 0.5, h * 0.56, w * 0.22));
    hits.push(Hit { x: w * 0.5, y: limb + h * 0.3, r: h * 0.12, text: format!("the Earth · {} where you are (the Sun at {:.0}°), clouds {:.0}% as in your sky", if d.sun > 0.0 { "day" } else { "night" }, d.sun, d.cloud) });
    // a service down: its region of the Earth blacks out, with its name
    for (i, (name, lv)) in d.down.iter().take(4).enumerate() {
        let (x, y) = (w * (0.3 + 0.13 * i as f32), limb + h * (0.1 + 0.05 * (i % 2) as f32));
        let c = if lv == "crit" { pal.crit } else { pal.warn };
        g.glow(x, y, h * 0.05, c, if still { 0.55 } else { 0.35 + 0.25 * (tt * 3.0).sin() });
        g.disc(x, y, h * 0.004, c, 1.0);
        label(labels, x, y + h * 0.032, h * 0.012, c, 0.95, format!("{name} {}", if lv == "crit" { "down" } else { "degraded" }), true);
        hits.push(Hit { x, y, r: h * 0.05, text: format!("{name} · its status page says {}", if lv == "crit" { "down" } else { "degraded" }) });
    }
    // the aurora on the horizon, with the real Kp
    if d.kp >= 4.0 {
        let k = cl((d.kp - 3.0) / 5.0);
        for j in 0..40 {
            let x = w * (j as f32 / 40.0);
            let bend = ((x - w * 0.5) / w).powi(2) * h * 0.25; // the horizon curves down to the sides
            let hh = h * (0.03 + 0.04 * (0.5 + 0.5 * (j as f32 * 0.7 + tt * 0.4).sin())) * k;
            g.grad(x, limb + bend - hh, w / 40.0 + 1.0, hh, c8(140, 255, 190), 0.0, c8(90, 255, 160), 0.4 * k);
        }
        hits.push(Hit { x: w * 0.2, y: limb - h * 0.03, r: h * 0.06, text: format!("aurora · the real Kp is {:.0}", d.kp) });
    }
    // the planes really over you: their lights crossing the ground below
    for k in 0..d.planes.min(12) {
        let p = ((tt * 0.006 + k as f32 * 0.137) % 1.0 + 1.0) % 1.0;
        let (x, y) = (w * (0.1 + 0.8 * p), limb + h * (0.15 + 0.06 * (k % 5) as f32));
        g.disc(x, y, (u * 1.2).max(1.0), WHITE, 0.85);
        if (tt * 1.4 + k as f32).fract() < 0.2 {
            g.glow(x, y, u * 6.0, c8(255, 80, 80), 0.7);
        }
    }
    // the repos: their modules (the picture), here their names and a red CI's alarm
    repo_marks(g, d, pal, an, "module", 1.4, h, tt, still, labels, hits, "a module of the station");
    // the deploy: a new module being assembled below the truss, as far along as the deploy
    if let Some((note, v)) = &d.deploy {
        let (x, k) = (sx + half * 0.5, cl(v / 100.0));
        g.grad(x - h * 0.05, sy + h * 0.045, h * 0.1 * k, h * 0.03, c8(240, 240, 236), 1.0, c8(130, 132, 140), 1.0);
        g.rect(x - h * 0.05, sy + h * 0.045, h * 0.1, h * 0.03, pal.accent2, 0.15);
        g.glow(x - h * 0.05 + h * 0.1 * k, sy + h * 0.06, h * 0.012, c8(180, 220, 255), if still { 0.5 } else { 0.5 + 0.4 * (tt * 9.0).sin() });
        label(labels, x, sy + h * 0.095, h * 0.0105, pal.accent2, 0.9, format!("deploy {v:.0}%"), false);
        hits.push(Hit { x, y: sy + h * 0.06, r: h * 0.04, text: format!("the new module · your deploy at {v:.0}%{}", if note.is_empty() { String::new() } else { format!(" · {note}") }) });
    }
    // the dish's beam to the ground: your network, down and up
    let (dx, dy, _) = at(an, "dish", 0).unwrap_or((sx - half * 0.35, sy + h * 0.05, 0.0));
    let ground = (w * 0.36, limb + h * 0.33);
    let beam_c = if d.offline { pal.crit } else { pal.accent };
    g.line(&[(dx, dy), ground], (u * (1.0 + (d.rx / 2e6).min(4.0))).max(1.0), beam_c, 0.25);
    if !still && !d.offline {
        for k in 0..6 {
            let p = (tt * (0.25 + (d.rx / 4e6).min(1.5)) + k as f32 / 6.0).fract();
            g.glow(dx + (ground.0 - dx) * p, dy + (ground.1 - dy) * p, h * 0.006, pal.accent, 0.9);
            let q = 1.0 - (tt * (0.25 + (d.tx / 4e6).min(1.5)) + k as f32 / 6.0 + 0.5).fract();
            g.glow(dx + (ground.0 - dx) * q + u * 3.0, dy + (ground.1 - dy) * q, h * 0.005, pal.accent2, 0.8);
        }
    }
    let link = if d.offline { "the link is down: you are offline".to_string() } else { format!("↓ {} · ↑ {}{}", bytes(d.rx), bytes(d.tx), d.line_ms.map(|m| format!(" · {m:.0} ms")).unwrap_or_default()) };
    label(labels, dx - h * 0.06, dy + h * 0.035, h * 0.011, beam_c, 0.9, link.clone(), false);
    hits.push(Hit { x: dx, y: dy, r: h * 0.03, text: format!("the dish · your network: {link}") });
    // the solar wings: your roof's power
    let solar_words = match d.solar { Some(v) => format!("the solar wings · your roof makes {:.1} kW now", v / 1000.0), None => "the solar wings · they follow the Sun".into() };
    for wg in an.get("wing").into_iter().flatten() {
        hits.push(Hit { x: wg.0, y: wg.1, r: wg.2, text: solar_words.clone() });
    }
    // navigation lights at the truss's ends
    if !still && (tt * 0.8).fract() < 0.12 {
        g.glow(sx - half, sy, h * 0.012, c8(255, 60, 60), 0.9);
        g.glow(sx + half, sy, h * 0.012, c8(60, 255, 120), 0.9);
    }
    // the alarm lamp
    if d.alerts > 0 {
        let (lx, ly, _) = at(an, "lamp", 0).unwrap_or((sx, sy - h * 0.03, 0.0));
        let a = if still { 1.0 } else { (tt * 3.0).sin().max(0.0) };
        g.glow(lx, ly, h * 0.045, pal.crit, 0.8 * a);
        g.disc(lx, ly, u * 3.0, pal.crit, 1.0);
        hits.push(Hit { x: lx, y: ly, r: h * 0.03, text: format!("the alarm · {} alert{} on your machine", d.alerts, if d.alerts > 1 { "s" } else { "" }) });
    }
    // a cargo ship docks for every commit: it comes in from the right, thrusters firing, and docks at the port
    let since = (t - mem.dock) as f32;
    if (0.0..40.0).contains(&since) {
        let (px0, py0, _) = at(an, "port", 0).unwrap_or((sx + half * 0.53, sy - h * 0.012, 0.0));
        let k = cl(since / 16.0);
        let ease = 1.0 - (1.0 - k).powi(3);
        let (px, py) = (px0 + (1.0 - ease) * w * 0.3, py0 - (1.0 - ease) * h * 0.12);
        let dd = h * 0.02;
        g.grad(px, py - dd * 0.7, dd * 2.4, dd * 1.4, c8(250, 250, 250), 1.0, c8(150, 152, 160), 1.0);
        g.poly(&[(px, py - dd * 0.7), (px - dd * 0.8, py - dd * 0.3), (px - dd * 0.8, py + dd * 0.3), (px, py + dd * 0.7)], c8(200, 202, 210), 1.0);
        g.rect(px + dd * 2.4, py - dd * 0.5, dd * 0.6, dd, c8(30, 40, 90), 1.0);
        if k < 1.0 && !still && (tt * 10.0).fract() < 0.5 {
            g.glow(px - dd * 0.9, py, dd * 0.6, c8(200, 230, 255), 0.9);
        }
        label(labels, px + dd, py - dd * 1.8, h * 0.011, pal.ok, 0.95 * cl((40.0 - since) / 5.0), if k < 1.0 { "a commit · cargo docking".into() } else { "docked".into() }, false);
    }
    // the supplies: the updates waiting, in crates on the truss
    if d.updates > 0 {
        let n2 = (d.updates as f32 / 5.0).ceil().min(6.0) as usize;
        for k in 0..n2 {
            let x = sx + half * 0.12 + k as f32 * h * 0.017;
            g.grad(x, sy - h * 0.027, h * 0.014, h * 0.014, if d.kernel && k == 0 { pal.warn } else { c8(200, 180, 140) }, 1.0, c8(110, 96, 70), 1.0);
        }
        hits.push(Hit { x: sx + half * 0.17, y: sy - h * 0.02, r: h * 0.03, text: format!("supplies · {} updates waiting{}", d.updates, if d.kernel { ", a new kernel among them" } else { "" }) });
    }
    // the agents: astronauts out on a spacewalk, tied to the truss
    for (i, a) in d.agents.iter().enumerate() {
        let e = mem.agents.entry(a.0.clone()).or_insert_with(|| (0.0, 0.0, i as f32 * 1.7));
        let col = agent_col(&a.1, pal);
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let reach = match a.1.as_str() { "failed" => 0.2, "waiting" => 0.11, _ => 0.075 };
        let ph = e.2 + if still { 0.0 } else { tt * 0.15 };
        let anchor = (sx + side * half * (0.18 + 0.1 * (i / 2) as f32), sy);
        let up = if i % 3 == 0 { -1.0 } else { 1.0 };
        let (ax, ay) = (anchor.0 + ph.sin() * w * 0.01, sy + up * h * (reach + 0.025 * ph.cos()));
        e.0 = ax;
        e.1 = ay;
        g.line(&[anchor, ((anchor.0 + ax) / 2.0 + u * 8.0, (anchor.1 + ay) / 2.0), (ax, ay)], (u * 0.9).max(1.0), if a.1 == "failed" { pal.crit } else { c8(230, 230, 236) }, 0.7);
        let s = h * 0.016;
        let tilt = if a.1 == "failed" { 0.8 + tt * 0.3 } else { ph.sin() * 0.2 };
        let p = |x: f32, y: f32| (ax + x * tilt.cos() - y * tilt.sin(), ay + x * tilt.sin() + y * tilt.cos());
        // the suit: torso, backpack, arms, legs, helmet with a gold visor in the state's colour
        g.poly(&[p(-s * 0.75, -s * 0.3), p(s * 0.55, -s * 0.3), p(s * 0.55, s * 0.9), p(-s * 0.75, s * 0.9)], c8(200, 200, 206), 1.0);
        g.poly(&[p(-s * 0.55, -s * 0.35), p(s * 0.5, -s * 0.35), p(s * 0.45, s * 0.8), p(-s * 0.5, s * 0.8)], c8(246, 246, 248), 1.0);
        for (lx, ly) in [(-0.35f32, 0.85f32), (0.25, 0.85)] {
            g.poly(&[p(s * lx - s * 0.18, s * ly), p(s * lx + s * 0.18, s * ly), p(s * lx + s * 0.16, s * (ly + 0.8)), p(s * lx - s * 0.16, s * (ly + 0.8))], c8(236, 236, 240), 1.0);
        }
        let arm = if a.1 == "waiting" && !still { (tt * 4.0).sin() * 0.6 } else { 0.0 };
        g.line(&[p(s * 0.45, -s * 0.15), p(s * (0.9 + 0.2 * arm), s * (-0.4 - 0.5 * arm.abs()))], (s * 0.3).max(1.0), c8(236, 236, 240), 1.0);
        g.line(&[p(-s * 0.5, -s * 0.15), p(-s * 0.85, s * 0.35)], (s * 0.3).max(1.0), c8(226, 226, 232), 1.0);
        let hp = p(-s * 0.05, -s * 0.8);
        g.disc(hp.0, hp.1, s * 0.6, c8(244, 244, 246), 1.0);
        g.disc(hp.0 + s * 0.08, hp.1 + s * 0.02, s * 0.42, mix(c8(220, 170, 60), col, 0.45), 1.0);
        g.disc(hp.0 - s * 0.05, hp.1 - s * 0.12, s * 0.12, WHITE, 0.6);
        if a.1 == "working" && !still && (tt * 5.0 + i as f32).fract() < 0.4 {
            let tp = p(s * 1.0, -s * 0.4);
            g.glow(tp.0, tp.1, s * 1.3, c8(180, 220, 255), 0.95); // a weld
        }
        if a.1 == "waiting" {
            label(labels, ax, ay - s * 2.4, h * 0.016, pal.warn, if still || (tt * 2.5).fract() < 0.6 { 1.0 } else { 0.3 }, "!".into(), true);
        }
        g.glow(ax, ay, s * 2.2, col, 0.2);
        label(labels, ax, ay + s * 2.6, h * 0.0105, col, 0.9, a.0.chars().take(16).collect(), false);
        crate::agents::spot(&a.0, f.ox + ax, f.oy + ay, s * 3.0);
        hits.push(Hit { x: ax, y: ay, r: s * 3.0, text: agent_words(a) });
    }
    // Starlink: a train of satellites crossing above (as many as are over you now)
    for k in 0..d.starlink.min(24) {
        let p = ((tt * 0.006 + k as f32 * 0.018) % 1.0 + 1.0) % 1.0;
        let (x, y) = (w * (1.1 - p * 1.2), h * (0.1 + p * 0.06));
        g.rect(x, y, (u * 1.6).max(1.0), (u * 1.6).max(1.0), WHITE, 0.75);
    }
    if d.starlink > 0 {
        hits.push(Hit { x: w * 0.5, y: h * 0.13, r: h * 0.04, text: format!("Starlink · {} satellites over you now", d.starlink) });
    }
    // the real ISS: a bright light passing when it is above your horizon
    if d.iss > 0.0 {
        let p = (tt * 0.02).fract();
        let (x, y) = (w * (0.15 + 0.7 * p), h * 0.07);
        g.glow(x, y, h * 0.012, WHITE, 0.9);
        label(labels, x, y + h * 0.02, h * 0.011, pal.fg, 0.8, format!("ISS · {:.0}° over you", d.iss), false);
    }
    // a real launch: a rocket climbing from the horizon, its plume curving up into space
    let la = launch_age(d, mem, t);
    if (0.0..90.0).contains(&la) && !still {
        let k = cl(la / 80.0);
        let (x0, y0) = (w * 0.74, limb + h * 0.05);
        let pts: Vec<(f32, f32)> = (0..=24).map(|j| { let q = j as f32 / 24.0 * k; (x0 - q * w * 0.1, y0 - q * q * h * 0.75) }).collect();
        g.line(&pts, (u * 4.0).max(2.0), c8(255, 244, 228), 0.3);
        g.line(&pts, (u * 1.5).max(1.0), c8(255, 250, 240), 0.8);
        if let Some(p) = pts.last() {
            g.glow(p.0, p.1, h * 0.025, c8(255, 200, 120), 0.95);
        }
        label(labels, x0, y0 + h * 0.03, h * 0.012, pal.warn, 0.95, format!("liftoff{}", if d.mission.is_empty() { String::new() } else { format!(" · {}", d.mission) }), true);
    } else {
        let c = countdown(d.launch_in);
        if !c.is_empty() {
            label(labels, w * 0.74, limb + h * 0.07, h * 0.011, pal.fg, 0.7, c.clone(), false);
            hits.push(Hit { x: w * 0.74, y: limb + h * 0.07, r: h * 0.03, text: format!("{c}{}", if d.mission.is_empty() { String::new() } else { format!(" · {}", d.mission) }) });
        }
    }
}

// ============================================================================================ abyss

#[allow(clippy::too_many_arguments)]
fn abyss(g: &mut G, d: &D, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, tt: f32, still: bool, _docked: bool, an: &Anchors, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, u) = (f.w, f.h, f.u);
    let floor = at(an, "floor", 0).map(|a| a.1).unwrap_or(h * 0.8);
    let surf = at(an, "surface", 0).map(|a| a.1).unwrap_or(h * 0.07);
    // the ships on the surface: the planes over you
    for k in 0..d.planes.min(6) {
        let p = ((tt * 0.008 + k as f32 * 0.23) % 1.0 + 1.0) % 1.0;
        let x = w * p;
        g.ell(x, surf - h * 0.004, h * 0.03, h * 0.006, c8(4, 14, 24), 0.75);
        g.rect(x - h * 0.006, surf - h * 0.016, h * 0.012, h * 0.01, c8(4, 14, 24), 0.75);
    }
    if d.planes > 0 {
        hits.push(Hit { x: w * 0.5, y: surf, r: h * 0.04, text: format!("the surface · {} ships, the planes really over you", d.planes) });
    }
    // marine snow drifting down, and bubbles rising from the base
    for s in &mem.snow {
        let y = ((s[1] + if still { 0.0 } else { tt * 0.004 * (0.5 + s[2]) }) % 1.0) * h;
        let x = s[0] * w + if still { 0.0 } else { (tt * 0.3 + s[3] * 6.0).sin() * u * 4.0 };
        g.disc(x, y, (u * (0.6 + s[2] * 1.2)).max(0.8), c8(210, 235, 240), 0.2 + 0.35 * s[2]);
    }
    if !still {
        for k in 0..14 {
            let p = (tt * 0.05 + k as f32 / 14.0).fract();
            let x = w * 0.5 + (k as f32 - 7.0) * w * 0.012 + (tt * 2.0 + k as f32).sin() * u * 5.0;
            let y = floor - h * 0.06 - p * (floor - surf);
            g.disc(x, y, u * (1.5 + p * 2.0), c8(220, 245, 255), 0.35 * (1.0 - p));
        }
    }
    // fish: a calm machine, a slow school; a busy one, a fast one (the CPU); shaded bodies, a tail that beats
    let speed = 0.01 + d.cpu / 100.0 * 0.05;
    for (k, fsh) in mem.fish.iter().enumerate().take(18 + (d.cpu / 4.0) as usize) {
        let p = ((fsh[0] + if still { 0.0 } else { tt * speed * (0.6 + fsh[2]) }) % 1.0 + 1.0) % 1.0;
        let (x, y) = (w * (1.05 - p * 1.1), h * (0.3 + fsh[1] * 0.3) + if still { 0.0 } else { (tt * 2.0 + k as f32).sin() * u * 3.0 });
        let s = h * 0.007 * (0.6 + fsh[2]);
        let body = mix(c8(170, 210, 220), c8(250, 200, 120), (k % 4 == 0) as i32 as f32);
        let beat = if still { 0.0 } else { (tt * 12.0 + k as f32).sin() * s * 0.4 };
        g.ell(x, y, s * 1.8, s * 0.7, mix(body, [0.0; 3], 0.3), 0.85);
        g.ell(x, y - s * 0.2, s * 1.6, s * 0.4, body, 0.85);
        g.poly(&[(x + s * 1.6, y), (x + s * 2.6, y - s * 0.8 + beat), (x + s * 2.6, y + s * 0.8 + beat)], mix(body, [0.0; 3], 0.2), 0.85);
        g.disc(x - s * 1.1, y - s * 0.15, (s * 0.18).max(0.8), [0.0; 3], 0.9);
    }
    hits.push(Hit { x: w * 0.5, y: h * 0.42, r: h * 0.08, text: format!("the school · it swims as fast as your CPU works ({:.0}%)", d.cpu) });
    // the cable on the floor: your network. Pulses come in (down) and go out (up); a service down breaks one
    let cy = floor + h * 0.07;
    let cpts: Vec<(f32, f32)> = (0..=40).map(|i| { let x = w * i as f32 / 40.0; (x, cy + (x * 0.01).sin() * h * 0.006) }).collect();
    let cab = if d.offline { pal.crit } else { c8(40, 50, 54) };
    g.line(&cpts, (u * 5.0).max(2.0), cab, 1.0);
    g.line(&cpts, (u * 1.5).max(1.0), c8(120, 140, 150), 0.4);
    if !still && !d.offline {
        for k in 0..8 {
            let pin = (tt * (0.05 + (d.rx / 8e6).min(0.3)) + k as f32 / 8.0).fract();
            g.glow(w * pin, cy + (w * pin * 0.01).sin() * h * 0.006, h * 0.01, pal.accent, 0.9);
            let pout = 1.0 - (tt * (0.05 + (d.tx / 8e6).min(0.3)) + k as f32 / 8.0 + 0.3).fract();
            g.glow(w * pout, cy + (w * pout * 0.01).sin() * h * 0.006, h * 0.008, pal.accent2, 0.8);
        }
    }
    hits.push(Hit { x: w * 0.15, y: cy, r: h * 0.03, text: format!("the cable · your network: ↓ {} ↑ {}{}", bytes(d.rx), bytes(d.tx), if d.offline { " · broken: you are offline".to_string() } else { String::new() }) });
    for (i, (name, lv)) in d.down.iter().take(5).enumerate() {
        let x = w * (0.62 + i as f32 * 0.07);
        let c = if lv == "crit" { pal.crit } else { pal.warn };
        g.rect(x - u * 5.0, cy - u * 5.0, u * 10.0, u * 10.0, c8(10, 20, 30), 1.0);
        if !still {
            for s in 0..4 {
                let a = (tt * 5.0 + s as f32 * 1.7).sin();
                g.line(&[(x, cy), (x + a * u * 10.0, cy - u * (6.0 + s as f32 * 3.0))], (u * 1.0).max(1.0), c, 0.8);
            }
            g.glow(x, cy, h * 0.02, c, 0.5 + 0.3 * (tt * 7.0).sin());
        }
        label(labels, x, cy + h * 0.03, h * 0.011, c, 0.95, name.clone(), true);
        hits.push(Hit { x, y: cy, r: h * 0.025, text: format!("{name} · the cable to it is broken: {}", if lv == "crit" { "down" } else { "degraded" }) });
    }
    // the buoys: the devices of your network, anchored to the floor
    for (i, (name, new)) in d.lan.iter().take(10).enumerate() {
        let x = w * (0.26 + i as f32 * 0.018);
        let y = h * 0.52 + ((i * 37) % 11) as f32 * h * 0.01 + if still { 0.0 } else { (tt * 0.8 + i as f32).sin() * u * 4.0 };
        g.line(&[(x, y), (x, floor)], (u * 0.6).max(1.0), c8(80, 110, 120), 0.5);
        g.disc(x, y, h * 0.008, if *new { pal.warn } else { c8(230, 140, 60) }, 1.0);
        g.disc(x - h * 0.002, y - h * 0.003, h * 0.003, WHITE, 0.5);
        g.glow(x, y, h * 0.014, if *new { pal.warn } else { c8(255, 160, 80) }, 0.35);
        hits.push(Hit { x, y, r: h * 0.015, text: format!("{} · a device of your network{}", if name.is_empty() { "a buoy".into() } else { name.clone() }, if *new { " · new!" } else { "" }) });
    }
    // the domes (the picture): their names, and the sonar ring from the base as fast as your line answers
    repo_marks(g, d, pal, an, "dome", 1.6, h, tt, still, labels, hits, "a dome of the base");
    let every = 4.0 + d.line_ms.map(|m| (m / 50.0).min(6.0)).unwrap_or(2.0) as f64;
    if t - mem.ping > every {
        mem.ping = t;
    }
    let pa = ((t - mem.ping) / every) as f32;
    if !still {
        g.arc(w * 0.5, floor, h * 0.5 * pa, h * 0.2 * pa, PI, TAU, (u * 1.5).max(1.0), pal.accent, 0.35 * (1.0 - pa));
    }
    // a capsule rises to the surface for every commit
    let since = (t - mem.dock) as f32;
    if (0.0..20.0).contains(&since) {
        let y = floor - (floor - surf) * cl(since / 18.0);
        g.ell(w * 0.5, y, h * 0.012, h * 0.018, c8(250, 220, 120), 1.0);
        g.glow(w * 0.5, y, h * 0.03, c8(255, 220, 120), 0.6);
        label(labels, w * 0.5 + h * 0.03, y, h * 0.011, pal.ok, 0.9, "a commit · its capsule goes up".into(), false);
    }
    // the supplies: crates on the floor, one for every few updates
    if d.updates > 0 {
        let n2 = (d.updates as f32 / 5.0).ceil().min(6.0) as usize;
        for k in 0..n2 {
            let x = w * (0.66 + k as f32 * 0.025);
            g.grad(x, floor - h * 0.016, h * 0.022, h * 0.016, if d.kernel && k == 0 { pal.warn } else { c8(190, 150, 100) }, 1.0, c8(90, 70, 50), 1.0);
        }
        hits.push(Hit { x: w * 0.69, y: floor - h * 0.01, r: h * 0.04, text: format!("supplies · {} updates waiting{}", d.updates, if d.kernel { ", a new kernel" } else { "" }) });
    }
    // the agents: yellow submarines with their lamps
    for (i, a) in d.agents.iter().enumerate() {
        let col = agent_col(&a.1, pal);
        let ph = i as f32 * 1.9 + if still { 0.0 } else { tt * 0.07 };
        let failed = a.1 == "failed";
        let (x, y) = if failed { (w * (0.3 + 0.08 * i as f32), floor - h * 0.01) } else { (w * (0.5 + 0.2 * ph.sin()), h * (0.5 + 0.08 * (ph * 1.3).cos())) };
        let dir = if failed { 1.0 } else { -(ph.cos()).signum() };
        let s = h * 0.022;
        if !failed && a.1 != "done" {
            g.poly(&[(x + dir * s * 1.2, y), (x + dir * s * 8.0, y - s * 2.2), (x + dir * s * 8.0, y + s * 2.2)], c8(220, 240, 255), 0.07);
        }
        g.grad(x - s * 1.5, y - s * 0.75, s * 3.0, s * 1.5, c8(255, 220, 90), 1.0, c8(180, 120, 20), 1.0);
        g.ell(x - dir * s * 1.5, y, s * 0.3, s * 0.7, c8(200, 150, 40), 1.0);
        g.ell(x + dir * s * 1.5, y, s * 0.35, s * 0.72, c8(230, 180, 50), 1.0);
        g.disc(x + dir * s * 0.7, y - s * 0.05, s * 0.36, c8(30, 40, 50), 1.0);
        g.disc(x + dir * s * 0.7, y - s * 0.05, s * 0.26, mix(c8(120, 200, 230), col, 0.6), 1.0);
        g.grad(x - s * 0.35, y - s * 1.15, s * 0.7, s * 0.45, c8(255, 220, 90), 1.0, c8(200, 150, 40), 1.0);
        g.line(&[(x - dir * s * 1.8, y), (x - dir * s * 2.2, y + if still { 0.0 } else { (tt * 15.0).sin() * s * 0.3 })], (s * 0.15).max(1.0), c8(120, 120, 120), 1.0);
        g.glow(x + dir * s * 1.6, y, s * 0.8, c8(255, 250, 220), if failed { 0.0 } else { 0.8 });
        g.glow(x, y, s * 2.2, col, 0.25);
        if a.1 == "waiting" {
            label(labels, x, y - s * 2.0, h * 0.016, pal.warn, if still || (tt * 2.5).fract() < 0.6 { 1.0 } else { 0.3 }, "!".into(), true);
        }
        label(labels, x, y + s * 1.8, h * 0.0105, col, 0.9, a.0.chars().take(16).collect(), false);
        crate::agents::spot(&a.0, f.ox + x, f.oy + y, s * 2.5);
        hits.push(Hit { x, y, r: s * 2.5, text: agent_words(a) });
    }
    if d.alerts > 0 && !still {
        g.rect(0.0, 0.0, w, h, pal.crit, 0.05 * (tt * 2.0).sin().max(0.0));
    }
}

// ============================================================================================ racks

#[allow(clippy::too_many_arguments)]
fn racks(g: &mut G, d: &D, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, tt: f32, still: bool, _docked: bool, an: &Anchors, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, u) = (f.w, f.h, f.u);
    let (vx, vy) = (w * 0.5, h * 0.6);
    let cores: Vec<f32> = if d.cores.is_empty() { vec![d.cpu] } else { d.cores.clone() };
    // the LEDs on every server, blinking as busy as that rack's core, and their light on the floor
    let leds = an.get("led").cloned().unwrap_or_default();
    let which = an.get("led_core").cloned().unwrap_or_default();
    for (k, l) in leds.iter().enumerate() {
        let (core, row) = which.get(k).map(|c| (c.0 as usize, c.1)).unwrap_or((0, 0.0));
        let load = cores.get(core).copied().unwrap_or(d.cpu);
        let busy = load / 100.0;
        let on = still || ((tt * (2.0 + busy * 18.0) + row * 1.7 + core as f32).sin() > 1.0 - busy * 2.0);
        let c = if load > 90.0 { pal.warn } else { pal.ok };
        g.disc(l.0, l.1, l.2, c, if on { 1.0 } else { 0.18 });
        if on {
            g.glow(l.0, l.1, l.2 * 4.0, c, 0.35);
        }
        g.disc(l.0 - l.2 * 3.0, l.1, l.2 * 0.8, pal.accent, if on && row as i32 % 3 == 0 { 0.9 } else { 0.25 });
    }
    for (k, r) in an.get("rack").into_iter().flatten().enumerate() {
        let core = an.get("rack_core").and_then(|v| v.get(k)).map(|c| c.0 as usize).unwrap_or(k);
        let load = cores.get(core).copied().unwrap_or(d.cpu);
        hits.push(Hit { x: r.0, y: r.1, r: r.2.max(h * 0.02), text: format!("a rack · core {core} · {load:.0}% busy") });
        let z = an.get("rack_z").and_then(|v| v.get(k)).copied().unwrap_or((0.0, r.1, r.1));
        if z.0 > 0.8 {
            label(labels, r.0, z.1 + h * 0.018, h * 0.0105, pal.fg, 0.75, format!("core {core} · {load:.0}%"), false);
        }
    }
    // the switches' ports: they blink with your real traffic, faster as it grows
    let rate = ((d.rx + d.tx) / 2e6).clamp(0.05, 4.0);
    for (k, p) in an.get("port").into_iter().flatten().enumerate() {
        let on = still || ((k as u32).wrapping_mul(2654435761) ^ ((tt * (3.0 + rate * 6.0)) as u32).wrapping_mul(40503)) % 7 < (2.0 + rate).min(6.0) as u32;
        g.disc(p.0, p.1, p.2, if d.offline { pal.crit } else { c8(90, 255, 140) }, if on { 0.95 } else { 0.15 });
    }
    if let Some(cam) = at(an, "camera", 0) {
        if still || (tt * 0.5).fract() < 0.5 {
            g.glow(cam.0, cam.1, cam.2 * 3.0, c8(255, 60, 60), 0.8);
        }
    }
    // the cable trays carry your traffic: down along the left, up along the right
    let trays = an.get("tray").cloned().unwrap_or_default();
    if trays.len() >= 4 && !still && !d.offline {
        for (side, (p0, p1)) in [(trays[0], trays[1]), (trays[2], trays[3])].iter().enumerate() {
            let rate = if side == 0 { d.rx } else { d.tx };
            for k in 0..5 {
                let p = (tt * (0.1 + (rate / 6e6).min(0.6)) + k as f32 / 5.0).fract();
                let p = if side == 0 { p } else { 1.0 - p };
                g.glow(p0.0 + (p1.0 - p0.0) * p, p0.1 + (p1.1 - p0.1) * p, h * 0.012, if side == 0 { pal.accent } else { pal.accent2 }, 0.9);
            }
        }
    }
    hits.push(Hit { x: vx - w * 0.15, y: vy - h * 0.22, r: h * 0.05, text: format!("the trays · your network ↓ {}", bytes(d.rx)) });
    hits.push(Hit { x: vx + w * 0.15, y: vy - h * 0.22, r: h * 0.05, text: format!("the trays · your network ↑ {}", bytes(d.tx)) });
    // the console on the end wall: RAM, disk, temperature, the deploy, the power
    let (cx0, cy0, cw) = at(an, "console", 0).unwrap_or((vx - w * 0.05, vy - h * 0.11, w * 0.1));
    let chh = an.get("console_h").and_then(|v| v.first()).map(|v| v.0).unwrap_or(h * 0.135);
    let bars = [("RAM", d.ram), ("disk", d.disk), ("temp", d.temp.min(100.0))];
    for (k, (nm, v)) in bars.iter().enumerate() {
        let y = cy0 + chh * (0.14 + k as f32 * 0.2);
        let c = if *v > 85.0 { pal.crit } else if *v > 70.0 { pal.warn } else { pal.ok };
        g.rect(cx0 + cw * 0.3, y, cw * 0.62, chh * 0.08, pal.fg, 0.1);
        g.rect(cx0 + cw * 0.3, y, cw * 0.62 * cl(v / 100.0), chh * 0.08, c, 0.95);
        label(labels, cx0 + cw * 0.15, y + chh * 0.07, h * 0.0095, pal.fg, 0.75, (*nm).into(), false);
    }
    if let Some((_, v)) = &d.deploy {
        let y = cy0 + chh * 0.76;
        g.rect(cx0 + cw * 0.08, y, cw * 0.84, chh * 0.09, pal.fg, 0.1);
        g.rect(cx0 + cw * 0.08, y, cw * 0.84 * cl(v / 100.0), chh * 0.09, pal.accent2, 0.95);
        label(labels, cx0 + cw / 2.0, y + chh * 0.22, h * 0.0095, pal.accent2, 0.9, format!("deploy {v:.0}%"), false);
    }
    g.glow(cx0 + cw / 2.0, cy0 + chh / 2.0, cw * 0.7, pal.accent, 0.08);
    hits.push(Hit { x: cx0 + cw / 2.0, y: cy0 + chh / 2.0, r: chh / 2.0, text: format!("the console · RAM {:.0}% · disk {:.0}% · {:.0}°{}", d.ram, d.disk, d.temp, d.deploy.as_ref().map(|x| format!(" · deploy {:.0}%", x.1)).unwrap_or_default()) });
    if let Some(l) = d.load {
        label(labels, cx0 + cw / 2.0, cy0 - h * 0.02, h * 0.011, pal.warn, 0.9, format!("⚡ {:.2} kW", l / 1000.0), true);
    }
    // the services on the internet: a server each below the console, red when its status page says so
    for (k, (name, lv)) in d.down.iter().take(8).enumerate() {
        let x = vx - w * 0.064 + (k as f32 % 4.0) * w * 0.032;
        let y = vy + h * 0.045 + (k / 4) as f32 * h * 0.035;
        let c = if lv == "crit" { pal.crit } else { pal.warn };
        g.grad(x, y, w * 0.028, h * 0.025, c8(50, 56, 66), 1.0, c8(20, 24, 30), 1.0);
        g.rect(x + w * 0.002, y + h * 0.008, w * 0.024, h * 0.005, c, if still || (tt * 4.0 + k as f32).sin() > 0.0 { 1.0 } else { 0.3 });
        g.glow(x + w * 0.014, y + h * 0.01, h * 0.025, c, 0.3);
        label(labels, x + w * 0.014, y + h * 0.04, h * 0.009, c, 0.95, name.chars().take(14).collect(), false);
        hits.push(Hit { x: x + w * 0.014, y: y + h * 0.012, r: h * 0.02, text: format!("{name} · {}", if lv == "crit" { "down on its status page" } else { "degraded" }) });
    }
    if d.down.is_empty() {
        label(labels, vx, vy + h * 0.06, h * 0.0105, pal.ok, 0.6, "internet · all up".into(), false);
    }
    // the agents: technicians walking the aisle, a tablet in hand
    for (i, a) in d.agents.iter().enumerate() {
        let col = agent_col(&a.1, pal);
        let ph = i as f32 * 2.3 + if still { 0.0 } else { tt * 0.05 };
        let depth = 0.15 + 0.7 * ((ph.sin() + 1.0) / 2.0);
        let z = 1.0 - depth * 0.8;
        let at_rack = a.1 == "working";
        let x = vx + (if i % 2 == 0 { -1.0 } else { 1.0 }) * w * (0.01 + if at_rack { 0.045 } else { 0.015 }) * z * 3.0;
        let y = vy + h * 0.3 * z;
        let s = h * 0.032 * z;
        g.ell(x, y, s * 0.7, s * 0.16, [0.0; 3], 0.45);
        g.grad(x - s * 0.32, y - s * 1.5, s * 0.64, s * 1.05, mix(c8(70, 80, 100), col, 0.3), 1.0, c8(30, 34, 44), 1.0);
        g.rect(x - s * 0.28, y - s * 0.5, s * 0.22, s * 0.5, c8(30, 34, 44), 1.0);
        g.rect(x + s * 0.06, y - s * 0.5, s * 0.22, s * 0.5, c8(30, 34, 44), 1.0);
        g.disc(x, y - s * 1.85, s * 0.34, c8(220, 224, 232), 1.0);
        g.rect(x - s * 0.24, y - s * 1.92, s * 0.48, s * 0.14, col, 1.0); // its visor, in the state's colour
        g.rect(x + s * 0.25, y - s * 1.2, s * 0.3, s * 0.4, c8(20, 30, 50), 1.0);
        g.rect(x + s * 0.28, y - s * 1.17, s * 0.24, s * 0.34, pal.accent, 0.8);
        if at_rack && !still && (tt * 6.0 + i as f32).fract() < 0.3 {
            g.glow(x + s * 0.7, y - s * 1.2, s * 0.6, c8(180, 220, 255), 0.85);
        }
        if a.1 == "failed" && !still {
            for k in 0..3 {
                let q = (tt * 0.5 + k as f32 / 3.0).fract();
                g.glow(x, y - s * (2.2 + q * 2.0), s * (0.3 + q * 0.6), c8(120, 120, 120), 0.45 * (1.0 - q));
            }
        }
        if a.1 == "waiting" {
            label(labels, x, y - s * 2.6, h * 0.016 * z.max(0.6), pal.warn, if still || (tt * 2.5).fract() < 0.6 { 1.0 } else { 0.3 }, "!".into(), true);
        }
        label(labels, x, y + s * 0.6, h * 0.0105 * z.max(0.7), col, 0.9, a.0.chars().take(16).collect(), false);
        crate::agents::spot(&a.0, f.ox + x, f.oy + y - s, s * 2.0);
        hits.push(Hit { x, y: y - s, r: s * 2.0, text: agent_words(a) });
    }
    // a commit: a flash down the aisle
    let since = (t - mem.dock) as f32;
    if (0.0..4.0).contains(&since) {
        g.rect(0.0, 0.0, w, h, pal.ok, 0.06 * (1.0 - since / 4.0));
        label(labels, vx, vy - h * 0.2, h * 0.013, pal.ok, 1.0 - since / 4.0, "a commit · a new blade in".into(), true);
    }
    // the updates: boxes on a cart by the door
    if d.updates > 0 {
        let n2 = (d.updates as f32 / 5.0).ceil().min(6.0) as usize;
        for k in 0..n2 {
            g.grad(w * 0.66 + (k % 3) as f32 * h * 0.03, h * 0.9 - (k / 3) as f32 * h * 0.025, h * 0.028, h * 0.022, if d.kernel && k == 0 { pal.warn } else { c8(200, 160, 110) }, 1.0, c8(100, 76, 50), 1.0);
        }
        hits.push(Hit { x: w * 0.69, y: h * 0.89, r: h * 0.05, text: format!("boxes to unpack · {} updates waiting{}", d.updates, if d.kernel { ", a new kernel" } else { "" }) });
    }
    if d.alerts > 0 && !still {
        let a = (tt * 4.0).sin().max(0.0);
        g.glow(w * 0.3, h * 0.05, h * 0.1, pal.crit, 0.6 * a);
        g.glow(w * 0.7, h * 0.05, h * 0.1, pal.crit, 0.6 * (1.0 - a));
    }
    if d.temp > 75.0 {
        label(labels, vx, h * 0.96, h * 0.012, pal.crit, 0.9, format!("hot aisle · {:.0}°", d.temp), true);
    }
    let _ = u;
}

// ============================================================================================ moonbase

#[allow(clippy::too_many_arguments)]
fn moonbase(g: &mut G, d: &D, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, tt: f32, still: bool, _docked: bool, an: &Anchors, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, u) = (f.w, f.h, f.u);
    let hz = at(an, "horizon", 0).map(|a| a.1).unwrap_or(h * 0.66);
    let (ex, ey, er) = at(an, "earth", 0).unwrap_or((w * 0.64, h * 0.5, h * 0.06));
    let earth_lit = 1.0 - d.moon_lit;
    label(labels, ex, ey + er + h * 0.02, h * 0.011, c8(170, 200, 240), 0.8, format!("Earth · {:.0}% lit", earth_lit * 100.0), false);
    hits.push(Hit { x: ex, y: ey, r: er * 1.2, text: format!("the Earth · {:.0}% lit: tonight your Moon is {:.0}% lit, the other side of the same light", earth_lit * 100.0, d.moon_lit * 100.0) });
    // the laser link to Earth: your network, 1.28 light seconds away
    let (bx, by, _) = at(an, "antenna", 0).unwrap_or((w * 0.4, hz - h * 0.06, 0.0));
    let link_c = if d.offline { pal.crit } else { pal.accent };
    g.line(&[(bx, by), (ex - er * 0.4, ey + er * 0.4)], (u * (0.8 + (d.rx / 3e6).min(3.0))).max(1.0), link_c, 0.22);
    if !still && !d.offline {
        for k2 in 0..4 {
            let p = (tt * 0.78 + k2 as f32 / 4.0).fract();
            g.glow(ex - er * 0.4 + (bx - ex + er * 0.4) * p, ey + er * 0.4 + (by - ey - er * 0.4) * p, h * 0.006, link_c, 0.9);
        }
    }
    hits.push(Hit { x: (bx + ex) / 2.0, y: (by + ey) / 2.0, r: h * 0.04, text: format!("the link · light takes 1.28 s to the Earth; your line answers in {} · ↓ {} ↑ {}", d.line_ms.map(|m| format!("{m:.0} ms")).unwrap_or("—".into()), bytes(d.rx), bytes(d.tx)) });
    for (i, (name, lv)) in d.down.iter().take(4).enumerate() {
        let c = if lv == "crit" { pal.crit } else { pal.warn };
        label(labels, ex, ey + er + h * (0.045 + i as f32 * 0.02), h * 0.0105, c, 0.95, format!("{name} {}", if lv == "crit" { "down" } else { "degraded" }), true);
    }
    // Starlink round the Earth: sparks in a ring
    for k2 in 0..d.starlink.min(20) {
        let a = k2 as f32 / 20.0 * TAU + if still { 0.0 } else { tt * 0.05 };
        g.rect(ex + a.cos() * er * 1.4, ey + a.sin() * er * 0.4, (u * 1.0).max(1.0), (u * 1.0).max(1.0), WHITE, 0.6);
    }
    // the domes (the picture), their names
    repo_marks(g, d, pal, an, "dome", 1.7, h, tt, still, labels, hits, "a dome of the base");
    if let Some(sl) = at(an, "solar", 0) {
        hits.push(Hit { x: sl.0, y: sl.1, r: sl.2 * 0.6, text: match d.solar { Some(v) => format!("the solar field · your roof makes {:.1} kW now", v / 1000.0), None => "the solar field · it follows the Sun".into() } });
    }
    // the lander on its pad: it flies with Earth's real launches
    let la = launch_age(d, mem, t);
    let (lx, ly, _) = at(an, "pad", 0).unwrap_or((w * 0.72, hz + h * 0.09, 0.0));
    let flying = (0.0..60.0).contains(&la) && !still;
    let gone = (60.0..600.0).contains(&la);
    if !gone {
        let lift = if flying { cl((la - 3.0) / 30.0).powi(2) * ly * 1.2 } else { 0.0 };
        let y = ly - lift;
        let s = h * 0.03;
        let grey = |k: f32| -> C { [0.5 * k, 0.5 * k, 0.52 * k] };
        for sd in [-1.0f32, 1.0] {
            g.line(&[(lx + sd * s * 0.6, y - s * 0.4), (lx + sd * s * 1.3, y + s * 0.35)], (u * 2.0).max(1.0), grey(1.8), 1.0);
            g.ell(lx + sd * s * 1.3, y + s * 0.38, s * 0.25, s * 0.06, grey(1.6), 1.0);
        }
        g.grad(lx - s * 0.85, y - s * 0.95, s * 1.7, s * 0.85, c8(240, 200, 90), 1.0, c8(170, 120, 40), 1.0);
        g.poly(&[(lx - s * 0.55, y - s * 0.95), (lx + s * 0.55, y - s * 0.95), (lx + s * 0.35, y - s * 1.7), (lx - s * 0.35, y - s * 1.7)], grey(1.9), 1.0);
        g.disc(lx + s * 0.1, y - s * 1.35, s * 0.14, c8(40, 60, 90), 1.0);
        if flying && la > 2.0 {
            g.poly(&[(lx - s * 0.3, y), (lx + s * 0.3, y), (lx, y + s * 1.8)], c8(255, 230, 180), 0.9);
            g.glow(lx, y + s * 0.6, s * 1.4, c8(255, 200, 120), 0.7);
        }
    }
    let tag = if flying { format!("liftoff{}", if d.mission.is_empty() { String::new() } else { format!(" · {}", d.mission) }) } else if gone { "the next lander is on its way".into() } else { countdown(d.launch_in) };
    label(labels, lx, ly + h * 0.035, h * 0.0105, pal.fg, 0.75, tag.clone(), false);
    hits.push(Hit { x: lx, y: ly - h * 0.02, r: h * 0.04, text: format!("the lander · it flies with Earth's next launch{}", if tag.is_empty() { String::new() } else { format!(" · {tag}") }) });
    // the agents: rovers on the regolith, dust behind them
    for (i, a) in d.agents.iter().enumerate() {
        let col = agent_col(&a.1, pal);
        let ph = i as f32 * 1.3 + if still || a.1 != "working" { 0.0 } else { tt * 0.03 };
        let x = w * (0.3 + 0.4 * ((ph.sin() + 1.0) / 2.0));
        let y = hz + h * (0.17 + 0.035 * (i % 3) as f32);
        let s = h * 0.018;
        g.ell(x + s * 0.8, y + s * 0.4, s * 1.8, s * 0.25, [0.0; 3], 0.4);
        g.grad(x - s * 1.3, y - s * 0.9, s * 2.6, s * 0.75, c8(236, 236, 240), 1.0, c8(130, 134, 140), 1.0);
        g.rect(x - s * 0.9, y - s * 1.25, s * 1.0, s * 0.35, c8(30, 40, 90), 1.0); // its solar panel
        for wx in [-0.85f32, 0.0, 0.85] {
            g.disc(x + wx * s, y, s * 0.36, c8(40, 40, 44), 1.0);
            g.disc(x + wx * s, y, s * 0.16, c8(150, 150, 156), 1.0);
        }
        g.line(&[(x + s * 0.7, y - s * 0.9), (x + s * 0.7, y - s * 1.7)], (u * 1.2).max(1.0), c8(200, 200, 206), 1.0);
        g.disc(x + s * 0.7, y - s * 1.8, s * 0.24, col, 1.0);
        g.glow(x + s * 0.7, y - s * 1.8, s * 1.0, col, 0.55);
        if a.1 == "working" && !still {
            g.ell(x - s * 2.2, y, s * 1.2, s * 0.35, c8(160, 160, 164), 0.3);
        }
        if a.1 == "waiting" {
            label(labels, x, y - s * 2.7, h * 0.016, pal.warn, if still || (tt * 2.5).fract() < 0.6 { 1.0 } else { 0.3 }, "!".into(), true);
        }
        label(labels, x, y + s * 1.4, h * 0.0105, col, 0.9, a.0.chars().take(16).collect(), false);
        crate::agents::spot(&a.0, f.ox + x, f.oy + y - s, s * 2.5);
        hits.push(Hit { x, y: y - s, r: s * 2.5, text: agent_words(a) });
    }
    // a commit: a supply drop lands
    let since = (t - mem.dock) as f32;
    if (0.0..14.0).contains(&since) {
        let y = hz + h * 0.1 - (1.0 - cl(since / 8.0)).powi(2) * h * 0.7;
        g.grad(w * 0.62 - h * 0.012, y - h * 0.02, h * 0.024, h * 0.02, c8(240, 240, 244), 1.0, c8(150, 150, 156), 1.0);
        if since < 8.0 {
            g.glow(w * 0.62, y + h * 0.005, h * 0.02, c8(255, 210, 150), 0.7);
        }
        label(labels, w * 0.62, y - h * 0.03, h * 0.011, pal.ok, 0.9, "a commit · supplies landing".into(), false);
    }
    if d.updates > 0 {
        let n2 = (d.updates as f32 / 5.0).ceil().min(6.0) as usize;
        for k2 in 0..n2 {
            g.grad(w * 0.66 + (k2 % 3) as f32 * h * 0.025, hz + h * 0.09 - (k2 / 3) as f32 * h * 0.02, h * 0.022, h * 0.018, if d.kernel && k2 == 0 { pal.warn } else { c8(220, 220, 226) }, 1.0, c8(120, 120, 126), 1.0);
        }
        hits.push(Hit { x: w * 0.69, y: hz + h * 0.08, r: h * 0.04, text: format!("supplies · {} updates waiting{}", d.updates, if d.kernel { ", a new kernel" } else { "" }) });
    }
    if d.alerts > 0 && !still {
        g.glow(w * 0.5, hz + h * 0.04, h * 0.06, pal.crit, 0.5 * (tt * 3.0).sin().max(0.0));
    }
    let _ = (d.private, d.meeting, d.weather_code, d.iss, d.planes, d.music);
}

const WHITE: C = [1.0, 1.0, 1.0];

#[cfg(test)]
mod tests {
    #[test]
    fn kinds() {
        assert!(super::is_world("station") && super::is_world("moonbase") && !super::is_world("mars"));
    }
}
