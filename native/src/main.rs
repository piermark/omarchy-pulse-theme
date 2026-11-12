//! pulse-native: Omarchy Pulse drawn natively, in Rust with GPUI. (c) 2026 Marco Porro (x.com/MarcoPorro), MIT.
//!
//! A parallel renderer, not a replacement: the web page stays the default and is what runs on OmarchyTV. This one
//! reads the same hub (`omarchy-pulse`, /api/state, the same token and pulse.toml) and paints the wallpaper on the
//! GPU as a Wayland layer-shell surface behind your windows, with no browser underneath.
//!
//!     pulse-native            the wallpaper (Hyprland, Sway: any compositor with wlr-layer-shell)
//!     pulse-native --window   an ordinary window, to look at it or to develop
//!     pulse-native --scene city   the first world of pulse-native: a far city under a warm dusk
//!     pulse-native --url http://127.0.0.1:47800 --fps 30

mod backdrop;
mod cities_data;
mod city;
mod design;
mod draw;
mod drones;
mod extra;
mod heavens;
mod hub;
mod life;
mod look;
mod modes;
mod palette;
mod perf;
mod picture;
mod scene;
mod ui;
mod sound;
mod rockets;
mod sky;
mod floor;
mod sphere;
mod street;
mod weather;
mod worlds;
mod xf;

use gpui::{
    canvas, div, layer_shell::*, point, prelude::*, px, size, App, Bounds, Context, FontWeight, LinuxWindowBackground, MouseButton, MouseDownEvent,
    MouseMoveEvent, ObjectFit, QuitMode, SharedString, Size, Window, WindowBounds, WindowKind, WindowOptions,
};
use std::sync::{Arc, Mutex};
use hub::{get, list, num, text, FxQueue, Shared};
use palette::{col, Pal, C};
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FONT: &str = "CaskaydiaMono Nerd Font";

struct Pulse {
    hub: Shared,
    live: FxQueue,
    /// --scene on the command line; empty = the hub's own (ui.scene), so both renderers show the same scene
    forced: bool,
    world: Rc<RefCell<scene::World>>,
    city: Rc<RefCell<city::World>>,
    scene: String,
    marks: Rc<RefCell<Vec<scene::PlaneMark>>>,
    mem: Rc<RefCell<design::Mem>>,
    drawn: Rc<RefCell<(Vec<design::Words>, Vec<design::Picture>)>>,
    /// the pictures of the design on show, by asset id
    assets: (String, std::collections::HashMap<String, std::sync::Arc<gpui::Image>>),
    reacted: f64,
    started: Instant,
    font: SharedString,
    /// the hub's address, for what a click sends back
    base: String,
    /// the first monitor: alerts, the digest and the sound live there only (as on the web page)
    primary: bool,
    /// this monitor's key in the hub's `mons` ("x,y" of its corner), shared with the pacing loop
    monkey: Arc<Mutex<String>>,
    origin: (f32, f32),
    sound: sound::Sound,
    /// your own wallpaper: its version, the picture, its colours when you asked for them
    wallpaper: (String, Option<Arc<gpui::Image>>, Option<picture::Colors>),
    /// the cover of the song that plays: its version and its colours
    cover: (String, Option<picture::Colors>),
    seen_alert: i64,
    incoming: Option<(f64, Value)>,
    seen_digest: i64,
    digest: Option<(f64, Value)>,
    /// a storm you clicked once: click again within five seconds to acknowledge its problems
    ack: Option<(String, f64)>,
    last_cursor: Option<Instant>,
    sounded: f64,
    /// camera panels: the frame shown now, by panel id
    cams: std::collections::HashMap<String, (String, Arc<gpui::Image>)>,
    /// a design's pictures as they came, and the tinted ones made from them
    raw_assets: std::collections::HashMap<String, Vec<u8>>,
    tinted: std::collections::HashMap<String, Option<Arc<gpui::Image>>>,
    /// omarchy-pulse layout edit: the layout being arranged, the panel being dragged, where each panel is on the screen
    draft: Option<serde_json::Map<String, Value>>,
    drag: Option<String>,
    boxes: Rc<RefCell<std::collections::HashMap<String, (String, f32, f32)>>>,
    /// pulse.js frame(): a PC that cannot keep up gets fewer frames (the cap), tried again at full speed a minute later
    cap: Rc<std::cell::Cell<(u64, u32, f64)>>,
    /// what the last frame cost on the CPU (ms): the panels, then the scene
    cost: Rc<std::cell::Cell<(f64, f64)>>,
    /// every panel on the screen now, for the planes
    ui_boxes: Rc<RefCell<Vec<[f32; 4]>>>,
    /// text panels: the text and when it started to be typed out
    typed: std::collections::HashMap<String, (String, f64)>,
    was_away: bool,
    /// the first camera's frame as it came (the sphere shows it in its turn)
    cam_bytes: Option<(String, Arc<Vec<u8>>)>,
}

impl Pulse {
    #[allow(clippy::too_many_arguments)]
    fn new(hub: Shared, live: FxQueue, fps: u64, font: String, scene: String, base: String, primary: bool, cx: &mut Context<Self>) -> Self {
        let hub2 = hub.clone();
        let monkey = Arc::new(Mutex::new(String::new()));
        let (key2, forced_scene) = (monkey.clone(), scene.clone());
        cx.spawn(async move |this, cx| loop {
            // how often to draw, as the web page decides it: not at all while this monitor is covered, once a second
            // under a fullscreen window, slowly with windows open, on battery, away, or on a still scene
            let (covered, want) = {
                let st = hub2.lock().map(|s| s.state.clone()).unwrap_or_default();
                let key = key2.lock().map(|k| k.clone()).unwrap_or_default();
                pace(&st, &key, fps, &forced_scene, primary)
            };
            let mut want = want;
            if !covered {
                match this.update(cx, |this, cx| {
                    cx.notify();
                    // pulse.js: when a frame costs more than half its time, 70% of the frames; full speed again after a minute
                    let (mut cap, mut slow, mut at) = this.cap.get();
                    let (c1, c2) = this.cost.get();
                    let now = this.started.elapsed().as_secs_f64();
                    let budget = 1000.0 / want.min(cap).max(1) as f64;
                    if c1 + c2 > budget * 0.5 && want.min(cap) > 6 {
                        slow += 1;
                        if slow as u64 > 3 * want.min(cap) {
                            cap = ((want.min(cap) as f64 * 0.7) as u64).max(6);
                            (slow, at) = (0, now);
                        }
                    } else {
                        slow = slow.saturating_sub(1);
                    }
                    if at > 0.0 && now - at > 60.0 {
                        (cap, at) = (60, 0.0);
                    }
                    this.cap.set((cap, slow, at));
                    cap
                }) {
                    Ok(cap) => want = want.min(cap),
                    Err(_) => break,
                }
            }
            if covered && this.update(cx, |_, _| ()).is_err() {
                break;
            }
            cx.background_executor().timer(Duration::from_millis(if covered { 1000 } else { 1000 / want.clamp(1, 60) })).await;
        })
        .detach();
        let world = scene::World::new();
        Pulse { hub, live, forced: !scene.is_empty(), scene, city: Rc::new(RefCell::new(city::World::new())), world: Rc::new(RefCell::new(world)), marks: Rc::new(RefCell::new(Vec::new())), mem: Rc::new(RefCell::new(design::Mem::default())), drawn: Rc::new(RefCell::new((Vec::new(), Vec::new()))), assets: (String::new(), Default::default()), reacted: 0.0, started: Instant::now(), font: font.into(), base, primary, monkey, origin: (0.0, 0.0), sound: sound::Sound::new(), wallpaper: (String::new(), None, None), cover: (String::new(), None), seen_alert: -1, incoming: None, seen_digest: 0, digest: None, ack: None, last_cursor: None, sounded: 0.0, cams: Default::default(), raw_assets: Default::default(), tinted: Default::default(), draft: None, drag: None, boxes: Default::default(), ui_boxes: Default::default(), cap: Rc::new(std::cell::Cell::new((60, 0, 0.0))), cost: Default::default(), typed: Default::default(), was_away: false, cam_bytes: None }
    }

