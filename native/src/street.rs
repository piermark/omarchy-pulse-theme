//! The life of the city in Pulse 1.8, translated from pulse.js: the calendar's trains (drawTrains), your containers
//! and agents as robots walking on the floor (drawRobots), the candles of what expires (drawCandles), your datacenter
//! as a district of halls (drawDatacenter), the small plane towing a banner (drawBanners) and the hot-air balloon with
//! an Omarchy tip or the news (drawBalloons).

use crate::draw;
use crate::hub::{get, list, num, text};
use crate::palette::{mix, Pal, C};
use crate::scene::{radial, rect};
use crate::xf::{Path, Xf};
use gpui::Window;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

const TAU: f32 = std::f32::consts::TAU;

struct Walker {
    x: f32,
    lane: f32,
    dir: f32,
    ph: f32,
}

struct Flight {
    t: f64,
    text: String,
    dir: f32,
    warn: bool,
}

#[derive(Default)]
pub struct Street {
    seed: u64,
    walkers: HashMap<String, Walker>,
    banners: Vec<Flight>,
    banner_next: f64,
    balloons: Vec<Flight>,
    tip_next: f64,
    news_seen: String,
    dc_prev: Option<HashMap<String, String>>,
    dc_flash: HashMap<String, (f64, bool, bool)>,
    dc_zap: f64,
    logo: Option<(C, Arc<gpui::RenderImage>)>,
    /// what the street asks of the scene: a bolt on a hall that went down (x, top), a few sparks where one came back
    pub zaps: Vec<(f32, f32, String)>,
    pub sparks: Vec<(f32, f32)>,
}

/// One line of text centred on (x, y) as textBaseline = "middle".
#[allow(clippy::too_many_arguments)]
fn mid_text(win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, s: &str, x: f32, y: f32, size: f32, weight: f32, c: C, a: f32) {
    draw::text(win, cx, font, s, x, y + size * 0.36, size, weight, c, a, 0);
}

