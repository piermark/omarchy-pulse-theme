//! pulse-native: what is down on the internet right now. The public status pages of the services the internet and
//! your work stand on (Microsoft 365, Azure, AWS, Google, Cloudflare, GitHub, Slack, Zoom...) and of the main AIs
//! (ChatGPT, Claude, Gemini, Grok, Mistral, Copilot...), plus your own line (the router, DNS, the way out), so you can
//! tell at a glance whether the fault is theirs or yours.
//!
//! A thread of its own reads them every five minutes (every two while something is down), one after the other, and
//! never slows the wallpaper. Only public pages are read, with no account and nothing about you; `--status off` (or
//! PULSE_STATUS=off) turns it all off. `pulse-native --status-check` prints what it reads and exits.
//! Your own pages: ~/.config/omarchy-pulse/status.txt, one per line, "Name  https://status.example.com" (a
//! Statuspage, Instatus or incident.io page, or an RSS/Atom feed of incidents; "-Name" hides one of ours, "-*" all of ours).
//!
//! What it gives the state (merge()): `internet` = {services: [{name, group, level, note}], line: {...}, down: n};
//! when something is down, a status panel "internet" on the right and a hall for each in the datacenter district.

use serde_json::{json, Value};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// Atlassian Statuspage: /api/v2/status.json → status.indicator none|minor|major|critical (incident.io pages answer
    /// the same address)
    Statuspage,
    /// Instatus: /summary.json → page.status UP|HASISSUES|UNDERMAINTENANCE
    Instatus,
    /// Slack's own: {status: ok|active|broken, active_incidents: [...]}
    Slack,
    /// Google Cloud and Workspace: incidents.json, the open ones have no "end"
    Google,
    /// AWS: the public current events, an empty list when all is well
    Aws,
    /// Microsoft's public status (status.cloud.microsoft/api/posts/...): every workload with its Status
    Microsoft,
    /// Apple's system status (a JSONP of services and their events)
    Apple,
    /// an RSS or Atom feed of incidents: an entry of the last hours that does not say it is resolved
    Feed,
    /// try a Statuspage address, then Instatus, then the feed
    Auto,
    /// no status page at all (X, WhatsApp...): the site itself is asked; an error of the server or no answer counts
    Probe,
}

struct Svc {
    name: String,
    group: &'static str,
    /// the addresses to try, in order: the first that answers decides
    urls: Vec<(Kind, String)>,
    /// for a page that covers more than one thing: only these components or products count (empty: the whole page)
    only: Vec<&'static str>,
}

fn svc(name: &str, group: &'static str, urls: &[(Kind, &str)], only: &[&'static str]) -> Svc {
    Svc { name: name.into(), group, urls: urls.iter().map(|(k, u)| (*k, u.to_string())).collect(), only: only.to_vec() }
}

fn ours() -> Vec<Svc> {
    use Kind::*;
    vec![
        // the AIs
        svc("ChatGPT", "ai", &[(Statuspage, "https://status.openai.com/api/v2/status.json"), (Feed, "https://status.openai.com/feed.rss")], &[]),
        svc("Claude", "ai", &[(Statuspage, "https://status.claude.com/api/v2/status.json"), (Statuspage, "https://status.anthropic.com/api/v2/status.json"), (Feed, "https://status.claude.com/history.rss")], &[]),
        svc("Gemini", "ai", &[(Google, "https://status.cloud.google.com/incidents.json")], &["gemini", "vertex ai", "generative ai"]),
        svc("xAI · Grok", "ai", &[(Auto, "https://status.x.ai"), (Probe, "https://grok.com")], &[]),
        svc("Copilot", "ai", &[(Statuspage, "https://www.githubstatus.com/api/v2/summary.json")], &["copilot"]),
        svc("Mistral", "ai", &[(Auto, "https://status.mistral.ai")], &[]),
        svc("Perplexity", "ai", &[(Auto, "https://status.perplexity.com")], &[]),
        svc("DeepSeek", "ai", &[(Auto, "https://status.deepseek.com")], &[]),
        svc("Cursor", "ai", &[(Auto, "https://status.cursor.com")], &[]),
        svc("Groq", "ai", &[(Auto, "https://groqstatus.com")], &[]),
        svc("Hugging Face", "ai", &[(Auto, "https://status.huggingface.co")], &[]),
        // what the internet and your work stand on
        // X and the messages people live on (no public status page: the site itself answers or not)
        svc("X", "net", &[(Probe, "https://x.com")], &[]),
        svc("X API", "net", &[(Statuspage, "https://api.twitterstat.us/api/v2/status.json")], &[]),
        svc("WhatsApp", "net", &[(Probe, "https://web.whatsapp.com")], &[]),
        svc("Instagram", "net", &[(Probe, "https://www.instagram.com")], &[]),
        svc("Facebook", "net", &[(Probe, "https://www.facebook.com")], &[]),
        svc("Telegram", "net", &[(Probe, "https://web.telegram.org")], &[]),
        svc("Microsoft 365", "net", &[(Microsoft, "https://status.cloud.microsoft/api/posts/m365Consumer"), (Microsoft, "https://status.cloud.microsoft/api/posts/mac")], &[]),
        svc("Azure", "net", &[(Microsoft, "https://status.cloud.microsoft/api/posts/azure"), (Feed, "https://rssfeed.azure.status.microsoft/en-us/status/feed/")], &[]),
        svc("AWS", "net", &[(Aws, "https://health.aws.amazon.com/public/currentevents"), (Feed, "https://status.aws.amazon.com/rss/all.rss")], &[]),
        svc("Google Cloud", "net", &[(Google, "https://status.cloud.google.com/incidents.json")], &[]),
        svc("Google Workspace", "net", &[(Google, "https://www.google.com/appsstatus/dashboard/incidents.json")], &[]),
        svc("Cloudflare", "net", &[(Statuspage, "https://www.cloudflarestatus.com/api/v2/status.json")], &[]),
        svc("Akamai", "net", &[(Statuspage, "https://www.akamaistatus.com/api/v2/status.json")], &[]),
        svc("GitHub", "net", &[(Statuspage, "https://www.githubstatus.com/api/v2/status.json")], &[]),
        svc("GitLab", "net", &[(Auto, "https://status.gitlab.com")], &[]),
        svc("Slack", "net", &[(Slack, "https://slack-status.com/api/v2.0.0/current")], &[]),
        svc("Zoom", "net", &[(Statuspage, "https://status.zoom.us/api/v2/status.json")], &[]),
        svc("Discord", "net", &[(Statuspage, "https://discordstatus.com/api/v2/status.json")], &[]),
        svc("Atlassian", "net", &[(Statuspage, "https://jira-software.status.atlassian.com/api/v2/status.json")], &[]),
        svc("Apple", "net", &[(Apple, "https://www.apple.com/support/systemstatus/data/system_status_en_US.js")], &[]),
        svc("Dropbox", "net", &[(Statuspage, "https://status.dropbox.com/api/v2/status.json")], &[]),
    ]
}