    /// A click on the wallpaper: a storm over a failing panel (twice: acknowledge it), or something of the scene,
    /// which answers with a line of text.
    fn click(&mut self, x: f32, y: f32) {
        let t = self.started.elapsed().as_secs_f64();
        let mut world = self.world.borrow_mut();
        let storm = world.more.storms.iter().find(|s| ((x - s.1).powi(2) + (y - s.2).powi(2)).sqrt() <= s.3).cloned();
        if let Some((id, _, _, _, title)) = storm {
            if matches!(&self.ack, Some((was, at)) if *was == id && t - at < 5.0) {
                self.ack = None;
                hub::post(&self.base, &format!("/api/ack/{}", urlenc(&id)));
                world.say = Some((t, format!("acknowledged · {title}")));
            } else {
                self.ack = Some((id, t));
                world.say = Some((t, format!("click again to acknowledge · {title}")));
            }
            return;
        }
        let hit = world.hits.iter().rev().find(|h| (x - h.x).abs() <= h.r && (y - h.y).abs() <= h.r).map(|h| h.text.clone());
        if let Some(words) = hit {
            world.say = Some((t, words));
            return;
        }
        // a scene of the editor: the topmost shape under the pointer says its line, and the actions that wait for
        // a click on it fire
        let found = design::click(&self.mem.borrow(), x, y);
        if let Some((words, id)) = found {
            if !words.is_empty() {
                world.say = Some((t, words));
            }
            let (design, st) = match self.hub.lock() {
                Ok(s) => (s.design.clone(), s.state.clone()),
                Err(_) => return,
            };
            if let Some((_, d)) = design {
                let scene_st = if get(&st, &["scene"]).is_object() { get(&st, &["scene"]).clone() } else { (*st).clone() };
                for ask in design::react_to(&d, &scene_st, &mut self.mem.borrow_mut(), t, Some(&id)) {
                    let kind = if ask.kind == "text" { "say" } else { ask.kind.as_str() };
                    world.fx(kind, &ask.text, false, t);
                }
            }
        }
    }

    /// The end of a drag in `omarchy-pulse layout edit`: the column under the pointer, and the panel it goes before.
    fn drop_panel(&mut self, x: f32, y: f32, w: f32) {
        let Some(id) = self.drag.take() else { return };
        let Some(draft) = self.draft.as_mut() else { return };
        let slot = if x < w * 0.3 { "left" } else if x > w * 0.7 { "right" } else { "center" };
        let mut sibs: Vec<(String, f32)> = self.boxes.borrow().iter().filter(|(k, v)| **k != id && v.0 == slot).map(|(k, v)| (k.clone(), v.1 + v.2 / 2.0)).collect();
        sibs.sort_by(|a, b| a.1.total_cmp(&b.1));
        let at = sibs.iter().position(|sb| y < sb.1).unwrap_or(sibs.len());
        sibs.insert(at, (id, 0.0));
        for (k, (sid, _)) in sibs.iter().enumerate() {
            let o = draft.entry(sid.clone()).or_insert_with(|| serde_json::json!({}));
            o["slot"] = serde_json::json!(slot);
            o["order"] = serde_json::json!(k as f64 * 10.0);
        }
    }
}

fn urlenc(s: &str) -> String {
    s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

/// Is this monitor covered (nothing to draw), and how many frames a second otherwise: pulse.js's Scene.mode.
fn pace(st: &Value, key: &str, fps: u64, forced_scene: &str, primary: bool) -> (bool, u64) {
    let m = get(st, &["mons", key]);
    let ui = get(st, &["ui"]);
    let covered = get(ui, &["pause_when_covered"]).as_bool().unwrap_or(true) && get(m, &["covered"]).as_bool().unwrap_or(false);
    let full = get(m, &["fullscreen"]).as_bool().unwrap_or(false);
    let busy = num(m, &["windows"]).unwrap_or(0.0) > 0.0 && get(ui, &["dim_with_windows"]).as_bool().unwrap_or(true);
    let on_battery = text(st, &["sys", "bat", "status"]) == "Discharging";
    let base = if fps > 0 { fps } else { num(ui, &["fps"]).map(|v| v as u64).filter(|v| *v > 0).unwrap_or(20) };
    let mut want = if full { 1 } else if busy { 8 } else { base.min(if on_battery { 12 } else { 60 }) };
    let kind = if forced_scene.is_empty() { text(ui, &["scene"]) } else { forced_scene.to_string() };
    if kind == "calm" {
        want = want.min(10);
    }
    if get(ui, &["motion"]).as_str() == Some("reduced") {
        want = want.min(4); // nothing moves: a few frames a second keep the clock and the data fresh
    }
    if get(st, &["away"]).as_bool().unwrap_or(false) {
        want = want.min(8); // nobody is looking: save power
    }
    if get(ui, &["wallpaper", "mode"]).as_str() == Some("panels") {
        want = want.min(2); // a still picture: almost no work
    }
    if !primary && get(ui, &["secondary"]).as_str() == Some("simple") {
        want = want.min(12); // a calm, light second screen
    }
    (covered, want.max(1))
}

fn clock(st: &Value) -> (String, String, String) {
    let twelve = get(st, &["ui", "clock"]).as_str() == Some("12h");
    // the hub's clock when it is there (so both renderers agree), else this machine's
    let t = num(st, &["t"]).unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0));
    let local = local_offset_secs();
    let s = (t as i64) + local;
    let (d, sod) = (s.div_euclid(86400), s.rem_euclid(86400));
    let (hh, mm, ss) = (sod / 3600, (sod / 60) % 60, sod % 60);
    // civil date from days since 1970 (Howard Hinnant's algorithm)
    let z = d + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    let wd = ["THU", "FRI", "SAT", "SUN", "MON", "TUE", "WED"][d.rem_euclid(7) as usize];
    let mon = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"][(month - 1) as usize];
    // the ISO week: the week of this day's Thursday
    let thursday = d + 3 - (d + 3).rem_euclid(7);
    let jan1 = {
        // days since 1970 of the first of January of the Thursday's year
        let ty = { let z2 = thursday + 719468; let era2 = z2.div_euclid(146097); let doe2 = z2.rem_euclid(146097); let yoe2 = (doe2 - doe2 / 1460 + doe2 / 36524 - doe2 / 146096) / 365; let doy2 = doe2 - (365 * yoe2 + yoe2 / 4 - yoe2 / 100); yoe2 + era2 * 400 + if (5 * doy2 + 2) / 153 >= 10 { 1 } else { 0 } };
        let y2 = ty - 1;
        let era3 = y2.div_euclid(400);
        let yoe3 = y2.rem_euclid(400);
        era3 * 146097 + yoe3 * 365 + yoe3 / 4 - yoe3 / 100 + 306 - 719468
    };
    let week = (thursday - jan1) / 7 + 1;
    let h12 = if twelve { if hh % 12 == 0 { 12 } else { hh % 12 } } else { hh };
    (format!("{h12:02}:{mm:02}"), format!("{ss:02}"), format!("{wd} {day} {mon} {year} · W{week}"))
}

/// The local time's offset, from libc (the TZ the session runs in); 0 if it can't be read.
pub fn local_offset_secs() -> i64 {
    unsafe {
        let now = libc_time();
        let mut tm: Tm = std::mem::zeroed();
        if localtime_r(&now, &mut tm).is_null() {
            0
        } else {
            tm.tm_gmtoff
        }
    }
}

#[repr(C)]
struct Tm {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,
    tm_year: i32,
    tm_wday: i32,
    tm_yday: i32,
    tm_isdst: i32,
    tm_gmtoff: i64,
    tm_zone: *const i8,
}
extern "C" {
    fn time(t: *mut i64) -> i64;
    fn localtime_r(t: *const i64, out: *mut Tm) -> *mut Tm;
}
unsafe fn libc_time() -> i64 {
    time(std::ptr::null_mut())
}



