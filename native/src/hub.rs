//! The hub: the same `omarchy-pulse` that feeds the web page. pulse-native reads its state once a second over
//! http://127.0.0.1:PORT/api/state with the same token, so both renderers show the same world from the same data.

use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const DEFAULT_PORT: u16 = 47800;
/// Pulse's own scenes; any other name is a scene of yours (the editor's, the community's, or one in code).
pub const BUILTIN: [&str; 10] = ["sphere", "skyline", "orbit", "calm", "hacker", "noir", "city", "mars", "monolith", "pond"];

/// What the hub said last, and when.
#[derive(Clone, Default)]
pub struct Snapshot {
    pub state: Arc<Value>,
    pub at: Option<Instant>,
    pub error: Option<String>,
    /// the design of the hub's scene when it is one made with the editor (or a community `.pulse`): its name, its data
    pub design: Option<(String, Value)>,
    /// a scene in code (JavaScript): the web page runs it; this renderer says so and shows the skyline
    pub handwritten: Option<String>,
    /// your own wallpaper ([pulse] wallpaper): its version and the picture, read by the hub from your disk
    pub wall: Option<(String, Arc<Vec<u8>>)>,
    /// the cover of the song that plays ([music] colors): its version and the picture
    pub cover: Option<(String, Arc<Vec<u8>>)>,
    /// the frames of your camera panels (type image with a src on the hub): by panel id, the frame's number and the picture
    pub cams: std::collections::HashMap<String, (String, Arc<Vec<u8>>)>,
}

pub type Shared = Arc<Mutex<Snapshot>>;

/// An effect the hub asked for (a launch, fireworks, lightning...): what, and its words.
#[derive(Clone, Debug)]
pub struct Fx {
    pub kind: String,
    pub text: String,
    pub small: bool,
    /// everything the hub said about it (an earthquake's magnitude, where it came from)
    pub raw: Value,
}

/// What the live channel brings between two states: the effects, the music's bars, the pointer.
#[derive(Default)]
pub struct Live {
    pub fx: Vec<Fx>,
    pub audio: Vec<f32>,
    pub audio_at: Option<Instant>,
    /// the pointer in the desktop's pixels (the hub reads it from the compositor), and when it last moved
    pub cursor: Option<(f32, f32)>,
    pub cursor_at: Option<Instant>,
}
pub type FxQueue = Arc<Mutex<Live>>;

/// A POST to the hub from a click (acknowledge an incident, a panel's command): sent from its own thread, so the
/// wallpaper never waits for it.
pub fn post(base: &str, path: &str) {
    post_body(base, path, String::new())
}

/// The same, with a body (the layout you saved).
pub fn post_body(base: &str, path: &str, body: String) {
    let sep = if path.contains('?') { '&' } else { '?' };
    let url = format!("{base}{path}{sep}token={}", token());
    let _ = std::thread::Builder::new().name("pulse-post".into()).spawn(move || {
        let _ = ureq::AgentBuilder::new().timeout(Duration::from_secs(4)).build().post(&url).send_string(&body);
    });
}

fn bytes_of(agent: &ureq::Agent, url: &str) -> Option<Vec<u8>> {
    use std::io::Read;
    let resp = agent.get(url).call().ok()?;
    let mut out = Vec::new();
    resp.into_reader().take(40 << 20).read_to_end(&mut out).ok()?;
    if out.is_empty() { None } else { Some(out) }
}

