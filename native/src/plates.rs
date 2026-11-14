//! pulse-native 1.1: the four new worlds' painted pictures (see plate.rs for how they are made and used).
//! Every place handed back in `anchors` is in logical pixels, for the frame that draws over the picture.

use crate::plate::*;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};
use tiny_skia::Pixmap;

type Anchors = HashMap<&'static str, Vec<(f32, f32, f32)>>;

fn put(an: &mut Anchors, k: &'static str, x: f32, y: f32, s: f32, sf: f32) {
    an.entry(k).or_default().push((x / sf, y / sf, s / sf));
}

fn ci_rgb(ci: &str) -> Rgb {
    match ci {
        "fail" => c8(255, 85, 85),
        "run" => c8(255, 196, 87),
        "ok" => c8(124, 227, 139),
        _ => c8(130, 200, 255),
    }
}

// ================================================================================================ the sky of space

/// Deep space: its gradient, the Milky Way with its dust lanes, faint nebulae, and the stars (in the canvas).
fn space(cv: &mut Canvas, y1: usize, seed: u32, tint: Rgb) {
    let (w, h) = (cv.w as f32, cv.h as f32);
    let sc = 3.2 / h;
    cv.shade(0, y1, &|x, y, _| {
        let base = mix(c8(3, 4, 12), c8(1, 1, 4), y / h);
        // the band of the Milky Way, across the sky
        let (nx, ny) = (x / w - 0.5, y / h - 0.35);
        let along = nx * 0.92 + ny * 0.38;
        let across = -nx * 0.38 + ny * 0.92 + 0.08 * fbm(along * 3.0, 0.0, 3, seed);
        let band = (-(across / 0.11).powi(2)).exp();
        let mut c = base;
        if band > 0.01 {
            let cloud = 0.5 + 0.5 * fbm(x * sc * 1.4, y * sc * 1.4, 6, seed + 1);
            let dust = ridged(x * sc * 2.2 + 3.0, y * sc * 2.2, 5, seed + 2);
            let lane = smooth_dust(dust, across);
            let core = (-(along + 0.12).powi(2) / 0.05).exp();
            let col = mix(c8(150, 160, 210), c8(255, 214, 170), core * 0.8);
            let k = band * cloud * (1.0 - lane * 0.85) * (0.10 + 0.14 * core);
            c = [c[0] + col[0] * k, c[1] + col[1] * k, c[2] + col[2] * k];
        }
        // faint nebulae in your theme's tint and a far red one
        let neb = (fbm(x * sc * 0.8 + 11.0, y * sc * 0.8, 5, seed + 3) - 0.35).max(0.0);
        let neb2 = (fbm(x * sc * 0.7 - 7.0, y * sc * 0.9, 5, seed + 4) - 0.45).max(0.0);
        let t2 = mix(tint, c8(90, 140, 255), 0.4);
        [c[0] + t2[0] * neb * 0.07 + 0.09 * neb2, c[1] + t2[1] * neb * 0.07 + 0.02 * neb2, c[2] + t2[2] * neb * 0.08 + 0.05 * neb2]
    });
    // the stars: many faint, a few bright with a soft halo; their colour from blue-white to amber
    let n = ((w * h) / 900.0) as u32;
    for i in 0..n {
        let (x, y) = (rnd(i, seed + 10) * w, rnd(i, seed + 11) * y1 as f32);
        let m = rnd(i, seed + 12).powf(7.0); // most are faint
        let t = rnd(i, seed + 13);
        let col = if t < 0.2 { c8(170, 195, 255) } else if t > 0.88 { c8(255, 210, 160) } else { c8(240, 240, 255) };
        let b = 0.25 + m * 2.2;
        let r = 0.6 + m * 2.4 * (h / 1080.0);
        splat(cv, x, y, r, col, b);
    }
}

fn smooth_dust(d: f32, across: f32) -> f32 {
    sstep(0.55, 0.85, d) * (-(across / 0.05).powi(2)).exp().max(0.3)
}

/// A soft point of light added to the canvas.
fn splat(cv: &mut Canvas, x: f32, y: f32, r: f32, c: Rgb, b: f32) {
    let rr = (r * 2.5).ceil() as i32;
    let (xi, yi) = (x as i32, y as i32);
    for dy in -rr..=rr {
        for dx in -rr..=rr {
            let (px, py) = (xi + dx, yi + dy);
            if px < 0 || py < 0 || px >= cv.w as i32 || py >= cv.h as i32 {
                continue;
            }
            let d2 = ((px as f32 + 0.5 - x).powi(2) + (py as f32 + 0.5 - y).powi(2)) / (r * r);
            let k = b * (-d2 * 1.6).exp();
            if k < 0.002 {
                continue;
            }
            let p = &mut cv.px[py as usize * cv.w + px as usize];
            for i in 0..3 {
                p[i] += c[i] * k;
            }
        }
    }
}

// ================================================================================================ a planet

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / l, v[1] / l, v[2] / l]
}

/// What the Earth's surface is at a point of the unit sphere (`foot`: the size of a pixel there, in the same units).
struct Ground {
    albedo: Rgb,
    water: f32,
    ice: f32,
    city: f32,
    cloud: f32,
    /// the cloud a little toward the Sun, for its shadow on the ground
    shadow: f32,
    /// the cloud's own relief toward the Sun (-1 in its shade .. 1 lit)
    relief: f32,
}

