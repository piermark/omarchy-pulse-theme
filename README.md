# Omarchy Pulse — a live wallpaper for Omarchy

![Omarchy Pulse](preview.png)

Your desktop, alive. Pulse turns the Omarchy wallpaper into a live scene made of your own data:

- **the skyline is your CPU**: one tower per core, as tall as its load; the lit windows follow your RAM
- **the horizon is your pulse**: it beats faster when the PC works, and every beat runs from the core to the edges
- **the aurora is your network**, **the sky is your real weather** (rain, snow, fog, storms), and an alert sends a
  shockwave through the city with an **INCOMING** banner

Around it, Omarchy-style panels show **any live data you want**: a command, a JSON API, a file, a stream, a camera
or anything you push from a script. Camera feeds, AI briefs, Home Assistant, servers, builds, GitHub, MQTT, logs.

Made for **Omarchy 4.5**. By [Marco Porro](https://x.com/MarcoPorro), the author of
[OmarchyTV](https://github.com/piermark/omarchytv) (the Live screen of OmarchyTV, now on your desktop).

## Install

```
omarchy-theme-install https://github.com/piermark/omarchy-pulse-theme
~/.config/omarchy/themes/pulse/extras/install-live
```

The first line installs the **pulse** theme (colours, borders, btop, two still wallpapers). The second makes the
wallpaper live: it installs what it needs (`python-gobject`, `gtk-layer-shell`, `webkit2gtk-4.1`, and `ffmpeg` /
`playerctl` if missing), asks your city for the weather and starts Pulse. From then on it starts at every login while
the theme is **pulse**. Run it from a terminal on the PC itself, not over SSH.

- Live with **every** theme (Pulse takes each theme's colours): `extras/install-live --always`
- Back to the pulse theme only: `extras/install-live --theme`
- Remove it (your sources stay): `extras/install-live --remove`
- Check what is missing: `omarchy-pulse doctor` · logs: `journalctl --user -u omarchy-pulse -f`

**The full guide** (every source, the AI brief, cameras, a cookbook for Zabbix, Home Assistant, Proxmox, GitHub
Actions, MQTT and more, privacy, tuning, troubleshooting): [docs/GUIDE.md](docs/GUIDE.md).

## Put anything on your wallpaper

Edit `~/.config/omarchy-pulse/pulse.toml`; when you save, the wallpaper picks it up in a few seconds. Each
`[[source]]` is a panel:

```toml
[[source]]
name = "nas"
title = "nas · disk"
command = "ssh nas df --output=pcent /volume1 | tail -1"   # anything that prints something
every = 300
warn_above = 80
crit_above = 92          # amber, red, and INCOMING when it crosses
```

| Source | What it does |
|---|---|
| `command = "..."` | runs every `every` seconds; what it prints is the data |
| `stream = "..."` | keeps running; every line is an update (`mosquitto_sub`, `journalctl -f`, `tail -f`) |
| `url = "..."` | JSON (pick a part with `path = "data.0.value"`), text or an image; `headers = { ... }` |
| `file = "..."` | read again when it changes (text, JSON or an image) |
| `camera = "rtsp://..."` | a live thumbnail through ffmpeg (`fps = 2`); `snapshot = true` polls an image URL |
| `push = true` | waits for `omarchy-pulse push` |

What a source prints decides the panel: a **number** (`42`, `21.5 °C`) is a big value with a sparkline; **text** is
typed out; JSON `{"items": [...]}` is a list (`"type": "status"` turns it into green/amber/red tiles),
`{"rows": [[k, v]]}` a table, `{"value", "unit"}` a value (`"type": "gauge"` a bar), `{"series": [...]}` a chart.
Any of them may add `"level": "ok" | "info" | "warn" | "crit"`, `"title"` and `"note"`.

More keys: `title`, `icon` (a Nerd Font glyph), `slot = "left" | "right" | "center" | "ticker"`, `order`, `type`,
`timeout`, `private = true` (hidden while you share the screen), `notify = true` (also a desktop notification).

### AI on your wallpaper

`pulse-ai` gives an AI everything your wallpaper shows and puts its brief on the desktop: what needs you first,
in four short lines. `command = "pulse-ai claude -p"`, or `pulse-ai ollama run llama3.2` to keep it all on your PC
(also `codex exec`, `gemini -p`, any CLI). Private panels are left out. Details in the [guide](docs/GUIDE.md#8-ai-on-your-wallpaper).

`pulse.toml` comes with ready examples to uncomment: an AI brief (`pulse-ai`, or any CLI that prints text), an RTSP
camera, Home Assistant, MQTT, a live log, any JSON API, pacman updates, Docker. Your own scripts go in
`~/.config/omarchy-pulse/sources/`, which is on the PATH of every source. Built in: `pulse-ping` (hosts as tiles,
`gateway` = your router), `pulse-systemd` (failed units), `pulse-table` (lines to a table), `pulse-ai`.

### Push from anywhere

```
omarchy-pulse push deploy 72 --type gauge --title "deploy"
make && omarchy-pulse push build ok --level ok || omarchy-pulse push build failed --level crit
omarchy-pulse push door ~/Pictures/doorbell.jpg
curl -s -X POST -H "X-Pulse-Token: $(cat ~/.config/omarchy-pulse/token)" \
     --data '{"value": 21.5, "unit": "°C"}' http://127.0.0.1:47800/api/push/garage
```

## Community sources

Pulse shows any live data, and the best sources come from the people who use it.
[`community/`](community/) is a growing collection of ready-made sources: Zabbix, Proxmox, GitHub
Actions, Docker, Hacker News and more. Each one is a folder with the script, the `[[source]]` block
to paste in your `pulse.toml` and a short README.

Install one:

    cp community/zabbix/pulse-zabbix ~/.config/omarchy-pulse/sources/
    chmod +x ~/.config/omarchy-pulse/sources/pulse-zabbix

then paste `community/zabbix/source.toml` into `~/.config/omarchy-pulse/pulse.toml`.

**Made a source for your homelab, your printer, your smart home?** Open a pull request: see
[CONTRIBUTING.md](CONTRIBUTING.md). Sources run on other people's PCs, so they are read-only,
keep secrets out of the files and talk only to the service they say. Wanted: UniFi, Pi-hole,
OctoPrint/Klipper, Uptime Kuma, Grafana, TrueNAS, Synology, Wazuh, Jellyfin, Tailscale.

## Built in

System (CPU, every core, RAM, GPU, temperature, disk, battery), network, weather (Open-Meteo, no key), now
playing (playerctl), your last notifications (mako), GitHub (releases and commits of the repos you follow, plus your
notifications after `gh auth login`), clock and a ticker. Turn any of them off in `[builtin]`.

## Privacy

When you share or record the screen, Pulse notices (Hyprland tells it) and hides cameras, notifications, your IP and
every `private = true` panel until you stop. By hand: `omarchy-pulse privacy on | off | auto | toggle` (bind it to a
key). The hub listens only on `127.0.0.1`, every request needs the token in `~/.config/omarchy-pulse/token`
(mode 600), and camera addresses and passwords never reach the page.

## Light on the PC

The scene runs at 20 frames a second only while you look at an empty desktop; with windows open it drops to 8, with
something full screen it stops, on battery it stays at 12 at most, and a PC that cannot keep up gets fewer frames by
itself. Panels refresh once a second. Tune it in `[pulse]`: `fps`, `scene = "calm"` (still grid), `scene = "orbit"`
(the cores become moons around the core), and `gpu = "never"` on a PC without a working GPU driver. It uses
WebKitGTK: about 400 MB of memory with one monitor.

## Files

- `colors.toml`, `hyprland.conf` (cyan to magenta borders), `btop.theme`, `icons.theme`
- `backgrounds/01-pulse-skyline.png`, `backgrounds/02-pulse-orbit.png`: the stills, under the live wallpaper
- `extras/install-live`, `extras/pulse-live` (theme-set hook and login start), `extras/pulse/` (the engine:
  `omarchy-pulse`, the page in `web/`, `pulse.toml`, `sources/` with `pulse-ai`, `pulse-ping`, `pulse-systemd`,
  `pulse-table`)
- `docs/GUIDE.md`: the full guide

## License

MIT, © 2026 Marco Porro ([x.com/MarcoPorro](https://x.com/MarcoPorro)). Weather data by
[Open-Meteo](https://open-meteo.com) (CC BY 4.0).
