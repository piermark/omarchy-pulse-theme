# pulse-native: Omarchy Pulse in Rust + GPUI (experimental)

A parallel renderer for Omarchy Pulse. The web page stays the default (and is what runs on OmarchyTV); this one
paints the same wallpaper natively on the GPU, with no browser underneath. It reads the same hub: the same
`omarchy-pulse` service, the same token and the same `pulse.toml`.

**Status: 0.1, a first step.** Drawn so far: the sky at its real hour, stars, the Moon with its phase and the Sun
where they are, clouds that move with your wind, the far city with windows that come on at dusk, rain and snow as
they really are, the planes over you with their tags; the clock and the system, network, weather and agents panels in
your Omarchy theme's colours. Not yet: scenes, the editor's scenes, your own `[[source]]` panels, rockets, Starlink,
the ISS, effects, clicks.

## Build and run

It needs Rust, a Wayland compositor with wlr-layer-shell (Hyprland on Omarchy) and a Vulkan driver.

```bash
sudo pacman -S --needed rust wayland libxkbcommon vulkan-icd-loader fontconfig
cd native
cargo build --release
./target/release/pulse-native            # the wallpaper, behind your windows
./target/release/pulse-native --window   # an ordinary window, to look at it
```

Options: `--url http://127.0.0.1:47800` (the hub; default from `PULSE_PORT` or `[pulse] port`), `--fps 20`,
`--font "CaskaydiaMono Nerd Font"`.

The hub must be running (`omarchy-pulse serve`, as the live wallpaper already does). While the web wallpaper is on,
both draw: stop it first to compare.

Built with [gpui-ce](https://github.com/gpui-ce/gpui-ce), the community edition of Zed's GPUI (Apache-2.0), which
has the layer-shell support a wallpaper needs.
