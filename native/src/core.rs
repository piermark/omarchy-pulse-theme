//! The core of the other scenes (skyline, calm, hacker, orbit), translated from pulse.js draw(): a wireframe sphere
//! rising on the horizon (a planet over it in orbit), beating with the load, its meridians turning with the CPU, the
//! agent's face in it (drawFace); the time left of a focus as a ring; in orbit every core a moon on its ring.

use crate::draw;
use crate::palette::{mix, Pal, C};
use crate::scene::radial;
use crate::xf::{ellipse_pts, grad_fill, Xf};
use gpui::Window;

const PI: f32 = std::f32::consts::PI;
const TAU: f32 = std::f32::consts::TAU;

/// pulse.js drawFace(), on the screen.
#[allow(clippy::too_many_arguments)]
pub fn face(win: &mut Window, x: f32, y: f32, s: f32, mood: &str, t: f32, col: C, pal: &Pal, gaze: (f32, f32), woke: f32) {
    let (mut ec, mut eye_w, mut eye_h, gap) = (col, s * 0.17, s * 0.3, s * 0.26);
    let (mut look_x, mut look_y) = (0.0f32, 0.0f32);
    if mood == "think" {
        look_x = (t * 1.3).sin() * s * 0.05;
        look_y = -s * 0.05;
    }
    if mood != "sleep" && mood != "sad" {
        look_x += gaze.0 * s * 0.15; // at your pointer
        look_y += gaze.1 * s * 0.1;
    }
    let mut blink = if mood == "idle" || mood == "think" { if t.rem_euclid(4.2) < 0.13 { 0.12 } else { 1.0 } } else { 1.0 };
    if mood == "wake" {
        blink = (woke / 1.1).clamp(0.1, 1.0);
    }
    if mood == "alert" {
        eye_w *= 1.15;
        eye_h *= 1.2;
        ec = pal.warn;
    }
    if mood == "listen" {
        eye_w *= 1.1;
        eye_h *= 1.12;
        look_y -= s * 0.03;
    }
    if mood == "sad" {
        ec = pal.crit;
    }
    let lw = (s * 0.07).max(1.5);
    let arc = |win: &mut Window, cx: f32, cy: f32, r: f32, a0: f32, a1: f32| draw::line(win, &ellipse_pts(cx, cy, r, r, a0, a1, 16), lw, ec, 0.95);
    for side in [-1.0f32, 1.0] {
        let (ex, ey) = (x + side * gap + look_x, y - s * 0.08 + look_y);
        match mood {
            "happy" => arc(win, ex, ey + s * 0.06, eye_w * 0.9, PI * 1.1, PI * 1.9),
            "sleep" => arc(win, ex, ey - s * 0.03, eye_w * 0.75, PI * 0.15, PI * 0.85),
            "sad" => {
                draw::line(win, &[(ex - eye_w * 0.7, ey - eye_w * 0.7), (ex + eye_w * 0.7, ey + eye_w * 0.7)], lw, ec, 0.95);
                draw::line(win, &[(ex + eye_w * 0.7, ey - eye_w * 0.7), (ex - eye_w * 0.7, ey + eye_w * 0.7)], lw, ec, 0.95);
            }
            _ => {
                let hh = eye_h * blink;
                let r = eye_w.min(hh) / 2.0;
                win.paint_quad(gpui::PaintQuad::new(gpui::Bounds::new(gpui::point(gpui::px(ex - eye_w / 2.0), gpui::px(ey - hh / 2.0)), gpui::size(gpui::px(eye_w), gpui::px(hh)))).background(crate::palette::col(ec, 0.95)).corner_radii(gpui::px(r)));
            }
        }
        if mood == "alert" {
            draw::line(win, &[(ex - eye_w * 0.8, ey - eye_h * 0.85), (ex + eye_w * 0.6, ey - eye_h * 1.0)], lw, ec, 0.95);
        }
    }
    let my = y + s * 0.26;
    match mood {
        "sleep" => {
            // asleep: a small mouth, and z z z drifting up
            arc(win, x, my, s * 0.04, 0.0, TAU);
            for zi in 0..3 {
                let zp = (t * 0.35 + zi as f32 / 3.0).rem_euclid(1.0);
                let (zs, zx, zy) = (s * (0.07 + zp * 0.09), x + s * (0.42 + zp * 0.3), y - s * (0.25 + zp * 0.55));
                draw::line(win, &[(zx, zy), (zx + zs, zy), (zx, zy + zs), (zx + zs, zy + zs)], lw, ec, 0.95 * (zp * PI).sin());
            }
        }
        "happy" | "wake" => arc(win, x, my - s * 0.08, s * 0.16, PI * 0.15, PI * 0.85),
        "sad" => arc(win, x, my + s * 0.08, s * 0.13, PI * 1.2, PI * 1.8),
        "alert" => arc(win, x, my, s * 0.06, 0.0, TAU),
        "talk" => {
            let o = (0.3 + 0.7 * (t * 9.0).sin().abs()) * s * 0.09;
            draw::arc_fill(win, x, my, s * 0.1, o, 0.0, 0.0, TAU, ec, 0.95);
        }
        "listen" => {
            for li in -2..=2 {
                let lh = s * (0.03 + 0.07 * (t * 7.0 + li as f32 * 1.3).sin().abs());
                crate::scene::rect(win, x + li as f32 * s * 0.05 - s * 0.015, my - lh / 2.0, s * 0.03, lh, ec, 0.95);
            }
        }
        _ => draw::line(win, &[(x - s * 0.08, my), (x + s * 0.08, my)], lw, ec, 0.95),
    }
    if mood == "think" {
        // thinking dots
        for i in 0..3 {
            let on = (t * 2.2 - i as f32 * 0.33).rem_euclid(1.0) < 0.5;
            draw::arc_fill(win, x + s * (0.55 + i as f32 * 0.13), y - s * 0.45 - i as f32 * s * 0.08, s * 0.035, s * 0.035, 0.0, 0.0, TAU, ec, if on { 0.95 } else { 0.3 });
        }
    }
}

