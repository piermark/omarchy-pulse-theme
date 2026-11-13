//! pulse-native: the landmarks of your city, from pulse.js (cities_data.rs) plus what the Rust version adds: the
//! Statue of Liberty in New York's harbour, Christ the Redeemer over Rio, and more cities (Verona, Palermo, Catania,
//! Trieste, Bari, Agra with the Taj Mahal, St Petersburg, Philadelphia, Houston, Dallas, Denver, Jerusalem,
//! Marrakesh, Casablanca, Brasília, Macau). A new landmark is either drawn here (an outline in screen heights, its
//! foot on the horizon) or one of pulse.js's shapes taken from another city and resized.

use crate::cities_data::{city as data_city, Landmark};

/// One landmark, as the backdrop draws it.
pub struct Lm {
    pub name: &'static str,
    pub x: f32,
    pub w: f32,
    pub h: f32,
    pub stroked: bool,
    pub nat: bool,
    pub flat: bool,
    pub top: bool,
    pub paths: Vec<Vec<(f32, f32)>>,
}

impl Lm {
    fn of(l: &Landmark) -> Lm {
        Lm { name: l.name, x: l.x, w: l.w, h: l.h, stroked: l.stroked, nat: l.nat, flat: l.flat, top: l.top, paths: l.paths.iter().map(|p| p.to_vec()).collect() }
    }
    /// The same shape at another place, `k` times as big.
    fn at(mut self, x: f32, k: f32) -> Lm {
        self.x = x;
        self.w *= k;
        self.h *= k;
        for p in &mut self.paths {
            for q in p.iter_mut() {
                q.0 *= k;
                q.1 *= k;
            }
        }
        self
    }
}

/// pulse.js's shape `name` as it stands in `city` (the first one of that name there).
fn take(city: &str, name: &str) -> Lm {
    data_city(city).and_then(|l| l.iter().find(|m| m.name == name)).map(Lm::of).unwrap_or_else(|| Lm { name: "blocks", x: 0.0, w: 0.0, h: 0.0, stroked: false, nat: false, flat: true, top: false, paths: Vec::new() })
}

/// An outline given in units of its own height (x from the middle, y up from the foot), made `h` screen heights tall.
fn drawn(name: &'static str, x: f32, h: f32, top: bool, shapes: &[&[(f32, f32)]]) -> Lm {
    let paths: Vec<Vec<(f32, f32)>> = shapes.iter().map(|s| s.iter().map(|q| (q.0 * h, -q.1 * h)).collect()).collect();
    let w = paths.iter().flatten().fold(0.0f32, |m, q| m.max(q.0.abs())) * 2.0;
    Lm { name, x, w, h, stroked: false, nat: false, flat: false, top, paths }
}

/// The Statue of Liberty on its star fort and its pedestal: the robe, the tablet on her left arm, the crown's rays,
/// the torch held high.
fn liberty(x: f32, h: f32) -> Lm {
    drawn("liberty", x, h, true, &[
        &[(-0.50, 0.0), (0.50, 0.0), (0.44, 0.07), (0.30, 0.09), (-0.30, 0.09), (-0.44, 0.07)],
        &[(-0.21, 0.09), (0.21, 0.09), (0.17, 0.36), (0.20, 0.38), (0.20, 0.41), (-0.20, 0.41), (-0.20, 0.38), (-0.17, 0.36)],
        &[
            (-0.12, 0.41), (-0.11, 0.52), (-0.125, 0.60), (-0.16, 0.63), (-0.165, 0.72), (-0.09, 0.715), (-0.075, 0.76), (-0.05, 0.785),
            (-0.045, 0.81), (-0.085, 0.825), (-0.04, 0.835), (-0.065, 0.875), (-0.02, 0.85), (-0.005, 0.90), (0.012, 0.85), (0.05, 0.875),
            (0.035, 0.83), (0.07, 0.82), (0.042, 0.805), (0.045, 0.78), (0.062, 0.765), (0.072, 0.80), (0.078, 0.91), (0.07, 0.935),
            (0.066, 0.955), (0.082, 0.995), (0.098, 0.958), (0.094, 0.935), (0.088, 0.91), (0.098, 0.80), (0.092, 0.745), (0.085, 0.70),
            (0.10, 0.60), (0.115, 0.50), (0.125, 0.41),
        ],
    ])
}

