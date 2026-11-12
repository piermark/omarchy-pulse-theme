//! PULSE_PERF=1: how long each part of a frame takes on the CPU, printed every five seconds (as omarchy-pulse perf
//! does for the web page). Off, it costs one check of a flag.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Instant;

thread_local! {
    static ON: bool = std::env::var("PULSE_PERF").is_ok();
    static SUMS: RefCell<(Option<Instant>, BTreeMap<&'static str, (f64, u32)>)> = RefCell::new((None, BTreeMap::new()));
}

pub fn on() -> bool {
    ON.with(|o| *o)
}

/// Adds a time measured elsewhere.
pub fn record(name: &'static str, ms: f64) {
    if on() {
        time(name, || std::thread::sleep(std::time::Duration::ZERO));
        SUMS.with(|s| {
            if let Some(e) = s.borrow_mut().1.get_mut(name) {
                e.0 += ms;
            }
        });
    }
}

/// Runs `f`, adding its time to `name` when measuring.
pub fn time<R>(name: &'static str, f: impl FnOnce() -> R) -> R {
    if !on() {
        return f();
    }
    let a = Instant::now();
    let r = f();
    let ms = a.elapsed().as_secs_f64() * 1000.0;
    SUMS.with(|s| {
        let mut s = s.borrow_mut();
        let e = s.1.entry(name).or_insert((0.0, 0));
        e.0 += ms;
        e.1 += 1;
        let start = *s.0.get_or_insert_with(Instant::now);
        if start.elapsed().as_secs_f64() > 5.0 {
            let secs = start.elapsed().as_secs_f64();
            let line: Vec<String> = s.1.iter().map(|(k, v)| format!("{k} {:.2}ms×{:.1}/s", v.0 / v.1.max(1) as f64, v.1 as f64 / secs)).collect();
            eprintln!("perf: {}", line.join(" · "));
            s.1.clear();
            s.0 = Some(Instant::now());
        }
    });
    r
}

/// Times the parts of a long function one after the other: `mark("part")` records the time since the last mark.
pub struct Lap(Option<Instant>, u32);
impl Lap {
    pub fn new() -> Self {
        SECTION.with(|c| c.set(0));
        Lap(if on() { Some(Instant::now()) } else { None }, 0)
    }
    pub fn mark(&mut self, name: &'static str) {
        SECTION.with(|c| c.set(c.get() + 1));
        if let Some(a) = self.0 {
            record(name, a.elapsed().as_secs_f64() * 1000.0);
            self.0 = Some(Instant::now());
            // and how many shapes it built (in the PULSE_PERF line, after a slash)
            let n = crate::draw::PATHS.with(|p| p.get());
            if std::env::var("PULSE_PATHS").is_ok() {
                eprintln!("paths {name}: {}", n - self.1);
            }
            self.1 = n;
        }
    }
}

thread_local! { pub static SECTION: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
pub fn skip_paths() -> bool {
    static N: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    N.get_or_init(|| std::env::var("PULSE_NOPATH_SEC").ok().and_then(|v| v.parse().ok())).map(|n| SECTION.with(|c| c.get()) == n).unwrap_or(false)
}
