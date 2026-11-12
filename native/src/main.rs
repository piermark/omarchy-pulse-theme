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

mod city;
mod design;
mod draw;
mod extra;
mod hub;
mod life;
mod look;
mod modes;
mod palette;
mod picture;
mod scene;
mod sound;
mod worlds;

use gpui::{
    canvas, div, layer_shell::*, point, prelude::*, px, size, App, Bounds, Context, FontWeight, LinuxWindowBackground, MouseButton, MouseDownEvent,
    MouseMoveEvent, ObjectFit, QuitMode, SharedString, Size, Window, WindowBounds, WindowKind, WindowOptions,
};
use std::sync::{Arc, Mutex};
use hub::{get, list, num, text, FxQueue, Shared};
use palette::{col, mix, Pal, C};
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
            if !covered && this.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
            if covered && this.update(cx, |_, _| ()).is_err() {
                break;
            }
            cx.background_executor().timer(Duration::from_millis(if covered { 1000 } else { 1000 / want.clamp(1, 60) })).await;
        })
        .detach();
        let world = scene::World::new();
        Pulse { hub, live, forced: !scene.is_empty(), scene, city: Rc::new(RefCell::new(city::World::new())), world: Rc::new(RefCell::new(world)), marks: Rc::new(RefCell::new(Vec::new())), mem: Rc::new(RefCell::new(design::Mem::default())), drawn: Rc::new(RefCell::new((Vec::new(), Vec::new()))), assets: (String::new(), Default::default()), reacted: 0.0, started: Instant::now(), font: font.into(), base, primary, monkey, origin: (0.0, 0.0), sound: sound::Sound::new(), wallpaper: (String::new(), None, None), cover: (String::new(), None), seen_alert: -1, incoming: None, seen_digest: 0, digest: None, ack: None, last_cursor: None, sounded: 0.0, cams: Default::default(), raw_assets: Default::default(), tinted: Default::default(), draft: None, drag: None, boxes: Default::default() }
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
                let scene_st = if get(&st, &["scene"]).is_object() { get(&st, &["scene"]).clone() } else { st };
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
fn local_offset_secs() -> i64 {
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

fn bytes(v: f64) -> String {
    if v >= 1e6 {
        format!("{:.1} MB/s", v / 1e6)
    } else {
        format!("{:.1} KB/s", v / 1e3)
    }
}

/// A Pulse panel: a thin frame with its title sitting on the top edge, as on the web page.
fn panel(title: &str, pal: &Pal, u: f32) -> gpui::Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .gap(px(7.0 * u))
        .px(px(18.0 * u))
        .pt(px(20.0 * u))
        .pb(px(14.0 * u))
        .bg(col(pal.bg, 0.50))
        .border_1()
        .border_color(col(pal.accent, 0.40))
        .child(
            div().absolute().top(px(-10.0 * u)).left(px(12.0 * u)).px(px(7.0 * u)).bg(col(pal.bg, 0.92)).text_size(px(14.0 * u)).text_color(col(pal.accent, 1.0)).child(format!("◆ {title}")),
        )
        .child(div().absolute().bottom(px(-1.0)).left(px(-1.0)).w(px(12.0 * u)).h(px(2.0 * u)).bg(col(pal.accent, 1.0)))
        .child(div().absolute().bottom(px(-1.0)).right(px(-1.0)).w(px(12.0 * u)).h(px(2.0 * u)).bg(col(pal.accent, 1.0)))
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

fn ago(t: f64, now: f64) -> String {
    let d = (now - t).max(0.0);
    if d < 3600.0 {
        format!("{:.0}m", d / 60.0)
    } else if d < 86400.0 {
        format!("{:.0}h", d / 3600.0)
    } else {
        format!("{:.0}d", d / 86400.0)
    }
}

/// A small chart of a value's history: a line, and the area under it.
fn spark(values: Vec<f32>, c: C, u: f32) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |b, _, window, _| {
            let (lo, hi) = values.iter().fold((f32::MAX, f32::MIN), |(l, h2), v| (l.min(*v), h2.max(*v)));
            let span = (hi - lo).max(1e-6);
            let (x0, y0, w, h) = (f32::from(b.origin.x), f32::from(b.origin.y), f32::from(b.size.width), f32::from(b.size.height));
            let at = |i: usize| point(px(x0 + w * i as f32 / (values.len() - 1) as f32), px(y0 + h - (values[i] - lo) / span * h * 0.9 - h * 0.05));
            let mut line = gpui::PathBuilder::stroke(px((1.6 * u).max(1.0)));
            let mut area = gpui::PathBuilder::fill();
            area.move_to(point(px(x0), px(y0 + h)));
            for i in 0..values.len() {
                if i == 0 { line.move_to(at(i)) } else { line.line_to(at(i)) }
                area.line_to(at(i));
            }
            area.line_to(point(px(x0 + w), px(y0 + h)));
            area.close();
            if let Ok(p) = area.build() {
                window.paint_path(p, col(c, 0.14));
            }
            if let Ok(p) = line.build() {
                window.paint_path(p, col(c, 0.95));
            }
        },
    )
    .w_full()
    .h(px(34.0 * u))
}