/// Yours from status.txt, added to ours ("-Name" takes one of ours away).
fn services() -> Vec<Svc> {
    let mut all = ours();
    let home = std::env::var("HOME").unwrap_or_default();
    let cfg = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
    if let Ok(s) = std::fs::read_to_string(format!("{cfg}/omarchy-pulse/status.txt")) {
        for line in s.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            if line == "-*" {
                all.retain(|x| x.group == "yours"); // only yours
                continue;
            }
            if let Some(n) = line.strip_prefix('-') {
                all.retain(|x| !x.name.eq_ignore_ascii_case(n.trim()));
                continue;
            }
            let Some(at) = line.find("http") else { continue };
            let (name, url) = (line[..at].trim().trim_end_matches(['=', ':']).trim(), line[at..].trim());
            if name.is_empty() {
                continue;
            }
            all.retain(|x| !x.name.eq_ignore_ascii_case(name));
            let k = if url.ends_with(".json") { Kind::Statuspage } else if url.contains("rss") || url.contains("atom") || url.ends_with(".xml") { Kind::Feed } else { Kind::Auto };
            all.push(Svc { name: name.chars().take(24).collect(), group: "yours", urls: vec![(k, url.to_string())], only: vec![] });
        }
    }
    all
}

/// One service's state: 0 fine, 1 degraded (warn), 2 down (crit); None when no address answered.
#[derive(Clone)]
pub struct Read {
    pub name: String,
    pub group: &'static str,
    pub level: Option<u8>,
    pub note: String,
}

#[derive(Clone, Default)]
pub struct Line {
    /// milliseconds to the router, to resolve a name, to reach the internet (a TCP handshake to 1.1.1.1 and 8.8.8.8)
    pub gw: Option<u32>,
    pub dns: Option<u32>,
    pub out: Option<u32>,
}

#[derive(Default)]
struct Shared {
    reads: Vec<Read>,
    line: Line,
    at: Option<Instant>,
    on: bool,
}

fn shared() -> &'static Mutex<Shared> {
    static S: OnceLock<Mutex<Shared>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Shared::default()))
}

pub fn off_by_args() -> bool {
    let arg = std::env::args().skip_while(|a| a != "--status").nth(1);
    let v = arg.or_else(|| std::env::var("PULSE_STATUS").ok()).unwrap_or_default().to_lowercase();
    matches!(v.as_str(), "off" | "0" | "no" | "false")
}

/// Starts the reader (once).
pub fn start() {
    if off_by_args() {
        return;
    }
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        if let Ok(mut s) = shared().lock() {
            s.on = true;
        }
        let _ = std::thread::Builder::new().name("pulse-status".into()).spawn(|| {
            std::thread::sleep(Duration::from_secs(8)); // the wallpaper first
            loop {
                let line = check_line();
                let offline = line.out.is_none() && line.dns.is_none();
                let reads = if offline { Vec::new() } else { read_all(false) };
                let down = reads.iter().any(|r| r.level.unwrap_or(0) > 0);
                if let Ok(mut s) = shared().lock() {
                    if !offline {
                        s.reads = reads;
                    }
                    s.line = line;
                    s.at = Some(Instant::now());
                }
                std::thread::sleep(Duration::from_secs(if offline { 30 } else if down { 120 } else { 300 }));
            }
        });
    });
}