fn ground_at(p: [f32; 3], foot: f32, cover: f32, l: [f32; 3]) -> Ground {
    let seed = 77;
    // continents: big shapes plus coasts down to the pixel
    let hgt = fbm3(p, 5.0, foot, 12, seed) * 1.5 + 0.5 * fbm3(p, 1.6, foot, 3, seed + 9) + 0.02;
    let land = sstep(-0.005, 0.015, hgt);
    let lat = p[2].abs(); // the z axis is the Earth's axis
    let moist = fbm3(p, 8.0, foot, 7, seed + 1) * 1.5;
    let desert = sstep(0.1, 0.35, 1.0 - (lat - 0.4).abs() * 3.2) * sstep(-0.05, 0.3, -moist);
    let rough = fbm3p(p, 40.0, foot, 10, seed + 3, 0.6) * 1.1 + 0.5;
    let mut gr = mix(c8(52, 84, 42), c8(128, 124, 80), sstep(-0.3, 0.5, moist + hgt * 0.6));
    gr = mix(gr, c8(186, 150, 104), desert);
    gr = mix(gr, c8(150, 128, 104), sstep(0.25, 0.6, hgt) * 0.8); // mountains
    gr = mix(gr, c8(225, 228, 235), sstep(0.5, 0.75, hgt) * 0.7); // their snow
    gr = [gr[0] * (0.8 + 0.4 * rough), gr[1] * (0.8 + 0.4 * rough), gr[2] * (0.8 + 0.4 * rough)];
    let deep = mix(c8(6, 30, 80), c8(16, 70, 130), sstep(-0.5, 0.0, hgt));
    let ocean = mix(deep, c8(24, 120, 140), sstep(-0.03, 0.0, hgt) * 0.6);
    let ice = sstep(0.83, 0.9, lat + 0.05 * fbm3(p, 6.0, foot, 4, seed + 2));
    let mut albedo = mix(ocean, gr, land);
    albedo = mix(albedo, c8(238, 242, 250), ice);
    // clouds: big weather systems, streets of cumulus and their fine edges, as much as your sky has
    // (a big swirl warps them, as the weather turns them)
    let thr = lerp(0.75, -0.55, cover);
    let warp = [p[0] + 0.06 * fbm3(p, 6.0, foot, 4, seed + 20), p[1] + 0.06 * fbm3(p, 6.0, foot, 4, seed + 21), p[2] + 0.06 * fbm3(p, 6.0, foot, 4, seed + 22)];
    let cl_n = |q: [f32; 3]| fbm3p(q, 10.0, foot, 15, seed + 5, 0.64) * 2.2 + 0.25 * fbm3(q, 2.5, foot, 2, seed + 30);
    let cn = cl_n(warp);
    let cloud = sstep(thr, thr + 0.14, cn);
    let off = 0.004;
    let q = [warp[0] - l[0] * off, warp[1] - l[1] * off, warp[2] - l[2] * off];
    let towards = sstep(thr, thr + 0.14, cl_n(q));
    let shadow = towards * 0.55;
    // the cloud's relief: thicker toward the Sun means this side is in its own shade
    let relief = (cn - thr).clamp(0.0, 0.6) / 0.6;
    let city = if land > 0.5 {
        sstep(0.25, 0.6, fbm3(p, 180.0, foot, 6, seed + 7) * 1.5) * sstep(0.0, 0.4, fbm3(p, 14.0, foot, 4, seed + 8) * 1.5) * (1.0 - ice) * (1.0 - desert * 0.7)
    } else {
        0.0
    };
    Ground { albedo, water: 1.0 - land, ice, city, cloud, shadow, relief: (relief - (cl_n(q) - thr).clamp(0.0, 0.6) / 0.6).clamp(-1.0, 1.0) }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A point of the Earth lit: the ground under its clouds, the Sun's glint on the water, the night's cities, and
/// the air between it and the eye (`air`: 0 straight below, 1 at the horizon).
fn light_ground(gd: &Ground, n: [f32; 3], l: [f32; 3], v: [f32; 3], air: f32, night_lights: bool) -> Rgb {
    let ndl = dot(n, l);
    let diff = ndl.max(0.0);
    let day = sstep(-0.1, 0.15, ndl);
    let hv = norm3([l[0] + v[0], l[1] + v[1], l[2] + v[2]]);
    let glint = dot(n, hv).max(0.0).powf(400.0) * 1.6 + dot(n, hv).max(0.0).powf(40.0) * 0.06;
    let sh = 1.0 - gd.shadow * (1.0 - gd.cloud);
    let lit = diff * 1.35 * sh + 0.008;
    let mut c = [gd.albedo[0] * lit, gd.albedo[1] * lit, gd.albedo[2] * lit];
    let g = glint * gd.water * (1.0 - gd.ice) * (1.0 - gd.cloud) * day;
    c = [c[0] + g * 1.0, c[1] + g * 0.96, c[2] + g * 0.88];
    // the clouds: bright tops, a soft grey underside toward the terminator
    let cl_lit = (diff * 1.3 + 0.01) * (0.82 + 0.3 * gd.relief);
    let ccol = mix([cl_lit * 0.78, cl_lit * 0.82, cl_lit * 0.92], [cl_lit; 3], diff.sqrt() * (0.6 + 0.4 * gd.relief.max(0.0)));
    c = mix(c, ccol, gd.cloud * 0.95);
    if night_lights {
        let k = gd.city * (1.0 - day) * (1.0 - gd.cloud * 0.85) * 1.2;
        c = [c[0] + k, c[1] + 0.62 * k, c[2] + 0.25 * k];
    }
    // the air: blue haze by day, deeper toward the horizon; orange where the Sun is setting
    let dusk = (-(ndl / 0.14).powi(2)).exp();
    let haze = mix(c8(110, 160, 235), c8(255, 150, 80), dusk * 0.8);
    let k = air.powf(1.6) * (day * 0.85 + dusk * 0.4 + 0.03);
    mix(c, [haze[0] * (diff * 0.9 + 0.1 + dusk * 0.4), haze[1] * (diff * 0.9 + 0.1 + dusk * 0.3), haze[2] * (diff * 0.9 + 0.1 + dusk * 0.2)], k)
}

/// From orbit, as from a rocket's camera: the camera 400 km up looks toward the horizon; every pixel is a ray to
/// the real sphere, so the curvature, the foreshortening and the thin bright band of the atmosphere are true.
fn earth_from_orbit(cv: &mut Canvas, light: [f32; 3], turn: f32, cover: f32, horizon_y: f32) {
    let (w, h) = (cv.w as f32, cv.h as f32);
    let r = 1.0f32;
    let alt = 400.0 / 6371.0;
    let f = (h * 0.5) / (30.0f32).to_radians().tan(); // 60° of view up and down
    let dip = (r / (r + alt)).acos();
    let pitch = dip + ((horizon_y - h * 0.5) / f).atan();
    let (cp, sp) = (pitch.cos(), pitch.sin());
    let o = [0.0, r + alt, 0.0];
    // the Earth turns under us: a rotation about a tilted axis, and the Sun stays where it is
    let (ct, st) = (turn.cos(), turn.sin());
    let tilt: f32 = 0.6;
    let (cti, sti) = (tilt.cos(), tilt.sin());
    let to_earth = move |p: [f32; 3]| -> [f32; 3] {
        // into the Earth's own frame: turn about y, then tilt the axis
        let x = p[0] * ct + p[2] * st;
        let z = -p[0] * st + p[2] * ct;
        let y = p[1];
        [x, y * cti - z * sti, y * sti + z * cti]
    };
    let l = norm3(light);
    let y0 = (horizon_y - h * 0.08).max(0.0) as usize;
    cv.shade(y0, cv.h, &|x, y, p| {
        let dc = [(x - w * 0.5) / f, -(y - h * 0.5) / f, 1.0];
        let d = norm3([dc[0], dc[1] * cp - dc[2] * sp, dc[1] * sp + dc[2] * cp]);
        let b = dot(o, d);
        let c = dot(o, o) - r * r;
        let disc = b * b - c;
        if disc < 0.0 {
            // the ray misses: the band of the atmosphere above the limb (how low it passed)
            let tmin = -b;
            let q = [o[0] + d[0] * tmin, o[1] + d[1] * tmin, o[2] + d[2] * tmin];
            let hmin = (dot(q, q).sqrt() - r) / alt; // in units of our height
            let n = norm3(q);
            let sun = dot(n, l);
            let lit = sstep(-0.25, 0.2, sun);
            let k = (-hmin * 9.0).exp() * 1.4 + (-hmin * 2.5).exp() * 0.25;
            let col = mix(c8(40, 110, 255), c8(170, 210, 255), (-hmin * 20.0).exp());
            let dusk = (-(sun / 0.15).powi(2)).exp();
            let col = mix(col, c8(255, 130, 60), dusk * 0.7 * (-hmin * 14.0).exp());
            let a = k * (lit * 0.95 + 0.05);
            return [p[0] * (1.0 - a.min(1.0) * 0.6) + col[0] * a, p[1] * (1.0 - a.min(1.0) * 0.6) + col[1] * a, p[2] * (1.0 - a.min(1.0) * 0.6) + col[2] * a];
        }
        let t = -b - disc.sqrt();
        let q = [o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t];
        let n = norm3(q);
        // the size of a pixel on the ground: distance over focal, stretched by the grazing angle
        let graze = (-dot(d, n)).max(0.02);
        let foot = t / f / graze.sqrt();
        let pe = to_earth(n);
        let le = to_earth(l);
        let gd = ground_at(pe, foot, cover, le);
        let v = [-d[0], -d[1], -d[2]];
        let air = (1.0 - graze).powf(3.0);
        let mut c = light_ground(&gd, n, l, v, air, true);
        // the limb's own glow on the last pixels of ground
        let rim = (-graze * 18.0).exp() * sstep(-0.2, 0.2, dot(n, l));
        c = [c[0] + 0.15 * rim, c[1] + 0.3 * rim, c[2] + 0.6 * rim];
        c
    });
}

/// The Earth as a ball far away (from the Moon): the same surface, seen whole.
fn earth_globe(cv: &mut Canvas, cx: f32, cy: f32, r: f32, light: [f32; 3], turn: f32, cover: f32) {
    let l = norm3(light);
    let (ct, st) = (turn.cos(), turn.sin());
    let y0 = (cy - r * 1.3).max(0.0) as usize;
    let y1 = (cy + r * 1.3) as usize;
    cv.shade(y0, y1, &|x, y, p| {
        let (dx, dy) = ((x - cx) / r, (y - cy) / r);
        let d2 = dx * dx + dy * dy;
        if d2 >= 1.0 {
            let d = d2.sqrt();
            let k = (-(d - 1.0) / 0.05).exp();
            let lit = ((dx * l[0] - dy * l[1]) / d * 0.5 + 0.5 + l[2] * 0.4).clamp(0.0, 1.0);
            let a = k * lit * 0.8;
            return [p[0] + 0.3 * a, p[1] + 0.55 * a, p[2] + a];
        }
        let n = [dx, -dy, (1.0 - d2).sqrt()]; // y up
        let pe = [n[0] * ct + n[2] * st, n[1] * 0.92 + 0.0, -n[0] * st + n[2] * ct];
        let pe = [pe[0], pe[2], pe[1]]; // the axis up the screen
        // seen whole, the continents are bigger than from orbit: the same world at a wider scale
        let pe = [pe[0] * 0.35, pe[1] * 0.35, pe[2] * 0.35];
        let gd = ground_at(pe, 0.35 / r, cover, l);
        let air = (1.0 - n[2]).powf(2.0);
        light_ground(&gd, n, l, [0.0, 0.0, 1.0], air, true)
    });
}

// ================================================================================================ station

pub fn station(a: &Ask, w: usize, h: usize, an: &mut Anchors) -> Option<Pixmap> {
    let sf = a.sf;
    let (wf, hf) = (w as f32, h as f32);
    let mut cv = Canvas::new(w, h);
    let tint = [a.accent[0] as f32 / 255.0, a.accent[1] as f32 / 255.0, a.accent[2] as f32 / 255.0];
    space(&mut cv, h, 4242, tint);
    // the Earth below, as a rocket's camera sees it from orbit; lit by your Sun: day where you are is day under us
    // (the Sun ahead and a little to the right, high at noon, behind the Earth at night)
    let s = (a.sun as f32 / 60.0).clamp(-1.0, 1.0);
    let light = [0.45, s * 0.9 + 0.08, 0.7 - s.abs() * 0.3];
    let turn = a.turn as f32 * 0.0011;
    let horizon = hf * 0.4;
    earth_from_orbit(&mut cv, light, turn, 0.15 + a.cloud as f32 / 100.0 * 0.55, horizon);
    put(an, "limb", wf * 0.5, horizon, 0.0, sf);
    let mut pm = cv.to_pixmap()?;
    let mut sk = Sk { pm: &mut pm };

    // ---- the station: truss, radiators, solar wings, modules
    let (sx, sy) = (wf * 0.5, hf * 0.56);
    let half = wf * 0.22;
    let u = hf / 1080.0;
    let sun_side = if light[0] > 0.0 { 1.0 } else { -1.0 }; // light comes from the right
    let metal = |k: f32| mix(c8(70, 74, 82), c8(226, 228, 232), k);
    // the solar wings: four pairs, dark blue cells in a gold frame, the Sun's sheen across them
    for side in [-1.0f32, 1.0] {
        for k in 0..2 {
            let x0 = sx + side * half * (0.66 + k as f32 * 0.2);
            let mast = (x0, sy);
            for up in [-1.0f32, 1.0] {
                let (pw, ph) = (half * 0.16, hf * 0.17);
                // a slight perspective: the far edge narrower
                let (y_near, y_far) = (sy + up * hf * 0.012, sy + up * (hf * 0.012 + ph));
                let skew = side * pw * 0.08;
                let quad = [(x0 - pw / 2.0, y_near), (x0 + pw / 2.0, y_near), (x0 + pw / 2.0 * 0.86 + skew, y_far), (x0 - pw / 2.0 * 0.86 + skew, y_far)];
                sk.fill(&[(quad[0].0 - u * 2.0, quad[0].1), (quad[1].0 + u * 2.0, quad[1].1), (quad[2].0 + u * 2.0, quad[2].1 + up * u * 2.0), (quad[3].0 - u * 2.0, quad[3].1 + up * u * 2.0)], c8(150, 118, 60), 1.0);
                sk.fill_lin(&quad, (x0 - pw, y_near), (x0 + pw, y_far), &[(0.0, c8(18, 26, 74), 1.0), (0.45, c8(42, 52, 128), 1.0), (0.55, c8(70, 86, 170), 1.0), (1.0, c8(14, 20, 60), 1.0)]);
                // the cells: a fine grid
                let rows = 22;
                for r in 1..rows {
                    let t = r as f32 / rows as f32;
                    let (l0, r0) = (lerp_pt(quad[0], quad[3], t), lerp_pt(quad[1], quad[2], t));
                    sk.stroke(&[l0, r0], (u * 0.8).max(0.6), c8(120, 100, 60), 0.45);
                }
                for c in 1..6 {
                    let t = c as f32 / 6.0;
                    sk.stroke(&[lerp_pt(quad[0], quad[1], t), lerp_pt(quad[3], quad[2], t)], (u * 0.7).max(0.5), c8(120, 100, 60), 0.35);
                }
                // the Sun's sheen sweeping across the wing
                sk.fill_lin(&quad, (x0 - pw * 0.6, y_near), (x0 + pw * 0.6, y_far), &[(0.0, c8(255, 255, 255), 0.0), (0.48, c8(200, 220, 255), 0.0), (0.52, c8(220, 235, 255), 0.22), (0.58, c8(200, 220, 255), 0.0), (1.0, c8(255, 255, 255), 0.0)]);
                sk.stroke(&[mast, ((quad[2].0 + quad[3].0) / 2.0, y_far)], (u * 2.0).max(1.0), metal(0.6), 1.0);
            }
            put(an, "wing", x0, sy - hf * 0.09, hf * 0.08, sf);
        }
    }
    // the radiators: white fins below the truss, angled
    for side in [-1.0f32, 1.0] {
        for k in 0..3 {
            let x = sx + side * half * (0.3 + k as f32 * 0.06);
            let pts = [(x - half * 0.02, sy + hf * 0.01), (x + half * 0.02, sy + hf * 0.01), (x + half * 0.05, sy + hf * 0.09), (x + half * 0.01, sy + hf * 0.09)];
            sk.fill_lin(&pts, (x, sy), (x + half * 0.05, sy + hf * 0.09), &[(0.0, c8(235, 236, 238), 1.0), (1.0, c8(150, 154, 160), 1.0)]);
            for j in 1..6 {
                let t = j as f32 / 6.0;
                sk.stroke(&[lerp_pt(pts[0], pts[3], t), lerp_pt(pts[1], pts[2], t)], (u * 0.6).max(0.5), c8(120, 124, 130), 0.5);
            }
        }
    }
    // the truss: a box girder lit from above, with its lattice
    let th = hf * 0.016;
    sk.rect_lin(sx - half, sy - th / 2.0, half * 2.0, th, (0.0, sy - th / 2.0), (0.0, sy + th / 2.0), &[(0.0, metal(0.95), 1.0), (0.5, metal(0.62), 1.0), (1.0, metal(0.25), 1.0)]);
    let bays = 44;
    for k in 0..bays {
        let x = sx - half + k as f32 * half * 2.0 / bays as f32;
        let x2 = x + half * 2.0 / bays as f32;
        sk.stroke(&[(x, sy - th / 2.0), (x2, sy + th / 2.0)], (u * 1.0).max(0.7), metal(0.2), 0.7);
        sk.stroke(&[(x, sy - th / 2.0), (x, sy + th / 2.0)], (u * 0.8).max(0.6), metal(0.3), 0.6);
    }
    sk.stroke(&[(sx - half, sy - th / 2.0), (sx + half, sy - th / 2.0)], (u * 1.2).max(0.8), metal(1.0), 0.9);
    // the modules: one cylinder for each repo, as long as its week of commits, along the truss's middle
    let n = a.repos.len().max(1);
    let mw = (half * 0.95 / n as f32).min(hf * 0.11);
    for (i, (_, ci, week)) in a.repos.iter().enumerate() {
        let x = sx - (n as f32 * mw) / 2.0 + i as f32 * mw + mw / 2.0;
        let mh = hf * (0.04 + 0.025 * (*week as f32 / 60.0).clamp(0.0, 1.0));
        let (l, r) = (x - mw * 0.43, x + mw * 0.43);
        let (t, b) = (sy - mh / 2.0, sy + mh / 2.0);
        // the hull: a cylinder, light on its upper side, white thermal blankets with their quilting
        sk.rect_lin(l, t, r - l, b - t, (0.0, t), (0.0, b), &[(0.0, c8(150, 152, 158), 1.0), (0.18, c8(246, 246, 244), 1.0), (0.42, c8(226, 226, 222), 1.0), (0.85, c8(120, 122, 128), 1.0), (1.0, c8(70, 72, 80), 1.0)]);
        for q in 1..5 {
            let xx = l + (r - l) * q as f32 / 5.0;
            sk.stroke(&[(xx, t), (xx, b)], (u * 0.8).max(0.6), c8(150, 150, 150), 0.45);
        }
        // the end rings
        for xx in [l, r] {
            sk.fill_lin(&ellipse_pts(xx, sy, mw * 0.05, mh / 2.0, 0.0, TAU, 24), (0.0, t), (0.0, b), &[(0.0, metal(0.9), 1.0), (1.0, metal(0.3), 1.0)]);
        }
        // its lit band: the colour of its CI, through three portholes
        let cc = ci_rgb(ci);
        for wdw in 0..3 {
            let wx = l + (r - l) * (0.22 + wdw as f32 * 0.28);
            sk.disc(wx, sy - mh * 0.08, (mh * 0.12).max(1.5), c8(20, 24, 30), 1.0);
            sk.disc(wx, sy - mh * 0.08, (mh * 0.085).max(1.2), cc, 0.9);
            sk.glow(wx, sy - mh * 0.08, mh * 0.35, cc, 0.35);
        }
        put(an, "module", x, sy, mw * 0.5, sf);
    }
    // the docking port at the right end, the dish at the left, the alarm lamp on top
    put(an, "port", sx + half * 0.53, sy - hf * 0.012, hf * 0.02, sf);
    let (dx, dy) = (sx - half * 0.35, sy + hf * 0.05);
    sk.stroke(&[(dx, sy), (dx, dy)], (u * 2.0).max(1.0), metal(0.7), 1.0);
    let dish = ellipse_pts(dx, dy, hf * 0.022, hf * 0.013, 0.0, PI, 24);
    sk.fill_lin(&dish, (dx - hf * 0.02, dy), (dx + hf * 0.02, dy + hf * 0.013), &[(0.0, c8(250, 250, 250), 1.0), (1.0, c8(130, 134, 140), 1.0)]);
    put(an, "dish", dx, dy, hf * 0.02, sf);
    put(an, "lamp", sx, sy - hf * 0.03, hf * 0.01, sf);
    put(an, "truss", sx, sy, half, sf);
    // the Sun's light on the station's sunward ends, and a faint shadow on the far ones
    sk.glow(sx + sun_side * half * 0.9, sy - hf * 0.05, hf * 0.12, c8(255, 240, 220), 0.12 * (0.4 + s.max(0.0)));
    drop(sk);
    Some(pm)
}

fn lerp_pt(a: (f32, f32), b: (f32, f32), t: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

// ================================================================================================ abyss

pub fn abyss(a: &Ask, w: usize, h: usize, an: &mut Anchors) -> Option<Pixmap> {
    let sf = a.sf;
    let (wf, hf) = (w as f32, h as f32);
    let mut cv = Canvas::new(w, h);
    let day = sstep(-6.0, 12.0, a.sun as f32);
    let clouds = a.cloud as f32 / 100.0;
    let surf = hf * 0.07;
    let floor = hf * 0.8;
    let sc = 3.0 / hf;
    // the water: light from above fading into the deep, god rays through it, the surface's bright skin
    cv.shade(0, h, &|x, y, _| {
        let depth = (y / hf).clamp(0.0, 1.0);
        let top = mix(c8(4, 30, 54), c8(26, 120, 150), day);
        let mut c = mix(top, c8(1, 8, 18), depth.powf(0.7));
        if y < surf * 1.6 {
            // the surface seen from below: a rippled bright skin (Snell's window)
            let rip = 0.5 + 0.5 * fbm(x * sc * 3.0, y * sc * 12.0, 4, 3);
            let k = sstep(surf * 1.6, surf * 0.6, y) * (0.25 + 0.6 * day) * rip;
            c = mix(c, mix(c8(120, 190, 220), c8(220, 245, 255), day), k);
        }
        // god rays: the Sun through the waves, slanted, fewer under clouds and by night
        if day > 0.02 && y > surf * 0.8 {
            let slant = x / wf + (y / hf) * 0.18;
            let rays = (fbm(slant * 14.0, 0.0, 4, 5) * 0.5 + 0.5).powf(3.0) * 1.6;
            let fade = (1.0 - (y - surf) / (floor - surf)).clamp(0.0, 1.0).powf(1.4);
            let k = rays * fade * day * (1.0 - clouds * 0.6) * 0.16;
            c = [c[0] + 0.55 * k, c[1] + 0.85 * k, c[2] + 0.9 * k];
        }
        c
    });
    // the sea floor in perspective: rippled sand, stones, dark rock, caustics dancing on it by day, the blue taking
    // everything far away; rocky outcrops on the far floor
    let hz = hf * 0.64;
    let f = hf * 0.9;
    let cam = 3.0;
    let water_far = mix(c8(4, 34, 60), c8(18, 90, 120), day * 0.8);
    let outcrop: Vec<f32> = (0..w).map(|x| { let t = x as f32 / wf; hz - hf * 0.05 * ridged(t * 3.0, 1.3, 5, 140).powf(1.5) - hf * 0.01 * fbm(t * 9.0, 0.2, 3, 141) }).collect();
    cv.shade((hz - hf * 0.06) as usize, h, &|x, y, p| {
        let top = outcrop[(x as usize).min(w - 1)];
        if y < top {
            return p;
        }
        if y < hz {
            let k = fbm(x * 0.01, y * 0.02, 4, 142) * 0.5 + 0.5;
            let rock = mix(c8(20, 40, 52), c8(36, 62, 70), k);
            return mix(rock, water_far, 0.65 - 0.3 * (hz - y) / (hz - top + 1.0));
        }
        let dy = (y - hz).max(0.3);
        let z = cam * f / dy;
        let xw = (x - wf * 0.5) * z / f;
        let foot = z / f;
        // sand ripples across the current, warped
        let warp = fbmf(xw, z, 0.05, foot, 4, 143, 0.5) * 6.0;
        let rip = (((xw * 0.35 + z * 1.6 + warp) * 2.4).sin() * 0.5 + 0.5).powf(1.5);
        let ripk = (1.0 - (foot * 8.0).min(1.0)) * 0.12; // they fade when smaller than a pixel
        let grain = fbmf(xw, z, 0.6, foot, 7, 144, 0.6);
        let mut c = mix(c8(150, 140, 110), c8(110, 106, 90), grain * 0.5 + 0.5);
        let m = 1.0 - ripk + rip * ripk * 2.0;
        c = [c[0] * m, c[1] * m, c[2] * m];
        // stones and dark rock
        let st = fbmf(xw + 50.0, z, 0.25, foot, 6, 145, 0.55);
        c = mix(c, c8(60, 64, 62), sstep(0.35, 0.42, st));
        let rk = fbmf(xw - 30.0, z, 0.04, foot, 6, 146, 0.6);
        c = mix(c, c8(44, 52, 54), sstep(0.3, 0.38, rk));
        // the light that reaches the floor: blue-green, less in the depth; caustics by day
        let light = mix(c8(40, 90, 110), c8(150, 210, 210), day);
        c = [c[0] * light[0], c[1] * light[1], c[2] * light[2]];
        let ca = (1.0 - (noise(xw * 0.9, z * 0.9 + warp * 0.1, 50) * 2.2).abs()).powf(7.0) + (1.0 - (noise(xw * 1.4 + 3.0, z * 1.4, 51) * 2.2).abs()).powf(9.0);
        let k = ca * day * (1.0 - clouds * 0.5) * 0.35 * (1.0 - (foot * 10.0).min(1.0));
        c = [c[0] + k * 0.55, c[1] + k * 0.9, c[2] + k * 0.95];
        // the water between: everything far goes blue
        let fog = 1.0 - (-z / 26.0).exp();
        mix(c, water_far, fog)
    });
    let mut pm = cv.to_pixmap()?;
    let mut sk = Sk { pm: &mut pm };
    let u = hf / 1080.0;
    // kelp: tall ribbons with their blades, darker and bluer far away; corals and sponges by the base; boulders
    let persp = |y: f32| ((y - hf * 0.64) / (hf * 0.36)).clamp(0.05, 1.0); // 0 far .. 1 near
    let mut plants: Vec<(f32, f32, u32)> = (0..90u32).map(|i| (rnd(i, 61) * wf, hf * 0.66 + rnd(i, 62).powf(1.6) * hf * 0.34, i)).collect();
    plants.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    for (x, base_y, i) in plants {
        if (x - wf * 0.5).abs() < wf * 0.17 && base_y < hf * 0.86 {
            continue; // the base's clearing
        }
        let k = persp(base_y);
        let fog = 1.0 - k;
        let tall = hf * (0.12 + rnd(i, 63) * 0.3) * k;
        let lean = (rnd(i, 64) - 0.5) * tall * 0.5;
        let wid = (hf * 0.006 * k).max(1.0);
        let col0 = mix(c8(70, 110, 40), c8(120, 130, 50), rnd(i, 65));
        let col = mix(col0, mix(c8(4, 34, 60), c8(18, 90, 120), day * 0.8), fog * 0.8);
        let pts: Vec<(f32, f32)> = (0..=16).map(|q| { let t = q as f32 / 16.0; (x + lean * t * t + (t * 5.0 + i as f32).sin() * wid * 3.0 * t, base_y - tall * t) }).collect();
        // the stipe as a tapered ribbon
        let mut left = Vec::new();
        let mut right = Vec::new();
        for (q, p) in pts.iter().enumerate() {
            let t = q as f32 / 16.0;
            let ww = wid * (1.0 - t * 0.7);
            left.push((p.0 - ww, p.1));
            right.push((p.0 + ww, p.1));
        }
        right.reverse();
        left.extend(right);
        sk.fill_lin(&left, (x - wid, 0.0), (x + wid, 0.0), &[(0.0, mix(col, [0.0; 3], 0.4), 1.0), (0.5, col, 1.0), (1.0, mix(col, [0.0; 3], 0.5), 1.0)]);
        // the blades: long leaves along it, alternating, lit at their tips
        for q in (3..16).step_by(2) {
            let p = pts[q];
            let dir = if q % 4 == 3 { 1.0 } else { -1.0 };
            let len = tall * 0.16 * (1.0 - q as f32 / 20.0);
            let tip = (p.0 + dir * len, p.1 - len * 0.5);
            let leaf = [p, (p.0 + dir * len * 0.5, p.1 - len * 0.45), tip, (p.0 + dir * len * 0.45, p.1 - len * 0.05)];
            sk.fill_lin(&leaf, p, tip, &[(0.0, mix(col, [0.0; 3], 0.3), 0.95), (1.0, mix(col, c8(200, 230, 120), 0.35 * day * k), 0.85)]);
        }
    }
    // boulders half buried, shaded from above
    for i in 0..26u32 {
        let x = rnd(i, 81) * wf;
        let y = hf * 0.67 + rnd(i, 82).powf(1.4) * hf * 0.33;
        let k = persp(y);
        let r = hf * (0.012 + rnd(i, 83) * 0.03) * k;
        let pts: Vec<(f32, f32)> = (0..=24).map(|q| { let a2 = PI + PI * q as f32 / 24.0; let wob = 1.0 + 0.18 * noise(q as f32 * 0.5, i as f32, 84); (x + a2.cos() * r * 1.4 * wob, y + a2.sin() * r * wob) }).collect();
        let fogc = mix(c8(4, 34, 60), c8(18, 90, 120), day * 0.8);
        sk.fill_lin(&pts, (x, y - r), (x, y), &[(0.0, mix(c8(110, 120, 110), fogc, 1.0 - k), 1.0), (1.0, mix(c8(30, 36, 38), fogc, (1.0 - k) * 0.8), 1.0)]);
    }
    // corals: branching ones, fans and sponges, in reef colours, around the base
    let coral = [c8(255, 110, 100), c8(255, 165, 80), c8(190, 110, 255), c8(110, 220, 255), c8(255, 220, 120)];
    for i in 0..34u32 {
        let x = wf * (0.28 + rnd(i, 71) * 0.44);
        let y = hf * 0.8 + rnd(i, 72) * hf * 0.12;
        let k = persp(y);
        let c = mix(coral[(i % 5) as usize], c8(20, 40, 50), 0.25);
        let size = hf * (0.02 + rnd(i, 73) * 0.03) * k;
        match i % 3 {
            0 => {
                // a branching coral
                let mut tips = vec![(x, y, -PI / 2.0, size, 0)];
                while let Some((bx, by, ang, len, depth)) = tips.pop() {
                    let (ex, ey) = (bx + ang.cos() * len, by + ang.sin() * len);
                    sk.stroke(&[(bx, by), (ex, ey)], (size * 0.12 * (1.0 - depth as f32 * 0.25)).max(1.0), c, 1.0);
                    if depth < 3 {
                        tips.push((ex, ey, ang - 0.45, len * 0.7, depth + 1));
                        tips.push((ex, ey, ang + 0.4, len * 0.68, depth + 1));
                    } else {
                        sk.disc(ex, ey, (size * 0.07).max(1.0), mix(c, [1.0; 3], 0.4), 1.0);
                    }
                }
            }
            1 => {
                // a sea fan: a fine lattice in a half disc
                let fan = ellipse_pts(x, y, size * 1.1, size * 1.3, PI, TAU, 24);
                let mut closed = fan.clone();
                closed.push((x, y));
                sk.fill_lin(&closed, (x, y - size), (x, y), &[(0.0, mix(c, [1.0; 3], 0.2), 0.75), (1.0, c, 0.9)]);
                for q in 0..9 {
                    let a2 = PI + PI * (q as f32 + 0.5) / 9.0;
                    sk.stroke(&[(x, y), (x + a2.cos() * size * 1.1, y + a2.sin() * size * 1.3)], (size * 0.03).max(0.7), mix(c, [0.0; 3], 0.4), 0.7);
                }
            }
            _ => {
                // tube sponges
                for q in 0..3 {
                    let (tx, th2) = (x + (q as f32 - 1.0) * size * 0.35, size * (0.9 + 0.4 * rnd(i * 3 + q, 74)));
                    sk.rect_lin(tx - size * 0.13, y - th2, size * 0.26, th2, (tx - size * 0.13, 0.0), (tx + size * 0.13, 0.0), &[(0.0, mix(c, [0.0; 3], 0.4), 1.0), (0.5, c, 1.0), (1.0, mix(c, [0.0; 3], 0.5), 1.0)]);
                    sk.fill(&ellipse_pts(tx, y - th2, size * 0.13, size * 0.05, 0.0, TAU, 12), mix(c, [0.0; 3], 0.7), 1.0);
                }
            }
        }
    }
    // the base: a glass dome for each repo, lit from inside by its CI
    let n = a.repos.len().max(1);
    for (i, (_, ci, week)) in a.repos.iter().enumerate() {
        let x = wf * 0.5 + (i as f32 - (n as f32 - 1.0) / 2.0) * wf * 0.07;
        let r = hf * (0.04 + 0.02 * (*week as f32 / 60.0).clamp(0.0, 1.0));
        let cc = ci_rgb(ci);
        sk.glow(x, floor - r * 0.3, r * 2.6, cc, 0.35);
        let dome = ellipse_pts(x, floor, r * 1.2, r, PI, TAU, 40);
        sk.fill_lin(&dome, (x, floor - r), (x, floor), &[(0.0, mix(c8(120, 200, 220), cc, 0.3), 0.55), (1.0, mix(c8(20, 60, 80), cc, 0.4), 0.75)]);
        // its inside: floors and lights
        for f in 0..2 {
            let yy = floor - r * (0.25 + f as f32 * 0.35);
            sk.rect(x - r * 0.8, yy, r * 1.6, r * 0.05, c8(30, 40, 50), 0.8);
            for k in 0..5 {
                sk.glow(x - r * 0.6 + k as f32 * r * 0.3, yy - r * 0.1, r * 0.18, cc, 0.5);
            }
        }
        // the frame and the glass's reflection
        for k in 0..5 {
            let aa = PI + PI * (k as f32 + 1.0) / 6.0;
            sk.stroke(&[(x, floor - r), (x + aa.cos() * r * 1.2, floor + aa.sin() * r)], (u * 1.2).max(0.8), c8(160, 200, 210), 0.5);
        }
        sk.stroke(&dome, (u * 2.0).max(1.0), c8(190, 225, 235), 0.9);
        sk.stroke(&ellipse_pts(x - r * 0.25, floor - r * 0.1, r * 0.75, r * 0.7, PI * 1.15, PI * 1.45, 12), (u * 3.0).max(1.0), c8(255, 255, 255), 0.35);
        sk.rect_lin(x - r * 1.25, floor - u * 3.0, r * 2.5, u * 8.0, (0.0, floor - u * 3.0), (0.0, floor + u * 5.0), &[(0.0, c8(150, 160, 170), 1.0), (1.0, c8(50, 60, 70), 1.0)]);
        put(an, "dome", x, floor - r * 0.5, r, sf);
    }
    put(an, "floor", wf * 0.5, floor, 0.0, sf);
    put(an, "surface", wf * 0.5, surf, 0.0, sf);
    drop(sk);
    Some(pm)
}

// ================================================================================================ racks

pub fn racks(a: &Ask, w: usize, h: usize, an: &mut Anchors) -> Option<Pixmap> {
    let sf = a.sf;
    let (wf, hf) = (w as f32, h as f32);
    let hot = (a.hot as f32 / 100.0).clamp(0.0, 1.0);
    // a real room, in metres: the camera stands in the cold aisle, 1.4 m up, looking down it
    let hz = hf * 0.6;
    let f = hf * 0.9;
    let cam_y = 1.4;
    let proj = |x: f32, y: f32, z: f32| -> (f32, f32) { (wf * 0.5 + x * f / z, hz + (cam_y - y) * f / z) };
    let aisle = 1.0; // the rack fronts at x = ±1
    let rack_w = 0.6;
    let rack_h = 2.0;
    let z0 = 1.6;
    let per = a.cores.div_ceil(2).clamp(8, 16);
    let z_end = z0 + per as f32 * rack_w;
    let mut cv = Canvas::new(w, h);
    // the room's air: dark, a little warmer when your machine is hot
    cv.shade(0, h, &|x, y, _| {
        let n = fbm(x * 0.004, y * 0.004, 3, 9) * 0.01;
        let c = mix(c8(14, 17, 23), c8(8, 10, 14), ((y - hz).abs() / hf * 2.0).clamp(0.0, 1.0));
        [c[0] + n + hot * 0.06, c[1] + n, c[2] + n]
    });
    let mut pm = cv.to_pixmap()?;
    let mut sk = Sk { pm: &mut pm };
    let u = hf / 1080.0;
    let quad = |sk: &mut Sk, p: [(f32, f32); 4], stops: &[(f32, Rgb, f32)], vertical: bool| {
        let (a0, a1) = if vertical { (p[0], p[3]) } else { (p[0], p[1]) };
        sk.fill_lin(&p, a0, a1, stops);
    };
    // ---- the ceiling: tiles, rows of light troffers, the cable ladders over the racks
    let ceil = 3.0;
    for k in 0..40 {
        let z = z0 * 0.5 + k as f32 * 0.6;
        let (l, r) = (proj(-3.0, ceil, z), proj(3.0, ceil, z));
        if l.1 > -hf {
            sk.stroke(&[l, r], (u * 1.0).max(0.7), c8(40, 46, 56), 0.6);
        }
    }
    for k in -5..=5 {
        let x = k as f32 * 0.6;
        sk.stroke(&[proj(x, ceil, 0.8), proj(x, ceil, 40.0)], (u * 1.0).max(0.7), c8(40, 46, 56), 0.5);
    }
    for k in 0..14 {
        let z = z0 + 0.6 + k as f32 * 1.2;
        for x in [-0.6f32, 0.6] {
            let p = [proj(x - 0.3, ceil, z), proj(x + 0.3, ceil, z), proj(x + 0.3, ceil, z + 0.6), proj(x - 0.3, ceil, z + 0.6)];
            if p[0].1 < hf * 0.27 {
                continue; // none behind the clock
            }
            sk.fill(&p, c8(210, 222, 240), 0.55);
            let c = ((p[0].0 + p[2].0) / 2.0, (p[0].1 + p[2].1) / 2.0);
            sk.glow(c.0, c.1, (p[1].0 - p[0].0).abs() * 1.2, c8(180, 205, 240), 0.1);
        }
    }
    // the cable ladders above each row, full of bundled cables
    for side in [-1.0f32, 1.0] {
        let x = side * (aisle + 0.3);
        let y = 2.45;
        for (k, col) in [c8(250, 200, 50), c8(70, 160, 255), c8(240, 240, 240), c8(255, 120, 60), c8(120, 220, 120), c8(190, 110, 255)].iter().enumerate() {
            let off = (k as f32 - 2.5) * 0.04;
            sk.stroke(&[proj(x + off, y + 0.04, z0 - 0.5), proj(x + off, y + 0.04, z_end + 1.0)], (u * 3.0).max(1.0), mix(*col, c8(20, 20, 20), 0.4), 0.95);
        }
        for rail in [-0.22f32, 0.22] {
            sk.stroke(&[proj(x + rail, y, z0 - 0.5), proj(x + rail, y, z_end + 1.0)], (u * 3.0).max(1.2), c8(140, 146, 156), 1.0);
        }
        for k in 0..((z_end - z0) / 0.3) as i32 + 4 {
            let z = z0 - 0.5 + k as f32 * 0.3;
            sk.stroke(&[proj(x - 0.22, y, z), proj(x + 0.22, y, z)], (u * 1.5).max(0.8), c8(110, 116, 126), 0.8);
        }
        put(an, "tray", proj(x, y, z_end).0, proj(x, y, z_end).1, 0.0, sf);
        put(an, "tray", proj(x, y, z0 - 0.3).0, proj(x, y, z0 - 0.3).1, 0.0, sf);
    }
    // ---- the raised floor: grey tiles, the perforated ones in the aisle breathing cold air, a polished sheen
    let floor_q = [proj(-3.0, 0.0, 0.6), proj(3.0, 0.0, 0.6), proj(3.0, 0.0, 40.0), proj(-3.0, 0.0, 40.0)];
    quad(&mut sk, floor_q, &[(0.0, c8(44, 48, 56), 1.0), (1.0, c8(20, 22, 28), 1.0)], true);
    for k in 0..60 {
        let z = 0.6 + k as f32 * 0.6;
        sk.stroke(&[proj(-3.0, 0.0, z), proj(3.0, 0.0, z)], (u * 1.0).max(0.6), c8(14, 16, 20), 0.8);
    }
    for k in -5..=5 {
        sk.stroke(&[proj(k as f32 * 0.6, 0.0, 0.6), proj(k as f32 * 0.6, 0.0, 40.0)], (u * 1.0).max(0.6), c8(14, 16, 20), 0.8);
    }
    for k in 0..(per * 2) {
        let z = z0 + k as f32 * 0.6;
        if k % 2 == 1 {
            continue;
        }
        for x0 in [-0.6f32, 0.0] {
            let p = [proj(x0 + 0.03, 0.0, z + 0.03), proj(x0 + 0.57, 0.0, z + 0.03), proj(x0 + 0.57, 0.0, z + 0.57), proj(x0 + 0.03, 0.0, z + 0.57)];
            sk.fill(&p, c8(90, 140, 200), 0.18);
            // the holes, as a fine grid where they are big enough to see
            if (p[1].0 - p[0].0).abs() > 40.0 * u {
                for i in 1..8 {
                    for j in 1..8 {
                        let (s, t) = (i as f32 / 8.0, j as f32 / 8.0);
                        let q = proj(x0 + 0.03 + s * 0.54, 0.0, z + 0.03 + t * 0.54);
                        sk.disc(q.0, q.1, (u * 1.3 * f / (z * 600.0)).max(0.5), c8(10, 14, 20), 0.8);
                    }
                }
            }
        }
    }
    // ---- the far end: the cooling units on the wall, the exit sign, the containment's glass doors
    let wall_z = z_end + 2.5;
    let wq = [proj(-3.0, 3.0, wall_z), proj(3.0, 3.0, wall_z), proj(3.0, 0.0, wall_z), proj(-3.0, 0.0, wall_z)];
    quad(&mut sk, wq, &[(0.0, c8(40, 44, 52), 1.0), (1.0, c8(26, 28, 34), 1.0)], true);
    for cx in [-1.9f32, 1.9] {
        let q = [proj(cx - 0.8, 2.1, wall_z - 0.05), proj(cx + 0.8, 2.1, wall_z - 0.05), proj(cx + 0.8, 0.0, wall_z - 0.05), proj(cx - 0.8, 0.0, wall_z - 0.05)];
        quad(&mut sk, q, &[(0.0, c8(200, 204, 210), 1.0), (1.0, c8(130, 134, 142), 1.0)], true);
        for k in 0..12 {
            let y = 0.3 + k as f32 * 0.14;
            sk.stroke(&[proj(cx - 0.7, y, wall_z - 0.06), proj(cx + 0.7, y, wall_z - 0.06)], (u * 1.2).max(0.7), c8(90, 94, 100), 0.8);
        }
        let d = proj(cx + 0.5, 1.8, wall_z - 0.06);
        sk.rect(d.0 - 6.0 * u, d.1 - 4.0 * u, 12.0 * u, 8.0 * u, c8(20, 30, 40), 1.0);
        sk.glow(d.0, d.1, 6.0 * u, c8(90, 200, 255), 0.6);
    }
    let ex = proj(0.0, 2.6, wall_z - 0.05);
    let exs = 0.5 * f / wall_z;
    sk.rect(ex.0 - exs * 0.5, ex.1 - exs * 0.15, exs, exs * 0.3, c8(30, 200, 90), 1.0);
    sk.glow(ex.0, ex.1, exs, c8(60, 255, 140), 0.25);
    // the glass doors that close the cold aisle, and its glass roof
    let door = [proj(-aisle, 2.1, z_end), proj(aisle, 2.1, z_end), proj(aisle, 0.0, z_end), proj(-aisle, 0.0, z_end)];
    sk.fill(&door, c8(120, 170, 220), 0.12);
    sk.stroke(&[door[0], door[1], door[2], door[3], door[0]], (u * 3.0 * f / z_end / 120.0).max(1.0), c8(150, 156, 166), 0.9);
    sk.stroke(&[proj(0.0, 2.1, z_end), proj(0.0, 0.0, z_end)], (u * 2.0).max(1.0), c8(150, 156, 166), 0.8);
    for k in 0..per {
        let z = z0 + k as f32 * rack_w;
        let roof = [proj(-aisle, 2.1, z), proj(aisle, 2.1, z), proj(aisle, 2.1, z + rack_w), proj(-aisle, 2.1, z + rack_w)];
        sk.fill(&roof, c8(150, 190, 230), 0.07);
        sk.stroke(&[roof[0], roof[1]], (u * 1.5).max(0.8), c8(130, 136, 146), 0.6);
    }
    sk.stroke(&[proj(-aisle, 2.1, z0 - 0.3), proj(-aisle, 2.1, z_end)], (u * 2.0).max(1.0), c8(140, 146, 156), 0.8);
    sk.stroke(&[proj(aisle, 2.1, z0 - 0.3), proj(aisle, 2.1, z_end)], (u * 2.0).max(1.0), c8(140, 146, 156), 0.8);
    // the screen of the room on the doors: what the console shows is drawn live over it
    let (s0, s1) = (proj(-0.62, 1.95, z_end - 0.02), proj(0.62, 1.2, z_end - 0.02));
    sk.rect_lin(s0.0 - 4.0 * u, s0.1 - 4.0 * u, s1.0 - s0.0 + 8.0 * u, s1.1 - s0.1 + 8.0 * u, s0, (s0.0, s1.1), &[(0.0, c8(60, 64, 72), 1.0), (1.0, c8(20, 22, 26), 1.0)]);
    sk.rect(s0.0, s0.1, s1.0 - s0.0, s1.1 - s0.1, c8(4, 8, 14), 1.0);
    sk.glow((s0.0 + s1.0) / 2.0, (s0.1 + s1.1) / 2.0, s1.0 - s0.0, c8(90, 160, 255), 0.12);
    put(an, "console", s0.0, s0.1, s1.0 - s0.0, sf);
    an.entry("console_h").or_default().push(((s1.1 - s0.1) / sf, 0.0, 0.0));
    // a camera in the corner, watching
    let cam = proj(-aisle - 0.1, 2.7, z0 + 0.4);
    sk.rect(cam.0 - 14.0 * u, cam.1 - 6.0 * u, 28.0 * u, 12.0 * u, c8(220, 222, 226), 1.0);
    sk.disc(cam.0 + 12.0 * u, cam.1, 4.0 * u, c8(20, 20, 24), 1.0);
    put(an, "camera", cam.0 + 12.0 * u, cam.1, 3.0 * u, sf);
    // ---- the racks: two rows, far to near, each one filled with real equipment
    let mut order: Vec<usize> = (0..per * 2).collect();
    order.sort_by(|a2, b| (b / 2).cmp(&(a2 / 2)));
    let mut leds: Vec<(usize, (f32, f32, f32), (f32, f32, f32))> = Vec::new();
    let mut ports: Vec<(f32, f32, f32)> = Vec::new();
    for i in order {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let k = i / 2;
        let (za, zb) = (z0 + k as f32 * rack_w, z0 + (k + 1) as f32 * rack_w);
        let x = side * aisle;
        // a point on the rack's front: t from the top (0) to the bottom (1), s along the row (0 near, 1 far)
        let pt = |t: f32, s: f32| proj(x, rack_h * (1.0 - t), za + (zb - za) * s);
        let face = [pt(0.0, 0.0), pt(0.0, 1.0), pt(1.0, 1.0), pt(1.0, 0.0)];
        // the cabinet: dark frame; its side toward us shows at the row's near end
        sk.fill(&face, c8(18, 20, 26), 1.0);
        if k == 0 {
            let side_q = [proj(x, rack_h, za), proj(x + side * 1.0, rack_h, za), proj(x + side * 1.0, 0.0, za), proj(x, 0.0, za)];
            quad(&mut sk, side_q, &[(0.0, c8(40, 44, 52), 1.0), (1.0, c8(22, 24, 30), 1.0)], false);
            // a label plate on its end
            let lp = proj(x + side * 0.5, 1.7, za - 0.001);
            let ls = 0.05 * f / za;
            sk.rect(lp.0 - ls, lp.1 - ls * 0.3, ls * 2.0, ls * 0.6, c8(200, 204, 210), 0.6);
        }
        // the equipment, unit by unit (42U): servers, storage shelves full of drives, switches with their ports
        // and cables, patch panels, blanking plates
        let mut uu = 1usize;
        let mut slot = 0u32;
        while uu < 41 {
            let r = rnd(i as u32 * 97 + slot, 900);
            let (kind, size) = if slot == 0 { (2u8, 1usize) } else if slot == 1 { (3, 1) } else if r < 0.42 { (0, 1) } else if r < 0.62 { (0, 2) } else if r < 0.78 { (1, 3) } else if r < 0.86 { (2, 1) } else if r < 0.92 { (3, 1) } else { (4, 1) };
            let size = size.min(41 - uu);
            let (t0, t1) = (uu as f32 / 42.0, (uu + size) as f32 / 42.0 - 0.004);
            let unit = [pt(t0, 0.03), pt(t0, 0.97), pt(t1, 0.97), pt(t1, 0.03)];
            let px_h = (unit[3].1 - unit[0].1).abs();
            match kind {
                0 => {
                    // a server: a bezel, its drive bays, the power LED at the far end
                    let shade = 0.6 + 0.4 * rnd(i as u32 * 31 + slot, 5);
                    quad(&mut sk, unit, &[(0.0, mix(c8(86, 92, 104), c8(120, 126, 138), shade), 1.0), (1.0, c8(40, 44, 52), 1.0)], true);
                    if px_h > 2.5 {
                        let bays = if size == 2 { 12 } else { 8 };
                        for b in 0..bays {
                            let s0 = 0.06 + b as f32 * (0.78 / bays as f32);
                            let q = [pt(t0 + 0.15 / 42.0, s0), pt(t0 + 0.15 / 42.0, s0 + 0.7 / bays as f32), pt(t1 - 0.12 / 42.0, s0 + 0.7 / bays as f32), pt(t1 - 0.12 / 42.0, s0)];
                            sk.fill(&q, c8(22, 24, 30), 0.95);
                        }
                    }
                    let l = pt((t0 + t1) / 2.0, 0.92);
                    leds.push((i, (l.0, l.1, (px_h * 0.18).clamp(0.8, (4.0 * u).max(0.8))), (0.0, 0.0, 0.0)));
                }
                1 => {
                    // a storage shelf: three rows of drives, each with its little blue light
                    quad(&mut sk, unit, &[(0.0, c8(60, 64, 74), 1.0), (1.0, c8(26, 28, 34), 1.0)], true);
                    for row in 0..3 {
                        for b in 0..12 {
                            let (s0, tt0) = (0.05 + b as f32 * 0.075, t0 + (row as f32 + 0.15) / 42.0);
                            let q = [pt(tt0, s0), pt(tt0, s0 + 0.065), pt(tt0 + 0.75 / 42.0, s0 + 0.065), pt(tt0 + 0.75 / 42.0, s0)];
                            sk.fill(&q, c8(150, 156, 168), 0.9);
                            let led = pt(tt0 + 0.6 / 42.0, s0 + 0.055);
                            sk.disc(led.0, led.1, (px_h * 0.025).clamp(0.5, (2.0 * u).max(0.5)), c8(80, 170, 255), 0.9);
                        }
                    }
                    let l = pt((t0 + t1) / 2.0, 0.95);
                    leds.push((i, (l.0, l.1, (px_h * 0.06).clamp(0.8, (4.0 * u).max(0.8))), (0.0, 0.0, 0.0)));
                }
                2 => {
                    // a switch: two rows of ports, cables leaving to the side in bundles
                    quad(&mut sk, unit, &[(0.0, c8(40, 44, 52), 1.0), (1.0, c8(16, 18, 22), 1.0)], true);
                    let cols = [c8(250, 200, 50), c8(70, 160, 255), c8(255, 120, 60), c8(120, 220, 120)];
                    for row in 0..2 {
                        for b in 0..24 {
                            let s0 = 0.08 + b as f32 * 0.035;
                            let tt0 = t0 + (0.12 + row as f32 * 0.42) / 42.0;
                            let q = [pt(tt0, s0), pt(tt0, s0 + 0.026), pt(tt0 + 0.32 / 42.0, s0 + 0.026), pt(tt0 + 0.32 / 42.0, s0)];
                            sk.fill(&q, c8(6, 8, 10), 1.0);
                            if k < 5 && ports.len() < 600 && b % 2 == 0 {
                                let lp = pt(tt0 + 0.05 / 42.0, s0 + 0.013);
                                ports.push((lp.0, lp.1, (px_h * 0.05).clamp(0.5, (2.0 * u).max(0.5))));
                            }
                        }
                    }
                    // the cables, drooping toward the cable manager at the near edge
                    for c in 0..6 {
                        let s0 = 0.1 + c as f32 * 0.12;
                        let from = pt(t0 + 0.6 / 42.0, s0);
                        let to = pt(t1 + 1.5 / 42.0, 0.02);
                        let mid = ((from.0 + to.0) / 2.0, from.1.max(to.1) + px_h * 0.8);
                        sk.stroke(&[from, mid, to], (px_h * 0.12).clamp(0.6, (3.0 * u).max(0.6)), cols[c % 4], 0.85);
                    }
                }
                3 => {
                    // a patch panel: a row of ports, short colourful patch cords
                    quad(&mut sk, unit, &[(0.0, c8(30, 32, 38), 1.0), (1.0, c8(14, 16, 20), 1.0)], true);
                    for b in 0..24 {
                        let s0 = 0.06 + b as f32 * 0.037;
                        let p0 = pt(t0 + 0.3 / 42.0, s0 + 0.012);
                        let col = [c8(250, 200, 50), c8(70, 160, 255), c8(240, 240, 240), c8(255, 120, 60), c8(200, 120, 255)][(b + slot as usize) % 5];
                        sk.stroke(&[p0, pt(t1 + 0.4 / 42.0, s0 + 0.01)], (px_h * 0.1).clamp(0.5, (2.0 * u).max(0.5)), col, 0.9);
                    }
                }
                _ => {
                    // a blanking plate with its vents
                    quad(&mut sk, unit, &[(0.0, c8(30, 32, 38), 1.0), (1.0, c8(20, 22, 26), 1.0)], true);
                }
            }
            uu += size;
            slot += 1;
        }
        // the cabinet's posts and its top light bar
        sk.stroke(&[pt(0.0, 0.0), pt(1.0, 0.0)], (u * 2.0 * f / za / 300.0).max(1.0), c8(60, 64, 72), 1.0);
        sk.stroke(&[pt(0.0, 1.0), pt(1.0, 1.0)], (u * 1.0).max(0.7), c8(10, 12, 16), 1.0);
        let lb = [pt(0.0, 0.0), pt(0.0, 1.0), pt(0.5 / 42.0, 1.0), pt(0.5 / 42.0, 0.0)];
        sk.fill(&lb, c8(120, 200, 255), 0.5);
        // its reflection on the polished floor
        let refl = [pt(1.0, 0.0), pt(1.0, 1.0), proj(x, -0.45, zb), proj(x, -0.45, za)];
        sk.fill_lin(&refl, refl[0], refl[3], &[(0.0, c8(80, 100, 130), 0.28), (1.0, c8(80, 100, 130), 0.0)]);
        let mid = pt(0.5, 0.5);
        an.entry("rack_core").or_default().push(((i % a.cores.max(1)) as f32, 0.0, 0.0));
        put(an, "rack", mid.0, mid.1, (face[1].0 - face[0].0).abs() / 2.0, sf);
        an.entry("rack_z").or_default().push(((1.0 - (za - z0) / (z_end - z0)).clamp(0.1, 1.0), pt(1.0, 0.5).1 / sf, pt(0.0, 0.5).1 / sf));
    }
    // the LEDs: each rack's to its core (the frame lights them with that core's real load)
    for (rack, l, _) in leds {
        put(an, "led", l.0, l.1, l.2, sf);
        an.entry("led_core").or_default().push(((rack % a.cores.max(1)) as f32, rack as f32 + l.1 * 0.01, 0.0));
    }
    for p in ports {
        put(an, "port", p.0, p.1, p.2, sf);
    }
    put(an, "aisle", wf * 0.5, hz, 0.0, sf);
    drop(sk);
    Some(pm)
}

// ================================================================================================ moonbase

pub fn moonbase(a: &Ask, w: usize, h: usize, an: &mut Anchors) -> Option<Pixmap> {
    let sf = a.sf;
    let (wf, hf) = (w as f32, h as f32);
    let hz = hf * 0.66;
    let mut cv = Canvas::new(w, h);
    let tint = [a.accent[0] as f32 / 255.0, a.accent[1] as f32 / 255.0, a.accent[2] as f32 / 255.0];
    space(&mut cv, (hz + hf * 0.06) as usize, 777, tint);
    // the Earth in its real phase: lit as much as the Moon is dark in your sky
    let earth_lit = 1.0 - a.moon as f32 / 100.0;
    let (ex, ey, er) = (wf * 0.64, hf * 0.5, hf * 0.06);
    // the phase angle: 0 full (light from us), π new (light from behind)
    let ph = (2.0 * earth_lit - 1.0).clamp(-1.0, 1.0).acos(); // the lit part is (1 + cos ph) / 2
    let side = if a.waxing { -1.0 } else { 1.0 };
    let light = [side * ph.sin(), 0.05, ph.cos()];
    earth_globe(&mut cv, ex, ey, er, light, a.turn as f32 * 0.004, a.cloud as f32 / 100.0);
    put(an, "earth", ex, ey, er, sf);
    // the ground: grey regolith seen in perspective (every pixel a point on the plain), its craters and boulders lit
    // low by the Sun so every bump has its shadow; far mountains on the horizon
    let sun = sstep(-10.0, 25.0, a.sun as f32) * 0.75 + 0.25;
    let sun_x = if a.sun > 0 { 1.0 } else { -1.0 };
    let el = (8.0f32 + (a.sun.max(0) as f32) * 0.5).to_radians();
    let l = norm3([sun_x * el.cos() * 0.8, el.sin(), el.cos() * 0.6]);
    let ridge: Vec<f32> = (0..w).map(|x| { let t = x as f32 / wf; hz - hf * 0.045 * ridged(t * 2.4, 0.7, 6, 102).powf(1.4) - hf * 0.01 * fbm(t * 6.0, 0.3, 4, 101) }).collect();
    let span = (w / 160).max(4);
    let slope: Vec<f32> = (0..w).map(|x| { let (a2, b2) = (x.saturating_sub(span), (x + span).min(w - 1)); (ridge[a2] - ridge[b2]) / ((b2 - a2).max(1) as f32) }).collect();
    let f = hf * 0.9;
    let cam = 1.6; // metres above the ground
    let regolith = |x: f32, y: f32| -> Rgb {
        let dy = (y - hz).max(0.3);
        let z = cam * f / dy; // the distance on the plain
        let xw = (x - wf * 0.5) * z / f;
        let foot = z / f;
        // gentle swells, craters of every size (the big ones far apart), a fine grain of dust and pebbles
        let near_fade = sstep(4.0, 12.0, z);
        let hgt = |a2: f32, b2: f32| fbmf(a2, b2, 0.012, foot, 4, 103, 0.5) * 6.0
            + (crater_field(a2 * 0.012, b2 * 0.012) * 9.0 + crater_field(a2 * 0.05 + 7.0, b2 * 0.05) * 2.5) * near_fade
            + crater_field(a2 * 0.25 + 3.0, b2 * 0.25) * 0.45
            + fbmf(a2, b2, 0.8, foot, 6, 107, 0.5) * 0.08;
        let e = foot.max(0.02);
        let (h0, hx, hz2) = (hgt(xw, z), hgt(xw + e, z), hgt(xw, z + e));
        let n = norm3([-(hx - h0) / e, 1.0, -(hz2 - h0) / e]);
        let ndl = (n[0] * l[0] + n[1] * l[1] + n[2] * l[2]).max(0.0);
        // the bright rays around fresh craters, darker old lava
        let alb = 0.34 + 0.06 * fbmf(xw, z, 0.004, foot, 4, 104, 0.5) + 0.025 * fbmf(xw, z, 0.3, foot, 6, 105, 0.6);
        let v = alb * (ndl * 1.25 + 0.03) * sun;
        let far = (z / 900.0).min(1.0);
        let v = v * (1.0 - far * 0.35);
        [v, v, v * 1.04]
    };
    cv.shade(0, h, &|x, y, p| {
        let top = ridge[(x as usize).min(w - 1)];
        if y < top {
            return p;
        }
        if y < hz {
            // the far mountains: lit on the Sun's side of each crest
            // the far mountains: each slope lit by how it faces the Sun, rock texture, darker low down
            let sl = slope[(x as usize).min(w - 1)] * sun_x;
            let rough = 0.75 + 0.25 * fbm(x * 0.03, y * 0.06, 4, 106);
            let down = ((y - top) / (hz - top + 1.0)).clamp(0.0, 1.0);
            let v = (0.22 + sl * 1.6).clamp(0.05, 0.5) * sun * rough * (1.0 - down * 0.35);
            return [v, v, v * 1.05];
        }
        regolith(x, y)
    });
    let mut pm = cv.to_pixmap()?;
    let mut sk = Sk { pm: &mut pm };
    let u = hf / 1080.0;
    let grey = |k: f32| -> Rgb { [0.42 * k * sun, 0.42 * k * sun, 0.44 * k * sun] };
    // the solar field: rows of panels catching the Sun
    for k in 0..6 {
        let x = wf * (0.25 + k as f32 * 0.032);
        let y = hz + hf * 0.12;
        let q = [(x, y), (x + wf * 0.028, y - hf * 0.012), (x + wf * 0.028, y + hf * 0.012), (x, y + hf * 0.022)];
        sk.fill_lin(&q, q[0], q[2], &[(0.0, c8(20, 30, 90), 1.0), (0.5, mix(c8(40, 60, 150), c8(200, 210, 255), sun * 0.4), 1.0), (1.0, c8(16, 22, 70), 1.0)]);
        sk.stroke(&[(x + wf * 0.014, y + hf * 0.005), (x + wf * 0.014, y + hf * 0.03)], (u * 1.5).max(1.0), grey(1.5), 1.0);
        sk.fill(&[(x, y + hf * 0.022), (x + wf * 0.028, y + hf * 0.012), (x + wf * 0.04, y + hf * 0.03), (x + wf * 0.012, y + hf * 0.04)], [0.0; 3], 0.35); // its shadow
    }
    put(an, "solar", wf * 0.33, hz + hf * 0.125, wf * 0.1, sf);
    // the habitat: a dome for each repo, white panels, lit windows in its CI's colour
    let n = a.repos.len().max(1);
    for (i, (_, ci, week)) in a.repos.iter().enumerate() {
        let x = wf * 0.5 + (i as f32 - (n as f32 - 1.0) / 2.0) * wf * 0.075;
        let y = hz + hf * 0.07;
        let r = hf * (0.04 + 0.02 * (*week as f32 / 60.0).clamp(0.0, 1.0));
        let cc = ci_rgb(ci);
        sk.fill(&ellipse_pts(x + sun_x * -r * 0.8, y + r * 0.08, r * 1.9, r * 0.22, 0.0, TAU, 32), [0.0; 3], 0.45);
        let dome = ellipse_pts(x, y, r * 1.2, r, PI, TAU, 40);
        let mut closed = dome.clone();
        closed.push((x + r * 1.2, y));
        sk.fill_lin(&closed, (x - sun_x * r, y - r), (x + sun_x * r, y), &[(0.0, grey(0.9), 1.0), (0.5, grey(1.9), 1.0), (1.0, grey(2.3), 1.0)]);
        for k in 1..5 {
            let aa = PI + PI * k as f32 / 5.0;
            sk.stroke(&[(x, y - r), (x + aa.cos() * r * 1.2, y + aa.sin() * r)], (u * 1.0).max(0.7), grey(1.2), 0.6);
        }
        for k in 0..4 {
            let wx = x - r * 0.6 + k as f32 * r * 0.4;
            sk.rect(wx, y - r * 0.35, r * 0.2, r * 0.16, cc, 0.95);
            sk.glow(wx + r * 0.1, y - r * 0.27, r * 0.45, cc, 0.4);
        }
        sk.rect_lin(x - r * 1.25, y - u * 2.0, r * 2.5, u * 6.0, (0.0, y), (0.0, y + u * 6.0), &[(0.0, grey(2.0), 1.0), (1.0, grey(0.8), 1.0)]);
        put(an, "dome", x, y - r * 0.5, r, sf);
    }
    // the antenna, its dish toward the Earth; the lander's pad
    let (bx, by) = (wf * 0.4, hz - hf * 0.06);
    sk.stroke(&[(bx, hz + hf * 0.045), (bx, by)], (u * 2.0).max(1.0), grey(1.8), 1.0);
    let dish = ellipse_pts(bx, by, hf * 0.028, hf * 0.016, -0.6, PI - 0.6, 24);
    sk.fill_lin(&dish, (bx - hf * 0.02, by), (bx + hf * 0.02, by + hf * 0.01), &[(0.0, grey(2.4), 1.0), (1.0, grey(1.0), 1.0)]);
    put(an, "antenna", bx, by, hf * 0.02, sf);
    let (lx, ly) = (wf * 0.72, hz + hf * 0.09);
    sk.fill(&ellipse_pts(lx, ly + hf * 0.005, hf * 0.06, hf * 0.01, 0.0, TAU, 32), grey(0.7), 1.0);
    sk.stroke(&ellipse_pts(lx, ly + hf * 0.005, hf * 0.045, hf * 0.007, 0.0, TAU, 32), (u * 1.5).max(1.0), c8(255, 200, 90), 0.5);
    put(an, "pad", lx, ly, hf * 0.03, sf);
    put(an, "horizon", wf * 0.5, hz, 0.0, sf);
    drop(sk);
    Some(pm)
}

/// Craters: rings with a raised rim and a hollow, scattered by a grid of cells.
fn crater_field(x: f32, y: f32) -> f32 {
    let (cx, cy) = (x.floor(), y.floor());
    let mut v = 0.0;
    for oy in -1..=1 {
        for ox in -1..=1 {
            let (gx, gy) = (cx as i32 + ox, cy as i32 + oy);
            let hsh = (gx.wrapping_mul(73856093) ^ gy.wrapping_mul(19349663)) as u32;
            if hsh % 3 != 0 {
                continue;
            }
            let px = gx as f32 + ((hsh >> 4) & 255) as f32 / 255.0;
            let py = gy as f32 + ((hsh >> 12) & 255) as f32 / 255.0;
            let r = 0.15 + ((hsh >> 20) & 255) as f32 / 255.0 * 0.35;
            let d = ((x - px).powi(2) + (y - py).powi(2)).sqrt() / r;
            if d < 1.4 {
                // a bowl inside, a rim at 1, fading out
                v += if d < 1.0 { 0.25 + (d * d - 1.0) * 0.6 } else { let k = (1.4 - d) / 0.4; 0.25 * k * k };
            }
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    /// PLATE_OUT=dir cargo test --release preview -- --ignored: the four pictures as PNG files, to look at.
    #[test]
    #[ignore]
    fn preview() {
        let dir = std::env::var("PLATE_OUT").unwrap_or_else(|_| ".".into());
        let (w, h) = (std::env::var("PLATE_W").ok().and_then(|v| v.parse().ok()).unwrap_or(1600u32), std::env::var("PLATE_H").ok().and_then(|v| v.parse().ok()).unwrap_or(900u32));
        let sun: i32 = std::env::var("PLATE_SUN").ok().and_then(|v| v.parse().ok()).unwrap_or(35);
        let only = std::env::var("PLATE_ONLY").unwrap_or_default();
        for k in ["station", "abyss", "racks", "moonbase"] {
            if !only.is_empty() && only != k {
                continue;
            }
            let a = Ask { kind: k.into(), w, h, sf: 1.0, sun, cloud: 45, moon: 30, waxing: true, turn: 120, repos: vec![("pulse".into(), "ok".into(), 34), ("omarchytv".into(), "fail".into(), 12), ("website".into(), "ok".into(), 5), ("dotfiles".into(), "run".into(), 20)], accent: [120, 200, 255], hot: 40, cores: 16 };
            let t0 = std::time::Instant::now();
            let p = crate::plate::paint(&a).unwrap();
            eprintln!("{k}: {:?}", t0.elapsed());
            image::save_buffer(format!("{dir}/plate-{k}.png"), &p.rgba, p.w, p.h, image::ExtendedColorType::Rgba8).unwrap();
        }
    }
    #[test]
    fn plates_paint() {
        // a normal small screen and a tiny one (every size scaled by the height must still make sense)
        for (k, w, h) in ["station", "abyss", "racks", "moonbase"].iter().flat_map(|k| [(*k, 320u32, 180u32), (*k, 64, 36)]) {
            let a = Ask { kind: k.into(), w, h, sf: 1.0, sun: 20, cloud: 40, moon: 60, waxing: true, turn: 3, repos: vec![("a".into(), "ok".into(), 10), ("b".into(), "fail".into(), 40)], accent: [120, 200, 255], hot: 50, cores: 8 };
            let p = crate::plate::paint(&a).expect(k);
            assert_eq!(p.rgba.len(), (w * h * 4) as usize);
            assert!(!p.anchors.is_empty(), "{k}");
        }
    }
}
