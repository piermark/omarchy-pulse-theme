# pulse-native: Omarchy Pulse in Rust + GPUI (experimental)

A parallel renderer for Omarchy Pulse. The web page stays the default (and is what runs on OmarchyTV); this one
paints the same wallpaper natively on the GPU, with no browser underneath. It reads the same hub: the same
`omarchy-pulse` service, the same token and the same `pulse.toml`.

**Status: 0.15, experimental.** The web page stays the default. This renderer is being made a line-by-line
translation of the published Pulse 1.8 (`pulse.js` and `pulse.css`), so that it looks and moves the same, and a
little more like a game where the web page cannot: lights that burn white at their core (the canvas's "lighter"
mode, which GPUI does not have), the Sun's starburst and lens flare, beams of the sphere's light turning in the wet
air with dust drifting in it, LEDs that glide instead of stepping.

## Install, and go back

It needs Rust, a Wayland compositor with wlr-layer-shell (Hyprland on Omarchy) and a Vulkan driver. Pulse must be
installed already (`extras/install-live`): its hub, `omarchy-pulse serve`, is what both renderers read. The code
lives on the `native` branch, not on `main`.

```bash
sudo pacman -S --needed rust wayland libxkbcommon libxkbcommon-x11 libxcb vulkan-icd-loader fontconfig freetype2
git clone -b native https://github.com/piermark/omarchy-pulse-theme.git ~/pulse-native
cd ~/pulse-native/native
cargo build --release            # the first build takes a few minutes
./install-native                 # makes it the wallpaper, now and at every login
./install-native --remove        # back to the web wallpaper
```

It adds `pulse-native` and `pulse-native-live` to `~/.local/share/omarchy-pulse/` and one systemd drop-in
(`omarchy-pulse.service.d/native.conf`); none of Pulse's own files change, and `pulse.toml` stays as it is.

It draws only when Pulse itself would: with the `pulse` Omarchy theme, or with every theme if
`~/.config/omarchy-pulse/live.conf` has `ALWAYS=1`. Otherwise the service exits quietly and nothing changes.
`journalctl --user -u omarchy-pulse -e` shows what it did.

It fills every monitor, also with a fractional scale (1.25, 1.5): the size of each screen is read from
`hyprctl monitors -j`, because GPUI alone would see a smaller screen and draw a box in the middle.

### Update

`install-native` builds only when there is no binary yet, so after a pull build again first:

```bash
cd ~/pulse-native && git pull && cd native && cargo build --release && ./install-native
./target/release/pulse-native --version   # the version you have
```

### Try it without installing

```bash
./target/release/pulse-native --window   # an ordinary window, to look at it
./target/release/pulse-native            # the wallpaper, behind your windows (stop the web one first)
```

## Options

| | |
|---|---|
| `--window` | an ordinary window instead of the wallpaper |
| `--scene S` | `skyline`, `sphere`, `orbit`, `calm`, `hacker`, `noir`, `mars`, `monolith`, `pond`, `city`; by default the hub's `[pulse] scene` |
| `--fps N` | the most frames a second, 5 to 60 (default: the hub's `[pulse] fps`, else 20) |
| `--cpu N` | the most of one core it may use, in percent (default 15; also `PULSE_CPU`) |
| `--url URL` | the hub (default `http://127.0.0.1:` + `PULSE_PORT` or `[pulse] port`) |
| `--font NAME` | the font (default: `[pulse] font`, else CaskaydiaMono Nerd Font) |
| `--version` | print the version |
| `PULSE_PERF=1` | print every five seconds what each part of a frame costs |

## How light it is

The GPU does the drawing: rectangles, circles and straight lines are quads it draws as instances, glows are small
pictures it keeps, the panels are a view GPUI keeps and draws again until they change (a new state, a new second, a
text typing out), many lines (the rain, the grid) go as one shape. The CPU still builds the slanted shapes and
computes the sphere's pixels (GPUI has no shaders of one's own). Frames come at an even pace; when nothing fast
moves (no music, nothing falling, no fireworks, launch or shooting star, no pointer to follow) 12 frames a second
are drawn, all of them when something does; and the wallpaper stays under its CPU budget (`--cpu`), never under 8
frames a second. Under a fullscreen window it draws once a second; covered, not at all.

To compare it with the web wallpaper on your PC, measure both the same way, a minute each. The web wallpaper runs
in the hub's own process (`omarchy-pulse`) with WebKit's processes beside it; the native one runs beside the hub.
So count them all, the hub included, both times:

```bash
pids=$(pgrep -f 'omarchy-pulse|pulse-native|WebKitWebProcess|WebKitGPUProcess|WebKitNetworkProcess' | tr '\n' ' ')
t() { for p in $pids; do awk '{s+=$14+$15} END{print s}' /proc/$p/stat; done | awk '{s+=$1} END{print s}'; }
a=$(t); sleep 60; b=$(t); echo "CPU $(( (b-a) * 100 / 60 / $(getconf CLK_TCK) ))% of a core"
```