/// Christ the Redeemer: arms open, on top of the Corcovado (the hill is the hill shape, under him).
fn redeemer(x: f32, h: f32) -> Lm {
    drawn("redeemer", x, h, true, &[
        &[(-0.08, 0.0), (0.08, 0.0), (0.06, 0.08), (0.05, 0.62), (0.40, 0.66), (0.42, 0.70), (0.06, 0.72), (0.045, 0.80), (0.05, 0.86),
          (0.03, 0.90), (0.0, 0.92), (-0.03, 0.90), (-0.05, 0.86), (-0.045, 0.80), (-0.06, 0.72), (-0.42, 0.70), (-0.40, 0.66), (-0.05, 0.62), (-0.06, 0.08)],
    ])
}

/// The Taj Mahal: the onion dome on its drum, the four little domes, the four minarets, the platform.
fn taj(x: f32, h: f32) -> Lm {
    let mut shapes: Vec<Vec<(f32, f32)>> = vec![
        vec![(-0.95, 0.0), (0.95, 0.0), (0.95, 0.06), (-0.95, 0.06)],
        vec![(-0.42, 0.06), (0.42, 0.06), (0.42, 0.40), (-0.42, 0.40)],
        vec![(-0.17, 0.40), (0.17, 0.40), (0.17, 0.50), (0.24, 0.62), (0.22, 0.74), (0.14, 0.84), (0.05, 0.90), (0.02, 0.96), (0.0, 1.0),
             (-0.02, 0.96), (-0.05, 0.90), (-0.14, 0.84), (-0.22, 0.74), (-0.24, 0.62), (-0.17, 0.50)],
    ];
    for cx in [-0.30f32, 0.30] {
        shapes.push(vec![(cx - 0.07, 0.40), (cx + 0.07, 0.40), (cx + 0.08, 0.47), (cx + 0.04, 0.53), (cx, 0.56), (cx - 0.04, 0.53), (cx - 0.08, 0.47)]);
    }
    for mx in [-0.85f32, -0.62, 0.62, 0.85] {
        shapes.push(vec![(mx - 0.025, 0.06), (mx + 0.025, 0.06), (mx + 0.02, 0.62), (mx + 0.035, 0.64), (mx + 0.02, 0.68), (mx, 0.72), (mx - 0.02, 0.68), (mx - 0.035, 0.64), (mx - 0.02, 0.62)]);
    }
    let refs: Vec<&[(f32, f32)]> = shapes.iter().map(|v| v.as_slice()).collect();
    drawn("taj", x, h, true, &refs)
}

/// The Dome of the Rock: the octagon and its golden dome.
fn rock_dome(x: f32, h: f32) -> Lm {
    drawn("rockdome", x, h, true, &[
        &[(-0.55, 0.0), (0.55, 0.0), (0.55, 0.38), (0.22, 0.42), (0.22, 0.50), (0.21, 0.62), (0.16, 0.74), (0.08, 0.82), (0.02, 0.85), (0.0, 0.95),
          (-0.02, 0.85), (-0.08, 0.82), (-0.16, 0.74), (-0.21, 0.62), (-0.22, 0.50), (-0.22, 0.42), (-0.55, 0.38)],
    ])
}

/// Brasília's Congress: the two towers between two bowls, one up and one down.
fn congress(x: f32, h: f32) -> Lm {
    drawn("congress", x, h, true, &[
        &[(-0.9, 0.0), (0.9, 0.0), (0.9, 0.10), (-0.9, 0.10)],
        &[(-0.09, 0.10), (-0.01, 0.10), (-0.01, 1.0), (-0.09, 1.0)],
        &[(0.01, 0.10), (0.09, 0.10), (0.09, 1.0), (0.01, 1.0)],
        &[(-0.75, 0.10), (-0.40, 0.10), (-0.36, 0.16), (-0.46, 0.24), (-0.58, 0.26), (-0.70, 0.24), (-0.79, 0.16)],
        &[(0.47, 0.10), (0.63, 0.10), (0.78, 0.24), (0.32, 0.24)],
    ])
}

