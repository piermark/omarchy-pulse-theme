//! The soundscape ([sound] in pulse.toml), as the web page makes it with Web Audio: rain as heavy as it rains,
//! wind, a plane passing high over you, thunder a few seconds after its bolt, a chime for what happens. All of it
//! is made here from noise and sines and handed to the system's player (pw-cat, paplay or aplay): no audio library,
//! nothing recorded. Nothing runs until you set a volume.

use crate::hub::{get, list, num};
use serde_json::Value;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

const RATE: f32 = 48000.0;

#[derive(Default)]
struct Mix {
    vol: f32,
    rain: f32,
    wind: f32,
    wind_hz: f32,
    plane: f32,
    thunder_on: bool,
    events_on: bool,
    /// one-shot sounds: (what, seconds until it starts, strength or pitch)
    shots: Vec<(Shot, f32)>,
}

enum Shot {
    Thunder(f32),
    Rumble(f32),
    Note(f32),
}

#[derive(Clone)]
pub struct Sound(Arc<Mutex<Mix>>, Arc<Mutex<bool>>);

struct Lp(f32, f32);
impl Lp {
    fn new(hz: f32) -> Self { Lp(1.0 - (-std::f32::consts::TAU * hz / RATE).exp(), 0.0) }
    fn set(&mut self, hz: f32) { self.0 = 1.0 - (-std::f32::consts::TAU * hz / RATE).exp() }
    fn run(&mut self, x: f32) -> f32 { self.1 += self.0 * (x - self.1); self.1 }
}

fn player() -> Option<Child> {
    let tries: [(&str, &[&str]); 3] = [
        ("pw-cat", &["--playback", "--format", "s16", "--rate", "48000", "--channels", "1", "--media-role", "Game", "-"]),
        ("paplay", &["--raw", "--format=s16le", "--rate=48000", "--channels=1", "--client-name=omarchy-pulse"]),
        ("aplay", &["-q", "-t", "raw", "-f", "S16_LE", "-r", "48000", "-c", "1"]),
    ];
    tries.iter().find_map(|(bin, args)| Command::new(bin).args(*args).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().ok())
}

impl Sound {
    pub fn new() -> Self {
        Sound(Arc::new(Mutex::new(Mix { wind_hz: 400.0, ..Default::default() })), Arc::new(Mutex::new(false)))
    }

    /// Every second, from the state: how loud each voice is now. `quiet`: a fullscreen window, or nobody there.
    pub fn update(&self, st: &Value, quiet: bool) {
        let cfg = get(st, &["ui", "sound"]);
        let vol = if cfg.is_object() && !quiet { num(cfg, &["volume"]).unwrap_or(0.0) as f32 } else { 0.0 };
        let on = |k: &str| get(cfg, &[k]).as_bool().unwrap_or(false);
        let code = num(st, &["weather", "code"]).unwrap_or(0.0) as i32;
        let mm = num(st, &["weather", "rain"]).unwrap_or(0.0) as f32;
        let snow = (71..=77).contains(&code) || code == 85 || code == 86;
        let wet = (51..=67).contains(&code) || (80..=82).contains(&code) || code >= 95 || mm > 0.0;
        let unit = if get(st, &["weather", "windUnit"]).as_str() == Some("mph") { 1.609 } else { 1.0 };
        let kmh = unit * (num(st, &["weather", "wind"]).unwrap_or(0.0) as f32).max(num(st, &["weather", "gust"]).unwrap_or(0.0) as f32 * 0.8);
        let wind_k = (kmh / 60.0).min(1.0);
        let mut plane = 0.0f32;
        if on("planes") {
            for p in list(st, &["sky"]) {
                let el = num(p, &["el"]).unwrap_or(0.0) as f32;
                if el > 25.0 {
                    plane = plane.max((el - 25.0) / 65.0 * 0.5 * (30000.0 / (num(p, &["ft"]).unwrap_or(30000.0) as f32).max(5000.0)).min(1.0));
                }
            }
        }
        if let Ok(mut m) = self.0.lock() {
            m.vol = vol.clamp(0.0, 1.0) * 0.7;
            m.rain = if !on("rain") { 0.0 } else if snow { 0.05 } else if wet { 0.08 + 0.3 * (mm / 6.0).clamp(0.3, 1.0) } else { 0.0 };
            m.wind = if on("wind") { wind_k * wind_k * 0.45 } else { 0.0 };
            m.wind_hz = 300.0 + wind_k * 500.0;
            m.plane = plane;
            m.thunder_on = on("thunder");
            m.events_on = on("events");
        }
        let mut running = self.1.lock().unwrap();
        if vol > 0.0 && !*running {
            *running = true;
            let (mix, flag) = (self.0.clone(), self.1.clone());
            let _ = std::thread::Builder::new().name("pulse-sound".into()).spawn(move || {
                run(mix);
                if let Ok(mut r) = flag.lock() { *r = false }
            });
        }
    }

