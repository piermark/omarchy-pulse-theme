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
mod hub;
mod life;
mod palette;
mod scene;

use gpui::{
    canvas, div, layer_shell::*, point, prelude::*, px, size, App, Bounds, Context, FontWeight, LinuxWindowBackground, QuitMode, SharedString, Size,
    Window, WindowBounds, WindowKind, WindowOptions,
};
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
}

impl Pulse {
    fn new(hub: Shared, live: FxQueue, fps: u64, font: String, scene: String, cx: &mut Context<Self>) -> Self {
        let every = Duration::from_millis(1000 / fps.clamp(5, 60));
        cx.spawn(async move |this, cx| loop {
            if this.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
            cx.background_executor().timer(every).await;
        })
        .detach();
        Pulse { hub, live, forced: !scene.is_empty(), scene, city: Rc::new(RefCell::new(city::World::new())), world: Rc::new(RefCell::new(scene::World::new())), marks: Rc::new(RefCell::new(Vec::new())), mem: Rc::new(RefCell::new(design::Mem::default())), drawn: Rc::new(RefCell::new((Vec::new(), Vec::new()))), assets: (String::new(), Default::default()), reacted: 0.0, started: Instant::now(), font: font.into() }
    }
}

fn clock(st: &Value) -> (String, String, String) {
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
    (format!("{hh:02}:{mm:02}"), format!("{ss:02}"), format!("{wd} {day} {mon} {year}"))
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
fn widget(wd: &Value, pal: &Pal, u: f32, now: f64, rows_max: usize, private: bool) -> gpui::Div {
    let s = |k: &str| wd.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let data = wd.get("data").unwrap_or(&Value::Null);
    let title = if s("title").is_empty() { s("id") } else { s("title") };
    let mut p = panel(&title, pal, u);
    let dim = col(pal.dim, 1.0);
    if private {
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
            for it in data.get("items").and_then(Value::as_array).map(|a| a.as_slice()).unwrap_or(&[]).iter().take(rows_max) {
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
                p = p.child(div().flex().justify_between().child(div().text_color(dim).child(cell(0))).child(div().text_color(col(pal.fg, 1.0)).child(cell(1))));
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
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let snap = self.hub.lock().map(|s| s.clone()).unwrap_or_default();
        let st = snap.state.clone();
        let pal = Pal::from_state(&st);
        let vp = window.viewport_size();
        let (w, h) = (f32::from(vp.width), f32::from(vp.height));
        let u = (h / 1080.0).max(0.5);
        let t = self.started.elapsed().as_secs_f64();
        let (hm, sec, date) = clock(&st);
        // the hub's effects (a launch, fireworks, lightning...), as the web page plays them
        let fx: Vec<hub::Fx> = self.live.lock().map(|mut q| q.drain(..).collect()).unwrap_or_default();
        for f in &fx {
            self.world.borrow_mut().fx(&f.kind, &f.text, f.small, t);
        }
        let hub_scene = text(&st, &["ui", "scene"]);
        let scene_name = if self.forced { self.scene.clone() } else if hub_scene.is_empty() { "skyline".to_string() } else { hub_scene };
        let sphere = scene_name == "sphere";
        // a scene made with the editor (or a community .pulse): its design, drawn over Pulse's real sky
        let design = snap.design.clone().filter(|d| d.0 == scene_name);
        let scene_st = if get(&st, &["scene"]).is_object() { get(&st, &["scene"]).clone() } else { st.clone() };
        if let Some((name, d)) = &design {
            if &self.assets.0 != name {
                self.assets = (name.clone(), load_assets(d));
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
        let (design2, scene_st2, mem2, drawn_out) = (design.clone(), scene_st.clone(), self.mem.clone(), self.drawn.clone());
        let sky = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let marks = if is_city { city::paint(bounds, &mut old.borrow_mut(), &st2, &pal2, t, window) } else { scene::paint(bounds, &mut world.borrow_mut(), &st2, &pal2, t, sphere, bare, window) };
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
        let who = if user.is_empty() { "pulse-native".to_string() } else { format!("{user}@{host} · pulse-native{upt}") };
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
            .child(div().mt(px(8.0 * u)).text_size(px(14.0 * u)).text_color(col(pal.fg, 0.75)).child(who));

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
        let mut left = div().absolute().left(px(36.0 * u)).top(px(72.0 * u)).w(px(col_w)).flex().flex_col().gap(px(24.0 * u)).child(system).child(network);
        for wd in mine.iter().filter(|w| slot(w) == "left").take(3) {
            left = left.child(widget(wd, &pal, u, now, 4, private));
        }

        // ---- weather and the AI agents, on the right
        let mut right = div().absolute().right(px(36.0 * u)).top(px(72.0 * u)).w(px(col_w)).flex().flex_col().gap(px(24.0 * u));
        if get(&st, &["weather", "temp"]).is_number() {
            let unit = text(&st, &["weather", "unit"]);
            let wunit = text(&st, &["weather", "windUnit"]);
            let tmp = num(&st, &["weather", "temp"]).unwrap_or(0.0);
            right = right.child(
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
                    ))),
            );
        }
        for wd in mine.iter().filter(|w| slot(w) == "right").take(3) {
            right = right.child(widget(wd, &pal, u, now, 4, private));
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
        if let Some(wd) = mine.iter().find(|w| slot(w) == "center") {
            centre = centre.child(widget(wd, &pal, u, now, 4, private));
        }
        // ---- the line at the bottom: the next launch, as the web page's ticker says it
        let ticker = match (text(&st, &["space", "launch", "rocket"]), text(&st, &["space", "launch", "mission"]), text(&st, &["space", "launch", "status"])) {
            (r, m, s2) if !r.is_empty() => format!("◆ {r} · {m} · {s2}"),
            _ => String::new(),
        };
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
                if let Some(im) = self.assets.1.get(&pc.asset) {
                    scene_layer = scene_layer.child(gpui::img(im.clone()).absolute().left(px(pc.x - pc.w / 2.0)).top(px(pc.y - pc.h / 2.0)).w(px(pc.w)).h(px(pc.h)).opacity(pc.a));
                }
            }
            for wd in &drawn.0 {
                scene_layer = scene_layer.child(
                    div().absolute().left(px(wd.x - w * 0.3)).top(px(wd.y - wd.size * 0.7)).w(px(w * 0.6)).flex().justify_center().text_size(px(wd.size)).line_height(px(wd.size * 1.4)).font_weight(if wd.bold { FontWeight::EXTRA_BOLD } else { FontWeight::MEDIUM }).text_color(col(wd.c, wd.a)).child(wd.text.clone()),
                );
            }
        }
        let status = match (&snap.at, &snap.error) {
            (None, Some(e)) => format!("waiting for the hub (omarchy-pulse): {e}"),
            (None, None) => "waiting for the hub (omarchy-pulse)…".to_string(),
            (Some(at), _) if at.elapsed() > Duration::from_secs(8) => "the hub stopped answering: showing the last it said".to_string(),
            _ if snap.handwritten.as_deref() == Some(scene_name.as_str()) => format!("the scene \"{scene_name}\" is written in code: the web page runs it; here, the skyline"),
            _ => String::new(),
        };
        let foot = div().absolute().bottom(px(34.0 * u)).left(px(0.0)).w(px(w)).flex().justify_center().text_size(px(13.0 * u)).text_color(col(pal.warn, 0.9)).child(status);

        div().relative().size_full().bg(col(pal.bg, 1.0)).font_family(self.font.clone()).text_size(px(15.0 * u)).text_color(col(pal.fg, 1.0)).child(sky).child(scene_layer).child(clock_el).child(tags).child(say_el).child(centre).child(left).child(right).child(ticker_el).child(foot)
    }
}

