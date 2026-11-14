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
- New in 1.1:
  [15. The LED sphere](#15-the-led-sphere) ·
  [16. AI agents as drones](#16-ai-agents-as-drones) ·
  [17. Music, fireworks and focus](#17-music-fireworks-and-focus) ·
  [18. Colours, news, clicks and the sun](#18-colours-news-clicks-and-the-sun) ·
  [19. The face that sees you, and "while you were away"](#19-the-face-that-sees-you-and-while-you-were-away) ·
  [20. Your network on the grid](#20-your-network-on-the-grid) ·
  [21. The voice](#21-the-voice)
- New in 1.2:
  [22. Your real city](#22-your-real-city) ·
  [23. The Git city](#23-the-git-city) ·
  [24. Space over your city](#24-space-over-your-city) ·
  [25. Your heartbeat](#25-your-heartbeat) ·
  [26. The crew](#26-the-crew) ·
  [27. Hey sphere](#27-hey-sphere)
- New in 1.3:
  [28. The city that grows with you](#28-the-city-that-grows-with-you) ·
  [29. The planes over you](#29-the-planes-over-you) ·
  [30. Your day as a video](#30-your-day-as-a-video) ·
  [31. Your calendar is a station](#31-your-calendar-is-a-station) ·
  [32. Your home](#32-your-home) ·
  [33. Earthquakes and tsunamis](#33-earthquakes-and-tsunamis)
  [34. The wind](#34-the-wind)
  [35. Your electric car](#35-your-electric-car)
- [36. Every command](#36-every-command)
- New in 1.4:
  [37. The real night sky](#37-the-real-night-sky) ·
  [38. Starlink trains](#38-starlink-trains) ·
  [39. Every kind of weather](#39-every-kind-of-weather) ·
  [40. The air you breathe](#40-the-air-you-breathe) ·
  [41. The sound of your city](#41-the-sound-of-your-city) ·
  [42. The lights of your room, and Alexa](#42-the-lights-of-your-room-and-alexa) ·
  [43. Your phone](#43-your-phone) ·
  [44. Pulse Wrapped](#44-pulse-wrapped) ·
  [45. Robots on the floor](#45-robots-on-the-floor) ·
  [46. The banner plane](#46-the-banner-plane)
- New in 1.5:
  [47. Community scenes](#47-community-scenes-15) ·
  [48. The holidays](#48-the-holidays-15) ·
  [49. Eclipses](#49-eclipses-15) ·
  [50. The code construction site](#50-the-code-construction-site-15) ·
  [51. What expires](#51-what-expires-15) ·
  [52. The weather of your infrastructure](#52-the-weather-of-your-infrastructure-15) ·
  [53. Home Assistant with the age of every sensor](#53-home-assistant-with-the-age-of-every-sensor-15) ·
  [54. The energy of your home](#54-the-energy-of-your-home-15) ·
  [55. The noir scene](#55-the-noir-scene-15) ·
  [56. The hacker scene and the terminal](#56-the-hacker-scene-and-the-terminal-15) ·
  [57. Pulse Net](#57-pulse-net-15-beta-read-only) ·
  [58. NOC mode](#58-noc-mode-15-beta) ·
  [59. Reduced motion](#59-reduced-motion-15) ·
  [60. Your own wallpaper](#60-your-own-wallpaper-151)
- New in 1.6:
  [61. The scene editor](#61-the-scene-editor-16) ·
  [62. All your AI agents](#62-all-your-ai-agents-their-health-and-working-together-16) ·
  [63. Move and hide the panels](#63-move-and-hide-the-panels-16) ·
  [64. The workspaces tower and the ivy](#64-the-workspaces-tower-and-the-ivy-16) ·
  [65. Omarchy tips on a hot-air balloon](#65-omarchy-tips-on-a-hot-air-balloon-16) ·
  [66. Updates as parcels](#66-updates-as-parcels-and-the-news-before-you-update-16) ·
  [67. The colours of your music](#67-the-colours-of-your-music-16) ·
  [68. A theme from your wallpaper](#68-a-theme-from-your-wallpaper-16) ·
  [69. Your day's diary, and your week](#69-your-days-diary-and-your-week-16) ·
  [70. Ask the sphere, with a local AI](#70-ask-the-sphere-with-a-local-ai-16) ·
  [71. WLED](#71-wled-your-led-strips-16) ·
  [72. News that strikes](#72-news-that-strikes-16) ·
  [73. Backups as trucks, a second monitor, focus, the radar](#73-backups-as-trucks-a-calmer-second-monitor-focus-and-the-radar-16) ·
  [74. The friends' neighbourhood](#74-the-friends-neighbourhood-16) ·
  [75. Creator: pictures, animations, actions, .pulse files and the Community (1.7)](#75-creator-pictures-animations-actions-pulse-files-and-the-community-17) ·
  [76. The look (1.8)](#76-the-look-18) ·
  [77. Your datacenter is a district of the city (1.8)](#77-your-datacenter-is-a-district-of-the-city-18) ·
  [78. Describe your world (1.8)](#78-describe-your-world-18) ·
  [79. Monolith, a scene that comes with Pulse (1.8)](#79-monolith-a-scene-that-comes-with-pulse-18) ·
  [80. Mars, another world (1.8)](#80-mars-another-world-18)
- pulse-native: [81. Pulse native, the same wallpaper in Rust (1.1)](#81-pulse-native-the-same-wallpaper-in-rust-11)
- Creating scenes: with the mouse, [docs/EDITOR.md](EDITOR.md) (1.6); in code, [docs/SCENES.md](SCENES.md)

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

On a PC (x86_64) it also asks once: **draw it natively?** Yes installs **pulse-native**, the same wallpaper drawn on
the GPU in Rust, without a browser (lighter, smoother, and with the worlds only it has: section 81). It downloads a
ready binary from the release and checks it before it runs; if it does not start on your machine, the web wallpaper
takes over again by itself. No keeps the web wallpaper, as always (OmarchyTV always uses the web one). The answer
is kept, and one command changes it whenever you like:

| | |
|---|---|
| Live with every theme, in that theme's colours | `extras/install-live --always` |
| Back to the pulse theme only | `extras/install-live --theme` |
| Update | `omarchy-theme-install https://github.com/piermark/omarchy-pulse-theme` again, then `extras/install-live` |
| Remove the live wallpaper (your `pulse.toml` stays) | `extras/install-live --remove` |
| Draw it natively (pulse-native) / with the web page | `omarchy-pulse native on` / `omarchy-pulse native off` |
| Answer the question beforehand | `extras/install-live --native` or `extras/install-live --web` |
| Update pulse-native / which one draws | `omarchy-pulse native update` / `omarchy-pulse native status` |
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
| the LED sphere (scene **sphere**) | a screen: clock, weather, CPU, news, camera, alerts, music, focus, your agent's face (section 15) |
| the wireframe core | beats with the pulse and turns faster with the load; your agent's face lives in it |
| the drones | your AI agents and jobs at work (section 16) |
| the grid on the floor | runs toward you as fast as the CPU works |
| the aurora | your network traffic |
| the sky | your real weather: stars on a clear night, rain, snow, fog, lightning in a storm |
| a red or amber shockwave | an alert (section 6), with the **INCOMING** banner |

Scenes (`scene =` in `[pulse]`): **sphere** (the LED sphere in the middle of the city, the default from 1.1),
**skyline** (the city around a wireframe core), **orbit** (each core becomes a moon on its own ring around a
larger core; faster moons, busier cores) and **calm** (a still grid, at most 10 frames a second).
New in 1.5: **noir** (a futuristic megacity at night, a big screen instead of the sphere; section 55), **hacker**
(phosphor green or neon noir, a radar on your network; section 56) and the community scenes, such as **pond**, or
your own (section 47).

To change the scene, edit `scene` in `~/.config/omarchy-pulse/pulse.toml`, or in one line:

```
sed -i 's/^scene = .*/scene = "orbit"/' ~/.config/omarchy-pulse/pulse.toml    # sphere | skyline | orbit | calm | noir | hacker | pond | monolith | mars
```

The live wallpaper switches by itself within about 3 seconds (`omarchy-pulse reload` to do it now). Check the
current one with `grep ^scene ~/.config/omarchy-pulse/pulse.toml`.

The three still wallpapers of the theme (`01-pulse-skyline`, `02-pulse-orbit`, `03-pulse-sphere`, switched with Omarchy's
next-background key) are what you see while Pulse is off or starting; they do not change the live scene.

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
| `notifications` | the last desktop notifications (Omarchy 4.5 or mako); private |
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
- In privacy mode the drones, the names and addresses of the cars and the private lines of "while you were away"
  are hidden, and the voice is silent. MAC addresses never leave the hub; the pointer position never leaves the PC.
- 1.5: community scenes run in a sandbox with no network, no token and no access to Pulse, and get only a small
  read-only part of the state (no location, devices, calendar or notifications; no names while sharing). Eclipses and
  holidays are computed on your PC. Certificates are read from the servers you list, domains with whois. Pulse Net is
  off by default and only reads from your devices (secrets in files, TLS verified unless you say otherwise); no
  address and no MAC ever reaches the page, and no name while you share the screen; the syslog listener takes only
  the addresses you allow. The terminal's lines are hidden while sharing. The only write to another system is the
  Zabbix acknowledgement, off unless you add `ack`, and only with a double click.
- 1.4: the night sky is computed on your PC (nothing is sent). Starlink orbits are downloaded from CelesTrak (nothing
  about you is sent). Storms and air quality ask Open-Meteo about your city's coordinates, like the weather. The
  lights and the speakers are driven on your network (Home Assistant, the Hue bridge). Phone notifications go through
  ntfy.sh (or your own server) to your topic: anyone who knows the topic can read them, so keep it secret, and
  `details = false` leaves meeting titles and focus names out. The year's numbers stay in `stats.json`, planes only
  as short hashes.
- 1.3: the timelapse pictures are the scene alone, drawn without names (no repositories, devices, flights or events)
  and without an "auto" city; none are taken while you share the screen. The door camera never shows while you share
  the screen. Your Home Assistant token stays in `pulse.toml`; calendar titles are hidden in privacy mode.
- 1.2: in privacy mode the names of your repositories, your heart rate and an "auto" city skyline are hidden too. Your
  heart rate is never written to the log or sent anywhere. A "hey sphere" recording is deleted as soon as it is
  understood and never leaves the PC (only the text of a question goes to `ask`, if you set it). The crew sends only
  the kind of agent and its state (or the folder name with `share = "full"`), sealed with your code.

**1.6.** The agents' hooks (section 62) send only the event, the tool's name and the folder, never prompts, answers or
code; agents in the same repo are matched by a short hash of the folder. A local AI (section 70) and WLED strips
(section 71) are reached only at addresses on this PC or your network. The friends' neighbourhood (section 74) shares
only numbers, and only with `[crew] city = true`. Pulse reads the update news from archlinux.org every few hours and
runs `checkupdates` (section 66): it never installs anything and asks for no password. The scene editor saves only
into `~/.config/omarchy-pulse/scenes/`, and what it saves is data, not code (section 61).

## 12. Light on the PC

- Pulse pauses each monitor's drawing and panel updates when windows cover at least 95% of its usable area; the small
  allowance avoids treating normal window gaps as an exposed desktop. It resumes when the monitor is uncovered. Set
  `[pulse]` `pause_when_covered = false` to keep the original behavior. This uses window geometry, so translucent
  windows can appear covered even when the wallpaper is visible through them. If compositor data is unavailable,
  Pulse keeps drawing. Source collection and alert checks continue in the hub while a view is paused.
- 20 frames a second only while you look at an empty desktop (`fps`), 8 with windows open (`dim_with_windows = false`
  keeps full speed and full brightness under windows), almost nothing with an app
  full screen; at most 12 on battery; panels refresh once a second; no CSS animations (blinking happens once a
  second, with the data).
- A PC that cannot keep up gets fewer frames by itself.
- **How sharp** (`quality`, 1.3): the panels, the clock and every text are always at your screen's full resolution
  (HD or 4K); the scene behind (the city, the sphere, the planes) can be drawn at the screen's resolution or at
  half, which is four times fewer pixels.

  ```toml
  [pulse]
  quality = "auto"   # "full" sharp (up to 2x on HiDPI screens) | "half" light | "auto"
  ```

  `auto` draws it sharp, and at half on screens bigger than 2560x1440, on battery, or as soon as the frames get slow
  (it tries sharp again 10 minutes later). On a 4K screen with a strong GPU, try `quality = "full"` and check it
  with `omarchy-pulse perf`.
- `scene = "calm"`: a still grid at 10 frames a second.
- `gpu = "never"` if the wallpaper is heavy on a PC without a working GPU driver (with no GPU device at all Pulse does
  that by itself); `"always"` to force the GPU.
- WebKitGTK uses about 400 MB of memory with one monitor. Every monitor gets its own page (only the first shows
  INCOMING).
- Check it in `btop`: the `WebKitWebProcess` and `python3` processes.
- **Measure it**: `omarchy-pulse perf` shows, for every monitor, the frames per second it really gets, how long a
  frame takes to draw and how long it takes to reach the screen. When a frame takes much longer to reach the screen
  than to draw (for example 150 ms against 5 ms), the time goes into copying and presenting the frame, not into the
  scene: changing the scene does not help. Pulse then lowers that monitor's frames by itself so the CPU calms down,
  and writes a line in the log (`journalctl --user -u omarchy-pulse`).
- **NVIDIA, 4K and several monitors**: on NVIDIA, Pulse turns off WebKit's DMA-BUF renderer by default, because some
  drivers draw nothing with it. The price is that every frame is copied through the CPU, which is heavy at 4K and
  with two or more monitors (choppy motion, a busy `WebKitWebProcess`). With a recent driver try:

  ```toml
  [pulse]
  dmabuf = "on"        # "auto" (default: off on NVIDIA) | "on" | "off"
  ```

  then `systemctl --user restart omarchy-pulse`. If the wallpaper turns black or empty, go back to `"auto"`. If Pulse
  crashes right after starting with it (a Wayland "Error 71" in the log), it turns DMA-BUF off by itself after two such
  starts and says so in the log; delete `~/.local/state/omarchy-pulse/dmabuf-crash` to try it again.
- Laptops with two GPUs (1.5.2): `gpu_prefer = "discrete"` draws the wallpaper on the NVIDIA GPU (PRIME render offload)
  or on the AMD one (`DRI_PRIME=1`) instead of the integrated one. On a laptop where the integrated GPU struggles it can
  go from a few frames a second to the full rate; it uses more power, so it is off unless you choose it.
- **Measure without limits**: when Pulse has lowered a monitor's frames, `perf` ends up measuring that limit (2 fps
  looks like 500 ms). `omarchy-pulse perf --probe` lifts every limit for 16 seconds: switch to an empty workspace
  while it counts down (Hyprland slows down a covered wallpaper), and it prints the real numbers.

## 13. Troubleshooting

| Problem | Try |
|---|---|
| Still a still wallpaper | `systemctl --user status omarchy-pulse`, `journalctl --user -u omarchy-pulse -e`, `omarchy-pulse doctor`. Is the theme **pulse** (or `--always` set)? Run `extras/install-live` from a terminal on the PC, not SSH. |
| `missing dependencies` in the log | `extras/install-live` again (it installs them). |
| "pulse-native did not start here" | the web wallpaper draws again by itself. `journalctl --user -u omarchy-pulse -e` says why; on Intel Ivy Bridge or Haswell OpenGL is chosen by itself; see [native/README.md](../native/README.md#why-it-may-not-start-or-look-wrong). `omarchy-pulse native on` tries again. |
| pulse-native: "does not match SHA256SUMS" or "the signature does not match" | the download was damaged or is not the release's: nothing was installed. Try again later (`omarchy-pulse native on`). |
| Slow on a laptop with two GPUs (a few frames a second) | `[pulse] gpu_prefer = "discrete"`: Pulse draws on the NVIDIA (PRIME offload) or AMD (`DRI_PRIME`) GPU; much faster when the integrated one struggles, more battery used. The log says `drawing on the discrete GPU`. |
| Pink accents that are not in your theme | update to 1.5.2: Pulse now finds the theme in `~/.local/state/omarchy/current/theme` and `/usr/share/omarchy/themes`, and reads themes that name their colours (`magenta`, `cyan`...) instead of numbering them. |
| Black or empty on NVIDIA | keep `dmabuf = "auto"` (Pulse turns WebKit's DMA-BUF renderer off on NVIDIA); also try `gpu = "never"`. |
| An old notification stays on the wallpaper | click the panel (or `omarchy-pulse notifications clear`) to empty it; only the last `notifications_hours` (3) are shown, Omarchy's own quick feedback ("layout set to tiling", from `omarchy-action`) leaves after `notifications_quick` (120) seconds, and a "battery low" one goes away by itself when you plug the charger in; `notifications = false` in `[builtin]` hides the panel. |
| The weather icon on the sphere is wrong | from 1.3 it follows the real weather (sun or moon, clouds, rain, snow, storm, fog); check `city`. |
| No buildings in my district | a focus must last 10 minutes or more (`min_minutes`) and run to the end: `omarchy-pulse focus 25`. |
| No planes | the weather's `city` gives the place; planes show only above your horizon; `source = "opensky"` if adsb.lol is down. |
| No timelapse at midnight | `ffmpeg` must be installed; the pictures are taken only while you see the wallpaper; `omarchy-pulse timelapse today`. |
| No trains | `[calendar] ics = [...]` (the secret iCal address) or `command`; only timed events in the next `hours`, no all-day ones. |
| The doorbell does nothing | `[home] url`, `token`, `doorbell` (a binary sensor or an event entity); `journalctl --user -u omarchy-pulse` shows HA errors. |
| No stars at night | the sky follows the real clouds (`cloud_cover`); on a clear night they come out after the Sun is 6° under the horizon. `[night] enabled = true`. Many stars may be behind your panels: `omarchy-pulse sky` says what is up. |
| No Starlink train | they are visible only a few days after a launch, at dusk or dawn, when the satellites are in the Sun and you are in the dark: `omarchy-pulse sky` shows the next one. Install the `sgp4` Python package for the most precise orbits (section 38). |
| No sound | `[sound] enabled = true`; it is silent in a meeting, in the quiet hours (`quiet`), while sharing or with an app full screen; only the main monitor plays it. |
| The lights do nothing | `[lights] enabled = true`, the `lights` list, and `[home] url` and `token` (or the Hue bridge with `omarchy-pulse lights pair`); `only_if_on = true` leaves the lights that are off alone; try `omarchy-pulse lights test`. |
| Nothing on my phone | `omarchy-pulse phone test`; the ntfy app must be subscribed to the same topic. |
| The car does not appear | `[car] enabled = true` and the Tessie `token` (or the Home Assistant entities); the log says `car: ...`. A car being driven is hidden. |
| No tsunami wave | it only comes when USGS marks an earthquake with a tsunami message; try `curl -X POST -H "X-Pulse-Token: $(cat ~/.config/omarchy-pulse/token)" "localhost:47800/api/fx?kind=quake&mag=7&tsunami=1"`. |
| A community scene shows nothing | `omarchy-pulse scene check NAME`, then `omarchy-pulse scene lab NAME`: its errors are listed there (and once a minute in the log, `scene NAME: ...`). |
| No eclipse | `omarchy-pulse sky` lists the next ones you can see from your city; most years have none or only a small one where you are. |
| A candle says `?` | the date could not be read: for a domain install `whois`; for a certificate check the host and port. |
| Pulse Net shows nothing | `[net] enabled = true`; the log says `net switches: ...`, `net fw: ...`; try `snmpbulkwalk -v2c -c public HOST 1.3.6.1.2.1.2.2.1.8` yourself; for a self-signed certificate `verify = false` or `ca_file`. |
| No syslog bolts | the device must send to this PC's address, UDP 5514 (or your `listen`), and be in `allow`; only errors and worse (`max_severity`). |
| The city looks soft | `quality = "full"` in `[pulse]` (section 12); `auto` draws it at half on battery and on screens above 2560x1440. |
| Choppy, or a `WebKitWebProcess` at 100% (NVIDIA, 4K, two monitors) | `quality = "half"`; `omarchy-pulse perf`; if frames take much longer to reach the screen than to draw, try `dmabuf = "on"` (section 12). |
| A panel does not appear | read the amber line at its bottom; run the command yourself in a terminal; `omarchy-pulse reload`. |
| A panel is missing from a full column | it does not fit: lower its `order`, move it to another `slot`, or turn something off. |
| The weather says `set city` | `city = "Your city"` in `[pulse]`. |
| `omarchy-pulse: command not found` | `~/.local/bin` must be on your PATH (it is on Omarchy). |
| Port 47800 busy | `port = 47801` in `[pulse]`. |
| Too heavy | lower `fps`, `scene = "calm"` or `"skyline"` (the sphere draws the most), `gpu = "never"`, fewer cameras or a lower camera `fps`. |
| The city does not dance | install cava (`sudo pacman -S cava`); it listens only while something plays (playerctl); `[music] input = "pipewire"` if `pulse` does not work; `omarchy-pulse doctor`. |
| No drone for Claude Code | check the hooks in `~/.claude/settings.json` (section 16) and that `omarchy-pulse` is on the PATH of the shell Claude Code uses. |
| Too many fireworks | `celebrate = false` in `[pulse]` (only `omarchy-pulse fireworks` still fires them). |
| The eyes do not follow the pointer | Hyprland only (Pulse reads the pointer from its socket); `eyes = true` in `[presence]`; `face` must not be `"off"`. |
| No "while you were away" | you must be away at least `digest_after` minutes; locking the screen counts, or add the hypridle listener (section 19). |
| No cars on the grid | `ip` must work (`iproute2`) and the network must be a private one (192.168.x, 10.x, 172.16-31.x); install `fping` to see every device. |
| A new-device alert for a known device | name it in `[lan] names`; a phone with a private Wi-Fi address gets a new one per network (turn it off for your home Wi-Fi). |
| My city is not drawn | `skyline = "auto"` needs the weather's `city` to be one of the cities in section 22; else set `skyline = "rome"` (or another); it is hidden while you share the screen. |
| No towers for my repositories | `[git] root = "~/your/folder"` or `repos = [...]`; `git` must be installed; the beacon colour needs `gh auth login`. |
| The ISS never shows | it only shows while it is really above your horizon (a few minutes, a few times a day); `city` must be set for the weather. |
| The crew sees nothing | the same `code` on every PC, at least 16 characters (`omarchy-pulse crew new`); `omarchy-pulse crew`; the relay must be reachable. |
| Hey sphere does not hear | `omarchy-pulse doctor` (whisper-cli or faster-whisper); test with `omarchy-pulse listen --file question.wav`; the microphone needs `pw-record` (PipeWire) or `arecord`. |
| The voice says nothing | `enabled = true` in `[voice]`; `omarchy-pulse say "test"`; `omarchy-pulse doctor` (piper-tts or espeak-ng); check `quiet`. |

**Preview without installing anything live**: `omarchy-pulse serve` runs only the hub and prints a URL to open in a
browser; `omarchy-pulse serve --demo` does the same with demo data (no personal data), as in the videos.

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
~/.local/state/omarchy-pulse/cava.conf     written by Pulse for cava (1.1)
~/.local/state/omarchy-pulse/lan.json      the devices already seen on each network (1.1)
~/.local/state/omarchy-pulse/voice.wav     the last line spoken (1.1)
~/.local/state/omarchy-pulse/listen.wav    a "hey sphere" recording, deleted as soon as it is understood (1.2)
~/.local/state/omarchy-pulse/city.json     the buildings of your district: when, how long (1.3)
~/.local/state/omarchy-pulse/timelapse/    today's pictures of the scene, made into a video at midnight (1.3)
~/.local/state/omarchy-pulse/stats.json    the numbers of your year, for Pulse Wrapped (1.4)
~/.local/state/omarchy-pulse/starlink.json the orbits of the last Starlink launches, from CelesTrak (1.4)
~/.config/omarchy-pulse/scenes/            your community scenes (1.5)
~/Videos/omarchy-pulse/pulse-DATE.mp4      your days as videos (1.3), a diary with their numbers (1.6)
~/Videos/omarchy-pulse/pulse-week-YEAR-WNN.mp4   your weeks in a minute (1.6)
~/.config/omarchy-pulse/layout.json        where you put the panels (1.6, omarchy-pulse layout edit)
~/.local/state/omarchy-pulse/days.json     the numbers of each day, kept 60 days (1.6)
~/.local/state/omarchy-pulse/agents.json   how your agents' last runs ended, for their health (1.6)
~/.local/state/omarchy-pulse/news-read.json   the update news you marked read (1.6)
~/.config/omarchy/themes/NAME/             a theme made by omarchy-pulse theme from-wallpaper (1.6)
~/.config/opencode/plugins/omarchy-pulse.js, ~/.pi/agent/extensions/omarchy-pulse.ts   (1.6, hook install)
```

MIT, © 2026 Marco Porro ([x.com/MarcoPorro](https://x.com/MarcoPorro)). Weather by [Open-Meteo](https://open-meteo.com).

---

## 15. The LED sphere

`scene = "sphere"` (the default from 1.1). A globe of LEDs stands in the middle of the city and turns. Every pixel
looks up its place on a screen painted live, so from your chair it reads as one continuous image, with the fine LED
grid, a softer edge, a halo in the colours it shows, its light on the plaza and its reflection on the wet floor.
What faces you stays readable while the sphere turns.

It changes program every 11 seconds, and something important takes over at once:

| Program | When |
|---|---|
| **alert** (red, with a big "!") | right after an alert (section 6) |
| **focus** (the countdown) | during focus mode (section 17) |
| **face** | while one of your agents works, waits, has finished or failed (section 16), or always with `face = "always"` |
| **music** (the spectrum) | while music plays and cava listens (section 17) |
| **clock**, **weather**, **cpu** (your CPU history), **news** (the latest headline or the AI brief) | in turn, the rest of the time |
| **camera** | in turn, when a camera panel exists (never in privacy mode) |

Two looks, `sphere =` in `[pulse]`: **real** (default: one continuous image, as from across the street) or **led**
(thousands of separate lights on a dark globe, the clock and the agent's eyes lit up in them).

It is the heaviest scene to draw: on a PC without a GPU, `scene = "skyline"` is lighter.

## 16. AI agents as drones

Every agent (or any long job) is a drone over the city, with a tag: its name, what it is doing, for how long.

| State | The drone |
|---|---|
| `working` | flies and scans the city with its beam; the face thinks |
| `waiting` | blinks amber with a "!": it needs you; the face looks up, surprised |
| `done` | lands with a green light and small fireworks; the face smiles; gone after 90 s |
| `failed` | lands red and raises an alert; the face is sad; gone after 90 s |
| `idle` | hovers, dim |

From anything:

```
omarchy-pulse agent grok working "reading the issue"
omarchy-pulse agent grok waiting "needs your OK"
omarchy-pulse agent grok done            # or failed, idle, gone
omarchy-pulse watch build -- make release      # working while it runs, done or failed with its exit code
curl -s -X POST -H "X-Pulse-Token: $(cat ~/.config/omarchy-pulse/token)" \
     "http://127.0.0.1:47800/api/agent/grok?state=working&text=reading"
```

**Claude Code**: add the hooks to `~/.claude/settings.json` (merge them with the hooks you already have):

```json
{
  "hooks": {
    "SessionStart":     [{ "hooks": [{ "type": "command", "command": "omarchy-pulse hook claude" }] }],
    "UserPromptSubmit": [{ "hooks": [{ "type": "command", "command": "omarchy-pulse hook claude" }] }],
    "PreToolUse":       [{ "matcher": "", "hooks": [{ "type": "command", "command": "omarchy-pulse hook claude" }] }],
    "Notification":     [{ "hooks": [{ "type": "command", "command": "omarchy-pulse hook claude" }] }],
    "Stop":             [{ "hooks": [{ "type": "command", "command": "omarchy-pulse hook claude" }] }],
    "SessionEnd":       [{ "hooks": [{ "type": "command", "command": "omarchy-pulse hook claude" }] }]
  }
}
```

One drone per project (`claude · <folder>`). The hook reads only the event, the tool name and the folder name: your
prompts, your code and the commands it runs never reach the wallpaper. It always exits at once and never blocks
Claude Code, even when Pulse is not running.

**Codex CLI**: in `~/.codex/config.toml`, `notify = ["omarchy-pulse", "hook", "codex"]`: a drone lands at the end of
every turn.

**Grok Build, Gemini CLI, Copilot CLI, Hermes Agent, OpenCode, Pi (1.6)**: section 62, or `omarchy-pulse hook setup`.
**Any other agent**: call `omarchy-pulse agent NAME STATE "text"` from its hooks or its scripts, or start it with
`omarchy-pulse watch NAME -- ...`.

Drones are hidden in privacy mode: agent names are often project names.

## 17. Music, fireworks and focus

**Music.** With [cava](https://github.com/karlstav/cava) installed (`sudo pacman -S cava`), the skyline, the horizon,
the floor, the aurora and the sphere move with what you play. Pulse starts cava only while something plays
(playerctl) and stops it after. In `[music]`: `enabled = "auto"` (default) | `"always"` | `"off"`, `input = "pulse"`
(works with PipeWire) or `"pipewire"`, `bars = 24`.

**Fireworks.** By themselves when an agent finishes, a focus session ends, or an alert goes back to green (a small
burst); `celebrate = false` turns those off. By hand, with an optional text in the sky:

```
omarchy-pulse fireworks "v1.1 shipped"
git push && omarchy-pulse fireworks "pushed"
```

**Focus mode.** `omarchy-pulse focus 25 "write the guide"`: every panel dims, the countdown goes on the sphere (or
around the core) and under the clock; at the end, fireworks and a desktop notification. `omarchy-pulse focus off`
stops it.

## 18. Colours, news, clicks and the sun

**Colours.** In `[colors]`: `preset = "theme"` (default: the colours of the Omarchy theme in use), or `synthwave`,
`matrix`, `amber`, `ice`, `vapor`, `inferno`; then any single colour on top: `bg`, `fg`, `accent`, `accent2`, `ok`,
`warn`, `crit`, `dim` (`accent = "#39e1ff"`).

**News.** The `news` source, on by default, shows the Omarchy releases and DHH's posts. `pulse-feed` reads any RSS
or Atom feed, merges several by date and marks what is new in the last 24 hours:

```toml
command = "pulse-feed 'Omarchy=https://github.com/basecamp/omarchy/releases.atom' 'DHH=https://world.hey.com/dhh/feed.atom' 'HN=https://hnrss.org/frontpage'"
```

**Clicks.** Any source can run a command when you click its panel on the desktop:
`click = "xdg-open https://zabbix.example.lan"`, `click = "alacritty -e journalctl -f"`. Only commands written in
your `pulse.toml` run: the page cannot send its own.

**The sun.** The sky follows sunrise and sunset from your weather: a warm dawn and dusk, a lighter day, the night
with its stars. `sun = false` keeps the night.

**Shooting stars** cross the sky when a notification or a news item arrives.

## 19. The face that sees you, and "while you were away"

**Eyes.** The face's eyes follow your pointer, on every monitor (Pulse reads it from Hyprland a few times a second,
only while a wallpaper is on screen). Move the pointer on an empty desktop and the face wakes up and looks at you,
even with no agent at work; after a few seconds it goes back to the sphere's programs.

**Sleep.** When you leave, the face falls asleep (closed eyes, z z z) and the wallpaper slows down to save power.
Pulse knows you left when:

- you lock the screen (hyprlock, swaylock or gtklock) or the Omarchy screensaver starts: by itself;
- hypridle tells it, with a listener in `~/.config/hypr/hypridle.conf` (the most precise, also without locking):

```
listener {
    timeout = 300
    on-timeout = omarchy-pulse away
    on-resume = omarchy-pulse back
}
```

- or you say so: `omarchy-pulse away` / `omarchy-pulse back`.

**While you were away.** When you are back the face wakes up and smiles, and a card shows what happened since you
left: the alerts, the agents that finished, the new devices on your network, the new posts, how many notifications
(without their text). Click it to close it, or it goes by itself. It also comes as a desktop notification, and the
voice can read it (section 21).

```toml
[presence]
eyes = true          # the eyes follow the pointer
lock = true          # lock screen / screensaver = away
digest_after = 2     # minutes away before a digest
notify = true        # the digest as a desktop notification too
```

## 20. Your network on the grid

Every device on your network is a car driving on the grid: two lights and a light trail, coming toward you or going
away. Named devices carry their name; a **new** device (never seen on this network before) blinks amber with its
address, raises an alert (INCOMING) and a notification.

Pulse reads the devices your PC knows (`ip neigh`, no root); with `fping` installed (`sudo pacman -S fping`) it also
looks for every device on the network every 5 minutes. On a network it has never seen it first learns: for 10 minutes
(and until its first full scan, with fping) the devices already there raise no alerts. Only private networks, and nothing leaves the PC; the MAC addresses stay in
`~/.local/state/omarchy-pulse/lan.json` and never reach the page.

```toml
[lan]
enabled = true
cars = true                  # false: no cars, only the alert
scan = "auto"                # "off": only the devices your PC already talks to
alert_new = true
names = { "192.168.1.5" = "nas", "aa:bb:cc:dd:ee:ff" = "tv" }   # by IP or MAC; else the name your router gives
every = 60
```

In privacy mode the cars drive without names or addresses.

## 21. The voice

Pulse can speak, offline: alerts, an agent that finishes, fails or needs you, the digest when you are back ("Welcome
back. While you were away, 25 minutes: claude, omarchytv finished; new device on your network"), the end of a focus.
Off by default.

```toml
[voice]
enabled = true
events = ["alerts", "agents", "digest", "focus"]
quiet = "23:00-08:00"        # never at night ("" = any time)
language = "it"              # en (default), it, es, fr, de
# model = "~/.local/share/piper/it_IT-paola-medium.onnx"
# command = "spd-say -e"     # or any command that reads the text on stdin
```

It uses [piper](https://github.com/rhasspy/piper) (a natural voice, on your PC) or, if piper is not there,
`espeak-ng` (`sudo pacman -S espeak-ng`). For piper: `yay -S piper-tts-bin` (or `pipx install piper-tts`), then put
a voice (the `.onnx` file and its `.onnx.json`) in `~/.local/share/piper/`, from
[the piper voices](https://huggingface.co/rhasspy/piper-voices): `en_US-lessac-medium` (American English, the one in
the demo video), `en_GB-alan-medium`, `it_IT-paola-medium` and many more. espeak-ng speaks American English by default
(`espeak_voice = "en-gb"`, `"it"`... to change it). Pulse plays it with `pw-play`, `paplay` or `aplay`.

It never speaks while you share or record the screen, while you are away (the digest waits for you), in focus mode,
or in the quiet hours. Private panels are never read aloud. Try it: `omarchy-pulse say "hello from Pulse"`.

**Your language** (1.3). With `language = "it"` (or `es`, `fr`, `de`) everything Pulse says by itself is in your
language: "C'è qualcuno alla porta", "standup tra 5 minuti", "Terremoto di magnitudo 6.8...", "L'auto è carica: 80
per cento", "Bentornato. Mentre eri via...". Names, meeting titles and places stay as they are. Pulse picks a piper
voice of that language by itself if you have one in `~/.local/share/piper/` (`it_IT-paola-medium` for Italian,
`es_ES-davefx-medium`, `fr_FR-siwis-medium`, `de_DE-thorsten-medium`), or espeak-ng in that language. "Hey sphere"
answers in Italian or English, the language you ask in; with `language = "it"` Italian is the default.

## 22. Your real city

Behind the towers stands the skyline of a real city, drawn with its landmarks: the Colosseum in Rome, the Duomo in
Milan, the Mole in Turin, the One World Trade Center in New York, the Eiffel Tower in Paris, Big Ben and the Shard in
London, the Burj Khalifa in Dubai, the Petronas Towers in Kuala Lumpur... Their lights blink red at the top, their
windows are lit. In front of a real city the CPU towers are a little lower, so the skyline reads.

```toml
[pulse]
skyline = "auto"     # the city of your weather (default); "off" for the generic city; or a city below
```

**Choose your city** in one line (any city of the list below), or go back to the weather's city:

```
sed -i 's/^skyline = .*/skyline = "new-york"/' ~/.config/omarchy-pulse/pulse.toml   # or "rome", "tokyo", "alessandria"...
sed -i 's/^skyline = .*/skyline = "auto"/' ~/.config/omarchy-pulse/pulse.toml       # the weather's city again
```

If your `pulse.toml` is older than 1.2 and has no `skyline` line, add `skyline = "rome"` under `[pulse]`. The wallpaper
changes in about 3 seconds. Or ask the sphere (section 27): "hey sphere, city Tokyo", "ehi sfera, città Roma" (until
Pulse restarts; `pulse.toml` decides after).

With `"auto"` Pulse uses the weather's `city` in any language ("Roma", "München", "São Paulo"); while you share
the screen it shows the generic city, so your skyline does not tell where you live. The cities:

- **Italy**: `rome`, `milan`, `turin`, `alessandria`, `florence`, `venice`, `naples`, `pisa`, `bologna`, `genoa`
- **Europe**: `london`, `paris`, `berlin`, `madrid`, `barcelona`, `lisbon`, `amsterdam`, `brussels`, `vienna`, `prague`, `budapest`, `warsaw`, `moscow`, `athens`, `istanbul`, `dublin`, `stockholm`, `copenhagen`, `oslo`, `helsinki`, `zurich`, `munich`, `cologne`, `frankfurt`, `kyiv`, `bern`, `reykjavik`, `edinburgh`
- **The Americas**: `new-york`, `washington`, `chicago`, `los-angeles`, `san-francisco`, `seattle`, `las-vegas`, `miami`, `boston`, `toronto`, `montreal`, `vancouver`, `mexico-city`, `sao-paulo`, `rio-de-janeiro`, `buenos-aires`, `ottawa`, `havana`, `lima`, `bogota`, `santiago`
- **Asia and the Middle East**: `tokyo`, `beijing`, `shanghai`, `hong-kong`, `shenzhen`, `seoul`, `singapore`, `kuala-lumpur`, `taipei`, `osaka`, `kyoto`, `hanoi`, `manila`, `jakarta`, `kathmandu`, `tehran`, `ankara`, `bangkok`, `mumbai`, `delhi`, `dubai`, `abu-dhabi`, `doha`, `riyadh`, `tel-aviv`, `cairo`
- **Oceania and Africa**: `sydney`, `melbourne`, `auckland`, `cape-town`, `johannesburg`, `lagos`, `nairobi`

They are original line drawings of public buildings: no photos and no logos.

## 23. The Git city

Your repositories become towers in the city, behind the CPU towers: the more commits this week, the taller the
tower, with its name and the number of commits. On top a beacon: **green** when GitHub Actions passed on that branch,
**red and blinking** when they failed (with an alert), **amber** while they run. Uncommitted changes make its lower
windows flicker. A **commit** lights the tower up; a **push** lights it up with fireworks over it.

```toml
[git]
root = "auto"        # ~/code, ~/src, ~/projects, ~/dev, ~/git, ~/repos, ~/work (and one folder deeper)
# repos = ["~/code/omarchytv", "~/code/omarchy-pulse-theme"]
ci = "auto"          # the beacon: needs gh and `gh auth login`; "off" without it
max = 8              # the busiest repositories this week
every = 60
```

Pulse reads your repositories locally with `git` (nothing is sent); the CI colour asks GitHub through `gh` every 5
minutes per repository. In privacy mode the towers have no names.

## 24. Space over your city

- **The ISS**: when the International Space Station really passes over your city (from its orbit, wheretheiss.at),
  a bright point crosses your sky in the right direction, with a trail, its height and how high it is above your
  horizon. A shooting star marks its arrival, and the voice can tell you ("The International Space Station is
  passing over you right now"). The wallpaper looks south: east is on the left, west on the right.
- **The aurora**: in a geomagnetic storm (NOAA's Kp index 4 and more) a green aurora appears over the city, stronger
  as the storm grows.
- **Solar flares**: a strong one (class M or X) flashes the sky warm and writes its class in the sky.
- **Rocket launches**: the next SpaceX launches (every one: Starlink, crew and cargo for the ISS, NASA, Starship) in
  the ticker with their countdown and status (Go, TBC, Hold...). In the last hour the sphere shows the T-minus; at
  liftoff a rocket climbs from your horizon with its flame and its trail (bigger for Starship), and the voice says
  "Liftoff!". Launch times come from [Launch Library 2](https://thespacedevs.com/llapi) (The Space Devs), accurate to
  the second when the launch is confirmed.

```toml
[space]
enabled = true
iss = true
launches = "spacex"          # "spacex" (all of them, Starlink included) | "all" (every provider) | "rocketlab" |
                             # "blueorigin" | "nasa" | "off"
launches_panel = "ticker"    # or "right", "left", "off"
```

The ISS needs the weather's `city` (or `latitude`/`longitude`). ISS passes and liftoffs go into "while you were away".
The free launch API allows 15 calls an hour: Pulse asks every 10 minutes.

## 25. Your heartbeat

The horizon is your pulse: from 1.2 it can beat with your heart. Connect any source that gives your heart rate (a
watch through Home Assistant, a script, a file); a small ♥ with the beats per minute appears under the clock.

```toml
[heart]
url = "http://homeassistant.local:8123/api/states/sensor.watch_heart_rate"
headers = { Authorization = "Bearer YOUR_TOKEN" }
# path = "attributes.bpm"     # if the number is deeper in the JSON (Home Assistant's "state" is read by itself)
# command = "my-heart-script" # or a command, or file = "~/heart.txt"
every = 10
```

Your heart rate stays on your PC: it is never logged, never sent, hidden while you share the screen, and after two
minutes without a value the horizon goes back to beating with the CPU.

## 26. The crew

Share a code with your friends and your cities are linked: their AI agents fly as drones in your sky (with a
**crew** tag and a magenta outline), yours in theirs; when one of their agents finishes, fireworks go off in your
city too, and `omarchy-pulse fireworks "shipped!"` goes off in all of them.

```
omarchy-pulse crew new         # makes a code; put it on every PC of the crew
```
```toml
[crew]
code = "the code"
name = "marco"
share = "agent"     # what travels: "agent" (claude, codex...) or "full" (with the project folder)
# relay = "https://ntfy.sh"    # the default relay; or your own ntfy server
```

The messages go through a relay ([ntfy](https://ntfy.sh), or your own) sealed with your code (encrypted and signed
with HMAC-SHA256): the relay sees only noise, and a message from someone without the code is thrown away. Only the
kind of agent and its state travel, never your prompts, your code or the commands. `omarchy-pulse crew` shows the
status; remove the code to leave the crew.

## 27. Hey sphere

Talk to the sphere. Bind a key and hold it (or press it once: it listens for 5 seconds):

```
# ~/.config/hypr/bindings.conf
bind  = SUPER, A, exec, omarchy-pulse listen start
bindr = SUPER, A, exec, omarchy-pulse listen stop
```

The face looks up and listens (a level meter for a mouth), thinks, then answers out loud and in words under the
sphere. It understands English and Italian:

| You say | It does |
|---|---|
| "focus 25 minutes" · "metti un focus di 25 minuti" | starts focus mode |
| "stop focus" · "ferma il focus" | stops it |
| "how are my servers?" · "come stanno i server?" | the panels that need attention, or "everything is fine" |
| "what did I miss?" · "cosa mi sono perso?" | the last "while you were away" |
| "how are my agents?" | each agent and its state |
| "is the ISS over me?" | where the ISS is |
| "weather" · "what time is it?" · "che ore sono?" | the weather, the time |
| "city Tokyo" · "città Roma" | the skyline of that city (until Pulse restarts) |
| "privacy on / off" · "fireworks" · "scene orbit" | just that |

It never runs anything else: what it does not understand gets a short list of what it can do. With `ask` set, those
questions go to an AI instead (only the text of the question), and it reads the answer:

```toml
[listen]
language = "auto"         # or "en", "it"
# model = "~/.local/share/whisper/ggml-base.bin"
# ask = "claude -p"       # or "ollama run llama3.2", nothing leaves the PC
```

Speech to text runs on your PC: [whisper.cpp](https://github.com/ggml-org/whisper.cpp) (`yay -S whisper.cpp` and a
model such as `ggml-base.bin` in `~/.local/share/whisper/`) or [faster-whisper](https://github.com/SYSTRAN/faster-whisper)
(`pipx install faster-whisper`: it downloads its model the first time). The recording goes to
`~/.local/state/omarchy-pulse/listen.wav` and is deleted as soon as it is understood; Pulse never listens on its
own. The answer uses the voice of section 21 (it speaks even with `[voice]` off, since you asked). Try it without a
microphone: `omarchy-pulse listen --file question.wav`.

## 28. The city that grows with you

Every focus you finish builds a building in your own district, in front of the city (`omarchy-pulse focus 25 "write
the guide"`): the longer the focus, the taller the building. The newest rises from the ground with a glow. While
your streak lives (a finished focus today or yesterday) the windows are lit; miss a day and the district goes dark
until your next focus. Every 7 days in a row a monument with a beacon rises beside it, with fireworks.

```
omarchy-pulse city      # Your city: 34 buildings · streak 5 days · best 12 · today 2 · 1 monuments
```
```toml
[city]
enabled = true
min_minutes = 10        # shorter focus sessions build nothing; a focus you stop before the end neither
```

Ask the sphere too: "hey sphere, how is my city?". The buildings are kept in `~/.local/state/omarchy-pulse/city.json`
(delete it to start again).

## 29. The planes over you

The real planes over your city cross your sky where they really are, with their red and green lights and the
white strobe and a contrail. Next to each one: the flight, the aircraft (A321, B738...) and the height, and below
where it comes from and where it goes (`FCO → LHR  Rome → London`, from [adsbdb](https://www.adsbdb.com), kept for a
day; only the flight code is sent, never your position). Pulse looks every 15 seconds within 25 nautical miles of the weather's
city, from [adsb.lol](https://adsb.lol) (free, open data) or OpenSky, or from your own ADS-B receiver.

```toml
[sky]
enabled = true
source = "adsb.lol"     # "opensky" (asked every 4 minutes: its free limit), or "http://localhost:8080/data/aircraft.json"
radius = 25
routes = true           # where each flight comes from and goes (adsbdb.com); false to never ask
```

Like the ISS, the wallpaper looks south: east on the left, west on the right; the higher a plane, the higher it flies
on the screen (the sky above the towers, since most planes are 10-40° up): straight above you is the top of the screen. So that the sky stays readable: only planes at least 10°
above your horizon, at most 6 at a time, the ones behind you (north) only when they are almost overhead, and a label
never covers another one. They fly in front of the sphere, never hidden behind it; a plane where a panel is flies
just beside the panel, at its real height, so you still see it.

## 30. Your day as a video

Every few minutes, while you see the wallpaper, Pulse keeps a picture of the scene; at midnight it makes them into a
short video of your day (about 30 seconds) in `~/Videos/omarchy-pulse/`, and tells you with a notification. It is
made to share: only the scene, no panels, no names (repositories, devices, flights, events), the generic city
when yours comes from the weather, and no pictures while you share the screen.

```
omarchy-pulse timelapse today          # the video of today so far
omarchy-pulse timelapse 2026-11-07     # a day still on disk
```
```toml
[timelapse]
enabled = true
every = 90              # seconds between pictures (90: about 30 seconds of video for a day)
# dir = "~/Videos/omarchy-pulse"
```

It needs `ffmpeg` (`sudo pacman -S ffmpeg`). The pictures of a day are deleted once its video is made.

## 31. Your calendar is a station

An elevated line runs across the city with a station on the left: each event of the next hours is a train of light
coming in, with its title and when. Five minutes before, the train blinks amber, its name is written in the sky and
the voice tells you ("standup in 5 minutes"); while the event runs, the train waits green at the station.

```toml
[calendar]
ics = ["https://calendar.google.com/calendar/ical/YOU/private-KEY/basic.ics"]   # or a file, or several
# command = "my-events"     # or a command printing [{"title": "review", "start": "2026-11-07T15:00:00+01:00"}]
warn = 5
hours = 3
every = 300
```

Google Calendar: Settings, your calendar, "Secret address in iCal format". Outlook: "Publish a calendar", ICS. Pulse
reads timed events (not all-day ones) with their daily and weekly repeats and exceptions. The address is secret: it
stays in your `pulse.toml`. Titles are hidden in privacy mode; ask "hey sphere, next meeting?", or run
`omarchy-pulse calendar`.

- **In the last minute** before a meeting the sphere counts it down ("MEETING IN 0:42").
- **While a meeting runs**, privacy mode is on by itself (you may share your screen), off again at the end
  (`privacy = false` to keep it off).
- **In the evening** (`summary_at = "18:00"`) a summary of your day, on the wallpaper, as a notification and aloud:
  "today: 4 meetings · 3 h 20 min" (overlapping meetings counted once). It goes into "while you were away" too.

```toml
[calendar]
privacy = true
summary_at = "18:00"    # "" = never
```

## 32. Your home

With [Home Assistant](https://www.home-assistant.io):

- **The doorbell**: when it rings, the sphere becomes the door camera for a minute, a "door" panel shows it too, a
  notification and the voice say "someone is at the door", and the doorbell rings on your speakers too (ding-dong,
  `chime = false` to keep it quiet). Never while you share the screen.
- **Your lights**: a "home" panel with your lights; click one to switch it on or off.

```toml
[home]
url = "http://homeassistant.local:8123"
token = "a long-lived access token"   # in Home Assistant: your profile, Security, Long-lived access tokens
doorbell = "binary_sensor.doorbell"   # a binary sensor, or an event entity (event.doorbell)
camera = "camera.front_door"
lights = ["light.living_room", "light.kitchen", "switch.desk_lamp"]
every = 2
```

Pulse only switches the lights you list. Try the doorbell without a doorbell: `curl -X POST
-H "X-Pulse-Token: $(cat ~/.config/omarchy-pulse/token)" http://127.0.0.1:47800/api/doorbell`.

## 33. Earthquakes and tsunamis

Real earthquakes ([USGS](https://earthquake.usgs.gov), every 5 minutes): the city shakes, stronger with the
magnitude and when it is near, and waves run across the floor from the direction where it happened, with its
magnitude, place and distance. A strong one near you (4 or more, within `near_km`) is an alert (INCOMING).

When a tsunami warning centre has issued a message for an earthquake (the `tsunami` flag of the USGS feed), the sea
at the foot of the city swells and a long wave crosses it from that side, with TSUNAMI written on it, and the voice
says so. Any tsunami message is shown; one closer than `tsunami_km` is a red alert. Pulse only shows the message:
for what to do, always follow your local authorities.

```toml
[quakes]
enabled = true
min_world = 6       # anywhere in the world
min_near = 3        # near you
near_km = 500
tsunami_km = 1500   # a tsunami message closer than this is a red alert
```

## 34. The wind

The wind of your weather (speed, gusts and direction, from Open-Meteo) blows in the scene: rain and snow slant and
drift with it, the way it really blows (you look south, so a west wind blows to the left), and from about 25 km/h gust
lines cross the sky. With strong wind the weather icon on the sphere gets gust lines and shows the wind and the gusts
(WIND 52 KM/H) in turn with the city. Nothing to set up: it comes with `city`. Try it with `?wind=50` on the page.

## 35. Your electric car

Your car is parked at the foot of the city: a light bar along its side is the battery (the white mark is the charge
limit), and while it charges a cable from the charger carries light into it. When it starts charging, or finishes,
the sphere shows the battery and the range for a few seconds; when it is charged the voice tells you. In the evening,
if the battery is low and the car is not plugged in, a reminder. A panel (slot `panel`) shows the battery and lets you
start or stop charging, lock it and turn the climate on or off, with one click. Ask "hey sphere, how is my car?",
or from a terminal:

```bash
omarchy-pulse car               # car: 64% · 312 km · charging · 11 kW · limit 80%
omarchy-pulse car charge        # or stop, lock, unlock, climate on, climate off, flash
```

Two ways to connect it:

- **Tessie** (a Tesla): in the Tessie app, Settings, API Access, make a token. Pulse reads Tessie's last state every
  minute, so it never wakes the car up and does not drain the battery.
- **Home Assistant** (any car Home Assistant knows: Tesla Fleet, Teslemetry, and the integrations of other brands):
  name the entities, with `url` and `token` from `[home]`.

```toml
[car]
enabled = true
source = "tessie"          # or "homeassistant"
token = "your Tessie API token"   # or token_file = "~/.config/omarchy-pulse/tessie"
# vin = ""                 # with more cars
low = 30                   # the evening reminder below this (evening = "21:00")
allow_unlock = false       # unlocking from the wallpaper stays off unless you turn it on

# source = "homeassistant":
# battery = "sensor.car_battery"
# range = "sensor.car_range"
# charging = "binary_sensor.car_charging"
# plugged = "binary_sensor.car_charge_cable"
# lock = "lock.car_doors"
# climate = "climate.car_climate"
# charge_switch = "switch.car_charge"
```

Privacy: Pulse never reads or keeps where the car is, nor its VIN; the car hides while you drive. The car drawn is a
generic one, of no brand.

## 36. Every command

**From a terminal** (`omarchy-pulse help` prints the same list):

| Command | What it does |
|---|---|
| `omarchy-pulse run` | the wallpaper (the systemd unit runs it for you) |
| `omarchy-pulse serve [--demo]` | only the hub, in a browser; `--demo` with demo data |
| `omarchy-pulse push NAME VALUE [--title T] [--type T] [--level L] [--slot S]` | anything onto the wallpaper (section 7) |
| `omarchy-pulse privacy on \| off \| auto \| toggle` | privacy mode (section 11) |
| `omarchy-pulse agent NAME STATE [TEXT]` | an AI agent as a drone (section 16) |
| `omarchy-pulse watch NAME -- COMMAND...` | a drone while a command runs |
| `omarchy-pulse hook claude \| codex \| grok \| gemini \| copilot \| hermes \| opencode \| pi` | for the hooks of your AI agents (16, 62) |
| `omarchy-pulse hook setup [AGENT]` / `hook install opencode \| pi` | what to paste in each agent's config / install the plugin (62) |
| `omarchy-pulse ask QUESTION` | ask the sphere in writing; a local Ollama for the rest (70) |
| `omarchy-pulse layout edit \| done \| reset` | move and hide the panels with the mouse (63) |
| `omarchy-pulse theme from-wallpaper [PICTURE] NAME` | a new Omarchy theme from a picture (68) |
| `omarchy-pulse news [read]` | the news to read before you update (66) |
| `omarchy-pulse native on \| off \| update [--check] \| status` | pulse-native, the native renderer: install the checked release and draw with it, back to the web wallpaper, the newest release, what is installed (1, 81) |
| `omarchy-pulse recap [DATE]` | your week in a minute (69) |
| `omarchy-pulse fireworks [TEXT]` | fireworks (in the whole crew, too) |
| `omarchy-pulse focus MINUTES [LABEL]` / `focus off` | focus mode; when it ends, a building in your city (28) |
| `omarchy-pulse away` / `back` | you left / you are back: the digest (19) |
| `omarchy-pulse say TEXT` | the voice (21) |
| `omarchy-pulse listen [SECONDS \| start \| stop \| --file F.wav]` | hey sphere (27) |
| `omarchy-pulse crew new` / `crew` | the crew code / its status (26) |
| `omarchy-pulse city` | your city: buildings, streak, monuments (28) |
| `omarchy-pulse calendar` | the next events, the trains (31) |
| `omarchy-pulse timelapse [today \| DATE]` | your day as a video, a diary with its numbers (30, 69) |
| `omarchy-pulse sky` | tonight: the Moon, the planets, the meteor shower, the next Starlink train, the next eclipses, the air (37-40, 49) |
| `omarchy-pulse wrapped [YEAR]` | Pulse Wrapped, your year as a video (44) |
| `omarchy-pulse lights pair [IP] \| test \| alert \| lightning \| quake` | the lights of your room (42) |
| `omarchy-pulse phone new \| test` | notifications on your phone (43) |
| `omarchy-pulse banner [TEXT]` | a little plane tows a banner across the sky (46) |
| `omarchy-pulse robot NAME ok \| working \| waiting \| failed \| gone` | an AI bot of yours as a robot on the floor (45) |
| `omarchy-pulse car [charge \| stop \| lock \| unlock \| climate on \| climate off \| flash]` | your car (35) |
| `omarchy-pulse doorbell` | ring the doorbell, to try it (32) |
| `omarchy-pulse notifications clear` | empty the notifications panel (a click on it does the same) |
| `omarchy-pulse perf [--probe]` | frame times on every monitor (12) |
| `omarchy-pulse reload` | read `pulse.toml` again now |
| `omarchy-pulse doctor` | check dependencies, config and sources |
| `omarchy-pulse scene list \| new NAME \| check NAME \| lab [NAME] \| editor [NAME]` | community scenes, the scene lab and the scene editor (47, 61, docs/SCENES.md) |
| `omarchy-pulse scene export NAME --author A --license MIT [--title T --description D --preview PNG -o FILE]` | 1.7: a scene as a .pulse file, to share (75, docs/PACKAGES.md) |
| `omarchy-pulse scene import FILE.pulse [--name N --force] \| gallery \| get NAME` | 1.7: install a .pulse file; the Community gallery; install one from it (75) |
| `omarchy-pulse scene describe NAME "what you want" [--agent "claude -p"] [--try]` | 1.8: your AI agent writes the scene (78) |
| `omarchy-pulse scene prompt "what you want" \| make NAME --from REPLY \| guide` | 1.8: the request for any AI; a scene from its reply; the guide agents read (78) |
| `omarchy-pulse term` | (beta) a read-only terminal on the city: scan, ping, status (56) |
| `omarchy-pulse noc on \| off \| ack` | (beta) NOC mode and its incidents (58) |
| `omarchy-pulse trap` | (beta) for snmptrapd's traphandle: SNMP traps as bolts (57) |

**Hey sphere** (English or Italian; section 27):

| Say | It does |
|---|---|
| "focus 25" · "concentrazione 25 minuti" / "stop focus" | starts or ends a focus |
| "privacy on" / "privacy off" | privacy mode |
| "fireworks" · "fuochi" | fireworks |
| "what did I miss?" · "cosa mi sono perso?" | the digest |
| "how is my city?" · "com'è la mia città?" | buildings and streak |
| "how is my car?" · "come sta la mia auto?" | battery, range, charging |
| "next meeting?" · "prossima riunione?" | the next event |
| "how is the moon?" · "com'è la luna?" | its phase, and whether it is up (37) |
| "next Starlink?" · "starlink" | the next train over you (38) |
| "my year" · "il mio anno" | the numbers of your year (44) |
| "where is the ISS?" · "stazione spaziale" | the ISS |
| "weather" · "che tempo fa?" | the weather |
| "what time is it?" · "che ore sono?" | the time |
| "agents?" · "cosa fanno gli agenti?" | your drones |
| "city Tokyo" · "città Roma" | your real city in the skyline (22) |
| "scene orbit" · "scena sfera" | the scene |
| "status" · "tutto bene?" | alerts and panels |

**From a script** (the token is in `~/.config/omarchy-pulse/token`, sent as `X-Pulse-Token`; only on 127.0.0.1):

```bash
T="X-Pulse-Token: $(cat ~/.config/omarchy-pulse/token)"
curl -X POST -H "$T" "localhost:47800/api/fx?kind=fireworks&text=shipped"
curl -X POST -H "$T" "localhost:47800/api/fx?kind=tip&text=SUPER%20%2B%20K%20%C2%B7%20Show%20key%20bindings"   # 1.6: a balloon now
curl -X POST -H "$T" "localhost:47800/api/fx?kind=lightning&text=CVE"    # 1.6: one bolt over the city
curl -X POST -H "$T" "localhost:47800/api/fx?kind=quake&mag=6.8&az=100&tsunami=1&text=test"   # try a quake and a tsunami
curl -X POST -H "$T" "localhost:47800/api/doorbell"
curl -X POST -H "$T" "localhost:47800/api/car?do=start_charging"    # stop_charging, lock, start_climate, stop_climate, flash
curl -X POST -H "$T" -d on "localhost:47800/api/privacy"                  # on, off, auto
curl -H "$T" "localhost:47800/api/state"                            # everything Pulse knows, as JSON
```

**Try the scene** by adding to the page address (`omarchy-pulse serve` prints it): `&wx=rain` (clear, part, cloud,
rain, snow, storm, fog), `&wind=50` (km/h), `&scene=orbit` (or noir, hacker), `&prog=weather` (what the sphere shows: clock, weather,
cpu, news, car, launch...).

## 37. The real night sky

At night your sky is the real one, over your city, at this minute:

- **the stars you know and their constellations**, with their names: Orion, Canis Major (Sirius), Canis Minor,
  Gemini, Taurus (Aldebaran, the Pleiades), Leo, Scorpius (Antares), the Summer Triangle (Vega in Lyra, Deneb in
  Cygnus, Altair in Aquila), Pegasus, Cassiopeia, the Big Dipper, Polaris, Arcturus, Spica, Capella, Fomalhaut.
  They rise in the east and set in the west through the night, as they really do;
- **the planets**: Venus, Mars, Jupiter and Saturn where they are, steady and a little coloured (Mars reddish,
  Saturn golden), with their names;
- **the Moon**, rising and setting at its real time, with its real phase: a thin crescent, the first quarter, full,
  waning... lit on the side where the Sun is;
- **the Sun** by day, where it is, redder near the horizon;
- **meteor showers**: in the nights of the Quadrantids, Lyrids, Eta Aquariids, Perseids, Draconids, Orionids,
  Leonids, Geminids and Ursids, more shooting stars fall, all from the shower's radiant (the point of the sky they seem
  to come from), more near the peak. The voice tells you in the evening ("Tonight, the Perseids meteor shower. Look
  up!").

**How it works.** Everything is computed on your PC from the position of your city (`city` in `[pulse]`, the same as
the weather): the time, your latitude and longitude, and astronomical formulas (the Moon's orbit with its main
perturbations, the planets' orbits from NASA JPL's elements, the stars' positions). No internet, nothing sent. The
positions are good to a fraction of a degree for the Sun and the planets and about a degree for the Moon, far more
than the eye can tell on a wallpaper.

**What you see, and when.** The stars come out when the Sun is 6° under the horizon and are all there after 12°;
they fade behind the real clouds (the weather's cloud cover), and so does the Moon. You look south, like with the
planes: east on the left, west on the right, the top of the screen straight above you. The stars behind you (north)
show only when they are almost overhead; many may be behind your panels, which is why `omarchy-pulse sky` says what
is up.

**Use it.**

```bash
omarchy-pulse sky
#   Moon: waxing crescent, 33% lit, 13° up, south
#   Saturn: 29° up, south-east
#   Meteor shower: Leonids (37% of its peak)
#   Next Starlink train: 18:38, 41° up, west, 22 satellites
#   Air: fair (AQI 28)
```

Ask the sphere: "hey sphere, how is the moon?" · "ehi sfera, com'è la luna?". With `[voice] enabled = true` it also
announces the meteor showers.

```toml
[night]
enabled = true
```

## 38. Starlink trains

**What it is.** SpaceX launches Starlink satellites in batches of 20-30. For a few days after a launch they fly in a
line, one after the other: a "train" of lights crossing the sky in a couple of minutes. It is one of the most
beautiful things you can see with your eyes, and it is visible only at dusk or dawn, when the satellites are still
in the sunlight and you are already in the dark.

**What Pulse does.**

1. Every 6 hours it downloads the real orbits (the official element sets) of the objects launched in the last 30
   days from [CelesTrak](https://celestrak.org), and keeps the Starlink batches (one per launch). Nothing about you is
   sent; the file is kept in `~/.local/state/omarchy-pulse/starlink.json`, so a restart does not download it again.
2. Every hour it works out, for your city, the passes of the next 24 hours that you can really see: the train more
   than 10° above your horizon, lit by the Sun, while the Sun is more than 6° under your horizon.
3. Five minutes before a pass (`warn`) it tells you on the wallpaper ("Starlink train at 18:38 · west · 22
   satellites"), aloud (with the voice), on your phone (section 43) and with a chime (with the sound, section 41).
4. During the pass every satellite of the train is drawn where it is, every few seconds: a line of lights moving
   across your sky, with "STARLINK · 22" next to the first one.

**Precision.** With the `sgp4` Python package installed, Pulse uses SGP4, the same orbit model as the satellite
trackers. Without it, a simpler orbit (Kepler with the Earth's flattening), within about 15 km of SGP4, a degree or
two in your sky and a few seconds in time. To install it:

```bash
pip install --user --break-system-packages sgp4    # or the python-sgp4 package of the AUR
systemctl --user restart omarchy-pulse
```

**Use it.**

```bash
omarchy-pulse sky          # the last line: the next train, its time, height, side and size
```

Ask the sphere: "hey sphere, next Starlink?". If there is no train in the next 24 hours, it says so: there are none
when no batch was launched in the last days, or when the passes fall in daylight or in the middle of the night (when
the satellites are in the Earth's shadow).

```toml
[starlink]
enabled = true
warn = 5           # minutes before; 0 = no warning
```

## 39. Every kind of weather

The sky is your real weather, all of it, from [Open-Meteo](https://open-meteo.com) (updated every few minutes):

| Weather | On your wallpaper |
|---|---|
| clear, sunny | the Sun where it is; at night the stars and the Moon |
| partly cloudy, overcast | clouds as many as the real cloud cover (%), drifting with the real wind |
| drizzle, rain, showers | drops as many as it really rains (mm per hour), slanted by the wind |
| freezing rain | rain (Pulse does not draw ice) |
| snow, snow grains, snow showers | flakes drifting with the wind, as many as it snows |
| hail (in a thunderstorm) | white pellets bouncing on the ground |
| fog | a veil over the city, the stars hidden |
| thunderstorm | dark clouds, bolts down to the city and the thunder after them |
| wind | gust lines, rain and snow blown sideways (section 34) |

**Thunderstorms come from where they really are.** Every 10 minutes Pulse asks the weather of 8 points 30 km around
you (north, north-east, east...) and of your city. A storm 30 km to the west lights the clouds on the west of your
horizon from inside (you see it, you would not hear it); when the storm is over you, bolts fall on the city, the sky
flashes and, with the sound on, the thunder comes a few seconds later (about 3 seconds for every kilometre, as in
reality). In Central Europe Open-Meteo also gives the lightning potential of its model: the more charged the air, the
more often it flashes. With the room lights on (section 42), your lights flash too.

The single strokes measured by lightning networks (Blitzortung) need the network's permission to be used in an app;
Pulse uses the weather models, which have no such limit.

```toml
[storms]
enabled = true
km = 30            # how far around you it looks
```

To see every weather on your screen now, add it to the page address (`omarchy-pulse serve` prints it):
`&wx=clear`, `part`, `cloud`, `rain`, `snow`, `storm`, `fog`.

## 40. The air you breathe

The air quality of your city, from Open-Meteo's air quality model, every 30 minutes:

| European AQI | Level | On your wallpaper |
|---|---|---|
| 0-20 | good | clear |
| 20-40 | fair | clear |
| 40-60 | moderate | a light haze over the city |
| 60-80 | poor | a haze; noted in "while you were away" |
| 80-100 | very poor | a thick haze |
| 100+ | extremely poor | a thick haze |

And the pollen (Europe, in its season: alder, birch, grass, mugwort, olive, ragweed): its grains float in the light
of the city, as many as the real ones (low, moderate, high, very high). The weather panel adds a line: "air fair · 28
· grass pollen high".

```toml
[air]
enabled = true
```

## 41. The sound of your city

A quiet soundscape, off unless you turn it on, made live in the page (nothing is downloaded):

- the real rain, louder when it rains harder; the real wind, stronger with the gusts;
- the rumble of a plane flying over you (higher than 25°), louder when it is lower;
- the thunder of a storm over you, a few seconds after the flash;
- a chime when a meeting's train comes into the station (section 31) or a Starlink train is near;
- the earth rumbling in an earthquake.

It plays only on the main monitor, and it is silent in a meeting, in the quiet hours, while you share the screen,
while you are away and when an app is full screen.

```toml
[sound]
enabled = true
volume = 0.25          # 0..1
quiet = "23:00-08:00"
# rain = true, wind = true, planes = true, thunder = true, events = true   (false turns one off)
```

The wallpaper window of 1.4 plays sound without a click: after updating, restart Pulse
(`systemctl --user restart omarchy-pulse`). If you hear nothing,
check that the sound of the session works (`pw-play` something) and that `[sound] enabled = true`.

## 42. The lights of your room, and Alexa

Your room's lights follow Pulse:

| What happens | Your lights |
|---|---|
| lightning strikes near you (a storm over your city) | a white flash, now and then while it lasts |
| a red alert (`crit`; `warn_too = true` for amber ones) | red, three times |
| the doorbell rings (section 32) | two white flashes |
| an earthquake near you (section 33) | an orange flicker |
| you start a focus (section 17) | a calm blue, until the focus ends |

After every effect each light goes back exactly as it was: on or off, brightness, colour or white. By default only the
lights that are already on take part (`only_if_on = true`), so Pulse never wakes a dark room at night.

### With Home Assistant (any light, and the Ambilight of a Philips TV)

1. In Home Assistant, open your profile (bottom left), **Security**, **Long-lived access tokens**, **Create token**;
   copy it.
2. Put the address and the token in `[home]` (the same as for the doorbell, section 32):
   ```toml
   [home]
   url = "http://homeassistant.local:8123"
   token = "eyJ..."
   ```
3. Find the entity ids of your lights: **Settings**, **Devices & services**, **Entities**, filter by "light."; or
   **Developer tools**, **States**. They look like `light.living_room`. Philips Hue, IKEA, Shelly, WiZ, Govee...
   every light Home Assistant knows works.
4. **Ambilight**: add the **Philips TV** integration in Home Assistant (Settings, Devices & services, Add
   integration, Philips TV; the TV must be on). Its Ambilight appears as a light, for example
   `light.living_room_tv_ambilight`: put it in the list like any other light.
5. Turn it on:
   ```toml
   [lights]
   enabled = true
   source = "homeassistant"
   lights = ["light.living_room", "light.living_room_tv_ambilight"]
   ```

### With a Philips Hue bridge, directly

1. Press the round button on top of the Hue bridge.
2. Within 30 seconds:
   ```bash
   omarchy-pulse lights pair            # finds the bridge on your network
   omarchy-pulse lights pair 192.168.1.20   # or give its address (Hue app, Settings, My Hue system, the bridge)
   ```
3. It prints the lines to paste in `pulse.toml`, with the number and name of every light:
   ```toml
   [lights]
   enabled = true
   source = "hue"
   bridge = "192.168.1.20"
   user = "aBcD..."
   lights = ["1", "3", "4"]      # keep only the ones of the room with your PC
   ```

### Try it, and choose

```bash
omarchy-pulse lights test        # the doorbell effect
omarchy-pulse lights alert       # or lightning, quake
```

```toml
[lights]
only_if_on = true                # false: also the lights that are off (they go back off after)
focus = true                     # alert, doorbell, lightning, quake = true too; false turns one off
focus_brightness = 90            # 0..255
warn_too = false                 # true: amber alerts flash too
```

### Alexa, Google Home, Sonos: Pulse speaks from your speakers

Everything Pulse says aloud (section 21) can come out of your Echo too: the doorbell, a meeting in 5 minutes, an
earthquake, your car charged, a Starlink train. Amazon has no official way for a PC to make an Echo speak, so this goes
through Home Assistant:

1. Install [HACS](https://hacs.xyz) in Home Assistant, then the **Alexa Media Player** integration from HACS, and
   add it (Settings, Devices & services) with your Amazon account.
2. In **Developer tools**, **Actions**, find the action of your Echo: `notify.alexa_media_kitchen_echo` (the name
   follows the Echo's name).
3. Put it in `[voice]`:
   ```toml
   [voice]
   enabled = true
   speakers = ["notify.alexa_media_kitchen_echo"]   # several are fine
   speakers_type = "announce"                       # "announce" (with the chime) or "tts"
   ```

A Google Home, Sonos or any `media_player` works with Home Assistant's text to speech:

```toml
[voice]
speakers = ["media_player.living_room"]
tts = "tts.google_translate_say"                   # your TTS entity (Settings, Voice assistants)
```

The quiet hours, privacy and "away" apply to the speakers too.

## 43. Your phone

The moments that matter as a notification on your phone, with the free [ntfy](https://ntfy.sh) app (iOS, Android):

| Event (`events`) | Notification |
|---|---|
| `focus` | "Focus done · +1 building" |
| `doorbell` | "Someone is at the door" (high priority) |
| `car` | "car charged · 80%", "car at 18% and not plugged in" |
| `earth` | "Earthquake near you · M4.6", "Tsunami message · M7.1" (high priority) |
| `calendar` | "In 5 minutes · standup" |
| `space` | "Starlink train at 18:38 · look west, 41° high" |
| `alerts` | your red alerts (high priority) |
| `wrapped` | "Your 2026 · Pulse Wrapped" |

1. Make a secret topic (a random name only you know):
   ```bash
   omarchy-pulse phone new
   ```
2. Install the ntfy app on your phone, tap **+** (Subscribe to topic) and type that topic. On iOS, allow
   notifications.
3. Put the lines it printed in `pulse.toml`, then:
   ```bash
   omarchy-pulse phone test      # "Hello from your city" on your phone
   ```

```toml
[phone]
enabled = true
topic = "pulse-..."              # keep it secret: anyone who knows it can read the messages
details = true                   # false: no meeting titles, focus names or alert texts on the phone
# events = ["focus", "doorbell", "car", "earth", "calendar", "space", "alerts", "wrapped"]
# server = "https://ntfy.example.com"   # your own ntfy server
# token = "tk_..."                      # for a server with access control
```

The messages go through ntfy.sh, which is free and open source; for full control run your own ntfy server.

## 44. Pulse Wrapped

From the day you update to 1.4, Pulse counts your year in `~/.local/state/omarchy-pulse/stats.json`: focus sessions
and their hours, the buildings and your best streak, the planes that flew over you (each one once a day, kept only as
a short hash), the launches and Starlink trains, the earthquakes, the meetings and their hours, the doorbell.

On 31 December, in the evening, it makes your year into a video: a moment of each day of your city (from the daily
timelapses, section 30: keep `[timelapse] enabled = true`), and a card for each number, ending with "see you in 2027".
It goes to `~/Videos/omarchy-pulse/pulse-wrapped-2026.mp4`, with a notification (and on your phone), ready to share:
like the timelapse it has no names, no panels, no private data.

```bash
omarchy-pulse wrapped          # this year so far, any time
omarchy-pulse wrapped 2026
```

Ask the sphere: "hey sphere, my year" · "ehi sfera, il mio anno". It needs `ffmpeg` (and `fc-match` for the font,
part of fontconfig).

```toml
[wrapped]
enabled = true                 # false: no video on 31 December (omarchy-pulse wrapped still works)
```

## 45. Robots on the floor

Little humanoid robots walk on the floor of the city, in front of the towers.

**Your containers**, by themselves. Every running Docker or Podman container is a robot; its eyes tell its health:
green when it is fine, amber while it restarts, red when it is unhealthy (then it stops and blinks). A container you
start walks in, one you stop leaves; at most 12. Pulse runs `docker ps` (or `podman ps`) every 10 seconds, so your
user must be able to: for Docker, `sudo usermod -aG docker $USER` and log in again; rootless Podman works as it is.

**Your AI bots**, the ones that always run: Grok Bot, a Discord or Telegram bot, an n8n flow, a home server job...
A robot with a light on its antenna and a glowing core, in the accent colour. The bot says how it is, from its own
code, hooks or a script:

```bash
omarchy-pulse robot grok-bot working    # it hurries
omarchy-pulse robot grok-bot ok         # it strolls
omarchy-pulse robot grok-bot waiting    # amber eyes
omarchy-pulse robot grok-bot failed     # it stops, red eyes
omarchy-pulse robot grok-bot gone       # it leaves (it also leaves by itself after an hour with no news)
```

From another program, the same with HTTP:

```bash
curl -X POST -H "X-Pulse-Token: $(cat ~/.config/omarchy-pulse/token)" "localhost:47800/api/robot/grok-bot?state=working"
```

For example, a systemd service of your bot: `ExecStartPost=omarchy-pulse robot mybot ok` and
`ExecStopPost=omarchy-pulse robot mybot gone`.

**Your AI agents** (Claude Code, Codex, Grok, Hermes or any other, section 16) are the drones in the sky; with
`agents_as = "robots"` they walk on the floor instead, with `"both"` they fly and walk. Claude Code and Codex tell
Pulse by themselves through their hooks (`omarchy-pulse hook claude`, `omarchy-pulse hook codex`); any other agent can
call `omarchy-pulse agent NAME working|waiting|done|failed` from its hooks or tools.

Names are hidden in privacy mode.

```toml
[robots]
enabled = true
source = "auto"      # "docker", "podman", or "auto"

[pulse]
agents_as = "drones" # "robots", "both"
```

## 46. The banner plane

A thank-you to Omarchy. A minute or two after Pulse starts, and then now and then (about every 45 minutes, at random,
while you are at the PC, waiting until no window covers the desktop so you see it) a little propeller
plane crosses the whole sky from left to right, towing a big banner that waves in the wind: the Omarchy icon and
"OMARCHY · WE CAN FIX EVERYTHING", in your theme's font and colours.

And with real data: when a new version of Omarchy is released, Pulse reads it from the releases of basecamp/omarchy
on GitHub (the github panel, section 10) and the plane flies right away with the news: "OMARCHY 4.6 IS OUT".

```toml
[pulse]
banner = "OMARCHY · WE CAN FIX EVERYTHING"   # your own words; "" = never
banner_every = 45                            # minutes, more or less
```

```bash
omarchy-pulse banner                 # now, with your words
omarchy-pulse banner "SHIPPED!"      # now, with these (from a script, a git hook, a CI job)
```

The Omarchy icon comes from [Dashboard Icons](https://dashboardicons.com/icons/omarchy) (Apache-2.0). Omarchy is a
project by Basecamp; Pulse is a community theme that thanks it, not an official Omarchy project.

## 47. Community scenes (1.5)

Anyone can draw a whole new world for Pulse, in one JavaScript file, without touching the engine: a pond, a garage,
a forest of commits, a room. Pulse keeps the sky, the weather, the planes, the effects and all the panels; the scene
draws its world on top of the sky. The full guide for creators is [docs/SCENES.md](SCENES.md).

```bash
omarchy-pulse scene list                 # the built-in scenes and the community ones
omarchy-pulse scene new myscene          # a new scene from a template, in ~/.config/omarchy-pulse/scenes/
omarchy-pulse scene lab myscene          # the scene lab in your browser: live preview, demo data, checks, export
omarchy-pulse scene check myscene        # is it ready to share?
```

```toml
[pulse]
scene = "pond"      # the example that comes with Pulse: every AI agent a fish, every repo a water lily
# scene = "monolith" # 1.8: a crystal tower over a mirror lake: your machine, your agents, your repos, your services (79)
# scene = "mars"     # 1.8: a colony on Mars with its own sky: rovers, launch pads, pods, dust storms (80)
```

**Safe by design.** A scene runs alone in a sandboxed frame with no network at all, no token, no access to Pulse, to
your files or to the page: it only gets a read-only copy of a small part of the state (your agents, CPU, RAM, Git,
the weather, the theme, the holidays), never your location, your network devices, your calendar or notifications,
and no names while you share the screen. You can install a scene from anyone without worrying about it.

## 48. The holidays (1.5)

From 8 December to 6 January, lights along the sky and a Christmas tree on the floor; snow when it is really cold
where you are (2 °C or less); on Christmas morning a greeting. On 31 December, in the last hour, the countdown under
the city, then the last minute big on the sphere, second by second, on your own clock; at midnight "HAPPY 2027",
fireworks for a quarter of an hour, the banner plane and the voice (even in the quiet hours, if the voice is on).

```toml
[festive]
enabled = true
christmas = true
new_year = true
lights_from = "12-08"
lights_to = "01-06"
```

## 49. Eclipses (1.5)

Pulse computes the eclipses you can see from your city, on your PC (the same astronomy as the night sky, with the
Moon's parallax, which moves it by up to a degree depending on where you stand). During an eclipse of the Sun, the
Moon's disc covers the Sun where it really is, the light drops with how much is covered, and near totality the stars
come out and the corona shines. During an eclipse of the Moon, the Moon turns copper red in the Earth's shadow.
Half an hour before, the voice and your phone tell you; `omarchy-pulse sky` lists the next ones (here from Berlin):

```
Eclipse of the Moon (penumbral): Sun 21 Feb 2027 00:07, a faint dimming of the Moon, from 22:09 to 02:07
Eclipse of the Sun (partial): Mon 02 Aug 2027 11:14, 36% of the Sun covered, from 10:17 to 12:12
```

Times are right to a few minutes and sizes to a few percent: a wallpaper, not an almanac. On the edge of a path of
totality, check a precise source (for example timeanddate.com) before you travel. `[night] eclipses = false` turns
them off.

## 50. The code construction site (1.5)

The Git city (section 10) becomes a building site. Every commit lowers a new floor onto its tower; scaffolding stands
on the top floors while there are changes not committed; a tower crane works next to the tower of every repo an AI
agent is working on (its name or task mentions the repo; the initial of the agent is on the cab). When the CI of a
repo fails its roof catches fire and a fire engine drives in and sprays water until the CI is green again; then,
fireworks.

## 51. What expires (1.5)

Certificates, domains, licences: a candle each, on the floor, as tall as the days left. Under `warn_days` it turns
amber, under `crit_days` red: an alert, the voice and your phone, once a day. Certificates are read from the server
itself (no credentials), domains with `whois` (`sudo pacman -S whois`), anything else from a date you write.

```toml
[expiry]
hosts = ["example.com", "vpn.example.com:8443"]
domains = ["example.com"]
items = [{ name = "licence", date = "2027-03-31" }]
warn_days = 14
crit_days = 3
```

## 52. The weather of your infrastructure (1.5)

A panel in trouble brings the weather over the city: a grey cloud when it is amber, a storm with rain and lightning
when it is red, with its name under it. When everything is fine, clear sky. It works with every source: Zabbix,
Prometheus, Uptime Kuma, Home Assistant, your own scripts. `[pulse] infra_weather = false` turns it off.

**Acknowledge from the wallpaper (off by default).** For the Zabbix source, add `ack = '''... pulse-zabbix --ack'''`
(see community/zabbix): a **double click** on its storm (two clicks within 5 seconds) acknowledges its problems on
Zabbix. Any source can do the same: `ack` is a command that gets the `id` of every item of the panel. It is the only
thing in Pulse that writes to another system, and only if you add it.

## 53. Home Assistant with the age of every sensor (1.5)

`community/homeassistant`: your entities as tiles, each with how long ago it last reported. Past its max age a tile
turns amber and says `stale 3h` instead of showing an old "all clear"; `unavailable` is red. It reads
`last_reported` (Home Assistant 2024.3+), which moves at every report even when the value stays the same, so a door
that stays closed for a week is not stale. Details in community/homeassistant/README.md.

## 54. The energy of your home (1.5)

A river across the floor: golden when the sun powers your home, blue when the grid does, wider with more load,
flowing backwards when you send power to the grid. From Home Assistant (`[home]` url and token, section 32):

```toml
[energy]
enabled = true
solar = "sensor.solar_power"     # W or kW
grid = "sensor.grid_power"       # positive from the grid, negative to it
load = "sensor.house_power"      # or nothing: solar + grid
peak = 6000                      # W, the river at its widest
```

## 55. The noir scene (1.5)

`scene = "noir"`: a futuristic megacity at night, all original. Towers in layers with thousands of windows, red
lights on the antennas, searchlights in the clouds, rain that never stops, neon signs, cars in the air with their
headlights, the city's lights on the wet street. Instead of the sphere, a big screen on the central tower shows
everything the sphere shows: the clock, the weather, the agent's face when it works, listens or talks ("hey
sphere" too), the door camera, alerts, the countdowns. Everything else of Pulse stays: panels, planes, real weather,
drones, the Git city. Keep it as your default if you like it.

## 56. The hacker scene and the terminal (1.5)

`scene = "hacker"`: the city in phosphor green (`hacker_style = "phosphor"`) or in neon noir (`"noir"`), with a
radar that sweeps your network: every device Pulse knows (your LAN, and with Pulse Net your switches, access points
and servers) is a blip that lights up when the sweep passes, red when it is down.

`omarchy-pulse term` opens a small terminal; what you type and what it answers also appear on the wallpaper (not
while you share the screen):

```
pulse> scan          the devices Pulse knows, and your neighbour table
pulse> ping nas      three pings, by name
pulse> status        switches, firewall, Wi-Fi, servers, backups
```

It only reads. Commands that change something (reboot, block...) answer that they come in a later version, with
confirmations.

## 57. Pulse Net (1.5 beta, read-only)

Your network as part of the city. Everything is **off by default** and **read-only**: Pulse only reads from your
devices. It is a beta: each part was built on the vendors' documented APIs and tested on recorded answers, not on
every model; tell us on GitHub what works on yours. Secrets go in files (`token_file`, `key_file`, `password_file`,
`chmod 600`), never in pulse.toml. `verify = false` only for a self-signed certificate on your own LAN (or give its
CA in `ca_file`). Nothing of this is on the page while you share the screen: no names, no addresses ever.

```toml
[net]
enabled = true
every = 60
```

**Switches (SNMP).** A low building for each switch, a window for each port: lit when the port is up, brighter with
more traffic, dark when down, red and blinking when it has new errors. The roads between switches are the links
their LLDP (or Cisco CDP) neighbours tell, with lights running on them; a saturated uplink is a traffic jam. A switch
that stops answering is an alert. Any managed switch: Cisco, HPE/Aruba, Juniper, MikroTik, UniFi, Netgear...
(`sudo pacman -S net-snmp`; a read-only community or, better, SNMPv3 in `~/.snmp/snmp.conf`).

```toml
[[net.switch]]
name = "core"
host = "10.0.0.2"
snmp = "-v2c -c public"     # or "-v3" with your user in ~/.snmp/snmp.conf
saturate = 0.8              # an uplink above 80% is a jam
```

**The firewall: the city walls.** A wall along the horizon. Every blocked connection is an arrow that comes from the
real direction of its country (GeoIP of the firewall itself) and breaks on the wall; a serious IPS event is a meteor;
VPN users are a lit gate with their number. FortiGate (REST API token, read-only admin profile), OPNsense (API key
and secret, in one file as `key:secret`), pfSense (with its REST API package; key in `key_file`). OPNsense and pfSense
do not send countries: their arrows keep a steady direction for each address.

```toml
[net.firewall]
type = "fortigate"            # fortigate | opnsense | pfsense
url = "https://10.0.0.1"
token_file = "~/.config/omarchy-pulse/fortigate"   # opnsense, pfsense: key_file
```

**Syslog and SNMP traps: bolts, now.** Pulse can listen for syslog on UDP; an error or worse from one of your devices
(a link that drops, a power supply, a fan) strikes its building with lightning, the moment it happens, without
polling. Only from the addresses in `allow`. For SNMP traps, let snmptrapd hand them to Pulse:
`traphandle default /usr/bin/omarchy-pulse trap` in `/etc/snmp/snmptrapd.conf`.

```toml
[net.syslog]
listen = "0.0.0.0:5514"       # point your devices at this PC, UDP 5514
allow = ["10.0.0.0/24"]
max_severity = 3              # 0 emergency ... 3 error
```

**Wi-Fi.** An antenna for each access point, a little person for each client, nearer when the signal is stronger;
when a phone roams it walks to its new antenna. No MAC addresses on the page. UniFi (API key), MikroTik (REST API,
RouterOS 7, a read-only user), OpenWrt (ubus over HTTP with an ACL that allows `hostapd.* get_clients`).

```toml
[net.wifi]
type = "unifi"                # unifi | mikrotik | openwrt
url = "https://10.0.0.1"
key_file = "~/.config/pulse-unifi/key"   # mikrotik, openwrt: user + password_file
```

**The datacenter.** A tower for each host, a floor for each VM (lit when it runs); a live migration (vMotion) is a lit
floor that travels from one tower to the other; a host in maintenance has scaffolding. Proxmox (an API token with the
PVEAuditor role) or vCenter (a read-only user).

```toml
[net.datacenter]
type = "proxmox"              # proxmox | vcenter (user + password_file)
url = "https://10.0.0.10:8006"
token_file = "~/.config/omarchy-pulse/proxmox"   # PVEAPIToken=pulse@pve!wall=SECRET
```

**Backups: the night trucks.** A truck for each job. Running: it drives out of the city. Done: it has arrived. Failed,
or its last good run older than `max_age`: stopped halfway with its hazard lights on, an alert and your phone.

```toml
[[net.backup]]
name = "nas"
type = "restic"               # restic | pbs | veeam | command
repo = "/mnt/backup/restic"
password_file = "~/.config/omarchy-pulse/restic"
max_age = "26h"
# pbs: url, datastore, token_file · veeam: url, user, password_file, job · command: prints {"state": "ok", "t": EPOCH}
```

**Intruders.** A security alert is a robot with red eyes walking to the server it hit, bigger when more serious.
CrowdSec (`cscli`, on this PC) or Wazuh (its indexer, a read-only user).

```toml
[net.intruders]
type = "crowdsec"             # crowdsec | wazuh (url, user, password_file, min_level = 10)
```

## 58. NOC mode (1.5 beta)

For a screen that watches your infrastructure (a second monitor, the TV in the office): `omarchy-pulse noc on`. The
personal panels go away (`noc_hide`, by default music and notifications); a critical event (a red panel, an alert,
a switch that stops answering, a bolt from syslog) opens an **incident**: the screen turns red, with its timeline,
until someone acknowledges it with a click on the incident or `omarchy-pulse noc ack`. `[pulse] noc = true` keeps it
on.

## 59. Reduced motion (1.5)

`[pulse] motion = "reduced"`: nothing runs or swings. The grid stands still, no shaking in a quake, cranes, flames,
trucks and lights keep still, fireworks are rare, and community scenes are told to keep still too.

## 60. Your own wallpaper (1.5.1)

Your own picture under the panels, instead of Pulse's city:

```toml
[pulse]
wallpaper = "~/Pictures/city.jpg"   # jpg, png, webp; or "omarchy": the current Omarchy background, following the theme
wallpaper_mode = "live"             # or "panels"
wallpaper_dim = 0.25                # 0 to 0.9: darker, so the panels stay easy to read
wallpaper_colors = "theme"          # or "picture": the colours come from your picture
```

- **live**: your picture, and on top of it Pulse's live things: the real rain and snow, the planes over you, the
  fireworks when an agent finishes, the rockets, the banner plane, the holidays and the eclipse countdowns. Pulse does
  not draw a sky, a city or a sphere of its own.
- **panels**: your picture and the panels only. It is drawn once and Pulse draws at most twice a second: the lightest
  way to have your widgets on your wallpaper.

**Colours from your picture.** With `wallpaper_colors = "picture"` Pulse reads your picture and takes its colours
from it: a dark background from its shadows, the two accents from its strongest colours, the text a light tint of
them. The panels, the clock, the planes' labels and the fireworks follow. Green, amber and red stay the theme's, so
an alert always looks like an alert. A black and white picture keeps the theme's colours. Change the picture and the
colours change with it.

The picture fills the screen like a wallpaper (cropped, never stretched). It is read from your disk by Pulse itself,
never from the network; change the file or the theme and Pulse picks it up in a few seconds. `wallpaper = ""` goes
back to Pulse's own scene.

## 61. The scene editor (1.6)

Draw your own world on the wallpaper with the mouse, no code: shapes that move with your data. **The full guide, with
a first scene in two minutes and recipes, is [EDITOR.md](EDITOR.md)**; here is the short version.

```bash
omarchy-pulse scene editor            # a new scene: pick a template or start blank
omarchy-pulse scene editor my-world   # open one the editor made
```

Pulse must be running (or `omarchy-pulse serve --demo`). The editor opens in your browser:

- **Add** (left): box, circle, line, star, text, gauge, bars, building, wave, particles. Click one and it appears in
  the middle of the stage.
- **The stage** (middle): Pulse's sky, the horizon line and the dashed zones the panels and the clock cover (what you
  put there is hidden on the wallpaper). Drag a shape to move it (its bottom snaps to the horizon; hold Alt to stop
  that), drag a corner to resize it (Shift keeps its shape). A click on a shape that has a click text shows it, as the
  wallpaper will.
- **Demo data** (under the stage): CPU, RAM, network, the hour, agents, rain, reduced motion and privacy, to see how the
  scene reacts; **my live data** uses your real data instead.
- **The inspector** (right), for the selected shape:
  - **place**: position, size, rotation, and what is particular to each shape (corners, points, thickness, waves...);
  - **look**: one of the theme's colours (so the scene fits every Omarchy theme) or your own, a second colour, opacity,
    glow, filled or outline;
  - **motion**: still, pulse, float, sway, spin, blink, breathe, drift across, and its speed. With `motion = "reduced"`
    everything keeps still by itself;
  - **follows your data**: what moves (height, width, position, opacity, rotation, glow, the colour towards the second
    colour, speed, level, how many, wave height) and with which data (CPU, RAM, GPU, temperature, disk, network, the
    hour, the seconds, the outside temperature, rain, wind, clouds, day or night, your agents, how many work, wait or
    failed, the commits of the week, the failing CI, focus, away). The data goes from its low to its high, the
    property from "from" to "to";
  - **repeat**: one copy per AI agent, Git repo or CPU core, or a fixed number; spread across, or a step each;
    coloured by state (working, waiting, done, failed) or all the same. "Each copy's own value" in *follows your data*
    is that agent, repo or core;
  - **click**: the line Pulse shows when you click the shape on the wallpaper. `{cpu}`, `{ram}`, `{gpu}`, `{temp}`,
    `{net}`, `{time}`, `{date}`, `{outside}`, `{agents}`, `{working}`, `{waiting}`, `{commits}`, and for copies
    `{name}`, `{state}`, `{item}` are filled in live. Names are empty in privacy mode.
- **Layers**: the order (the top one is in front), show or hide, and a dot on the shapes that move with your data.
- **Scene**: Pulse's real sky behind, or a sky of its own; Pulse's far city behind or not.

**Save** (Ctrl+S) writes `~/.config/omarchy-pulse/scenes/NAME.js` and checks it. **Try on wallpaper** shows it on
your desktop at once (until Pulse restarts; `scene = "NAME"` in `[pulse]` keeps it). **Export .js** downloads the
file, to share it. **Lab** opens it in the scene lab (section 47). Ctrl+Z / Ctrl+Y undo and redo, Ctrl+D duplicates,
Delete removes, the arrows move (Shift: more). A draft you did not save comes back when you open the editor again.

A scene made with the editor is a normal community scene: the design is plain data at the top of the file, and the
code after it draws it. It runs in the same sandbox as every scene (no network, no token), and it shows the time a
frame takes, so you can see if it is light enough. The editor never overwrites a scene written by hand. More in
[docs/SCENES.md](SCENES.md#12-the-scene-editor-16-and-17). What 1.7 added to the editor is in section 75.

## 62. All your AI agents, their health, and working together (1.6)

**Every agent Omarchy 4.5 ships** (OpenCode, Claude Code, Codex, Gemini CLI, Copilot CLI, Pi) and Grok Build and
Hermes Agent tell Pulse what they do through their own hooks or plugins. `omarchy-pulse hook setup` prints what to
paste in each one's configuration (`omarchy-pulse hook setup gemini` for one):

| Agent | Where | How |
|---|---|---|
| Claude Code | `~/.claude/settings.json` | hooks, `omarchy-pulse hook claude` (section 16) |
| Codex | `~/.codex/hooks.json` | hooks, `omarchy-pulse hook codex`; approve them once with `/hooks` in Codex (the older `notify` still works) |
| Grok Build | `~/.grok/hooks/pulse.json` | hooks, `omarchy-pulse hook grok`. It also reads Claude Code's hook files: if you set those, Grok already shows up, named `claude` |
| Gemini CLI | `~/.gemini/settings.json` | hooks, `omarchy-pulse hook gemini` |
| Copilot CLI | `~/.copilot/hooks/pulse.json` | hooks, `omarchy-pulse hook copilot EVENT` |
| Hermes Agent | `~/.hermes/config.yaml` | shell hooks, `omarchy-pulse hook hermes`; Hermes asks once to allow each one (or `hooks_auto_accept: true`) |
| OpenCode | a plugin | `omarchy-pulse hook install opencode` |
| Pi | an extension | `omarchy-pulse hook install pi` |

Each one is a drone named after the agent and the project folder (`gemini · omarchy-pulse`). The hooks read only the
event, the tool's name and the folder: never your prompts, the answers or the code. They exit at once and never slow
an agent down, even when Pulse is not running.

**Their health.** Pulse remembers how the last 50 runs of each agent ended (on your PC, in
`~/.local/state/omarchy-pulse/agents.json`). After three runs, its tag shows `ok 92%`; a healthy agent flies a bigger
drone, one that fails often is smaller and its lights flicker, its percentage red under 60%.

**Working together.** Agents working in the same Git repo (or folder) are linked by a line of light, with packets of
light running between them: Claude Code and Codex on the same project, a reviewer and a writer. Pulse compares only a
short hash of the folder, never its path. An agent that does not say its folder is never linked.

**Agents that live in the cloud** (Claude Duet, Meta Muse...) have no hooks on your PC, but they send desktop
notifications when they finish or need your approval. Pulse can turn those into a drone:

```toml
[agents]
from_notifications = ["Claude Duet", "Muse"]
```

Only the app's name decides; a notification that asks for an approval or your input makes the drone wait (amber),
any other means done. The text of the notification never leaves the notifications panel.

**Anything else**: `omarchy-pulse agent NAME STATE "text"`, or `omarchy-pulse watch NAME -- COMMAND ...`.

## 63. Move and hide the panels (1.6)

```bash
omarchy-pulse layout edit     # then use the mouse on the wallpaper (an empty desktop)
```

Every panel gets a dashed outline: drag it to another column or another place in its column, × hides it (click again
to show it). **Save** on the wallpaper keeps the layout in `~/.config/omarchy-pulse/layout.json`; **Cancel** leaves
everything as it was; **Reset** (or `omarchy-pulse layout reset`) gives the panels back to Pulse. The wallpaper has no
keyboard, so the buttons are on screen. While you edit, clicks on panels do not run their commands.

## 64. The workspaces tower and the ivy (1.6)

On the left of the city stands a tower of your Hyprland workspaces: a floor for each, its number on the side, a lit
room for each app on it with the app's icon (a browser, a terminal, an editor, music, chat, files...; always the same
colour for the same app), and a lift that goes up or down to the floor you are on. It shows
the kind of app (chromium, ghostty), never the windows' titles; in privacy mode the windows are lit with no names
anywhere. `tower = false` in `[pulse]` removes it.

**The ivy of your uptime** climbs the tower and the city's buildings while the PC stays on: a little every hour, all
the way up the tower after a week, and then it flowers. After a reboot it starts again from the ground: the quiet
pride of a sysadmin. `ivy = false` removes it.

## 65. Omarchy tips on a hot-air balloon (1.6)

Now and then a hot-air balloon in the theme's colours drifts across the sky with an Omarchy shortcut hanging under it:
`SUPER + SPACE · Launch apps`. The tips are your own key bindings, with the descriptions Omarchy gives them
(`hyprctl binds`), so they are always true for your setup. A balloon comes only on an empty desktop, never during a
meeting, a focus or while you share the screen.

```toml
[tips]
enabled = true
every = 40                     # minutes, more or less
extra = ["SUPER + K · Show key bindings"]   # tips of your own
```

## 66. Updates as parcels, and the news before you update (1.6)

Pending updates pile up as parcels at the city's door, one for each (up to 14), bigger when there are many, with the
count under them: the kernel's parcel is red (a reboot after the update), the AUR ones in the second accent. When the
[Arch Linux news](https://archlinux.org/news/) asks for manual intervention before an update, a balloon crosses the
sky once with "OMARCHY NEWS · read the news before you update", and a road sign with a blinking light stays next to the pile
until you read it.

```bash
omarchy-pulse news         # the news that asks for manual intervention before an update (last 30 days), with links
omarchy-pulse news read    # read: the sign goes away
```

`checkupdates` comes with `pacman-contrib`; the AUR ones need `yay` or `paru`. Pulse looks every 3 hours
(`[updates] every`), never updates anything, and asks for no password. `[updates] enabled = false` turns it off.

## 67. The colours of your music (1.6)

```toml
[music]
colors = true
```

While a song plays (Spotify, YouTube Music in the browser, any MPRIS player that gives its cover), the city takes the
colours of its cover, like the picture colours of section 60. When the music stops, the theme's colours come back.
The cover is read from the player, never searched on the internet, and not while privacy is on.

## 68. A theme from your wallpaper (1.6)

```bash
omarchy-pulse theme from-wallpaper ~/Pictures/city.jpg my-city   # or no picture: your current wallpaper
```

Pulse reads the picture's colours (the same way as section 60) and writes a new Omarchy theme in
`~/.config/omarchy/themes/my-city/`: `colors.toml` (accent, foreground, background, selection, the 16 terminal
colours) and the picture as its background. It never applies it and never overwrites a theme: open `colors.toml`,
change what you like, then `omarchy-theme-set my-city`. Needs `ffmpeg`.

## 69. Your day's diary, and your week (1.6)

The day's video (section 30) is now a diary: it opens with the date and ends with the numbers of the day (commits,
agent tasks done, focus sessions and minutes, meetings, planes over you, rockets, Starlink, quakes, doorbell). Counts
only, never names.

```toml
[timelapse]
cards = true          # the date and the numbers (false: the plain video)
at = "18:00"          # make it at the end of your work day instead of at midnight
weekly = true         # Sunday night: your week in a minute
```

**Your week**: on Sunday night Pulse puts together three seconds of each day's video and the numbers of the week,
`~/Videos/omarchy-pulse/pulse-week-2026-W45.mp4`. `omarchy-pulse recap [DATE]` makes it now (the week that ends on
DATE). The numbers of each day are kept for 60 days in `~/.local/state/omarchy-pulse/days.json`.

## 70. Ask the sphere, with a local AI (1.6)

What the sphere does not know by itself ("why is my PC slow?", "how did my agents do today?") can go to a model on
your own PC, with [Ollama](https://ollama.com) (Omarchy: Install > AI):

```toml
[ai]
ollama = "http://127.0.0.1:11434"
model = "llama3.2"
```

```bash
omarchy-pulse ask "is everything fine?"     # in writing; "hey sphere" (27) asks the same way by voice
```

The model gets a short summary of what Pulse knows (time, CPU, RAM, disk, uptime, weather, agents, alerts, updates,
focus; the numbers of today and yesterday, the alerts of the last 24 hours with their time, the average and the peak of
CPU and temperature of the last minutes), so it can answer "what did I do yesterday?" or "why was the server slow this
morning?" in one or two sentences. Pulse sends it only to an address on this PC or your network,
never to the internet; in privacy mode the agents have no names and the alerts no titles. The voice's own commands
(focus, privacy, weather...) still come first; `[listen] ask` (any CLI) is still there, after Ollama.

## 71. WLED: your LED strips (1.6)

Your [WLED](https://kno.wled.ge) strips glow with the colour of the sphere:

```toml
[wled]
enabled = true
hosts = ["192.168.1.60", "wled-desk.local"]
brightness = 128
alerts = true          # a critical alert blinks them red
```

Fireworks make them a rainbow for a few seconds; while you are away they dim. Pulse sends a colour only when it
changes (and once a minute), to addresses on your network only. Works with any WLED 0.10 or newer, no pairing.

## 72. News that strikes (1.6)

A feed panel (news, GitHub, any list source) can fire an effect when a new item matches a word:

```toml
[[news.rule]]
match = "CVE"              # a word or a regular expression, any case
effect = "lightning"       # lightning | fireworks | banner | shooting | alert

[[news.rule]]
match = "Starship|launch"
effect = "shooting"
```

Each item fires once; what is already there when Pulse starts is not news; private panels are never read. There are
no rules by default.

## 73. Backups as trucks, a calmer second monitor, focus and the radar (1.6)

**Backups (Pulse Net).** Each backup job is now a truck on a road between the city's BACKUP depot and the VAULT: it
drives to the vault while the backup runs, comes back home when it is done and parks with a green stripe; a failed or
late backup stops halfway, hazard lights on and smoke from the engine, until the next good run (section 57).

**A second monitor.** `secondary = "simple"` in `[pulse]`: on monitors other than the main one, only the scene and the
side panels, no drones, planes, robots or cars, at 12 frames a second. Light for the PC and calm at the edge of your eyes.

**Focus.** During a focus the CPU towers slowly come down and the drones rest on the ground (one that needs you
still flies and blinks).

**The radar.** In the hacker scene, a new device on your network keeps pinging on the radar with NEW next to it,
and the terminal writes it down.

## 74. The friends' neighbourhood (1.6)

With a crew (section 26), your friends' cities appear on the far right of your horizon: a building for each friend,
as tall as their commits of the week, their nickname over it. When a friend commits, a floor of their building
lights up; the small light on top is their CI, green or red.

```toml
[crew]
code = "..."          # the code you shared by hand (omarchy-pulse crew new)
city = true           # share your city with them, and see theirs
```

Only numbers travel, sealed with the crew's code: how many commits this week, "a commit now", the CI green or red.
Never a repo's name, a branch, a message or a file. The building of a friend who stops sharing fades away after three
days. `city = false` (the default when your `pulse.toml` was written before 1.6) sends nothing.

## 75. Creator: pictures, animations, actions, .pulse files and the Community (1.7)

The scene editor (section 61) now makes complete scenes, and they travel as one file. The full guide is
[EDITOR.md](EDITOR.md) (sections 9 to 16); sharing is [PACKAGES.md](PACKAGES.md).

- **Your pictures**: PNG, JPEG, WebP or SVG, as shapes (tinted with the theme or with their own colours, mirrored,
  moving with your data like any shape) or as the **background** of the scene, dimmed to keep the panels readable. A
  scene can also have **its own palette** instead of the Omarchy theme's.
- **Every value of the engine**: besides CPU, agents, Git and the weather, a shape can follow the Sun's height, the
  Moon, the planes overhead (or repeat once per plane), the ISS, Starlink, the aurora Kp, the next launch, music,
  meetings, and **your own panels**: anything a `[[source]]`, a push or Home Assistant gives a number to (`panel:power`:
  a tower that grows with your homelab's power draw). Only numbers, and never a private panel.
- **One copy per row of your panel**: a building per Proxmox VM, a fish per Docker container, a lamp per light,
  green, amber or red with each row; an action when a row turns red, or green again. The *Homelab city* template.
- **Animations**: keyframes (move, size, opacity, turn) that loop, or play when an action says so; presets jump, fly,
  fade, grow, shake, spin.
- **Actions**: *when* something happens (an agent waits for you, CI fails or passes, a commit, a plane, the ISS, a
  launch, rain, a click on a shape, every N seconds, a value crossing a line) *do* something: play an animation, flash,
  show or hide a shape, or ask Pulse for one of its effects (fireworks, a banner, lightning, a shooting star, a
  balloon with your text, a rocket, a solar flare, a chime). Pulse keeps its effects rare: one every 4 seconds, the
  same one every 15, none while a window covers the desktop.
- **.pulse files**: **Export .pulse** makes one file with the scene, its pictures, animations and actions, a preview,
  your name and a licence (MIT, CC0, CC-BY, CC-BY-SA). **Import…** (or drop it on the editor) opens one: open it in
  the editor to change it, or install it.
- **The Community**: the editor's **Community** button shows the scenes people shared in the project
  (`community/scenes/`), with **Install** and **Install & edit**. To add yours: one pull request with your `.pulse`
  file (PACKAGES.md, section 4).

```bash
omarchy-pulse scene export my-world --author "Your Name" --license MIT    # my-world.pulse
omarchy-pulse scene import my-world.pulse                                 # install one
omarchy-pulse scene gallery                                               # the Community
omarchy-pulse scene get homelab-tower                                     # install one from it
```

Safe by design: a package is data, or code that runs in the sandbox; it never carries commands or `[[source]]`
scripts, which are shared only as reviewed pull requests. Pictures are checked byte by byte (an SVG with scripts or
links is refused), and gallery packages must match the reviewed index.

## 76. The look (1.8)

Pulse looks more like a place. Nothing changes in what it shows or how it behaves: the same scenes, the same data in
the same spots. What changes is the light and the depth.

- **One light.** The real Sun by day and the real Moon by night (as bright as it is lit) light the scene from where
  they are in your sky: the side of every tower that looks at them catches their colour, and their light lies on the
  floor as a path of glints. A low Sun sends its beams across the sky and its glow along the horizon.
- **Buildings, not cards.** The towers of your CPU cores have a side that goes back toward the horizon, a roof with a
  mast, a tank or the air conditioning, a lit ground floor, and their reflection on the wet floor. Their height and
  their windows still follow each core and your RAM.
- **A far city.** Three rows of buildings, paler with the distance, with setbacks, spires and slanted roofs; their
  windows come on as the Sun goes down. Mist drifts between them, thicker at dawn, at night and in fog.
- **A sky without steps.** Night, twilight and day blend with the Sun's real height, and the sky fades into its new
  colour instead of jumping. Clouds have a body, lit from the side of the Sun or the Moon. How many clouds there are,
  where the wind takes them, the rain, the snow and the storms follow the real weather exactly as before.
- **Rain with depth.** Far drops are thin and slow and end near the horizon, near ones are long and bright; where
  they land a ring opens on the wet floor, and heavy rain veils the far city. How hard it rains is still the real
  rain (mm/h and the kind of rain); snow has depth too.
- **Planes you can read.** Every real plane over you has a tag: the flight and the aircraft, its route with the
  cities, how high, how fast and where it is heading, on a plate with a line to the plane. Still hidden while you
  share the screen.
- **A rocket you can recognise.** When a real launch lifts off, a Falcon 9 (white, the black interstage, legs, grid
  fins, the fairing) or a Starship (steel, flaps) climbs from your horizon: smoke rolls off the pad, the flame has a
  white heart and shock diamonds, the trail is smoke that widens and fades; in twilight the plume opens like a
  jellyfish, as it really does; after ten seconds the booster comes off and falls back while the second stage goes
  on. A tag carries its name and the time since liftoff.
- **A Starlink train you can see.** The satellites really in your sky glide instead of jumping, a faint thread joins
  those that fly in a row, each one glints and the first is the brightest; the tag says how many and which way.

It costs almost nothing (what never moves is painted once). `polish = false` in `[pulse]` brings back the plain look;
a PC that struggles drops the mist and the beams by itself; your own wallpaper and the noir scene are not touched;
with `motion = "reduced"` nothing new moves.

## 77. Your datacenter is a district of the city (1.8)

```toml
[datacenter]
panels = ["proxmox", "docker"]   # the ids of your panels
```

Every row of those panels is a hall in front of the city: a Proxmox VM, a Docker container, a host of Uptime Kuma, a
light of Home Assistant. Any panel with a status, a list or a table works (the community sources in `community/` give
you most of them). Its racks' lights are green, amber or red with the row; the taller halls are the rows with the
bigger number, when they have one.

- **It goes down**: its lights go out, a red beacon turns on its roof, lightning strikes it and its name crosses the
  sky (one bolt every 20 seconds at most). Its name stays above it while it is down or in trouble.
- **It is back**: a few green sparks.
- `labels = false` hides the names; `alerts = false` keeps the district without lightning and sparks.

Private panels are never drawn, and while you share the screen (or in privacy mode) there are no names. Nothing is
drawn until you name a panel. Up to 36 halls. In the scene editor the same rows are *repeat → row of your panel*
(section 75).

## 78. Describe your world (1.8)

Say what you want to see, and an AI agent writes the scene. A scene is data that Pulse checks, never code, so this is
safe with any AI: what Pulse does not know is left out and you are told.

**In the editor**: **✨ Describe…**, write your idea ("a harbour at night: every Docker container is a ship, red when
it is down; a lighthouse that turns faster with the CPU; fireworks when CI passes"), then:

- **Make it with my agent**, if you set one:

  ```toml
  [creator]
  agent = "claude -p"     # any command that reads the request on its standard input and prints the reply
  ```

- or **Copy the request**, paste it into any AI (Claude, ChatGPT, Gemini, a local model), and paste its reply back:
  **Use the reply**.

The scene lands on the stage: move things, change colours, add actions, **Save**, **Export .pulse** to share it.

**From a terminal**:

```bash
omarchy-pulse scene describe harbour "a harbour where every container is a ship" --try
omarchy-pulse scene prompt "a harbour..." | your-agent | omarchy-pulse scene make harbour --from -
omarchy-pulse scene guide        # what the agent reads: the whole format, in one page
```

The request holds the guide of the format, the names of your panels (their ids, units and row names, never their
values, and never a private panel) and your words. Your agent is your own command, run by you, like a `[[source]]`;
Pulse sends nothing anywhere by itself. For agents that work in your repo there is the same guide in
`extras/pulse/web/scene-guide.md`.

## 79. Monolith, a scene that comes with Pulse (1.8)

```toml
[pulse]
scene = "monolith"
```

A crystal the size of a tower floats over a mirror lake, under Pulse's real sky (the Sun, the Moon, the weather, the
planes are Pulse's, as always). Everything in it is something of yours:

| You see | It is |
|---|---|
| the monolith's bands | your CPU cores, from the top: the busier, the brighter; red above 90% |
| the colour of its heart | the CPU temperature: the theme's accent when cool, amber, then red |
| its two rings | memory (inside) and disk (outside): the lit arc is how full |
| the light it drops on the water | the load |
| the shards that orbit it | your AI agents: fast with a trail while working, big and amber with a `!` when one waits for you, green when done, red and fallen on the shore when failed |
| the pylons on the shore | your repos: the beam is as tall as the week's commits, green, amber (running) or red with the CI; it flares at every commit; an amber dot for changes not committed |
| the cubes on the water | the rows of your panels that have a state (VMs, containers, hosts): green, amber, red and blinking |
| rings on the lake | the network |
| a ring closing around it | a focus session, as the time goes |
| the bands dancing | music is playing |
| an aurora | the real Kp is 4 or more, at night |

It asks Pulse for its effects, rarely: a rocket when a build turns green again, a chime when an agent starts waiting
for you, lightning when a service goes down and a shooting star when it is back, a balloon when the aurora is strong
(Kp 6), fireworks at midnight. Click anything to read it. In a meeting, or when you are away, it all glows lower.

**It fits the PC.** The scene measures how long a frame takes and draws in one of four ways: *light* (flat lights, no
reflection: an old laptop), *medium*, *high* (the reflection in the lake, the aurora, trails), *ultra* (more of
everything, on a fast PC). It goes down at once when the PC struggles and up slowly when there is room; with
`quality = "half"` it stays at medium at most. Every size is a share of the screen, so it is as sharp at 1366×768 as
in 4K. Click the monolith: the last words say how it is drawing. With `motion = "reduced"` it stands still.

It is a scene in code, in the sandbox like every scene: `extras/pulse/web/scenes/monolith.js` is commented from top to
bottom, to read or to start your own from (`omarchy-pulse scene lab monolith`).

## 80. Mars, another world (1.8)

```toml
[pulse]
scene = "mars"
```

A colony on Mars. Unlike every other scene it does not use your theme's colours and it paints its own sky: it is
meant to look like nothing else in Pulse. The panels and the clock stay yours, on top. Everything in it is something
of yours:

| You see | It is |
|---|---|
| the dome | your machine. The lights on its wall are the CPU cores (the busier, the brighter; red above 90%); the reactor inside goes from blue to amber to red with the temperature; the greenhouse is fuller when the machine is cool |
| the two tanks | memory and disk, as full as they are (red above 90%) |
| the antenna | the network: the busier, the more it sends to orbit |
| the rovers | your AI agents: one that works drives and raises dust (with a headlight at night), one that waits for you stops and blinks amber with a `!`, a green tick when done, tilted and smoking when failed |
| the launch pads | your repos (six at most): crates for the week's commits, the beacon on the tower green, amber (running, and the rocket vents) or red with the CI; it flares at every commit; an amber light for changes not committed. **When a build goes back to green, its rocket leaves** |
| the pods | the rows of your panels that have a state (VMs, containers, hosts), twelve at most: the window is green, amber, or red and blinking |
| a green arc over the dome | a focus session, as the time goes |
| the wall's lights dancing | music is playing |

**The hour is yours.** The Sun is as high as it really is where you are: by day the sky is butterscotch, at sunset
the Sun is small and the sky around it is blue (on Mars it really is), at night there are the stars, Phobos crossing
fast, Deimos, and the Earth as a blue star with the Moon beside it.

**Your weather, translated.** Rain is a dust storm, as strong as the rain; wind raises dust devils far away; snow is
frost falling; clouds are thin streaks of ice; a strong solar storm (the real Kp, 5 or more) makes the night sky
glow.

Pulse's own fireworks, rockets and rain are behind this world's sky, so Mars answers with its own: a rocket leaves a
pad when a build turns green, when a real launch lifts off on Earth and at midnight ("a new sol"); a chime and a line
of text when a rover starts waiting for you or a pod goes down. Click anything to read it. In a meeting, or when you
are away, the lights are lower.

**It fits the PC**, as monolith does: four ways of drawing (*light*, *medium*, *high*, *ultra*) chosen from how long
a frame takes, medium at most with `quality = "half"`, every size a share of the screen (sharp from 1366×768 to 4K),
still with `motion = "reduced"`. Click the dome: the last words say how it is drawing.

The code is `extras/pulse/web/scenes/mars.js`, commented, to read or to start your own from
(`omarchy-pulse scene lab mars`).

---

## 81. Pulse native, the same wallpaper in Rust (1.1)

**pulse-native** draws Pulse on the GPU, in Rust with GPUI (the toolkit of the Zed editor), with no browser. It reads
the same hub, token and `pulse.toml` as the web wallpaper, so sources, panels, alerts, agents, the editor's scenes
and the voice stay as they are; only the drawing changes. Install it with `omarchy-pulse native on` (section 1); its
own guide, every option and the changelog are in [native/README.md](../native/README.md).

It draws everything Pulse 1.8 draws, and some things only it can:

- **What is down on the internet.** The public status pages of the main AIs (ChatGPT, Claude, Gemini, xAI/Grok,
  Copilot, Mistral, Perplexity, DeepSeek, Cursor, Groq, Hugging Face) and of what the internet stands on (X,
  WhatsApp, Instagram, Facebook, Telegram, Microsoft 365, Azure, AWS, Google, Cloudflare, GitHub, Slack, Zoom,
  Discord, Apple...), plus your own line: a line in the network panel, a panel and a dark hall in the datacenter
  district when something goes down. Only public pages, every five minutes; `pulse-native --status-check` prints
  them, `--status off` stops it, your own pages go in `~/.config/omarchy-pulse/status.txt`.
- **Your AI agents answer a click**: their card (what they do, for how long, how their runs went), **its window**
  (the terminal it runs in comes to the front, to give it the OK it waits for) and **take it away**.
- **Pip**, a creature on the floor of every scene whose needs are real: food (updates waiting: feeding it opens
  `omarchy-update`), health (alerts, a red CI, a service down), rest (the hour, the CPU, the heat), clean (the disk),
  joy (your streak, the music, and you: a click pets it). `[pet] name = "..."` names it, `[pet] enabled = false` sends it away.
- **Four more worlds** for `[pulse] scene`, painted pixel by pixel and every piece one of your data (click it):
  `station` (the real Earth from orbit, as a rocket's camera sees it), `abyss` (an undersea base), `racks` (a whole
  server room, the switches blinking with your traffic) and `moonbase` (the Earth in its real phase). The web
  wallpaper does not have them: with it, these names draw the default scene.
- **Mars on its own clock**: the real time at Jezero, Perseverance's sol, the light's delay from the Earth, Phobos
  and Deimos at their real periods (`PULSE_MARS_SUN=earth` gives it your own day back).
