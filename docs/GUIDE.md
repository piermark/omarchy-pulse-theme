# Omarchy Pulse — the guide

Everything about Pulse: what the scene means, how to put any live data on your wallpaper, the AI brief, cameras,
privacy, tuning and troubleshooting. The short version is in the [README](../README.md).

- [1. Install, update, remove](#1-install-update-remove)
- [2. The scene](#2-the-scene)
- [3. Panels and where they go](#3-panels-and-where-they-go)
- [4. Sources](#4-sources)
- [5. What a source can print](#5-what-a-source-can-print)
- [6. Levels, thresholds and alerts](#6-levels-thresholds-and-alerts)
- [7. Push from anywhere](#7-push-from-anywhere)
- [8. AI on your wallpaper](#8-ai-on-your-wallpaper)
- [9. Cookbook](#9-cookbook)
- [10. Built-in panels](#10-built-in-panels)
- [11. Privacy and security](#11-privacy-and-security)
- [12. Light on the PC](#12-light-on-the-pc)
- [13. Troubleshooting](#13-troubleshooting)
- [14. Files](#14-files)

---

## 1. Install, update, remove

Made for **Omarchy 4.5** (Hyprland, Wayland). Run these from a terminal on the PC itself, not over SSH.

```
omarchy-theme-install https://github.com/piermark/omarchy-pulse-theme
~/.config/omarchy/themes/pulse/extras/install-live
```

The first line installs the **pulse** theme: colours, window borders (cyan to magenta), btop, icons and two still
wallpapers (`01-pulse-skyline`, `02-pulse-orbit`; Omarchy's next-background key switches between them). The still is
what you see under the live wallpaper and when Pulse is off.

The second line makes it live. It installs what is missing (`python-gobject`, `gtk-layer-shell`, `webkit2gtk-4.1`,
`ffmpeg`, `playerctl`) with pacman, copies the engine to `~/.local/share/omarchy-pulse`, links the
`omarchy-pulse` command in `~/.local/bin`, asks your city for the weather, writes your config
`~/.config/omarchy-pulse/pulse.toml` (never overwritten later) and starts Pulse. From then on:

- it starts at every login (XDG autostart + a systemd user unit, `omarchy-pulse.service`);
- it follows the theme: pick **pulse** and it starts, pick another theme and it stops (Omarchy's theme-set hook).

| | |
|---|---|
| Live with every theme, in that theme's colours | `extras/install-live --always` |
| Back to the pulse theme only | `extras/install-live --theme` |
| Update | `omarchy-theme-install https://github.com/piermark/omarchy-pulse-theme` again, then `extras/install-live` |
| Remove the live wallpaper (your `pulse.toml` stays) | `extras/install-live --remove` |
| Check dependencies, config and sources | `omarchy-pulse doctor` |
| Logs | `journalctl --user -u omarchy-pulse -f` |
| Restart | `systemctl --user restart omarchy-pulse` |

## 2. The scene

Nothing in the scene is decoration only:

| You see | It is |
|---|---|
| the towers of the skyline | your CPU cores, one tower each, as tall as that core's load (red top above 90 %) |
| the lit windows | your RAM: the more is used, the more windows are on |
| the horizon line | your pulse: one beat per heartbeat, from 46 to about 140 a minute as the CPU works |
| the wireframe core | beats with the pulse and turns faster with the load |
| the grid on the floor | runs toward you as fast as the CPU works |
| the aurora | your network traffic |
| the sky | your real weather: stars on a clear night, rain, snow, fog, lightning in a storm |
| a red or amber shockwave | an alert (section 6), with the **INCOMING** banner |

Scenes (`scene =` in `[pulse]`): **skyline** (default), **orbit** (each core becomes a moon on its own ring around a
larger core; faster moons, busier cores) and **calm** (a still grid, at most 10 frames a second).

## 3. Panels and where they go

Every source is a panel in the Omarchy box style (title cut into the border). Four places:

| `slot` | Where | Good for |
|---|---|---|
| `left` | the left column, under system and network | values, gauges, tables |
| `right` | the right column, under the weather | cameras, status tiles, lists |
| `center` | under the clock | the AI brief, text, notifications |
| `ticker` | the line at the bottom, one headline every 6 seconds | news, prices, feeds |
| `auto` (default) | cameras and tiles right, text center, the rest wherever there is more room | |

`order` sorts panels in a column (smaller first). A panel that does not fit in its column is hidden, never cut in
half: put the important ones first. The age of the data (`2m`, `1h`) is shown top right; an error appears in amber
at the bottom of its panel.

## 4. Sources

`~/.config/omarchy-pulse/pulse.toml`. When you save it, Pulse reloads in about 3 seconds (or run
`omarchy-pulse reload`). A typo does not take the wallpaper down: a red **pulse.toml** panel tells you where.

```toml
[[source]]
name = "nas"                  # unique; also the name you push to
title = "nas · disk"          # shown on the panel (default: name)
icon = "󰋊"                    # any Nerd Font glyph (Omarchy's font has them all)
command = "ssh nas df --output=pcent /volume1 | tail -1"
every = 300                   # seconds
slot = "left"
warn_above = 80
crit_above = 92
```

Exactly one of these says where the data comes from:

| Key | What happens |
|---|---|
| `command = "..."` | runs in bash every `every` seconds (default 30); what it prints is the data. A failing command with no output shows its error. |
| `stream = "..."` | starts once and keeps running; every line it prints is an update. Restarted if it stops (3 s, then up to 60 s). For text/list/status panels the last `lines` lines (default 8) are kept. |
| `url = "..."` | HTTP GET every `every` seconds. JSON (pick a part with `path`), text or an image. `headers = { Authorization = "Bearer ..." }`. |
| `file = "..."` | read again whenever it changes (checked every 2 s). Text, JSON or an image (`.jpg .png .webp .gif`). |
| `camera = "..."` | a live thumbnail: one ffmpeg kept open (`rtsp://`, `rtsps://`, `http(s)://` video, anything ffmpeg opens), `fps` frames a second (default 2), `width` pixels (default 640), `transport = "udp"` if your camera needs it. With `snapshot = true` the URL is an image fetched every `every` seconds instead. |
| `push = true` | runs nothing; it waits for `omarchy-pulse push NAME ...` (section 7). |

The other keys:

| Key | Default | |
|---|---|---|
| `title`, `icon` | name, none | |
| `type` | `auto` | force `stat`, `gauge`, `list`, `status`, `text`, `kv`, `chart`, `image` |
| `slot`, `order` | `auto`, file order | section 3 |
| `every` | 30 | seconds between runs/requests |
| `timeout` | 60 (url: 20) | seconds before a run is given up |
| `path` | | for `url`: `data.items.0.value`, `result[2].name` |
| `unit` | | shown next to a value when the source does not say it |
| `warn_above`, `crit_above`, `warn_below`, `crit_below` | | section 6 |
| `notify` | false | also a desktop notification when the panel turns amber/red |
| `private` | false (cameras: true) | hidden while you share the screen (section 11) |
| `lines` | 8 | for `stream` text/list panels |
| `enabled` | true | `false` keeps the source in the file but off |

`command` and `stream` run with `~/.config/omarchy-pulse/sources/` and the built-in helpers on the PATH, and with
`PULSE_THEME` set to the current theme. Tip: TOML `'''...'''` strings take quotes and backslashes as they are, so a
long `curl | jq` pipeline needs no escaping.

Built-in helpers (in `~/.local/share/omarchy-pulse/sources/`):

- `pulse-ping HOST...`: one status tile per host with its ping time; `gateway` means your router; amber above 150 ms,
  red when it does not answer.
- `pulse-systemd`: failed system and user units (red tiles), or the number of running services.
- `pulse-table`: tab-separated (or 2+ space) lines on stdin to a two-column table.
- `pulse-ai`: section 8.

## 5. What a source can print

Pulse looks at the output and picks the panel:

| Output | Panel |
|---|---|
| `42` · `21.5 °C` · `1.2 kW` (a number, optional unit) | **stat**: big value + sparkline of the last 90 values |
| any other text | **text**, typed out when it changes (the last lines of a `stream`) |
| `{"value": 72, "unit": "%", "note": "step 4/6"}` | **stat**; with `"type": "gauge"` (or `type = "gauge"`) a segmented bar, `min`/`max` default 0–100 |
| `{"items": [{"title": "...", "text": "...", "right": "...", "level": "ok"}]}` or a JSON list | **list** |
| the same with `"type": "status"` (or `type = "status"`) | **status** tiles; `"note"` on the right, `"ok": true/false` works too |
| `{"rows": [["immich", "up 3d"], ["n8n", "up 6h"]]}` | **kv** table |
| `{"series": [{"values": [1, 3, 2]}, {"values": [2, 2, 4]}]}` | **chart** (up to 4 lines) |
| `{"text": "..."}` | **text** |
| any other JSON object | a table of its keys |
| an image (url, file, camera, push) | **image** |

Any JSON may also carry `"title"` (replaces the panel title unless you set one), `"level"` and `"note"`. In items,
`"t"` (a Unix time) shows as an age (`5m`, `2h`). Plain lines with `type = "list"` or `"status"` become items.

## 6. Levels, thresholds and alerts

Every panel and item can have a level: **ok** (green), **info** (cyan), **warn** (amber), **crit** (red, blinking).

- From the data: `"level": "crit"` in the JSON, or `--level crit` on a push.
- From thresholds, for numbers: `warn_above`, `crit_above`, `warn_below`, `crit_below`. Above/below the line the panel
  turns amber/red, back to ok on its own.

When a panel **becomes** warn or crit, Pulse raises an alert: a shockwave runs through the city, the horizon turns
red, and **INCOMING** shows the title and the text for 7 seconds (decrypted letter by letter); the alert also goes in
the ticker for an hour. With `notify = true` you also get a desktop notification. Built-in alerts: the CPU above
`temp_crit` (90 °C by default) and the root disk above 95 %.

## 7. Push from anywhere

Any script, cron job, git hook, CI step or home automation can put something on the wallpaper. A name that is not in
`pulse.toml` gets a panel of its own; a configured name keeps its title, slot and thresholds.

```
omarchy-pulse push deploy 72 --type gauge --title "deploy · web"
omarchy-pulse push backup "38 GB · 04:12" --level ok
make && omarchy-pulse push build ok --level ok || omarchy-pulse push build "failed" --level crit
omarchy-pulse push doorbell ~/Pictures/doorbell.jpg            # an image
journalctl -n 5 -p err -o cat | omarchy-pulse push errors -      # stdin
omarchy-pulse push news '{"items":[{"title":"Omarchy 4.5 is out"}]}' --slot ticker
```

Options: `--title`, `--type`, `--level`, `--slot`. Over HTTP (local only):

```
curl -s -X POST -H "X-Pulse-Token: $(cat ~/.config/omarchy-pulse/token)" \
     --data '{"value": 21.5, "unit": "°C"}' "http://127.0.0.1:47800/api/push/garage?level=ok"
curl -s -X POST -H "X-Pulse-Token: $(cat ~/.config/omarchy-pulse/token)" -H "Content-Type: image/jpeg" \
     --data-binary @snap.jpg http://127.0.0.1:47800/api/push/snapshot
```

Other endpoints: `GET /api/state` (everything the wallpaper shows, as JSON), `POST /api/privacy` (`on`, `off`,
`auto`, `toggle`), `POST /api/reload`. All need the token.

## 8. AI on your wallpaper

Any AI you can run from a terminal can write a panel. `pulse-ai` (ships with Pulse) gives it everything the wallpaper
shows right now — system, weather, every panel, their levels and errors, the last alerts — and asks for at most four
short lines on what deserves your attention first.

```toml
[[source]]
name = "ai"
title = "ai · brief"
icon = "󰚩"
command = "pulse-ai claude -p"
every = 1800
timeout = 180
slot = "center"
```

| AI | command |
|---|---|
| Claude Code | `pulse-ai claude -p` |
| a local model with ollama (nothing leaves the PC) | `pulse-ai ollama run llama3.2` |
| OpenAI Codex CLI | `pulse-ai codex exec` |
| Gemini CLI | `pulse-ai gemini -p` |
| any other CLI that takes the prompt as its last argument | `pulse-ai my-ai --flag` |

- Your own question: `pulse-ai --ask "Anything wrong with my servers? One line." ollama run llama3.2`
- See what would be sent, without sending it: `pulse-ai --dry-run`
- Private panels (cameras, notifications, anything `private = true`) are **left out** unless you add `--private`.
  Cloud AIs receive what you send them: use a local model when the data should stay home.
- The panel named `ai` never reads itself (`PULSE_AI_SELF` changes the name).
- Not only briefs: any command whose output is text is a panel, so `command = "claude -p 'One line on today in tech'"`
  or a script that asks your AI about your inbox work too. Keep `every` long (AI calls cost time and money).

## 9. Cookbook

Ready-made community sources (copy a folder, paste the TOML): [`community/`](../community/).

**Zabbix: current problems as tiles** (API token from *User settings → API tokens*)

```toml
[[source]]
name = "zabbix"
title = "zabbix"
icon = "󰒋"
type = "status"
every = 30
slot = "right"
notify = true
command = '''curl -s -H "Authorization: Bearer ZABBIX_TOKEN" -H "Content-Type: application/json-rpc" \
  -d '{"jsonrpc":"2.0","method":"problem.get","params":{"output":["name","severity"],"recent":true,"sortfield":["eventid"],"sortorder":"DESC","limit":8},"id":1}' \
  https://zabbix.example.lan/api_jsonrpc.php \
  | jq '{items: [.result[] | {title: .name, level: (if (.severity|tonumber) >= 4 then "crit" elif (.severity|tonumber) >= 2 then "warn" else "info" end)}]}' '''
```

**Home Assistant: any entity** (long-lived token from your profile)

```toml
[[source]]
name = "power"
title = "home · power"
icon = "󱐋"
url = "http://homeassistant.local:8123/api/states/sensor.house_power"
headers = { Authorization = "Bearer HA_TOKEN" }
path = "state"
type = "stat"
unit = "W"
warn_above = 3000
every = 10
```

**Security cameras**

```toml
[[source]]
name = "cam-gate"
title = "cam · gate"
camera = "rtsp://user:password@192.168.1.50:554/stream2"   # the camera's small stream is plenty
fps = 2

[[source]]
name = "cam-garden"
camera = "http://go2rtc.local:1984/api/frame.jpeg?src=garden"   # or any snapshot URL
snapshot = true
every = 2
```

**Proxmox: VMs and containers**

```toml
[[source]]
name = "proxmox"
type = "status"
every = 60
command = '''ssh root@pve pvesh get /cluster/resources --type vm --output-format json \
  | jq '{items: map({title: .name, note: .status, level: (if .status == "running" then "ok" else "warn" end)})}' '''
```

**GitHub Actions: the last runs of a repo**

```toml
[[source]]
name = "ci"
title = "ci · omarchytv"
type = "status"
every = 120
command = '''gh run list -R piermark/omarchytv -L 6 --json displayTitle,status,conclusion \
  | jq '{items: map({title: .displayTitle, level: (if .conclusion == "success" then "ok" elif .conclusion == "failure" then "crit" elif .status != "completed" then "info" else "warn" end)})}' '''
```

**MQTT: a sensor that keeps the panel live**

```toml
[[source]]
name = "garage"
title = "garage · °C"
stream = "mosquitto_sub -h 192.168.1.10 -t home/garage/temperature"
type = "stat"
crit_above = 45
```

**A log that never stops**

```toml
[[source]]
name = "errors"
stream = "journalctl -f -p err -o cat"
type = "text"
lines = 6
slot = "center"
```

**News in the ticker (RSS)**

```toml
[[source]]
name = "hn"
title = "HN"
slot = "ticker"
type = "list"
every = 900
command = '''curl -s https://hnrss.org/frontpage | python3 -c "import sys,xml.etree.ElementTree as E; print('\n'.join(i.findtext('title') for i in E.parse(sys.stdin).iter('item')))" | head -8'''
```

**Docker, pacman updates, hosts, failed units**

```toml
[[source]]
name = "docker"
command = "docker ps --format '{{.Names}}\t{{.Status}}' | pulse-table"
every = 30

[[source]]
name = "updates"
command = "checkupdates 2>/dev/null | wc -l"     # pacman-contrib
warn_above = 50
every = 1800

[[source]]
name = "lan"
type = "status"
command = "pulse-ping gateway nas.local pve.local 1.1.1.1"
every = 20

[[source]]
name = "services"
type = "status"
command = "pulse-systemd"
every = 60
```

**Backups from cron** (nothing in `pulse.toml` needed)

```
0 4 * * * restic backup ~ && omarchy-pulse push backup "ok $(date +\%H:\%M)" --level ok || omarchy-pulse push backup "FAILED" --level crit
```

## 10. Built-in panels

In `[builtin]` (`false` hides one):

| Key | Panel |
|---|---|
| `system` | CPU (with every core and a sparkline), load, uptime, RAM, GPU (AMD through sysfs, NVIDIA through `nvidia-smi`; not shown on other GPUs), CPU temperature, disk, battery |
| `network` | download/upload with sparklines, `user@host`, local IP |
| `weather` | Open-Meteo, no key: `city = "Milan"` (or `latitude`/`longitude`) in `[pulse]`, `units = "imperial"` for °F and mph |
| `media` | now playing (playerctl): title, artist, progress |
| `notifications` | the last desktop notifications (mako); private |
| `github = ["owner/repo", ...]` | latest release and commits of the repos you follow (`[]` to hide) |
| `github_notifications` | your GitHub notifications, after `gh auth login` |

`[pulse]` also has: `clock = "12h"`, `title = "MISSION CONTROL"` (a word under the clock), `font`, `port`
(default 47800), `temp_crit`, `dim_with_windows`.

## 11. Privacy and security

- **Screen sharing**: when you share or record the screen, Hyprland tells Pulse and the **privacy mode** starts by
  itself: cameras, notifications, your IP and every `private = true` panel are covered, the INCOMING banner hides its
  text, and a `◉ PRIVACY · screen shared` badge appears. It ends when you stop sharing.
  (`privacy_on_screenshare = false` turns this off.)
- **By hand**: `omarchy-pulse privacy on | off | auto | toggle`. Bind it to a key in your Hyprland config to cover
  the desktop before a screenshot.
- The hub listens on **127.0.0.1 only**, answers only requests addressed to that address, and every API call needs
  the token in `~/.config/omarchy-pulse/token` (created with mode 600).
- Camera addresses and passwords stay in the hub: the page only sees `/img/name`, and errors are shown with the
  password replaced by `***`. In privacy mode the hub does not even send camera frames.
- Sources run as you, with your permissions: only put commands you trust in `pulse.toml`.

## 12. Light on the PC

- 20 frames a second only while you look at an empty desktop (`fps`), 8 with windows open, almost nothing with an app
  full screen; at most 12 on battery; panels refresh once a second; no CSS animations (blinking happens once a
  second, with the data).
- A PC that cannot keep up gets fewer frames by itself.
- `scene = "calm"`: a still grid at 10 frames a second.
- `gpu = "never"` if the wallpaper is heavy on a PC without a working GPU driver (with no GPU device at all Pulse does
  that by itself); `"always"` to force the GPU.
- WebKitGTK uses about 400 MB of memory with one monitor. Every monitor gets its own page (only the first shows
  INCOMING).
- Check it in `btop`: the `WebKitWebProcess` and `python3` processes.

## 13. Troubleshooting

| Problem | Try |
|---|---|
| Still a still wallpaper | `systemctl --user status omarchy-pulse`, `journalctl --user -u omarchy-pulse -e`, `omarchy-pulse doctor`. Is the theme **pulse** (or `--always` set)? Run `extras/install-live` from a terminal on the PC, not SSH. |
| `missing dependencies` in the log | `extras/install-live` again (it installs them). |
| Black or empty on NVIDIA | Pulse already sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` on NVIDIA; also try `gpu = "never"`. |
| A panel does not appear | read the amber line at its bottom; run the command yourself in a terminal; `omarchy-pulse reload`. |
| A panel is missing from a full column | it does not fit: lower its `order`, move it to another `slot`, or turn something off. |
| The weather says `set city` | `city = "Your city"` in `[pulse]`. |
| `omarchy-pulse: command not found` | `~/.local/bin` must be on your PATH (it is on Omarchy). |
| Port 47800 busy | `port = 47801` in `[pulse]`. |
| Too heavy | lower `fps`, `scene = "calm"`, `gpu = "never"`, fewer cameras or a lower camera `fps`. |

## 14. Files

```
~/.config/omarchy/themes/pulse/            the theme (colours, borders, btop, stills, extras/)
~/.local/share/omarchy-pulse/              the engine: omarchy-pulse, pulse-live, web/, sources/
~/.local/bin/omarchy-pulse                 the command
~/.config/omarchy-pulse/pulse.toml         your sources
~/.config/omarchy-pulse/sources/           your own scripts (on the sources' PATH)
~/.config/omarchy-pulse/token              the local API token
~/.config/omarchy-pulse/live.conf          ALWAYS=1 with --always
~/.config/systemd/user/omarchy-pulse.service
~/.config/autostart/omarchy-pulse.desktop
~/.config/omarchy/hooks/theme-set.d/omarchy-pulse
```

MIT, © 2026 Marco Porro ([x.com/MarcoPorro](https://x.com/MarcoPorro)). Weather by [Open-Meteo](https://open-meteo.com).
