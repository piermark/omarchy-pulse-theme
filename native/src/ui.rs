//! The panels, the clock and the ticker of Pulse 1.8, as pulse.css and pulse.js build them: every size is in rem, and
//! 1rem is the screen's height / 68 (its width / 92 on a screen no wider than 4:3), so the same layout fits every
//! screen, from a 1366×768 laptop to a 4K monitor. A panel that does not fit in its column is hidden, never cut.

use crate::hub::{get, list, num, text};
use crate::palette::{col, Pal, C};
use gpui::{canvas, div, point, prelude::*, px, relative, FontWeight, PathBuilder};
use serde_json::Value;

/// One screen's measures: the rem, and the palette.
pub struct Ui<'a> {
    pub r: f32,
    pub pal: &'a Pal,
    pub private: bool,
    /// the hub's clock, in Unix seconds
    pub now: f64,
}

/// pulse.css: `html { font-size: calc(100vh / 68) }`, and `100vw / 92` when the screen is no wider than 4:3.
pub fn rem(w: f32, h: f32) -> f32 {
    if w / h.max(1.0) <= 4.0 / 3.0 { w / 92.0 } else { h / 68.0 }
}

/// A panel and how tall it is, in rem (the web page measures it after layout; the sizes here are the CSS ones).
pub struct Panel {
    pub id: String,
    pub el: gpui::Div,
    pub h: f32,
}

