//! pulse-native: your AI agents answer a click. A click on a drone (or a rover on Mars, a shard by the monolith)
//! opens its card: what it is doing, for how long, how it went; and three buttons: go to its window (the terminal it
//! runs in comes to the front, on its workspace: the place to answer it when it waits for your OK), take it away
//! (it is done, or it is stale), close. Pulse never types for an agent: you answer it yourself, in its window.

use crate::draw;
use crate::hub::{get, list, num, text};
use crate::palette::{mix, Pal, C};
use crate::scene::rect;
use gpui::Window;
use serde_json::Value;
use std::cell::RefCell;

thread_local! {
    /// where each agent is on the screen this frame: (name, x, y, radius), filled by whatever draws them
    static SPOTS: RefCell<Vec<(String, f32, f32, f32)>> = const { RefCell::new(Vec::new()) };
}

thread_local! {
    /// the panels on the screen this frame ([x, y, w, h]): a card goes where they are not (they are drawn over the scene)
    static BOXES: RefCell<Vec<[f32; 4]>> = const { RefCell::new(Vec::new()) };
}

pub fn set_boxes(b: &[[f32; 4]]) {
    BOXES.with(|x| *x.borrow_mut() = b.to_vec());
}

/// Where a card of cw × ch goes by something at (ax, ay) of radius ar: above, below, beside, slid sideways; the
/// place that the panels cover least (ties: the first, above).
pub fn place(cw: f32, ch: f32, ax: f32, ay: f32, ar: f32, ox: f32, oy: f32, w: f32, h: f32, gap: f32) -> (f32, f32) {
    let clampx = |x: f32| x.clamp(ox + gap, (ox + w - cw - gap).max(ox + gap));
    let clampy = |y: f32| y.clamp(oy + gap, (oy + h - ch - gap).max(oy + gap));
    let mut cands = Vec::new();
    for dx in [0.0f32, -0.5, 0.5, -1.0, 1.0] {
        let x = clampx(ax - cw / 2.0 + dx * cw);
        cands.push((x, clampy(ay - ar - ch - gap)));
        cands.push((x, clampy(ay + ar + gap)));
    }
    cands.push((clampx(ax + ar + gap), clampy(ay - ch / 2.0)));
    cands.push((clampx(ax - ar - gap - cw), clampy(ay - ch / 2.0)));
    let cover = |x: f32, y: f32| -> f32 {
        BOXES.with(|bx| bx.borrow().iter().map(|b| ((x + cw).min(b[0] + b[2]) - x.max(b[0])).max(0.0) * ((y + ch).min(b[1] + b[3]) - y.max(b[1])).max(0.0)).sum())
    };
    let mut best = cands[0];
    let mut bc = f32::MAX;
    for c in cands {
        let v = cover(c.0, c.1) + ((c.0 + cw / 2.0 - ax).abs() + (c.1 + ch / 2.0 - ay).abs()) * 0.01;
        if v < bc {
            bc = v;
            best = c;
        }
    }
    best
}

/// Called by what draws an agent: here it is this frame.
pub fn spot(name: &str, x: f32, y: f32, r: f32) {
    if !name.is_empty() {
        SPOTS.with(|s| s.borrow_mut().push((name.to_string(), x, y, r)));
    }
}

pub fn clear_spots() {
    SPOTS.with(|s| s.borrow_mut().clear());
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    Focus,
    Remove,
    Close,
}

#[derive(Default)]
pub struct Card {
    /// the agent shown, and when the card opened
    open: Option<(String, f64)>,
    /// its buttons this frame: (x, y, w, h, what)
    buttons: Vec<(f32, f32, f32, f32, Act)>,
    /// the card itself, so a click on it does not fall through
    area: Option<[f32; 4]>,
    /// the answer of the last button, shown on the card for a few seconds
    pub note: Option<(String, f64)>,
    /// where it opened: it stays there (a card that followed its drone would slide from under the pointer)
    pos: Option<(f32, f32, f32)>,
}

