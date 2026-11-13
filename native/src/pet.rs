//! pulse-native: Pip, the creature that lives on your wallpaper, a tamagotchi whose needs are your machine's and
//! your day's, all real:
//!   food      the updates waiting (a new kernel is a big meal): feed it and the update starts, in a terminal
//!   health    alerts, a red CI, a service down, a certificate about to expire, a container that keeps restarting
//!   rest      the hour (it sleeps when you are away, yawns at 2 in the morning), a CPU at full load, a hot machine
//!   clean     the disk: past 80% flies come round
//!   joy       the days with Pulse and their streak, the music, your agents' work done, and you: a click pets it
//! It walks on the floor of every scene, looks at the pointer, grows with the days you spend with Pulse (a second
//! antenna after a month), wears a helmet on Mars. A click opens its card: the five needs, why, and what to do.
//! [pet] enabled = false in pulse.toml (or PULSE_PET=off) sends it away; [pet] name = "..." names it.

use crate::draw;
use crate::hub::{get, list, num, text};
use crate::palette::{mix, Pal};
use crate::scene::{disc, glow, rect};
use gpui::Window;
use serde_json::{json, Value};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mood {
    Asleep,
    Sick,
    Hungry,
    Dirty,
    Tired,
    Hot,
    Happy,
    Fine,
}

pub struct Needs {
    pub food: (f32, String),
    pub health: (f32, String),
    pub rest: (f32, String),
    pub clean: (f32, String),
    pub joy: (f32, String),
    pub mood: Mood,
    pub updates: i64,
}

#[derive(Clone, Copy, PartialEq)]
enum Btn {
    Pet,
    Feed,
    Close,
}

pub struct Pet {
    on: Option<bool>,
    pub name: String,
    x: f32,
    target: f32,
    next_pick: f64,
    last_t: f64,
    petted: f64,
    pets: u64,
    hearts: Vec<(f32, f32, f64)>,
    card: Option<f64>,
    buttons: Vec<(f32, f32, f32, f32, Btn)>,
    area: Option<[f32; 4]>,
    body: Option<(f32, f32, f32)>,
    note: Option<(String, f64)>,
    loaded: bool,
    pos: Option<(f32, f32)>,
}

impl Default for Pet {
    fn default() -> Self {
        Pet { on: None, name: "Pip".into(), x: 0.5, target: 0.5, next_pick: 0.0, last_t: 0.0, petted: -1e9, pets: 0, hearts: Vec::new(), card: None, buttons: Vec::new(), area: None, body: None, note: None, loaded: false, pos: None }
    }
}

fn state_file() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let base = std::env::var("XDG_STATE_HOME").ok().filter(|x| !x.is_empty()).map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from(home).join(".local/state"));
    base.join("omarchy-pulse/native-pet.json")
}

