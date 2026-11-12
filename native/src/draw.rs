//! Small drawing helpers over GPUI's quads and paths: ellipses, arcs, polygons and lines, with the angles of a
//! web canvas (0 = east, clockwise on the screen), so a scene written for the web page ports line by line.

use crate::palette::{col, C};
use gpui::{point, px, PathBuilder, Window};

pub const TAU: f32 = std::f32::consts::TAU;
pub const PI: f32 = std::f32::consts::PI;

/// A colour from its 0..255 parts.
pub const fn c8(r: u8, g: u8, b: u8) -> C {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

/// Words the scene wants written: centred on x, in the scene's own size and colour (main.rs writes them).
#[derive(Clone)]
pub struct Label {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub c: C,
    pub a: f32,
    pub text: String,
    pub bold: bool,
}

/// What a click can find: a place, how wide, and what it says.
#[derive(Clone)]
pub struct Hit {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub text: String,
}

pub fn poly(win: &mut Window, pts: &[(f32, f32)], c: C, a: f32) {
    if pts.len() < 3 || a <= 0.004 {
        return;
    }
    let mut b = PathBuilder::fill();
    b.move_to(point(px(pts[0].0), px(pts[0].1)));
    for p in &pts[1..] {
        b.line_to(point(px(p.0), px(p.1)));
    }
    b.close();
    if let Ok(path) = b.build() {
        win.paint_path(path, col(c, a));
    }
}

pub fn line(win: &mut Window, pts: &[(f32, f32)], lw: f32, c: C, a: f32) {
    if pts.len() < 2 || a <= 0.004 {
        return;
    }
    let mut b = PathBuilder::stroke(px(lw.max(1.0)));
    b.move_to(point(px(pts[0].0), px(pts[0].1)));
    for p in &pts[1..] {
        b.line_to(point(px(p.0), px(p.1)));
    }
    if let Ok(path) = b.build() {
        win.paint_path(path, col(c, a));
    }
}

fn arc_pts(cx: f32, cy: f32, rx: f32, ry: f32, rot: f32, a0: f32, a1: f32) -> Vec<(f32, f32)> {
    let n = (((a1 - a0).abs() / TAU * 40.0).ceil() as usize).clamp(2, 48);
    let (sn, cs) = rot.sin_cos();
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            let (x, y) = (a.cos() * rx, a.sin() * ry);
            (cx + x * cs - y * sn, cy + x * sn + y * cs)
        })
        .collect()
}

/// A filled ellipse, or a slice of one closed by its chord (a0..a1, canvas angles).
#[allow(clippy::too_many_arguments)]
pub fn arc_fill(win: &mut Window, cx: f32, cy: f32, rx: f32, ry: f32, rot: f32, a0: f32, a1: f32, c: C, a: f32) {
    if rx <= 0.0 || ry <= 0.0 {
        return;
    }
    poly(win, &arc_pts(cx, cy, rx, ry, rot, a0, a1), c, a);
}

pub fn ellipse(win: &mut Window, cx: f32, cy: f32, rx: f32, ry: f32, c: C, a: f32) {
    arc_fill(win, cx, cy, rx, ry, 0.0, 0.0, TAU, c, a);
}

#[allow(clippy::too_many_arguments)]
pub fn arc(win: &mut Window, cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32, lw: f32, c: C, a: f32) {
    if rx <= 0.0 || ry <= 0.0 {
        return;
    }
    line(win, &arc_pts(cx, cy, rx, ry, 0.0, a0, a1), lw, c, a);
}

/// The outline of a rectangle.
pub fn frame(win: &mut Window, x: f32, y: f32, w: f32, h: f32, lw: f32, c: C, a: f32) {
    line(win, &[(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)], lw, c, a);
}

/// Text drawn in the scene, as fillText on a canvas: `x` is its left edge (or its middle, or its right edge, by
/// `align`: -1, 0, 1), `y` its baseline. Returns its width.
#[allow(clippy::too_many_arguments)]
pub fn text(win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, s: &str, x: f32, y: f32, size: f32, weight: f32, c: C, a: f32, align: i8) -> f32 {
    if s.is_empty() || a <= 0.004 {
        return 0.0;
    }
    let f = gpui::Font { family: font.clone(), features: Default::default(), fallbacks: None, weight: gpui::FontWeight(weight), style: gpui::FontStyle::Normal, width: Default::default() };
    let run = gpui::TextRun { len: s.len(), font: f, color: gpui::rgb_to_hsla(col(c, a)), background_color: None, underline: None, strikethrough: None, letter_spacing: None };
    let line = win.text_system().shape_line(s.to_string().into(), px(size), &[run]);
    let w = f32::from(line.width());
    let x0 = match align { 0 => x - w / 2.0, 1 => x - w, _ => x };
    let asc = f32::from(line.ascent);
    let lh = asc + f32::from(line.descent);
    let _ = line.paint(point(px(x0), px(y - asc)), px(lh), gpui::TextAlign::Left, None, win, cx);
    w
}

/// measureText: how wide this text would be.
pub fn measure(win: &mut Window, font: &gpui::SharedString, s: &str, size: f32, weight: f32) -> f32 {
    if s.is_empty() {
        return 0.0;
    }
    let f = gpui::Font { family: font.clone(), features: Default::default(), fallbacks: None, weight: gpui::FontWeight(weight), style: gpui::FontStyle::Normal, width: Default::default() };
    let run = gpui::TextRun { len: s.len(), font: f, color: gpui::rgb_to_hsla(col([0.0; 3], 1.0)), background_color: None, underline: None, strikethrough: None, letter_spacing: None };
    f32::from(win.text_system().shape_line(s.to_string().into(), px(size), &[run]).width())
}

