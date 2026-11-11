//! The Omarchy theme's colours, mapped the way pulse.js maps them (applyTheme), with Pulse's own as the default.

use crate::hub::get;
use gpui::{rgba, Rgba};
use serde_json::Value;

pub type C = [f32; 3];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pal {
    pub bg: C,
    pub fg: C,
    pub accent: C,
    pub accent2: C,
    pub ok: C,
    pub warn: C,
    pub crit: C,
    pub dim: C,
}

impl Default for Pal {
    fn default() -> Self {
        Pal {
            bg: hex("#070a12").unwrap(),
            fg: hex("#c9d6f2").unwrap(),
            accent: hex("#39e1ff").unwrap(),
            accent2: hex("#ff3fb4").unwrap(),
            ok: hex("#6ff0a8").unwrap(),
            warn: hex("#ffbd4a").unwrap(),
            crit: hex("#ff4d6a").unwrap(),
            dim: hex("#4b5a78").unwrap(),
        }
    }
}

pub fn hex(s: &str) -> Option<C> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some([((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0])
}

impl Pal {
    /// state.theme.colors: background, foreground, accent (or color6/cyan/color4/blue), color5..., as in pulse.js.
    pub fn from_state(st: &Value) -> Pal {
        let c = get(st, &["theme", "colors"]);
        let pick = |keys: &[&str]| keys.iter().find_map(|k| c.get(*k).and_then(|v| v.as_str()).and_then(hex));
        let d = Pal::default();
        let accent = pick(&["accent", "color6", "cyan", "color4", "blue"]);
        Pal {
            bg: pick(&["background"]).unwrap_or(d.bg),
            fg: pick(&["foreground"]).unwrap_or(d.fg),
            accent: accent.unwrap_or(d.accent),
            accent2: pick(&["accent2", "color5", "color13", "magenta", "bright_magenta"]).or(accent).unwrap_or(d.accent2),
            ok: pick(&["color2", "green"]).unwrap_or(d.ok),
            warn: pick(&["color3", "yellow"]).unwrap_or(d.warn),
            crit: pick(&["color1", "red"]).unwrap_or(d.crit),
            dim: pick(&["color8", "muted", "bright_black"]).unwrap_or(d.dim),
        }
    }
}

pub fn mix(a: C, b: C, t: f32) -> C {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

pub fn scale(a: C, k: f32) -> C {
    [(a[0] * k).min(1.0), (a[1] * k).min(1.0), (a[2] * k).min(1.0)]
}

/// A colour with its alpha, for GPUI.
pub fn col(c: C, a: f32) -> Rgba {
    let b = |x: f32| ((x.clamp(0.0, 1.0) * 255.0).round() as u32) & 255;
    rgba((b(c[0]) << 24) | (b(c[1]) << 16) | (b(c[2]) << 8) | b(a))
}