/// Two clients: through the system's proxy when there is one (HTTPS_PROXY), and straight for what NO_PROXY names
/// and for your own network (ureq reads the first, not the second).
struct Agents {
    via: ureq::Agent,
    direct: ureq::Agent,
}

impl Agents {
    fn for_url(&self, url: &str) -> &ureq::Agent {
        let host = url.split("://").nth(1).unwrap_or("").split(['/', '?']).next().unwrap_or("");
        let host = if host.starts_with('[') { host.trim_start_matches('[').split(']').next().unwrap_or("") } else { host.rsplit_once(':').map(|h| h.0).unwrap_or(host) };
        if no_proxy(host) { &self.direct } else { &self.via }
    }
}

fn no_proxy(host: &str) -> bool {
    if host == "localhost" {
        return true;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return match ip {
            std::net::IpAddr::V4(v) => v.is_loopback() || v.is_private() || v.is_link_local(),
            std::net::IpAddr::V6(v) => v.is_loopback(),
        };
    }
    let np = std::env::var("NO_PROXY").or_else(|_| std::env::var("no_proxy")).unwrap_or_default();
    np.split(',').map(|x| x.trim().trim_start_matches('*').trim_start_matches('.')).filter(|x| !x.is_empty()).any(|d| d == "*" || host == d || host.ends_with(&format!(".{d}")))
}

fn agent() -> Agents {
    let b = || ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(5)).timeout(Duration::from_secs(8)).user_agent(concat!("omarchy-pulse-native/", env!("CARGO_PKG_VERSION"), " (status)"));
    Agents { via: b().try_proxy_from_env(true).build(), direct: b().build() }
}

fn read_all(verbose: bool) -> Vec<Read> {
    let ag = agent();
    let mut cache: std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();
    let mut out = Vec::new();
    let mut probed = Vec::new();
    for s in services() {
        let mut got: Option<(u8, String)> = None;
        'urls: for (k, u) in &s.urls {
            let tries: Vec<(Kind, String)> = if *k == Kind::Auto {
                let b = u.trim_end_matches('/');
                vec![(Kind::Statuspage, format!("{b}/api/v2/status.json")), (Kind::Instatus, format!("{b}/summary.json")), (Kind::Feed, format!("{b}/history.rss")), (Kind::Feed, format!("{b}/feed.rss")), (Kind::Feed, format!("{b}/feed.xml")), (Kind::Probe, b.to_string())]
            } else {
                vec![(*k, u.clone())]
            };
            for (k, u) in tries {
                if k == Kind::Probe {
                    let r = probe(ag.for_url(&u), &u);
                    if verbose {
                        eprintln!("  {:<17} {:<8} {}  ({u})", s.name, ["ok", "degraded", "DOWN"][r.0 as usize], r.1);
                    }
                    got = Some(r);
                    break 'urls;
                }
                let body = cache.entry(u.clone()).or_insert_with(|| fetch(ag.for_url(&u), &u)).clone();
                if let Some(b) = body {
                    if let Some(r) = judge(k, &b, &s.only) {
                        if verbose {
                            eprintln!("  {:<17} {:<8} {}  ({u})", s.name, ["ok", "degraded", "DOWN"][r.0 as usize], r.1);
                        }
                        got = Some(r);
                        break 'urls;
                    }
                }
            }
        }
        if verbose && got.is_none() {
            eprintln!("  {:<17} unknown  (no page answered: {})", s.name, s.urls.iter().map(|u| u.1.as_str()).collect::<Vec<_>>().join(", "));
        }
        out.push(Read { name: s.name.clone(), group: s.group, level: got.as_ref().map(|g| g.0), note: got.map(|g| g.1).unwrap_or_default() });
        probed.push(s.urls.iter().all(|u| u.0 == Kind::Probe) || matches!(out.last().map(|r| r.note.as_str()), Some("no answer")));
    }
    // a site that did not answer counts only when the status pages did: else it is the way out that is closed (a
    // proxy, a firewall), not every site at once
    let pages = out.iter().zip(&probed).filter(|(r, p)| !**p && r.level.is_some()).count();
    if pages == 0 {
        for r in out.iter_mut().filter(|r| r.note == "no answer") {
            r.level = None;
            r.note.clear();
        }
    }
    out
}

/// A site with no status page: an answer below 500 (a page, a redirect, even "forbidden") means it is up; a server
/// error means down; no answer at all is only "degraded", since it may be the way there rather than the site.
fn probe(ag: &ureq::Agent, url: &str) -> (u8, String) {
    let t0 = Instant::now();
    match ag.head(url).call() {
        Ok(_) => (0, format!("answers · {} ms", t0.elapsed().as_millis())),
        Err(ureq::Error::Status(c, _)) if c < 500 => (0, format!("answers · {} ms", t0.elapsed().as_millis())),
        Err(ureq::Error::Status(c, _)) => (2, format!("server error {c}")),
        Err(_) => (1, "no answer".into()),
    }
}

