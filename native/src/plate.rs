//! pulse-native 1.1: the painted picture behind the new worlds. Each world's stage (space and the Earth, the sea
//! floor, the server room, the Moon) is painted here pixel by pixel, the way a matte painting is: fractal noise for
//! land, clouds, rock and water, real lighting from the Sun's direction, atmosphere and depth fog, then the
//! structures with their shading, then the lights, and a dither so no gradient bands. It runs on a thread of its
//! own and only when the light or your data change (a minute at most for the turning Earth), never in a frame; the
//! frame lays the picture and draws what moves over it, at the places the picture gives back (`anchors`).

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};
use tiny_skia::{BlendMode, Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, RadialGradient, Rect, SpreadMode, Stroke, Transform};

pub type Rgb = [f32; 3]; // 0..1

// ------------------------------------------------------------------------------------------------ noise

fn hash(x: i32, y: i32, s: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6b343) ^ (y as u32).wrapping_mul(0xd8163841) ^ s.wrapping_mul(0xcb1ab31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1e995);
    h ^ (h >> 15)
}

fn grad(h: u32, x: f32, y: f32) -> f32 {
    let a = (h & 255) as f32 * (std::f32::consts::TAU / 256.0);
    a.cos() * x + a.sin() * y
}

/// Gradient noise, about -0.7..0.7.
pub fn noise(x: f32, y: f32, s: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (xf, yf) = (x - xi as f32, y - yi as f32);
    let f = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let (u, v) = (f(xf), f(yf));
    let n00 = grad(hash(xi, yi, s), xf, yf);
    let n10 = grad(hash(xi + 1, yi, s), xf - 1.0, yf);
    let n01 = grad(hash(xi, yi + 1, s), xf, yf - 1.0);
    let n11 = grad(hash(xi + 1, yi + 1, s), xf - 1.0, yf - 1.0);
    let a = n00 + u * (n10 - n00);
    let b = n01 + u * (n11 - n01);
    a + v * (b - a)
}

fn hash3(x: i32, y: i32, z: i32, s: u32) -> u32 {
    hash(x ^ z.wrapping_mul(0x27d4eb2d_u32 as i32), y.wrapping_add(z.wrapping_mul(374761393)), s)
}