/// What it needs, from the state: each need 0 (dire) to 1 (all good), with the reason in words.
pub fn needs(st: &Value, local_hour: i64, petted_ago: f64) -> Needs {
    let ups = num(st, &["updates", "n"]).unwrap_or(0.0) as i64 + num(st, &["updates", "aur"]).unwrap_or(0.0) as i64;
    let kernel = get(st, &["updates", "kernel"]).as_bool().unwrap_or(false);
    let food = (1.0 - (ups as f32 / 40.0).min(1.0) * 0.8 - if kernel { 0.15 } else { 0.0 }).clamp(0.0, 1.0);
    let food_w = if ups == 0 { "nothing to update: full".to_string() } else { format!("{ups} updates waiting{}", if kernel { ", a new kernel" } else { "" }) };

    let mut hl = 1.0f32;
    let mut why: Vec<String> = Vec::new();
    let crit_alerts = list(st, &["alerts"]).iter().filter(|a| text(a, &["level"]) != "info").count();
    if crit_alerts > 0 {
        hl -= 0.35;
        why.push(format!("{crit_alerts} alert{}", if crit_alerts > 1 { "s" } else { "" }));
    }
    let fails = list(st, &["git"]).iter().filter(|r| text(r, &["ci"]) == "fail").count();
    if fails > 0 {
        hl -= 0.25;
        why.push(format!("CI red on {fails}"));
    }
    let inet = get(st, &["internet"]);
    if text(inet, &["line", "level"]) == "crit" {
        hl -= 0.3;
        why.push("you are offline".into());
    }
    let down = num(inet, &["down"]).unwrap_or(0.0) as i64;
    if down > 0 {
        hl -= (0.15 * down as f32).min(0.3);
        why.push(format!("{down} service{} down", if down > 1 { "s" } else { "" }));
    }
    let exp = list(st, &["expiry"]).iter().filter(|e| text(e, &["level"]) == "crit").count();
    if exp > 0 {
        hl -= 0.2;
        why.push("a certificate expires".into());
    }
    let sick_bots = list(st, &["robots"]).iter().filter(|r| matches!(text(r, &["health"]).as_str(), "restart" | "failed" | "crit" | "unhealthy")).count();
    if sick_bots > 0 {
        hl -= 0.15;
        why.push(format!("{sick_bots} container{} restarting", if sick_bots > 1 { "s" } else { "" }));
    }
    let health = hl.clamp(0.0, 1.0);
    let health_w = if why.is_empty() { "all well".into() } else { why.join(" · ") };

    let cpu = num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32;
    let temp = num(st, &["sys", "temp"]).unwrap_or(0.0) as f32;
    let away = get(st, &["away"]).as_bool().unwrap_or(false);
    let late = (1..6).contains(&local_hour);
    let (rest, rest_w) = if away {
        (1.0, "asleep: you are away".to_string())
    } else if temp >= 85.0 {
        (0.35, format!("hot: {temp:.0}°"))
    } else if cpu >= 85.0 {
        (0.4, format!("worn out: CPU at {cpu:.0}%"))
    } else if late {
        (0.3, format!("it is {local_hour} in the morning"))
    } else {
        (1.0 - (cpu / 100.0) * 0.4, format!("CPU {cpu:.0}% · {temp:.0}°"))
    };

    let disk = num(st, &["sys", "disk"]).unwrap_or(0.0) as f32;
    let clean = (1.0 - ((disk - 75.0) / 20.0).clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let clean_w = format!("disk {disk:.0}% full");

    let days = num(st, &["growth", "n"]).unwrap_or(0.0);
    let streak = num(st, &["growth", "streak"]).unwrap_or(0.0);
    let today = num(st, &["growth", "today"]).unwrap_or(0.0);
    let music = get(st, &["music"]).as_bool().unwrap_or(false) || text(st, &["music", "status"]) == "Playing";
    let done = list(st, &["agents"]).iter().filter(|a| text(a, &["state"]) == "done").count();
    let failed = list(st, &["agents"]).iter().filter(|a| text(a, &["state"]) == "failed").count();
    let mut joy = 0.45 + (streak as f32 / 7.0).min(1.0) * 0.15 + if today > 0.0 { 0.1 } else { 0.0 } + if music { 0.1 } else { 0.0 } + (done as f32 * 0.05).min(0.1) - failed as f32 * 0.1;
    if petted_ago < 600.0 {
        joy += 0.3 * (1.0 - petted_ago as f32 / 600.0);
    }
    let joy = joy.clamp(0.0, 1.0);
    let mut jw = vec![format!("{days:.0} days together · streak {streak:.0}")];
    if music {
        jw.push("music!".into());
    }
    if petted_ago < 600.0 {
        jw.push("you petted it".into());
    }

    let mood = if away || (late && cpu < 30.0) {
        Mood::Asleep
    } else if health < 0.45 {
        Mood::Sick
    } else if food < 0.45 {
        Mood::Hungry
    } else if temp >= 85.0 {
        Mood::Hot
    } else if rest < 0.5 {
        Mood::Tired
    } else if clean < 0.4 {
        Mood::Dirty
    } else if joy > 0.72 {
        Mood::Happy
    } else {
        Mood::Fine
    };
    Needs { food: (food, food_w), health: (health, health_w), rest: (rest, rest_w), clean: (clean, clean_w), joy: (joy, jw.join(" · ")), mood, updates: ups }
}

fn mood_words(m: Mood) -> &'static str {
    match m {
        Mood::Asleep => "asleep",
        Mood::Sick => "not well",
        Mood::Hungry => "hungry",
        Mood::Dirty => "needs a clean",
        Mood::Tired => "tired",
        Mood::Hot => "too hot",
        Mood::Happy => "happy",
        Mood::Fine => "fine",
    }
}