/// One of your own panels (a `[[source]]` of pulse.toml), by its type: a stat or gauge, a list, key and value rows,
/// a status board, or text. What this renderer can't draw yet shows its note, never an empty box.
fn widget(wd: &Value, pal: &Pal, u: f32, now: f64, rows_max: usize, private: bool, base: &str, cam: Option<Arc<gpui::Image>>) -> gpui::Div {
    let s = |k: &str| wd.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let data = wd.get("data").unwrap_or(&Value::Null);
    let title = if s("title").is_empty() { s("id") } else { s("title") };
    let mut p = panel(&title, pal, u);
    let dim = col(pal.dim, 1.0);
    let id = s("id");
    let truthy = |k: &str| matches!(wd.get(k), Some(v) if !v.is_null() && v.as_bool() != Some(false) && v.as_str() != Some(""));
    if truthy("click") && !base.is_empty() {
        // a panel with click = "..." in pulse.toml runs that command (the hub runs it, not the wallpaper)
        let (b2, path) = (base.to_string(), format!("/api/click/{}", urlenc(&id)));
        p = p.cursor_pointer().on_mouse_down(MouseButton::Left, move |_, _, cx| {
            hub::post(&b2, &path);
            cx.stop_propagation();
        });
    }
    let item_click = truthy("itemclick") && !base.is_empty();
    match s("level").as_str() {
        "warn" => p = p.border_color(col(pal.warn, 0.8)),
        "crit" => p = p.border_color(col(pal.crit, 0.9)),
        _ => {}
    }
    if private || wd.get("private").and_then(Value::as_bool).unwrap_or(false) && private {
        // someone else is looking at this screen: your panels keep their place and say nothing
        return p.child(div().text_color(dim).child("•••  hidden while the screen is shared"));
    }
    match s("type").as_str() {
        "stat" | "gauge" => {
            let v = data.get("value").and_then(Value::as_f64);
            let unit = data.get("unit").and_then(Value::as_str).unwrap_or("").to_string();
            let shown = match v {
                Some(x) if x.fract().abs() < 1e-9 || x.abs() >= 100.0 => format!("{x:.0}"),
                Some(x) => format!("{x:.2}"),
                None => data.get("value").and_then(Value::as_str).unwrap_or("-").to_string(),
            };
            p = p.child(
                div().flex().items_end().gap(px(8.0 * u)).child(div().text_size(px(44.0 * u)).line_height(px(46.0 * u)).text_color(col(pal.fg, 1.0)).child(shown)).child(div().pb(px(5.0 * u)).text_color(col(pal.accent, 1.0)).child(unit.clone())),
            );
            let hist: Vec<f32> = data.get("history").and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_f64().map(|v| v as f32)).collect()).unwrap_or_default();
            if hist.len() > 2 {
                p = p.child(spark(hist, pal.accent, u));
            }
            if s("type") == "gauge" || unit == "%" {
                let f = (v.unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
                p = p.child(div().h(px(8.0 * u)).bg(col(pal.dim, 0.35)).child(div().h_full().w(gpui::relative(f)).bg(col(pal.accent, 0.95))));
            }
        }
        "list" | "status" => {
            for (row_i, it) in data.get("items").and_then(Value::as_array).map(|a| a.as_slice()).unwrap_or(&[]).iter().take(rows_max).enumerate() {
                let g = |k: &str| it.get(k).and_then(Value::as_str).unwrap_or("").to_string();
                let right = if !g("right").is_empty() { g("right") } else if !g("note").is_empty() { g("note") } else { it.get("t").and_then(Value::as_f64).map(|t| ago(t, now)).unwrap_or_default() };
                let mut row = div().flex().flex_col().child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0 * u))
                        .child(div().size(px(7.0 * u)).rounded_full().bg(col(level_color(&g("level"), pal), 1.0)))
                        .child(div().flex_1().overflow_hidden().whitespace_nowrap().text_color(col(pal.fg, 1.0)).child(g("title").chars().take(34).collect::<String>()))
                        .child(div().text_size(px(12.0 * u)).text_color(dim).child(right)),
                );
                if !g("text").is_empty() {
                    row = row.child(div().pl(px(15.0 * u)).text_size(px(12.5 * u)).text_color(dim).child(g("text").chars().take(40).collect::<String>()));
                }
                if item_click {
                    // a panel whose items can be clicked: your lights (Home Assistant)
                    let (b2, path) = (base.to_string(), format!("/api/click/{}?item={row_i}", urlenc(&id)));
                    row = row.cursor_pointer().on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        hub::post(&b2, &path);
                        cx.stop_propagation();
                    });
                }
                p = p.child(row);
            }
        }
        "kv" | "table" => {
            for r in data.get("rows").and_then(Value::as_array).map(|a| a.as_slice()).unwrap_or(&[]).iter().take(rows_max + 2) {
                let cell = |i: usize| match r.get(i) {
                    Some(Value::String(x)) => x.clone(),
                    Some(Value::Null) | None => String::new(),
                    Some(x) => x.to_string(),
                };
                let cols = r.as_array().map(|a| a.len()).unwrap_or(0).min(5);
                if cols <= 2 {
                    p = p.child(div().flex().justify_between().child(div().text_color(dim).child(cell(0))).child(div().text_color(col(pal.fg, 1.0)).child(cell(1))));
                } else {
                    let mut row = div().flex().gap(px(8.0 * u));
                    for i in 0..cols {
                        row = row.child(div().flex_1().overflow_hidden().whitespace_nowrap().text_size(px(13.0 * u)).text_color(if i == 0 { dim } else { col(pal.fg, 1.0) }).child(cell(i).chars().take(18).collect::<String>()));
                    }
                    p = p.child(row);
                }
            }
        }
        "chart" => {
            // one line for each series, in the theme's colours
            for (i, sr) in data.get("series").and_then(Value::as_array).map(|a| a.as_slice()).unwrap_or(&[]).iter().take(4).enumerate() {
                let vals: Vec<f32> = sr.get("values").or_else(|| sr.get("v")).unwrap_or(sr).as_array().map(|a| a.iter().filter_map(|x| x.as_f64().map(|v| v as f32)).collect()).unwrap_or_default();
                if vals.len() > 2 {
                    p = p.child(spark(vals, [pal.accent, pal.accent2, pal.ok, pal.warn][i % 4], u));
                }
            }
        }
        "image" => {
            // a picture or a camera's frame the hub already holds (a data: URL): nothing is fetched from here
            let src = data.get("frame").or_else(|| data.get("src")).and_then(Value::as_str).unwrap_or("");
            if let Some(im) = cam.or_else(|| data_image(src)) {
                p = p.child(gpui::img(im).w_full().h(px(170.0 * u)));
            }
        }
        "text" => {
            for line in data.get("text").and_then(Value::as_str).unwrap_or("").lines().take(6) {
                p = p.child(div().text_size(px(16.0 * u)).text_color(col(pal.fg, 1.0)).child(line.chars().take(90).collect::<String>()));
            }
        }
        _ => {}
    }
    let note = data.get("note").and_then(Value::as_str).unwrap_or("");
    if !note.is_empty() {
        p = p.child(div().text_size(px(12.5 * u)).text_color(dim).child(note.chars().take(60).collect::<String>()));
    }
    p
}