const G3: [[f32; 3]; 12] = [[1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [1.0, 0.0, -1.0], [-1.0, 0.0, -1.0], [0.0, 1.0, 1.0], [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, -1.0, -1.0]];

/// 3D gradient noise (for a sphere's surface: no seams, no squeezed poles), about -1..1.
pub fn noise3(x: f32, y: f32, z: f32, s: u32) -> f32 {
    let (xi, yi, zi) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let (xf, yf, zf) = (x - xi as f32, y - yi as f32, z - zi as f32);
    let f = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let (u, v, w) = (f(xf), f(yf), f(zf));
    let g = |dx: i32, dy: i32, dz: i32| {
        let gv = G3[(hash3(xi + dx, yi + dy, zi + dz, s) % 12) as usize];
        gv[0] * (xf - dx as f32) + gv[1] * (yf - dy as f32) + gv[2] * (zf - dz as f32)
    };
    let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
    l(
        l(l(g(0, 0, 0), g(1, 0, 0), u), l(g(0, 1, 0), g(1, 1, 0), u), v),
        l(l(g(0, 0, 1), g(1, 0, 1), u), l(g(0, 1, 1), g(1, 1, 1), u), v),
        w,
    )
}

/// 3D fractal noise with as many octaves as the place deserves: `foot` is the size of a pixel there (in the
/// noise's units), so detail finer than a pixel is not computed (far and near get the same sharpness).
pub fn fbm3(p: [f32; 3], freq: f32, foot: f32, max_oct: u32, s: u32) -> f32 {
    fbm3p(p, freq, foot, max_oct, s, 0.52)
}

/// The same with its own persistence (how much each finer octave keeps: higher is rougher).
pub fn fbm3p(p: [f32; 3], freq: f32, foot: f32, max_oct: u32, s: u32, pers: f32) -> f32 {
    let (mut sum, mut amp, mut fq, mut norm) = (0.0, 1.0, freq, 0.0);
    for o in 0..max_oct {
        // fade the last octave in, so the cut is not seen
        let k = ((0.5 / (fq * foot.max(1e-6))) - 0.5).clamp(0.0, 1.0);
        if k <= 0.0 {
            break;
        }
        sum += noise3(p[0] * fq + o as f32 * 3.1, p[1] * fq - o as f32 * 1.7, p[2] * fq + o as f32 * 5.3, s.wrapping_add(o)) * amp * k;
        norm += amp;
        amp *= pers;
        fq *= 2.07;
    }
    if norm > 0.0 { sum / norm.max(1.0) * 1.3 } else { 0.0 }
}

/// Fractal noise, about -1..1.
pub fn fbm(x: f32, y: f32, oct: u32, s: u32) -> f32 {
    let (mut sum, mut amp, mut fq, mut norm) = (0.0, 1.0, 1.0, 0.0);
    for o in 0..oct {
        sum += noise(x * fq + o as f32 * 17.3, y * fq - o as f32 * 9.1, s.wrapping_add(o)) * amp;
        norm += amp;
        amp *= 0.5;
        fq *= 2.03;
    }
    (sum / norm) * 1.6
}

/// 2D fractal noise that stops at the pixel (`foot`: a pixel's size in the noise's units), about -1..1.
pub fn fbmf(x: f32, y: f32, freq: f32, foot: f32, max_oct: u32, s: u32, pers: f32) -> f32 {
    let (mut sum, mut amp, mut fq, mut norm) = (0.0, 1.0, freq, 0.0);
    for o in 0..max_oct {
        let k = ((0.5 / (fq * foot.max(1e-6))) - 0.5).clamp(0.0, 1.0);
        if k <= 0.0 {
            break;
        }
        sum += noise(x * fq + o as f32 * 17.3, y * fq - o as f32 * 9.1, s.wrapping_add(o)) * amp * k;
        norm += amp;
        amp *= pers;
        fq *= 2.03;
    }
    if norm > 0.0 { sum / norm.max(1.0) * 1.6 } else { 0.0 }
}

/// Ridged noise (mountain crests, rock veins), 0..1.
pub fn ridged(x: f32, y: f32, oct: u32, s: u32) -> f32 {
    let (mut sum, mut amp, mut fq, mut norm) = (0.0, 1.0, 1.0, 0.0);
    for o in 0..oct {
        let n = 1.0 - (noise(x * fq, y * fq, s.wrapping_add(o * 7)) * 1.4).abs();
        sum += n * n * amp;
        norm += amp;
        amp *= 0.5;
        fq *= 2.1;
    }
    sum / norm
}

pub fn rnd(i: u32, s: u32) -> f32 {
    (hash(i as i32, 7, s) & 0xffffff) as f32 / 16777216.0
}

pub fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

pub fn c8(r: u8, g: u8, b: u8) -> Rgb {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ------------------------------------------------------------------------------------------------ the canvas

/// A picture in light (linear-ish floats), painted per pixel, then handed to tiny-skia for the shapes.
pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub px: Vec<Rgb>,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        Canvas { w, h, px: vec![[0.0; 3]; w * h] }
    }

    /// Fills every pixel from a function of (x, y) in pixels, on all the cores.
    pub fn shade(&mut self, y0: usize, y1: usize, f: &(dyn Fn(f32, f32, Rgb) -> Rgb + Sync)) {
        let w = self.w;
        let (y0, y1) = (y0.min(self.h), y1.min(self.h));
        let rows = &mut self.px[y0 * w..y1 * w];
        let n = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).clamp(1, 8);
        let chunk = (rows.len() / w).div_ceil(n).max(1) * w;
        std::thread::scope(|sc| {
            for (k, part) in rows.chunks_mut(chunk).enumerate() {
                sc.spawn(move || {
                    for (i, p) in part.iter_mut().enumerate() {
                        let idx = k * chunk + i;
                        let (x, y) = ((idx % w) as f32 + 0.5, (y0 + idx / w) as f32 + 0.5);
                        *p = f(x, y, *p);
                    }
                });
            }
        });
    }

    /// Into tiny-skia, with a blue-noise-like dither of one step (no banding in the dark gradients).
    pub fn to_pixmap(&self) -> Option<Pixmap> {
        let mut pm = Pixmap::new(self.w as u32, self.h as u32)?;
        let data = pm.data_mut();
        for (i, p) in self.px.iter().enumerate() {
            let (x, y) = ((i % self.w) as i32, (i / self.w) as i32);
            let d = ((hash(x, y, 99) & 255) as f32 / 255.0 - 0.5) / 255.0 * 1.6;
            for c in 0..3 {
                // a gentle filmic shoulder: lights that pile up roll off instead of clipping flat
                let v = p[c].max(0.0);
                let v = if v > 0.8 { 0.8 + (1.0 - (-(v - 0.8) * 5.0).exp()) * 0.2 } else { v };
                data[i * 4 + c] = ((v + d).clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            data[i * 4 + 3] = 255;
        }
        Some(pm)
    }
}

// ------------------------------------------------------------------------------------------------ shapes

pub struct Sk<'a> {
    pub pm: &'a mut Pixmap,
}

fn color(c: Rgb, a: f32) -> Color {
    Color::from_rgba(c[0].clamp(0.0, 1.0), c[1].clamp(0.0, 1.0), c[2].clamp(0.0, 1.0), a.clamp(0.0, 1.0)).unwrap_or(Color::BLACK)
}