impl Pet {
    fn load(&mut self) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        let conf = std::fs::read_to_string(crate::hub::config_dir().join("pulse.toml")).ok().and_then(|x| x.parse::<toml::Table>().ok());
        let pet = conf.as_ref().and_then(|c| c.get("pet")).and_then(|p| p.as_table()).cloned();
        let off_env = matches!(std::env::var("PULSE_PET").unwrap_or_default().to_lowercase().as_str(), "off" | "0" | "no" | "false");
        let off_conf = pet.as_ref().and_then(|p| p.get("enabled")).and_then(|v| v.as_bool()) == Some(false);
        self.on = Some(!(off_env || off_conf));
        if let Some(n) = pet.as_ref().and_then(|p| p.get("name")).and_then(|v| v.as_str()) {
            let n: String = n.chars().filter(|c| !c.is_control()).take(16).collect();
            if !n.trim().is_empty() {
                self.name = n.trim().to_string();
            }
        }
        if let Ok(s) = std::fs::read_to_string(state_file()) {
            if let Ok(v) = serde_json::from_str::<Value>(&s) {
                self.pets = v.get("pets").and_then(Value::as_u64).unwrap_or(0);
            }
        }
    }

    fn save(&self) {
        let f = state_file();
        if let Some(d) = f.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::write(f, json!({ "pets": self.pets }).to_string());
    }

    pub fn on(&mut self) -> bool {
        self.load();
        self.on.unwrap_or(true)
    }

    pub fn busy(&self) -> bool {
        self.card.is_some() || !self.hearts.is_empty()
    }

    /// A click: its card's buttons, its card, or Pip itself (opens the card and pets it).
    pub fn click(&mut self, x: f32, y: f32, t: f64) -> bool {
        if self.card.is_some() {
            for b in self.buttons.clone() {
                if x >= b.0 && x <= b.0 + b.2 && y >= b.1 && y <= b.1 + b.3 {
                    match b.4 {
                        Btn::Close => self.card = None,
                        Btn::Pet => self.pet(t),
                        Btn::Feed => {
                            let words = feed();
                            self.note = Some((words, t));
                        }
                    }
                    return true;
                }
            }
            if let Some(a) = self.area {
                if x >= a[0] && x <= a[0] + a[2] && y >= a[1] && y <= a[1] + a[3] {
                    return true;
                }
            }
        }
        if let Some((bx, by, r)) = self.body {
            if ((x - bx).powi(2) + (y - by).powi(2)).sqrt() <= r * 1.6 {
                self.pet(t);
                if self.card.is_none() {
                    self.card = Some(t);
                    self.note = None;
                    self.pos = None;
                }
                return true;
            }
        }
        if self.card.take().is_some() {
            return true;
        }
        false
    }

    fn pet(&mut self, t: f64) {
        self.petted = t;
        self.pets += 1;
        if let Some((bx, by, r)) = self.body {
            for k in 0..5 {
                self.hearts.push((bx + (k as f32 - 2.0) * r * 0.4, by - r, t + k as f64 * 0.12));
            }
        }
        self.save();
    }

    /// Pip, on the floor. `pointer`: where the pointer is and how long ago it moved; `mars`: it wears a helmet.
    #[allow(clippy::too_many_arguments)]
    pub fn paint(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f64, band: (f32, f32), local_hour: i64, pointer: Option<(f32, f32, f64)>, mars: bool, still: bool) {
        if !self.on() {
            self.body = None;
            return;
        }
        let dt = ((t - self.last_t) as f32).clamp(0.0, 0.5);
        self.last_t = t;
        let n = needs(st, local_hour, t - self.petted);
        let days = num(st, &["growth", "n"]).unwrap_or(0.0) as f32;
        let scale = if days < 7.0 { 0.72 } else if days < 30.0 { 0.86 } else { 1.0 };
        let r = h * 0.03 * scale;
        let floor = oy + h * 0.885;
        let asleep = n.mood == Mood::Asleep;
        // it wanders between the edges of the city, stops now and then; asleep it stays where it is
        let (lo, hi) = (((band.0 - ox) / w).max(0.22), ((band.1 - ox) / w).min(0.78));
        if t > self.next_pick && !asleep {
            let rnd = ((t * 7919.0).sin() * 43758.5453).fract().abs() as f32;
            self.target = lo + (hi - lo) * rnd;
            self.next_pick = t + 8.0 + (rnd as f64) * 14.0;
        }
        let pointing = pointer.filter(|p| t - p.2 < 4.0);
        let near = pointing.map(|p| ((p.0 - (ox + self.x * w)).abs() < r * 6.0) && (p.1 - floor).abs() < r * 6.0).unwrap_or(false);
        let speed = match n.mood { Mood::Happy => 0.018, Mood::Sick | Mood::Tired | Mood::Hungry => 0.006, _ => 0.011 };
        let go = if asleep || near || still { 0.0 } else { (self.target - self.x).clamp(-speed * dt, speed * dt) };
        self.x += go;
        let walking = go.abs() > 1e-5;
        let facing = if walking { go.signum() } else { pointing.map(|p| (p.0 - (ox + self.x * w)).signum()).unwrap_or(1.0) };
        let tf = t as f32;
        let hop = if still {
            0.0
        } else if near || t - self.petted < 2.0 {
            (tf * 9.0).sin().abs() * r * 0.9
        } else if walking {
            (tf * 7.0).sin().abs() * r * if n.mood == Mood::Happy { 0.45 } else { 0.22 }
        } else if asleep {
            0.0
        } else {
            0.0
        };
        let breathe = if still { 1.0 } else { 1.0 + 0.04 * (tf * if asleep { 1.2 } else { 2.4 }).sin() };
        let bx = ox + self.x * w;
        let by = floor - r - hop;
        self.body = Some((bx, by, r));
        // its colour: your theme's accent; green when it is not well
        let base = match n.mood { Mood::Sick => mix(pal.accent, pal.ok, 0.6), Mood::Hot => mix(pal.accent, pal.crit, 0.45), _ => pal.accent };
        let body = mix(base, pal.fg, 0.12);
        // shadow on the floor
        draw::ellipse(win, bx, floor + r * 0.05, r * (1.0 - hop / (r * 3.0)), r * 0.22, [0.0; 3], 0.35);
        glow(win, bx, by, r * 2.4, base, 0.18);
        // the antenna: its light beats with your pulse (your heart when connected, else the CPU)
        let bpm = num(st, &["heart"]).filter(|v| *v > 20.0).map(|v| v as f32).unwrap_or(50.0 + n.rest.0.min(1.0) * 10.0 + num(st, &["sys", "cpu"]).unwrap_or(0.0) as f32 * 0.6);
        let beat = if still { 0.6 } else { ((tf * bpm / 60.0).fract() * -6.0).exp() };
        let antennas: &[f32] = if days >= 30.0 { &[-0.35, 0.35] } else { &[0.0] };
        for a in antennas {
            let (x0, y0) = (bx + a * r * 0.6, by - r * 0.85 * breathe);
            let (x1, y1) = (x0 + a * r * 0.5 + facing * r * 0.1, y0 - r * 0.7);
            draw::line(win, &[(x0, y0), (x1, y1)], (r * 0.09).max(1.0), body, 1.0);
            disc(win, x1, y1, r * 0.16, mix(base, pal.fg, 0.5), 1.0);
            glow(win, x1, y1, r * (0.4 + 0.5 * beat), if n.health.0 < 0.45 { pal.crit } else { pal.fg }, 0.25 + 0.6 * beat);
        }
        // the body: a round blob that squashes as it lands
        let squash = if walking && !still { 1.0 + 0.06 * (tf * 14.0).sin() } else { 1.0 };
        draw::ellipse(win, bx, by, r * squash, r * breathe / squash, body, 1.0);
        draw::ellipse(win, bx - r * 0.3, by - r * 0.35, r * 0.35, r * 0.22, pal.fg, 0.22); // a shine
        // feet
        for s in [-1.0f32, 1.0] {
            let step = if walking && !still { (tf * 7.0 + s * 1.6).sin() * r * 0.15 } else { 0.0 };
            draw::ellipse(win, bx + s * r * 0.45 + step, floor - hop * 0.2 - r * 0.08, r * 0.28, r * 0.14, mix(body, [0.0; 3], 0.35), 1.0);
        }
        // the eyes: they follow the pointer (or look where it walks); closed asleep, half closed tired
        let (ex, ey) = match pointing {
            Some(p) => {
                let (dx, dy) = (p.0 - bx, p.1 - by);
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                (dx / d * r * 0.09, dy / d * r * 0.07)
            }
            None => (facing * r * 0.08, 0.0),
        };
        let blink = !still && (tf * 0.37).fract() < 0.03;
        for s in [-1.0f32, 1.0] {
            let (cx0, cy0) = (bx + s * r * 0.36 + facing * r * 0.1, by - r * 0.12);
            if asleep || blink {
                draw::arc(win, cx0, cy0, r * 0.17, r * 0.1, 0.1, std::f32::consts::PI - 0.1, (r * 0.08).max(1.0), pal.bg, 0.95);
            } else if n.mood == Mood::Happy {
                draw::arc(win, cx0, cy0 + r * 0.06, r * 0.17, r * 0.15, std::f32::consts::PI + 0.2, std::f32::consts::TAU - 0.2, (r * 0.09).max(1.0), pal.bg, 0.95);
            } else {
                let lid = matches!(n.mood, Mood::Tired | Mood::Sick);
                draw::ellipse(win, cx0, cy0, r * 0.19, r * if lid { 0.13 } else { 0.22 }, pal.fg, 1.0);
                disc(win, cx0 + ex, cy0 + ey, r * 0.1, pal.bg, 1.0);
                disc(win, cx0 + ex + r * 0.035, cy0 + ey - r * 0.04, r * 0.035, pal.fg, 1.0);
                if lid {
                    rect(win, cx0 - r * 0.2, cy0 - r * 0.22, r * 0.4, r * 0.12, body, 1.0);
                }
            }
        }
        // the mouth
        let (mx, my) = (bx + facing * r * 0.1, by + r * 0.32);
        match n.mood {
            Mood::Happy | Mood::Fine => draw::arc(win, mx, my - r * 0.06, r * 0.2, r * if n.mood == Mood::Happy { 0.16 } else { 0.08 }, 0.15, std::f32::consts::PI - 0.15, (r * 0.08).max(1.0), pal.bg, 0.9),
            Mood::Hungry => draw::ellipse(win, mx, my, r * 0.13, r * 0.15, pal.bg, 0.9),
            Mood::Asleep => {}
            _ => draw::arc(win, mx, my + r * 0.08, r * 0.18, r * 0.08, std::f32::consts::PI + 0.2, std::f32::consts::TAU - 0.2, (r * 0.08).max(1.0), pal.bg, 0.9),
        }
        // what goes with its mood
        match n.mood {
            Mood::Asleep if !still => {
                for k in 0..3 {
                    let a = ((tf * 0.35 + k as f32 / 3.0).fract()) as f32;
                    draw::text(win, cx, font, "z", bx + r * (0.8 + a * 1.2), by - r * (1.0 + a * 2.2), r * (0.45 + a * 0.4), 600.0, pal.fg, 0.8 * (1.0 - a), 0);
                }
            }
            Mood::Dirty if !still => {
                for k in 0..3 {
                    let a = tf * (2.0 + k as f32 * 0.7) + k as f32 * 2.1;
                    disc(win, bx + a.cos() * r * 1.5, by - r * 0.6 + (a * 1.7).sin() * r * 0.6, (r * 0.07).max(1.2), pal.bg, 0.9);
                }
            }
            Mood::Hot | Mood::Tired if !still => {
                let a = (tf * 0.8).fract();
                draw::ellipse(win, bx + r * 0.85, by - r * 0.4 + a * r * 0.8, r * 0.08, r * 0.13, mix(pal.accent2, pal.fg, 0.5), 1.0 - a);
            }
            Mood::Hungry => {
                // its bowl, empty, beside it
                let bwx = bx - facing * r * 1.7;
                draw::arc_fill(win, bwx, floor - r * 0.18, r * 0.42, r * 0.28, 0.0, 0.0, std::f32::consts::PI, mix(pal.warn, pal.bg, 0.3), 1.0);
                rect(win, bwx - r * 0.42, floor - r * 0.2, r * 0.84, r * 0.05, pal.warn, 1.0);
            }
            Mood::Sick => {
                // a thermometer in its mouth
                draw::line(win, &[(mx, my), (mx + facing * r * 0.55, my - r * 0.2)], (r * 0.08).max(1.0), pal.fg, 0.9);
                disc(win, mx + facing * r * 0.55, my - r * 0.2, r * 0.07, pal.crit, 1.0);
            }
            _ => {}
        }
        if mars {
            // a helmet: there is no air out there
            draw::arc(win, bx, by - r * 0.1, r * 1.45, r * 1.45, std::f32::consts::PI * 1.05, std::f32::consts::TAU * 0.975, (r * 0.07).max(1.0), [0.8, 0.95, 1.0], 0.55);
            draw::ellipse(win, bx, by - r * 0.1, r * 1.42, r * 1.42, [0.7, 0.9, 1.0], 0.07);
            draw::ellipse(win, bx - r * 0.6, by - r * 0.85, r * 0.28, r * 0.14, [1.0; 3], 0.35);
        }
        // hearts when you pet it
        self.hearts.retain(|hh| t - hh.2 < 1.6);
        for hh in &self.hearts {
            let a = ((t - hh.2) as f32 / 1.6).clamp(0.0, 1.0);
            if a <= 0.0 {
                continue;
            }
            let (x, y, s) = (hh.0 + (a * 9.0).sin() * r * 0.2, hh.1 - a * r * 2.5, r * 0.3);
            disc(win, x - s * 0.5, y, s * 0.55, pal.crit, 1.0 - a);
            disc(win, x + s * 0.5, y, s * 0.55, pal.crit, 1.0 - a);
            draw::poly(win, &[(x - s * 1.02, y + s * 0.15), (x + s * 1.02, y + s * 0.15), (x, y + s * 1.2)], pal.crit, 1.0 - a);
        }
        // its name, small, under it
        draw::text(win, cx, font, &self.name, bx, floor + r * 0.75, (r * 0.48).max(9.0), 400.0, pal.fg, 0.5, 0);
        self.card(win, cx, font, pal, &n, ox, oy, w, h, t, bx, by, r);
    }

    #[allow(clippy::too_many_arguments)]
    fn card(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, pal: &Pal, n: &Needs, ox: f32, oy: f32, w: f32, h: f32, t: f64, bx: f32, by: f32, r: f32) {
        self.buttons.clear();
        self.area = None;
        let Some(at) = self.card else { return };
        if t - at > 60.0 {
            self.card = None;
            return;
        }
        let rem = crate::ui::rem(w, h);
        let (fs, small) = ((0.95 * rem).round(), (0.8 * rem).round());
        let k = ((t - at) as f32 / 0.18).clamp(0.0, 1.0);
        let pad = 0.8 * rem;
        let rows: [(&str, &(f32, String)); 5] = [("food", &n.food), ("health", &n.health), ("rest", &n.rest), ("clean", &n.clean), ("joy", &n.joy)];
        let label_w = rows.iter().map(|r2| draw::measure(win, font, r2.0, small, 400.0)).fold(0.0f32, f32::max);
        let bar_w = 6.0 * rem;
        let why_w = rows.iter().map(|r2| draw::measure(win, font, &r2.1 .1, small, 400.0)).fold(0.0f32, f32::max);
        let mut btns: Vec<(String, Btn)> = vec![("♥ pet".into(), Btn::Pet)];
        if n.updates > 0 {
            btns.push(("⟳ feed: update".into(), Btn::Feed));
        }
        btns.push(("✕".into(), Btn::Close));
        let bws: Vec<f32> = btns.iter().map(|b| draw::measure(win, font, &b.0, small, 400.0) + 1.2 * rem).collect();
        let title = format!("{} · {}", self.name, mood_words(n.mood));
        let content_w = (label_w + 0.6 * rem + bar_w + 0.6 * rem + why_w).max(draw::measure(win, font, &title, fs, 600.0)).max(bws.iter().sum::<f32>() + 0.5 * rem * (bws.len() as f32 - 1.0));
        let cw = (content_w + pad * 2.0).min(w * 0.6);
        let row_h = small * 1.6;
        let note = self.note.as_ref().filter(|nn| t - nn.1 < 8.0).map(|nn| nn.0.clone());
        let ch = pad + fs * 1.5 + small * 1.4 + rows.len() as f32 * row_h + if note.is_some() { small * 1.5 } else { 0.0 } + 0.5 * rem + small * 1.9 + pad;
        let (x0, y0) = match self.pos {
            Some((x, y)) => (x, y.min(oy + h - ch - 0.6 * rem)),
            None => {
                let p = crate::agents::place(cw, ch, bx, by, r * 1.8, ox, oy, w, h, 0.6 * rem);
                self.pos = Some(p);
                p
            }
        };
        rect(win, x0, y0, cw, ch, pal.bg, 0.93 * k);
        draw::frame(win, x0, y0, cw, ch, 1.0, mix(pal.bg, pal.accent, 0.8), k);
        let mut y = y0 + pad;
        draw::text(win, cx, font, &title, x0 + pad, y + fs * 1.05, fs, 600.0, pal.fg, k, -1);
        y += fs * 1.5;
        draw::text(win, cx, font, &format!("petted {} times · it lives on what your machine and your day are", self.pets), x0 + pad, y + small, small, 400.0, pal.fg, 0.5 * k, -1);
        y += small * 1.4;
        for (lab, (v, why)) in rows {
            let c = if *v < 0.4 { pal.crit } else if *v < 0.7 { pal.warn } else { pal.ok };
            draw::text(win, cx, font, lab, x0 + pad, y + small * 1.1, small, 400.0, pal.fg, 0.7 * k, -1);
            let bxx = x0 + pad + label_w + 0.6 * rem;
            rect(win, bxx, y + small * 0.45, bar_w, small * 0.6, pal.fg, 0.1 * k);
            rect(win, bxx, y + small * 0.45, bar_w * v.clamp(0.03, 1.0), small * 0.6, c, 0.9 * k);
            draw::text(win, cx, font, why, bxx + bar_w + 0.6 * rem, y + small * 1.1, small, 400.0, pal.fg, 0.6 * k, -1);
            y += row_h;
        }
        if let Some(nn) = note {
            draw::text(win, cx, font, &nn, x0 + pad, y + small * 1.1, small, 400.0, pal.accent2, k, -1);
            y += small * 1.5;
        }
        y += 0.5 * rem;
        let bh = small * 1.9;
        let mut x = x0 + pad;
        for ((lab, b), bw) in btns.iter().zip(bws) {
            let ac = match b { Btn::Pet => pal.crit, Btn::Feed => pal.ok, Btn::Close => pal.dim };
            rect(win, x, y, bw, bh, mix(pal.bg, ac, 0.18), k);
            draw::frame(win, x, y, bw, bh, 1.0, mix(pal.bg, ac, 0.7), k);
            draw::text(win, cx, font, lab, x + bw / 2.0, y + bh * 0.5 + small * 0.35, small, 400.0, pal.fg, k, 0);
            self.buttons.push((x, y, bw, bh, *b));
            x += bw + 0.5 * rem;
        }
        self.area = Some([x0, y0, cw, ch]);
    }
}

