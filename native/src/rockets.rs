//! A real launch as Pulse 1.8 draws it, translated from pulse.js drawLiftoffsFine() and drawRocketBody(): a rocket you
//! can recognise climbs from the left of your city on a curving path, smoke rolls off the pad, its trail widens and
//! fades, the flame with its shock diamonds, the plume lit by a Sun already set for you at twilight, the booster let go
//! at stage separation, and a tag with the mission and the time since liftoff.

use crate::draw;
use crate::palette::{mix, Pal, C};
use crate::scene::radial;
use crate::xf::{grad_fill, Path, Xf};
use gpui::Window;

pub struct Liftoff {
    t: f32,
    name: String,
    big: bool,
    trail: Vec<(f32, f32, f32)>,
    sep: Option<(f32, f32, f32, f32)>,
}

#[derive(Default)]
pub struct Rockets {
    all: Vec<Liftoff>,
}

/// drawRocketBody(): nose up, the base at 0; stage 0 the whole rocket, 1 the booster alone, 2 the second stage alone.
fn body(win: &mut Window, xf: &Xf, pal: &Pal, big: bool, rw: f32, rh: f32, stage: u8) {
    let white = mix(pal.fg, [1.0, 1.0, 1.0], 0.5);
    let dark = mix(pal.bg, [0.0; 3], 0.3);
    let steel = mix(mix(pal.fg, pal.bg, 0.25), pal.accent, 0.12);
    let b = if big { steel } else { white };
    let (y0, y1) = (if stage == 2 { -rh * 0.42 } else { 0.0 }, if stage == 1 { -rh * 0.58 } else { -rh });
    if stage != 2 {
        // the booster: engines, legs, the interstage and its grid fins
        xf.rect(win, -rw / 2.0, -rh * 0.58, rw, rh * 0.58, b, 1.0);
        xf.rect(win, -rw / 2.0, -rh * 0.05, rw, rh * 0.05, dark, 1.0);
        xf.rect(win, -rw / 2.0, -rh * 0.62, rw, rh * 0.06, dark, 1.0);
        if big {
            xf.rect(win, -rw * 0.95, -rh * 0.16, rw * 0.45, rh * 0.14, dark, 1.0);
            xf.rect(win, rw * 0.5, -rh * 0.16, rw * 0.45, rh * 0.14, dark, 1.0);
        } else {
            xf.poly(win, &[(-rw / 2.0, -rh * 0.14), (-rw * 0.85, 0.0), (-rw / 2.0, 0.0)], dark, 1.0);
            xf.poly(win, &[(rw / 2.0, -rh * 0.14), (rw * 0.85, 0.0), (rw / 2.0, 0.0)], dark, 1.0);
            xf.rect(win, -rw * 0.95, -rh * 0.57, rw * 0.45, (rh * 0.012).max(1.0), dark, 1.0);
            xf.rect(win, rw * 0.5, -rh * 0.57, rw * 0.45, (rh * 0.012).max(1.0), dark, 1.0);
        }
    }
    if stage != 1 {
        // the second stage and what it carries
        let fw = if big { rw } else { rw * 1.18 };
        xf.rect(win, -rw / 2.0, -rh * 0.78, rw, rh * 0.20, b, 1.0);
        let nose = Path::new(-fw / 2.0, -rh * 0.78).to(-fw / 2.0, -rh * 0.9).quad(-fw * 0.3, -rh * 1.0, 0.0, -rh * 1.02).quad(fw * 0.3, -rh * 1.0, fw / 2.0, -rh * 0.9).to(fw / 2.0, -rh * 0.78).0;
        xf.poly(win, &nose, b, 1.0);
        if big {
            xf.rect(win, -rw * 0.95, -rh * 0.9, rw * 0.45, rh * 0.09, dark, 1.0);
            xf.rect(win, rw * 0.5, -rh * 0.9, rw * 0.45, rh * 0.09, dark, 1.0);
        }
        if stage == 2 {
            xf.rect(win, -rw * 0.28, -rh * 0.58, rw * 0.56, rh * 0.05, dark, 1.0); // its one engine
        }
    }
    xf.rect(win, 0.0, y1, rw / 2.0, y0 - y1, [0.0; 3], 0.22); // the side away from the light
}