/// What a click did.
pub enum Clicked {
    Nothing,
    Opened,
    Act(String, Act),
    Inside,
}

impl Card {
    pub fn close(&mut self) {
        self.open = None;
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// A click: a button of the open card, the card itself, or an agent (which opens its card).
    pub fn click(&mut self, x: f32, y: f32, t: f64) -> Clicked {
        if let Some((name, _)) = self.open.clone() {
            for b in &self.buttons {
                if x >= b.0 && x <= b.0 + b.2 && y >= b.1 && y <= b.1 + b.3 {
                    if b.4 == Act::Close {
                        self.open = None;
                    }
                    return Clicked::Act(name, b.4);
                }
            }
            if let Some(a) = self.area {
                if x >= a[0] && x <= a[0] + a[2] && y >= a[1] && y <= a[1] + a[3] {
                    return Clicked::Inside;
                }
            }
        }
        if std::env::var("PULSE_CLICK_LOG").is_ok() {
            SPOTS.with(|s| eprintln!("click {x:.0},{y:.0} · agents at {:?}", s.borrow().iter().map(|p| (p.0.clone(), p.1 as i32, p.2 as i32, p.3 as i32)).collect::<Vec<_>>()));
        }
        let hit = SPOTS.with(|s| s.borrow().iter().rev().find(|p| ((x - p.1).powi(2) + (y - p.2).powi(2)).sqrt() <= p.3).map(|p| p.0.clone()));
        match hit {
            Some(n) => {
                if self.open.as_ref().map(|o| o.0 == n).unwrap_or(false) {
                    self.open = None; // a second click on the same agent closes it
                    return Clicked::Inside;
                }
                self.open = Some((n, t));
                self.note = None;
                self.pos = None;
                Clicked::Opened
            }
            None => {
                if self.open.take().is_some() {
                    return Clicked::Inside; // a click elsewhere closes the card first
                }
                Clicked::Nothing
            }
        }
    }

    /// The card, over everything, by the agent (or in the middle when the agent is not on the screen any more).
    #[allow(clippy::too_many_arguments)]
    pub fn paint(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, st: &Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f64) {
        self.buttons.clear();
        self.area = None;
        let Some((name, at)) = self.open.clone() else { return };
        let Some(a) = list(st, &["agents"]).iter().find(|a| text(a, &["name"]) == name).cloned() else {
            self.open = None;
            return;
        };
        if t - at > 90.0 {
            self.open = None; // a card left open closes by itself
            return;
        }
        let rem = crate::ui::rem(w, h);
        let fs = (0.95 * rem).round();
        let small = (0.82 * rem).round();
        let state = text(&a, &["state"]);
        let col: C = match state.as_str() { "waiting" => pal.warn, "done" => pal.ok, "failed" => pal.crit, "idle" => pal.dim, _ => pal.accent };
        let now = num(st, &["t"]).unwrap_or(0.0);
        let since = num(&a, &["since"]).unwrap_or(now);
        let mins = ((now - since) / 60.0).max(0.0).floor() as i64;
        let what = text(&a, &["text"]);
        let health = num(&a, &["health"]);
        let runs = num(&a, &["runs"]).unwrap_or(0.0);
        let state_words = match state.as_str() {
            "working" => format!("working · {}", if mins >= 60 { format!("{}h {}m", mins / 60, mins % 60) } else { format!("{mins}m") }),
            "waiting" => "waiting for you · answer it in its window".to_string(),
            "done" => "done".to_string(),
            "failed" => "it failed".to_string(),
            s => s.to_string(),
        };
        let mut lines: Vec<(String, C, f32, f32)> = vec![(name.clone(), pal.fg, 1.0, fs), (state_words, col, 1.0, small)];
        if !what.is_empty() {
            lines.push((what.chars().take(60).collect(), pal.fg, 0.7, small));
        }
        if runs >= 3.0 {
            if let Some(hv) = health {
                lines.push((format!("{} runs · {:.0}% went well", runs as i64, hv * 100.0), pal.fg, 0.55, small));
            }
        }
        if let Some((n, nt)) = &self.note {
            if t - nt < 6.0 {
                lines.push((n.clone(), pal.accent2, 1.0, small));
            }
        }
        let buttons: Vec<(&str, Act)> = vec![("↗ its window", Act::Focus), ("✓ take it away", Act::Remove), ("✕", Act::Close)];
        let pad = 0.8 * rem;
        let text_w = lines.iter().map(|l| draw::measure(win, font, &l.0, l.3, 400.0)).fold(0.0f32, f32::max);
        let bws: Vec<f32> = buttons.iter().map(|b| draw::measure(win, font, b.0, small, 400.0) + 1.2 * rem).collect();
        let row_w = bws.iter().sum::<f32>() + 0.5 * rem * (bws.len() as f32 - 1.0);
        let cw = text_w.max(row_w) + pad * 2.0;
        let lh: f32 = lines.iter().map(|l| l.3 * 1.45).sum();
        let bh = small * 1.9;
        let ch = pad + lh + 0.6 * rem + bh + pad;
        // by the agent: above it when there is room, else below
        let at_spot = SPOTS.with(|s| s.borrow().iter().find(|p| p.0 == name).map(|p| (p.1, p.2, p.3)));
        let (ax, ay, ar) = at_spot.unwrap_or((ox + w / 2.0, oy + h * 0.45, 0.0));
        // placed once; a taller card (a note) grows downwards from there
        let (x0, y0) = match self.pos {
            Some((x, y, _)) => (x, y.min(oy + h - ch - 0.6 * rem)),
            None => {
                let p = place(cw, ch, ax, ay, ar, ox, oy, w, h, 0.6 * rem);
                self.pos = Some((p.0, p.1, ch));
                p
            }
        };
        let open_k = ((t - at) as f32 / 0.18).clamp(0.0, 1.0);
        rect(win, x0, y0, cw, ch, pal.bg, 0.92 * open_k);
        draw::frame(win, x0, y0, cw, ch, 1.0, mix(pal.bg, col, 0.85), open_k);
        rect(win, x0, y0, 0.25 * rem, ch, col, 0.9 * open_k); // its state, down the side
        if at_spot.is_some() {
            // a thin line to the agent
            let (lx, ly) = (ax.clamp(x0, x0 + cw), ay.clamp(y0, y0 + ch));
            draw::line(win, &[(lx, ly), (ax, ay)], 1.0, col, 0.5 * open_k);
        }
        let mut y = y0 + pad;
        for (s, c, a2, size) in &lines {
            draw::text(win, cx, font, s, x0 + pad, y + size * 1.05, *size, if *size == fs { 600.0 } else { 400.0 }, *c, a2 * open_k, -1);
            y += size * 1.45;
        }
        y += 0.6 * rem;
        let mut x = x0 + pad;
        for ((label, act), bw) in buttons.iter().zip(bws) {
            let accent = match act { Act::Focus => col, Act::Remove => pal.ok, Act::Close => pal.dim };
            rect(win, x, y, bw, bh, mix(pal.bg, accent, 0.18), open_k);
            draw::frame(win, x, y, bw, bh, 1.0, mix(pal.bg, accent, 0.7), open_k);
            draw::text(win, cx, font, label, x + bw / 2.0, y + bh * 0.5 + small * 0.35, small, 400.0, pal.fg, open_k, 0);
            self.buttons.push((x, y, bw, bh, *act));
            x += bw + 0.5 * rem;
        }
        self.area = Some([x0, y0, cw, ch]);
        if std::env::var("PULSE_CLICK_LOG").is_ok() && t - at < 0.05 {
            eprintln!("card {name} buttons {:?}", self.buttons.iter().map(|b| ((b.0 + b.2 / 2.0) as i32, (b.1 + b.3 / 2.0) as i32, b.4)).collect::<Vec<_>>());
        }
        let _ = get; // (the readers above)
    }
}

/// Its window to the front: Hyprland's windows, the one whose title or class names the agent (and, among several,
/// the one that also names its repo). Runs on its own thread; the answer comes back as the card's note.
pub fn focus(name: String, what: String, done: std::sync::Arc<std::sync::Mutex<Option<String>>>) {
    let _ = std::thread::Builder::new().name("pulse-focus".into()).spawn(move || {
        let words = match find_window(&name, &what) {
            Some((addr, title)) => {
                let ok = std::process::Command::new("hyprctl").args(["dispatch", "focuswindow", &format!("address:{addr}")]).output().map(|o| o.status.success()).unwrap_or(false);
                if ok { format!("→ {}", title.chars().take(48).collect::<String>()) } else { "Hyprland did not answer".into() }
            }
            None => format!("no window that says {}", first_word(&name)),
        };
        if let Ok(mut d) = done.lock() {
            *d = Some(words);
        }
    });
}

fn first_word(name: &str) -> String {
    name.split(|c: char| c.is_whitespace() || c == '·' || c == ':' || c == '@').find(|w| !w.is_empty()).unwrap_or(name).to_lowercase()
}

fn find_window(name: &str, what: &str) -> Option<(String, String)> {
    let out = std::process::Command::new("hyprctl").args(["clients", "-j"]).output().ok()?;
    let v: Value = serde_json::from_slice(&out.stdout).ok()?;
    pick_window(&v, name, what)
}

fn pick_window(v: &Value, name: &str, what: &str) -> Option<(String, String)> {
    let key = first_word(name);
    let full = name.to_lowercase();
    // the repo or project, when the agent's words or name carry one ("claude · omarchytv", "omarchytv editing")
    let mut extra: Vec<String> = full.split([' ', '·', ':', '@']).map(str::to_string).collect();
    extra.extend(what.to_lowercase().split_whitespace().take(1).map(str::to_string));
    extra.retain(|w| w.len() > 2 && *w != key);
    let mut best: Option<(i32, String, String)> = None;
    for c in v.as_array()? {
        let title = c.get("title").and_then(Value::as_str).unwrap_or("").to_string();
        let class = c.get("class").and_then(Value::as_str).unwrap_or("").to_lowercase();
        let (tl, addr) = (title.to_lowercase(), c.get("address").and_then(Value::as_str).unwrap_or("").to_string());
        if addr.is_empty() {
            continue;
        }
        let mut score = 0;
        if tl.contains(&key) || class.contains(&key) {
            score += 10;
        }
        if score == 0 {
            continue;
        }
        score += extra.iter().filter(|e| tl.contains(e.as_str())).count() as i32 * 3;
        // a terminal is where an agent runs
        if ["ghostty", "alacritty", "kitty", "foot", "wezterm", "terminal"].iter().any(|t| class.contains(t)) {
            score += 2;
        }
        if best.as_ref().map(|b| score > b.0).unwrap_or(true) {
            best = Some((score, addr, title));
        }
    }
    best.map(|b| (b.1, b.2))
}

#[cfg(test)]
mod tests {
    #[test]
    fn words() {
        assert_eq!(super::first_word("Claude · omarchytv"), "claude");
        assert_eq!(super::first_word("codex"), "codex");
    }
    #[test]
    fn windows() {
        let v: serde_json::Value = serde_json::from_str(r#"[{"address":"0x1","class":"chromium","title":"Claude Code docs"},{"address":"0x2","class":"com.mitchellh.ghostty","title":"✳ claude · pulse"},{"address":"0x3","class":"com.mitchellh.ghostty","title":"claude · omarchytv"},{"address":"0x4","class":"com.mitchellh.ghostty","title":"nvim"}]"#).unwrap();
        assert_eq!(super::pick_window(&v, "claude · omarchytv", "editing").unwrap().0, "0x3");
        assert_eq!(super::pick_window(&v, "claude", "").unwrap().0, "0x2");
        assert!(super::pick_window(&v, "grok", "").is_none());
    }
}