pub fn path(pts: &[(f32, f32)], close: bool) -> Option<tiny_skia::Path> {
    if pts.len() < 2 {
        return None;
    }
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        pb.line_to(p.0, p.1);
    }
    if close {
        pb.close();
    }
    pb.finish()
}

pub fn ellipse_pts(cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32, n: usize) -> Vec<(f32, f32)> {
    (0..=n).map(|i| { let a = a0 + (a1 - a0) * i as f32 / n as f32; (cx + a.cos() * rx, cy + a.sin() * ry) }).collect()
}

impl Sk<'_> {
    pub fn fill(&mut self, pts: &[(f32, f32)], c: Rgb, a: f32) {
        if let Some(p) = path(pts, true) {
            let mut pa = Paint::default();
            pa.set_color(color(c, a));
            pa.anti_alias = true;
            self.pm.fill_path(&p, &pa, FillRule::Winding, Transform::identity(), None);
        }
    }
    /// Filled with a gradient from (x0, y0) to (x1, y1) along the given stops.
    pub fn fill_lin(&mut self, pts: &[(f32, f32)], p0: (f32, f32), p1: (f32, f32), stops: &[(f32, Rgb, f32)]) {
        let st: Vec<GradientStop> = stops.iter().map(|s| GradientStop::new(s.0, color(s.1, s.2))).collect();
        if let (Some(p), Some(sh)) = (path(pts, true), LinearGradient::new(Point::from_xy(p0.0, p0.1), Point::from_xy(p1.0, p1.1), st, SpreadMode::Pad, Transform::identity())) {
            let pa = Paint { shader: sh, anti_alias: true, ..Default::default() };
            self.pm.fill_path(&p, &pa, FillRule::Winding, Transform::identity(), None);
        }
    }
    pub fn rect_lin(&mut self, x: f32, y: f32, w: f32, h: f32, p0: (f32, f32), p1: (f32, f32), stops: &[(f32, Rgb, f32)]) {
        self.fill_lin(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)], p0, p1, stops);
    }
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgb, a: f32) {
        if w > 0.0 && h > 0.0 {
            if let Some(r) = Rect::from_xywh(x, y, w, h) {
                let mut pa = Paint::default();
                pa.set_color(color(c, a));
                pa.anti_alias = true;
                self.pm.fill_rect(r, &pa, Transform::identity(), None);
            }
        }
    }
    pub fn stroke(&mut self, pts: &[(f32, f32)], lw: f32, c: Rgb, a: f32) {
        if let Some(p) = path(pts, false) {
            let mut pa = Paint::default();
            pa.set_color(color(c, a));
            pa.anti_alias = true;
            self.pm.stroke_path(&p, &pa, &Stroke { width: lw, line_cap: tiny_skia::LineCap::Round, ..Default::default() }, Transform::identity(), None);
        }
    }
    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, c: Rgb, a: f32) {
        self.fill(&ellipse_pts(cx, cy, r, r, 0.0, std::f32::consts::TAU, 48), c, a);
    }
    #[allow(dead_code)]
    /// A shaded ball: lit from `light` (a direction on screen), with a rim of reflected light.
    pub fn ball(&mut self, cx: f32, cy: f32, r: f32, base: Rgb, light: (f32, f32)) {
        let pts = ellipse_pts(cx, cy, r, r, 0.0, std::f32::consts::TAU, 64);
        let (hx, hy) = (cx + light.0 * r * 0.45, cy + light.1 * r * 0.45);
        let st = vec![
            GradientStop::new(0.0, color(mix(base, [1.0; 3], 0.45), 1.0)),
            GradientStop::new(0.55, color(base, 1.0)),
            GradientStop::new(1.0, color(mix(base, [0.0; 3], 0.75), 1.0)),
        ];
        if let (Some(p), Some(sh)) = (path(&pts, true), RadialGradient::new(Point::from_xy(hx, hy), Point::from_xy(cx, cy), r * 1.25, st, SpreadMode::Pad, Transform::identity())) {
            self.pm.fill_path(&p, &Paint { shader: sh, anti_alias: true, ..Default::default() }, FillRule::Winding, Transform::identity(), None);
        }
    }
    /// Light added (screen blend): a soft round glow.
    pub fn glow(&mut self, cx: f32, cy: f32, r: f32, c: Rgb, a: f32) {
        if r < 0.5 || a <= 0.0 {
            return;
        }
        let st = vec![GradientStop::new(0.0, color(c, a)), GradientStop::new(0.35, color(c, a * 0.35)), GradientStop::new(1.0, color(c, 0.0))];
        if let (Some(sh), Some(rc)) = (RadialGradient::new(Point::from_xy(cx, cy), Point::from_xy(cx, cy), r, st, SpreadMode::Pad, Transform::identity()), Rect::from_xywh(cx - r, cy - r, r * 2.0, r * 2.0)) {
            self.pm.fill_rect(rc, &Paint { shader: sh, anti_alias: false, blend_mode: BlendMode::Screen, ..Default::default() }, Transform::identity(), None);
        }
    }
    #[allow(dead_code)]
    /// A path filled with light added (windows, beams).
    pub fn fill_add(&mut self, pts: &[(f32, f32)], c: Rgb, a: f32) {
        if let Some(p) = path(pts, true) {
            let mut pa = Paint::default();
            pa.set_color(color(c, a));
            pa.anti_alias = true;
            pa.blend_mode = BlendMode::Screen;
            self.pm.fill_path(&p, &pa, FillRule::Winding, Transform::identity(), None);
        }
    }
}