/// Feeding it is updating: Omarchy's own update, in a terminal of its own, where you confirm it as always.
fn feed() -> String {
    let has = |c: &str| std::env::var("PATH").unwrap_or_default().split(':').any(|d| std::path::Path::new(d).join(c).is_file());
    if !has("omarchy-update") {
        return "omarchy-update is not here".into();
    }
    let run = if has("omarchy-launch-floating-terminal-with-presentation") {
        std::process::Command::new("omarchy-launch-floating-terminal-with-presentation").arg("omarchy-update").spawn()
    } else if has("xdg-terminal-exec") {
        std::process::Command::new("xdg-terminal-exec").arg("omarchy-update").spawn()
    } else {
        return "no terminal to run omarchy-update in".into();
    };
    match run {
        Ok(_) => "the update opened in a terminal: confirm it there".into(),
        Err(e) => format!("could not start it: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moods() {
        let st = json!({ "updates": { "n": 30, "kernel": true }, "sys": { "cpu": 20, "temp": 50, "disk": 60 }, "growth": { "n": 10, "streak": 3 } });
        assert_eq!(needs(&st, 14, 1e9).mood, Mood::Hungry);
        let st = json!({ "updates": { "n": 0 }, "sys": { "cpu": 5, "temp": 50, "disk": 60 }, "away": true });
        assert_eq!(needs(&st, 14, 1e9).mood, Mood::Asleep);
        let st = json!({ "updates": { "n": 0 }, "sys": { "cpu": 5, "temp": 50, "disk": 60 }, "alerts": [{ "level": "crit" }], "git": [{ "ci": "fail" }] });
        assert_eq!(needs(&st, 14, 1e9).mood, Mood::Sick);
        let st = json!({ "updates": { "n": 0 }, "sys": { "cpu": 5, "temp": 50, "disk": 60 }, "growth": { "n": 40, "streak": 9, "today": 2 } });
        assert_eq!(needs(&st, 14, 10.0).mood, Mood::Happy);
    }
}