fn fetch(ag: &ureq::Agent, url: &str) -> Option<String> {
    let r = ag.get(url).call().ok()?;
    let mut buf = Vec::new();
    use std::io::Read as _;
    r.into_reader().take(4 << 20).read_to_end(&mut buf).ok()?;
    // AWS answers in UTF-16 with a byte order mark
    if buf.len() >= 2 && (buf[0] == 0xff && buf[1] == 0xfe || buf[0] == 0xfe && buf[1] == 0xff) {
        let le = buf[0] == 0xff;
        let units: Vec<u16> = buf[2..].chunks_exact(2).map(|c| if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) }).collect();
        return Some(String::from_utf16_lossy(&units));
    }
    Some(String::from_utf8_lossy(&buf).trim_start_matches('\u{feff}').to_string())
}

fn short(s: &str) -> String {
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.chars().count() > 60 { format!("{}…", s.chars().take(59).collect::<String>()) } else { s }
}

fn has(hay: &str, only: &[&str]) -> bool {
    let h = hay.to_lowercase();
    only.is_empty() || only.iter().any(|o| h.contains(o))
}

/// What a page says: (0 fine / 1 degraded / 2 down, the words); None when it is not that kind of page.
fn judge(k: Kind, body: &str, only: &[&str]) -> Option<(u8, String)> {
    let js = || serde_json::from_str::<Value>(body).ok();
    match k {
        Kind::Statuspage => {
            let v = js()?;
            if !only.is_empty() {
                // a summary: the components that count
                let comps = v.get("components")?.as_array()?;
                let mut worst = (0u8, String::new());
                let mut found = false;
                for c in comps.iter().filter(|c| has(c.get("name").and_then(Value::as_str).unwrap_or(""), only)) {
                    found = true;
                    let st = c.get("status").and_then(Value::as_str).unwrap_or("");
                    let l = match st { "operational" | "under_maintenance" => 0, "degraded_performance" => 1, "partial_outage" => 1, "major_outage" => 2, _ => 0 };
                    if l > worst.0 {
                        worst = (l, format!("{} · {}", c.get("name").and_then(Value::as_str).unwrap_or(""), st.replace('_', " ")));
                    }
                }
                return found.then(|| if worst.0 == 0 { (0, "operational".into()) } else { (worst.0, short(&worst.1)) });
            }
            let st = v.get("status")?;
            let ind = st.get("indicator").and_then(Value::as_str)?;
            let desc = st.get("description").and_then(Value::as_str).unwrap_or("");
            Some((match ind { "none" | "maintenance" => 0, "minor" => 1, _ => 2 }, short(desc)))
        }
        Kind::Instatus => {
            let v = js()?;
            let st = v.get("page")?.get("status").and_then(Value::as_str)?;
            Some((match st { "UP" | "UNDERMAINTENANCE" => 0, "HASISSUES" => 1, _ => 2 }, st.to_lowercase().replace("hasissues", "has issues")))
        }
        Kind::Slack => {
            let v = js()?;
            let st = v.get("status").and_then(Value::as_str)?;
            let inc = v.get("active_incidents").and_then(Value::as_array).cloned().unwrap_or_default();
            let title = inc.first().and_then(|i| i.get("title")).and_then(Value::as_str).unwrap_or("");
            let outage = inc.iter().any(|i| i.get("type").and_then(Value::as_str) == Some("outage"));
            Some(if st == "ok" && inc.is_empty() { (0, "ok".into()) } else { (if outage { 2 } else { 1 }, short(title)) })
        }
        Kind::Google => {
            let v = js()?;
            let arr = v.as_array()?;
            let mut worst = (0u8, String::new());
            for i in arr.iter().filter(|i| i.get("end").map(Value::is_null).unwrap_or(true)) {
                let prods: Vec<String> = i.get("affected_products").and_then(Value::as_array).map(|a| a.iter().filter_map(|p| p.get("title").and_then(Value::as_str).map(str::to_string)).collect()).unwrap_or_default();
                let what = format!("{} {}", prods.join(" "), i.get("service_name").and_then(Value::as_str).unwrap_or(""));
                if !has(&what, only) {
                    continue;
                }
                let sev = i.get("severity").and_then(Value::as_str).unwrap_or("medium");
                let l = if sev == "high" { 2 } else { 1 };
                if l > worst.0 {
                    let d = i.get("external_desc").and_then(Value::as_str).unwrap_or("");
                    worst = (l, short(if d.is_empty() { &what } else { d }));
                }
            }
            Some(if worst.0 == 0 { (0, "no open incident".into()) } else { worst })
        }
        Kind::Aws => {
            let v = js()?;
            let arr = v.as_array()?;
            // status "1" is informational, "2" a degradation, "3" a disruption
            let mut worst = (0u8, String::new());
            for e in arr {
                let st: u8 = e.get("status").and_then(|s| s.as_str().map(str::to_string).or_else(|| s.as_i64().map(|n| n.to_string()))).and_then(|s| s.parse().ok()).unwrap_or(1);
                let l = match st { 3.. => 2, 2 => 1, _ => 0 };
                if l > worst.0 {
                    worst = (l, short(e.get("summary").and_then(Value::as_str).or_else(|| e.get("service_name").and_then(Value::as_str)).unwrap_or("an event")));
                }
            }
            Some(if worst.0 == 0 { (0, "no current event".into()) } else { worst })
        }
        Kind::Microsoft => {
            let v = js()?;
            // every object with a status: "Operational", "Service restored"... are fine; the rest count
            let mut found = false;
            let mut worst = (0u8, String::new());
            walk(&v, &mut |o| {
                let st = ["Status", "status", "ServiceStatus", "serviceStatus"].iter().find_map(|k| o.get(*k).and_then(Value::as_str));
                let Some(st) = st else { return };
                found = true;
                let s = st.to_lowercase().replace(' ', "");
                let l = if s.contains("operational") || s.contains("restored") || s.contains("resolved") || s == "good" || s == "available" || s.contains("falsepositive") || s.contains("postincident") {
                    0
                } else if s.contains("interruption") || s.contains("outage") || s.contains("down") {
                    2
                } else {
                    1
                };
                if l > worst.0 {
                    let t = ["Title", "title", "WorkloadDisplayName", "Name", "name", "service"].iter().find_map(|k| o.get(*k).and_then(Value::as_str)).unwrap_or("");
                    worst = (l, short(&format!("{t} · {st}")));
                }
            });
            found.then(|| if worst.0 == 0 { (0, "operational".into()) } else { worst })
        }
        Kind::Apple => {
            let a = body.find('{')?;
            let b = body.rfind('}')?;
            let v: Value = serde_json::from_str(&body[a..=b]).ok()?;
            let mut worst = (0u8, String::new());
            for s in v.get("services")?.as_array()? {
                for e in s.get("events").and_then(Value::as_array).into_iter().flatten() {
                    if e.get("eventStatus").and_then(Value::as_str).map(|x| x == "resolved" || x == "completed" || x == "upcoming").unwrap_or(false) {
                        continue;
                    }
                    let ty = e.get("statusType").and_then(Value::as_str).unwrap_or("");
                    let l = if ty.contains("Outage") { 2 } else if ty.contains("Maintenance") { 0 } else { 1 };
                    if l > worst.0 {
                        worst = (l, short(&format!("{} · {ty}", s.get("serviceName").and_then(Value::as_str).unwrap_or(""))));
                    }
                }
            }
            Some(if worst.0 == 0 { (0, "all available".into()) } else { worst })
        }
        Kind::Feed => feed(body),
        Kind::Auto | Kind::Probe => None,
    }
}