impl Street {
    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.seed >> 33) % 1_000_000) as f32 / 1_000_000.0
    }

    /// The hub's effects: "banner" (a plane tows your words) and "tip" (a balloon).
    pub fn fx(&mut self, kind: &str, words: &str, t: f64, tips: &[Value]) {
        match kind {
            "banner" if !words.is_empty() => self.banners.push(Flight { t, text: words.to_uppercase(), dir: 1.0, warn: false }),
            "tip" => {
                let tx = if !words.is_empty() { words.to_string() } else { tips.first().and_then(Value::as_str).unwrap_or("SUPER + K · Show key bindings").to_string() };
                self.balloons.push(Flight { t, text: tx, dir: 1.0, warn: false });
            }
            _ => {}
        }
    }

    /// drawTrains(): your next meetings come into the station from the right as they come near.
    #[allow(clippy::too_many_arguments)]
    pub fn trains(&self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, h: f32, t: f32, band: (f32, f32), share: bool) {
        let trains = list(st, &["calendar"]);
        if trains.is_empty() {
            return;
        }
        let hz = oy + h * 0.745;
        let (b0, b1) = (ox + band.0, ox + band.1);
        let y = hz - h * 0.006;
        let stn = b0 + (b1 - b0) * 0.16;
        let now = num(st, &["t"]).unwrap_or(0.0);
        let span = 3.0 * 3600.0;
        draw::line(win, &[(b0, y), (b1, y)], (h / 600.0).max(1.0), pal.accent, 0.5);
        rect(win, stn - h * 0.03, y - h * 0.028, h * 0.06, h * 0.004, pal.accent, 0.8); // the station roof
        rect(win, stn - h * 0.03, y - h * 0.028, h * 0.003, h * 0.028, pal.accent, 0.8);
        rect(win, stn + h * 0.027, y - h * 0.028, h * 0.003, h * 0.028, pal.accent, 0.8);
        for e in trains {
            let (start, end) = (num(e, &["start"]).unwrap_or(0.0), num(e, &["end"]).unwrap_or(0.0));
            let dt = start - now;
            let inside = dt <= 0.0 && end > now;
            let x = if inside { stn } else { stn + ((dt / span).clamp(0.0, 1.0) as f32) * (b1 - stn) * if dt > 0.0 { 1.0 } else { 0.0 } };
            let soon = dt > 0.0 && dt < 300.0;
            let col = if inside { pal.ok } else if soon { pal.warn } else { pal.accent2 };
            let (cw, ch) = (h * 0.016, h * 0.009);
            for k in 0..4 {
                let cx2 = x + k as f32 * (cw + h * 0.002);
                rect(win, cx2, y - ch - h * 0.002, cw, ch, mix(pal.bg, col, 0.35), 1.0);
                for wv in 0..3 {
                    rect(win, cx2 + cw * (0.12 + wv as f32 * 0.3), y - ch, cw * 0.18, ch * 0.4, col, if soon { if (t * 3.0).rem_euclid(1.0) < 0.6 { 1.0 } else { 0.4 } } else { 0.9 });
                }
            }
            radial(win, x, y - ch / 2.0, 0.0, h * 0.025, 1.0, col, &[(0.0, 0.7), (1.0, 0.0)]);
            let title = text(e, &["title"]);
            if !share && !title.is_empty() {
                let mins = (dt / 60.0).round() as i64;
                let when = if inside {
                    "now".to_string()
                } else if mins < 60 {
                    format!("in {mins} min")
                } else {
                    let sod = (start as i64 + crate::local_offset_secs()).rem_euclid(86400);
                    format!("{:02}:{:02}", sod / 3600, (sod / 60) % 60)
                };
                draw::text(win, cx, font, &format!("{title} · {when}"), x, y - ch - h * 0.008, (h * 0.012).round(), 600.0, col, 0.95, -1);
            }
        }
    }

    /// drawRobots(): your containers walk on the floor (and your agents with agents_as = "robots" or "both"); the eyes
    /// say how they are: green, amber restarting, red unhealthy (it stops and blinks).
    #[allow(clippy::too_many_arguments)]
    pub fn robots(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, h: f32, t: f32, band: (f32, f32), frames: f32, share: bool) {
        let mut robots: Vec<Value> = list(st, &["robots"]).to_vec();
        if get(st, &["ui", "agentsAs"]).as_str().map(|a| a != "drones").unwrap_or(false) {
            let mut mine: Vec<Value> = list(st, &["agents"]).iter().filter(|a| text(a, &["state"]) != "gone").map(|a| {
                let s = text(a, &["state"]);
                serde_json::json!({ "id": format!("a{}", text(a, &["name"])), "name": text(a, &["name"]), "bot": true, "busy": s == "working",
                    "health": if s == "failed" { "bad" } else if s == "waiting" { "restart" } else { "ok" } })
            }).collect();
            mine.extend(robots);
            mine.truncate(12);
            robots = mine;
        }
        let (hz, u) = (oy + h * 0.745, h / 1080.0);
        let (b0, b1) = (ox + band.0, ox + band.1);
        let mut seen = Vec::new();
        for (i, rb) in robots.iter().take(12).enumerate() {
            let id = text(rb, &["id"]);
            seen.push(id.clone());
            if !self.walkers.contains_key(&id) {
                // a new container: a robot walks in
                let lane = (i64::from_str_radix(&id.chars().take(4).collect::<String>(), 16).ok().filter(|v| *v != 0).unwrap_or(i as i64 * 997) % 1000) as f32 / 1000.0;
                let (r1, r2, r3) = (self.rand(), self.rand(), self.rand());
                self.walkers.insert(id.clone(), Walker { x: b0 + (0.1 + 0.8 * r1) * (b1 - b0), lane, dir: if r2 < 0.5 { 1.0 } else { -1.0 }, ph: r3 * 6.0 });
            }
            let flip = self.rand() < 0.001 * frames;
            let p = self.walkers.get_mut(&id).expect("the walker");
            let y = hz + (oy + h - hz) * (0.18 + p.lane * 0.55);
            let sc = (0.7 + p.lane * 0.9) * u; // nearer is bigger
            let health = text(rb, &["health"]);
            let bad = health == "bad";
            let bot = get(rb, &["bot"]).as_bool().unwrap_or(false);
            if !bad {
                let fast = if get(rb, &["busy"]).as_bool().unwrap_or(false) { 2.2 } else { 1.0 }; // a bot at work hurries
                p.x += p.dir * sc * (0.55 + p.lane * 0.3) * fast * frames;
                p.ph += 0.16 * fast * frames;
            }
            if p.x > b1 - 20.0 { p.dir = -1.0 }
            if p.x < b0 + 20.0 { p.dir = 1.0 }
            if flip { p.dir = -p.dir }
            let sw = if bad { 0.0 } else { p.ph.sin() };
            let bob = if bad { 0.0 } else { p.ph.cos().abs() * 1.5 * sc };
            let x = p.x;
            let body = if bot { mix(pal.bg, pal.accent2, 0.6) } else { mix(pal.bg, pal.fg, 0.55) };
            let line = if bot { mix(pal.fg, pal.accent2, 0.4) } else { mix(pal.fg, pal.accent, 0.3) };
            let eye = if bad { pal.crit } else if health == "restart" { pal.warn } else { pal.ok };
            let xf = Xf::at(x, y - bob).scale(p.dir, 1.0);
            let lw = 2.2 * sc;
            // the legs and the arms, one shape
            let limbs = [((-2.0 * sc, -12.0 * sc), (-2.0 * sc + sw * 4.0 * sc, 0.0)), ((2.0 * sc, -12.0 * sc), (2.0 * sc - sw * 4.0 * sc, 0.0)),
                ((-4.5 * sc, -24.0 * sc), (-4.5 * sc - sw * 3.0 * sc, -15.0 * sc)), ((4.5 * sc, -24.0 * sc), (4.5 * sc + sw * 3.0 * sc, -15.0 * sc))];
            let limbs: Vec<_> = limbs.iter().map(|(a, b)| (xf.p(a.0, a.1), xf.p(b.0, b.1))).collect();
            draw::lines(win, &limbs, lw, line, 0.9 * xf.alpha);
            for (rx, ry, rw, rh) in [(-4.5, -26.0, 9.0, 14.0), (-4.0, -34.0, 8.0, 7.0)] {
                // the body, the head
                xf.rect(win, rx * sc, ry * sc, rw * sc, rh * sc, body, 0.95);
                xf.line(win, &[(rx * sc, ry * sc), ((rx + rw) * sc, ry * sc), ((rx + rw) * sc, (ry + rh) * sc), (rx * sc, (ry + rh) * sc), (rx * sc, ry * sc)], lw, line, 0.9);
            }
            xf.line(win, &[(0.0, -34.0 * sc), (0.0, -37.0 * sc)], lw, line, 0.9); // the antenna
            if bot {
                // an AI bot: a light on its antenna, its core on the chest
                let (a1, a2) = (xf.p(0.0, -38.0 * sc), xf.p(0.0, -20.0 * sc));
                draw::arc_fill(win, a1.0, a1.1, 1.6 * sc, 1.6 * sc, 0.0, 0.0, TAU, pal.accent, if (t * 2.0).rem_euclid(1.0) < 0.6 { 1.0 } else { 0.4 });
                draw::arc_fill(win, a2.0, a2.1, 1.8 * sc, 1.8 * sc, 0.0, 0.0, TAU, pal.accent, 0.9);
            }
            let blink = if bad { if (t * 3.0).rem_euclid(1.0) < 0.5 { 1.0 } else { 0.2 } } else { 1.0 };
            xf.rect(win, -2.6 * sc, -32.0 * sc, 2.0 * sc, 2.0 * sc, eye, blink);
            xf.rect(win, 0.8 * sc, -32.0 * sc, 2.0 * sc, 2.0 * sc, eye, blink);
            let ec = xf.p(1.0 * sc, -31.0 * sc);
            radial(win, ec.0, ec.1, 0.0, 8.0 * sc, 1.0, eye, &[(0.0, 0.35 * blink), (1.0, 0.0)]);
            draw::arc_fill(win, x, y + sc, 6.0 * sc, 1.6 * sc, 0.0, 0.0, TAU, pal.fg, 0.12); // its shadow
            let name = text(rb, &["name"]);
            if !share && !name.is_empty() {
                draw::text(win, cx, font, &name, x, y + 11.0 * sc, (10.0 * sc).max(9.0).round(), 600.0, eye, 0.75, 0);
            }
        }
        self.walkers.retain(|k, _| seen.contains(k));
    }

    /// drawCandles(): what expires (certificates, domains, licences) as candles, shorter as the day comes near.
    #[allow(clippy::too_many_arguments)]
    pub fn candles(&self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, still: bool, share: bool) {
        let ex = list(st, &["expiry"]);
        if ex.is_empty() {
            return;
        }
        let hz = h * 0.745;
        let (base, step, x0) = (oy + hz + (h - hz) * 0.66, w * 0.04, ox + w * 0.64);
        let fs = (h * 0.0105).round();
        for (i, e) in ex.iter().take(6).enumerate() {
            let x = x0 + i as f32 * step;
            let days = num(e, &["days"]);
            let d = days.unwrap_or(0.0).max(0.0) as f32;
            let (hh, cw) = (h * (0.01 + 0.05 * (d / 90.0).min(1.0)), h * 0.011);
            let lv = text(e, &["level"]);
            let wax = if lv == "crit" { pal.crit } else if lv == "warn" { pal.warn } else { mix(pal.fg, pal.bg, 0.25) };
            rect(win, x - cw / 2.0, base - hh, cw, hh, wax, 0.92);
            let fk = if still { 1.0 } else { 0.8 + 0.2 * (t * if lv == "crit" { 23.0 } else { 9.0 } + i as f32 * 3.0).sin() };
            let fy = base - hh - h * 0.008;
            radial(win, x, fy, 0.0, h * 0.03, 1.0, pal.warn, &[(0.0, 0.4 * fk), (1.0, 0.0)]);
            draw::arc_fill(win, x, fy, cw * 0.3, h * 0.008 * fk, 0.0, 0.0, TAU, mix(pal.warn, [1.0, 1.0, 220.0 / 255.0], 0.4), fk);
            let name = text(e, &["name"]);
            if !share && !name.is_empty() {
                let c = if lv == "ok" { pal.fg } else { wax };
                draw::text(win, cx, font, &name.chars().take(14).collect::<String>(), x, base + h * 0.016, fs, 400.0, c, 0.75, 0);
                let when = match days { None => "?".to_string(), Some(v) if v < 0.0 => "expired".to_string(), Some(v) => format!("{}d", v.floor()) };
                draw::text(win, cx, font, &when, x, base + h * 0.03, fs, 400.0, c, 0.75, 0);
            }
        }
    }

    /// drawDatacenter(): a hall for every row of your datacenter panels, its racks' lights in the colour of its state;
    /// when one goes down lightning strikes it, when it is back a few sparks.
    #[allow(clippy::too_many_arguments)]
    pub fn datacenter(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, now: f64, band: (f32, f32), night_k: f32, still: bool) {
        let d = get(st, &["datacenter"]);
        let groups = list(d, &["groups"]);
        // dcSync(): what changed since the last state
        let mut cur = HashMap::new();
        for gp in groups {
            for (i, r) in list(gp, &["rows"]).iter().enumerate() {
                let nm = text(r, &["name"]);
                cur.insert(format!("{}|{}", text(gp, &["id"]), if nm.is_empty() { i.to_string() } else { nm }), text(r, &["level"]));
            }
        }
        if let Some(prev) = &self.dc_prev {
            if get(d, &["alerts"]).as_bool() != Some(false) {
                for (k, v) in &cur {
                    match prev.get(k) {
                        Some(p) if p != v => {
                            if v == "crit" {
                                self.dc_flash.insert(k.clone(), (now, true, false));
                            } else if v == "ok" && (p == "crit" || p == "warn") {
                                self.dc_flash.insert(k.clone(), (now, false, false));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        self.dc_prev = Some(cur);
        let n: usize = groups.iter().map(|gp| list(gp, &["rows"]).len()).sum();
        if n == 0 {
            return;
        }
        let u = h / 1080.0;
        let hz = oy + h * 0.745;
        let recent = !list(st, &["growth", "recent"]).is_empty();
        let span = (band.1 - band.0) * if recent { 0.40 } else { 0.62 };
        let ggap = h * 0.022;
        let unit = (h * 0.034).min((span - ggap * (groups.len() as f32 - 1.0)) / n as f32);
        let bw = unit * 0.8;
        let y = hz + h * 0.062;
        let mut x = ox + w / 2.0 - (unit * n as f32 + ggap * (groups.len() as f32 - 1.0)) / 2.0;
        let labels = get(d, &["labels"]).as_bool() != Some(false);
        for gp in groups {
            let rows = list(gp, &["rows"]);
            let mx = rows.iter().filter_map(|r| num(r, &["value"])).fold(0.0f64, f64::max);
            let gx0 = x;
            let gid = text(gp, &["id"]);
            for (i, r) in rows.iter().enumerate() {
                let nm = text(r, &["name"]);
                let key = format!("{gid}|{}", if nm.is_empty() { i.to_string() } else { nm.clone() });
                let lv = text(r, &["level"]);
                let col = match lv.as_str() { "crit" => pal.crit, "warn" => pal.warn, "ok" => pal.ok, "info" => pal.accent2, _ => pal.accent };
                let v = match num(r, &["value"]) { Some(val) if mx > 0.0 => (val / mx).clamp(0.0, 1.0) as f32, _ => 0.55 };
                let (hh, dp, bx) = (h * (0.030 + 0.040 * v), unit * 0.38, x + (unit - bw) / 2.0);
                let down = lv == "crit";
                let blink = if down && !still { 0.55 + 0.45 * (t * 6.0).sin() } else { 1.0 };
                let mut fk = 0.0;
                if let Some(fl) = self.dc_flash.get(&key).copied() {
                    fk = (1.0 - (now - fl.0) as f32 / 5.0).max(0.0);
                    if fk <= 0.0 {
                        self.dc_flash.remove(&key);
                    }
                }
                // the hall: its front, its roof going back, its side
                let body = mix(mix(pal.bg, pal.accent, 0.26), col, if down { 0.28 } else { 0.10 });
                rect(win, bx, y - hh, bw, hh, mix(body, [0.0; 3], 0.25), 1.0);
                draw::poly(win, &[(bx, y - hh), (bx + dp * 0.5, y - hh - dp * 0.42), (bx + bw + dp * 0.5, y - hh - dp * 0.42), (bx + bw, y - hh)], mix(body, pal.fg, 0.10), 1.0);
                draw::poly(win, &[(bx + bw, y - hh), (bx + bw + dp * 0.5, y - hh - dp * 0.42), (bx + bw + dp * 0.5, y - dp * 0.42), (bx + bw, y)], mix(body, [0.0; 3], 0.5), 1.0);
                let cc = mix(body, pal.fg, 0.3); // the chillers on the roof
                rect(win, bx + bw * 0.22, y - hh - dp * 0.42 - u * 2.0, bw * 0.22, u * 3.0, cc, 0.9);
                rect(win, bx + bw * 0.6, y - hh - dp * 0.42 - u * 2.0, bw * 0.22, u * 3.0, cc, 0.9);
                // the racks' lights behind the glass, in the colour of its state; out when it is down
                let rws = (((hh - u * 6.0) / (u * 5.0)).floor() as i32).max(1);
                let cls = ((bw / (u * 5.5)).floor() as i32).max(2);
                for ry in 0..rws {
                    for c2 in 0..cls {
                        let hs = ((i as i32 * 131 + ry * 17 + c2 * 7 + gid.len() as i32 * 13) % 10) as f32 / 10.0;
                        let on = if down { hs < 0.25 } else { hs < 0.8 };
                        if on {
                            let tw = if still { 1.0 } else { 0.75 + 0.25 * (t * (1.3 + hs * 3.0) + hs * 40.0).sin() };
                            rect(win, bx + u * 2.0 + c2 as f32 * (bw - u * 3.2) / cls as f32, y - hh + u * 4.5 + ry as f32 * u * 5.0, (u * 2.4).max(1.5), (u * 2.4).max(1.5),
                                if hs > 0.55 && !down { mix(col, pal.fg, 0.35) } else { col }, (0.45 + 0.45 * night_k) * tw * blink);
                        }
                    }
                }
                rect(win, bx, y - hh, bw, (u * 2.2).max(1.5), col, 0.95 * blink); // the state, along the roof line
                rect(win, bx, y, bw, hh * 0.6, col, 0.10 + 0.10 * night_k); // and on the wet floor
                if down {
                    radial(win, bx + bw / 2.0, y - hh - dp * 0.5, 0.0, h * 0.03, 1.0, pal.crit, &[(0.0, 0.55 * blink), (1.0, 0.0)]);
                }
                if let Some(fl) = self.dc_flash.get_mut(&key) {
                    if !fl.2 {
                        fl.2 = true;
                        if fl.1 {
                            if now - self.dc_zap > 20.0 {
                                self.dc_zap = now;
                                self.zaps.push((bx + bw / 2.0, y - hh, format!("{} is down", if nm.is_empty() { "a service".to_string() } else { nm.clone() })));
                            }
                        } else if !still {
                            self.sparks.push((bx + bw / 2.0, y - hh - h * 0.02));
                        }
                    }
                }
                if fk > 0.0 {
                    let down_fl = self.dc_flash.get(&key).map(|f| f.1).unwrap_or(false);
                    draw::frame(win, bx - u * 2.0, y - hh - dp * 0.42 - u * 3.0, bw + dp * 0.5 + u * 4.0, hh + dp * 0.42 + u * 5.0, (u * 2.0).max(1.5), if down_fl { pal.crit } else { pal.ok }, fk);
                }
                if labels && !nm.is_empty() && (lv == "crit" || lv == "warn" || fk > 0.0) {
                    // a name only where you need to look
                    let tx: String = nm.chars().take(18).collect();
                    let fs = (h * 0.0105).round();
                    let twd = draw::measure(win, font, &tx, fs, 600.0) + u * 10.0;
                    rect(win, bx + bw / 2.0 - twd / 2.0, y - hh - dp * 0.42 - h * 0.026, twd, h * 0.017, pal.bg, 0.85);
                    mid_text(win, cx, font, &tx, bx + bw / 2.0, y - hh - dp * 0.42 - h * 0.0175, fs, 600.0, col, 1.0);
                }
                x += unit;
            }
            let title = text(gp, &["title"]);
            if labels && !title.is_empty() {
                draw::text(win, cx, font, &format!("{} · {}", title.to_lowercase(), rows.len()), (gx0 + x) / 2.0, y + h * 0.0155, (h * 0.0098).round(), 400.0, pal.fg, 0.5, 0);
            }
            x += ggap;
        }
    }

    fn logo(&mut self, win: &mut Window, c: C) -> Option<Arc<gpui::RenderImage>> {
        if let Some((k, i)) = &self.logo {
            if *k == c {
                return Some(i.clone());
            }
        }
        let img = tinted_logo(c)?;
        if let Some((_, old)) = self.logo.replace((c, img.clone())) {
            let _ = win.drop_image(old);
        }
        Some(img)
    }

    /// drawBanners(): a small plane tows the Omarchy icon and your words across the sky, now and then and when asked.
    #[allow(clippy::too_many_arguments)]
    pub fn banners(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, now: f64, primary: bool, desk_empty: bool) {
        let words = text(st, &["ui", "banner"]);
        let every = num(st, &["ui", "bannerEvery"]).unwrap_or(0.0);
        if !words.is_empty() && every > 0.0 && !get(st, &["away"]).as_bool().unwrap_or(false) && primary {
            if self.banner_next == 0.0 {
                self.banner_next = now + (1.0 + self.rand() as f64 * 2.0) * 60.0; // the first one a minute or two after start
            }
            if now > self.banner_next && desk_empty && self.banners.is_empty() {
                self.banners.push(Flight { t: now, text: words.to_uppercase(), dir: 1.0, warn: false });
                self.banner_next = now + every * 60.0 * (0.7 + self.rand() as f64 * 0.6);
            }
        }
        self.banners.retain(|b| (now - b.t) / 16.0 <= 1.0);
        let hz = h * 0.745;
        let lg = if self.banners.is_empty() { None } else { self.logo(win, pal.accent2) };
        for b in &self.banners {
            let u = ((now - b.t) / 16.0) as f32;
            let s = h / 1080.0;
            let x = ox + if b.dir > 0.0 { -w * 0.25 + u * w * 1.5 } else { w * 1.25 - u * w * 1.5 };
            let y = oy + hz * 0.07 + (u * 9.0).sin() * h * 0.006;
            let fs = (h * 0.036).round();
            let bh = h * 0.06;
            let lw = if lg.is_some() { bh * 0.8 + h * 0.016 } else { 0.0 };
            let tw = draw::measure(win, font, &b.text, fs, 800.0) + h * 0.03 + lw;
            let bx = x - b.dir * (h * 0.05) - if b.dir > 0.0 { tw } else { 0.0 };
            draw::line(win, &[(x - b.dir * h * 0.012, y), (if b.dir > 0.0 { bx + tw } else { bx }, y + bh * 0.1)], s.max(1.0), pal.fg, 0.6); // the tow line
            // the banner, waving in the wind
            let wave = |k: usize| ((now * 1000.0 / 180.0) as f32 + k as f32 * 0.6).sin() * bh * 0.08;
            let mut pts: Vec<(f32, f32)> = (0..=20).map(|k| (bx + tw * k as f32 / 20.0, y - bh * 0.4 + wave(k))).collect();
            pts.extend((0..=20).rev().map(|k| (bx + tw * k as f32 / 20.0, y + bh * 0.6 + wave(k))));
            draw::poly(win, &pts, mix(pal.bg, pal.fg, 0.92), 0.95);
            let mut ring = pts.clone();
            ring.push(pts[0]);
            draw::line(win, &ring, 1.0, pal.accent, 0.9);
            if let Some(img) = &lg {
                let at = gpui::Bounds::new(gpui::point(gpui::px(bx + h * 0.012), gpui::px(y - bh * 0.3)), gpui::size(gpui::px(bh * 0.8), gpui::px(bh * 0.8)));
                let _ = win.paint_image(at, at, gpui::Corners::default(), 0.0, img.clone(), 0, false);
            }
            mid_text(win, cx, font, &b.text, bx + lw + (tw - lw) / 2.0, y + bh * 0.1, fs, 800.0, pal.accent2, 1.0);
            // the little propeller plane
            let xf = Xf::at(x, y).scale(b.dir * 1.6, 1.6);
            let pc = mix(pal.fg, pal.accent, 0.3);
            xf.ellipse(win, 0.0, 0.0, 18.0 * s, 4.5 * s, pc, 1.0);
            xf.rect(win, -4.0 * s, -11.0 * s, 6.0 * s, 22.0 * s, pc, 1.0);
            xf.rect(win, -17.0 * s, -6.0 * s, 4.0 * s, 6.0 * s, pc, 1.0);
            let pr = ((now * 1000.0 / 30.0) as f32).sin().abs();
            xf.rect(win, 17.0 * s, -7.0 * s * pr, 1.5 * s, 14.0 * s * pr, pal.fg, 0.5);
            xf.rect(win, -1.0 * s, -11.0 * s, 2.0 * s, 2.0 * s, pal.crit, if (now * 2.0).rem_euclid(1.0) < 0.5 { 1.0 } else { 0.3 });
        }
    }

    /// drawBalloons(): an Omarchy tip on a hot-air balloon now and then, only when you can see the sky (never in a
    /// meeting, a focus or while you share the screen); the warning to read the news when Arch has some.
    #[allow(clippy::too_many_arguments)]
    pub fn balloons(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, now: f64, primary: bool, desk_empty: bool, wind_x: f32, still: bool) {
        let privacy = get(st, &["privacy"]).as_bool().unwrap_or(false);
        if let Some(n0) = list(st, &["updates", "news"]).first() {
            let k = n0.to_string();
            if k != self.news_seen {
                let first = self.news_seen.is_empty() && false;
                self.news_seen = k;
                if !privacy && primary && !first {
                    self.balloons.push(Flight { t: now, text: "read the news before you update".into(), dir: 1.0, warn: true });
                }
            }
        }
        let tips = list(st, &["tips"]);
        let every = num(st, &["ui", "tipsEvery"]).unwrap_or(40.0);
        if !tips.is_empty() && every > 0.0 && primary && !get(st, &["away"]).as_bool().unwrap_or(false) {
            if self.tip_next == 0.0 {
                self.tip_next = now + (2.0 + self.rand() as f64 * 3.0) * 60.0;
            }
            let busy = get(st, &["meeting"]).as_bool().unwrap_or(false) || get(st, &["focus"]).is_object() || privacy;
            if now > self.tip_next && desk_empty && self.balloons.is_empty() && !busy {
                let pick = ((self.rand() * tips.len() as f32) as usize).min(tips.len() - 1);
                let tx = tips[pick].as_str().unwrap_or("").to_string();
                self.balloons.push(Flight { t: now, text: tx, dir: if wind_x < -0.05 { -1.0 } else { 1.0 }, warn: false });
                self.tip_next = now + every * 60.0 * (0.7 + self.rand() as f64 * 0.6);
            }
        }
        let dur = if still { 20.0 } else { 42.0 };
        self.balloons.retain(|b| (now - b.t) / dur <= 1.0);
        let hz = h * 0.745;
        for b in &self.balloons {
            let u = ((now - b.t) / dur) as f32;
            let (s, r) = (h / 1080.0, h * 0.04);
            let x = ox + if still { w * 0.5 } else if b.dir > 0.0 { -w * 0.1 + u * w * 1.2 } else { w * 1.1 - u * w * 1.2 };
            let y = oy + hz * 0.7 + if still { 0.0 } else { (u * 7.0).sin() * h * 0.012 - (u * std::f32::consts::PI).sin() * h * 0.04 };
            let fade = (u.min(1.0 - u) * 12.0).min(1.0);
            let xf = Xf::default().with_alpha(fade);
            // the envelope: stripes of the theme's two accents
            for k in 0..6 {
                xf.ellipse(win, x, y, r * (1.0 - k as f32 / 6.0), r * 1.15, if k % 2 == 1 { pal.accent2 } else { pal.accent }, 0.95);
            }
            xf.poly(win, &Path::new(x - r * 0.62, y + r * 0.8).quad(x, y + r * 1.55, x + r * 0.62, y + r * 0.8).0, pal.accent, 0.95);
            radial(win, x - r * 0.2, y - r * 0.25, 0.0, r * 1.05, 1.1, [1.0, 1.0, 1.0], &[(0.0, 0.22 * fade), (1.0, 0.0)]); // a soft light on the envelope
            // the ropes and the basket, and the burner now and then
            let (by, bw) = (y + r * 1.75, r * 0.42);
            xf.line(win, &[(x - r * 0.3, y + r * 1.1), (x - bw / 2.0, by)], s.max(1.0), pal.fg, 0.7);
            xf.line(win, &[(x + r * 0.3, y + r * 1.1), (x + bw / 2.0, by)], s.max(1.0), pal.fg, 0.7);
            xf.rect(win, x - bw / 2.0, by, bw, r * 0.32, mix(pal.warn, pal.bg, 0.45), 1.0);
            if !still && now.rem_euclid(4.0) < 0.5 {
                xf.ellipse(win, x, y + r * 1.3, r * 0.06, r * 0.16, pal.warn, 0.9);
            }
            // the sign hanging under the basket: the tip
            let fs = (h * 0.019).round();
            let tw = draw::measure(win, font, &b.text, fs, 700.0) + h * 0.03;
            let (sy, sh) = (by + r * 0.32 + h * 0.022, h * 0.034);
            let sw = if still { 0.0 } else { ((now * 1000.0 / 700.0) as f32).sin() * h * 0.004 };
            xf.line(win, &[(x - bw * 0.3, by + r * 0.32), (x - tw * 0.35 + sw, sy)], s.max(1.0), pal.fg, 0.7);
            xf.line(win, &[(x + bw * 0.3, by + r * 0.32), (x + tw * 0.35 + sw, sy)], s.max(1.0), pal.fg, 0.7);
            xf.rect(win, x - tw / 2.0 + sw, sy, tw, sh, mix(pal.bg, pal.fg, 0.92), 0.96);
            draw::frame(win, x - tw / 2.0 + sw + 0.5, sy + 0.5, tw - 1.0, sh - 1.0, 1.0, pal.accent, 0.9 * fade);
            mid_text(win, cx, font, &b.text, x + sw, sy + sh / 2.0 + 1.0, fs, 700.0, pal.bg, fade);
            let fs2 = (h * 0.013).round();
            let tag = if b.warn { "OMARCHY NEWS" } else { "OMARCHY TIP" };
            let lw2 = draw::measure(win, font, tag, fs2, 700.0) + h * 0.012;
            xf.rect(win, x + sw - lw2 / 2.0, sy - h * 0.021, lw2, h * 0.018, pal.bg, 0.85);
            mid_text(win, cx, font, tag, x + sw, sy - h * 0.011, fs2, 700.0, if b.warn { pal.warn } else { pal.accent }, fade);
        }
    }
}

/// The Omarchy icon (Dashboard Icons, Apache-2.0), in one colour (source-in), for the GPU.
pub fn tinted_logo(c: C) -> Option<Arc<gpui::RenderImage>> {
    let im = image::load_from_memory(include_bytes!("../../extras/pulse/web/omarchy.png")).ok()?.resize_exact(128, 128, image::imageops::FilterType::Triangle).to_rgba8();
    let mut px4 = im.into_raw();
    for p in px4.chunks_exact_mut(4) {
        p[0] = (c[2] * 255.0) as u8;
        p[1] = (c[1] * 255.0) as u8;
        p[2] = (c[0] * 255.0) as u8;
    }
    Some(Arc::new(gpui::RenderImage::new(vec![image::Frame::new(image::RgbaImage::from_raw(128, 128, px4)?)])))
}