fn level_color(level: &str, pal: &Pal) -> C {
    match level {
        "ok" => pal.ok,
        "warn" => pal.warn,
        "crit" => pal.crit,
        "info" => pal.accent,
        _ => pal.dim,
    }
}





impl Render for Pulse {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let a = Instant::now();
        let el = crate::perf::time("render", || self.render_inner(window, cx));
        let c = self.cost.get();
        self.cost.set((a.elapsed().as_secs_f64() * 1000.0, c.1));
        el
    }
}

impl Pulse {
    fn render_inner(&mut self, window: &mut Window, cx: &mut Context<Self>) -> gpui::Div {
        let (st, snap_at, snap_error, snap_design, snap_handwritten) = {
            let Ok(snap) = self.hub.lock() else { return div() };
            // your own wallpaper and the song's cover: decoded once for each version
            let wkey = snap.wall.as_ref().map(|x| x.0.clone()).unwrap_or_default();
            if wkey != self.wallpaper.0 {
                self.wallpaper = (wkey, snap.wall.as_ref().map(|x| picture::image(&x.1)), snap.wall.as_ref().and_then(|x| picture::colors(&x.1)));
            }
            let ckey = snap.cover.as_ref().map(|x| x.0.clone()).unwrap_or_default();
            if ckey != self.cover.0 {
                self.cover = (ckey, snap.cover.as_ref().and_then(|x| picture::colors(&x.1)));
            }
            (snap.state.clone(), snap.at, snap.error.clone(), snap.design.clone(), snap.handwritten.clone())
        };
        let mut pal = Pal::from_state(&st);
        {
            // camera panels: the frame the hub fetched, decoded once
            if let Ok(snap) = self.hub.lock() {
                self.cams.retain(|k, _| snap.cams.contains_key(k));
                for (id, (frame, bytes)) in snap.cams.iter() {
                    if self.cams.get(id).map(|c| &c.0 != frame).unwrap_or(true) {
                        self.cams.insert(id.clone(), (frame.clone(), picture::image(bytes)));
                    }
                }
            }
            self.world.borrow_mut().has_cam = !self.cams.is_empty();
            // pulse.js: the panel "cam", or the first picture panel; never while the screen is shared
            self.cam_bytes = if get(&st, &["privacy"]).as_bool().unwrap_or(false) { None } else { self.hub.lock().ok().and_then(|snap| snap.cams.get("cam").or_else(|| snap.cams.values().next()).map(|c| (c.0.clone(), c.1.clone()))) };
        }
        let editing = get(&st, &["layoutEdit"]).as_bool().unwrap_or(false) && self.primary;
        if editing && self.draft.is_none() {
            self.draft = Some(get(&st, &["layout"]).as_object().cloned().unwrap_or_default());
        } else if !editing {
            (self.draft, self.drag) = (None, None);
        }
        let own_wall = get(&st, &["ui", "wallpaper"]).is_object() && self.wallpaper.1.is_some();
        if let (true, Some(c), Some("picture")) = (own_wall, self.wallpaper.2, get(&st, &["ui", "wallpaper", "colors"]).as_str()) {
            (pal.bg, pal.fg, pal.accent, pal.accent2) = (c.bg, c.fg, c.accent, c.accent2);
        }
        if let (true, Some(c)) = (get(&st, &["ui", "musicCover"]).is_object(), self.cover.1) {
            (pal.accent, pal.accent2) = (c.accent, c.accent2); // while a song plays, the accents come from its cover
        }
        let vp = window.viewport_size();
        let (w, h) = (f32::from(vp.width), f32::from(vp.height));
        let u = (h / 1080.0).max(0.5);
        let t = self.started.elapsed().as_secs_f64();
        let (hm, sec, date) = clock(&st);
        let (hm0, date0) = (hm.clone(), date.clone());
        // ---- this monitor, as the hub knows it: by the place of its corner on the desktop
        if let Some(d) = window.display(cx) {
            let o = d.bounds().origin;
            self.origin = (f32::from(o.x), f32::from(o.y));
        }
        let key = format!("{},{}", self.origin.0 as i64, self.origin.1 as i64);
        if let Ok(mut k) = self.monkey.lock() {
            if *k != key { *k = key.clone() }
        }
        let mon = get(&st, &["mons", &key]).clone();
        let full = get(&mon, &["fullscreen"]).as_bool().unwrap_or(false);
        let desk_empty = num(&mon, &["windows"]).unwrap_or(0.0) <= 0.0;
        let busy = !desk_empty && get(&st, &["ui", "dim_with_windows"]).as_bool().unwrap_or(true);
        let simple = !self.primary && get(&st, &["ui", "secondary"]).as_str() == Some("simple");
        let away = get(&st, &["away"]).as_bool().unwrap_or(false);
        let hub_now = num(&st, &["t"]).unwrap_or(0.0);
        // ---- the hub's live channel: effects (a launch, fireworks, lightning...), the music's bars, the pointer
        let (fx, audio, cursor): (Vec<hub::Fx>, Vec<f32>, Option<((f32, f32), Instant)>) = self
            .live
            .lock()
            .map(|mut q| (q.fx.drain(..).collect(), if q.audio_at.map(|a| a.elapsed() < Duration::from_secs(1)).unwrap_or(false) { q.audio.clone() } else { Vec::new() }, q.cursor.zip(q.cursor_at)))
            .unwrap_or_default();
        let quiet = get(&st, &["meeting"]).as_bool().unwrap_or(false);
        {
            let mut world = self.world.borrow_mut();
            for f in &fx {
                world.fx_raw(if quiet { "text" } else { &f.kind }, &f.text, f.small, t, &f.raw);
                if self.primary && !quiet {
                    match f.kind.as_str() {
                        "chime" => self.sound.chime(&[659.0, 784.0, 988.0]),
                        "train" => self.sound.chime(&[659.0, 523.0, 440.0]),
                        "starlink" => self.sound.chime(&[880.0, 1175.0]),
                        "lightning" => self.sound.thunder(1.5, 0.5),
                        "quake" => self.sound.rumble(0.5),
                        _ => {}
                    }
                }
            }
            for sd in std::mem::take(&mut world.sounds) {
                if self.primary {
                    match sd {
                        "thunder" => { let km = 1.0 + (t * 7.3).fract() as f32 * 6.0; self.sound.thunder(km / 0.343, (1.0 - km / 8.0).max(0.2)) } // sound: 3 s per km
                        _ => self.sound.chime(&[659.0, 784.0, 988.0]),
                    }
                }
            }
            world.audio = audio;
            world.desk_empty = desk_empty;
            world.hm = hm0.clone();
            // "THU NOV 12", as toDateString() says it
            world.date_short = { let p: Vec<&str> = date0.split(' ').collect(); if p.len() >= 3 { format!("{} {} {}", p[0], p[2], p[1]) } else { date0.clone() } };
            world.typing = self.typed.values().any(|(full, since)| (((t - since) / 0.04) as usize).saturating_mul(3) < full.chars().count());
            if self.was_away && !away {
                world.woke_at = t;
            }
            self.was_away = away;
            for f in &fx {
                if f.kind == "car" { world.car_at = t }
            }
            world.cam = self.cam_bytes.clone();
            world.tz = local_offset_secs();
            world.simple = simple;
            world.primary = self.primary;
            world.wall = own_wall && get(&st, &["ui", "wallpaper", "mode"]).as_str() != Some("panels");
            // the pointer the hub sees on the desktop, in this monitor's pixels (the wallpaper gets no motion under windows)
            if let Some(((cxp, cyp), at)) = cursor {
                if self.last_cursor != Some(at) {
                    self.last_cursor = Some(at);
                    let (lx, ly) = (cxp - self.origin.0, cyp - self.origin.1);
                    if lx >= 0.0 && ly >= 0.0 && lx <= w && ly <= h && desk_empty {
                        world.pointer = Some((lx, ly, t));
                    }
                }
            }
        }
        if self.primary && hub_now != self.sounded {
            self.sounded = hub_now;
            self.sound.update(&st, full || away);
        }
        // ---- INCOMING: a new alert, big in the middle for seven seconds (not the old ones at start)
        {
            let alerts = list(&st, &["alerts"]);
            let last_seq = alerts.last().and_then(|a| num(a, &["seq"])).unwrap_or(0.0) as i64;
            if self.seen_alert < 0 {
                if snap_at.is_some() { self.seen_alert = last_seq }
            } else if last_seq > self.seen_alert {
                self.seen_alert = last_seq;
                { let mut wl = self.world.borrow_mut(); wl.alert_at = t; wl.alert_env = 1.0; }
                if self.primary { self.incoming = alerts.last().map(|a| (t, a.clone())) }
            }
            // while you were away: what happened, for forty seconds or until you click it
            let dg = get(&st, &["digest"]);
            let seq = num(dg, &["seq"]).unwrap_or(0.0) as i64;
            if dg.is_object() && seq > self.seen_digest {
                self.seen_digest = seq;
                if self.primary && hub_now - num(dg, &["until"]).unwrap_or(0.0) <= 120.0 { self.digest = Some((t + 2.2, dg.clone())) } // first the face wakes up
            }
        }
        let hub_scene = text(&st, &["ui", "scene"]);
        let scene_name = if self.forced { self.scene.clone() } else if hub_scene.is_empty() { "sphere".to_string() } else { hub_scene };
        // a scene made with the editor (or a community .pulse): its design, drawn over Pulse's real sky
        let design = snap_design.clone().filter(|d| d.0 == scene_name);
        let scene_st = if get(&st, &["scene"]).is_object() { get(&st, &["scene"]).clone() } else { (*st).clone() };
        if let Some((name, d)) = &design {
            if &self.assets.0 != name {
                self.assets = (name.clone(), load_assets(d));
                self.raw_assets = load_asset_bytes(d);
                self.tinted.clear();
                *self.mem.borrow_mut() = design::Mem::default();
            }
            let hub_t = num(&st, &["t"]).unwrap_or(0.0);
            if hub_t != self.reacted {
                // a new state: the design's actions ("when this happens, do that")
                self.reacted = hub_t;
                for ask in design::react(d, &scene_st, &mut self.mem.borrow_mut(), t) {
                    let kind = if ask.kind == "text" { "say" } else { ask.kind.as_str() };
                    self.world.borrow_mut().fx(kind, &ask.text, false, t);
                }
            }
        }
        let bare = design.as_ref().map(|d| !d.1.get("city").and_then(Value::as_bool).unwrap_or(false)).unwrap_or(false);
        let private = get(&st, &["privacy"]).as_bool().unwrap_or(false) || get(&st, &["screencast"]).as_bool().unwrap_or(false);

        // ---- the world, on a canvas under everything
        let (world, old, is_city, marks_out, st2, pal2) = (self.world.clone(), self.city.clone(), scene_name == "city", self.marks.clone(), st.clone(), pal);
        let kind2 = scene_name.clone();
        let still_picture = own_wall && get(&st, &["ui", "wallpaper", "mode"]).as_str() == Some("panels");
        let scene_pal = if scene_name == "hacker" { modes::green(&pal) } else { pal };
        let (design2, scene_st2, mem2, drawn_out) = (design.clone(), scene_st.clone(), self.mem.clone(), self.drawn.clone());
        let scene_font: SharedString = { let f = text(&st, &["ui", "font"]); if f.is_empty() { self.font.clone() } else { f.into() } };
        let boxes_in = self.ui_boxes.clone();
        let cost_out = self.cost.clone();
        let sky = canvas(
            |_, _, _| {},
            move |bounds, _, window, cx| {
                if still_picture {
                    marks_out.borrow_mut().clear();
                    return;
                }
                let began = Instant::now();
                let marks = if is_city { city::paint(bounds, &mut old.borrow_mut(), &st2, &pal2, t, window) } else { crate::perf::time("scene", || scene::paint(bounds, &mut world.borrow_mut(), &st2, &scene_pal, t, &kind2, bare, window, cx, &scene_font, &boxes_in.borrow())) };
                if let Some((_, d)) = &design2 {
                    let (ox, oy, w2, h2) = (f32::from(bounds.origin.x), f32::from(bounds.origin.y), f32::from(bounds.size.width), f32::from(bounds.size.height));
                    *drawn_out.borrow_mut() = design::paint(window, d, &scene_st2, &pal2, &mut mem2.borrow_mut(), t, ox, oy, w2, h2);
                } else {
                    *drawn_out.borrow_mut() = (Vec::new(), Vec::new());
                }
                *marks_out.borrow_mut() = marks;
                let c = cost_out.get();
                cost_out.set((c.0, began.elapsed().as_secs_f64() * 1000.0));
            },
        )
        .absolute()
        .size_full();

        // ---- the panels, the clock and the ticker: pulse.css in rem, so the same layout fits every screen
        let rr = ui::rem(w, h);
        let uix = ui::Ui { r: rr, pal: &pal, private, now: hub_now };
        let sec_n: f32 = sec.parse().unwrap_or(0.0);
        let clock_el = div().absolute().top(px((3.6 + 1.2 - 0.75) * rr)).left(px(29.6 * rr)).w(px((w - 59.2 * rr).max(0.0))).flex().justify_center().child(uix.clock(&st, &hm, &sec, sec_n / 59.0, &date));
        let mut mine: Vec<&Value> = list(&st, &["widgets"]).iter().collect();
        mine.sort_by(|a, b| a.get("order").and_then(Value::as_f64).unwrap_or(50.0).total_cmp(&b.get("order").and_then(Value::as_f64).unwrap_or(50.0)));
        // pulse.js render(): system and network on the left, the weather on the right, yours where they say (or,
        // with slot "auto", images and status boards right, text in the middle, the rest where there is less)
        let mut items: Vec<(String, String, f64, ui::Panel)> = Vec::new();
        let ui_cfg = get(&st, &["ui"]);
        let has_cpu = get(&st, &["sys", "cpu"]).is_number();
        let mut order_of = std::collections::HashMap::<&str, f64>::new();
        let mut push = |items: &mut Vec<(String, String, f64, ui::Panel)>, slot: &str, p: ui::Panel| {
            let o = order_of.entry(if slot == "left" { "left" } else if slot == "right" { "right" } else { "center" }).or_insert(0.0);
            items.push((p.id.clone(), slot.to_string(), *o, p));
            *o += 10.0;
        };
        if has_cpu && get(ui_cfg, &["system"]).as_bool() != Some(false) {
            push(&mut items, "left", uix.sys(&st));
        }
        if has_cpu && get(ui_cfg, &["network"]).as_bool() != Some(false) {
            push(&mut items, "left", uix.net(&st));
        }
        if get(&st, &["weather"]).is_object() {
            push(&mut items, "right", uix.weather(&st));
        }
        let (mut nl, mut nr) = (2, 1);
        let mut ticker_ws: Vec<&Value> = Vec::new();
        for wd in mine.iter() {
            let mut sl = text(wd, &["slot"]);
            if sl == "ticker" {
                ticker_ws.push(wd);
                continue;
            }
            let ty = text(wd, &["type"]);
            if sl.is_empty() || sl == "auto" {
                sl = if ty == "image" || ty == "status" { "right".into() } else if ty == "text" { "center".into() } else if nl <= nr { "left".into() } else { "right".into() };
            }
            let id = text(wd, &["id"]);
            // a text panel types itself out again when its text changes
            let typed = if ty == "text" || ty.is_empty() {
                let full = text(wd, &["data", "text"]);
                let e = self.typed.entry(id.clone()).or_insert((full.clone(), t));
                if e.0 != full {
                    *e = (full.clone(), t);
                }
                Some((((t - e.1) / 0.04) as usize).saturating_mul(3))
            } else {
                None
            };
            let p = uix.generic(wd, &sl, typed, self.cams.get(&id).map(|c| c.1.clone()));
            match sl.as_str() {
                "left" => nl += 1,
                "center" => {}
                _ => { nr += 1; sl = "right".into() }
            }
            push(&mut items, &sl, p);
        }
        let col_h = h / rr - 3.6 - 3.0 - 0.8;
        let mut left = div().absolute().left(px(2.2 * rr)).top(px((3.6 + 0.8) * rr)).w(px(25.0 * rr)).h(px(col_h * rr)).flex().flex_col().gap(px(1.5 * rr));
        let mut right = div().absolute().right(px(2.2 * rr)).top(px((3.6 + 0.8) * rr)).w(px(25.0 * rr)).h(px(col_h * rr)).flex().flex_col().gap(px(1.5 * rr));
        let centre_w = (w / rr - 59.2).min(46.0).max(0.0);
        let centre_top = 3.6 + 1.2 + 8.64 + 1.9 + 1.5 + 1.8 + if text(&st, &["ui", "title"]).is_empty() { 0.0 } else { 1.6 } + 0.8 + 0.8;
        let mut centre = div().absolute().top(px(centre_top * rr)).left(px((w / rr - centre_w) / 2.0 * rr)).w(px(centre_w * rr)).max_h(px(17.7 * rr)).overflow_hidden().flex().flex_col().gap(px(1.5 * rr));
        {
            // the saved layout (~/.config/omarchy-pulse/layout.json, through the hub): a column, a place, or hidden;
            // in NOC mode the personal panels go away
            let draft_v = self.draft.clone().map(Value::Object);
            let layout = draft_v.as_ref().unwrap_or_else(|| get(&st, &["layout"]));
            self.boxes.borrow_mut().clear();
            self.ui_boxes.borrow_mut().clear();
            let noc_on = get(&st, &["noc", "on"]).as_bool().unwrap_or(false);
            let noc_hide: Vec<String> = get(&st, &["ui", "nocHide"]).as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| vec!["media".into(), "notifications".into()]);
            let mut placed: Vec<(String, f64, ui::Panel)> = Vec::new();
            let dragging = self.drag.clone();
            for (id, mut sl, mut order, p) in items {
                let o = get(layout, &[id.as_str()]);
                let hidden = get(o, &["hidden"]).as_bool().unwrap_or(false);
                if (hidden && !editing) || (noc_on && noc_hide.contains(&id)) { continue }
                if let Some(x) = get(o, &["slot"]).as_str().filter(|x| matches!(*x, "left" | "right" | "center")) { sl = x.to_string() }
                if let Some(x) = num(o, &["order"]) { order = x }
                let p = if editing {
                    // layout edit: every panel can be picked up (left button) or hidden and shown again (right button);
                    // a hidden recorder says where it is on the screen, so a drop knows which panel it lands before
                    let (boxes, id_box, slot_box) = (self.boxes.clone(), id.clone(), sl.clone());
                    let (id_down, id_hide) = (id.clone(), id.clone());
                    let el = div().relative().border_1().border_dashed().border_color(col(if dragging.as_deref() == Some(id.as_str()) { pal.warn } else { pal.accent }, 0.7)).cursor_pointer().opacity(if hidden { 0.3 } else { 1.0 }).child(p.el)
                        .child(canvas(move |b, _, _| { boxes.borrow_mut().insert(id_box, (slot_box, f32::from(b.origin.y), f32::from(b.size.height))); }, |_, _, _, _| ()).absolute().top_0().left_0().size_full())
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _: &MouseDownEvent, _, cx| { this.drag = Some(id_down.clone()); cx.stop_propagation(); }))
                        .on_mouse_down(MouseButton::Right, cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                            if let Some(d) = this.draft.as_mut() {
                                let e = d.entry(id_hide.clone()).or_insert_with(|| serde_json::json!({}));
                                let was = e.get("hidden").and_then(Value::as_bool).unwrap_or(false);
                                e["hidden"] = serde_json::json!(!was);
                            }
                            cx.stop_propagation();
                        }));
                    ui::Panel { id: p.id, el, h: p.h }
                } else {
                    // where it is on the screen, for the planes (pulse.js uiBoxes): a plane never flies hidden under a panel
                    let rec = self.ui_boxes.clone();
                    ui::Panel { id: p.id, el: p.el.child(canvas(move |b, _, _| rec.borrow_mut().push([f32::from(b.origin.x), f32::from(b.origin.y), f32::from(b.size.width), f32::from(b.size.height)]), |_, _, _, _| ()).absolute().top_0().left_0().size_full()), h: p.h }
                };
                placed.push((sl, order, p));
            }
            placed.sort_by(|a, b| a.1.total_cmp(&b.1));
            let take = |sl: &str, placed: &mut Vec<(String, f64, ui::Panel)>| -> Vec<ui::Panel> {
                let mut out = Vec::new();
                let mut rest = Vec::new();
                for x in placed.drain(..) {
                    if x.0 == sl { out.push(x.2) } else { rest.push(x) }
                }
                *placed = rest;
                out
            };
            // pulse.js fit(): a panel that does not fit in its column is hidden, never cut in half
            for p in ui::fit(take("left", &mut placed), col_h, 1.5) { left = left.child(p.el) }
            for p in ui::fit(take("right", &mut placed), col_h, 1.5) { right = right.child(p.el) }
            if !simple {
                for p in ui::fit(take("center", &mut placed), 17.7, 1.5) { centre = centre.child(p.el) }
            }
            if get(&st, &["focus"]).is_object() && !editing {
                (left, right, centre) = (left.opacity(0.16), right.opacity(0.16), centre.opacity(0.16)); // a focus: the panels step back
            } else if busy {
                (left, right, centre) = (left.opacity(0.9), right.opacity(0.9), centre.opacity(0.9));
            }
        }
        // ---- pulse.js tickerUpdate(): your ticker panels, the alerts of the last hour, the weather, the sources
        let mut lines: Vec<(String, String)> = Vec::new();
        for wd in ticker_ws.iter() {
            let d = get(wd, &["data"]);
            let (title, id) = (text(wd, &["title"]), text(wd, &["id"]));
            if let Some(items) = d.get("items").and_then(Value::as_array) {
                for it in items {
                    let pre = if !title.is_empty() && title != id && id != "ticker" { format!("{title} · ") } else { String::new() };
                    let sub = text(it, &["text"]);
                    lines.push((format!("{pre}{}{}", text(it, &["title"]), if sub.is_empty() { String::new() } else { format!(" · {sub}") }), text(it, &["level"])));
                }
            } else if !text(d, &["text"]).is_empty() {
                lines.push((format!("{} · {}", if title.is_empty() { id } else { title }, text(d, &["text"]).lines().next().unwrap_or("")), text(wd, &["level"])));
            } else if let Some(v) = d.get("value").filter(|v| !v.is_null()) {
                lines.push((format!("{} {} {}", if title.is_empty() { id } else { title }, ui::num_s(v), text(d, &["unit"])), text(wd, &["level"])));
            }
        }
        let alerts = list(&st, &["alerts"]);
        for a in alerts.iter().skip(alerts.len().saturating_sub(3)) {
            let at = num(a, &["t"]).unwrap_or(0.0);
            if hub_now - at < 3600.0 {
                lines.push((format!("{} · {} · {}", text(a, &["title"]), text(a, &["text"]), ui::ago(at, hub_now)), text(a, &["level"])));
            }
        }
        let wxs = get(&st, &["weather"]);
        if wxs.is_object() && text(wxs, &["error"]).is_empty() {
            lines.push((format!("{} {:.0}{} {} · sunset {}", text(wxs, &["city"]), num(wxs, &["temp"]).unwrap_or(0.0), text(wxs, &["unit"]), weather_word(num(wxs, &["code"]).unwrap_or(0.0) as i32), text(wxs, &["sunset"])), String::new()));
        }
        if !text(&st, &["sys", "host"]).is_empty() {
            lines.push((format!("omarchy pulse · {} live sources", list(&st, &["widgets"]).len()), String::new()));
        }
        let tick_i = ((t / 6.0) as usize) % lines.len().max(1);
        let ticker_el = if simple {
            div()
        } else {
            uix.ticker(w, lines.get(tick_i).cloned(), &if lines.is_empty() { String::new() } else { format!("{}/{}", tick_i + 1, lines.len()) })
        };
        // pulse.js fxText(): a line of light low over the plaza (#fxtext: 3.4rem, thin, spaced), 3.8 s, fading in .6 s
        let say = self.world.borrow().say.clone().filter(|(at, _)| t - at < 4.4);
        let say_el = match say {
            Some((at, words)) => {
                let age = (t - at) as f32;
                let a = (age / 0.6).min(1.0).min(((4.4 - age) / 0.6).clamp(0.0, 1.0));
                let fs = 3.4 * rr;
                div().absolute().top(px(h * 0.82)).left(px(0.0)).w(px(w)).flex().justify_center().opacity(a)
                    .text_size(px(fs)).font_weight(FontWeight::EXTRA_LIGHT).text_color(col(pal.fg, 1.0))
                    .child(ui::spaced(&words, 0.18, fs))
            }
            None => div(),
        };

        // ---- the planes' tags (from the last frame's positions), and one line that says what this is
        let mut tags = div().absolute().size_full();
        for m in self.marks.borrow().iter() {
            tags = tags.child(
                div()
                    .absolute()
                    .left(px(m.x.clamp(8.0, w - 260.0 * u)))
                    .top(px(m.y.clamp(8.0, h - 60.0 * u)))
                    .px(px(8.0 * u))
                    .py(px(4.0 * u))
                    .bg(col(pal.bg, 0.72))
                    .border_l_2()
                    .border_color(col(m.c, 0.9))
                    .flex()
                    .flex_col()
                    .child(if m.line.is_empty() { div().text_size(px(12.5 * u)).text_color(col(m.c, 1.0)).child(m.title.clone()) } else { div().font_weight(FontWeight::BOLD).text_color(col(pal.fg, 1.0)).child(m.title.clone()) })
                    .child(if m.line.is_empty() { div() } else { div().text_size(px(12.0 * u)).text_color(col(pal.dim, 1.0)).child(m.line.clone()) }),
            );
        }
        let mut scene_layer = div().absolute().size_full();
        {
            let drawn = self.drawn.borrow();
            for pc in &drawn.1 {
                let tinted = pc.tint.and_then(|c| {
                    let key = format!("{}#{:.3}-{:.3}-{:.3}", pc.asset, c[0], c[1], c[2]);
                    if !self.tinted.contains_key(&key) {
                        let made = self.raw_assets.get(&pc.asset).and_then(|b| picture::tint(b, c));
                        self.tinted.insert(key.clone(), made);
                    }
                    self.tinted.get(&key).cloned().flatten()
                });
                if let Some(im) = tinted.as_ref().or(self.assets.1.get(&pc.asset)) {
                    scene_layer = scene_layer.child(gpui::img(im.clone()).absolute().left(px(pc.x - pc.w / 2.0)).top(px(pc.y - pc.h / 2.0)).w(px(pc.w)).h(px(pc.h)).opacity(pc.a));
                }
            }
            for wd in &drawn.0 {
                scene_layer = scene_layer.child(
                    div().absolute().left(px(wd.x - w * 0.3)).top(px(wd.y - wd.size * 0.7)).w(px(w * 0.6)).flex().justify_center().text_size(px(wd.size)).line_height(px(wd.size * 1.4)).font_weight(if wd.bold { FontWeight::EXTRA_BOLD } else { FontWeight::MEDIUM }).text_color(col(wd.c, wd.a)).child(wd.text.clone()),
                );
            }
        }
        let status = match (&snap_at, &snap_error) {
            (None, Some(e)) => format!("waiting for the hub (omarchy-pulse): {e}"),
            (None, None) => "waiting for the hub (omarchy-pulse)…".to_string(),
            (Some(at), _) if at.elapsed() > Duration::from_secs(8) => "the hub stopped answering: showing the last it said".to_string(),
            _ if snap_handwritten.as_deref() == Some(scene_name.as_str()) => format!("the scene \"{scene_name}\" is written in code: the web page runs it; here, the skyline"),
            _ => String::new(),
        };
        let foot = div().absolute().bottom(px(34.0 * u)).left(px(0.0)).w(px(w)).flex().justify_center().text_size(px(13.0 * u)).text_color(col(pal.warn, 0.9)).child(status);

        // ---- the scene's own words (a scene in code, the garden, the tower...), from the last frame
        let mut words = div().absolute().size_full();
        let labels = self.world.borrow().labels.clone();
        for l in labels.iter().take(160) {
            let size = l.size.max(8.0);
            words = words.child(div().absolute().left(px(l.x - w * 0.2)).top(px(l.y)).w(px(w * 0.4)).flex().justify_center().whitespace_nowrap().text_size(px(size)).line_height(px(size * 1.25)).font_weight(if l.bold { FontWeight::BOLD } else { FontWeight::NORMAL }).text_color(col(l.c, l.a)).child(l.text.clone()));
        }
        // ---- your own wallpaper behind everything, dimmed as you asked
        let wall_el = match (&self.wallpaper.1, own_wall) {
            (Some(im), true) => div().absolute().size_full().child(gpui::img(im.clone()).absolute().size_full().object_fit(ObjectFit::Cover)).child(div().absolute().size_full().bg(col([0.0, 0.0, 0.0], num(&st, &["ui", "wallpaper", "dim"]).unwrap_or(0.25) as f32))),
            _ => div(),
        };
        // ---- INCOMING, the digest, the incident of the NOC mode, what the sphere heard
        self.incoming = self.incoming.take().filter(|a| t - a.0 < 7.0);
        let incoming_el = match &self.incoming {
            Some((at, a)) => {
                let c = if text(a, &["level"]) == "warn" { pal.warn } else { pal.crit };
                // the title arrives as noise that settles, as on the web page
                let full_title = text(a, &["title"]);
                let shown = (((t - at) * 1000.0 / 28.0) / 1.6) as usize;
                let glyphs: Vec<char> = "█▓▒░<>/\\|#$%&*+=?01".chars().collect();
                let title: String = full_title.chars().enumerate().map(|(i, ch)| if i < shown || ch == ' ' { ch } else { glyphs[((t * 37.0) as usize + i * 7) % glyphs.len()] }).collect();
                // pulse.css #incoming: in the middle at 41% of the height, nearly opaque, a glow in the alert's colour
                let rr = ui::rem(w, h);
                div().absolute().top(px(0.0)).left(px(0.0)).w(px(w)).h(px(h * 0.82)).flex().items_center().justify_center().child(
                    div().min_w(px(44.0 * rr)).max_w(px(70.0 * rr)).pt(px(1.6 * rr)).pb(px(1.5 * rr)).px(px(2.6 * rr)).flex().flex_col().items_center().bg(col(pal.bg, 0.97)).border_2().border_color(col(c, 1.0))
                        .shadow(vec![gpui::BoxShadow { color: col(c, 0.45).into(), offset: point(px(0.0), px(0.0)), blur_radius: px(4.0 * rr), spread_radius: px(0.0), inset: false }])
                        .child(div().text_size(px(1.1 * rr)).text_color(col(c, if (t * 2.0) as i64 % 2 == 0 { 1.0 } else { 0.3 })).child(ui::spaced("INCOMING", 0.7, 1.1 * rr)))
                        .child(div().mt(px(0.6 * rr)).text_size(px(2.6 * rr)).text_color(col(pal.fg, 1.0)).child(title))
                        .child(div().mt(px(0.5 * rr)).text_size(px(1.3 * rr)).text_color(col(pal.fg, 0.75)).child(if private { String::new() } else { text(a, &["text"]) }))
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _: &MouseDownEvent, _, cx| {
                            this.incoming = None;
                            cx.stop_propagation();
                            cx.notify();
                        })),
                )
            }
            None => div(),
        };
        self.digest = self.digest.take().filter(|d| t - d.0 < 40.0);
        let digest_el = match &self.digest {
            Some((at, d)) if t >= *at => {
                let mins = num(d, &["mins"]).unwrap_or(0.0) as i64;
                let mut frame = div().min_w(px(500.0 * u)).max_w(px(840.0 * u)).px(px(30.0 * u)).py(px(20.0 * u)).flex().flex_col().gap(px(6.0 * u)).bg(col(pal.bg, 0.88)).border_1().border_color(col(pal.accent, 0.7)).cursor_pointer()
                    .child(div().mb(px(8.0 * u)).text_size(px(14.5 * u)).text_color(col(pal.accent, 1.0)).child(format!("◂ WHILE YOU WERE AWAY · {}{} min", if mins >= 60 { format!("{}h ", mins / 60) } else { String::new() }, mins % 60)));
                let lines = list(d, &["lines"]);
                for l in lines.iter().take(8) {
                    let icon = match text(l, &["kind"]).as_str() { "alert" => "▲", "agent" => "✓", "news" => "✦", "notifications" => "●", _ => "·" };
                    frame = frame.child(div().flex().gap(px(10.0 * u)).text_size(px(19.0 * u)).child(div().w(px(22.0 * u)).text_color(col(level_color(&text(l, &["level"]), &pal), 1.0)).child(icon)).child(div().overflow_hidden().whitespace_nowrap().text_color(col(pal.fg, 0.9)).child(text(l, &["text"]).chars().take(70).collect::<String>())));
                }
                if lines.is_empty() {
                    frame = frame.child(div().text_size(px(19.0 * u)).text_color(col(pal.ok, 1.0)).child("✓  all quiet"));
                }
                if let Some(more) = num(d, &["more"]).filter(|m| *m > 0.0) {
                    frame = frame.child(div().text_size(px(19.0 * u)).text_color(col(pal.fg, 0.5)).child(format!("+  {more:.0} more")));
                }
                div().absolute().top(px(h * 0.28)).left(px(0.0)).w(px(w)).flex().justify_center().child(frame.on_mouse_down(MouseButton::Left, cx.listener(|this, _: &MouseDownEvent, _, cx| {
                    this.digest = None;
                    cx.stop_propagation();
                    cx.notify();
                })))
            }
            _ => div(),
        };
        let incident = get(&st, &["noc", "incident"]);
        let incident_el = if get(&st, &["noc", "on"]).as_bool().unwrap_or(false) && incident.is_object() {
            let mins = ((hub_now - num(incident, &["start"]).unwrap_or(hub_now)) / 60.0).max(0.0) as i64;
            let mut frame = div().min_w(px(520.0 * u)).max_w(px(900.0 * u)).px(px(28.0 * u)).py(px(18.0 * u)).flex().flex_col().gap(px(6.0 * u)).bg(col(pal.bg, 0.9)).border_2().border_color(col(pal.crit, 1.0)).cursor_pointer()
                .child(div().mb(px(6.0 * u)).text_size(px(15.0 * u)).text_color(col(pal.crit, 1.0)).child(format!("▲ INCIDENT · {mins} min · click to acknowledge")));
            let evs = list(incident, &["events"]);
            for e in evs.iter().skip(evs.len().saturating_sub(6)) {
                let sod = (num(e, &["t"]).unwrap_or(0.0) as i64 + local_offset_secs()).rem_euclid(86400);
                frame = frame.child(div().flex().gap(px(10.0 * u)).text_size(px(17.0 * u)).child(div().font_weight(FontWeight::BOLD).text_color(col(level_color(&text(e, &["level"]), &pal), 1.0)).child(format!("{:02}:{:02}", sod / 3600, (sod / 60) % 60))).child(div().text_color(col(pal.fg, 0.9)).child({ let x = text(e, &["text"]); if x.is_empty() { "an event".to_string() } else { x.chars().take(80).collect() } })));
            }
            let base = self.base.clone();
            div().absolute().top(px(h * 0.60)).left(px(0.0)).w(px(w)).flex().justify_center().child(frame.on_mouse_down(MouseButton::Left, move |_, _, cx| {
                hub::post(&base, "/api/noc?do=ack");
                cx.stop_propagation();
            }))
        } else {
            div()
        };
        let listen = get(&st, &["listen"]);
        let heard_el = if listen.is_object() {
            let inner = match text(listen, &["state"]).as_str() {
                "listening" => div().text_size(px(20.0 * u)).text_color(col(pal.accent, 1.0)).child("● listening…"),
                "thinking" => div().text_size(px(20.0 * u)).text_color(col(pal.fg, 0.8)).child("…"),
                _ => div().flex().flex_col().items_center().gap(px(6.0 * u))
                    .child({ let q = text(listen, &["heard"]); if q.is_empty() { div() } else { div().text_size(px(18.0 * u)).text_color(col(pal.fg, 0.7)).child(format!("“{q}”")) } })
                    .child(div().text_size(px(22.0 * u)).text_color(col(pal.fg, 1.0)).child(text(listen, &["answer"]))),
            };
            div().absolute().top(px(h * 0.85)).left(px(w * 0.2)).w(px(w * 0.6)).flex().justify_center().child(inner)
        } else {
            div()
        };

        let layout_el = if editing {
            let button = |label: &'static str, what: &'static str, this_base: String, c: palette::C| {
                div().px(px(14.0 * u)).py(px(5.0 * u)).border_1().border_color(col(c, 0.9)).text_color(col(c, 1.0)).cursor_pointer().child(label).on_mouse_down(MouseButton::Left, cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                    let body = if what == "save" { serde_json::to_string(&this.draft.clone().unwrap_or_default()).unwrap_or_default() } else { String::new() };
                    hub::post_body(&this_base, &format!("/api/layout?do={what}"), body);
                    (this.draft, this.drag) = (None, None);
                    cx.stop_propagation();
                }))
            };
            let mut bar = div().px(px(22.0 * u)).py(px(10.0 * u)).flex().items_center().gap(px(16.0 * u)).bg(col(pal.bg, 0.92)).border_1().border_color(col(pal.accent2, 0.9)).text_size(px(15.0 * u))
                .child(div().text_color(col(pal.fg, 0.9)).child("Drag the panels where you want them · right click hides one"))
                .child(button("Save", "save", self.base.clone(), pal.ok))
                .child(button("Cancel", "cancel", self.base.clone(), pal.fg))
                .child(button("Reset", "reset", self.base.clone(), pal.warn));
            if let Some(id) = &self.drag {
                bar = bar.child(div().text_color(col(pal.warn, 1.0)).child(format!("moving {id}: drop it on a column")));
            }
            div().absolute().top(px(12.0 * u)).left(px(0.0)).w(px(w)).flex().justify_center().child(bar)
        } else {
            div()
        };
        // the font: yours ([pulse] font), else the one Omarchy ships, as pulse.css says
        let font: SharedString = { let f = text(&st, &["ui", "font"]); if f.is_empty() { self.font.clone() } else { f.into() } };
        let root = div().relative().size_full().bg(col(pal.bg, 1.0)).font_family(font).text_size(px(15.0 * u)).text_color(col(pal.fg, 1.0));
        let root = root
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _, _| {
                let t = this.started.elapsed().as_secs_f64();
                this.world.borrow_mut().pointer = Some((f32::from(ev.position.x), f32::from(ev.position.y), t));
            }))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, ev: &MouseDownEvent, _, _| this.click(f32::from(ev.position.x), f32::from(ev.position.y))))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, ev: &gpui::MouseUpEvent, window, _| {
                let vw = f32::from(window.viewport_size().width);
                this.drop_panel(f32::from(ev.position.x), f32::from(ev.position.y), vw);
            }));
        if full {
            return root.child(wall_el); // a fullscreen window is over this monitor: nothing to show under it
        }
        if simple {
            // [pulse] secondary = "simple": the scene and the clock, no panels
            return root.child(wall_el).child(sky).child(scene_layer).child(words).child(clock_el).child(ticker_el);
        }
        root.child(wall_el).child(sky).child(scene_layer).child(words).child(clock_el).child(tags).child(say_el).child(centre).child(left).child(right).child(ticker_el).child(foot).child(heard_el).child(incident_el).child(digest_el).child(incoming_el).child(layout_el)
    }
}