## What it draws

From the same state the web page reads:

- **Scenes**: `skyline`, `sphere`, `orbit`, `calm`, `hacker`, `noir`, and the two scenes in code that come with
  Pulse, **`mars`** and **`monolith`**, written again in Rust (the same worlds, the same data behind every piece,
  the same lines when you click); the scenes made with the scene editor and the community's `.pulse` scenes (the
  1.7 design engine: shapes, live data, repeats, motions, pictures, actions); `city`, pulse-native's first world.
- **The sky**: its real hour, the stars with their constellations, the planets, the Moon with its phase, the Sun,
  solar and lunar eclipses where they happen, meteor showers, the aurora, clouds with your wind, rain, snow, the
  haze of poor air, pollen, gusts of a strong wind, thunderstorms (bolts over you, light on the horizon when they
  are far), a solar flare.
- **Over you**: planes with their flight, the ISS, the Starlink train, a real launch, fireworks, shooting stars.
- **The city**: your repos as towers, the datacenter district, the car, the robots, the calendar's trains, the
  candles of what expires, the tips' balloon, the banner plane, the garden of your days and its monuments, your
  friends' towers, updates as parcels (and the sign of Arch news), the workspaces tower with its lift and the ivy
  of your uptime, the firewall's wall, a storm over every panel that is failing (click it twice to acknowledge),
  syslog bolts, Christmas (the tree, the garland, flakes when it is cold), the new year's countdown and fireworks,
  earthquakes that shake the picture, a tsunami, the doorbell, the lines of `omarchy-pulse term`.
- **The sphere**: Pulse 1.8's face, which looks at the pointer, and its programmes in turn (clock, weather, CPU,
  news, the music's bars, a launch countdown, focus, the next meeting, an alert).
- **Panels**: clock (24h or 12h, the week, your title, the heartbeat), system, network, the whole weather panel
  with the air, every `[[source]]` panel, in the places you saved (`omarchy-pulse layout`), clickable where
  pulse.toml says so; the ticker; INCOMING for a new alert; "while you were away"; the NOC mode and its incident
  (click to acknowledge); what the sphere heard and answered.
- **Your own wallpaper** behind it (`[pulse] wallpaper`, live or panels only, its dim, its colours for the theme),
  and the colours of the song's cover.
- **Sound** (`[sound]`): rain, wind, a plane overhead, thunder, chimes, made from noise and sines and played with
  `pw-cat` (or `paplay`, `aplay`); nothing runs until you set a volume.
- **Monitors and power**: a wallpaper on each monitor, `secondary = "simple"` or `"off"`, nothing drawn while a
  window covers the monitor, one frame a second under a fullscreen window, slower with windows open, on battery,
  away or on a still scene; focus, meeting, away, privacy and `motion = "reduced"` as on the web page.
- **The look of 1.8** (`polish`, on by default): one light, the real Sun or Moon, that the towers catch and that
  lies on the floor as a path of glints; a tower for each CPU core with a side, a roof and its reflection, as tall
  as the core's load, dancing with your music; the far city mirrored on the wet floor; air and mist on the
  horizon; the beams of a low Sun; rings where the rain lands; your heartbeat running along the line of the horizon.
- **Pulse Net**: the firewall and what it blocks, the devices' radar, a building for each switch with its ports and
  the roads between neighbours, a tower for each host with its VMs and their migrations, the Wi-Fi antennas and
  their people, a truck for each backup, the intruders.
- **Layout edit** (`omarchy-pulse layout edit`): drag a panel to a column, the right button hides it and shows it
  again, Save, Cancel, Reset.
- **pond**, the example scene; in the editor's scenes a picture's tint and the clicks on shapes with their actions;
  camera panels, and the camera and the car on the sphere.

## Still different from the web page

- community scenes written in JavaScript by other people cannot run here (there is no JavaScript engine): they show
  the skyline and say so. The three that come with Pulse (mars, monolith, pond) are written again in Rust;
- the neo-noir scene (`noir`) is still pulse-native's own drawing of it;
- in layout edit the right button hides a panel and shows it again (the web page has a × on it);
- some pieces written from pulse.js in the first versions are not yet compared line by line again: the
  earthquake, Christmas and the new year, Pulse Net, the garden, the workspaces tower, the friends, the parcels.

## What stays the same

What the hub does is unchanged, because it is the same hub: sources, alerts, AI, the voice, lights and WLED, the
phone, the crew, the day's video, Wrapped, the scene editor. Only the drawing is new.

`--scene city` keeps pulse-native's first world: a far city under a warm dusk.

Built with [gpui-ce](https://github.com/gpui-ce/gpui-ce), the community edition of Zed's GPUI (Apache-2.0), which
has the layer-shell support a wallpaper needs.