impl Rockets {
    pub fn launch(&mut self, t: f32, name: String, big: bool) {
        self.all.push(Liftoff { t, name, big, trail: Vec::new(), sep: None });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(&mut self, win: &mut Window, cx: &mut gpui::App, font: &gpui::SharedString, pal: &Pal, ox: f32, oy: f32, w: f32, h: f32, t: f32, band: (f32, f32), sky: [f32; 3], still: bool, share: bool) {
        let hz = oy + h * 0.745;
        let u1 = h / 1080.0;
        let tw = sky[1].max(sky[0] * 0.6);
        self.all.retain(|l| t - l.t <= 16.0);
        for l in self.all.iter_mut() {
            let age = t - l.t;
            let big = if l.big { 1.5 } else { 1.0 };
            let x0 = ox + band.0 + (band.1 - band.0) * 0.14;
            let climb = (age / 15.0).min(1.0);
            let y = hz - h * 0.9 * climb.powf(1.6);
            let x = x0 + w * 0.3 * climb.powf(2.4);
            let ang = (w * 0.3 * 2.4 * climb.powf(1.4)).atan2(h * 0.9 * 1.6 * climb.powf(0.6) + 1e-3);
            let (rw, rh) = (h * 0.0095 * big, h * 0.082 * big);
            let sep = age > 9.5;
            let fl = if still { 1.0 } else { 0.85 + 0.15 * (t * 43.0).sin() * (t * 17.0).sin() };
            l.trail.push((x, y, t));
            if l.trail.len() > 150 {
                l.trail.remove(0);
            }
            if sep && l.sep.is_none() {
                l.sep = Some((x, y, ang, t));
            }
            // smoke rolling off the pad
            if age < 7.0 {
                for k in 0..9 {
                    let sa = (age / 3.0).min(1.0);
                    let sd = (k as f32 - 4.0) / 4.0;
                    let sr = h * (0.012 + 0.03 * sa) * (1.2 - sd.abs() * 0.5) * big;
                    let (sx, sy) = (x0 + sd * h * 0.07 * sa * big, hz - sr * 0.45);
                    radial(win, sx, sy, 0.0, sr, 1.0, mix(pal.fg, pal.warn, if age < 1.5 { 0.5 } else { 0.1 }), &[(0.0, 0.34 * (1.0 - age / 7.0)), (1.0, 0.0)]);
                }
            }
            // the trail: smoke that widens and fades with its age, lit near the engine
            let mut k = 2;
            while k < l.trail.len() {
                let p = l.trail[k];
                let ta = t - p.2;
                let r = h * (0.004 + 0.011 * (ta / 5.0).min(1.0)) * big;
                let c = if ta < 0.5 { mix(pal.warn, pal.fg, 0.4) } else { mix(pal.fg, pal.accent2, 0.25) };
                draw::arc_fill(win, p.0 + (k as f32 * 1.7).sin() * r * 0.3, p.1, r, r, 0.0, 0.0, std::f32::consts::TAU, c, (if ta < 0.5 { 0.5 } else { 0.17 }) * (1.0 - ta / 9.0).max(0.0));
                k += 2;
            }
            if age < 1.6 {
                radial(win, x0, hz, 0.0, h * 0.2 * big, 1.0, pal.warn, &[(0.0, 0.8 * (1.0 - age / 1.6)), (1.0, 0.0)]);
            }
            if tw > 0.2 && climb > 0.5 {
                // twilight: the Sun, already set for you, still lights the plume up there
                let jk = (climb - 0.5) / 0.5;
                let jr = h * (0.05 + 0.2 * jk) * big;
                let c0 = Xf::at(x, y).rotate(ang).p(0.0, jr * 0.35 * 1.7);
                radial(win, c0.0, c0.1, 0.0, jr, 1.7, mix(pal.fg, pal.accent, 0.4), &[(0.0, 0.30 * tw * (1.0 - jk * 0.4)), (0.6, 0.10 * tw), (1.0, 0.0)]);
            }
            let xf = Xf::at(x, y).rotate(ang);
            // the flame, under whatever stage is burning
            let by = if sep { -rh * 0.42 } else { 0.0 };
            let fl2 = (if sep { 0.55 } else { 1.0 }) * fl;
            let fh = rh * if sep { 0.5 } else { 0.95 } * fl2;
            let wide = if sep { 2.2 } else { 1.1 };
            let flame = Path::new(-rw * 0.42, by).quad(-rw * wide, by + fh * 0.5, 0.0, by + fh).quad(rw * wide, by + fh * 0.5, rw * 0.42, by).0;
            let (c0, c1, c2) = if sep { (mix(pal.accent, pal.fg, 0.6), pal.accent, pal.accent) } else { (pal.fg, pal.warn, pal.crit) };
            grad_fill(win, &xf, &flame, by, by + fh, &[(0.0, c0, 0.95), (0.25, c1, 0.8), (1.0, c2, 0.0)], 10);
            if !sep {
                for k in 1..=3 {
                    // shock diamonds
                    let kf = k as f32;
                    xf.ellipse(win, 0.0, by + fh * 0.17 * kf, rw * 0.3 / kf.sqrt(), rw * 0.2, pal.fg, 0.75 * fl / kf);
                }
            }
            let hc = xf.p(0.0, by + rw);
            radial(win, hc.0, hc.1, 0.0, rh * 0.8, 1.0, if sep { pal.accent } else { pal.warn }, &[(0.0, 0.45 * fl), (1.0, 0.0)]);
            body(win, &xf, pal, l.big, rw, rh, if sep { 2 } else { 0 });
            if let Some((sx, sy, sang, st)) = l.sep {
                // the booster, let go: it coasts up a little, turns, and falls back
                let sa2 = t - st;
                let (bx, bys) = (sx + w * 0.012 * sa2, sy - h * 0.03 * sa2 + h * 0.012 * sa2 * sa2);
                let bxf = Xf::at(bx, bys).rotate(sang + sa2 * 0.35).with_alpha((1.0 - sa2 / 6.5).max(0.0));
                body(win, &bxf, pal, l.big, rw, rh, 1);
                if sa2 < 0.8 {
                    let pc = bxf.p(0.0, -rh * 0.6);
                    let pr = rw * (1.0 + sa2 * 5.0);
                    draw::arc_fill(win, pc.0, pc.1, pr, pr, 0.0, 0.0, std::f32::consts::TAU, pal.fg, 0.5 * (1.0 - sa2 / 0.8) * bxf.alpha);
                }
            }
            // its tag: the name, and the time since liftoff
            if age < 13.0 && !share {
                let (f1, f2) = ((h * 0.0148).round(), (h * 0.0122).round());
                let al = ((13.0 - age) / 2.0).min(1.0);
                let l1: String = (if l.name.is_empty() { "liftoff".to_string() } else { l.name.clone() }).to_uppercase().chars().take(40).collect();
                let sec = age.floor() as i32;
                let l2 = format!("T+00:{sec:02}{}", if sep { " · stage separation" } else if age > 6.0 { " · max Q" } else { " · liftoff" });
                let (w1, w2) = (draw::measure(win, font, &l1, f1, 700.0), draw::measure(win, font, &l2, f2, 500.0));
                let (pw, ph) = (w1.max(w2) + h * 0.02, f1 + f2 * 1.45 + h * 0.012);
                let px0 = (ox + w - pw - h * 0.02).min(x + h * 0.045);
                let py0 = (oy + h * 0.02).max(y - rh * 0.6);
                draw::line(win, &[(x + rw, y - rh * 0.4), (px0, py0 + ph / 2.0)], u1.max(1.0), pal.fg, 0.35 * al);
                crate::scene::rect(win, px0, py0, pw, ph, pal.bg, 0.78 * al);
                crate::scene::rect(win, px0, py0, (h / 500.0).max(2.0), ph, pal.warn, 0.95 * al);
                draw::text(win, cx, font, &l1, px0 + h * 0.011, py0 + h * 0.006 + f1 * 0.85, f1, 700.0, pal.fg, 0.97 * al, -1);
                draw::text(win, cx, font, &l2, px0 + h * 0.011, py0 + h * 0.006 + f1 * 0.85 + f2 * 1.45, f2, 500.0, pal.warn, 0.95 * al, -1);
            }
        }
    }
}

#[allow(dead_code)]
fn unused(_: C) {}