/// The pictures of a design: its assets are data: URLs (PNG, JPEG, WebP, GIF or SVG), so a scene stays one file.
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
    let mut a = Args { window: false, url: format!("http://127.0.0.1:{}", hub::port()), fps: 20, font: FONT.to_string(), scene: String::new() };
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
                println!("pulse-native {}: Omarchy Pulse, drawn natively (Rust + GPUI)\n\n  pulse-native            the wallpaper (a Wayland layer-shell surface behind your windows)\n  pulse-native --window   an ordinary window\n  --url URL   the hub (default: http://127.0.0.1:PORT from PULSE_PORT or pulse.toml)\n  --fps N     frames a second, 5 to 60 (default 20)\n  --font NAME the font (default: {FONT})\n  --scene S   the scene: by default the hub's own ([pulse] scene); skyline, sphere, or city (pulse-native's first world)", env!("CARGO_PKG_VERSION"));
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
        let (fps, font, scene, hub, live) = (a.fps, a.font.clone(), a.scene.clone(), shared.clone(), live.clone());
        if let Err(e) = cx.open_window(options, move |_, cx| cx.new(|cx| Pulse::new(hub, live, fps, font, scene, cx))) {
            eprintln!("pulse-native: could not open the wallpaper: {e}\n(it needs a Wayland compositor with wlr-layer-shell, such as Hyprland; try --window)");
            std::process::exit(1);
        }
    });
}
