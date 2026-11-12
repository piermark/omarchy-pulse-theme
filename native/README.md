# pulse-native: Omarchy Pulse in Rust + GPUI (experimental)

A parallel renderer for Omarchy Pulse. The web page stays the default (and is what runs on OmarchyTV); this one
paints the same wallpaper natively on the GPU, with no browser underneath. It reads the same hub: the same
`omarchy-pulse` service, the same token and the same `pulse.toml`.

**Status: 0.7, experimental: the first complete port of Pulse 1.8.** Every part of the web wallpaper has its
counterpart here; the list at the end says what is still simpler or missing. It has been built and looked at on a
headless test compositor only: on a real Omarchy PC it has yet to be tried.

What it draws, from the same state the web page reads:

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
- **In place of the web wallpaper**: `native/install-native` (and `--remove` to go back).

Simpler than the web page, or still missing:

- the 1.8 polish made with blur (bloom, glass, depth of field): GPUI has no blur; a later step with shaders;
- the sphere is the LED face and its programmes as words in a frame, not the web page's textured ball; no camera or
  car programme on it;
- Pulse Net: the firewall, its events and the devices' radar are here; the full map of the network, Wi-Fi and the
  backup trucks are not;
- the layout is read, not edited: drag the panels on the web wallpaper (`omarchy-pulse layout edit`);
- community scenes written in JavaScript (other than mars and monolith) show the skyline and say so; in design
  scenes a picture's tint and clicks are missing;
- the heartbeat is a number beside your name, the line of the horizon does not beat with it;
- live camera panels refresh with the state (once a second), not as a stream.

What the hub does is unchanged, because it is the same hub: sources, alerts, AI, the voice, lights and WLED, the
phone, the crew, the day's video, Wrapped, the scene editor.

`--scene city` keeps pulse-native's first world: a far city under a warm dusk.

## Build and run

It needs Rust, a Wayland compositor with wlr-layer-shell (Hyprland on Omarchy) and a Vulkan driver.

```bash
sudo pacman -S --needed rust wayland libxkbcommon libxkbcommon-x11 libxcb vulkan-icd-loader fontconfig freetype2
cd native
cargo build --release
./target/release/pulse-native            # the wallpaper, behind your windows
./target/release/pulse-native --window   # an ordinary window, to look at it
```

Options: `--scene skyline|sphere|orbit|calm|hacker|noir|mars|monolith|city` (default: the hub's), `--url http://127.0.0.1:47800` (the hub; default from `PULSE_PORT` or `[pulse] port`), `--fps 20`,
`--font "CaskaydiaMono Nerd Font"`.

The hub must be running (`omarchy-pulse serve`, as the live wallpaper already does). While the web wallpaper is on,
both draw: stop it first to compare.

To let it be the wallpaper at every login, in place of the web page (Pulse must be installed already):

```bash
native/install-native            # builds if needed, installs pulse-native, switches the user unit to it
native/install-native --remove   # back to the web wallpaper
```

It adds `pulse-native` and `pulse-native-live` to `~/.local/share/omarchy-pulse/` and one systemd drop-in
(`omarchy-pulse.service.d/native.conf`); none of Pulse's own files change.

Built with [gpui-ce](https://github.com/gpui-ce/gpui-ce), the community edition of Zed's GPUI (Apache-2.0), which
has the layer-shell support a wallpaper needs.