pub fn config_dir() -> PathBuf {
    if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
        if !x.is_empty() {
            return PathBuf::from(x).join("omarchy-pulse");
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config/omarchy-pulse")
}

/// `PULSE_PORT`, else `[pulse] port` in pulse.toml, else 47800: the same rule as the hub.
pub fn port() -> u16 {
    if let Some(p) = std::env::var("PULSE_PORT").ok().and_then(|v| v.parse().ok()) {
        return p;
    }
    std::fs::read_to_string(config_dir().join("pulse.toml"))
        .ok()
        .and_then(|s| s.parse::<toml::Table>().ok())
        .and_then(|t| t.get("pulse")?.get("port")?.as_integer())
        .and_then(|p| u16::try_from(p).ok())
        .unwrap_or(DEFAULT_PORT)
}

pub fn token() -> String {
    std::fs::read_to_string(config_dir().join("token")).map(|s| s.trim().to_string()).unwrap_or_default()
}

/// Reads the hub every second in its own thread. A hub that isn't running yet is not an error worth stopping for:
/// the wallpaper shows what it has and says it is waiting.
pub fn start(base: String, token: String) -> Shared {
    let shared: Shared = Arc::new(Mutex::new(Snapshot::default()));
    let out = shared.clone();
    std::thread::Builder::new()
        .name("pulse-hub".into())
        .spawn(move || {
            let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(4)).build();
            let url = format!("{base}/api/state?token={token}");
            let (base, token) = (base.clone(), token.clone());
            // everything is fetched into this copy first, so the wallpaper never waits for the network
            let mut cur = Snapshot::default();
            loop {
                let got = agent.get(&url).call().map_err(|e| e.to_string()).and_then(|r| r.into_string().map_err(|e| e.to_string())).and_then(|b| serde_json::from_str::<Value>(&b).map_err(|e| e.to_string()));
                {
                    let s = &mut cur;
                    match got {
                        Ok(v) => {
                            // a scene that is not one of Pulse's own: ask the hub for its design, once
                            let name = text(&v, &["ui", "scene"]);
                            let known = s.design.as_ref().map(|d| d.0 == name).unwrap_or(false) || s.handwritten.as_deref() == Some(name.as_str());
                            if !BUILTIN.contains(&name.as_str()) && !name.is_empty() && !known {
                                let durl = format!("{base}/api/scene-design?name={name}&token={token}");
                                if let Ok(d) = agent.get(&durl).call().map_err(|e| e.to_string()).and_then(|r| r.into_string().map_err(|e| e.to_string())).and_then(|b| serde_json::from_str::<Value>(&b).map_err(|e| e.to_string())) {
                                    if d.get("design").map(|x| x.is_object()).unwrap_or(false) {
                                        s.design = Some((name.clone(), d["design"].clone()));
                                        s.handwritten = None;
                                    } else {
                                        s.handwritten = Some(name.clone());
                                    }
                                }
                            }
                            // your own wallpaper and the song's cover: fetched once for each version
                            for (key, path, is_wall) in [(text(&v, &["ui", "wallpaper", "v"]), "wallpaper", true), (text(&v, &["ui", "musicCover", "v"]), "cover", false)] {
                                let slot = if is_wall { &mut s.wall } else { &mut s.cover };
                                if key.is_empty() {
                                    *slot = None;
                                } else if slot.as_ref().map(|x| x.0 != key).unwrap_or(true) {
                                    if let Some(b) = bytes_of(&agent, &format!("{base}/api/{path}?token={token}&v={key}")) {
                                        *slot = Some((key, Arc::new(b)));
                                    }
                                }
                            }
                            // camera panels: a new frame is fetched when its number changes (never while sharing the screen)
                            let private = get(&v, &["privacy"]).as_bool().unwrap_or(false) || get(&v, &["screencast"]).as_bool().unwrap_or(false);
                            let mut seen = Vec::new();
                            for wd in list(&v, &["widgets"]).iter().filter(|wd| text(wd, &["type"]) == "image").take(6) {
                                let (id, src, frame) = (text(wd, &["id"]), text(wd, &["data", "src"]), get(wd, &["data", "frame"]).to_string());
                                if private || !src.starts_with('/') || get(wd, &["data", "frame"]).is_null() {
                                    continue;
                                }
                                seen.push(id.clone());
                                if s.cams.get(&id).map(|c| c.0 != frame).unwrap_or(true) {
                                    let sep = if src.contains('?') { '&' } else { '?' };
                                    if let Some(b) = bytes_of(&agent, &format!("{base}{src}{sep}token={token}&f={}", frame.trim_matches('"'))) {
                                        s.cams.insert(id, (frame, Arc::new(b)));
                                    }
                                }
                            }
                            s.cams.retain(|k, _| seen.contains(k));
                            s.state = Arc::new(v);
                            s.at = Some(Instant::now());
                            s.error = None;
                        }
                        Err(e) => s.error = Some(e.chars().take(120).collect()),
                    }
                }
                if let Ok(mut s) = out.lock() {
                    *s = cur.clone();
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        })
        .expect("the hub thread");
    shared
}

/// The hub's live channel (/api/live, server-sent events): the effects, as the web page hears them. It reconnects
/// by itself; only effects asked after this program started are played.
pub fn start_live(base: String, token: String) -> FxQueue {
    let q: FxQueue = Arc::new(Mutex::new(Live::default()));
    let out = q.clone();
    std::thread::Builder::new()
        .name("pulse-live".into())
        .spawn(move || {
            use std::io::{BufRead, BufReader};
            let agent = ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(4)).build();
            let url = format!("{base}/api/live?fx=-1&token={token}");
            loop {
                if let Ok(resp) = agent.get(&url).call() {
                    for line in BufReader::new(resp.into_reader()).lines() {
                        let Ok(line) = line else { break };
                        let Some(data) = line.strip_prefix("data: ") else { continue };
                        let Ok(v) = serde_json::from_str::<Value>(data) else { continue };
                        let fx: Vec<Fx> = list(&v, &["fx"])
                            .iter()
                            .map(|f| Fx { kind: text(f, &["kind"]), text: text(f, &["text"]), small: get(f, &["small"]).as_bool().unwrap_or(false), raw: f.clone() })
                            .collect();
                        let audio: Vec<f32> = list(&v, &["a"]).iter().filter_map(|x| x.as_f64().map(|n| n as f32)).collect();
                        let cursor = match (get(&v, &["c"]).get(0).and_then(Value::as_f64), get(&v, &["c"]).get(1).and_then(Value::as_f64)) {
                            (Some(x), Some(y)) => Some((x as f32, y as f32)),
                            _ => None,
                        };
                        if let Ok(mut g) = out.lock() {
                            if !audio.is_empty() {
                                g.audio = audio;
                                g.audio_at = Some(Instant::now());
                            }
                            if cursor.is_some() && cursor != g.cursor {
                                g.cursor = cursor;
                                g.cursor_at = Some(Instant::now());
                            }
                            g.fx.extend(fx);
                            let n = g.fx.len();
                            if n > 32 {
                                g.fx.drain(0..n - 32);
                            }
                        }
                    }
                }
                std::thread::sleep(Duration::from_secs(3));
            }
        })
        .expect("the live thread");
    q
}

// ---- small readers over the state, so a missing field is a default and never a crash

pub fn get<'a>(v: &'a Value, path: &[&str]) -> &'a Value {
    let mut cur = v;
    for k in path {
        cur = cur.get(*k).unwrap_or(&Value::Null);
    }
    cur
}

pub fn num(v: &Value, path: &[&str]) -> Option<f64> {
    get(v, path).as_f64()
}

pub fn text(v: &Value, path: &[&str]) -> String {
    match get(v, path) {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

pub fn list<'a>(v: &'a Value, path: &[&str]) -> &'a [Value] {
    get(v, path).as_array().map(|a| a.as_slice()).unwrap_or(&[])
}