    pub fn thunder(&self, delay: f32, strength: f32) {
        if let Ok(mut m) = self.0.lock() { if m.thunder_on && m.vol > 0.0 { m.shots.push((Shot::Thunder(strength), delay)) } }
    }
    pub fn rumble(&self, strength: f32) {
        if let Ok(mut m) = self.0.lock() { if m.events_on && m.vol > 0.0 { m.shots.push((Shot::Rumble(strength), 0.0)) } }
    }
    pub fn chime(&self, notes: &[f32]) {
        if let Ok(mut m) = self.0.lock() {
            if m.events_on && m.vol > 0.0 {
                for (i, hz) in notes.iter().enumerate() { m.shots.push((Shot::Note(*hz), i as f32 * 0.35)) }
            }
        }
    }
}

/// A voice that is playing: noise through a low-pass with an envelope (thunder, rumble), or a sine that fades.
struct Voice {
    kind: u8,
    age: f32,
    a: f32,
    lp: Lp,
    phase: f32,
}

fn run(mix: Arc<Mutex<Mix>>) {
    let Some(mut child) = player() else { return };
    let Some(mut pipe) = child.stdin.take() else { return };
    let mut seed = 0x2545F491u32;
    let mut noise = move || { seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5; (seed as f32 / u32::MAX as f32) * 2.0 - 1.0 };
    let (mut rain_hp, mut rain_lp, mut wind_a, mut wind_b, mut plane_lp) = (Lp::new(900.0), Lp::new(7000.0), Lp::new(400.0), Lp::new(250.0), Lp::new(260.0));
    let (mut vol, mut rain, mut wind, mut plane) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let mut voices: Vec<Voice> = Vec::new();
    let mut pending: Vec<(Shot, f32)> = Vec::new();
    let block = (RATE * 0.02) as usize;
    let mut buf = vec![0u8; block * 2];
    let mut silent = 0.0f32;
    loop {
        let (tv, tr, tw, twf, tp) = {
            let Ok(mut m) = mix.lock() else { break };
            pending.append(&mut m.shots);
            (m.vol, m.rain, m.wind, m.wind_hz, m.plane)
        };
        wind_a.set(twf * 1.6);
        wind_b.set(twf * 0.6);
        let dt = block as f32 / RATE;
        for p in pending.iter_mut() { p.1 -= dt }
        let mut i = 0;
        while i < pending.len() {
            if pending[i].1 <= 0.0 {
                let (shot, _) = pending.remove(i);
                voices.push(match shot {
                    Shot::Thunder(s) => Voice { kind: 0, age: 0.0, a: s, lp: Lp::new(160.0 + 200.0 * s), phase: 0.0 },
                    Shot::Rumble(s) => Voice { kind: 1, age: 0.0, a: s, lp: Lp::new(90.0), phase: 0.0 },
                    Shot::Note(hz) => Voice { kind: 2, age: 0.0, a: hz, lp: Lp::new(8000.0), phase: 0.0 },
                });
            } else {
                i += 1;
            }
        }
        for k in 0..block {
            // the levels glide to where the state wants them (about a second), as the web page ramps its gains
            vol += (tv - vol) * 0.00003;
            rain += (tr - rain) * 0.00002;
            wind += (tw - wind) * 0.00002;
            plane += (tp - plane) * 0.00001;
            let n = noise();
            let r = { let lo = rain_lp.run(n); lo - rain_hp.run(lo) } * rain * 1.6;
            let w = (wind_a.run(n) - wind_b.run(n)) * wind * 3.0;
            let p = plane_lp.run(n) * plane * 4.0;
            let mut x = r + w + p;
            for v in voices.iter_mut() {
                v.age += 1.0 / RATE;
                x += match v.kind {
                    0 => v.lp.run(n) * 6.0 * 0.9 * v.a * if v.age < 0.08 { v.age / 0.08 } else { (-(v.age - 0.08) * 2.0).exp() },
                    1 => v.lp.run(n) * 9.0 * v.a * if v.age < 0.6 { v.age / 0.6 } else { (-(v.age - 0.6) * 1.5).exp() },
                    _ => {
                        v.phase += std::f32::consts::TAU * v.a / RATE;
                        v.phase.sin() * 0.25 * if v.age < 0.02 { v.age / 0.02 } else { (-(v.age - 0.02) * 5.0).exp() }
                    }
                };
            }
            let s = ((x * vol).clamp(-1.0, 1.0) * 32000.0) as i16;
            buf[k * 2..k * 2 + 2].copy_from_slice(&s.to_le_bytes());
        }
        voices.retain(|v| v.age < if v.kind == 2 { 1.3 } else { 5.1 });
        if pipe.write_all(&buf).is_err() { break }
        // a minute with the volume at zero: the player goes away until you want sound again
        silent = if tv <= 0.0 && voices.is_empty() { silent + dt } else { 0.0 };
        if silent > 60.0 { break }
    }
    drop(pipe);
    let _ = child.kill();
    let _ = child.wait();
}