fn walk(v: &Value, f: &mut dyn FnMut(&serde_json::Map<String, Value>)) {
    match v {
        Value::Object(o) => {
            f(o);
            for x in o.values() {
                walk(x, f);
            }
        }
        Value::Array(a) => {
            for x in a {
                walk(x, f);
            }
        }
        _ => {}
    }
}

/// An RSS or Atom feed of incidents: its newest entry, when it is of the last six hours and does not say resolved.
fn feed(body: &str) -> Option<(u8, String)> {
    let b = body.trim_start();
    if !(b.starts_with("<?xml") || b.starts_with("<rss") || b.starts_with("<feed")) {
        return None;
    }
    let tag = if b.contains("<entry") { "entry" } else { "item" };
    let Some(start) = b.find(&format!("<{tag}")) else { return Some((0, "no incident".into())) };
    let end = b[start..].find(&format!("</{tag}>")).map(|e| start + e).unwrap_or(b.len());
    let e = &b[start..end];
    let field = |n: &str| -> String {
        let Some(a) = e.find(&format!("<{n}")) else { return String::new() };
        let Some(gt) = e[a..].find('>') else { return String::new() };
        let from = a + gt + 1;
        let to = e[from..].find(&format!("</{n}>")).map(|x| from + x).unwrap_or(from);
        e[from..to].replace("<![CDATA[", "").replace("]]>", "")
    };
    let title = field("title");
    let when = [field("pubDate"), field("updated"), field("published")].into_iter().find(|s| !s.is_empty()).unwrap_or_default();
    let age = parse_time(&when).map(|t| now_secs() - t).unwrap_or(f64::MAX);
    let all = format!("{title} {}", field(if tag == "entry" { "content" } else { "description" })).to_lowercase();
    let closed = ["resolved", "completed", "mitigated", "restored", "postmortem", "post-incident", "scheduled"].iter().any(|w| all.contains(w));
    if age < 6.0 * 3600.0 && !closed {
        let l = if all.contains("outage") || all.contains("unavailable") || all.contains("down") { 2 } else { 1 };
        Some((l, short(&strip_tags(&title))))
    } else {
        Some((0, "no open incident".into()))
    }
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in s.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&").replace("&#39;", "'").replace("&quot;", "\"")
}

fn now_secs() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

