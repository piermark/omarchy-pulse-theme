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
