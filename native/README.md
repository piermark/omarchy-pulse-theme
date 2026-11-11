# pulse-native: Omarchy Pulse in Rust + GPUI (experimental)

A parallel renderer for Omarchy Pulse. The web page stays the default (and is what runs on OmarchyTV); this one
paints the same wallpaper natively on the GPU, with no browser underneath. It reads the same hub: the same
`omarchy-pulse` service, the same token and the same `pulse.toml`.

**Status: 0.3, experimental; the goal is the same Pulse as the web page, function by function.**

Done, in the look of Pulse 1.8:

- the sky at its real hour (the web page's own three skies, mixed by the Sun's height), stars, the Moon with its
  phase, the Sun, clouds that move with your wind, rain leaning with the wind, snow;
- the scenes `skyline` (far city, neon towers, the dome) and `sphere`: the LED sphere, after the Sphere of Las Vegas,
  with Pulse 1.8's face (it thinks, needs you, is happy, sad, asleep, with your AI agents) and the CPU round it; by default the one your hub says
  (`[pulse] scene`), so both renderers show the same;
- the planes over you, the ISS, the Starlink train (it glides, a thread joins the row), a real launch (the rocket,
  its smoke, the stage separation, T+), fireworks, lightning, shooting stars and the words of an effect: the hub's
  live channel, as the web page hears it;
- your AI agents as drones; the grid, and the line of the machine's pulse;
- the clock, system (with its history), network, weather, and **your own panels** (every `[[source]]`: stat and
  gauge with their history, list, status, key and value rows, text) in their slots; the next launch at the bottom;
- privacy: while the screen is shared your panels, the tags and the address say nothing.

Still to port (the web page remains the complete Pulse): the scenes orbit, calm, hacker, noir, mars, monolith and
the editor's own; the datacenter district, robots, the car, cranes, the balloon and banner plane; repos as towers;
charts, tables with more columns and images in panels; sounds and the voice; clicks; the second monitor; pausing when
a window covers it; the calendar's trains, eclipses, the aurora, the seasons.

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

Options: `--scene skyline|sphere|city` (default: the hub's), `--url http://127.0.0.1:47800` (the hub; default from `PULSE_PORT` or `[pulse] port`), `--fps 20`,
`--font "CaskaydiaMono Nerd Font"`.

The hub must be running (`omarchy-pulse serve`, as the live wallpaper already does). While the web wallpaper is on,
both draw: stop it first to compare.

Built with [gpui-ce](https://github.com/gpui-ce/gpui-ce), the community edition of Zed's GPUI (Apache-2.0), which
has the layer-shell support a wallpaper needs.
