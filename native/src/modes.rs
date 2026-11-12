//! Pulse's other scenes: `orbit` (the machine as a planet, a moon on its ring for every CPU core), `hacker` (a
//! radar sweeping your network, in phosphor green) and `noir` (a megacity at night: towers in layers with thousands
//! of windows, searchlights in the clouds, a hologram board, cars in the air, rain that never stops, a wet street).

use crate::hub::{list, num, text};
use crate::life::Frame;
use crate::palette::{col, mix, scale, Pal, C};
use crate::scene::{glow, gradient, rect, PlaneMark, Rand};
use gpui::{point, px, PathBuilder, Window};
use serde_json::Value;

/// `scene = "hacker"`: everything in phosphor green, as an old terminal.
pub fn green(pal: &Pal) -> Pal {
    let g = |k: f32| -> C { [0.16 * k, 1.0 * k, 0.55 * k] };
    Pal { bg: [0.0, 0.055, 0.025], fg: g(0.85), accent: g(1.0), accent2: g(0.6), ok: g(0.9), warn: [0.75, 1.0, 0.3], crit: pal.crit, dim: g(0.32) }
}



/// `scene = "noir"`: the megacity's own sky and street (its colours are its own, as on the web page).
pub fn noir_sky(win: &mut Window, f: &Frame) {
    let c = |r: u8, g: u8, b: u8| -> C { [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0] };
    gradient(win, f.ox, f.oy, f.w, f.hz * 0.55 + 1.0, c(6, 9, 18), 1.0, c(22, 30, 48), 1.0);
    gradient(win, f.ox, f.oy + f.hz * 0.55, f.w, f.hz * 0.45 + 1.0, c(22, 30, 48), 1.0, c(92, 64, 48), 1.0);
    gradient(win, f.ox, f.oy + f.hz, f.w, f.h - f.hz, c(24, 22, 30), 1.0, c(6, 6, 10), 1.0);
}