impl Ui<'_> {
    fn fg(&self, a: f32) -> gpui::Rgba {
        col(self.pal.fg, a)
    }

    /// pulse.css .panel: the Omarchy box, its legend cut into the top border, two brackets at the bottom corners.
    fn panel(&self, title: &str, icon: &str, level: &str, corner: &str, err: &str, body: gpui::Div) -> gpui::Div {
        let (r, p) = (self.r, self.pal);
        let border = match level {
            "warn" => col(p.warn, 0.7),
            "crit" => col(p.crit, 0.9),
            _ => col(p.accent, 0.32),
        };
        let legend_c = match level {
            "warn" => p.warn,
            "crit" => p.crit,
            _ => p.accent,
        };
        let bracket = |left: bool| {
            let b = div().absolute().bottom(px(-2.0)).w(px(0.9 * r)).h(px(0.9 * r)).border_b_2().border_color(col(p.accent, 0.9));
            if left { b.left(px(-2.0)).border_l_2() } else { b.right(px(-2.0)).border_r_2() }
        };
        let mut el = div()
            .relative()
            .flex_none()
            .w_full()
            .pt(px(1.25 * r))
            .px(px(1.2 * r))
            .pb(px(1.0 * r))
            .border_1()
            .border_color(border)
            .bg(gpui::linear_gradient(180.0, gpui::linear_color_stop(col(p.bg, 0.78), 0.0), gpui::linear_color_stop(col(p.bg, 0.52), 1.0)))
            .child(body)
            .child(bracket(true))
            .child(bracket(false))
            .child(
                div()
                    .absolute()
                    .top(px(-0.72 * r))
                    .left(px(0.9 * r))
                    .px(px(0.5 * r))
                    .bg(col(p.bg, 1.0))
                    .flex()
                    .text_size(px(0.95 * r))
                    .line_height(px(1.3 * r))
                    .text_color(col(legend_c, 1.0))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .max_w(px(18.6 * r))
                    .child(if icon.is_empty() { div() } else { div().mr(px(0.45 * r)).text_color(col(p.accent2, 1.0)).child(icon.to_string()) })
                    .child(title.to_string()),
            );
        if level == "crit" {
            el = el.shadow(vec![gpui::BoxShadow { color: col(p.crit, 0.35).into(), offset: point(px(0.), px(0.)), blur_radius: px(1.4 * r), spread_radius: px(0.), inset: false }]);
        }
        if !corner.is_empty() {
            el = el.child(div().absolute().top(px(-0.65 * r)).right(px(0.9 * r)).px(px(0.4 * r)).bg(col(p.bg, 1.0)).text_size(px(0.8 * r)).line_height(px(1.2 * r)).text_color(col(p.dim, 1.0)).child(corner.to_string()));
        }
        // pulse.css .panel .err: the line is always there (margin-top .5rem), empty when all is well
        el = el.child(if err.is_empty() { div().mt(px(0.5 * r)) } else { div().mt(px(0.5 * r)).text_size(px(0.8 * r)).line_height(px(0.96 * r)).text_color(col(p.warn, 0.85)).whitespace_nowrap().overflow_hidden().child(format!("⚠ {err}")) });
        el
    }

    /// pulse.css .row: a key on the left at 60%, its value on the right.
    fn row(&self, k: impl Into<gpui::SharedString>, v: impl Into<gpui::SharedString>) -> gpui::Div {
        let r = self.r;
        div().flex().justify_between().items_end().gap(px(0.8 * r)).text_size(px(1.02 * r)).line_height(px(1.785 * r))
            .child(div().text_color(self.fg(0.6)).whitespace_nowrap().child(k.into()))
            .child(div().text_color(self.fg(1.0)).whitespace_nowrap().overflow_hidden().child(v.into()))
    }

    /// pulse.css .big with its small unit.
    fn big(&self, value: String, unit: &str, size: f32) -> gpui::Div {
        let r = self.r;
        div().flex().items_end().line_height(px(size * r))
            .child(div().text_size(px(size * r)).font_weight(FontWeight::LIGHT).text_color(self.fg(1.0)).child(value))
            .child(if unit.is_empty() { div() } else { div().ml(px(0.3 * r)).pb(px(0.12 * size * r)).text_size(px(1.2 * r)).text_color(col(self.pal.accent, 1.0)).child(unit.to_string()) })
    }

    fn note(&self, t: &str) -> gpui::Div {
        let r = self.r;
        div().mt(px(0.35 * r)).text_size(px(0.9 * r)).line_height(px(1.08 * r)).text_color(self.fg(0.6)).whitespace_nowrap().overflow_hidden().child(t.to_string())
    }

    /// pulse.css .seg: a bar of segments, .42rem lit and a gap, as full as the value.
    fn seg(&self, pct: f32, c: Option<C>, hgt: f32) -> gpui::Div {
        let (r, p) = (self.r, self.pal);
        let c = c.unwrap_or(p.accent);
        let (dim, on) = (col(p.dim, 0.45), col(c, 1.0));
        let pct = pct.clamp(0.0, 100.0) / 100.0;
        let h = hgt * r;
        div().w_full().h(px(h)).child(canvas(|_, _, _| (), move |b, _, win, _| {
            let (x0, y0, w) = (f32::from(b.origin.x), f32::from(b.origin.y), f32::from(b.size.width));
            let (step, cell) = (0.55 * r, 0.42 * r);
            let mut x = 0.0;
            while x < w {
                let cw = cell.min(w - x);
                let lit = x + cw <= w * pct + 0.5;
                win.paint_quad(gpui::PaintQuad::new(gpui::Bounds { origin: point(px(x0 + x), px(y0)), size: gpui::size(px(cw), px(h)) }).background(if lit { on } else { dim }));
                x += step;
            }
        }).size_full())
    }

    /// pulse.css .meter: name, segments, value.
    fn meter(&self, name: &str, pct: f32, value: String, c: Option<C>) -> gpui::Div {
        let r = self.r;
        div().flex().items_center().gap(px(0.6 * r)).text_size(px(1.0 * r)).line_height(px(1.7 * r))
            .child(div().w(px(3.4 * r)).flex_none().text_color(self.fg(0.6)).child(name.to_string()))
            .child(div().flex_1().py(px(0.25 * r)).child(self.seg(pct, c, 0.55)))
            .child(div().w(px(4.6 * r)).flex_none().flex().justify_end().text_color(self.fg(1.0)).whitespace_nowrap().child(value))
    }

    /// pulse.js spark(): a line for each series and its fill fading down, over a faint baseline.
    fn spark(&self, series: Vec<(Vec<f32>, C)>, max: f32, n: usize, hgt: f32) -> gpui::Div {
        let r = self.r;
        let dim = col(self.pal.dim, 0.35);
        div().mt(px(0.5 * r)).w_full().h(px(hgt * r)).child(canvas(|_, _, _| (), move |b, _, win, _| {
            let (x0, y0, w, h) = (f32::from(b.origin.x), f32::from(b.origin.y), f32::from(b.size.width), f32::from(b.size.height));
            let mut base = PathBuilder::stroke(px(1.0));
            base.move_to(point(px(x0), px(y0 + h - 0.5)));
            base.line_to(point(px(x0 + w), px(y0 + h - 0.5)));
            if let Ok(p) = base.build() {
                win.paint_path(p, dim);
            }
            let mut m = max;
            for s in &series {
                for v in &s.0 {
                    m = m.max(*v);
                }
            }
            if m <= 0.0 {
                m = 1.0;
            }
            for (v, c) in &series {
                if v.len() < 2 {
                    continue;
                }
                let nn = v.len().max(n);
                let step = w / (nn - 1) as f32;
                let xs = w - (v.len() - 1) as f32 * step;
                let pt = |i: usize| point(px(x0 + xs + i as f32 * step), px(y0 + h - 2.0 - v[i] / m * (h - 5.0)));
                let mut fill = PathBuilder::fill();
                fill.move_to(pt(0));
                for i in 1..v.len() {
                    fill.line_to(pt(i));
                }
                fill.line_to(point(px(x0 + w), px(y0 + h)));
                fill.line_to(point(px(x0 + xs), px(y0 + h)));
                fill.close();
                if let Ok(p) = fill.build() {
                    win.paint_path(p, gpui::linear_gradient(180.0, gpui::linear_color_stop(col(*c, 0.35), 0.0), gpui::linear_color_stop(col(*c, 0.0), 1.0)));
                }
                let mut line = PathBuilder::stroke(px(1.6));
                line.move_to(pt(0));
                for i in 1..v.len() {
                    line.line_to(pt(i));
                }
                if let Ok(p) = line.build() {
                    win.paint_path(p, col(*c, 0.95));
                }
            }
        }).size_full())
    }

    fn level_c(&self, level: &str) -> Option<C> {
        match level {
            "ok" => Some(self.pal.ok),
            "info" => Some(self.pal.accent),
            "warn" => Some(self.pal.warn),
            "crit" => Some(self.pal.crit),
            _ => None,
        }
    }

    /// A round dot in the colour of a level, glowing as .lv-* does.
    fn dot(&self, level: &str, size: f32) -> gpui::Div {
        let r = self.r;
        let c = self.level_c(level);
        let mut d = div().flex_none().w(px(size * r)).h(px(size * r)).rounded_full().bg(col(c.unwrap_or(self.pal.dim), 1.0));
        if let Some(c) = c {
            d = d.shadow(vec![gpui::BoxShadow { color: col(c, 1.0).into(), offset: point(px(0.), px(0.)), blur_radius: px(0.5 * r), spread_radius: px(0.), inset: false }]);
        }
        d
    }

    // ------------------------------------------------------------------ the three panels of Pulse itself

    /// pulse.js sysPanel().
    pub fn sys(&self, st: &Value) -> Panel {
        let r = self.r;
        let s = get(st, &["sys"]);
        let cpu = num(s, &["cpu"]).unwrap_or(0.0);
        let load: Vec<String> = list(s, &["load"]).iter().map(|v| v.as_f64().map(|x| format!("{x}")).unwrap_or_else(|| v.as_str().unwrap_or("").to_string())).collect();
        let head = div().flex().items_end().justify_between()
            .child(self.big(format!("{cpu:.0}"), "% cpu", 3.1))
            .child(div().flex().flex_col().items_end().text_size(px(0.9 * r)).line_height(px(1.35 * r)).text_color(self.fg(0.6))
                .child(format!("load {}", load.join(" ")))
                .child(format!("up {}", upt(num(s, &["uptime"]).unwrap_or(0.0)))));
        let hist: Vec<f32> = list(st, &["hist", "cpu"]).iter().filter_map(|x| x.as_f64().map(|v| v as f32)).collect();
        let mut cores = div().flex().items_end().gap(px(2.0)).h(px(2.2 * r)).mt(px(0.5 * r)).mb(px(0.6 * r));
        for c in list(s, &["cores"]).iter().take(32) {
            let v = (c.as_f64().unwrap_or(0.0) as f32).max(4.0) / 100.0;
            cores = cores.child(div().flex_1().h(relative(v.min(1.0))).min_h(px(2.0)).opacity(0.85).bg(gpui::linear_gradient(0.0, gpui::linear_color_stop(col(self.pal.accent, 0.9), 0.0), gpui::linear_color_stop(col(self.pal.accent2, 0.9), 1.0))));
        }
        let gb = |n: f64| if n >= 1e10 { format!("{:.0}", n / 1e9) } else { format!("{:.1}", n / 1e9) };
        let mut meters: Vec<gpui::Div> = Vec::new();
        let ram = num(s, &["ram"]).unwrap_or(0.0) as f32;
        meters.push(self.meter("ram", ram, match (num(s, &["ramUsed"]), num(s, &["ramTotal"])) { (Some(u), Some(t)) => format!("{}/{}G", gb(u), gb(t)), _ => format!("{ram:.0}%") }, None));
        if let Some(g) = num(s, &["gpu"]) {
            meters.push(self.meter("gpu", g as f32, format!("{g:.0}%"), None));
        }
        if let Some(t) = num(s, &["temp"]) {
            meters.push(self.meter("temp", (t as f32).min(100.0), format!("{t:.0}°C"), if t >= 85.0 { Some(self.pal.crit) } else if t >= 75.0 { Some(self.pal.warn) } else { None }));
        }
        let disk = num(s, &["disk"]).unwrap_or(0.0) as f32;
        meters.push(self.meter("disk", disk, format!("{disk:.0}%"), if disk >= 90.0 { Some(self.pal.warn) } else { None }));
        let bat = get(s, &["bat"]);
        if let Some(pct) = num(bat, &["pct"]) {
            meters.push(self.meter("bat", pct as f32, format!("{pct:.0}%{}", if text(bat, &["status"]) == "Charging" { " ⚡" } else { "" }), if pct < 15.0 { Some(self.pal.crit) } else { None }));
        }
        let n = meters.len() as f32;
        let body = div().flex().flex_col().child(head).child(self.spark(vec![(hist, self.pal.accent)], 100.0, 90, 3.0)).child(cores).children(meters);
        Panel { id: "sys".into(), el: self.panel("system", "󰍛", "", "", "", body), h: 2.25 + 3.1 + 3.5 + 3.3 + 1.7 * n }
    }

    /// pulse.js netPanel().
    pub fn net(&self, st: &Value) -> Panel {
        let s = get(st, &["sys"]);
        let rx: Vec<f32> = list(st, &["hist", "rx"]).iter().filter_map(|x| x.as_f64().map(|v| v as f32)).collect();
        let tx: Vec<f32> = list(st, &["hist", "tx"]).iter().filter_map(|x| x.as_f64().map(|v| v as f32)).collect();
        let ip = { let i = text(s, &["ip"]); if self.private { String::new() } else if i.is_empty() { "offline".into() } else { i } };
        let body = div().flex().flex_col()
            .child(self.row("↓ down", bytes(num(s, &["rx"]).unwrap_or(0.0), true)))
            .child(self.row("↑ up", bytes(num(s, &["tx"]).unwrap_or(0.0), true)))
            .child(self.spark(vec![(rx, self.pal.accent), (tx, self.pal.accent2)], 0.0, 90, 3.0))
            .child(self.row(format!("{}@{}", text(s, &["user"]), text(s, &["host"])), ip));
        Panel { id: "net".into(), el: self.panel("network", "󰛳", "", "", "", body), h: 2.25 + 3.57 + 3.5 + 1.785 }
    }

    /// pulse.js weatherPanel(), with its drawn icon (wxSvg) and the air (airRow).
    pub fn weather(&self, st: &Value) -> Panel {
        let (r, p) = (self.r, self.pal);
        let w = get(st, &["weather"]);
        let city = text(w, &["city"]);
        let title = if city.is_empty() { "weather".to_string() } else { format!("weather · {city}") };
        let err = text(w, &["error"]);
        if !err.is_empty() {
            return Panel { id: "weather".into(), el: self.panel(&title, "󰖐", "", "", "", self.note(&err)), h: 2.25 + 1.43 };
        }
        let hours: Vec<&Value> = list(w, &["hours"]).iter().take(12).collect();
        let (lo, hi) = (num(w, &["lo"]).unwrap_or(0.0), num(w, &["hi"]).unwrap_or(0.0));
        let tmin = hours.iter().filter_map(|x| num(x, &["t"])).fold(lo, f64::min);
        let tmax = hours.iter().filter_map(|x| num(x, &["t"])).fold(hi, f64::max);
        let code = num(w, &["code"]).unwrap_or(0.0) as i32;
        let day = num(w, &["day"]).map(|d| d > 0.0).or_else(|| get(w, &["day"]).as_bool()).unwrap_or(true);
        let wxrow = |items: Vec<gpui::Div>| div().mt(px(0.7 * r)).flex().justify_between().text_size(px(0.85 * r)).line_height(px(1.02 * r)).text_color(self.fg(0.6)).children(items);
        let s = |t: String| div().whitespace_nowrap().child(t);
        let mut bars = div().mt(px(0.9 * r)).h(px(3.6 * r)).flex().items_end().gap(px(3.0));
        let mut rain_max = 0.0f64;
        for x in &hours {
            let t = num(x, &["t"]).unwrap_or(0.0);
            let pr = num(x, &["p"]).unwrap_or(0.0);
            rain_max = rain_max.max(pr);
            let hh = (12.0 + 78.0 * (t - tmin) / (tmax - tmin).max(1.0)) as f32 / 100.0;
            let c = if pr >= 40.0 { col(p.accent2, (0.35 + pr / 160.0) as f32) } else { col(p.accent, 0.55) };
            let label = x.get("h").map(|v| v.as_str().map(String::from).unwrap_or_else(|| v.to_string())).unwrap_or_default();
            bars = bars.child(div().flex_1().h_full().flex().flex_col().justify_end().items_center()
                .child(div().w_full().h(relative(hh * 0.78)).min_h(px(2.0)).bg(c))
                .child(div().mt(px(0.2 * r)).text_size(px(0.62 * r)).line_height(px(0.75 * r)).text_color(col(p.dim, 1.0)).child(label)));
        }
        let mut body = div().flex().flex_col()
            .child(div().flex().items_center().gap(px(1.0 * r))
                .child(wx_icon(code, day, p, 5.4 * r))
                .child(div().flex().flex_col()
                    .child(self.big(format!("{:.0}", num(w, &["temp"]).unwrap_or(0.0)), &text(w, &["unit"]), 3.1))
                    .child(div().mt(px(0.2 * r)).text_size(px(0.95 * r)).text_color(col(p.accent, 1.0)).child(spaced(&crate::weather_word(code).to_uppercase(), 0.1, 0.95 * r)))))
            .child(wxrow(vec![
                s(format!("feels {:.0}°", num(w, &["feels"]).unwrap_or(0.0))),
                s(format!("↑{:.0}° ↓{:.0}°", hi, lo)),
                s(format!("{:.0} {}", num(w, &["wind"]).unwrap_or(0.0), text(w, &["windUnit"]))),
                s(format!("{:.0}%", num(w, &["hum"]).unwrap_or(0.0))),
            ]))
            .child(bars);
        let mut h = 2.25 + 5.4 + 1.72 + 4.5 + 1.72;
        let air = get(st, &["air"]);
        if let Some(aqi) = num(air, &["aqi"]) {
            let level = text(air, &["level"]);
            let c = match level.as_str() { "good" | "fair" => p.ok, "moderate" => p.warn, _ => p.crit };
            let pollen = text(air, &["pollen"]);
            body = body.child(wxrow(vec![
                div().flex().whitespace_nowrap().child("air ").child(div().font_weight(FontWeight::BOLD).text_color(col(c, 1.0)).child(level)).child(format!(" · {aqi:.0}")),
                s(if pollen.is_empty() { "no pollen".into() } else { format!("{pollen} pollen {}", text(air, &["pollenLevel"])) }),
            ]));
            h += 1.72;
        }
        let stale = get(w, &["stale"]).as_bool().unwrap_or(false);
        body = body.child(wxrow(vec![
            s(format!("☀ {}", text(w, &["sunrise"]))),
            s(if stale { "offline".into() } else { format!("rain {rain_max:.0}% next 12h") }),
            s(format!("☾ {}", text(w, &["sunset"]))),
        ]));
        Panel { id: "weather".into(), el: self.panel(&title, "󰖐", "", "", "", body), h }
    }

    // ------------------------------------------------------------------ a panel of yours: pulse.js generic()

    /// `typed`: how many characters of a text panel are shown (the web page types them out); `cam`: its frame.
    pub fn generic(&self, wd: &Value, slot: &str, typed: Option<usize>, cam: Option<std::sync::Arc<gpui::Image>>) -> Panel {
        let (r, p) = (self.r, self.pal);
        let s = |k: &str| text(wd, &[k]);
        let id = s("id");
        let title = if s("title").is_empty() { id.clone() } else { s("title") };
        let d = get(wd, &["data"]);
        let unit = { let u = text(d, &["unit"]); if u.is_empty() { s("unit_override") } else { u } };
        let note = text(d, &["note"]);
        let t = s("type");
        let width_rem: f32 = if slot == "center" { 43.6 } else { 22.6 };
        let mut h = 2.25;
        let body: gpui::Div = match t.as_str() {
            "stat" => {
                let hist: Vec<f32> = list(d, &["history"]).iter().filter_map(|x| x.as_f64().map(|v| v as f32)).collect();
                let mut b = div().flex().flex_col().child(self.big(num_s(get(d, &["value"])), &unit, 3.1));
                h += 3.1;
                if !note.is_empty() { b = b.child(self.note(&note)); h += 1.43; }
                if hist.len() > 2 { b = b.child(self.spark(vec![(hist, p.accent)], 0.0, 90, 3.0)); h += 3.5; }
                b
            }
            "gauge" => {
                let (max, min) = (num(d, &["max"]).filter(|v| *v != 0.0).unwrap_or(100.0), num(d, &["min"]).unwrap_or(0.0));
                let pct = (100.0 * (num(d, &["value"]).unwrap_or(0.0) - min) / if max - min == 0.0 { 1.0 } else { max - min }).clamp(0.0, 100.0) as f32;
                let mut b = div().flex().flex_col().child(self.big(num_s(get(d, &["value"])), &unit, 2.2)).child(div().mt(px(0.25 * r)).mb(px(0.2 * r)).child(self.seg(pct, None, 0.8)));
                h += 2.2 + 1.25;
                if !note.is_empty() { b = b.child(self.note(&note)); h += 1.43; }
                b
            }
            "list" => {
                let items: Vec<&Value> = list(d, &["items"]).iter().take(if slot == "center" { 4 } else { 6 }).collect();
                let n = items.len();
                let mut ul = div().flex().flex_col();
                for (i, it) in items.into_iter().enumerate() {
                    let right = { let rr = text(it, &["right"]); if !rr.is_empty() { rr } else { let a = ago(num(it, &["t"]).unwrap_or(0.0), self.now); if !a.is_empty() { a } else { num(it, &["value"]).map(|v| format!("{}{}", num_s(&serde_json::json!(v)), text(it, &["unit"]))).unwrap_or_default() } } };
                    let sub = text(it, &["text"]);
                    h += 0.64 + 1.4 + if sub.is_empty() { 0.0 } else { 1.02 };
                    let mut li = div().flex().gap(px(0.55 * r)).py(px(0.32 * r))
                        .child(div().w(px(0.9 * r)).flex_none().pt(px(0.5 * r)).child(self.dot(&text(it, &["level"]), 0.45)))
                        .child(div().flex_1().min_w(px(0.0)).flex().flex_col()
                            .child(div().text_size(px(1.0 * r)).line_height(px(1.4 * r)).text_color(self.fg(1.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(text(it, &["title"])))
                            .child(if sub.is_empty() { div() } else { div().text_size(px(0.85 * r)).line_height(px(1.02 * r)).text_color(self.fg(0.55)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(sub) }))
                        .child(div().flex_none().pt(px(0.15 * r)).text_size(px(0.82 * r)).text_color(col(p.dim, 1.0)).whitespace_nowrap().child(right));
                    if i + 1 < n {
                        li = li.border_b_1().border_dashed().border_color(col(p.dim, 0.35));
                    }
                    ul = ul.child(li);
                }
                if !note.is_empty() { ul = ul.child(self.note(&note)); h += 1.43; }
                ul
            }
            "status" => {
                let all = list(d, &["items"]);
                let wide = all.iter().any(|it| text(it, &["title"]).chars().count() > 15);
                let per_row = if wide { 1 } else { ((width_rem + 0.5f32) / 11.0).floor().max(1.0) as usize };
                let shown: Vec<&Value> = all.iter().take(if wide { 6 } else { 12 }).collect();
                let rows = shown.len().div_ceil(per_row);
                h += rows as f32 * 2.04 + (rows.saturating_sub(1)) as f32 * if wide { 0.35 } else { 0.5 };
                let mut grid = div().flex().flex_col().gap(px(if wide { 0.35 } else { 0.5 } * r));
                for chunk in shown.chunks(per_row) {
                    let mut line = div().flex().gap(px(0.5 * r));
                    for it in chunk {
                        let lv = text(it, &["level"]);
                        let c = self.level_c(&lv).filter(|_| lv != "info").unwrap_or(p.dim);
                        line = line.child(div().flex_1().min_w(px(0.0)).flex().items_center().gap(px(0.55 * r)).py(px(0.45 * r)).px(px(0.6 * r))
                            .bg(if lv == "crit" { col(p.crit, 0.12) } else { col(p.dim, 0.14) }).border_l_2().border_color(col(c, 1.0))
                            .child(self.dot(&lv, 0.5))
                            .child(div().min_w(px(0.0)).text_size(px(0.95 * r)).line_height(px(1.14 * r)).text_color(self.fg(1.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(text(it, &["title"])))
                            .child(div().ml_auto().flex_none().text_size(px(0.78 * r)).text_color(self.fg(0.5)).whitespace_nowrap().child(text(it, &["note"]))));
                    }
                    for _ in chunk.len()..per_row {
                        line = line.child(div().flex_1());
                    }
                    grid = grid.child(line);
                }
                grid
            }
            "kv" => {
                let rows: Vec<&Value> = list(d, &["rows"]).iter().take(8).collect();
                h += rows.len() as f32 * 1.617;
                let cell = |v: Option<&Value>| v.map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string())).unwrap_or_default();
                div().flex().flex_col().children(rows.into_iter().map(|row| {
                    let a = row.as_array().cloned().unwrap_or_default();
                    div().flex().justify_between().text_size(px(0.98 * r)).line_height(px(1.617 * r))
                        .child(div().pr(px(0.8 * r)).text_color(self.fg(0.6)).whitespace_nowrap().child(cell(a.first())))
                        .child(div().max_w(px(14.0 * r)).text_color(self.fg(1.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(cell(a.get(1))))
                }))
            }
            "image" => {
                let ch = width_rem * 9.0 / 16.0;
                h += ch;
                let frame = num(d, &["frame"]).map(|f| { let s2 = (f as i64 + crate::local_offset_secs()).rem_euclid(86400); format!("{:02}:{:02}:{:02}", s2 / 3600, (s2 / 60) % 60, s2 % 60) }).unwrap_or_default();
                let live = get(d, &["live"]).as_bool().unwrap_or(false);
                let mut cam_el = div().relative().w_full().h(px(ch * r)).bg(gpui::black()).overflow_hidden();
                if let (Some(im), false) = (cam, self.private) {
                    cam_el = cam_el.child(gpui::img(im).absolute().size_full().object_fit(gpui::ObjectFit::Cover));
                }
                cam_el
                    .child(div().absolute().left(px(0.6 * r)).top(px(0.4 * r)).flex().text_size(px(0.8 * r)).text_color(gpui::white()).child(div().mr(px(0.35 * r)).text_color(col(p.crit, 1.0)).child("●")).child(format!("{}{title}", if live { "LIVE · " } else { "" })))
                    .child(div().absolute().right(px(0.6 * r)).bottom(px(0.4 * r)).text_size(px(0.78 * r)).text_color(gpui::white()).child(frame))
            }
            "media" => {
                h += 3.4;
                let playing = text(d, &["status"]) == "Playing";
                let (pos, len) = (num(d, &["pos"]).unwrap_or(0.0), num(d, &["len"]).unwrap_or(0.0));
                let mut eq = div().flex().items_end().gap(px(3.0)).h(px(1.2 * r));
                for i in 0..5 {
                    let k = if playing { 0.15 + 0.85 * ((self.now * 3.1 + i as f64 * 1.7).sin().abs() as f32) } else { 0.0 };
                    eq = eq.child(if playing { div().w(px(3.0)).h(relative(k)) } else { div().w(px(3.0)).h(px(2.0)) }.bg(col(p.accent, 1.0)));
                }
                let artist = { let a = text(d, &["artist"]); if a.is_empty() { text(d, &["player"]) } else { a } };
                div().flex().flex_col()
                    .child(div().flex().justify_between().gap(px(0.6 * r))
                        .child(div().min_w(px(0.0)).text_size(px(1.15 * r)).text_color(self.fg(1.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(text(d, &["title"])))
                        .child(eq))
                    .child(div().mt(px(0.15 * r)).text_size(px(0.92 * r)).text_color(self.fg(0.6)).child(artist))
                    .child(div().mt(px(0.7 * r)).h(px(2.0)).w_full().bg(col(p.dim, 0.4)).child(div().h_full().w(relative(if len > 0.0 { (pos / len).clamp(0.0, 1.0) as f32 } else { 0.0 })).bg(col(p.accent2, 1.0))))
            }
            "chart" => {
                h += 5.5;
                let colors = [p.accent, p.accent2, p.ok, p.warn];
                let series: Vec<(Vec<f32>, C)> = list(d, &["series"]).iter().enumerate().map(|(i, sr)| {
                    let vals = sr.get("values").or_else(|| sr.get("v")).unwrap_or(sr);
                    (vals.as_array().map(|a| a.iter().filter_map(|x| x.as_f64().map(|v| v as f32)).collect()).unwrap_or_default(), colors[i % 4])
                }).collect();
                self.spark(series, 0.0, 0, 5.0)
            }
            _ => {
                // text, typed out when it changes, with its block cursor
                let full = text(d, &["text"]);
                let shown: String = full.chars().take(typed.unwrap_or(usize::MAX)).collect();
                let per_line = (width_rem / (0.6 * 1.05)).max(10.0) as usize;
                let lines: usize = full.split('\n').map(|l| l.chars().count().div_ceil(per_line).max(1)).sum();
                h += (lines as f32 * 1.68).min(8.6);
                let blink = (self.now as i64) % 2 == 0;
                div().max_h(px(8.6 * r)).overflow_hidden().text_size(px(1.05 * r)).line_height(px(1.68 * r)).text_color(self.fg(1.0))
                    .child(div().flex().flex_wrap().child(shown).child(div().w(px(0.55 * 1.05 * r)).h(px(1.05 * 1.05 * r)).mt(px(0.35 * r)).bg(col(p.accent, if blink { 1.0 } else { 0.3 }))))
            }
        };
        let corner = if t == "image" { String::new() } else { let a = ago(num(wd, &["updated"]).unwrap_or(0.0), self.now); if a == "now" { String::new() } else { a } };
        let err = s("error");
        if !err.is_empty() {
            h += 0.96;
        }
        let private_panel = get(wd, &["private"]).as_bool().unwrap_or(false) && self.private;
        let body = if private_panel {
            div().relative().child(body.opacity(0.0)).child(div().absolute().size_full().flex().items_center().justify_center().bg(col(p.bg, 0.9)).text_size(px(0.9 * r)).text_color(col(p.dim, 1.0)).child(spaced("PRIVATE", 0.3, 0.9 * r)))
        } else {
            body
        };
        let icon = s("icon");
        Panel { id, el: self.panel(&title, &icon, &s("level"), &corner, &err, body), h }
    }

    // ------------------------------------------------------------------ the clock and the ticker

    /// pulse.js clock(), as #clock in pulse.css: the time, its seconds, the bar of the minute, the date, who you are.
    pub fn clock(&self, st: &Value, hm: &str, sec: &str, sec_frac: f32, date: &str) -> gpui::Div {
        let (r, p) = (self.r, self.pal);
        let focus = get(st, &["focus"]);
        let mut bar = sec_frac;
        let who: gpui::Div = if focus.is_object() {
            let (a, b) = (num(focus, &["start"]).unwrap_or(self.now), num(focus, &["until"]).unwrap_or(self.now));
            let left = (b - self.now).max(0.0);
            bar = (1.0 - left / (b - a).max(1.0)) as f32;
            div().flex().child(div().text_color(col(p.accent2, 1.0)).child("FOCUS")).child(format!(" · {} · {}:{:02} left", text(focus, &["label"]), (left / 60.0) as i64, (left % 60.0) as i64))
        } else {
            let s = get(st, &["sys"]);
            let theme = { let n = text(st, &["theme", "name"]); if n.is_empty() { "omarchy".to_string() } else { n } };
            let mut wl = div().flex().child(format!("{}@{} · ", text(s, &["user"]), text(s, &["host"]))).child(div().text_color(col(p.accent2, 1.0)).child(theme)).child(format!(" · up {}", upt(num(s, &["uptime"]).unwrap_or(0.0))));
            if let Some(hb) = num(st, &["heart"]).filter(|v| *v > 0.0) {
                wl = wl.child(" · ").child(div().text_color(col(p.crit, 1.0)).child(format!("♥ {hb:.0}")));
            }
            wl
        };
        let title = text(st, &["ui", "title"]);
        div().flex().flex_col().items_center()
            .child(div().relative().flex().items_start()
                .child(div().text_size(px(9.6 * r)).line_height(px(8.64 * r)).font_weight(FontWeight::EXTRA_LIGHT).text_color(self.fg(1.0)).child(hm.to_string()))
                .child(div().ml(px(0.6 * r)).mt(px(-0.2 * r)).text_size(px(2.2 * r)).font_weight(FontWeight::LIGHT).text_color(col(p.accent, 1.0)).child(sec.to_string())))
            .child(div().mt(px(1.0 * r)).mb(px(0.9 * r)).w(px(22.0 * r)).h(px(2.0)).bg(col(p.dim, 0.35))
                .child(div().h_full().w(relative(bar.clamp(0.0, 1.0))).bg(gpui::linear_gradient(90.0, gpui::linear_color_stop(col(p.accent2, 1.0), 0.0), gpui::linear_color_stop(col(p.accent, 1.0), 1.0)))))
            .child(div().text_size(px(1.25 * r)).line_height(px(1.5 * r)).text_color(col(p.accent, 1.0)).child(spaced(date, 0.42, 1.25 * r)))
            .child(div().mt(px(0.55 * r)).text_size(px(1.0 * r)).line_height(px(1.25 * r)).text_color(self.fg(0.6)).child(who))
            .child(if title.is_empty() { div() } else { div().mt(px(0.5 * r)).text_size(px(0.95 * r)).text_color(col(p.accent2, 1.0)).child(spaced(&title, 0.6, 0.95 * r)) })
    }

    /// #ticker: one headline at a time, every six seconds, with its count on the right.
    pub fn ticker(&self, w: f32, item: Option<(String, String)>, count: &str) -> gpui::Div {
        let (r, p) = (self.r, self.pal);
        let mut bar = div().absolute().left(px(0.0)).bottom(px(0.0)).w(px(w)).h(px(2.2 * r)).border_t_1().border_color(col(p.accent, 0.25))
            .bg(gpui::linear_gradient(0.0, gpui::linear_color_stop(col(p.bg, 0.92), 0.0), gpui::linear_color_stop(col(p.bg, 0.6), 1.0)))
            .text_size(px(0.95 * r)).line_height(px(2.2 * r)).whitespace_nowrap().overflow_hidden().flex().justify_center();
        if let Some((t, l)) = item {
            let c = match l.as_str() { "warn" => col(p.warn, 1.0), "crit" => col(p.crit, 1.0), _ => self.fg(0.75) };
            let dc = match l.as_str() { "warn" => col(p.warn, 1.0), "crit" => col(p.crit, 1.0), _ => col(p.accent2, 1.0) };
            bar = bar.child(div().mx(px(2.2 * r)).flex().items_center().text_color(c).child(div().mr(px(0.8 * r)).text_size(px(0.7 * r)).text_color(dc).child("◆")).child(t));
        }
        bar.child(div().absolute().right(px(1.6 * r)).top(px(0.0)).text_size(px(0.8 * r)).text_color(col(p.dim, 1.0)).child(count.to_string()))
    }
}

/// pulse.css letter-spacing, on a monospace font: each character in a cell a little wider than itself.
pub fn spaced(t: &str, em: f32, size: f32) -> gpui::Div {
    let mut d = div().flex();
    for ch in t.chars() {
        d = d.child(div().w(px(size * (0.5 + em))).flex_none().child(ch.to_string())); // the font's advance is .5em here, as measured on the web page
    }
    d
}

/// pulse.js wxSvg(): the weather icon, drawn on an 84×84 grid as the web page draws it.
fn wx_icon(code: i32, day: bool, p: &Pal, size: f32) -> gpui::Div {
    let (a, b, f, bg, warn) = (p.accent, p.accent2, p.fg, p.bg, p.warn);
    let kind = wx_kind(code);
    div().w(px(size)).h(px(size)).flex_none().child(canvas(|_, _, _| (), move |bd, _, win, _| {
        let k = f32::from(bd.size.width) / 84.0;
        let (ox, oy) = (f32::from(bd.origin.x), f32::from(bd.origin.y));
        let pt = |x: f32, y: f32| point(px(ox + x * k), px(oy + y * k));
        let lw = px(2.5 * k);
        let stroke = |win: &mut gpui::Window, pts: &[(f32, f32)], w: gpui::Pixels, c: gpui::Rgba| {
            let mut pb = PathBuilder::stroke(w);
            pb.move_to(pt(pts[0].0, pts[0].1));
            for q in &pts[1..] {
                pb.line_to(pt(q.0, q.1));
            }
            if let Ok(path) = pb.build() {
                win.paint_path(path, c);
            }
        };
        let circle = |cx: f32, cy: f32, rr: f32, a0: f32, a1: f32| -> Vec<(f32, f32)> { (0..=40).map(|i| { let t = a0 + (a1 - a0) * i as f32 / 40.0; (cx + t.cos() * rr, cy + t.sin() * rr) }).collect() };
        let sun = matches!(kind, "clear" | "part");
        if sun {
            if day {
                stroke(win, &circle(34.0, 30.0, 11.0, 0.0, std::f32::consts::TAU), lw, col(b, 1.0));
                for d in 0..8 {
                    let rr = d as f32 * std::f32::consts::FRAC_PI_4;
                    stroke(win, &[(34.0 + rr.cos() * 15.0, 30.0 + rr.sin() * 15.0), (34.0 + rr.cos() * 19.0, 30.0 + rr.sin() * 19.0)], lw, col(b, 1.0));
                }
            } else {
                // the moon: a crescent
                let mut m = circle(36.0, 31.0, 13.0, -1.9, 2.6);
                m.extend(circle(42.0, 28.0, 11.0, 2.4, -1.6));
                stroke(win, &m, lw, col(b, 1.0));
            }
        }
        if kind != "clear" {
            // the cloud: M22 58 h34 a11 11 0 0 0 0-22 a15 15 0 0 0-28-4 a10 10 0 0 0-6 26z
            let mut c: Vec<(f32, f32)> = vec![(22.0, 58.0), (56.0, 58.0)];
            c.extend(circle(56.0, 47.0, 11.0, std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2));
            c.extend(circle(41.29, 38.95, 15.0, -0.198, -2.66));
            c.extend(circle(25.0, 45.0, 13.34, -1.345, -4.486));
            c.push((22.0, 58.0));
            let mut fill = PathBuilder::fill();
            fill.move_to(pt(c[0].0, c[0].1));
            for q in &c[1..] {
                fill.line_to(pt(q.0, q.1));
            }
            fill.close();
            if let Ok(path) = fill.build() {
                win.paint_path(path, col(bg, 0.9));
            }
            stroke(win, &c, lw, col(a, 1.0));
        }
        let drops = |win: &mut gpui::Window, n: usize| {
            for i in 0..n {
                let x = 28.0 + i as f32 * 9.0;
                stroke(win, &[(x, 64.0), (x - 4.0, 74.0)], lw, col(a, 1.0));
            }
        };
        match kind {
            "fog" => {
                stroke(win, &[(18.0, 66.0), (66.0, 66.0)], px(2.0 * k), col(f, 0.6));
                stroke(win, &[(24.0, 73.0), (60.0, 73.0)], px(2.0 * k), col(f, 0.4));
            }
            "rain" => drops(win, 4),
            "snow" => {
                for i in 0..4 {
                    let x = 28.0 + i as f32 * 9.0;
                    for ang in [0.0f32, 1.047, 2.094] {
                        stroke(win, &[(x - ang.cos() * 3.0, 71.0 - ang.sin() * 3.0), (x + ang.cos() * 3.0, 71.0 + ang.sin() * 3.0)], px(1.5 * k), col(f, 1.0));
                    }
                }
            }
            "storm" => {
                stroke(win, &[(44.0, 60.0), (37.0, 70.0), (44.0, 70.0), (39.0, 79.0)], px(2.8 * k), col(warn, 1.0));
                drops(win, 2);
            }
            _ => {}
        }
    }).size_full())
}

pub fn upt(s: f64) -> String {
    let (d, h, m) = ((s / 86400.0) as i64, ((s % 86400.0) / 3600.0) as i64, ((s % 3600.0) / 60.0) as i64);
    format!("{}{}{m}m", if d > 0 { format!("{d}d ") } else { String::new() }, if d > 0 || h > 0 { format!("{h}h ") } else { String::new() })
}

/// pulse.js bytes(): 1000-based, one decimal under 100.
pub fn bytes(n: f64, rate: bool) -> String {
    let u: &[&str] = if rate { &["B/s", "KB/s", "MB/s", "GB/s"] } else { &["B", "KB", "MB", "GB", "TB"] };
    let (mut n, mut i) = (n.max(0.0), 0);
    while n >= 1000.0 && i < u.len() - 1 {
        n /= 1000.0;
        i += 1;
    }
    if n >= 100.0 || i == 0 { format!("{n:.0} {}", u[i]) } else { format!("{n:.1} {}", u[i]) }
}

/// pulse.js ago().
pub fn ago(t: f64, now: f64) -> String {
    if t <= 0.0 {
        return String::new();
    }
    let s = (now - t).max(0.0);
    if s < 60.0 { "now".into() } else if s < 3600.0 { format!("{}m", (s / 60.0) as i64) } else if s < 86400.0 { format!("{}h", (s / 3600.0) as i64) } else { format!("{}d", (s / 86400.0) as i64) }
}

/// pulse.js num().
pub fn num_s(v: &Value) -> String {
    let Some(x) = v.as_f64() else { return v.as_str().map(String::from).unwrap_or_default() };
    let a = x.abs();
    if a >= 10000.0 {
        let s = format!("{:.0}", x);
        let (sign, digits) = if let Some(d) = s.strip_prefix('-') { ("-", d.to_string()) } else { ("", s) };
        let mut out = String::new();
        for (i, ch) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i) % 3 == 0 {
                out.push(',');
            }
            out.push(ch);
        }
        format!("{sign}{out}")
    } else if a >= 100.0 {
        format!("{x:.0}")
    } else if a >= 10.0 {
        format!("{x:.1}")
    } else {
        let s = format!("{x:.2}");
        let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        if s.is_empty() || s == "-" { "0".into() } else { s }
    }
}

/// pulse.js fit(): in a column `avail` rem tall, the panels that fit, from the top; the first that does not fit and
/// every one after it are hidden.
pub fn fit(panels: Vec<Panel>, avail: f32, gap: f32) -> Vec<Panel> {
    let mut used = 0.0;
    let mut out = Vec::new();
    for p in panels {
        let ph = p.h + 0.5; // the .err line
        let need = if out.is_empty() { ph } else { used + gap + ph };
        if need > avail + 0.05 {
            break;
        }
        used = need;
        out.push(p);
    }
    out
}


/// pulse.js wxKind().
pub fn wx_kind(c: i32) -> &'static str {
    if c >= 95 { "storm" } else if (71..=77).contains(&c) || c == 85 || c == 86 { "snow" } else if c >= 51 { "rain" } else if c == 45 || c == 48 { "fog" } else if c == 3 { "cloud" } else if c == 1 || c == 2 { "part" } else { "clear" }
}
