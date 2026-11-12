//! The canvas of pulse.js, for what it draws turned or scaled (g.save, translate, rotate, scale, restore): a transform
//! applied to the points, and the few shapes the translations need (rectangles, paths with quadratic curves,
//! ellipses, and a shape filled with a linear gradient along its own vertical axis, drawn in bands).

use crate::draw;
use crate::palette::{mix, C};
use gpui::Window;

/// x' = a·x + c·y + e, y' = b·x + d·y + f (the canvas's setTransform).
#[derive(Clone, Copy, Debug)]
pub struct Xf {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
    /// globalAlpha
    pub alpha: f32,
}

impl Default for Xf {
    fn default() -> Self {
        Xf { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0, alpha: 1.0 }
    }
}

impl Xf {
    pub fn at(x: f32, y: f32) -> Self {
        Xf { e: x, f: y, ..Default::default() }
    }
    pub fn rotate(self, r: f32) -> Self {
        let (s, co) = (r.sin(), r.cos());
        Xf { a: self.a * co + self.c * s, b: self.b * co + self.d * s, c: -self.a * s + self.c * co, d: -self.b * s + self.d * co, ..self }
    }
    pub fn scale(self, sx: f32, sy: f32) -> Self {
        Xf { a: self.a * sx, b: self.b * sx, c: self.c * sy, d: self.d * sy, ..self }
    }
    pub fn with_alpha(self, a: f32) -> Self {
        Xf { alpha: self.alpha * a, ..self }
    }
    pub fn p(&self, x: f32, y: f32) -> (f32, f32) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }
    pub fn rect(&self, win: &mut Window, x: f32, y: f32, w: f32, h: f32, col: C, a: f32) {
        draw::poly(win, &[self.p(x, y), self.p(x + w, y), self.p(x + w, y + h), self.p(x, y + h)], col, a * self.alpha);
    }
    pub fn poly(&self, win: &mut Window, pts: &[(f32, f32)], col: C, a: f32) {
        let q: Vec<(f32, f32)> = pts.iter().map(|p| self.p(p.0, p.1)).collect();
        draw::poly(win, &q, col, a * self.alpha);
    }
    pub fn line(&self, win: &mut Window, pts: &[(f32, f32)], lw: f32, col: C, a: f32) {
        let q: Vec<(f32, f32)> = pts.iter().map(|p| self.p(p.0, p.1)).collect();
        draw::line(win, &q, lw, col, a * self.alpha);
    }
    pub fn ellipse(&self, win: &mut Window, cx: f32, cy: f32, rx: f32, ry: f32, col: C, a: f32) {
        self.poly(win, &ellipse_pts(cx, cy, rx, ry, 0.0, std::f32::consts::TAU, 28), col, a);
    }
}

pub fn ellipse_pts(cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32, n: usize) -> Vec<(f32, f32)> {
    (0..=n).map(|i| { let t = a0 + (a1 - a0) * i as f32 / n as f32; (cx + t.cos() * rx, cy + t.sin() * ry) }).collect()
}

/// A path: moveTo, lineTo and quadraticCurveTo, as points.
pub struct Path(pub Vec<(f32, f32)>);
impl Path {
    pub fn new(x: f32, y: f32) -> Self {
        Path(vec![(x, y)])
    }
    pub fn to(mut self, x: f32, y: f32) -> Self {
        self.0.push((x, y));
        self
    }
    pub fn quad(mut self, cx: f32, cy: f32, x: f32, y: f32) -> Self {
        let (x0, y0) = *self.0.last().unwrap_or(&(x, y));
        for k in 1..=10 {
            let u = k as f32 / 10.0;
            self.0.push(((1.0 - u) * (1.0 - u) * x0 + 2.0 * (1.0 - u) * u * cx + u * u * x, (1.0 - u) * (1.0 - u) * y0 + 2.0 * (1.0 - u) * u * cy + u * u * y));
        }
        self
    }
}

/// The part of a polygon between two horizontal lines (Sutherland–Hodgman, twice).
fn clip_band(pts: &[(f32, f32)], y0: f32, y1: f32) -> Vec<(f32, f32)> {
    let clip = |inp: &[(f32, f32)], inside: &dyn Fn(f32) -> bool, yc: f32| -> Vec<(f32, f32)> {
        let mut out = Vec::new();
        if inp.is_empty() {
            return out;
        }
        for i in 0..inp.len() {
            let (p, q) = (inp[i], inp[(i + 1) % inp.len()]);
            let (pi, qi) = (inside(p.1), inside(q.1));
            let cross = || { let t = (yc - p.1) / (q.1 - p.1); (p.0 + (q.0 - p.0) * t, yc) };
            if pi {
                out.push(p);
                if !qi { out.push(cross()) }
            } else if qi {
                out.push(cross());
            }
        }
        out
    };
    let a = clip(pts, &|y| y >= y0, y0);
    clip(&a, &|y| y <= y1, y1)
}

/// A shape (in the transform's own units) filled with createLinearGradient(0, g0, 0, g1) and these stops of
/// (offset, colour, alpha): cut into bands across its own vertical axis, each with the gradient's colour at its middle.
pub fn grad_fill(win: &mut Window, xf: &Xf, pts: &[(f32, f32)], g0: f32, g1: f32, stops: &[(f32, C, f32)], bands: usize) {
    let (ymin, ymax) = pts.iter().fold((f32::MAX, f32::MIN), |m, p| (m.0.min(p.1), m.1.max(p.1)));
    if ymax <= ymin {
        return;
    }
    let at = |y: f32| -> (C, f32) {
        let k = if (g1 - g0).abs() < 1e-6 { 0.0 } else { ((y - g0) / (g1 - g0)).clamp(0.0, 1.0) };
        let mut prev = stops[0];
        for s in stops {
            if k <= s.0 {
                let span = (s.0 - prev.0).max(1e-6);
                let u = ((k - prev.0) / span).clamp(0.0, 1.0);
                return (mix(prev.1, s.1, u), prev.2 + (s.2 - prev.2) * u);
            }
            prev = *s;
        }
        (prev.1, prev.2)
    };
    for b in 0..bands {
        let (y0, y1) = (ymin + (ymax - ymin) * b as f32 / bands as f32, ymin + (ymax - ymin) * (b + 1) as f32 / bands as f32);
        let part = clip_band(pts, y0, y1);
        if part.len() >= 3 {
            let (c, a) = at((y0 + y1) / 2.0);
            if a > 0.003 {
                xf.poly(win, &part, c, a);
            }
        }
    }
}