/// RFC 822 ("Fri, 13 Nov 2026 22:10:00 +0000" or "GMT") or RFC 3339 ("2026-11-13T22:10:00Z"), to seconds.
fn parse_time(s: &str) -> Option<f64> {
    let s = s.trim();
    let days = |y: i64, m: i64, d: i64| -> i64 {
        let (y, m) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * m + 2) / 5 + d - 1;
        era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468
    };
    let hms = |t: &str| -> Option<i64> {
        let p: Vec<i64> = t.split(':').map(|x| x.trim().parse::<f64>().map(|f| f as i64)).collect::<Result<_, _>>().ok()?;
        Some(p.first()? * 3600 + p.get(1).copied().unwrap_or(0) * 60 + p.get(2).copied().unwrap_or(0))
    };
    if s.len() >= 19 && s.as_bytes()[4] == b'-' {
        let (y, mo, d) = (s[0..4].parse().ok()?, s[5..7].parse().ok()?, s[8..10].parse().ok()?);
        let secs = hms(&s[11..19])?;
        let rest = &s[19..];
        let rest = rest.trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
        let off = if rest.starts_with('+') || rest.starts_with('-') {
            let sign = if rest.starts_with('-') { -1 } else { 1 };
            let r = rest[1..].replace(':', "");
            sign * (r.get(0..2)?.parse::<i64>().ok()? * 3600 + r.get(2..4).unwrap_or("0").parse::<i64>().unwrap_or(0) * 60)
        } else {
            0
        };
        return Some((days(y, mo, d) * 86400 + secs - off) as f64);
    }
    let p: Vec<&str> = s.split_whitespace().collect();
    let i = if p.first()?.ends_with(',') { 1 } else { 0 };
    let d: i64 = p.get(i)?.parse().ok()?;
    let mo = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"].iter().position(|m| p.get(i + 1).map(|x| x.to_lowercase().starts_with(m)).unwrap_or(false))? as i64 + 1;
    let y: i64 = p.get(i + 2)?.parse().ok()?;
    let secs = hms(p.get(i + 3)?)?;
    let z = p.get(i + 4).copied().unwrap_or("GMT");
    let off = if z.starts_with('+') || z.starts_with('-') {
        let sign = if z.starts_with('-') { -1 } else { 1 };
        sign * (z.get(1..3).and_then(|x| x.parse::<i64>().ok()).unwrap_or(0) * 3600 + z.get(3..5).and_then(|x| x.parse::<i64>().ok()).unwrap_or(0) * 60)
    } else {
        match z { "PST" => -8 * 3600, "PDT" => -7 * 3600, "EST" => -5 * 3600, "EDT" => -4 * 3600, _ => 0 }
    };
    Some((days(y, mo, d) * 86400 + secs - off) as f64)
}

/// Your line: the router (the default route in /proc/net/route), a name resolved, a TCP handshake out. A refused
/// connection answers too, so the router counts as reached even with no web page on it.
fn check_line() -> Line {
    use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
    let probe = |a: SocketAddr| -> Option<u32> {
        let t0 = Instant::now();
        match TcpStream::connect_timeout(&a, Duration::from_millis(1500)) {
            Ok(_) => Some(t0.elapsed().as_millis() as u32),
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => Some(t0.elapsed().as_millis() as u32),
            Err(_) => None,
        }
    };
    let gw = std::fs::read_to_string("/proc/net/route").ok().and_then(|s| {
        s.lines().skip(1).find_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            if f.get(1) == Some(&"00000000") && f.get(7) == Some(&"00000000") {
                let g = u32::from_str_radix(f.get(2)?, 16).ok()?;
                Some(std::net::Ipv4Addr::from(g.to_le_bytes()))
            } else {
                None
            }
        })
    });
    let gw_ms = gw.and_then(|ip| [53u16, 80, 443].iter().find_map(|p| probe(SocketAddr::new(ip.into(), *p))));
    let t0 = Instant::now();
    let dns = ("one.one.one.one", 443).to_socket_addrs().ok().and_then(|mut a| a.next()).map(|_| t0.elapsed().as_millis() as u32);
    let out = ["1.1.1.1:443", "8.8.8.8:443"].iter().filter_map(|a| a.parse().ok().and_then(probe)).min();
    Line { gw: gw_ms, dns, out }
}