/// The pictures of a design: its assets are data: URLs (PNG, JPEG, WebP, GIF or SVG), so a scene stays one file.
/// The same pictures as they came (not SVG): a tint is made from these.
fn load_asset_bytes(design: &Value) -> std::collections::HashMap<String, Vec<u8>> {
    use base64::Engine;
    let mut out = std::collections::HashMap::new();
    if let Some(a) = design.get("assets").and_then(Value::as_object) {
        for (id, v) in a.iter().take(40) {
            let Some((head, body)) = v.as_str().and_then(|u| u.split_once(',')) else { continue };
            if head.contains("base64") && !head.contains("svg") {
                if let Ok(b) = base64::engine::general_purpose::STANDARD.decode(body.trim()) {
                    out.insert(id.clone(), b);
                }
            }
        }
    }
    out
}

fn load_assets(design: &Value) -> std::collections::HashMap<String, std::sync::Arc<gpui::Image>> {
    use base64::Engine;
    let mut out = std::collections::HashMap::new();
    if let Some(a) = design.get("assets").and_then(Value::as_object) {
        for (id, v) in a.iter().take(40) {
            let Some(url) = v.as_str() else { continue };
            let Some((head, body)) = url.split_once(',') else { continue };
            let format = if head.contains("svg") { gpui::ImageFormat::Svg } else if head.contains("jpeg") || head.contains("jpg") { gpui::ImageFormat::Jpeg } else if head.contains("webp") { gpui::ImageFormat::Webp } else if head.contains("gif") { gpui::ImageFormat::Gif } else { gpui::ImageFormat::Png };
            let bytes = if head.contains("base64") { base64::engine::general_purpose::STANDARD.decode(body.trim()).ok() } else { Some(body.as_bytes().to_vec()) };
            if let Some(b) = bytes {
                out.insert(id.clone(), std::sync::Arc::new(gpui::Image::from_bytes(format, b)));
            }
        }
    }
    out
}