/// A row with a name, a bar as full as the value, and the value.
fn meter(name: &str, frac: f32, value: String, c: C, pal: &Pal, u: f32) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.0 * u))
        .child(div().w(px(40.0 * u)).text_color(col(pal.dim, 1.0)).child(name.to_string()))
        .child(div().flex_1().h(px(6.0 * u)).bg(col(pal.dim, 0.35)).child(div().h_full().w(gpui::relative(frac.clamp(0.0, 1.0))).bg(col(c, 0.95))))
        .child(div().w(px(64.0 * u)).flex().justify_end().text_color(col(pal.fg, 1.0)).child(value))
}

impl Render for Pulse {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            world.tz = local_offset_secs();
            world.simple = simple;
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
                self.world.borrow_mut().alert_at = t;
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
        let scene_name = if self.forced { self.scene.clone() } else if hub_scene.is_empty() { "skyline".to_string() } else { hub_scene };
        // a scene made with the editor (or a community .pulse): its design, drawn over Pulse's real sky
        let design = snap_design.clone().filter(|d| d.0 == scene_name);
        let scene_st = if get(&st, &["scene"]).is_object() { get(&st, &["scene"]).clone() } else { st.clone() };
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
        let heat = |v: f32| if v > 0.9 { pal.crit } else if v > 0.75 { pal.warn } else { pal.accent };

