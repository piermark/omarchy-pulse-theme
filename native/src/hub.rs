//! The hub: the same `omarchy-pulse` that feeds the web page. pulse-native reads its state once a second over
//! http://127.0.0.1:PORT/api/state with the same token, so both renderers show the same world from the same data.

use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const DEFAULT_PORT: u16 = 47800;

/// What the hub said last, and when.
#[derive(Clone, Default)]
pub struct Snapshot {
    pub state: Value,
    pub at: Option<Instant>,
    pub error: Option<String>,
}

pub type Shared = Arc<Mutex<Snapshot>>;

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
            loop {
                let got = agent.get(&url).call().map_err(|e| e.to_string()).and_then(|r| r.into_string().map_err(|e| e.to_string())).and_then(|b| serde_json::from_str::<Value>(&b).map_err(|e| e.to_string()));
                if let Ok(mut s) = out.lock() {
                    match got {
                        Ok(v) => {
                            s.state = v;
                            s.at = Some(Instant::now());
                            s.error = None;
                        }
                        Err(e) => s.error = Some(e.chars().take(120).collect()),
                    }
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        })
        .expect("the hub thread");
    shared
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