/// The core of skyline, calm, hacker and orbit; and orbit's moons. `rings` keeps where each moon is.
#[allow(clippy::too_many_arguments)]
pub fn paint(win: &mut Window, pal: &Pal, kind: &str, ox: f32, oy: f32, w: f32, h: f32, t: f32, dt: f32, cpu: f32, cores: &[f32], beat_env: f32, alert: Option<(C, f32)>, mood: Option<&str>, gaze: (f32, f32), woke: f32, focus_left: Option<f32>, rings: &mut Vec<f32>) {
    let orbit = kind == "orbit";
    let hz = oy + h * 0.745;
    let cx = ox + w / 2.0;
    let r = h * if orbit { 0.15 } else { 0.135 } * (1.0 + 0.035 * beat_env);
    let cy = if orbit { hz - r * 1.02 } else { hz };
    let (ac, a2, bg) = (pal.accent, pal.accent2, pal.bg);
    // the glow behind it
    let gc = match alert { Some((c, e)) if e > 0.05 => c, _ => a2 };
    let glow_clip = gpui::ContentMask { bounds: gpui::Bounds::new(gpui::point(gpui::px(cx - r * 2.5), gpui::px(cy - r * 2.5)), gpui::size(gpui::px(r * 5.0), gpui::px(r * 2.5 + if orbit { r * 2.5 } else { 1.0 }))), ..Default::default() };
    win.with_content_mask(Some(glow_clip), |win| radial(win, cx, cy, r * 0.2, r * 2.4, 1.0, gc, &[(0.0, 0.30 + 0.25 * beat_env), (1.0, 0.0)]));
    // the body, clipped at the horizon (but in orbit)
    let clip = gpui::ContentMask { bounds: gpui::Bounds::new(gpui::point(gpui::px(ox), gpui::px(oy)), gpui::size(gpui::px(w), gpui::px(if orbit { h } else { hz - oy }))), ..Default::default() };
    win.with_content_mask(Some(clip), |win| {
        let disc = ellipse_pts(cx, cy, r, r, 0.0, TAU, 48);
        grad_fill(win, &Xf::default(), &disc, cy - r, cy + r, &[(0.0, mix(bg, a2, 0.55), 1.0), (1.0, mix(bg, ac, 0.25), 1.0)], 24);
        let lw = (h / 700.0).max(1.0);
        for la in -3..=3 {
            // latitudes
            let yy = cy + (la as f32 / 4.0) * r;
            let rx = (r * r - (yy - cy) * (yy - cy)).max(0.0).sqrt();
            draw::line(win, &ellipse_pts(cx, yy, rx, rx * 0.08, 0.0, TAU, 36), lw, ac, 0.55);
        }
        for mer in 0..6 {
            // turning meridians
            let ph = (mer as f32 / 6.0) * PI + t * (0.12 + cpu / 600.0);
            let ex = ph.cos().abs() * r;
            draw::line(win, &ellipse_pts(cx, cy, ex.max(0.5), r, 0.0, TAU, 40), lw, ac, 0.18 + 0.4 * ph.sin().abs());
        }
        draw::line(win, &ellipse_pts(cx, cy, r, r, 0.0, TAU, 64), (h / 420.0).max(1.5), ac, 0.9);
        if let Some(m) = mood {
            // the agent lives in the core
            face(win, cx, if orbit { cy } else { cy - r * 0.45 }, r * if orbit { 0.9 } else { 0.55 }, m, t, pal.fg, pal, gaze, woke);
        }
    });
    if let Some(left) = focus_left {
        // focus: the time left as a ring around the core
        draw::line(win, &ellipse_pts(cx, cy, r * 1.12, r * 1.12, -PI / 2.0, -PI / 2.0 + TAU * left, 64), (h / 250.0).max(2.0), a2, 0.9);
    }
    if orbit {
        // every core is a moon on its ring
        let n = cores.len().min(16);
        rings.resize(n, -1.0);
        let tilt: f32 = -0.25;
        for i in 0..n {
            let load = cores[i];
            let rr = r * (1.25 + i as f32 * 0.09);
            let sp = 0.1 + load / 160.0;
            if rings[i] < 0.0 {
                rings[i] = i as f32 * 1.7;
            }
            rings[i] += sp * dt; // pulse.js: sp / fps each frame, sp a second
            let ring: Vec<(f32, f32)> = ellipse_pts(0.0, 0.0, rr, rr * 0.28, 0.0, TAU, 64).iter().map(|p| { let (s, c) = tilt.sin_cos(); (cx + p.0 * c - p.1 * s, cy + p.0 * s + p.1 * c) }).collect();
            draw::line(win, &ring, (h / 1080.0).max(1.0), ac, 0.12 + load / 400.0);
            let ang = rings[i];
            let px0 = cx + ang.cos() * rr * tilt.cos() - ang.sin() * rr * 0.28 * tilt.sin();
            let py0 = cy + ang.cos() * rr * tilt.sin() + ang.sin() * rr * 0.28 * tilt.cos();
            let front = ang.sin() > 0.0;
            let c = if load > 85.0 { pal.crit } else if front { ac } else { a2 };
            let s = (h / 300.0).max(1.5) * if front { 1.4 } else { 0.9 };
            draw::arc_fill(win, px0, py0, s, s, 0.0, 0.0, TAU, c, if front { 0.95 } else { 0.4 });
        }
    }
}

