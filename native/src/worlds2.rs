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
    stars: Vec<[f32; 4]>,
    snow: Vec<[f32; 4]>,
    fish: Vec<[f32; 4]>,
    rocks: Vec<[f32; 4]>,
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
        let stars = (0..260).map(|_| four(&mut r)).collect();
        let snow = (0..140).map(|_| four(&mut r)).collect();
        let fish = (0..60).map(|_| four(&mut r)).collect();
        let rocks = (0..70).map(|_| four(&mut r)).collect();
        Mem { stars, snow, fish, rocks, last_commit: 0.0, dock: -1e9, launch: -1e9, lin_prev: None, agents: HashMap::new(), ping: -1e9, events: Vec::new() }
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
    hour: f32,
    weather_code: i32,
    wind: f32,
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
        hour: sod / 3600.0,
        weather_code: num(st, &["weather", "code"]).unwrap_or(0.0) as i32,
        wind: num(st, &["weather", "wind"]).unwrap_or(0.0) as f32,
        cloud: num(st, &["weather", "cloud"]).unwrap_or(30.0) as f32,
    }
}

fn cl(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn ci_col(ci: &str, pal: &Pal) -> C {
    match ci {
        "fail" => pal.crit,
        "run" => pal.warn,
        "ok" => pal.ok,
        _ => pal.accent,
    }
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

/// Draws the world `kind`.
#[allow(clippy::too_many_arguments)]
pub fn paint(kind: &str, win: &mut Window, st: &Value, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, local_secs: i64, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let d = read(st, local_secs);
    let newest = list(st, &["git"]).iter().filter_map(|r| num(r, &["last"])).fold(0.0f64, f64::max);
    let docked = commit_came(newest, mem, t);
    let still = get(st, &["ui", "motion"]).as_str() == Some("reduced");
    let tt = if still { 0.0 } else { t as f32 };
    let mut g = G { win, ox: f.ox, oy: f.oy };
    match kind {
        "station" => station(&mut g, &d, pal, f, mem, t, tt, still, docked, labels, hits),
        "abyss" => abyss(&mut g, &d, pal, f, mem, t, tt, still, docked, labels, hits),
        "racks" => racks(&mut g, &d, pal, f, mem, t, tt, still, docked, labels, hits),
        _ => moonbase(&mut g, &d, pal, f, mem, t, tt, still, docked, labels, hits),
    }
    if d.away {
        g.rect(0.0, 0.0, f.w, f.h, [0.0; 3], 0.35); // you are away: the lights go down
    }
}

// ============================================================================================ station

#[allow(clippy::too_many_arguments)]
fn station(g: &mut G, d: &D, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, tt: f32, still: bool, docked: bool, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, u) = (f.w, f.h, f.u);
    // space, and its stars
    g.grad(0.0, 0.0, w, h, c8(2, 3, 9), 1.0, c8(6, 9, 22), 1.0);
    for s in &mem.stars {
        let tw = if still { 1.0 } else { 0.7 + 0.3 * (tt * (0.4 + s[2]) + s[3] * 9.0).sin() };
        let z = (u * (0.6 + s[2] * 1.3)).max(1.0);
        g.rect(s[0] * w, s[1] * h * 0.8, z, z, c8(230, 236, 255), (0.25 + 0.6 * s[2]) * tw);
    }
    // the Earth below: a huge disc whose top is the limb; its day is your day (the Sun's real height where you are)
    let (ecx, ecy, er) = (w * 0.5, h * 0.79 + w * 1.25, w * 1.25);
    let day = cl((d.sun + 6.0) / 18.0);
    let ocean = mix(c8(8, 20, 48), c8(30, 90, 170), day);
    let land = mix(c8(12, 18, 16), c8(80, 120, 70), day);
    g.disc(ecx, ecy, er, ocean, 1.0);
    // the continents drifting by (the station circles the Earth in 92 minutes)
    let drift = (tt / (92.0 * 60.0)) * TAU * 6.0;
    for k in 0..14 {
        let a = -PI / 2.0 + ((k as f32 * 0.37 + drift).rem_euclid(1.6) - 0.8) * 0.9;
        let (x, y) = (ecx + a.cos() * er * 0.985, ecy + a.sin() * er * 0.985);
        if y < h + 40.0 {
            let s = h * (0.05 + 0.04 * ((k * 7) % 5) as f32);
            g.ell(x, y + s * 0.6, s * 1.6, s * 0.45, land, 0.85);
            if day < 0.4 {
                // at night, its cities
                for c in 0..6 {
                    let (cx2, cy2) = (x + (c as f32 - 2.5) * s * 0.35, y + s * 0.5 + ((c * 13) % 7) as f32 * s * 0.06);
                    g.disc(cx2, cy2, (u * 1.4).max(1.0), c8(255, 200, 120), 0.8 * (1.0 - day / 0.4));
                    g.glow(cx2, cy2, u * 6.0, c8(255, 180, 90), 0.25 * (1.0 - day / 0.4));
                }
            }
        }
    }
    // the clouds: your weather's cover
    let cover = cl(d.cloud / 100.0);
    for k in 0..10 {
        let a = -PI / 2.0 + ((k as f32 * 0.19 + drift * 1.3).rem_euclid(1.6) - 0.8) * 0.9;
        let (x, y) = (ecx + a.cos() * er * 0.99, ecy + a.sin() * er * 0.99);
        g.ell(x, y + h * 0.03, h * 0.16, h * 0.022, c8(240, 244, 250), (0.15 + 0.45 * cover) * (0.3 + 0.7 * day));
    }
    // a service down: a dark patch on the Earth where its lights went out, with its name
    for (i, (name, lv)) in d.down.iter().take(4).enumerate() {
        let a = -PI / 2.0 - 0.09 + i as f32 * 0.06;
        let (x, y) = (ecx + a.cos() * er * 0.995, ecy + a.sin() * er * 0.995 + h * 0.04);
        let c = if lv == "crit" { pal.crit } else { pal.warn };
        g.ell(x, y, h * 0.07, h * 0.02, [0.0; 3], 0.55);
        g.glow(x, y, h * 0.03, c, if still { 0.6 } else { 0.4 + 0.3 * (tt * 3.0).sin() });
        label(labels, x, y + h * 0.035, h * 0.012, c, 0.9, format!("{name} {}", if lv == "crit" { "down" } else { "degraded" }), true);
        hits.push(Hit { x, y, r: h * 0.05, text: format!("{name} · its status page says {}", if lv == "crit" { "down" } else { "degraded" }) });
    }
    // the atmosphere on the limb, and the aurora (Kp)
    g.arc(ecx, ecy, er + h * 0.004, er + h * 0.004, PI * 1.2, PI * 1.8, h * 0.012, c8(90, 170, 255), 0.35 + 0.4 * day);
    g.arc(ecx, ecy, er + h * 0.012, er + h * 0.012, PI * 1.2, PI * 1.8, h * 0.02, c8(60, 120, 230), 0.12);
    if d.kp >= 4.0 {
        let k = cl((d.kp - 3.0) / 5.0);
        for j in 0..18 {
            let a = PI * 1.25 + j as f32 * 0.028 + if still { 0.0 } else { (tt * 0.3 + j as f32).sin() * 0.004 };
            let (x, y) = (ecx + a.cos() * er, ecy + a.sin() * er);
            let hh = h * (0.03 + 0.03 * ((j * 7) % 5) as f32 / 5.0) * k;
            g.grad(x - u * 6.0, y - hh, u * 12.0, hh, c8(140, 255, 190), 0.0, c8(90, 255, 160), 0.35 * k);
        }
        hits.push(Hit { x: ecx - er * 0.6, y: ecy - er * 0.8, r: h * 0.06, text: format!("aurora · the real Kp is {:.0}", d.kp) });
    }
    let limb_y = ecy - er;
    hits.push(Hit { x: w * 0.5, y: limb_y + h * 0.1, r: h * 0.12, text: format!("the Earth · {} where you are (the Sun at {:.0}°), clouds {:.0}%", if d.sun > 0.0 { "day" } else { "night" }, d.sun, d.cloud) });
    // the planes: lights crossing the Earth below
    for k in 0..d.planes.min(16) {
        let p = ((tt * 0.01 + k as f32 * 0.137) % 1.0 + 1.0) % 1.0;
        let x = w * (0.1 + 0.8 * p);
        let y = limb_y + h * (0.06 + 0.02 * (k % 5) as f32) + (x - w / 2.0).powi(2) / (2.0 * er);
        g.disc(x, y, (u * 1.3).max(1.0), WHITE, 0.8);
        if (tt * 1.4 + k as f32).fract() < 0.2 {
            g.glow(x, y, u * 5.0, c8(255, 80, 80), 0.6);
        }
    }

    // ---- the station: a truss, the solar wings, a module for each repo
    let (sx, sy) = (w * 0.5, h * 0.56 + if still { 0.0 } else { (tt * 0.2).sin() * h * 0.004 });
    let truss = mix(c8(190, 196, 206), pal.fg, 0.2);
    let half = w * 0.22;
    g.rect(sx - half, sy - h * 0.006, half * 2.0, h * 0.012, truss, 1.0);
    for k in 0..24 {
        let x = sx - half + k as f32 * half * 2.0 / 24.0;
        g.line(&[(x, sy - h * 0.006), (x + half / 12.0, sy + h * 0.006)], (u * 0.8).max(1.0), mix(truss, [0.0; 3], 0.4), 0.8);
    }
    // solar wings: their light is your roof's solar power (or the Sun, without one)
    let sun_k = match d.solar { Some(v) => cl(v / 4000.0), None => day };
    for side in [-1.0f32, 1.0] {
        for k in 0..2 {
            let x0 = sx + side * (half * (0.72 + k as f32 * 0.16));
            for up in [-1.0f32, 1.0] {
                let (pw, ph) = (half * 0.13, h * 0.13);
                let y0 = if up < 0.0 { sy - h * 0.01 - ph } else { sy + h * 0.01 };
                // deep blue cells in a gold frame, catching the light as much as your roof does
                g.rect(x0 - pw / 2.0 - u, y0 - u, pw + 2.0 * u, ph + 2.0 * u, c8(170, 140, 70), 1.0);
                g.rect(x0 - pw / 2.0, y0, pw, ph, mix(c8(14, 26, 70), c8(40, 80, 170), sun_k), 1.0);
                for c in 1..8 {
                    g.rect(x0 - pw / 2.0, y0 + c as f32 * ph / 8.0, pw, (u * 0.7).max(1.0), c8(150, 130, 80), 0.5);
                }
                g.rect(x0 - (u * 0.4).max(0.5), y0, (u * 0.8).max(1.0), ph, c8(150, 130, 80), 0.5);
                g.grad(x0 - pw / 2.0, y0, pw, ph, c8(200, 230, 255), 0.35 * sun_k, c8(200, 230, 255), 0.0);
            }
        }
    }
    let solar_words = match d.solar { Some(v) => format!("the solar wings · your roof makes {:.1} kW now", v / 1000.0), None => "the solar wings · they follow the Sun".into() };
    hits.push(Hit { x: sx - half * 0.8, y: sy - h * 0.07, r: h * 0.07, text: solar_words.clone() });
    hits.push(Hit { x: sx + half * 0.8, y: sy - h * 0.07, r: h * 0.07, text: solar_words });
    // the modules: one for each repo, its length its week of commits, its lights its CI
    let n = d.repos.len().max(1);
    let mw = (half * 0.95 / n as f32).min(h * 0.11);
    for (i, (name, week, ci, dirty)) in d.repos.iter().enumerate() {
        let x = sx - (n as f32 * mw) / 2.0 + i as f32 * mw + mw / 2.0;
        let mh = h * (0.035 + 0.025 * cl(*week / 60.0));
        let col = ci_col(ci, pal);
        let body = mix(c8(220, 222, 228), pal.fg, 0.1);
        g.rect(x - mw * 0.42, sy - mh / 2.0, mw * 0.84, mh, body, 1.0);
        g.rect(x - mw * 0.42, sy + mh * 0.15, mw * 0.84, mh * 0.35, [0.0; 3], 0.25);
        for wdw in 0..3 {
            let lit = if ci == "fail" { (tt * 4.0).sin() > 0.0 } else { true };
            g.rect(x - mw * 0.3 + wdw as f32 * mw * 0.22, sy - mh * 0.2, mw * 0.12, mh * 0.18, if lit { col } else { mix(col, [0.0; 3], 0.6) }, 0.95);
        }
        if *dirty > 0.0 {
            g.glow(x + mw * 0.4, sy - mh * 0.5, u * 6.0, pal.warn, 0.7);
        }
        label(labels, x, sy + mh / 2.0 + h * 0.016, h * 0.0105, pal.fg, 0.75, name.chars().take(14).collect(), false);
        hits.push(Hit { x, y: sy, r: mw * 0.5, text: format!("{} · {week:.0} commits this week{}{}", if name.is_empty() { "a repo".into() } else { name.clone() }, if ci.is_empty() { String::new() } else { format!(" · CI {ci}") }, if *dirty > 0.0 { format!(" · {dirty:.0} changed") } else { String::new() }) });
    }
    // the deploy: a new module being assembled at the end of the truss, as far along as the deploy
    if let Some((note, v)) = &d.deploy {
        let (x, k) = (sx + half * 0.5, cl(v / 100.0));
        g.rect(x - mw * 0.4, sy + h * 0.04, mw * 0.8, h * 0.04, truss, 0.25);
        g.rect(x - mw * 0.4, sy + h * 0.04, mw * 0.8 * k, h * 0.04, pal.accent2, 0.9);
        label(labels, x, sy + h * 0.1, h * 0.0105, pal.accent2, 0.85, format!("deploy {v:.0}%"), false);
        hits.push(Hit { x, y: sy + h * 0.06, r: h * 0.04, text: format!("the new module · your deploy at {v:.0}%{}", if note.is_empty() { String::new() } else { format!(" · {note}") }) });
    }
    // the comms dish: the beam to the Earth is your network, down and up; the ping is the link
    let (dx, dy) = (sx - half * 0.35, sy + h * 0.05);
    g.line(&[(dx, sy), (dx, dy)], (u * 1.5).max(1.0), truss, 1.0);
    g.arcf(dx, dy, h * 0.02, h * 0.012, 0.0, 0.0, PI, mix(truss, WHITE, 0.3), 1.0);
    let ground = (w * 0.38, limb_y + h * 0.05);
    let beam_c = if d.offline { pal.crit } else { pal.accent };
    g.line(&[(dx, dy), ground], (u * (1.0 + (d.rx / 2e6).min(4.0))).max(1.0), beam_c, 0.3);
    if !still && !d.offline {
        for k in 0..6 {
            let down_p = (tt * (0.25 + (d.rx / 4e6).min(1.5)) + k as f32 / 6.0).fract();
            let (x, y) = (dx + (ground.0 - dx) * down_p, dy + (ground.1 - dy) * down_p);
            g.disc(x, y, (u * 1.8).max(1.2), pal.accent, 0.9);
            let up_p = 1.0 - (tt * (0.25 + (d.tx / 4e6).min(1.5)) + k as f32 / 6.0 + 0.5).fract();
            let (x2, y2) = (dx + (ground.0 - dx) * up_p + u * 3.0, dy + (ground.1 - dy) * up_p);
            g.disc(x2, y2, (u * 1.4).max(1.0), pal.accent2, 0.8);
        }
    }
    let link = if d.offline { "the link is down: you are offline".to_string() } else { format!("↓ {} · ↑ {}{}", bytes(d.rx), bytes(d.tx), d.line_ms.map(|m| format!(" · {m:.0} ms")).unwrap_or_default()) };
    label(labels, dx - h * 0.05, dy + h * 0.035, h * 0.011, beam_c, 0.85, link.clone(), false);
    hits.push(Hit { x: dx, y: dy, r: h * 0.03, text: format!("the dish · your network: {link}") });
    // the alert lamp
    if d.alerts > 0 {
        let a = if still { 1.0 } else { (tt * 3.0).sin().max(0.0) };
        g.glow(sx, sy - h * 0.03, h * 0.04, pal.crit, 0.8 * a);
        g.disc(sx, sy - h * 0.025, u * 3.0, pal.crit, 1.0);
        hits.push(Hit { x: sx, y: sy - h * 0.03, r: h * 0.03, text: format!("the alarm · {} alert{} on your machine", d.alerts, if d.alerts > 1 { "s" } else { "" }) });
    }
    // a cargo ship docks for every commit
    let since = (t - mem.dock) as f32;
    if (0.0..40.0).contains(&since) {
        let k = cl(since / 14.0);
        let ease = 1.0 - (1.0 - k).powi(3);
        let (px, py) = (sx + half * 1.15 - ease * half * 0.62, sy - h * 0.12 + ease * h * 0.1);
        let dd = h * 0.026;
        g.rect(px - dd, py - dd * 0.6, dd * 2.0, dd * 1.2, c8(236, 236, 240), 1.0);
        g.poly(&[(px - dd, py - dd * 0.6), (px - dd * 1.6, py), (px - dd, py + dd * 0.6)], c8(200, 200, 210), 1.0);
        if k < 1.0 {
            g.glow(px + dd * 1.1, py, dd * 0.8, c8(160, 200, 255), 0.7);
        }
        label(labels, px, py - dd * 1.6, h * 0.011, pal.ok, 0.9 * cl((40.0 - since) / 5.0), if k < 1.0 { "a commit · cargo docking".into() } else { "docked".into() }, false);
    }
    let _ = docked;
    // the supplies: the updates waiting, in crates on the truss
    if d.updates > 0 {
        let n2 = (d.updates as f32 / 5.0).ceil().min(6.0) as usize;
        for k in 0..n2 {
            g.rect(sx + half * 0.15 + k as f32 * h * 0.016, sy - h * 0.026, h * 0.013, h * 0.013, if d.kernel && k == 0 { pal.warn } else { c8(170, 150, 110) }, 1.0);
        }
        hits.push(Hit { x: sx + half * 0.2, y: sy - h * 0.02, r: h * 0.03, text: format!("supplies · {} updates waiting{}", d.updates, if d.kernel { ", a new kernel among them" } else { "" }) });
    }
    // the agents: astronauts out on a spacewalk, tied to the station
    for (i, a) in d.agents.iter().enumerate() {
        let e = mem.agents.entry(a.0.clone()).or_insert_with(|| (0.0, 0.0, i as f32 * 1.7));
        let col = agent_col(&a.1, pal);
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let reach = match a.1.as_str() { "failed" => 0.22, "waiting" => 0.12, _ => 0.08 };
        let ph = e.2 + if still { 0.0 } else { tt * 0.15 };
        let (ax, ay) = (sx + side * half * (0.25 + 0.12 * (i / 2) as f32) + ph.sin() * w * 0.01, sy + h * (reach + 0.03 * ph.cos()) * if i % 3 == 0 { -1.0 } else { 1.0 });
        e.0 = ax;
        e.1 = ay;
        let anchor = (sx + side * half * (0.25 + 0.12 * (i / 2) as f32), sy);
        g.line(&[anchor, ((anchor.0 + ax) / 2.0 + u * 6.0, (anchor.1 + ay) / 2.0), (ax, ay)], (u * 0.8).max(1.0), if a.1 == "failed" { pal.crit } else { c8(220, 220, 230) }, 0.6);
        let s = h * 0.014;
        let tilt = if a.1 == "failed" { 0.8 + tt * 0.3 } else { 0.0 };
        let p = |x: f32, y: f32| (ax + x * tilt.cos() - y * tilt.sin(), ay + x * tilt.sin() + y * tilt.cos());
        g.poly(&[p(-s * 0.6, -s * 0.2), p(s * 0.6, -s * 0.2), p(s * 0.5, s * 1.3), p(-s * 0.5, s * 1.3)], c8(240, 240, 244), 1.0);
        let hp = p(0.0, -s * 0.7);
        g.disc(hp.0, hp.1, s * 0.62, c8(240, 240, 244), 1.0);
        g.disc(hp.0 + s * 0.1, hp.1, s * 0.4, mix(c8(255, 200, 90), col, 0.5), 1.0); // the visor in the state's colour
        if a.1 == "working" && !still && (tt * 5.0 + i as f32).fract() < 0.4 {
            g.glow(ax + side * s, ay, s * 1.2, c8(180, 220, 255), 0.9); // a weld
        }
        if a.1 == "waiting" {
            label(labels, ax, ay - s * 2.4, h * 0.016, pal.warn, if still || (tt * 2.5).fract() < 0.6 { 1.0 } else { 0.3 }, "!".into(), true);
        }
        g.glow(ax, ay, s * 2.0, col, 0.25);
        label(labels, ax, ay + s * 2.5, h * 0.0105, col, 0.85, a.0.chars().take(16).collect(), false);
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
    // a real launch: a rocket climbing from the limb, its plume curving into space
    let la = launch_age(d, mem, t);
    if (0.0..90.0).contains(&la) && !still {
        let k = cl(la / 80.0);
        let (x0, y0) = (w * 0.78, limb_y + h * 0.06);
        let pts: Vec<(f32, f32)> = (0..=24).map(|j| { let q = j as f32 / 24.0 * k; (x0 - q * w * 0.12, y0 - q * q * h * 0.9) }).collect();
        g.line(&pts, (u * 3.0).max(1.5), c8(255, 240, 220), 0.35);
        if let Some(p) = pts.last() {
            g.glow(p.0, p.1, h * 0.02, c8(255, 200, 120), 0.9);
        }
        label(labels, x0, y0 + h * 0.03, h * 0.012, pal.warn, 0.9, format!("liftoff{}", if d.mission.is_empty() { String::new() } else { format!(" · {}", d.mission) }), true);
    } else {
        let c = countdown(d.launch_in);
        if !c.is_empty() {
            label(labels, w * 0.78, limb_y + h * 0.09, h * 0.011, pal.fg, 0.6, c.clone(), false);
            hits.push(Hit { x: w * 0.78, y: limb_y + h * 0.09, r: h * 0.03, text: format!("{c}{}", if d.mission.is_empty() { String::new() } else { format!(" · {}", d.mission) }) });
        }
    }
}

// ============================================================================================ abyss

#[allow(clippy::too_many_arguments)]
fn abyss(g: &mut G, d: &D, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, tt: f32, still: bool, _docked: bool, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, u) = (f.w, f.h, f.u);
    let day = cl((d.sun + 6.0) / 18.0);
    let top = mix(c8(4, 22, 44), c8(20, 110, 150), day);
    g.grad(0.0, 0.0, w, h * 0.55, top, 1.0, c8(3, 18, 36), 1.0);
    g.grad(0.0, h * 0.55, w, h * 0.45, c8(3, 18, 36), 1.0, c8(1, 6, 14), 1.0);
    // the surface far above: its light is your day, its waves your wind, the ships your planes
    let surf = h * 0.07;
    let waves = 0.004 + cl(d.wind / 60.0) * 0.012;
    let pts: Vec<(f32, f32)> = (0..=64).map(|i| { let x = w * i as f32 / 64.0; (x, surf + (x * 0.012 + tt * 1.2).sin() * h * waves) }).collect();
    g.line(&pts, (u * 2.0).max(1.0), mix(c8(120, 200, 230), WHITE, day * 0.5), 0.35 + 0.3 * day);
    let mut sky = vec![(0.0, 0.0)];
    sky.extend(pts.iter().copied());
    sky.push((w, 0.0));
    g.poly(&sky, mix(c8(10, 20, 40), c8(150, 200, 230), day), 0.5);
    for k in 0..d.planes.min(6) {
        let p = ((tt * 0.008 + k as f32 * 0.23) % 1.0 + 1.0) % 1.0;
        let x = w * p;
        g.rect(x - h * 0.015, surf - h * 0.012, h * 0.03, h * 0.008, c8(10, 20, 30), 0.8);
        g.rect(x - h * 0.004, surf - h * 0.02, h * 0.008, h * 0.008, c8(10, 20, 30), 0.8);
    }
    if d.planes > 0 {
        hits.push(Hit { x: w * 0.5, y: surf, r: h * 0.04, text: format!("the surface · {} ships, the planes really over you", d.planes) });
    }
    // god rays: the Sun through the water (fewer under clouds)
    if day > 0.05 {
        let k = day * (1.0 - cl(d.cloud / 100.0) * 0.6);
        for r in 0..7 {
            let x = w * (0.12 + r as f32 * 0.13) + if still { 0.0 } else { (tt * 0.1 + r as f32).sin() * w * 0.01 };
            g.poly(&[(x - w * 0.02, surf), (x + w * 0.02, surf), (x + w * 0.09, h * 0.8), (x - w * 0.01, h * 0.8)], c8(170, 230, 255), 0.05 * k);
        }
    }
    // marine snow drifting down, and bubbles
    for s in &mem.snow {
        let y = ((s[1] + if still { 0.0 } else { tt * 0.004 * (0.5 + s[2]) }) % 1.0) * h;
        let x = s[0] * w + if still { 0.0 } else { (tt * 0.3 + s[3] * 6.0).sin() * u * 4.0 };
        g.rect(x, y, (u * (0.8 + s[2])).max(1.0), (u * (0.8 + s[2])).max(1.0), c8(200, 230, 240), 0.25 + 0.3 * s[2]);
    }
    // the sea floor
    let floor = h * 0.8;
    let fpts: Vec<(f32, f32)> = (0..=48).map(|i| { let x = w * i as f32 / 48.0; (x, floor + (x * 0.007).sin() * h * 0.012 + (x * 0.023).cos() * h * 0.006) }).collect();
    let mut sea = fpts.clone();
    sea.push((w, h));
    sea.push((0.0, h));
    g.poly(&sea, c8(16, 30, 38), 1.0);
    g.line(&fpts, (u * 1.5).max(1.0), c8(40, 70, 80), 0.8);
    for r in &mem.rocks {
        let (x, y) = (r[0] * w, floor + r[1] * h * 0.15);
        g.ell(x, y, h * 0.01 * (0.5 + r[2]), h * 0.005 * (0.5 + r[2]), c8(30, 48, 56), 1.0);
    }
    // fish: a calm machine, a slow school; a busy one, a fast one (the CPU)
    let speed = 0.01 + d.cpu / 100.0 * 0.05;
    for (k, fsh) in mem.fish.iter().enumerate().take(20 + (d.cpu / 4.0) as usize) {
        let p = ((fsh[0] + if still { 0.0 } else { tt * speed * (0.6 + fsh[2]) }) % 1.0 + 1.0) % 1.0;
        let (x, y) = (w * (1.05 - p * 1.1), h * (0.3 + fsh[1] * 0.35) + if still { 0.0 } else { (tt * 2.0 + k as f32).sin() * u * 3.0 });
        let s = h * 0.006 * (0.6 + fsh[2]);
        g.ell(x, y, s * 1.6, s * 0.6, c8(150, 210, 220), 0.6);
        g.poly(&[(x + s * 1.4, y), (x + s * 2.4, y - s * 0.7), (x + s * 2.4, y + s * 0.7)], c8(150, 210, 220), 0.6);
    }
    hits.push(Hit { x: w * 0.5, y: h * 0.45, r: h * 0.08, text: format!("the school · it swims as fast as your CPU works ({:.0}%)", d.cpu) });
    // the cables on the floor: your network. Pulses come in (down) and go out (up); a service down breaks one
    let cy = floor + h * 0.05;
    let cpts: Vec<(f32, f32)> = (0..=40).map(|i| { let x = w * i as f32 / 40.0; (x, cy + (x * 0.01).sin() * h * 0.008) }).collect();
    let cab = if d.offline { pal.crit } else { c8(60, 90, 100) };
    g.line(&cpts, (u * 3.0).max(1.5), cab, 1.0);
    if !still && !d.offline {
        for k in 0..8 {
            let pin = (tt * (0.05 + (d.rx / 8e6).min(0.3)) + k as f32 / 8.0).fract();
            g.glow(w * pin, cy + (w * pin * 0.01).sin() * h * 0.008, h * 0.008, pal.accent, 0.9);
            let pout = 1.0 - (tt * (0.05 + (d.tx / 8e6).min(0.3)) + k as f32 / 8.0 + 0.3).fract();
            g.glow(w * pout, cy + (w * pout * 0.01).sin() * h * 0.008, h * 0.006, pal.accent2, 0.8);
        }
    }
    hits.push(Hit { x: w * 0.15, y: cy, r: h * 0.03, text: format!("the cable · your network: ↓ {} ↑ {}{}", bytes(d.rx), bytes(d.tx), if d.offline { " · broken: you are offline".to_string() } else { String::new() }) });
    for (i, (name, lv)) in d.down.iter().take(5).enumerate() {
        let x = w * (0.62 + i as f32 * 0.07);
        let c = if lv == "crit" { pal.crit } else { pal.warn };
        g.rect(x - u * 4.0, cy - u * 4.0, u * 8.0, u * 8.0, pal.bg, 1.0);
        if !still {
            for s in 0..4 {
                let a = (tt * 5.0 + s as f32 * 1.7).sin();
                g.line(&[(x, cy), (x + a * u * 10.0, cy - u * (6.0 + s as f32 * 3.0))], (u * 1.0).max(1.0), c, 0.7);
            }
        }
        label(labels, x, cy + h * 0.03, h * 0.011, c, 0.95, name.clone(), true);
        hits.push(Hit { x, y: cy, r: h * 0.025, text: format!("{name} · the cable to it is broken: {}", if lv == "crit" { "down" } else { "degraded" }) });
    }
    // the buoys: the devices of your network, anchored to the floor
    for (i, (name, new)) in d.lan.iter().take(10).enumerate() {
        let x = w * (0.26 + i as f32 * 0.018);
        let y = h * 0.52 + ((i * 37) % 11) as f32 * h * 0.01 + if still { 0.0 } else { (tt * 0.8 + i as f32).sin() * u * 4.0 };
        g.line(&[(x, y), (x, floor)], (u * 0.6).max(1.0), c8(80, 110, 120), 0.5);
        g.disc(x, y, h * 0.007, if *new { pal.warn } else { c8(230, 140, 60) }, 1.0);
        g.glow(x, y, h * 0.012, if *new { pal.warn } else { c8(255, 160, 80) }, 0.35);
        hits.push(Hit { x, y, r: h * 0.015, text: format!("{} · a device of your network{}", if name.is_empty() { "a buoy".into() } else { name.clone() }, if *new { " · new!" } else { "" }) });
    }
    // the base: a dome for each repo, lit by its CI
    let n = d.repos.len().max(1);
    for (i, (name, week, ci, _)) in d.repos.iter().enumerate() {
        let x = w * 0.5 + (i as f32 - (n as f32 - 1.0) / 2.0) * w * 0.07;
        let r = h * (0.035 + 0.02 * cl(*week / 60.0));
        let col = ci_col(ci, pal);
        g.arcf(x, floor, r * 1.2, r, 0.0, PI, TAU, c8(120, 180, 200), 0.35);
        g.arc(x, floor, r * 1.2, r, PI, TAU, (u * 1.5).max(1.0), c8(170, 220, 230), 0.8);
        g.glow(x, floor - r * 0.4, r * 1.3, col, 0.35 * if ci == "fail" && !still { 0.5 + 0.5 * (tt * 4.0).sin() } else { 1.0 });
        g.rect(x - r * 0.3, floor - r * 0.45, r * 0.6, r * 0.3, col, 0.8);
        label(labels, x, floor + h * 0.02, h * 0.0105, pal.fg, 0.75, name.chars().take(12).collect(), false);
        hits.push(Hit { x, y: floor - r * 0.5, r, text: format!("{} · {week:.0} commits this week{}", if name.is_empty() { "a dome".into() } else { name.clone() }, if ci.is_empty() { String::new() } else { format!(" · CI {ci}") }) });
    }
    // the sonar: a ring from the base, as fast as your line answers
    let every = 4.0 + d.line_ms.map(|m| (m / 50.0).min(6.0)).unwrap_or(2.0) as f64;
    if t - mem.ping > every {
        mem.ping = t;
    }
    let pa = ((t - mem.ping) / every) as f32;
    if !still {
        g.arc(w * 0.5, floor, h * 0.5 * pa, h * 0.25 * pa, PI, TAU, (u * 1.5).max(1.0), pal.accent, 0.4 * (1.0 - pa));
    }
    // a capsule rises to the surface for every commit
    let since = (t - mem.dock) as f32;
    if (0.0..20.0).contains(&since) {
        let y = floor - (floor - surf) * cl(since / 18.0);
        g.ell(w * 0.5, y, h * 0.012, h * 0.018, c8(250, 220, 120), 1.0);
        g.glow(w * 0.5, y, h * 0.03, c8(255, 220, 120), 0.6);
        label(labels, w * 0.5 + h * 0.03, y, h * 0.011, pal.ok, 0.9, "a commit · its capsule goes up".into(), false);
    }
    // the supplies: crates sinking to the floor, one for every few updates
    if d.updates > 0 {
        let n2 = (d.updates as f32 / 5.0).ceil().min(6.0) as usize;
        for k in 0..n2 {
            let x = w * (0.66 + k as f32 * 0.025);
            let y = floor - h * 0.012 - ((tt * 0.02 + k as f32 * 0.3).fract() * 0.0) * h;
            g.rect(x, y, h * 0.02, h * 0.016, if d.kernel && k == 0 { pal.warn } else { c8(150, 120, 80) }, 1.0);
        }
        hits.push(Hit { x: w * 0.82, y: floor - h * 0.01, r: h * 0.04, text: format!("supplies · {} updates waiting{}", d.updates, if d.kernel { ", a new kernel" } else { "" }) });
    }
    // the agents: submarines with their lamps
    for (i, a) in d.agents.iter().enumerate() {
        let col = agent_col(&a.1, pal);
        let ph = i as f32 * 1.9 + if still { 0.0 } else { tt * 0.07 };
        let failed = a.1 == "failed";
        let (x, y) = if failed { (w * (0.3 + 0.08 * i as f32), floor - h * 0.012) } else { (w * (0.5 + 0.2 * ph.sin()), h * (0.56 + 0.08 * (ph * 1.3).cos())) };
        let dir = if failed { 1.0 } else { ph.cos().signum() };
        let s = h * 0.02;
        if !failed && a.1 != "done" {
            g.poly(&[(x + dir * s, y), (x + dir * s * 7.0, y - s * 2.0), (x + dir * s * 7.0, y + s * 2.0)], c8(220, 240, 255), 0.08);
        }
        g.ell(x, y, s * 1.4, s * 0.7, c8(240, 200, 60), 1.0);
        g.disc(x + dir * s * 0.6, y - s * 0.1, s * 0.3, mix(c8(120, 200, 230), col, 0.5), 1.0);
        g.rect(x - s * 0.3, y - s * 1.0, s * 0.5, s * 0.35, c8(220, 180, 50), 1.0);
        g.glow(x, y, s * 2.0, col, 0.3);
        if a.1 == "waiting" {
            label(labels, x, y - s * 2.0, h * 0.016, pal.warn, if still || (tt * 2.5).fract() < 0.6 { 1.0 } else { 0.3 }, "!".into(), true);
        }
        label(labels, x, y + s * 1.8, h * 0.0105, col, 0.85, a.0.chars().take(16).collect(), false);
        crate::agents::spot(&a.0, f.ox + x, f.oy + y, s * 2.5);
        hits.push(Hit { x, y, r: s * 2.5, text: agent_words(a) });
    }
    // the alarm: the base's lights turn red
    if d.alerts > 0 && !still {
        g.rect(0.0, 0.0, w, h, pal.crit, 0.05 * (tt * 2.0).sin().max(0.0));
    }
}

// ============================================================================================ racks

#[allow(clippy::too_many_arguments)]
fn racks(g: &mut G, d: &D, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, tt: f32, still: bool, _docked: bool, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, u) = (f.w, f.h, f.u);
    let hot = cl((d.temp - 55.0) / 35.0);
    // the room: a cold aisle in perspective, warmer as your machine heats up
    g.grad(0.0, 0.0, w, h, mix(c8(8, 12, 20), c8(30, 10, 10), hot * 0.6), 1.0, c8(4, 6, 10), 1.0);
    let (vx, vy) = (w * 0.5, h * 0.6); // the vanishing point
    // the floor tiles, with the cold air from the vents (stronger with the heat: the fans)
    for k in 0..14 {
        let y = vy + (h - vy) * (k as f32 / 14.0).powf(1.8);
        g.line(&[(0.0, y), (w, y)], (u * 0.6).max(1.0), c8(40, 60, 80), 0.35);
    }
    for k in -8..=8 {
        g.line(&[(vx + k as f32 * w * 0.01, vy), (vx + k as f32 * w * 0.16, h)], (u * 0.6).max(1.0), c8(40, 60, 80), 0.3);
    }
    if !still {
        for k in 0..6 {
            let a = (tt * (0.2 + hot * 0.6) + k as f32 / 6.0).fract();
            g.ell(vx + (k as f32 - 2.5) * w * 0.05, h - a * (h - vy) * 0.8, w * 0.04 * (1.0 - a), h * 0.01, c8(140, 200, 255), 0.06 * (1.0 - a));
        }
    }
    // the ceiling's cable trays: the network, its pulses down and up
    for side in [-1.0f32, 1.0] {
        let pts = [(vx + side * w * 0.02, vy - h * 0.1), (vx + side * w * 0.3, vy - h * 0.34)];
        g.line(&pts, (u * 4.0).max(2.0), c8(50, 60, 70), 0.9);
        if !still && !d.offline {
            for k in 0..5 {
                let rate = if side < 0.0 { d.rx } else { d.tx };
                let p = (tt * (0.1 + (rate / 6e6).min(0.6)) + k as f32 / 5.0).fract();
                let (x, y) = (pts[0].0 + (pts[1].0 - pts[0].0) * if side < 0.0 { p } else { 1.0 - p }, pts[0].1 + (pts[1].1 - pts[0].1) * if side < 0.0 { p } else { 1.0 - p });
                g.glow(x, y, h * 0.01, if side < 0.0 { pal.accent } else { pal.accent2 }, 0.9);
            }
        }
    }
    hits.push(Hit { x: vx - w * 0.15, y: vy - h * 0.22, r: h * 0.05, text: format!("the trays · your network ↓ {}", bytes(d.rx)) });
    hits.push(Hit { x: vx + w * 0.15, y: vy - h * 0.22, r: h * 0.06, text: format!("the trays · your network ↑ {}", bytes(d.tx)) });
    // the racks: one for each core (its lights blink as busy as that core), down both sides of the aisle
    let cores: Vec<f32> = if d.cores.is_empty() { vec![d.cpu] } else { d.cores.clone() };
    let per = cores.len().div_ceil(2).max(1);
    for (i, load) in cores.iter().enumerate() {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let depth = (i / 2) as f32 / per as f32; // 0 near, 1 far
        let z = 1.0 - depth * 0.82;
        let x_in = vx + side * w * (0.035 + 0.2 * z);
        let x_out = vx + side * w * (0.035 + 0.2 * (1.0 - (depth + 1.0 / per as f32) * 0.82).max(0.05));
        let (top, bot) = (vy - h * 0.3 * z, vy + h * 0.3 * z);
        let (xa, xb) = if side < 0.0 { (x_in.min(x_out), x_in.max(x_out)) } else { (x_in.min(x_out), x_in.max(x_out)) };
        let rw = (xb - xa).max(2.0);
        g.rect(xa, top, rw, bot - top, mix(c8(18, 22, 30), c8(30, 34, 44), z), 1.0);
        g.rect(xa, top, rw, (u * 1.0).max(1.0), c8(60, 70, 80), 1.0);
        let rows = 12;
        for r in 0..rows {
            let y = top + (bot - top) * (r as f32 + 0.5) / rows as f32;
            let busy = load / 100.0;
            let on = still || ((tt * (2.0 + busy * 18.0) + r as f32 * 1.7 + i as f32).sin() > 1.0 - busy * 2.0);
            let c = if *load > 90.0 { pal.warn } else { pal.ok };
            g.rect(xa + rw * 0.15, y, (rw * 0.08).max(1.0), (u * 1.5 * z).max(1.0), c, if on { 0.95 } else { 0.15 });
            g.rect(xa + rw * 0.3, y, rw * 0.5, (u * 0.8 * z).max(1.0), c8(70, 80, 90), 0.5);
        }
        if depth < 0.5 {
            label(labels, xa + rw / 2.0, bot + h * 0.018 * z, h * 0.0105 * z.max(0.6), pal.fg, 0.6, format!("core {i} · {load:.0}%"), false);
        }
        hits.push(Hit { x: xa + rw / 2.0, y: (top + bot) / 2.0, r: (rw / 2.0).max(h * 0.02), text: format!("core {i} · {load:.0}% busy") });
    }
    // the end wall: a big console with what matters (the deploy, the RAM, the disk, the temperature, the power)
    let (cxw, cyw, cw, chh) = (vx - w * 0.045, vy - h * 0.1, w * 0.09, h * 0.12);
    g.rect(cxw, cyw, cw, chh, c8(6, 10, 16), 1.0);
    g.rect(cxw, cyw, cw, chh, pal.accent, 0.06);
    let bars = [("RAM", d.ram), ("disk", d.disk), ("temp", (d.temp / 100.0 * 100.0).min(100.0))];
    for (k, (nm, v)) in bars.iter().enumerate() {
        let y = cyw + chh * (0.18 + k as f32 * 0.2);
        let c = if *v > 85.0 { pal.crit } else if *v > 70.0 { pal.warn } else { pal.ok };
        g.rect(cxw + cw * 0.3, y, cw * 0.62, chh * 0.07, pal.fg, 0.1);
        g.rect(cxw + cw * 0.3, y, cw * 0.62 * cl(v / 100.0), chh * 0.07, c, 0.9);
        label(labels, cxw + cw * 0.15, y + chh * 0.06, h * 0.0095, pal.fg, 0.7, (*nm).into(), false);
    }
    if let Some((_, v)) = &d.deploy {
        let y = cyw + chh * 0.78;
        g.rect(cxw + cw * 0.08, y, cw * 0.84, chh * 0.08, pal.fg, 0.1);
        g.rect(cxw + cw * 0.08, y, cw * 0.84 * cl(v / 100.0), chh * 0.08, pal.accent2, 0.9);
        label(labels, vx, y + chh * 0.2, h * 0.0095, pal.accent2, 0.85, format!("deploy {v:.0}%"), false);
    }
    hits.push(Hit { x: vx, y: cyw + chh / 2.0, r: chh / 2.0, text: format!("the console · RAM {:.0}% · disk {:.0}% · {:.0}°{}", d.ram, d.disk, d.temp, d.deploy.as_ref().map(|x| format!(" · deploy {:.0}%", x.1)).unwrap_or_default()) });
    // the power: the PDU's meter, your home's load
    if let Some(l) = d.load {
        label(labels, vx, cyw - h * 0.02, h * 0.011, pal.warn, 0.85, format!("⚡ {:.2} kW", l / 1000.0), true);
    }
    // the services on the internet: a server each on the wall, red when its status page says so
    let svc_n = d.down.len();
    for (k, (name, lv)) in d.down.iter().take(8).enumerate() {
        let x = vx - w * 0.064 + (k as f32 % 4.0) * w * 0.032;
        let y = vy + h * 0.045 + (k / 4) as f32 * h * 0.035;
        let c = if lv == "crit" { pal.crit } else { pal.warn };
        g.rect(x, y, w * 0.028, h * 0.025, c8(20, 24, 30), 1.0);
        g.rect(x + w * 0.002, y + h * 0.006, w * 0.024, h * 0.004, c, if still || (tt * 4.0 + k as f32).sin() > 0.0 { 1.0 } else { 0.3 });
        label(labels, x + w * 0.014, y + h * 0.04, h * 0.009, c, 0.9, name.chars().take(12).collect(), false);
        hits.push(Hit { x: x + w * 0.014, y: y + h * 0.012, r: h * 0.02, text: format!("{name} · {}", if lv == "crit" { "down on its status page" } else { "degraded" }) });
    }
    if svc_n == 0 {
        label(labels, vx, vy + h * 0.08, h * 0.0105, pal.ok, 0.6, "internet · all up".into(), false);
    }
    // the agents: technicians walking the aisle
    for (i, a) in d.agents.iter().enumerate() {
        let col = agent_col(&a.1, pal);
        let ph = i as f32 * 2.3 + if still { 0.0 } else { tt * 0.05 };
        let depth = 0.15 + 0.7 * ((ph.sin() + 1.0) / 2.0);
        let z = 1.0 - depth * 0.8;
        let at_rack = a.1 == "working";
        let x = vx + (if i % 2 == 0 { -1.0 } else { 1.0 }) * w * (0.01 + if at_rack { 0.045 } else { 0.015 }) * z * 3.0;
        let y = vy + h * 0.3 * z;
        let s = h * 0.03 * z;
        g.ell(x, y, s * 0.6, s * 0.15, [0.0; 3], 0.4);
        g.rect(x - s * 0.3, y - s * 1.4, s * 0.6, s * 1.0, mix(c8(60, 70, 90), col, 0.3), 1.0);
        g.rect(x - s * 0.35, y - s * 2.1, s * 0.7, s * 0.6, c8(200, 205, 215), 1.0);
        g.rect(x - s * 0.22, y - s * 1.95, s * 0.44, s * 0.18, col, 1.0); // its eyes, in the state's colour
        if at_rack && !still && (tt * 6.0 + i as f32).fract() < 0.3 {
            g.glow(x + s * 0.6, y - s * 1.2, s * 0.6, c8(180, 220, 255), 0.8);
        }
        if a.1 == "failed" && !still {
            for k in 0..3 {
                let q = (tt * 0.5 + k as f32 / 3.0).fract();
                g.glow(x, y - s * (2.2 + q * 2.0), s * (0.3 + q * 0.6), c8(120, 120, 120), 0.4 * (1.0 - q));
            }
        }
        if a.1 == "waiting" {
            label(labels, x, y - s * 2.6, h * 0.016 * z.max(0.6), pal.warn, if still || (tt * 2.5).fract() < 0.6 { 1.0 } else { 0.3 }, "!".into(), true);
        }
        label(labels, x, y + s * 0.6, h * 0.0105 * z.max(0.7), col, 0.85, a.0.chars().take(16).collect(), false);
        crate::agents::spot(&a.0, f.ox + x, f.oy + y - s, s * 2.0);
        hits.push(Hit { x, y: y - s, r: s * 2.0, text: agent_words(a) });
    }
    // a commit: a blade slides into a rack and flashes
    let since = (t - mem.dock) as f32;
    if (0.0..4.0).contains(&since) {
        g.rect(0.0, 0.0, w, h, pal.ok, 0.06 * (1.0 - since / 4.0));
        label(labels, vx, vy - h * 0.2, h * 0.013, pal.ok, 1.0 - since / 4.0, "a commit · a new blade in".into(), true);
    }
    // the updates: boxes on a cart by the door
    if d.updates > 0 {
        let n2 = (d.updates as f32 / 5.0).ceil().min(6.0) as usize;
        for k in 0..n2 {
            g.rect(w * 0.66 + (k % 3) as f32 * h * 0.03, h * 0.9 - (k / 3) as f32 * h * 0.025, h * 0.028, h * 0.022, if d.kernel && k == 0 { pal.warn } else { c8(160, 130, 90) }, 1.0);
        }
        hits.push(Hit { x: w * 0.69, y: h * 0.89, r: h * 0.05, text: format!("boxes to unpack · {} updates waiting{}", d.updates, if d.kernel { ", a new kernel" } else { "" }) });
    }
    // the alarm: the room's beacons turn
    if d.alerts > 0 && !still {
        let a = (tt * 4.0).sin().max(0.0);
        g.glow(w * 0.1, h * 0.05, h * 0.08, pal.crit, 0.6 * a);
        g.glow(w * 0.9, h * 0.05, h * 0.08, pal.crit, 0.6 * (1.0 - a));
    }
    if hot > 0.5 {
        label(labels, vx, h * 0.96, h * 0.012, pal.crit, 0.9, format!("hot aisle · {:.0}°", d.temp), true);
    }
    let _ = mem;
}

