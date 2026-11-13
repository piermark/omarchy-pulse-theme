//! Pulse's other scenes: `orbit` (the machine as a planet, a moon on its ring for every CPU core), `hacker` (a
//! radar sweeping your network, in phosphor green) and `noir` (a megacity at night: towers in layers with thousands
//! of windows, searchlights in the clouds, a hologram board, cars in the air, rain that never stops, a wet street).

use crate::life::Frame;
use crate::palette::{Pal, C};
use crate::scene::{gradient, rect};
use gpui::{size, Bounds};
use gpui::{point, px, Window};

/// `scene = "hacker"`: everything in phosphor green, as an old terminal.
pub fn green(pal: &Pal) -> Pal {
    let g = |k: f32| -> C { [0.16 * k, 1.0 * k, 0.55 * k] };
    Pal { bg: [0.0, 0.055, 0.025], fg: g(0.85), accent: g(1.0), accent2: g(0.6), ok: g(0.9), warn: [0.75, 1.0, 0.3], crit: pal.crit, dim: g(0.32) }
}



/// What the noir scene keeps between frames: the cars in the air (pulse.js flyers), the big screen's picture and
/// the Omarchy icon in pink.
#[derive(Default)]
pub struct Noir {
    flyers: Vec<[f32; 3]>,
    pub board: Option<std::sync::Arc<gpui::RenderImage>>,
    logo: Option<std::sync::Arc<gpui::RenderImage>>,
    seed: u64,
}

impl Noir {
    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.seed >> 33) as f32) / (1u64 << 31) as f32
    }
}