        // ---- the world, on a canvas under everything
        let (world, old, is_city, marks_out, st2, pal2) = (self.world.clone(), self.city.clone(), scene_name == "city", self.marks.clone(), st.clone(), pal);
        let kind2 = scene_name.clone();
        let still_picture = own_wall && get(&st, &["ui", "wallpaper", "mode"]).as_str() == Some("panels");
        let scene_pal = if scene_name == "hacker" { modes::green(&pal) } else { pal };
        let (design2, scene_st2, mem2, drawn_out) = (design.clone(), scene_st.clone(), self.mem.clone(), self.drawn.clone());
        let sky = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                if still_picture {
                    marks_out.borrow_mut().clear();
                    return;
                }
                let marks = if is_city { city::paint(bounds, &mut old.borrow_mut(), &st2, &pal2, t, window) } else { scene::paint(bounds, &mut world.borrow_mut(), &st2, &scene_pal, t, &kind2, bare, window) };
                if let Some((_, d)) = &design2 {
                    let (ox, oy, w2, h2) = (f32::from(bounds.origin.x), f32::from(bounds.origin.y), f32::from(bounds.size.width), f32::from(bounds.size.height));
                    *drawn_out.borrow_mut() = design::paint(window, d, &scene_st2, &pal2, &mut mem2.borrow_mut(), t, ox, oy, w2, h2);
                } else {
                    *drawn_out.borrow_mut() = (Vec::new(), Vec::new());
                }
                *marks_out.borrow_mut() = marks;
            },
        )
        .absolute()
        .size_full();

        // ---- the clock
        let user = text(&st, &["sys", "user"]);
        let host = text(&st, &["sys", "host"]);
        let up = num(&st, &["sys", "uptime"]).unwrap_or(0.0) as i64;
        let upt = if up > 86400 { format!(" · up {}d {}h", up / 86400, (up % 86400) / 3600) } else if up > 0 { format!(" · up {}h {}m", up / 3600, (up % 3600) / 60) } else { String::new() };
        let theme = { let n = text(&st, &["theme", "name"]); if n.is_empty() { "omarchy".to_string() } else { n } };
        let heart = num(&st, &["heart"]).filter(|v| *v > 0.0).map(|v| format!(" · ♥ {v:.0}")).unwrap_or_default();
        let title = { let x = text(&st, &["ui", "title"]); if x.is_empty() { String::new() } else { format!("   {x}") } };
        let who = if user.is_empty() { "pulse-native".to_string() } else { format!("{user}@{host} · {theme}{upt}{heart}{title}") };
        let date = date.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
        let clock_el = div()
            .absolute()
            .top(px(h * 0.062))
            .left(px(0.0))
            .w(px(w))
            .flex()
            .flex_col()
            .items_center()
            .child(
                div()
                    .flex()
                    .items_start()
                    .child(div().text_size(px(150.0 * u)).line_height(px(150.0 * u)).font_weight(FontWeight::LIGHT).text_color(col(mix(pal.fg, [1.0, 1.0, 1.0], 0.6), 1.0)).child(hm))
                    .child(div().mt(px(14.0 * u)).ml(px(8.0 * u)).text_size(px(30.0 * u)).text_color(col(pal.accent, 1.0)).child(sec)),
            )
            .child(div().mt(px(10.0 * u)).w(px(260.0 * u)).h(px(2.0 * u)).bg(col(pal.accent, 0.8)))
            .child(div().mt(px(14.0 * u)).text_size(px(19.0 * u)).text_color(col(pal.accent, 1.0)).child(date))
            .child(div().mt(px(8.0 * u)).text_size(px(14.0 * u)).text_color(col(pal.fg, 0.75)).child(who))
            .child(match (num(&st, &["focus", "start"]), num(&st, &["focus", "until"])) {
                (Some(a), Some(b)) if b > a => {
                    let now = num(&st, &["t"]).unwrap_or(a);
                    let left = (b - now).max(0.0) as i64;
                    let done = (((now - a) / (b - a)) as f32).clamp(0.0, 1.0);
                    div().mt(px(10.0 * u)).flex().flex_col().items_center().gap(px(5.0 * u))
                        .child(div().text_size(px(15.0 * u)).text_color(col(pal.ok, 1.0)).child({ let lb = text(&st, &["focus", "label"]); if lb.is_empty() { format!("FOCUS · {:02}:{:02} left", left / 60, left % 60) } else { format!("FOCUS · {lb} · {:02}:{:02} left", left / 60, left % 60) } }))
                        .child(div().w(px(260.0 * u)).h(px(4.0 * u)).bg(col(pal.dim, 0.4)).child(div().h_full().w(gpui::relative(done)).bg(col(pal.ok, 0.95))))
                }
                _ if get(&st, &["meeting"]).as_bool().unwrap_or(false) => div().mt(px(10.0 * u)).text_size(px(15.0 * u)).text_color(col(pal.warn, 1.0)).child("in a meeting · effects are quiet"),
                _ if get(&st, &["away"]).as_bool().unwrap_or(false) => div().mt(px(10.0 * u)).text_size(px(15.0 * u)).text_color(col(pal.dim, 1.0)).child("away"),
                _ => div(),
            });

        // ---- system and network, on the left (as the web page's own panels)
        let cpu = num(&st, &["sys", "cpu"]).unwrap_or(0.0) as f32;
        let ram = num(&st, &["sys", "ram"]).unwrap_or(0.0) as f32;
        let temp = num(&st, &["sys", "temp"]).unwrap_or(0.0) as f32;
        let disk = num(&st, &["sys", "disk"]).unwrap_or(0.0) as f32;
        let cores = list(&st, &["sys", "cores"]);
        let mut core_row = div().flex().gap(px(2.0 * u)).h(px(34.0 * u)).items_end();
        for c in cores.iter().take(32) {
            let v = (c.as_f64().unwrap_or(0.0) as f32 / 100.0).clamp(0.02, 1.0);
            core_row = core_row.child(div().flex_1().h(gpui::relative(v)).bg(col(mix(pal.accent, pal.accent2, v), 0.9)));
        }
        let mut system = panel("system", &pal, u)
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(8.0 * u))
                    .child(div().text_size(px(46.0 * u)).line_height(px(48.0 * u)).font_weight(FontWeight::BOLD).text_color(col(heat(cpu / 100.0), 1.0)).child(format!("{cpu:.0}")))
                    .child(div().pb(px(6.0 * u)).text_color(col(pal.accent, 1.0)).child("% cpu")),
            )
            .child({
                let hist: Vec<f32> = list(&st, &["hist", "cpu"]).iter().filter_map(|x| x.as_f64().map(|v| v as f32)).collect();
                if hist.len() > 2 { div().child(spark(hist, pal.accent, u)) } else { div() }
            })
            .child(core_row)
            .child(meter("ram", ram / 100.0, format!("{ram:.0}%"), heat(ram / 100.0), &pal, u))
            .child(meter("temp", (temp - 30.0) / 65.0, format!("{temp:.0}°C"), heat((temp - 30.0) / 65.0), &pal, u))
            .child(meter("disk", disk / 100.0, format!("{disk:.0}%"), heat(disk / 100.0), &pal, u));
        if let Some(bat) = num(&st, &["sys", "bat"]) {
            system = system.child(meter("bat", bat as f32 / 100.0, format!("{bat:.0}%"), if bat < 20.0 { pal.crit } else { pal.ok }, &pal, u));
        }
        let network = panel("network", &pal, u)
            .child(div().flex().justify_between().child(div().text_color(col(pal.dim, 1.0)).child("↓ down")).child(div().text_color(col(pal.fg, 1.0)).child(bytes(num(&st, &["sys", "rx"]).unwrap_or(0.0)))))
            .child(div().flex().justify_between().child(div().text_color(col(pal.dim, 1.0)).child("↑ up")).child(div().text_color(col(pal.fg, 1.0)).child(bytes(num(&st, &["sys", "tx"]).unwrap_or(0.0)))))
            .child(div().text_color(col(pal.dim, 1.0)).child(if private { "•••".to_string() } else { text(&st, &["sys", "ip"]) }));
        let now = num(&st, &["t"]).unwrap_or(0.0);
        let mut mine: Vec<&Value> = list(&st, &["widgets"]).iter().collect();
        mine.sort_by(|a, b| a.get("order").and_then(Value::as_f64).unwrap_or(50.0).total_cmp(&b.get("order").and_then(Value::as_f64).unwrap_or(50.0)));
        let slot = |w: &Value| w.get("slot").and_then(Value::as_str).unwrap_or("right").to_string();
        let col_w = 396.0 * u;
        // every panel with its id, its column and its place: yours can move them (omarchy-pulse layout edit)
        let mut items: Vec<(String, String, f64, gpui::Div)> = Vec::new();
        if get(&st, &["ui", "system"]).as_bool().unwrap_or(true) {
            items.push(("sys".into(), "left".into(), 0.0, system));
        }
        if get(&st, &["ui", "network"]).as_bool().unwrap_or(true) {
            items.push(("net".into(), "left".into(), 10.0, network));
        }
        let mut left = div().absolute().left(px(36.0 * u)).top(px(72.0 * u)).w(px(col_w)).flex().flex_col().gap(px(24.0 * u));

        // ---- weather and the AI agents, on the right
        let mut right = div().absolute().right(px(36.0 * u)).top(px(72.0 * u)).w(px(col_w)).flex().flex_col().gap(px(24.0 * u));
        if get(&st, &["weather", "temp"]).is_number() {
            let unit = text(&st, &["weather", "unit"]);
            let wunit = text(&st, &["weather", "windUnit"]);
            let tmp = num(&st, &["weather", "temp"]).unwrap_or(0.0);
            let weather_panel = (
                panel(&format!("weather · {}", text(&st, &["weather", "city"])), &pal, u)
                    .child(
                        div()
                            .flex()
                            .items_end()
                            .gap(px(8.0 * u))
                            .child(div().text_size(px(46.0 * u)).line_height(px(48.0 * u)).font_weight(FontWeight::BOLD).text_color(col(pal.fg, 1.0)).child(format!("{tmp:.0}")))
                            .child(div().pb(px(6.0 * u)).text_color(col(pal.accent, 1.0)).child(unit.clone()))
                            .child(div().pb(px(6.0 * u)).ml(px(8.0 * u)).text_color(col(pal.accent, 1.0)).child(weather_word(num(&st, &["weather", "code"]).unwrap_or(0.0) as i32))),
                    )
                    .child(div().flex().justify_between().text_color(col(pal.dim, 1.0)).child(format!("feels {:.0}°", num(&st, &["weather", "feels"]).unwrap_or(tmp))).child(format!(
                        "↑{:.0}° ↓{:.0}°",
                        num(&st, &["weather", "hi"]).unwrap_or(tmp),
                        num(&st, &["weather", "lo"]).unwrap_or(tmp)
                    )))
                    .child(div().flex().justify_between().text_color(col(pal.dim, 1.0)).child(format!("wind {:.0} {wunit}", num(&st, &["weather", "wind"]).unwrap_or(0.0))).child(format!(
                        "clouds {:.0}%",
                        num(&st, &["weather", "cloud"]).unwrap_or(0.0)
                    )))
                    .child({
                        // the next hours: a bar each, as tall as the chance of rain, the hour under it
                        let hours = list(&st, &["weather", "hours"]);
                        let mut row = div().flex().items_end().gap(px(2.0 * u)).h(px(52.0 * u));
                        for hr in hours.iter().take(12) {
                            let p = (hr.get("p").and_then(Value::as_f64).unwrap_or(0.0) as f32 / 100.0).clamp(0.0, 1.0);
                            row = row.child(
                                div().flex_1().h_full().flex().flex_col().justify_end().items_center().gap(px(2.0 * u))
                                    .child(div().w_full().h(px((4.0 + 30.0 * p) * u)).bg(col(mix(pal.accent, pal.accent2, p), 0.9)))
                                    .child(div().text_size(px(9.5 * u)).text_color(col(pal.dim, 1.0)).child(hr.get("h").and_then(Value::as_str).unwrap_or("").to_string())),
                            );
                        }
                        if hours.is_empty() { div() } else { row }
                    })
                    .child(div().flex().justify_between().text_size(px(12.5 * u)).text_color(col(pal.dim, 1.0)).child(format!("☀ {}", text(&st, &["weather", "sunrise"]))).child(match num(&st, &["air", "aqi"]) {
                        Some(aqi) => format!("air {aqi:.0}"),
                        None => format!("hum {:.0}%", num(&st, &["weather", "hum"]).unwrap_or(0.0)),
                    }).child(format!("☾ {}", text(&st, &["weather", "sunset"]))))
                    .child(match (num(&st, &["air", "aqi"]), text(&st, &["air", "level"]), text(&st, &["air", "pollen"])) {
                        // the air you breathe, under the hours
                        (Some(aqi), level, pollen) => div().flex().justify_between().text_size(px(12.5 * u)).text_color(col(pal.dim, 1.0)).child(div().flex().gap(px(5.0 * u)).child("air").child(div().text_color(col(match level.as_str() { "good" | "fair" => pal.ok, "moderate" => pal.warn, _ => pal.crit }, 1.0)).child(level.clone())).child(format!("· {aqi:.0}"))).child(if pollen.is_empty() { "no pollen".to_string() } else { format!("{pollen} pollen {}", text(&st, &["air", "pollenLevel"])) }),
                        _ => div(),
                    }),
            );
            items.push(("weather".into(), "right".into(), 0.0, weather_panel.0));
        }
        {
            let mut counts = std::collections::HashMap::<String, f64>::new();
            for wd in mine.iter() {
                let sl = slot(wd);
                if !matches!(sl.as_str(), "left" | "right" | "center") { continue }
                let n = counts.entry(sl.clone()).or_insert(if sl == "left" { 20.0 } else if sl == "right" { 10.0 } else { 0.0 });
                items.push((text(wd, &["id"]), sl, *n, widget(wd, &pal, u, now, 4, private, &self.base, self.cams.get(&text(wd, &["id"])).map(|c| c.1.clone()))));
                *n += 10.0;
            }
        }
        if scene_name == "city" {
            let agents = list(&st, &["agents"]);
            if !agents.is_empty() {
                let mut p = panel("agents", &pal, u);
                for a in agents.iter().take(6) {
                    let state = a.get("state").and_then(Value::as_str).unwrap_or("");
                    let c = match state {
                        "working" => pal.accent,
                        "waiting" => pal.warn,
                        "done" => pal.ok,
                        "failed" => pal.crit,
                        _ => pal.dim,
                    };
                    p = p.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0 * u))
                            .child(div().size(px(8.0 * u)).rounded_full().bg(col(c, 1.0)))
                            .child(div().flex_1().text_color(col(pal.fg, 1.0)).child(a.get("name").and_then(Value::as_str).unwrap_or("agent").to_string()))
                            .child(div().text_color(col(c, 1.0)).child(state.to_string())),
                    );
                }
                right = right.child(p);
            }
        }
        // ---- the centre: your first centre panel, under the clock (the AI brief, on the web page)
        let mut centre = div().absolute().top(px(h * 0.325)).left(px(w * 0.31)).w(px(w * 0.38)).flex().flex_col().gap(px(20.0 * u));
        {
            // the saved layout (~/.config/omarchy-pulse/layout.json, through the hub): a column, a place, or hidden;
            // in NOC mode the personal panels go away
            let draft_v = self.draft.clone().map(Value::Object);
            let layout = draft_v.as_ref().unwrap_or_else(|| get(&st, &["layout"]));
            self.boxes.borrow_mut().clear();
            let noc_on = get(&st, &["noc", "on"]).as_bool().unwrap_or(false);
            let noc_hide: Vec<String> = get(&st, &["ui", "nocHide"]).as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| vec!["media".into(), "notifications".into()]);
            let mut placed: Vec<(String, f64, gpui::Div)> = Vec::new();
            let dragging = self.drag.clone();
            for (id, mut sl, mut order, el) in items {
                let o = get(layout, &[id.as_str()]);
                if get(o, &["hidden"]).as_bool().unwrap_or(false) || (noc_on && noc_hide.contains(&id)) { continue }
                if let Some(x) = get(o, &["slot"]).as_str().filter(|x| matches!(*x, "left" | "right" | "center")) { sl = x.to_string() }
                if let Some(x) = num(o, &["order"]) { order = x }
                let el = if editing {
                    // layout edit: every panel can be picked up (left button) or hidden (right button); a hidden recorder
                    // says where it is on the screen, so a drop knows which panel it lands before
                    let (boxes, id_box, slot_box) = (self.boxes.clone(), id.clone(), sl.clone());
                    let (id_down, id_hide) = (id.clone(), id.clone());
                    div().relative().border_1().border_color(col(if dragging.as_deref() == Some(id.as_str()) { pal.warn } else { pal.accent2 }, 0.9)).cursor_pointer().child(el)
                        .child(canvas(move |b, _, _| { boxes.borrow_mut().insert(id_box, (slot_box, f32::from(b.origin.y), f32::from(b.size.height))); }, |_, _, _, _| ()).absolute().size_full())
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _: &MouseDownEvent, _, cx| { this.drag = Some(id_down.clone()); cx.stop_propagation(); }))
                        .on_mouse_down(MouseButton::Right, cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                            if let Some(d) = this.draft.as_mut() { d.entry(id_hide.clone()).or_insert_with(|| serde_json::json!({}))["hidden"] = serde_json::json!(true) }
                            cx.stop_propagation();
                        }))
                } else {
                    el
                };
                placed.push((sl, order, el));
            }
            placed.sort_by(|a, b| a.1.total_cmp(&b.1));
            let (mut nl, mut nr, mut nc) = (0, 0, 0);
            for (sl, _, el) in placed {
                match sl.as_str() {
                    "left" if nl < 5 => { nl += 1; left = left.child(el) }
                    "right" if nr < 4 => { nr += 1; right = right.child(el) }
                    "center" if nc < 2 && !simple => { nc += 1; centre = centre.child(el) }
                    _ => {}
                }
            }
            if get(&st, &["focus"]).is_object() && !editing {
                (left, right, centre) = (left.opacity(0.16), right.opacity(0.16), centre.opacity(0.16)); // a focus: the panels step back
            } else if busy {
                (left, right, centre) = (left.opacity(0.9), right.opacity(0.9), centre.opacity(0.9));
            }
            if !editing {
                centre = centre.max_h(px(296.0 * u)).overflow_hidden(); // as the web page: the middle column never covers the scene
            }
        }
        // ---- the line at the bottom: the next launch, as the web page's ticker says it
        let mut lines: Vec<String> = Vec::new();
        for wd in mine.iter().filter(|w| slot(w) == "ticker") {
            for it in get(wd, &["data", "items"]).as_array().map(|a| a.as_slice()).unwrap_or(&[]).iter().take(8) {
                let (a, b) = (text(it, &["title"]), text(it, &["text"]));
                if !a.is_empty() {
                    lines.push(if b.is_empty() { a } else { format!("{a} · {b}") });
                }
            }
        }
        if lines.is_empty() {
            let (r, m, s2) = (text(&st, &["space", "launch", "rocket"]), text(&st, &["space", "launch", "mission"]), text(&st, &["space", "launch", "status"]));
            if !r.is_empty() {
                lines.push(format!("{r} · {m} · {s2}"));
            }
        }
        for a in list(&st, &["alerts"]).iter().rev().take(3).rev() {
            let at = num(a, &["t"]).unwrap_or(0.0);
            if hub_now - at < 3600.0 {
                lines.push(format!("{} · {} · {}", text(a, &["title"]), text(a, &["text"]), ago(at, hub_now)));
            }
        }
        if get(&st, &["weather", "temp"]).is_number() {
            lines.push(format!("{} {:.0}{} {} · sunset {}", text(&st, &["weather", "city"]), num(&st, &["weather", "temp"]).unwrap_or(0.0), text(&st, &["weather", "unit"]), weather_word(num(&st, &["weather", "code"]).unwrap_or(0.0) as i32).to_lowercase(), text(&st, &["weather", "sunset"])));
        }
        if !text(&st, &["sys", "host"]).is_empty() {
            lines.push(format!("omarchy pulse · {} live sources", list(&st, &["widgets"]).len()));
        }
        let ticker = if lines.is_empty() || private { String::new() } else { format!("◆ {}   {}/{}", lines[((t / 6.0) as usize) % lines.len()], ((t / 6.0) as usize) % lines.len() + 1, lines.len()) };
        let say = self.world.borrow().say.clone().filter(|(at, _)| t - at < 6.0);
        let say_el = div().absolute().top(px(h * 0.56)).left(px(0.0)).w(px(w)).flex().justify_center().child(match say {
            Some((at, words)) => div().px(px(22.0 * u)).py(px(8.0 * u)).bg(col(pal.bg, 0.7)).border_1().border_color(col(pal.accent2, 0.8)).text_size(px(26.0 * u)).text_color(col(pal.fg, (1.0 - ((t - at) as f32 - 5.0).max(0.0)).clamp(0.0, 1.0))).child(words),
            None => div(),
        });
        let ticker_el = div().absolute().bottom(px(8.0 * u)).left(px(0.0)).w(px(w)).flex().justify_center().text_size(px(14.0 * u)).text_color(col(pal.fg, 0.8)).child(ticker);

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
        let (labels, centre_at, prog) = { let wd = self.world.borrow(); (wd.labels.clone(), wd.centre, wd.prog) };
        for l in labels.iter().take(160) {
            let size = l.size.max(8.0);
            words = words.child(div().absolute().left(px(l.x - w * 0.2)).top(px(l.y)).w(px(w * 0.4)).flex().justify_center().whitespace_nowrap().text_size(px(size)).line_height(px(size * 1.25)).font_weight(if l.bold { FontWeight::BOLD } else { FontWeight::NORMAL }).text_color(col(l.c, l.a)).child(l.text.clone()));
        }
        // ---- what the sphere shows when it is not its face: the clock, the weather, the news, a countdown
        if let (Some((sx, sy, sr)), true) = (centre_at, scene_name == "sphere" && !still_picture) {
            let (big, small): (String, String) = match prog {
                "clock" => (hm0.clone(), date0.clone()),
                "weather" if get(&st, &["weather", "temp"]).is_number() => (format!("{:.0}{}", num(&st, &["weather", "temp"]).unwrap_or(0.0), text(&st, &["weather", "unit"])), format!("{} · {}", weather_word(num(&st, &["weather", "code"]).unwrap_or(0.0) as i32), text(&st, &["weather", "city"]))),
                "news" => {
                    let nw = list(&st, &["widgets"]).iter().find(|x| matches!(text(x, &["id"]).as_str(), "news" | "ai"));
                    let line = nw.map(|x| { let it = text(x, &["data", "items", "0", "title"]); if !it.is_empty() { it } else { list(x, &["data", "items"]).first().map(|i0| text(i0, &["title"])).filter(|v| !v.is_empty()).unwrap_or_else(|| text(x, &["data", "text"]).lines().next().unwrap_or("").to_string()) } }).unwrap_or_default();
                    (String::new(), if private { String::new() } else { line.chars().take(64).collect() })
                }
                "launch" => { let left = (num(&st, &["space", "launch", "net"]).unwrap_or(hub_now) - hub_now).max(0.0) as i64; (format!("T-{:02}:{:02}", left / 60, left % 60), format!("{} · {}", text(&st, &["space", "launch", "rocket"]), text(&st, &["space", "launch", "mission"]))) }
                "focus" => { let left = (num(&st, &["focus", "until"]).unwrap_or(hub_now) - hub_now).max(0.0) as i64; (format!("{:02}:{:02}", left / 60, left % 60), { let lb = text(&st, &["focus", "label"]); if lb.is_empty() { "FOCUS".to_string() } else { lb } }) }
                "meeting" => { let c0 = list(&st, &["calendar"]).first().cloned().unwrap_or(Value::Null); (format!("{:.0}", (num(&c0, &["start"]).unwrap_or(hub_now) - hub_now).max(0.0)), if private { "meeting".to_string() } else { text(&c0, &["title"]) }) }
                "car" => (format!("{:.0}%", num(&st, &["car", "battery"]).unwrap_or(0.0)), format!("{} · {:.0} {}", match text(&st, &["car", "state"]).as_str() { "Complete" => "CHARGED", "Charging" => "CHARGING", _ => "CAR" }, num(&st, &["car", "range"]).unwrap_or(0.0), text(&st, &["car", "unit"]).to_uppercase())),
                "alert" => (String::new(), self.incoming.as_ref().map(|a| text(&a.1, &["title"])).unwrap_or_default()),
                _ => (String::new(), String::new()),
            };
            if let (true, Some((_, im)), false) = (prog == "camera", self.cams.values().next(), private) {
                // your camera, across the sphere
                words = words.child(gpui::img(im.clone()).absolute().left(px(sx - sr * 0.8)).top(px(sy - sr * 0.42)).w(px(sr * 1.6)).h(px(sr * 0.84)).object_fit(ObjectFit::Cover).rounded(px(sr * 0.08)));
            }
            if !big.is_empty() || !small.is_empty() {
                words = words.child(
                    div().absolute().left(px(sx - sr)).top(px(sy - sr * 0.42)).w(px(sr * 2.0)).h(px(sr * 0.84)).flex().flex_col().items_center().justify_center().gap(px(sr * 0.04))
                        .child(if big.is_empty() { div() } else { div().text_size(px(sr * 0.44)).line_height(px(sr * 0.46)).font_weight(FontWeight::BOLD).text_color(col(mix(pal.fg, [1.0, 1.0, 1.0], 0.5), 1.0)).child(big) })
                        .child(div().px(px(sr * 0.25)).text_size(px(sr * if prog == "news" { 0.10 } else { 0.085 })).text_color(col(pal.accent, 1.0)).child(small)),
                );
            }
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
                div().absolute().top(px(h * 0.34)).left(px(0.0)).w(px(w)).flex().justify_center().child(
                    div().min_w(px(520.0 * u)).max_w(px(980.0 * u)).px(px(36.0 * u)).py(px(22.0 * u)).flex().flex_col().items_center().gap(px(8.0 * u)).bg(col(pal.bg, 0.86)).border_2().border_color(col(c, 1.0))
                        .child(div().text_size(px(15.0 * u)).text_color(col(c, if (t * 2.0) as i64 % 2 == 0 { 1.0 } else { 0.3 })).child("I N C O M I N G"))
                        .child(div().text_size(px(36.0 * u)).text_color(col(pal.fg, 1.0)).child(title))
                        .child(div().text_size(px(18.0 * u)).text_color(col(pal.fg, 0.75)).child(if private { String::new() } else { text(a, &["text"]) })),
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
        let root = div().relative().size_full().bg(col(pal.bg, 1.0)).font_family(self.font.clone()).text_size(px(15.0 * u)).text_color(col(pal.fg, 1.0));
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

/// A picture from a data: URL (PNG, JPEG, WebP, GIF or SVG), or nothing.
fn data_image(url: &str) -> Option<std::sync::Arc<gpui::Image>> {
    use base64::Engine;
    let (head, body) = url.strip_prefix("data:")?.split_once(',')?;
    let format = if head.contains("svg") { gpui::ImageFormat::Svg } else if head.contains("jpeg") || head.contains("jpg") { gpui::ImageFormat::Jpeg } else if head.contains("webp") { gpui::ImageFormat::Webp } else if head.contains("gif") { gpui::ImageFormat::Gif } else { gpui::ImageFormat::Png };
    let bytes = if head.contains("base64") { base64::engine::general_purpose::STANDARD.decode(body.trim()).ok()? } else { body.as_bytes().to_vec() };
    Some(std::sync::Arc::new(gpui::Image::from_bytes(format, bytes)))
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
fn weather_word(code: i32) -> &'static str {
    match code {
        0 => "CLEAR",
        1 | 2 => "FAIR",
        3 => "CLOUDY",
        45 | 48 => "FOG",
        51..=57 => "DRIZZLE",
        61..=67 => "RAIN",
        71..=77 => "SNOW",
        80..=82 => "SHOWERS",
        85 | 86 => "SNOW",
        95..=99 => "STORM",
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
        let mut screens: Vec<Option<gpui::DisplayId>> = cx.displays().iter().map(|d| Some(d.id())).collect();
        screens.sort_by_key(|d| *d != first); // the first monitor first: alerts and sound live there
        if a.window || screens.is_empty() {
            screens = vec![None];
        } else if secondary_off {
            screens.truncate(1);
        }
        let mut opened = 0;
        for (i, screen) in screens.into_iter().enumerate() {
            let mut options = WindowOptions::new().titlebar(None).app_id(Some("omarchy-pulse-native".to_string()));
            options = if a.window {
                options.window_bounds(Some(WindowBounds::Windowed(Bounds { origin: point(px(0.), px(0.)), size: Size::new(px(1280.), px(720.)) })))
            } else {
                // the wallpaper: the background layer, stretched to every edge, under the bar too, and never taking the keyboard
                options
                    .window_bounds(Some(WindowBounds::Windowed(Bounds { origin: point(px(0.), px(0.)), size: size(px(1920.), px(1080.)) })))
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