// ============================================================================================ moonbase

#[allow(clippy::too_many_arguments)]
fn moonbase(g: &mut G, d: &D, pal: &Pal, f: &Frame, mem: &mut Mem, t: f64, tt: f32, still: bool, _docked: bool, labels: &mut Vec<Label>, hits: &mut Vec<Hit>) {
    let (w, h, u) = (f.w, f.h, f.u);
    let hz = h * 0.66;
    // no air: a black sky with every star, by day too
    g.rect(0.0, 0.0, w, hz + h * 0.05, c8(1, 1, 4), 1.0);
    for s in &mem.stars {
        let z = (u * (0.6 + s[2] * 1.2)).max(1.0);
        g.rect(s[0] * w, s[1] * hz, z, z, c8(235, 238, 255), 0.3 + 0.6 * s[2]);
    }
    // the Earth, in its real phase: from the Moon it is lit as much as the Moon is dark in your sky
    let (ex, ey, er) = (w * 0.64, h * 0.52, h * 0.055);
    let earth_lit = 1.0 - d.moon_lit;
    g.disc(ex, ey, er, c8(16, 30, 60), 1.0);
    // the lit part: a disc with the dark side cut by an ellipse (the terminator)
    let side = if d.moon_waxing { -1.0 } else { 1.0 };
    let k = earth_lit * 2.0 - 1.0; // -1 new, 1 full
    let steps = 40;
    let mut pts = Vec::with_capacity(steps * 2 + 2);
    for i in 0..=steps {
        let a = -PI / 2.0 + PI * i as f32 / steps as f32;
        pts.push((ex + side * a.cos() * er, ey + a.sin() * er));
    }
    for i in (0..=steps).rev() {
        let a = -PI / 2.0 + PI * i as f32 / steps as f32;
        pts.push((ex - side * a.cos() * er * k, ey + a.sin() * er));
    }
    g.poly(&pts, c8(70, 140, 220), 1.0);
    g.ell(ex + side * er * 0.2, ey - er * 0.2, er * 0.35, er * 0.15, WHITE, 0.4 * earth_lit);
    g.glow(ex, ey, er * 1.6, c8(90, 160, 255), 0.15 + 0.2 * earth_lit);
    label(labels, ex, ey + er + h * 0.02, h * 0.011, c8(170, 200, 240), 0.7, format!("Earth · {:.0}% lit", earth_lit * 100.0), false);
    hits.push(Hit { x: ex, y: ey, r: er * 1.2, text: format!("the Earth · {:.0}% lit: tonight your Moon is {:.0}% lit, the other side of the same light", earth_lit * 100.0, d.moon_lit * 100.0) });
    // the laser link to Earth: your network, 1.28 light seconds away
    let (bx, by) = (w * 0.4, hz - h * 0.06);
    let link_c = if d.offline { pal.crit } else { pal.accent };
    g.line(&[(bx, by), (ex - er * 0.4, ey + er * 0.4)], (u * (0.8 + (d.rx / 3e6).min(3.0))).max(1.0), link_c, 0.25);
    if !still && !d.offline {
        for k2 in 0..4 {
            let p = (tt * 0.78 + k2 as f32 / 4.0).fract(); // 1.28 s from end to end
            let (x, y) = (ex - er * 0.4 + (bx - ex + er * 0.4) * p, ey + er * 0.4 + (by - ey - er * 0.4) * p);
            g.disc(x, y, (u * 1.6).max(1.0), link_c, 0.9);
        }
    }
    hits.push(Hit { x: (bx + ex) / 2.0, y: (by + ey) / 2.0, r: h * 0.04, text: format!("the link · light takes 1.28 s to the Earth; your line answers in {} · ↓ {} ↑ {}", d.line_ms.map(|m| format!("{m:.0} ms")).unwrap_or("—".into()), bytes(d.rx), bytes(d.tx)) });
    for (i, (name, lv)) in d.down.iter().take(4).enumerate() {
        let c = if lv == "crit" { pal.crit } else { pal.warn };
        label(labels, ex, ey + er + h * (0.045 + i as f32 * 0.02), h * 0.0105, c, 0.95, format!("{name} {}", if lv == "crit" { "down" } else { "degraded" }), true);
    }
    // the Sun: where yours is (the Moon's day lasts two weeks; yours sets the light here)
    let lightk = cl((d.sun + 10.0) / 30.0) * 0.6 + 0.4;
    if d.sun > -5.0 {
        let sx = w * (0.1 + 0.8 * cl((d.hour - 5.0) / 14.0));
        let sy = hz - (d.sun.max(-5.0) / 90.0) * hz * 0.9;
        g.glow(sx, sy, h * 0.05, WHITE, 0.9);
        g.disc(sx, sy, h * 0.012, WHITE, 1.0);
    }
    // the ground: grey regolith, craters, long shadows
    let gpts: Vec<(f32, f32)> = (0..=48).map(|i| { let x = w * i as f32 / 48.0; (x, hz + (x * 0.004).sin() * h * 0.02 + (x * 0.013).cos() * h * 0.008) }).collect();
    let mut ground = gpts.clone();
    ground.push((w, h));
    ground.push((0.0, h));
    let grey = |k: f32| -> C { [0.42 * k * lightk, 0.42 * k * lightk, 0.44 * k * lightk] };
    g.poly(&ground, grey(1.0), 1.0);
    g.grad(0.0, hz, w, h - hz, grey(1.1), 0.0, [0.0; 3], 0.5);
    for r in &mem.rocks {
        let (x, y) = (r[0] * w, hz + h * 0.03 + r[1] * (h - hz) * 0.95);
        let s = h * 0.012 * (0.4 + r[2] * 1.4) * (0.4 + r[1]);
        g.ell(x, y, s * 1.6, s * 0.45, grey(0.7), 1.0);
        g.ell(x + s * 0.15, y - s * 0.08, s * 1.3, s * 0.32, grey(0.55), 1.0);
    }
    // the habitat: a dome for each repo, lit by its CI
    let n = d.repos.len().max(1);
    for (i, (name, week, ci, _)) in d.repos.iter().enumerate() {
        let x = w * 0.5 + (i as f32 - (n as f32 - 1.0) / 2.0) * w * 0.075;
        let y = hz + h * 0.07;
        let r = h * (0.035 + 0.02 * cl(*week / 60.0));
        let col = ci_col(ci, pal);
        g.arcf(x, y, r * 1.2, r, 0.0, PI, TAU, grey(1.4), 1.0);
        g.arc(x, y, r * 1.2, r, PI, TAU, (u * 1.2).max(1.0), grey(1.8), 1.0);
        g.rect(x - r * 0.25, y - r * 0.4, r * 0.5, r * 0.4, col, 0.9);
        g.glow(x, y - r * 0.2, r, col, 0.3);
        label(labels, x, y + h * 0.02, h * 0.0105, pal.fg, 0.75, name.chars().take(12).collect(), false);
        hits.push(Hit { x, y: y - r * 0.5, r, text: format!("{} · {week:.0} commits this week{}", if name.is_empty() { "a dome".into() } else { name.clone() }, if ci.is_empty() { String::new() } else { format!(" · CI {ci}") }) });
    }
    // the antenna by the base
    g.line(&[(bx, hz + h * 0.04), (bx, by)], (u * 1.5).max(1.0), grey(1.6), 1.0);
    g.arcf(bx, by, h * 0.025, h * 0.014, -0.5, 0.0, PI, grey(1.9), 1.0);
    // solar panels: your roof's power
    let sun_k = match d.solar { Some(v) => cl(v / 4000.0), None => cl((d.sun + 5.0) / 30.0) };
    for k2 in 0..5 {
        let x = w * (0.25 + k2 as f32 * 0.035);
        let y = hz + h * 0.12;
        g.poly(&[(x, y), (x + w * 0.032, y - h * 0.01), (x + w * 0.032, y + h * 0.012), (x, y + h * 0.022)], mix(c8(20, 30, 80), c8(150, 170, 230), sun_k * 0.6), 1.0);
    }
    hits.push(Hit { x: w * 0.33, y: hz + h * 0.125, r: h * 0.05, text: match d.solar { Some(v) => format!("the solar field · your roof makes {:.1} kW now", v / 1000.0), None => "the solar field · it follows the Sun".into() } });
    // the lander on its pad: it flies with Earth's real launches
    let la = launch_age(d, mem, t);
    let (lx, ly) = (w * 0.72, hz + h * 0.09);
    g.ell(lx, ly + h * 0.005, h * 0.05, h * 0.008, grey(0.6), 1.0);
    let flying = (0.0..60.0).contains(&la) && !still;
    let gone = (60.0..600.0).contains(&la);
    if !gone {
        let lift = if flying { (cl((la - 3.0) / 30.0)).powi(2) * ly * 1.2 } else { 0.0 };
        let y = ly - lift;
        let s = h * 0.03;
        g.poly(&[(lx - s, y), (lx - s * 1.3, y + s * 0.4), (lx - s * 1.1, y + s * 0.4)], grey(1.6), 1.0);
        g.poly(&[(lx + s, y), (lx + s * 1.3, y + s * 0.4), (lx + s * 1.1, y + s * 0.4)], grey(1.6), 1.0);
        g.rect(lx - s * 0.8, y - s * 0.8, s * 1.6, s * 0.8, c8(210, 170, 60), 1.0);
        g.poly(&[(lx - s * 0.5, y - s * 0.8), (lx + s * 0.5, y - s * 0.8), (lx + s * 0.3, y - s * 1.5), (lx - s * 0.3, y - s * 1.5)], grey(1.9), 1.0);
        if flying && la > 2.0 {
            g.poly(&[(lx - s * 0.3, y), (lx + s * 0.3, y), (lx, y + s * 1.6)], c8(255, 220, 160), 0.9);
            g.glow(lx, y + s * 0.5, s * 1.2, c8(255, 200, 120), 0.6);
        }
    }
    let tag = if flying { format!("liftoff{}", if d.mission.is_empty() { String::new() } else { format!(" · {}", d.mission) }) } else if gone { "the next lander is on its way".into() } else { countdown(d.launch_in) };
    label(labels, lx, ly + h * 0.03, h * 0.0105, pal.fg, 0.7, tag.clone(), false);
    hits.push(Hit { x: lx, y: ly - h * 0.02, r: h * 0.04, text: format!("the lander · it flies with Earth's next launch{}", if tag.is_empty() { String::new() } else { format!(" · {tag}") }) });
    // the agents: rovers on the regolith, dust behind them
    for (i, a) in d.agents.iter().enumerate() {
        let col = agent_col(&a.1, pal);
        let ph = i as f32 * 1.3 + if still || a.1 != "working" { 0.0 } else { tt * 0.03 };
        let x = w * (0.3 + 0.4 * ((ph.sin() + 1.0) / 2.0));
        let y = hz + h * (0.16 + 0.035 * (i % 3) as f32);
        let s = h * 0.016;
        g.rect(x - s * 1.2, y - s * 0.8, s * 2.4, s * 0.7, grey(1.8), 1.0);
        g.disc(x - s * 0.8, y, s * 0.35, grey(0.8), 1.0);
        g.disc(x + s * 0.8, y, s * 0.35, grey(0.8), 1.0);
        g.line(&[(x + s * 0.6, y - s * 0.8), (x + s * 0.6, y - s * 1.6)], (u * 1.0).max(1.0), grey(1.6), 1.0);
        g.disc(x + s * 0.6, y - s * 1.7, s * 0.25, col, 1.0);
        g.glow(x + s * 0.6, y - s * 1.7, s * 0.9, col, 0.5);
        if a.1 == "working" && !still {
            g.ell(x - s * 2.0, y, s * 1.0, s * 0.3, grey(1.3), 0.3);
        }
        if a.1 == "waiting" {
            label(labels, x, y - s * 2.6, h * 0.016, pal.warn, if still || (tt * 2.5).fract() < 0.6 { 1.0 } else { 0.3 }, "!".into(), true);
        }
        label(labels, x, y + s * 1.4, h * 0.0105, col, 0.85, a.0.chars().take(16).collect(), false);
        crate::agents::spot(&a.0, f.ox + x, f.oy + y - s, s * 2.5);
        hits.push(Hit { x, y: y - s, r: s * 2.5, text: agent_words(a) });
    }
    // a commit: a supply drop lands
    let since = (t - mem.dock) as f32;
    if (0.0..14.0).contains(&since) {
        let y = hz + h * 0.1 - (1.0 - cl(since / 8.0)).powi(2) * h * 0.7;
        g.rect(w * 0.62 - h * 0.012, y - h * 0.02, h * 0.024, h * 0.02, c8(230, 230, 236), 1.0);
        if since < 8.0 {
            g.glow(w * 0.62, y, h * 0.02, c8(255, 210, 150), 0.7);
        }
        label(labels, w * 0.62, y - h * 0.03, h * 0.011, pal.ok, 0.9, "a commit · supplies landing".into(), false);
    }
    // the updates: crates by the airlock
    if d.updates > 0 {
        let n2 = (d.updates as f32 / 5.0).ceil().min(6.0) as usize;
        for k2 in 0..n2 {
            g.rect(w * 0.66 + (k2 % 3) as f32 * h * 0.025, hz + h * 0.09 - (k2 / 3) as f32 * h * 0.02, h * 0.022, h * 0.018, if d.kernel && k2 == 0 { pal.warn } else { grey(1.5) }, 1.0);
        }
        hits.push(Hit { x: w * 0.69, y: hz + h * 0.08, r: h * 0.04, text: format!("supplies · {} updates waiting{}", d.updates, if d.kernel { ", a new kernel" } else { "" }) });
    }
    // Starlink and the ISS: the Earth's own satellites, seen from here as a ring of sparks round it
    for k2 in 0..d.starlink.min(20) {
        let a = k2 as f32 / 20.0 * TAU + if still { 0.0 } else { tt * 0.05 };
        g.rect(ex + a.cos() * er * 1.35, ey + a.sin() * er * 0.4, (u * 1.0).max(1.0), (u * 1.0).max(1.0), WHITE, 0.6);
    }
    if d.alerts > 0 && !still {
        g.glow(w * 0.5, hz + h * 0.04, h * 0.06, pal.crit, 0.5 * (tt * 3.0).sin().max(0.0));
    }
    if d.music && !still {
        // music in the habitat: its windows dance
        g.rect(w * 0.45, hz + h * 0.06, w * 0.1, h * 0.004, pal.accent2, 0.5 + 0.5 * (tt * 8.0).sin());
    }
    let _ = (d.private, d.meeting, d.weather_code, d.iss, d.planes);
}

const WHITE: C = [1.0, 1.0, 1.0];

#[cfg(test)]
mod tests {
    #[test]
    fn kinds() {
        assert!(super::is_world("station") && super::is_world("moonbase") && !super::is_world("mars"));
    }
}