/// pulse.js drawHacker(): the radar sweeping your network (phosphor style): every device a blip that brightens as the
/// sweep passes over it, a new one pinging with NEW beside it, switches, access points and hosts too, a dead one red.
#[allow(clippy::too_many_arguments)]
pub fn radar(win: &mut Window, cx_app: &mut gpui::App, font: &gpui::SharedString, st: &serde_json::Value, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, still: bool, share: bool) {
    use crate::hub::{get, list, text};
    let hz = oy + h * 0.745;
    let cx = ox + w / 2.0;
    // the phosphor: a dark green veil over everything (the colours themselves are already green)
    crate::scene::rect(win, ox, oy, w, h, [0.0, 20.0 / 255.0, 8.0 / 255.0], 0.35);
    let r = h * 0.23;
    let cy = hz - r * 0.95;
    let sw = if still { 0.8 } else { t * 0.9 };
    let green = [60.0 / 255.0, 1.0, 140.0 / 255.0];
    for k in 1..=4 {
        draw::line(win, &ellipse_pts(cx, cy, r * k as f32 / 4.0, r * k as f32 / 4.0, 0.0, TAU, 64), 1.0, green, 0.35);
    }
    draw::line(win, &[(cx - r, cy), (cx + r, cy)], 1.0, green, 0.35);
    draw::line(win, &[(cx, cy - r), (cx, cy + r)], 1.0, green, 0.35);
    for k in 0..24 {
        let a = sw - k as f32 * 0.03;
        draw::line(win, &[(cx, cy), (cx + a.cos() * r, cy + a.sin() * r)], 1.0, green, 0.5 * (1.0 - k as f32 / 24.0));
    }
    // (name, distance from the centre, a switch, down, new)
    let mut dev: Vec<(String, f32, bool, bool, bool)> = Vec::new();
    for d in list(st, &["lan"]) {
        let n = text(d, &["name"]);
        let key = if n.is_empty() { text(d, &["ip"]) } else { n.clone() };
        dev.push((n, 0.5 + 0.4 * (key.chars().count() % 7) as f32 / 7.0, false, false, get(d, &["new"]).as_bool().unwrap_or(false)));
    }
    let net = get(st, &["net"]);
    for s in list(net, &["switches"]) {
        dev.push((text(s, &["name"]), 0.3, true, get(s, &["up"]).as_bool() == Some(false), false));
    }
    for a in list(net, &["wifi", "aps"]) {
        dev.push((a.as_str().unwrap_or("").to_string(), 0.45, false, false, false));
    }
    for hst in list(net, &["dc", "hosts"]) {
        dev.push((text(hst, &["name"]), 0.38, false, get(hst, &["up"]).as_bool() == Some(false), false));
    }
    let fs = (h * 0.01).round();
    for (i, d) in dev.iter().enumerate() {
        let mut hs: u32 = 0;
        for c in d.0.encode_utf16() {
            hs = hs.wrapping_mul(31).wrapping_add(c as u32);
        }
        let ang = ((if hs == 0 { i as u32 * 977 } else { hs }) % 628) as f32 / 100.0;
        let (x, y) = (cx + ang.cos() * r * d.1, cy + ang.sin() * r * d.1);
        let since = (sw - ang).rem_euclid(TAU);
        let lum = if still { 0.8 } else { (1.0 - since / 4.0).max(0.15) };
        let col = if d.3 { pal.crit } else { green };
        let rr = if d.2 { 4.0 } else { 2.5 };
        draw::arc_fill(win, x, y, rr, rr, 0.0, 0.0, TAU, col, lum);
        if d.4 {
            // a new device: a blip that keeps pinging, and NEW next to it
            let pq = if still { 0.5 } else { (t * 0.8 + i as f32 * 0.13).rem_euclid(1.0) };
            let pr = 4.0 + pq * r * 0.12;
            draw::line(win, &ellipse_pts(x, y, pr, pr, 0.0, TAU, 32), (h / 700.0).max(1.0), pal.warn, 0.9 * (1.0 - pq));
            draw::arc_fill(win, x, y, 3.5, 3.5, 0.0, 0.0, TAU, pal.warn, 1.0);
            if !share {
                let label = if d.0.is_empty() { "NEW".to_string() } else { format!("NEW {}", d.0.chars().take(12).collect::<String>()) };
                draw::text(win, cx_app, font, &label, x + 7.0, y - 6.0, fs, 400.0, pal.warn, 0.95, -1);
            }
        }
        if !d.0.is_empty() && !share && lum > 0.5 {
            draw::text(win, cx_app, font, &d.0.chars().take(16).collect::<String>(), x + 6.0, y + 3.0, fs, 400.0, col, lum * 0.8, -1);
        }
    }
}