/// Gives the state what is known: `internet`, and when something is down the panel and the datacenter's halls.
pub fn merge(v: &mut Value) {
    let Ok(s) = shared().lock() else { return };
    if !s.on {
        return;
    }
    let line = s.line.clone();
    let known = s.at.is_some();
    let svcs: Vec<Value> = s.reads.iter().map(|r| json!({ "name": r.name, "group": r.group, "level": match r.level { Some(0) => "ok", Some(1) => "warn", Some(_) => "crit", None => "" }, "note": r.note })).collect();
    drop(s);
    let down: Vec<&Value> = svcs.iter().filter(|r| matches!(r["level"].as_str(), Some("warn" | "crit"))).collect();
    let up = svcs.iter().filter(|r| r["level"] == "ok").count();
    // the line: no way out at all is offline; the router answering but nothing beyond is "the provider"
    let line_level = if !known { "" } else if line.out.is_none() && line.dns.is_none() { "crit" } else if line.out.map(|m| m > 400).unwrap_or(true) || line.dns.map(|m| m > 800).unwrap_or(true) { "warn" } else { "ok" };
    let line_note = if !known {
        String::new()
    } else if line.out.is_none() && line.dns.is_none() {
        if line.gw.is_some() { "router ok · no internet".into() } else { "offline".into() }
    } else {
        format!("{} ms{}", line.out.unwrap_or(0), line.dns.map(|d| format!(" · dns {d} ms")).unwrap_or_default())
    };
    let n_down = down.len();
    let crit = down.iter().any(|r| r["level"] == "crit");
    v["internet"] = json!({ "on": true, "known": known, "services": svcs, "down": n_down, "up": up, "line": { "level": line_level, "note": line_note, "gw": line.gw, "dns": line.dns, "out": line.out } });
    if (n_down > 0 || line_level == "crit") && known {
        let mut items: Vec<Value> = Vec::new();
        if line_level == "crit" {
            items.push(json!({ "title": "your line", "level": "crit", "note": line_note }));
        }
        for r in &down {
            // two to a row in the panel: a short word, the page's own words are in `internet`
            items.push(json!({ "title": r["name"], "level": r["level"], "note": if r["level"] == "crit" { "down" } else { "" } }));
        }
        let title = if line_level == "crit" { "internet · you are offline".to_string() } else { format!("internet · {n_down} down") };
        let wd = json!({ "id": "internet", "title": title, "type": "status", "slot": "right", "order": 1, "icon": "󰖟", "data": { "items": items } });
        if let Some(a) = v.get_mut("widgets").and_then(Value::as_array_mut) {
            a.retain(|w| w.get("id").and_then(Value::as_str) != Some("internet"));
            a.insert(0, wd);
        }
        // a hall for each in the datacenter district: lightning when it goes down, sparks when it is back
        let rows: Vec<Value> = down.iter().map(|r| json!({ "name": r["name"], "level": r["level"], "value": null })).collect();
        if !rows.is_empty() {
            if !v.get("datacenter").map(Value::is_object).unwrap_or(false) {
                v["datacenter"] = json!({ "groups": [] });
            }
            if let Some(g) = v["datacenter"].get_mut("groups").and_then(Value::as_array_mut) {
                g.push(json!({ "id": "internet", "title": if crit { "internet · down" } else { "internet" }, "rows": rows }));
            }
        }
    }
}

/// The one line of the network panel: "26 up · 18 ms" or "Microsoft 365 down".
pub fn summary(st: &Value) -> Option<(String, String, &'static str)> {
    let i = st.get("internet")?;
    if !i.get("on").and_then(Value::as_bool).unwrap_or(false) || !i.get("known").and_then(Value::as_bool).unwrap_or(false) {
        return None;
    }
    let svcs = i.get("services").and_then(Value::as_array).cloned().unwrap_or_default();
    let line = &i["line"];
    if line["level"] == "crit" {
        return Some(("internet".into(), line["note"].as_str().unwrap_or("offline").into(), "crit"));
    }
    let down: Vec<&Value> = svcs.iter().filter(|r| matches!(r["level"].as_str(), Some("warn" | "crit"))).collect();
    if let Some(first) = down.iter().find(|r| r["level"] == "crit").or(down.first()) {
        let more = if down.len() > 1 { format!(" +{}", down.len() - 1) } else { String::new() };
        let lv = if first["level"] == "crit" { "crit" } else { "warn" };
        return Some(("internet".into(), format!("{} {}{more}", first["name"].as_str().unwrap_or(""), if lv == "crit" { "down" } else { "degraded" }), lv));
    }
    let up = svcs.iter().filter(|r| r["level"] == "ok").count();
    Some(("internet".into(), format!("{up} up · {}", line["note"].as_str().unwrap_or("").split(" · ").next().unwrap_or("")), if line["level"] == "warn" { "warn" } else { "ok" }))
}

/// The changes worth saying since the last look: (effect, words). "lightning" for one going down, "shooting" when it
/// is back, as the datacenter's halls do.
pub fn changes(prev: &mut std::collections::HashMap<String, String>, st: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Some(svcs) = st.get("internet").and_then(|i| i.get("services")).and_then(Value::as_array) else { return out };
    let first = prev.is_empty();
    for r in svcs {
        let (n, l) = (r["name"].as_str().unwrap_or("").to_string(), r["level"].as_str().unwrap_or("").to_string());
        if l.is_empty() {
            continue;
        }
        let was = prev.insert(n.clone(), l.clone());
        if first {
            continue;
        }
        match (was.as_deref(), l.as_str()) {
            (Some("ok"), "crit") | (Some("warn"), "crit") => out.push(("lightning".into(), format!("{n} is down"))),
            (Some("ok"), "warn") => out.push(("tip".into(), format!("{n} · degraded"))),
            (Some("crit") | Some("warn"), "ok") => out.push(("shooting".into(), format!("{n} is back"))),
            _ => {}
        }
    }
    out
}

