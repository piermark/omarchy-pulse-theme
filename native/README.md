# pulse-native: Omarchy Pulse in Rust + GPUI (experimental)

A parallel renderer for Omarchy Pulse. The web page stays the default (and is what runs on OmarchyTV); this one
paints the same wallpaper natively on the GPU, with no browser underneath. It reads the same hub: the same
`omarchy-pulse` service, the same token and the same `pulse.toml`.

**Status: 0.2, experimental.** Drawn so far, in the look of Pulse 1.8: the sky at its real hour (the web page's
own three skies, mixed by the Sun's height), stars, the Moon with its phase and the Sun where they are, clouds that
move with your wind, the far city, the neon towers and the dome, the grid running to the horizon and the line of the
machine's pulse, rain leaning with the wind and snow, the planes over you with their tags, your AI agents as drones;
the clock, the system, network and weather panels and **your own panels** (every `[[source]]`: stat, gauge, list,
status, key and value rows, text) in their slots, in your Omarchy theme's colours.

Not yet: the other scenes (sphere, orbit, mars, monolith, the editor's), charts and images in panels, rockets,
Starlink, the ISS, the datacenter district, effects, sounds, clicks, the second monitor, privacy when a window covers
it. The web page remains the complete Pulse.

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

Options: `--scene pulse|city`, `--url http://127.0.0.1:47800` (the hub; default from `PULSE_PORT` or `[pulse] port`), `--fps 20`,
`--font "CaskaydiaMono Nerd Font"`.

The hub must be running (`omarchy-pulse serve`, as the live wallpaper already does). While the web wallpaper is on,
both draw: stop it first to compare.

Built with [gpui-ce](https://github.com/gpui-ce/gpui-ce), the community edition of Zed's GPUI (Apache-2.0), which
has the layer-shell support a wallpaper needs.