/// pulse.js drawNoirCity() and drawNoir(t, hz, true): searchlights in the clouds, the antennas' red lights, the big
/// screen on the central tower (what the sphere shows in the other scenes), the neon signs, the cars in the air, the
/// rain that never stops, the city's lights on the wet street. `back` is the backdrop (in its pixels: `sf` per
/// screen pixel), `board_px` the sphere's texture seen from the front (BGRA, its size) when it is new.
#[allow(clippy::too_many_arguments)]
pub fn noir_city(win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, pal: &Pal, f: &Frame, back: &crate::backdrop::Backdrop, sf: f32, board_px: Option<(Vec<u8>, u32, u32)>, mem: &mut Noir, still: bool, frames: f32) {
    let c = |r: u8, g: u8, b: u8| -> C { [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0] };
    let (w, h, ox, oy, t) = (f.w, f.h, f.ox, f.oy, f.t);
    let hz = f.oy + f.hz;
    // searchlights sweeping the clouds ("lighter": added light)
    for (i, u) in [0.3f32, 0.55, 0.78].iter().enumerate() {
        let a = -std::f32::consts::FRAC_PI_2 + if still { (i as f32 - 1.0) * 0.3 } else { (t * (0.13 + i as f32 * 0.05) + i as f32 * 2.0).sin() * 0.55 };
        let (x, len) = (ox + w * u, h * 0.9);
        let pts = [(x, hz), (x + (a - 0.04).cos() * len, hz + (a - 0.04).sin() * len), (x + (a + 0.04).cos() * len, hz + (a + 0.04).sin() * len)];
        let ang = a.cos().atan2(-a.sin()).to_degrees();
        crate::draw::poly_grad(win, &pts, ang, c(200, 220, 255), 0.10, c(200, 220, 255), 0.0);
    }
    for (i, p) in back.noir_tops.iter().enumerate() {
        if still || (t + i as f32 * 0.37).rem_euclid(2.0) < 0.25 {
            rect(win, ox + p.0 / sf - 1.5, oy + p.1 / sf - 1.5, 3.0, 3.0, c(255, 40, 40), 0.95);
        }
    }
    // the big screen on the central tower: what the sphere shows elsewhere (the clock, the weather, the agent's face,
    // the door camera, alerts, countdowns), as a hologram
    let fl2 = if still { 0.9 } else { 0.8 + 0.2 * (t * 7.0).sin().abs() * if (t * 0.5).rem_euclid(3.0) < 2.9 { 1.0 } else { 0.3 } };
    if let Some(bd) = back.noir_board {
        let (bx, by, bw, bh) = (ox + bd[0] / sf, oy + bd[1] / sf, bd[2] / sf, bd[3] / sf);
        if let Some((px4, iw, ih)) = board_px {
            // straight alpha for the GPU, the screen's own brightness
            let mut px4 = px4;
            for q in px4.chunks_exact_mut(4) {
                q[3] = (q[3] as f32 * 0.88 * fl2) as u8;
            }
            if let Some(im) = image::RgbaImage::from_raw(iw, ih, px4) {
                if let Some(old) = mem.board.replace(std::sync::Arc::new(gpui::RenderImage::new(vec![image::Frame::new(im)]))) {
                    let _ = win.drop_image(old);
                }
            }
        }
        // pulse-native: a billboard as in Tokyo: its steel frame, its light spilling on the haze around it
        crate::draw::soft(win, bx, by + bh * 0.43, 0.0, bw * 0.95, 0.6, c(255, 120, 200), &[(0.0, 0.22 * fl2), (0.6, 0.08 * fl2), (1.0, 0.0)]);
        rect(win, bx - bw / 2.0 - bh * 0.025, by - bh * 0.025, bw + bh * 0.05, bh * 0.86 + bh * 0.05, c(14, 15, 22), 1.0);
        if let Some(im) = &mem.board {
            let at = Bounds::new(point(px(bx - bw / 2.0), px(by)), size(px(bw), px(bh * 0.86)));
            let _ = win.paint_image(at, at, gpui::Corners::default(), 0.0, im.clone(), 0, false);
        }
        crate::draw::frame(win, bx - bw / 2.0, by, bw, bh * 0.86, (h / 700.0).max(1.0), c(120, 230, 255), 0.6 * fl2);
        if mem.logo.is_none() {
            mem.logo = crate::street::tinted_logo([1.0, 120.0 / 255.0, 200.0 / 255.0]);
        }
        if let Some(lg) = &mem.logo {
            let ls = bh * 0.1;
            let at = Bounds::new(point(px(bx - bw / 2.0 + ls * 0.3), px(by + bh * 0.885)), size(px(ls), px(ls)));
            let _ = win.paint_image(at, at, gpui::Corners::default(), 0.0, lg.clone(), 0, false);
        }
        crate::draw::text(win, cx, font, "OMARCHY", bx, by + bh * 0.97, (bh * 0.08).round(), 700.0, c(255, 120, 200), 0.9 * fl2, 0);
        // (the LEDs make the lines pulse.js draws as scan lines; under the screen, a row of lamps on the frame)
        for k in 0..12 {
            let lx = bx - bw / 2.0 + bw * (k as f32 + 0.5) / 12.0;
            rect(win, lx - 1.0, by + bh * 0.86 + bh * 0.008, 2.0, 2.0, c(255, 230, 190), if (t * 2.0 + k as f32 * 0.5).rem_euclid(2.0) < 1.6 || still { 0.85 } else { 0.25 });
        }
        gradient(win, bx - bw / 2.0, hz, bw, h * 0.2, c(120, 200, 255), 0.12 * fl2, c(120, 200, 255), 0.0); // its reflection on the wet street
    }
    // ---- drawNoir(t, hz, true): an amber haze over the street
    gradient(win, ox, hz - h * 0.25, w, h * 0.21, c(255, 150, 60), 0.0, c(255, 150, 60), 0.16);
    gradient(win, ox, hz - h * 0.04, w, h * 0.09, c(255, 150, 60), 0.16, c(255, 150, 60), 0.0);
    // vertical neon signs, our own words
    let signs: [(&str, C); 4] = [("OMARCHY", pal.accent2), ("PULSE", c(80, 230, 255)), ("ARCH", c(255, 90, 160)), ("NIGHT SHIFT", c(255, 190, 80))];
    let fs = (h * 0.017).round();
    for (i, (word, sc)) in signs.iter().enumerate() {
        let (x, y) = match back.noir_signs.get(i) { Some(p) => (ox + p.0 / sf, oy + p.1 / sf), None => (ox + w * (0.29 + i as f32 * 0.14), hz - h * (0.33 - (i % 2) as f32 * 0.06)) };
        let on = still || (t * 1.3 + i as f32 * 0.7).rem_euclid(4.0) > 0.15;
        for (k, ch) in word.chars().enumerate() {
            let yk = y + k as f32 * h * 0.02;
            if on && ch != ' ' {
                crate::draw::soft(win, x, yk - fs * 0.35, 0.0, h * 0.015 * 1.4, 1.0, *sc, &[(0.0, 0.45), (1.0, 0.0)]); // its glow (shadowBlur)
            }
            let mut b = [0u8; 4];
            crate::draw::text(win, cx, font, ch.encode_utf8(&mut b), x, yk, fs, 700.0, *sc, if on { 0.9 } else { 0.25 }, 0);
        }
    }
    // the cars in the air
    if mem.flyers.is_empty() {
        mem.seed = 0x5eed_0f_a1;
        for _ in 0..4 {
            let (r1, r2, r3, r4) = (mem.rand(), mem.rand(), mem.rand(), mem.rand());
            mem.flyers.push([r1 * w, f.hz - h * (0.2 + r2 * 0.25), if r3 < 0.5 { -1.0 } else { 1.0 } * w * (0.0006 + r4 * 0.0008)]);
        }
    }
    for fl in mem.flyers.iter_mut() {
        if !still {
            fl[0] += fl[2] * frames;
            if fl[0] < -w * 0.05 {
                fl[0] = w * 1.05;
            }
            if fl[0] > w * 1.05 {
                fl[0] = -w * 0.05;
            }
        }
        let (d, s2) = (if fl[2] > 0.0 { 1.0 } else { -1.0 }, h * 0.012);
        let (fx, fy) = (ox + fl[0], oy + fl[1]);
        crate::draw::ellipse(win, fx, fy, s2 * 1.6, s2 * 0.45, c(20, 22, 30), 0.95);
        crate::draw::poly_grad(win, &[(fx + d * s2 * 1.5, fy), (fx + d * s2 * 9.0, fy - s2), (fx + d * s2 * 9.0, fy + s2 * 1.2)], if d > 0.0 { 90.0 } else { 270.0 }, c(255, 240, 200), 0.5, c(255, 240, 200), 0.0);
        rect(win, fx - d * s2 * 1.5, fy - 1.0, 2.0, 2.0, pal.crit, 0.9);
    }
    // the rain that never stops
    let segs: Vec<((f32, f32), (f32, f32))> = (0..220)
        .map(|k| {
            let kf = k as f32;
            let rx = (kf * 97.3 + if still { 0.0 } else { t * w * 0.02 * (1.0 + (k % 3) as f32) }).rem_euclid(w);
            let ry = (kf * 53.7 + if still { 0.0 } else { t * h * 0.9 * (1.0 + (k % 4) as f32 * 0.2) }).rem_euclid(h);
            ((ox + rx, oy + ry), (ox + rx - h * 0.003, oy + ry + h * 0.025))
        })
        .collect();
    crate::draw::lines(win, &segs, 1.0, c(170, 210, 255), 0.22);
    // the wet street: the city's lights stretched on the asphalt
    for k in 0..26 {
        let x = ox + (k as f32 * 211.7).rem_euclid(w);
        let lc = if k % 3 != 0 { c(255, 170, 90) } else if k % 2 != 0 { c(120, 220, 255) } else { c(255, 80, 160) };
        gradient(win, x, hz, 2.0 + (k % 4) as f32, h * 0.16, lc, 0.14, lc, 0.0);
    }
}