// ------------------------------------------------------------------------------------------------ the plate

/// What a world's picture depends on (anything else changing does not repaint it).
#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    pub kind: String,
    pub w: u32,
    pub h: u32,
    /// the scale of the screen (device pixels per logical)
    pub sf: f32,
    /// the Sun's height where you are (degrees, rounded), the cloud cover, the Moon's lit part
    pub sun: i32,
    pub cloud: i32,
    pub moon: i32,
    pub waxing: bool,
    /// the turning Earth (a step a minute)
    pub turn: i64,
    /// the repos: name, CI, commits this week (rounded)
    pub repos: Vec<(String, String, i32)>,
    /// the palette's accent (the worlds take your theme's tint in their lights)
    pub accent: [u8; 3],
    pub hot: i32,
    pub cores: usize,
}

pub struct Plate {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    /// places in the picture (logical pixels) for what moves over it: "module" (x, y, size)...
    pub anchors: HashMap<&'static str, Vec<(f32, f32, f32)>>,
}

/// Paints the plate (on the caller's thread: call it from a thread of its own).
pub fn paint(a: &Ask) -> Option<Plate> {
    let (w, h) = (a.w.max(16) as usize, a.h.max(16) as usize);
    let mut anchors = HashMap::new();
    let pm = match a.kind.as_str() {
        "station" => crate::plates::station(a, w, h, &mut anchors),
        "abyss" => crate::plates::abyss(a, w, h, &mut anchors),
        "racks" => crate::plates::racks(a, w, h, &mut anchors),
        _ => crate::plates::moonbase(a, w, h, &mut anchors),
    }?;
    // tiny-skia keeps premultiplied colours; the picture is opaque, so they are the colours
    Some(Plate { w: w as u32, h: h as u32, rgba: pm.take(), anchors })
}

/// The plate of the current world: asked on a thread, laid when it is ready, the old one shown meanwhile.
#[derive(Default)]
pub struct Holder {
    pub ask: Option<Ask>,
    pending: Option<(Ask, Receiver<Option<Plate>>)>,
    pub img: Option<std::sync::Arc<gpui::RenderImage>>,
    pub anchors: HashMap<&'static str, Vec<(f32, f32, f32)>>,
    /// the old picture to give back to GPUI once replaced
    pub stale: Vec<std::sync::Arc<gpui::RenderImage>>,
}

impl Holder {
    /// Asks for a new plate when `want` differs from the one shown (and none is being painted); takes a finished
    /// one. Returns true when a new picture just arrived.
    pub fn update(&mut self, want: Ask) -> bool {
        let mut fresh = false;
        if let Some((ask, rx)) = &self.pending {
            if let Ok(res) = rx.try_recv() {
                if let Some(p) = res {
                    if let Some(im) = image::RgbaImage::from_raw(p.w, p.h, bgra(p.rgba)) {
                        if let Some(old) = self.img.replace(std::sync::Arc::new(gpui::RenderImage::new(vec![image::Frame::new(im)]))) {
                            self.stale.push(old);
                        }
                        self.anchors = p.anchors;
                        fresh = true;
                    }
                }
                self.ask = Some(ask.clone());
                self.pending = None;
            }
        }
        if self.pending.is_none() && self.ask.as_ref() != Some(&want) {
            let (tx, rx) = channel();
            let a = want.clone();
            if std::thread::Builder::new().name("pulse-plate".into()).spawn(move || { let _ = tx.send(paint(&a)); }).is_ok() {
                self.pending = Some((want, rx));
            }
        }
        fresh
    }
}

/// GPUI's images are BGRA.
fn bgra(mut v: Vec<u8>) -> Vec<u8> {
    for p in v.chunks_exact_mut(4) {
        p.swap(0, 2);
    }
    v
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn sstep(a: f32, b: f32, x: f32) -> f32 {
    smooth(a, b, x)
}
