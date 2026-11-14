//! pulse-native: the real time on Mars, at Jezero crater where Perseverance is (18.44° N, 77.45° E), from the
//! algorithm NASA's Mars24 uses (Allison & McEwen 2000), and how far Mars is now: the light's delay from the
//! planets' mean orbital elements (Standish, JPL; good to a few seconds of light).

const JEZERO_LAT: f64 = 18.4447;
const JEZERO_EAST: f64 = 77.4508;

pub struct Mars {
    /// local mean solar time at Jezero, in hours
    pub lmst: f64,
    /// local true solar time (the Sun's real place), in hours
    pub ltst: f64,
    /// the Sun's height over Jezero's horizon, in degrees
    pub sun_el: f64,
    /// Perseverance's sol (sol 0: the landing, 18 February 2021)
    pub sol: i64,
    /// the season: the Sun's areocentric longitude, in degrees (0 = northern spring)
    pub ls: f64,
}

fn sind(x: f64) -> f64 {
    x.to_radians().sin()
}
fn cosd(x: f64) -> f64 {
    x.to_radians().cos()
}

/// Mars Sol Date and the days since J2000 (TT) for a moment in Unix seconds.
fn msd(unix: f64) -> (f64, f64) {
    let jd_ut = unix / 86400.0 + 2440587.5;
    let jd_tt = jd_ut + (37.0 + 32.184) / 86400.0;
    let dt = jd_tt - 2451545.0;
    ((dt - 4.5) / 1.0274912517 + 44796.0 - 0.0009626, dt)
}

pub fn at(unix: f64) -> Mars {
    let (m_sd, dt) = msd(unix);
    let mtc = (24.0 * m_sd).rem_euclid(24.0);
    let m = (19.3871 + 0.52402073 * dt).rem_euclid(360.0);
    let a_fms = (270.3871 + 0.524038496 * dt).rem_euclid(360.0);
    // the planetary perturbations (the main seven terms)
    let pbs: f64 = [(0.0071, 2.2353, 49.409), (0.0057, 2.7543, 168.173), (0.0039, 1.1177, 191.837), (0.0037, 15.7866, 21.736), (0.0021, 2.1354, 15.704), (0.0020, 2.4694, 95.528), (0.0018, 32.8493, 49.095)]
        .iter()
        .map(|(a, tau, phi)| a * cosd((0.985626 * dt / tau) + phi))
        .sum();
    let nu_m = (10.691 + 3.0e-7 * dt) * sind(m) + 0.623 * sind(2.0 * m) + 0.050 * sind(3.0 * m) + 0.005 * sind(4.0 * m) + 0.0005 * sind(5.0 * m) + pbs;
    let ls = (a_fms + nu_m).rem_euclid(360.0);
    let eot = 2.861 * sind(2.0 * ls) - 0.071 * sind(4.0 * ls) + 0.002 * sind(6.0 * ls) - nu_m;
    let west = 360.0 - JEZERO_EAST;
    let lmst = (mtc - west * 24.0 / 360.0).rem_euclid(24.0);
    let ltst = (lmst + eot * 24.0 / 360.0).rem_euclid(24.0);
    let dec = (0.42565 * sind(ls)).asin().to_degrees() + 0.25 * sind(ls);
    let hour_angle = (ltst - 12.0) * 15.0;
    let sun_el = (sind(JEZERO_LAT) * sind(dec) + cosd(JEZERO_LAT) * cosd(dec) * cosd(hour_angle)).asin().to_degrees();
    // the sol at Jezero: Mars Sol Date shifted by the longitude; landing at sol 0
    let (land, _) = msd(1613681700.0); // 2021-02-18 20:55 UTC
    let local = |x: f64| (x + JEZERO_EAST / 360.0).floor() as i64;
    Mars { lmst, ltst, sun_el, sol: local(m_sd) - local(land), ls }
}

/// Where a planet is around the Sun (AU, ecliptic), from its J2000 elements and their rates per century.
fn helio(el: [f64; 12], t: f64) -> [f64; 3] {
    let a = el[0] + el[6] * t;
    let e = el[1] + el[7] * t;
    let i = (el[2] + el[8] * t).to_radians();
    let l = el[3] + el[9] * t;
    let peri = el[4] + el[10] * t;
    let node = el[5] + el[11] * t;
    let w = (peri - node).to_radians();
    let m = (l - peri).rem_euclid(360.0).to_radians();
    let mut ea = m + e * m.sin();
    for _ in 0..8 {
        ea -= (ea - e * ea.sin() - m) / (1.0 - e * ea.cos());
    }
    let (xp, yp) = (a * (ea.cos() - e), a * (1.0 - e * e).sqrt() * ea.sin());
    let (cw, sw, cn, sn, ci, si) = (w.cos(), w.sin(), node.to_radians().cos(), node.to_radians().sin(), i.cos(), i.sin());
    [
        (cw * cn - sw * sn * ci) * xp + (-sw * cn - cw * sn * ci) * yp,
        (cw * sn + sw * cn * ci) * xp + (-sw * sn + cw * cn * ci) * yp,
        (sw * si) * xp + (cw * si) * yp,
    ]
}

/// How long the light takes from the Earth to Mars now, in seconds.
pub fn light_delay(unix: f64) -> f64 {
    let t = (unix / 86400.0 + 2440587.5 - 2451545.0) / 36525.0;
    let earth = helio([1.00000261, 0.01671123, -0.00001531, 100.46457166, 102.93768193, 0.0, 0.00000562, -0.00004392, -0.01294668, 35999.37244981, 0.32327364, 0.0], t);
    let mars = helio([1.52371034, 0.09339410, 1.84969142, -4.55343205, -23.94362959, 49.55953891, 0.00001847, 0.00007882, -0.00813131, 19140.30268499, 0.44441088, -0.29257343], t);
    let d = ((earth[0] - mars[0]).powi(2) + (earth[1] - mars[1]).powi(2) + (earth[2] - mars[2]).powi(2)).sqrt();
    d * 499.004784
}

pub fn delay_words(s: f64) -> String {
    format!("{}m {:02}s", (s / 60.0) as i64, (s % 60.0) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clock() {
        // Mars24's own example: 2000-01-06 00:00 UTC is MSD 44795.9998
        let (m, _) = msd(947116800.0);
        assert!((m - 44795.9998).abs() < 0.001, "{m}");
        // the landing is sol 0, and its sol counts on
        assert_eq!(at(1613681700.0).sol, 0);
        assert!(at(1613681700.0 + 88775.244 * 100.0).sol == 100);
        // the light: from 3 minutes (opposition) to 22 (conjunction)
        for k in 0..200 {
            let s = light_delay(1.7e9 + k as f64 * 86400.0 * 4.0);
            assert!((180.0..1340.0).contains(&s), "{s}");
        }
        // the opposition of 16 January 2025: about 0.642 AU, 5 min 20 s
        let s = light_delay(1737028800.0);
        assert!((s - 320.0).abs() < 15.0, "{s}");
    }
}