/// Your city's landmarks, or None when Pulse does not know it.
pub fn city(name: &str) -> Option<Vec<Lm>> {
    let base = || data_city(name).map(|l| l.iter().map(Lm::of).collect::<Vec<_>>());
    Some(match name {
        "new-york" => {
            // the Statue of Liberty out in the harbour, at the far left; the blocks move in a little
            let mut v = base()?;
            if let Some(b) = v.iter_mut().find(|l| l.name == "blocks") {
                b.x = -0.78;
            }
            v.push(liberty(-1.0, 0.25));
            v
        }
        "rio-de-janeiro" => {
            let mut v = base()?;
            // Christ the Redeemer on the Corcovado (the first hill): standing on its top
            let hill = v.iter().find(|l| l.name == "hill").map(|l| (l.x, l.h));
            if let Some((hx, hh)) = hill {
                let mut r = redeemer(hx, 0.10);
                for p in &mut r.paths {
                    for q in p.iter_mut() {
                        q.1 -= hh * 0.98;
                    }
                }
                r.h += hh;
                v.push(r);
            }
            v
        }
        "verona" => vec![take("rome", "colosseum").at(-0.4, 0.8), take("venice", "campanile").at(0.35, 0.85), take("turin", "hill").at(0.8, 1.0), take("rome", "blocks").at(-0.9, 1.0)],
        "palermo" => vec![take("florence", "dome").at(-0.35, 0.8), take("florence", "campanile").at(0.2, 0.8), take("naples", "volcano").at(0.75, 0.6), take("venice", "houses").at(-0.8, 1.0)],
        "catania" => vec![take("naples", "volcano").at(0.5, 1.25), take("rome", "dome").at(-0.4, 0.7), take("rome", "blocks").at(-0.85, 1.0), take("florence", "campanile").at(-0.1, 0.7)],
        "trieste" => vec![take("genoa", "lighthouse").at(-0.35, 0.8), take("naples", "castle").at(0.4, 1.0), take("genoa", "hill").at(0.85, 1.0), take("venice", "houses").at(-0.8, 1.0)],
        "bari" => vec![take("naples", "castle").at(-0.4, 1.1), take("rome", "dome").at(0.35, 0.7), take("genoa", "lighthouse").at(0.75, 0.6), take("rome", "blocks").at(-0.85, 1.0)],
        "agra" => vec![taj(-0.35, 0.30), take("delhi", "arch").at(0.45, 1.0), take("delhi", "mosque").at(0.85, 0.8)],
        "st-petersburg" => vec![take("rome", "dome").at(-0.4, 1.0), take("dublin", "needle").at(0.2, 0.9), take("moscow", "onion").at(0.6, 0.9), take("amsterdam", "houses").at(-0.85, 1.0)],
        "philadelphia" => vec![take("ottawa", "campanile").at(-0.3, 1.0), take("chicago", "taper").at(0.3, 0.9), take("chicago", "blocks").at(0.7, 1.0), take("boston", "blocks").at(-0.75, 1.0)],
        "houston" => vec![take("chicago", "willis").at(-0.3, 0.8), take("chicago", "taper").at(0.3, 0.8), take("chicago", "blocks").at(0.7, 1.0), take("miami", "blocks").at(-0.8, 0.8)],
        "dallas" => vec![take("johannesburg", "tv").at(-0.3, 0.9), take("chicago", "taper").at(0.3, 0.8), take("chicago", "blocks").at(0.7, 1.0), take("miami", "blocks").at(-0.8, 0.8)],
        "denver" => vec![take("chicago", "taper").at(-0.3, 0.7), take("chicago", "blocks").at(0.2, 0.8), take("santiago", "mesa").at(0.75, 1.0)],
        "jerusalem" => vec![rock_dome(-0.3, 0.20), take("prague", "campanile").at(0.35, 0.8), take("amsterdam", "houses").at(0.8, 1.0), take("athens", "hill").at(-0.8, 0.7)],
        "marrakesh" => vec![take("delhi", "minaret").at(-0.3, 1.1), take("venice", "houses").at(0.35, 1.0), take("kathmandu", "mesa").at(0.8, 1.0)],
        "casablanca" => vec![take("delhi", "minaret").at(-0.3, 1.6), take("abu-dhabi", "mosque").at(0.25, 0.8), take("lagos", "blocks").at(0.75, 0.9)],
        "brasilia" => vec![congress(-0.3, 0.22), take("rome", "dome").at(0.45, 0.6), take("sao-paulo", "blocks").at(0.85, 0.7)],
        "macau" => vec![take("auckland", "tv").at(-0.3, 0.95), take("hong-kong", "blocks").at(0.3, 1.0), take("hong-kong", "hill").at(0.8, 0.8)],
        _ => base()?,
    })
}