/// The weather code (WMO, as Open-Meteo gives it) in a word, as the web page says it.
/// pulse.js WX: the words for the weather codes of Open-Meteo.
pub fn weather_word(code: i32) -> &'static str {
    match code {
        0 => "clear",
        1 => "mostly clear",
        2 => "partly cloudy",
        3 => "overcast",
        45 | 48 => "fog",
        51 | 53 | 55 => "drizzle",
        56 | 57 => "freezing drizzle",
        61 | 63 => "rain",
        65 => "heavy rain",
        66 | 67 => "freezing rain",
        71 | 73 => "snow",
        75 => "heavy snow",
        77 => "snow grains",
        80 | 81 => "showers",
        82 => "violent showers",
        85 | 86 => "snow showers",
        95 | 96 | 99 => "thunderstorm",
        _ => "",
    }
}

struct Args {
    window: bool,
    url: String,
    fps: u64,
    font: String,
    scene: String,
}

fn args() -> Args {
    let mut a = Args { window: false, url: format!("http://127.0.0.1:{}", hub::port()), fps: 0, font: FONT.to_string(), scene: String::new() };
    let mut it = std::env::args().skip(1);
    while let Some(x) = it.next() {
        match x.as_str() {
            "--window" => a.window = true,
            "--url" => a.url = it.next().unwrap_or(a.url).trim_end_matches('/').to_string(),
            "--fps" => a.fps = it.next().and_then(|v| v.parse().ok()).unwrap_or(a.fps),
            "--font" => a.font = it.next().unwrap_or(a.font),
            "--scene" => a.scene = it.next().unwrap_or(a.scene),
            "--version" | "-V" => {
                println!("pulse-native {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => {
                println!("pulse-native {}: Omarchy Pulse, drawn natively (Rust + GPUI)\n\n  pulse-native            the wallpaper (a Wayland layer-shell surface behind your windows)\n  pulse-native --window   an ordinary window\n  --url URL   the hub (default: http://127.0.0.1:PORT from PULSE_PORT or pulse.toml)\n  --fps N     frames a second, 5 to 60 (default: the hub's [pulse] fps)\n  --font NAME the font (default: {FONT})\n  --scene S   the scene: by default the hub's own ([pulse] scene); skyline, sphere, or city (pulse-native's first world)", env!("CARGO_PKG_VERSION"));
                std::process::exit(if x == "--help" || x == "-h" { 0 } else { 2 });
            }
        }
    }
    a
}

fn main() {
    let a = args();
    let shared = hub::start(a.url.clone(), hub::token());
    let live = hub::start_live(a.url.clone(), hub::token());
    gpui_platform::application().with_quit_mode(QuitMode::LastWindowClosed).run(move |cx: &mut App| {
        // one wallpaper for each monitor, as the web one; [pulse] secondary = "off" keeps it on the first only
        let secondary_off = std::fs::read_to_string(hub::config_dir().join("pulse.toml")).ok().and_then(|x| x.parse::<toml::Table>().ok()).and_then(|c| c.get("pulse").and_then(|p| p.get("secondary")).and_then(|v| v.as_str().map(|v| v == "off"))).unwrap_or(false);
        let first = cx.primary_display().map(|d| d.id());
        // every monitor with its own size: a layer surface is as big as we ask, and a bigger one is centred and cut
        let mut screens: Vec<(Option<gpui::DisplayId>, Size<gpui::Pixels>)> = cx.displays().iter().map(|d| (Some(d.id()), d.bounds().size)).collect();
        screens.sort_by_key(|d| d.0 != first); // the first monitor first: alerts and sound live there
        if a.window || screens.is_empty() {
            screens = vec![(None, size(px(0.), px(0.)))];
        } else if secondary_off {
            screens.truncate(1);
        }
        let mut opened = 0;
        for (i, (screen, screen_size)) in screens.into_iter().enumerate() {
            let mut options = WindowOptions::new().titlebar(None).app_id(Some("omarchy-pulse-native".to_string()));
            options = if a.window {
                options.window_bounds(Some(WindowBounds::Windowed(Bounds { origin: point(px(0.), px(0.)), size: Size::new(px(1280.), px(720.)) })))
            } else {
                // the wallpaper: the background layer, stretched to every edge, under the bar too, and never taking the keyboard
                options
                    // the monitor's own size (0 × 0, stretched by the anchors, when it is not known)
                    .window_bounds(Some(WindowBounds::Windowed(Bounds { origin: point(px(0.), px(0.)), size: screen_size })))
                    .kind(WindowKind::LayerShell(LayerShellOptions {
                        namespace: "omarchy-pulse".to_string(),
                        layer: Layer::Background,
                        anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                        exclusive_zone: Some(px(-1.)),
                        keyboard_interactivity: KeyboardInteractivity::None,
                        ..Default::default()
                    }))
                    .linux_window_background(LinuxWindowBackground::Opaque)
            };
            if let Some(id) = screen {
                options = options.display_id(Some(id));
            }
            // each wallpaper listens to the hub's effects by itself; the state is read once for all of them
            let live = if i == 0 { live.clone() } else { hub::start_live(a.url.clone(), hub::token()) };
            let (fps, font, scene, hub, base) = (a.fps, a.font.clone(), a.scene.clone(), shared.clone(), a.url.clone());
            match cx.open_window(options, move |_, cx| cx.new(|cx| Pulse::new(hub, live, fps, font, scene, base, i == 0, cx))) {
                Ok(_) => opened += 1,
                Err(e) => eprintln!("pulse-native: could not open the wallpaper on monitor {}: {e}", i + 1),
            }
        }
        if opened == 0 {
            eprintln!("pulse-native: no wallpaper could be opened (it needs a Wayland compositor with wlr-layer-shell, such as Hyprland; try --window)");
            std::process::exit(1);
        }
    });
}