thread_local! {
    /// the soft lights already made: one small picture each, kept on the GPU (a gradient costs nothing to draw again)
    static SOFT: std::cell::RefCell<std::collections::HashMap<u64, std::sync::Arc<gpui::RenderImage>>> = Default::default();
}

/// createRadialGradient(cx, cy, r0, cx, cy, r1) with these stops of alpha, in one colour, squashed by `sy`: a small
/// picture made once (on the CPU, 64 pixels wide) and then only drawn by the GPU, stretched to its size.
#[allow(clippy::too_many_arguments)]
pub fn soft(win: &mut Window, cx: f32, cy: f32, r0: f32, r1: f32, sy: f32, c: C, stops: &[(f32, f32)]) {
    if r1 <= 0.5 || stops.is_empty() || stops.iter().all(|s| s.1 <= 0.004) {
        return;
    }
    // the shape at full strength, and how strong it is now (in 32 steps, so the pictures stay few)
    let top = stops.iter().fold(0.0f32, |m, s| m.max(s.1)).min(1.0);
    let level = (top * 32.0).round().clamp(1.0, 32.0) as u64;
    let r0f = (r0 / r1).clamp(0.0, 0.99);
    let mut key: u64 = 1469598103934665603;
    let mut mixk = |v: u64| { key ^= v; key = key.wrapping_mul(1099511628211); };
    for v in c { mixk((v.clamp(0.0, 1.0) * 255.0) as u64) }
    for s in stops { mixk((s.0 * 64.0) as u64); mixk(((s.1 / top.max(1e-4)) * 64.0) as u64) }
    mixk((r0f * 64.0) as u64);
    mixk(level);
    let img = SOFT.with(|m| m.borrow().get(&key).cloned());
    let img = match img {
        Some(i) => i,
        None => {
            const N: u32 = 64;
            let a_of = |k: f32| -> f32 {
                let mut prev = stops[0];
                for s in stops {
                    if k <= s.0 {
                        let span = (s.0 - prev.0).max(1e-6);
                        return prev.1 + (s.1 - prev.1) * ((k - prev.0) / span).clamp(0.0, 1.0);
                    }
                    prev = *s;
                }
                prev.1
            };
            let scale = (level as f32 / 32.0) / top.max(1e-4);
            let (b, g, r) = ((c[2].clamp(0.0, 1.0) * 255.0) as u8, (c[1].clamp(0.0, 1.0) * 255.0) as u8, (c[0].clamp(0.0, 1.0) * 255.0) as u8);
            let mut buf = vec![0u8; (N * N * 4) as usize];
            for y in 0..N {
                for x in 0..N {
                    let (dx, dy) = ((x as f32 + 0.5) / N as f32 * 2.0 - 1.0, (y as f32 + 0.5) / N as f32 * 2.0 - 1.0);
                    let d = (dx * dx + dy * dy).sqrt();
                    if d > 1.0 {
                        continue;
                    }
                    let k = ((d - r0f) / (1.0 - r0f)).clamp(0.0, 1.0);
                    let edge = ((1.0 - d) * N as f32 / 2.0).clamp(0.0, 1.0); // the outer edge, smooth
                    let a = (a_of(k) * scale).clamp(0.0, 1.0) * edge;
                    let o = ((y * N + x) * 4) as usize;
                    buf[o..o + 4].copy_from_slice(&[b, g, r, (a * 255.0).round() as u8]);
                }
            }
            let Some(im) = image::RgbaImage::from_raw(N, N, buf) else { return };
            let i = std::sync::Arc::new(gpui::RenderImage::new(vec![image::Frame::new(im)]));
            let old: Vec<_> = SOFT.with(|m| {
                let mut m = m.borrow_mut();
                let old = if m.len() > 800 { m.drain().map(|x| x.1).collect() } else { Vec::new() };
                m.insert(key, i.clone());
                old
            });
            for o in old {
                let _ = win.drop_image(o);
            }
            i
        }
    };
    let at = gpui::Bounds::new(point(px(cx - r1), px(cy - r1 * sy)), gpui::size(px(r1 * 2.0), px(r1 * 2.0 * sy)));
    let _ = win.paint_image(at, at, gpui::Corners::default(), 0.0, img, 0, false);
}

thread_local! {
    static VIG: std::cell::RefCell<Option<std::sync::Arc<gpui::RenderImage>>> = Default::default();
}

/// pulse.css #vig: radial-gradient(ellipse 75% 70% at 50% 55%, transparent 45%, rgba(0,0,0,.55) 100%) over the
/// whole scene. One small picture, made once, stretched by the GPU.
pub fn vignette(win: &mut Window, x: f32, y: f32, w: f32, h: f32) {
    let img = VIG.with(|v| {
        v.borrow_mut()
            .get_or_insert_with(|| {
                let (n, m) = (192u32, 108u32);
                let mut buf = vec![0u8; (n * m * 4) as usize];
                for j in 0..m {
                    for i in 0..n {
                        let (u, v2) = ((i as f32 + 0.5) / n as f32, (j as f32 + 0.5) / m as f32);
                        let d = (((u - 0.5) / 0.75).powi(2) + ((v2 - 0.55) / 0.7).powi(2)).sqrt();
                        let a = 0.55 * ((d - 0.45) / 0.55).clamp(0.0, 1.0);
                        buf[((j * n + i) * 4 + 3) as usize] = (a * 255.0).round() as u8;
                    }
                }
                let im = image::RgbaImage::from_raw(n, m, buf).expect("the vignette");
                std::sync::Arc::new(gpui::RenderImage::new(vec![image::Frame::new(im)]))
            })
            .clone()
    });
    let at = gpui::Bounds::new(point(px(x), px(y)), gpui::size(px(w), px(h)));
    let _ = win.paint_image(at, at, gpui::Corners::default(), 0.0, img, 0, false);
}