/// The megacity itself, over its sky: layers of towers, searchlights, the board, the cars in the air, the rain.
pub fn noir_city(win: &mut Window, st: &Value, pal: &Pal, f: &Frame, marks: &mut Vec<PlaneMark>) {
    let c = |r: u8, g: u8, b: u8| -> C { [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0] };
    let mut rnd = Rand::new(4242);
    // searchlights in the clouds
    for i in 0..4 {
        let bx = f.ox + f.w * (0.18 + 0.21 * i as f32);
        let a = -1.57 + (f.t * (0.12 + 0.03 * i as f32) + i as f32 * 1.3).sin() * 0.5;
        let len = f.hz * 0.9;
        let (ex, ey) = (bx + a.cos() * len, f.oy + f.hz + a.sin() * len);
        let mut beam = PathBuilder::fill();
        beam.move_to(point(px(bx - f.u * 2.0), px(f.oy + f.hz)));
        beam.line_to(point(px(ex - f.w * 0.018), px(ey)));
        beam.line_to(point(px(ex + f.w * 0.018), px(ey)));
        beam.line_to(point(px(bx + f.u * 2.0), px(f.oy + f.hz)));
        beam.close();
        if let Ok(p) = beam.build() {
            win.paint_path(p, col(c(170, 200, 255), 0.05));
        }
    }
    // three layers of towers, the far ones paler in the haze
    for layer in 0..3 {
        let depth = layer as f32 / 2.0;
        let body = mix(c(40, 44, 62), c(10, 11, 18), depth);
        let mut x = -0.02f32;
        while x < 1.02 {
            let bw = (0.014 + rnd.next() * 0.03) * (0.8 + depth * 0.7);
            let bh = (0.10 + rnd.next() * 0.30) * (0.55 + depth * 0.55) * if rnd.next() > 0.85 { 1.35 } else { 1.0 };
            let (px0, pw, ph) = (f.ox + x * f.w, bw * f.w, bh * f.h);
            let py = f.oy + f.hz - ph;
            rect(win, px0, py, pw, ph + 1.0, body, 1.0);
            rect(win, px0 + pw * 0.7, py, pw * 0.3, ph, scale(body, 0.7), 1.0);
            let (cw, ch) = ((f.u * (1.8 + depth)).max(1.2), (f.u * (2.2 + depth)).max(1.2));
            let (cols, rows) = ((((pw - cw) / (cw * 2.0)) as i32).min(14), (((ph - ch) / (ch * 2.2)) as i32).min(70));
            for iy in 0..rows {
                for ix in 0..cols {
                    let k = rnd.next();
                    if k < 0.34 {
                        let wc = if k < 0.03 { c(120, 220, 255) } else if k < 0.06 { c(255, 90, 160) } else { c(255, 206, 140) };
                        rect(win, px0 + cw + ix as f32 * cw * 2.0, py + ch * 1.5 + iy as f32 * ch * 2.2, cw, ch, wc, 0.30 + 0.5 * depth);
                    }
                }
            }
            if rnd.next() > 0.7 {
                let blink = if (f.t * 0.7 + x * 40.0).fract() < 0.5 { 1.0 } else { 0.2 };
                glow(win, px0 + pw / 2.0, py - f.u * 4.0, f.h * 0.008, c(255, 70, 70), blink);
            }
            if layer == 2 && rnd.next() > 0.6 {
                // a neon sign down its side
                let nc = if rnd.next() > 0.5 { c(255, 60, 150) } else { c(60, 220, 255) };
                rect(win, px0 + pw * 0.1, py + ph * 0.2, (f.u * 3.0).max(1.5), ph * 0.3, nc, 0.9);
                glow(win, px0 + pw * 0.1, py + ph * 0.35, ph * 0.18, nc, 0.25);
            }
            x += bw + rnd.next() * 0.006;
        }
    }
    // the hologram board over the city: the hour, and what the machine is doing
    let (bw, bh) = (f.w * 0.20, f.h * 0.11);
    let (bx, by) = (f.ox + f.w / 2.0 - bw / 2.0, f.oy + f.hz - f.h * 0.36);
    let flick = 0.85 + 0.15 * (f.t * 13.0).sin();
    rect(win, bx, by, bw, bh, c(60, 220, 255), 0.10 * flick);
    for k in 0..12 {
        rect(win, bx, by + bh * k as f32 / 12.0, bw, (f.u * 1.0).max(1.0), c(60, 220, 255), 0.18 * flick);
    }
    rect(win, bx, by, bw, (f.u * 2.0).max(1.0), c(60, 220, 255), 0.9);
    rect(win, bx, by + bh, bw, (f.u * 2.0).max(1.0), c(60, 220, 255), 0.9);
    let cpu = num(st, &["sys", "cpu"]).unwrap_or(0.0);
    marks.push(PlaneMark { c: c(60, 220, 255), x: f.w / 2.0 - bw * 0.42, y: by - f.oy + bh * 0.28, title: format!("CPU {cpu:.0}% · {} agents", list(st, &["agents"]).len()), line: text(st, &["space", "launch", "mission"]) });
    // cars in the air, between the towers
    for i in 0..7 {
        let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
        let lane = f.oy + f.hz - f.h * (0.07 + 0.045 * (i % 4) as f32);
        let k = (f.t * (0.035 + 0.008 * i as f32) + i as f32 * 0.19).fract();
        let x = f.ox + f.w * if dir > 0.0 { k * 1.1 - 0.05 } else { 1.05 - k * 1.1 };
        rect(win, x - f.u * 7.0, lane, f.u * 14.0, f.u * 3.0, c(30, 32, 44), 1.0);
        glow(win, x + dir * f.u * 8.0, lane + f.u * 1.5, f.u * 9.0, c(255, 240, 200), 0.8);
        rect(win, x - dir * f.u * 8.0, lane + f.u * 0.5, f.u * 2.0, f.u * 2.0, pal.crit, 0.9);
    }
    // the wet street: the city's light smeared down it
    for i in 0..26 {
        let x = f.ox + f.w * (i as f32 + 0.5) / 26.0;
        let wc = [c(255, 206, 140), c(60, 220, 255), c(255, 60, 150)][i % 3];
        gradient(win, x - f.w * 0.012, f.oy + f.hz + f.u * 2.0, f.w * 0.024, (f.h - f.hz) * (0.25 + 0.35 * (((i * 37) % 11) as f32 / 11.0)), wc, 0.10, wc, 0.0);
    }
    // the rain that never stops
    let mut rain = PathBuilder::stroke(px((f.u * 1.0).max(1.0)));
    for k in 0..220 {
        let kf = k as f32;
        let rx = (kf * 97.3 + f.t * f.w * 0.02 * (1.0 + (k % 3) as f32)).rem_euclid(f.w);
        let ry = (kf * 53.7 + f.t * f.h * 0.9 * (1.0 + (k % 4) as f32 * 0.2)).rem_euclid(f.h);
        rain.move_to(point(px(f.ox + rx), px(f.oy + ry)));
        rain.line_to(point(px(f.ox + rx - f.h * 0.003), px(f.oy + ry + f.h * 0.025)));
    }
    if let Ok(p) = rain.build() {
        win.paint_path(p, col(c(170, 210, 255), 0.22));
    }
}
