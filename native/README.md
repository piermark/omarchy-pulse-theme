# pulse-native 1.0: Omarchy Pulse in Rust + GPUI

A parallel renderer for Omarchy Pulse. The web page stays the default (and is what runs on OmarchyTV); this one
paints the same wallpaper natively on the GPU, with no browser underneath. It reads the same hub: the same
`omarchy-pulse` service, the same token and the same `pulse.toml`.

**Version 1.0.1.** A complete renderer of Pulse 1.8 beside the web page, which stays (and is what runs on
OmarchyTV): you choose which one draws your wallpaper, and go back with one command. It is a line-by-line
translation of the published Pulse 1.8 (`pulse.js` and `pulse.css`), so that it looks and moves the same, and a
little more like a game where the web page cannot: lights that burn white at their core (the canvas's "lighter"
mode, which GPUI does not have), the Sun's starburst and lens flare, beams of the sphere's light turning in the wet
air with dust drifting in it, LEDs that glide instead of stepping, landmarks lit by the real Sun and Moon, a colony on
Mars, a giant LED billboard in the noir city. Everything is drawn at the screen's own resolution, 4K included.

## Install, and go back

It needs Rust, a Wayland compositor with wlr-layer-shell (Hyprland on Omarchy) and a Vulkan or OpenGL driver (Mesa
on Intel and AMD, the NVIDIA driver). Pulse must be
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
| `--cpu N` | the most of one core it may use, in percent (also `PULSE_CPU`); by default 15 on a laptop, 25 on a desktop, up to twice that on a big screen (the square root of its pixels over 1080p's) |
| `--calm N` | frames a second when nothing fast moves (also `PULSE_CALM_FPS`); by default 10 on a laptop, 20 on a desktop |
| `--gpu G` | `auto` (default), `gl` or `vulkan` (also `PULSE_GPU`): `auto` draws with OpenGL on Intel Ivy Bridge and Haswell |
| `--url URL` | the hub (default `http://127.0.0.1:` + `PULSE_PORT` or `[pulse] port`) |
| `--font NAME` | the font (default: `[pulse] font`, else CaskaydiaMono Nerd Font) |
| `--version` | print the version |
| `PULSE_PERF=1` | print every five seconds what each part of a frame costs |

## How light it is

The GPU does the drawing: rectangles, circles and straight lines are quads it draws as instances, glows are small
pictures it keeps, the panels are a view GPUI keeps and draws again until they change (a new state, a new second, a
text typing out), many lines (the rain, the grid) go as one shape. The slanted shapes are made into triangles here at
once (a fan for a convex one, a band for a line), and GPUI draws each group of them in a target as big as the group,
not as big as the screen (see `vendor/gpui_ce_wgpu/PULSE-NATIVE.md`): the same antialiasing, a fraction of the memory
traffic, which is what held back older GPUs and 4K screens. Everything stays at the screen's full resolution.

Frames come at an even pace; when nothing fast moves (no music, nothing falling, no fireworks, launch or shooting
star, no pointer to follow) 10 frames a second are drawn on a laptop and 20 on a desktop (`--calm`), 6 while you are
away, all of them when something does. The wallpaper stays under its CPU budget (`--cpu`; 15% on a laptop, 25% on a
desktop, more on a big screen), measured as `top` measures it: the whole process, the GPU driver
included, every two seconds; over it the frames a second come down at once (never under 6), well under it they come
back one at a time. Under a fullscreen window it draws once a second; covered, not at all. Memory freed is given back
to the system once a minute.

On Intel Ivy Bridge and Haswell, Mesa's Vulkan driver says "Haswell Vulkan support is incomplete" and can stall: there
pulse-native draws with OpenGL (Mesa's crocus driver, complete on those chips). `--gpu vulkan` or `--gpu gl` choose it
by hand.

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
- **Layout edit** (`omarchy-pulse layout edit`): drag a panel to a column, the × (or the right button) hides it and
  shows it again, Save, Cancel, Reset.
- **pond**, the example scene; in the editor's scenes a picture's tint and the clicks on shapes with their actions;
  camera panels, and the camera and the car on the sphere.

## Why it may not start, or look wrong

- **Nothing changes**: the Omarchy theme is not `pulse` and `live.conf` has no `ALWAYS=1`; or the hub is not running
  (`systemctl --user status omarchy-pulse`). `journalctl --user -u omarchy-pulse -e` says which.
- **A box in the middle of the screen**: an old build; update (fractional scales are read from `hyprctl`).
- **Stalls or a black screen on an old Intel GPU**: `--gpu gl` (automatic on Ivy Bridge and Haswell).
- **Few frames a second on a big screen**: raise the budget, `--cpu 40` (or `PULSE_CPU=40` in the drop-in), and
  `--calm 30` for a smoother calm scene. `PULSE_PERF=1` prints what each part of a frame costs.
- **Night in the middle of the day**: the hub has no location for the real Sun; set it in `pulse.toml` (the
  weather's city): pulse-native then follows the weather's sunrise and sunset.

## Changes

- **1.0.1**: the pictures and words on the sphere glide as smoothly as it turns (they stepped a texel at a time);
  no more flashes of old shapes for a frame beside the panels or a plane's tag (a group of shapes GPUI lays as one
  picture is now fresh all over); a plane's contrail starts again after a jump instead of drawing stripes.
- **1.0**: the first release. Since 0.17: the workspaces tower, your friends' towers and the parcels checked again
  line by line against pulse.js (the apps' icons on their rooms, the friends' push that fades, the road sign that
  blinks, the words in the same place), the ivy's flowers on the city's towers after a week without a reboot.
- **0.17**: the noir scene translated from pulse.js (the megacity, its searchlights, signs, cars in the air), its big
  screen a giant LED billboard; on Mars the flights are flying saucers with their tags, and a colony grows from your
  data (a ship that flies with Earth's next real launch, solar farm, greenhouses, colonists, supply pods for your
  updates, orbiters, a scout helicopter); your city's landmarks lit from where the Sun or the Moon is, with floors of
  windows and floodlit monuments, the Statue of Liberty in New York, Christ the Redeemer in Rio and new cities
  (Verona, Palermo, Catania, Trieste, Bari, Agra, St Petersburg, Philadelphia, Houston, Dallas, Denver, Jerusalem,
  Marrakesh, Casablanca, Brasília, Macau); a × hides a panel in layout edit; the CPU budget and the calm frame rate
  follow the computer (issue #42); full resolution at any scale (up to 3×); day stays day when the hub has no
  location; a plane beside a panel no longer jumps back under it once a second.
- **0.16**: the shapes GPUI draws in groups cost a whole screen of memory per group (about 60 a frame): now only as
  much as the group, with the same quality; slanted shapes made without the tessellator; the sphere's screen painted
  once and its pixels in a faster loop; the CPU budget measured on the whole process; OpenGL on Ivy Bridge and
  Haswell; memory given back; a click on the notifications panel clears it, a click on a light toggles it.
- **0.15**: the 1.8 look as a game would draw it: hot lights, the Sun's flare, the sphere's beams and dust.
- **0.14**: a smooth sphere, an even pace, better rain and clouds. **0.13**: less CPU, steady memory, the core of
  the other scenes. **0.12**: what passes over you, the weather and the life of the city. **0.11**: the city, its
  light and the sky, translated from pulse.js. **0.10**: planes that fly, the network on the floor, clicks that
  close. **0.9**: the sphere of Pulse 1.8. **0.8**: the look of 1.8, Pulse Net, layout edit. **0.7**: the first
  complete port. **0.6**: the real stars, the whole weather panel. **0.5**: the scene editor's and the community's
  `.pulse` scenes. **0.4**: the living city. **0.3**: the LED sphere, the ISS, launches. **0.2**: the look of Pulse
  1.8. **0.1**: the first renderer.

## Still different from the web page

- community scenes written in JavaScript by other people cannot run here (there is no JavaScript engine): they show
  the skyline and say so. The three that come with Pulse (mars, monolith, pond) are written again in Rust;
- what pulse-native adds is its own (Mars's colony and saucers, the lit landmarks, the extra cities, the LED
  billboard): the web page does not have it;
- a few pieces written from pulse.js in the first versions have not been compared line by line again since: the
  earthquake, Christmas and the new year, Pulse Net and the garden. They are there and work; small differences in
  their look may remain.

## What stays the same

What the hub does is unchanged, because it is the same hub: sources, alerts, AI, the voice, lights and WLED, the
phone, the crew, the day's video, Wrapped, the scene editor. Only the drawing is new.

`--scene city` keeps pulse-native's first world: a far city under a warm dusk.

Built with [gpui-ce](https://github.com/gpui-ce/gpui-ce), the community edition of Zed's GPUI (Apache-2.0), which
has the layer-shell support a wallpaper needs.