/// `pulse-native --status-check`: reads every page now and prints what it finds.
pub fn check_and_print() {
    eprintln!("pulse-native {}: the status pages, read now", env!("CARGO_PKG_VERSION"));
    let l = check_line();
    let ms = |m: Option<u32>| m.map(|m| format!("{m} ms")).unwrap_or_else(|| "no answer".into());
    eprintln!("  your line         router {} · dns {} · out {}", ms(l.gw), ms(l.dns), ms(l.out));
    let r = read_all(true);
    let (ok, bad, unk) = (r.iter().filter(|x| x.level == Some(0)).count(), r.iter().filter(|x| x.level.unwrap_or(0) > 0).count(), r.iter().filter(|x| x.level.is_none()).count());
    eprintln!("\n  {ok} fine, {bad} with a problem, {unk} unknown");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pages() {
        assert_eq!(judge(Kind::Statuspage, r#"{"page":{},"status":{"indicator":"major","description":"Partial System Outage"}}"#, &[]).unwrap().0, 2);
        assert_eq!(judge(Kind::Statuspage, r#"{"status":{"indicator":"none","description":"All Systems Operational"}}"#, &[]).unwrap().0, 0);
        let summ = r#"{"status":{"indicator":"minor"},"components":[{"name":"Git Operations","status":"operational"},{"name":"Copilot","status":"major_outage"}]}"#;
        assert_eq!(judge(Kind::Statuspage, summ, &["copilot"]).unwrap().0, 2);
        assert_eq!(judge(Kind::Instatus, r#"{"page":{"status":"HASISSUES"}}"#, &[]).unwrap().0, 1);
        assert_eq!(judge(Kind::Slack, r#"{"status":"ok","active_incidents":[]}"#, &[]).unwrap().0, 0);
        assert_eq!(judge(Kind::Slack, r#"{"status":"active","active_incidents":[{"title":"Messages slow","type":"outage"}]}"#, &[]).unwrap().0, 2);
        let g = r#"[{"begin":"x","end":null,"severity":"high","external_desc":"Vertex Gemini API errors","affected_products":[{"title":"Vertex Gemini API"}]},{"end":"2026","severity":"high","affected_products":[{"title":"Compute"}]}]"#;
        assert_eq!(judge(Kind::Google, g, &["gemini"]).unwrap().0, 2);
        assert_eq!(judge(Kind::Google, r#"[{"end":null,"severity":"low","affected_products":[{"title":"BigQuery"}]}]"#, &["gemini"]).unwrap().0, 0);
        assert_eq!(judge(Kind::Aws, "[]", &[]).unwrap().0, 0);
        assert_eq!(judge(Kind::Aws, r#"[{"status":"3","service_name":"EC2","summary":"Increased error rates"}]"#, &[]).unwrap().0, 2);
        let ms = r#"{"Workloads":[{"Title":"Outlook.com","Status":"Operational"},{"Title":"Teams","Status":"Service degradation"}]}"#;
        assert_eq!(judge(Kind::Microsoft, ms, &[]).unwrap().0, 1);
        assert_eq!(judge(Kind::Microsoft, r#"[{"Title":"Outlook.com","Status":"Service restored"}]"#, &[]).unwrap().0, 0);
        let apple = r#"jsonCallback({"services":[{"serviceName":"iCloud Mail","events":[{"statusType":"Outage","eventStatus":"ongoing"}]}]});"#;
        assert_eq!(judge(Kind::Apple, apple, &[]).unwrap().0, 2);
        let fresh = chrono_like(now_secs() - 600.0);
        let rss = format!("<?xml version=\"1.0\"?><rss><channel><item><title>Elevated errors on API</title><description>Investigating</description><pubDate>{fresh}</pubDate></item></channel></rss>");
        assert_eq!(judge(Kind::Feed, &rss, &[]).unwrap().0, 1);
        let old = rss.replace("Investigating", "Resolved");
        assert_eq!(judge(Kind::Feed, &old, &[]).unwrap().0, 0);
        assert!(judge(Kind::Statuspage, "<html>", &[]).is_none());
    }
    fn chrono_like(t: f64) -> String {
        // back from seconds to an RFC 3339 time, through the same calendar
        let d = (t as i64).div_euclid(86400);
        let s = (t as i64).rem_euclid(86400);
        let z = d + 719468;
        let era = z.div_euclid(146097);
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
        format!("{y:04}-{m:02}-{day:02}T{:02}:{:02}:{:02}Z", s / 3600, (s / 60) % 60, s % 60)
    }
    #[test]
    fn times() {
        assert_eq!(parse_time("2026-11-13T22:10:00Z"), Some(1794607800.0));
        assert_eq!(parse_time("Fri, 13 Nov 2026 22:10:00 GMT"), Some(1794607800.0));
        assert_eq!(parse_time("Fri, 13 Nov 2026 23:10:00 +0100"), Some(1794607800.0));
        assert_eq!(parse_time("2026-11-13T23:10:00.123+01:00"), Some(1794607800.0));
    }
}
