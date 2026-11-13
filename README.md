# Omarchy Pulse — a live wallpaper for Omarchy

**Version 1.8** · **new: [Pulse native 1.0](#pulse-native-10-the-same-wallpaper-in-rust)**, drawn on the GPU without a browser · [what's new](#new-in-18) · [try it in ten minutes](#try-18-in-ten-minutes) · [the scene editor](docs/EDITOR.md) · [share and install scenes](docs/PACKAGES.md) · [create a scene](docs/SCENES.md) · [the guide](docs/GUIDE.md) · [every command](docs/GUIDE.md#36-every-command)

![Omarchy Pulse 1.5: the noir scene, a futuristic megacity at night with the agent's face on the big screen, candles for what expires, the energy river, robots on the floor (demo data)](preview.png)

> [!TIP]
> **🎨 Make a scene, and everyone gets it.** Draw your world in the scene editor (`omarchy-pulse scene editor`):
> your pictures, your homelab, your agents, your sky. **Export .pulse**, open a pull request that adds it to
> [`community/scenes/`](community/scenes/), and once it is merged it is in **every Pulse user's Community gallery**,
> with your name on it. One file, no code needed, and it runs in Pulse's sandbox.
> **[How to send yours, in four steps →](docs/PACKAGES.md#4-put-your-scene-in-the-gallery)** · already there:
> [the Community](community/scenes/)

Your desktop, alive. Pulse turns the Omarchy wallpaper into a live scene made of your own data:

- **new in 1.8**: a city that looks like one (one real light, volume, mist, reflections), **your datacenter as a
  district** (a hall per VM or container, lightning when one goes down) and **✨ Describe your world**: say what you
  want and your AI agent draws the scene ([what's new](#new-in-18))
- **new in 1.7, "Creator"**: the scene editor makes **complete scenes**: your own **pictures** and **backgrounds**,
  **animations**, **actions** ("when CI passes, launch a rocket"), shapes that follow **your own panels** (one per VM,
  per container...) and the sky;
  **.pulse files** to share a scene in one file, and a **Community gallery** to install other people's in one click
  ([what's new](#new-in-17))
- **new in 1.6: the scene editor**: draw your own world with the mouse and make it move with your data, no code.
  **Every AI agent Omarchy ships** (OpenCode, Claude Code, Codex, Gemini, Copilot, Pi) plus Grok and Hermes, with
  their **health** and **lines of light** when they work together; **move and hide the panels**; your **workspaces as a
  tower** with **ivy** that grows with your uptime; **Omarchy tips on a hot-air balloon**; **updates as parcels** and the
  news before you update; your **friends' cities on the horizon**; the **colours of your music**; **a theme from your wallpaper**; your day's **diary** and your **week**; ask
  the sphere with a **local AI**; **WLED** strips; **news that strikes** ([what's new](#new-in-16))
- **new in 1.5: scenes anyone can make** (with a scene lab), a futuristic **noir** megacity, **the holidays and the
  eclipses** on your own clock and sky, **a construction site for your code**, candles for what **expires**, the
  **weather of your infrastructure**, your home's **energy** as a river, and in beta **your network as part of the
  city**: switches, the firewall walls, Wi-Fi, servers, backups, intruders, a NOC mode ([what's new](#new-in-15))
- **new in 1.4: the real night sky** (stars, constellations, the Moon's phase, the planets, meteor showers),
  **Starlink trains**, **every kind of weather**, the air, sound, your room's lights, Alexa, your phone, **robots**
  (your containers and AI bots), **a banner plane that thanks Omarchy**, **Pulse Wrapped** ([what's new](#new-in-14))
- **new in 1.3: a city that grows with your work**, the real **planes** over you, **your day as a video**, your
  **calendar** as trains, your **doorbell** on the sphere, **earthquakes** ([what's new](#new-in-13))
- **new in 1.2: your real city** (90+, every continent), **your repositories as towers**, **the ISS and every SpaceX
  launch over your house**, your **heartbeat** on the horizon, a **crew** of friends in your sky, and **"hey sphere"**:
  talk to it ([what's new](#new-in-12))
- **the LED sphere**: a giant LED globe in the middle of the city, turning, showing your clock, weather, CPU, news,
  cameras, alerts and the face of your AI agent; it lights the plaza and shines on the wet floor
- **your AI agents are drones**: Claude Code, Codex, Gemini, OpenCode, Pi, Grok, Hermes or any job fly while they work, blink
  when they need you, land green (with fireworks) or red when they finish
- **the city dances with your music**, fireworks celebrate what goes right, a **focus mode** quiets everything down
- **it sees you**: the face's eyes follow your pointer, it falls asleep when you leave and, when you are back, tells
  you **what happened while you were away** (also aloud, with an offline voice)
- **your network drives on the grid**: every device is a car of light; a new one blinks and raises an alert
- **the skyline is your CPU**: one tower per core, as tall as its load; the lit windows follow your RAM
- **the horizon is your pulse**: it beats faster when the PC works, and every beat runs from the core to the edges
- **the aurora is your network**, **the sky is your real sky**: the stars, the Moon and the planets where they are,
  and your real weather (sun, clouds, rain, snow, fog, storms), and an alert sends a
  shockwave through the city with an **INCOMING** banner

Around it, Omarchy-style panels show **any live data you want**: a command, a JSON API, a file, a stream, a camera
or anything you push from a script. Camera feeds, AI briefs, Home Assistant, servers, builds, GitHub, MQTT, logs.

Made for **Omarchy 4.5**. By [Marco Porro](https://x.com/MarcoPorro), the author of
[OmarchyTV](https://github.com/piermark/omarchytv) (the Live screen of OmarchyTV, now on your desktop).

## Pulse native 1.0: the same wallpaper, in Rust

**pulse-native** draws Pulse 1.8 natively on the GPU, in Rust with [GPUI](https://github.com/gpui-ce/gpui-ce)
(the toolkit of the Zed editor), with no browser underneath. It reads the same hub, the same token and the same
`pulse.toml` as the web wallpaper, so everything Pulse does stays as it is: sources, panels, alerts, agents, the
scene editor, the voice. Only the drawing changes. The web wallpaper of 1.8 stays the default, and is still what
runs on OmarchyTV: you choose which one draws your desktop, and go back with one command.

- **The same Pulse 1.8**, translated line by line from `pulse.js` and `pulse.css`: every scene (`sphere`, `skyline`,
  `orbit`, `calm`, `hacker`, `noir`, `mars`, `monolith`, `pond`), the scene editor's and the community's `.pulse`
  scenes, the real sky, the weather, the city and its life, Pulse Net, the panels, layout edit.
- **More of a game** where a browser cannot: lights that burn white at their core, the Sun's starburst and lens flare,
  the sphere's light turning in the wet air, LEDs that glide; your city's landmarks lit from where the Sun or the
  Moon really is, floodlit at night (the Statue of Liberty in New York, Christ the Redeemer in Rio, and more
  cities); a giant LED billboard in the noir city; on Mars, flying saucers carry the flights over you and a colony
  grows from your data (a ship that flies with Earth's next real launch, a solar farm, greenhouses, colonists,
  supply pods for your updates).
- **Light on the PC**: no browser, the drawing on the GPU, a CPU budget measured as `top` measures it (15% of a core
  on a laptop, 25% on a desktop, more on a big screen), slower when nothing moves, nothing drawn when a window
  covers the monitor. Full resolution on every screen, 4K and fractional scales included.
- **Old GPUs too**: on Intel Ivy Bridge and Haswell it draws with OpenGL by itself.

### Install pulse-native

Pulse 1.8 must be installed first (above: `extras/install-live`), because its hub is what pulse-native reads. Then:

```bash
sudo pacman -S --needed rust wayland libxkbcommon libxkbcommon-x11 libxcb vulkan-icd-loader fontconfig freetype2
git clone -b native https://github.com/piermark/omarchy-pulse-theme.git ~/pulse-native
cd ~/pulse-native/native
cargo build --release            # the first build takes a few minutes
./install-native                 # it becomes your wallpaper, now and at every login
./target/release/pulse-native --version
```

It adds two files to `~/.local/share/omarchy-pulse/` and one systemd drop-in; none of Pulse's own files change. It
draws when Pulse itself would (the `pulse` theme, or `ALWAYS=1` in `~/.config/omarchy-pulse/live.conf`).

**Update**: `cd ~/pulse-native && git pull && cd native && cargo build --release && ./install-native`

**Back to the web wallpaper**: `cd ~/pulse-native/native && ./install-native --remove`

**Try it without installing**: `./target/release/pulse-native --window` (a window) or
`./target/release/pulse-native` (the wallpaper, with the web one stopped).

| Option | |
|---|---|
| `--scene S` | the scene (by default `[pulse] scene`) |
| `--fps N` | the most frames a second (by default `[pulse] fps`) |
| `--cpu N` | the most of one core it may use, in percent (`PULSE_CPU`) |
| `--calm N` | frames a second when nothing fast moves (`PULSE_CALM_FPS`) |
| `--gpu G` | `auto`, `gl` or `vulkan` (`PULSE_GPU`) |
| `PULSE_PERF=1` | what each part of a frame costs, every five seconds |

**If something is wrong**: `journalctl --user -u omarchy-pulse -e` says what it did. Few frames a second on a big
screen: `--cpu 40 --calm 30`. A stall on an old Intel GPU: `--gpu gl`. The whole guide, every option and how to
measure it against the web wallpaper: [native/README.md](https://github.com/piermark/omarchy-pulse-theme/blob/native/native/README.md).

**Changelog**

- **1.0**: the first release: the noir scene translated from pulse.js with its LED billboard, Mars's saucers and
  colony, landmarks lit by the real Sun and Moon and sixteen more cities, the × in layout edit, the CPU budget and
  the calm frame rate set by the computer ([#42](https://github.com/piermark/omarchy-pulse-theme/issues/42)), full
  resolution up to a scale of 3, the workspaces tower, friends and parcels checked again against pulse.js.
- **0.16**: each group of shapes drawn in a target as big as the group instead of the whole screen (about sixty a
  frame: a fraction of the GPU's memory traffic, same quality); OpenGL on Haswell; memory given back; a click on the
  notifications panel clears it.
- **0.15**: the 1.8 look as a game would draw it. **0.14**: a smooth sphere, better rain and clouds. **0.13**: less
  CPU, steady memory. **0.10–0.12**: planes that fly, the network on the floor, the sky and the city translated from
  pulse.js. **0.1–0.9**: from the first renderer to the whole of Pulse 1.8.

Still different from the web page: community scenes written in JavaScript do not run (the three that come with
Pulse, `mars`, `monolith` and `pond`, are written again in Rust).

## New in 1.8

**A city that looks like one, your datacenter in it, and scenes made by describing them.**

- **The look.** The real Sun and Moon light the whole scene: the side of every tower that looks at them, a path of
  glints on the floor, beams at dusk. Towers have volume, roofs and reflections; a far city in three depths whose
  windows come on as the Sun goes down; mist between the buildings; clouds with a body; a sky that changes without
  steps. Everything shows the same data in the same place, the weather follows the real one as before, and
  `polish = false` brings back the plain look. ([section 76](docs/GUIDE.md#76-the-look-18))
- **Your datacenter is a district of the city.** `[datacenter] panels = ["proxmox", "docker"]`: a hall for every VM,
  container or host, its lights in the colour of its state. One goes down: lightning strikes it and its name crosses
  the sky. ([section 77](docs/GUIDE.md#77-your-datacenter-is-a-district-of-the-city-18))
- **Describe your world.** In the editor, **✨ Describe…**: write "a harbour where every container is a ship" and your
  AI agent draws the scene, ready to finish and share. With your own agent (`[creator] agent = "claude -p"`) or by
  pasting the request into any AI. A scene is data that Pulse checks, never code.
  ([section 78](docs/GUIDE.md#78-describe-your-world-18))
- **A new scene: monolith.** `scene = "monolith"`: a crystal tower floating over a mirror lake is your machine (a
  band per CPU core, its heart as warm as the CPU, rings for memory and disk); your AI agents are the shards that
  orbit it, your repos the pylons on the shore, your VMs and containers a swarm of cubes. It asks Pulse for a rocket
  when a build turns green, a chime when an agent needs you, lightning when a service goes down, and it chooses by
  itself how rich to draw, from an old laptop to a fast PC in 4K. ([section 79](docs/GUIDE.md#79-monolith-a-scene-that-comes-with-pulse-18))
- **Another world: mars.** `scene = "mars"`: a colony on Mars with its own sky and its own colours, nothing like the
  neon city. The dome is your machine (a light per CPU core, a reactor as warm as the CPU, two tanks for memory and
  disk), your AI agents are rovers, your repos are launch pads (when a build goes back to green, the rocket leaves),
  your VMs and containers are habitat pods. The hour is yours: butterscotch by day, a blue sunset, Phobos and the
  Earth at night. Your weather arrives translated: rain is a dust storm, wind raises dust devils, snow is frost.
  ([section 80](docs/GUIDE.md#80-mars-another-world-18))
- **The first scene from the community: blackout-room**, by Anatolii ([tolixx](https://github.com/tolixx)), made for a
  homelab in Kyiv that rides out power cuts on battery stations. A server room on a street at night: when the grid
  goes down the houses go dark, but the rack keeps blinking on its batteries. It follows three panels, `grid`, `ups`
  and `watts` (with NUT, three lines of `upsc`). Install it with `omarchy-pulse scene get blackout-room`, or from
  **Community** in the editor: [step by step below](#6-a-scene-from-the-community-blackout-room).
  ([community/scenes/blackout-room](community/scenes/blackout-room/))
- **Rain with depth and planes you can read.** Near and far drops, rings where they land (as much rain as there
  really is); every plane overhead has a tag with its flight, route, height and speed.
- **A rocket and a Starlink train you can recognise.** A real launch is a Falcon 9 or a Starship climbing from your
  horizon: pad smoke, shock diamonds, the twilight plume, the booster falling back, a tag with its name and T+. The
  Starlink satellites really overhead glide in a row joined by a faint thread, with a tag that counts them.
  ([section 76](docs/GUIDE.md#76-the-look-18))

### Try 1.8 in ten minutes

Each step is on its own; none changes your panels or your sources. Every `pulse.toml` below is
`~/.config/omarchy-pulse/pulse.toml`: Pulse reads it again by itself within a few seconds, no restart.

**0. Update.** The same two lines as the install, from a terminal on the PC:

```bash
omarchy-theme-install https://github.com/piermark/omarchy-pulse-theme
~/.config/omarchy/themes/pulse/extras/install-live
omarchy-pulse doctor           # the first line says Omarchy Pulse 1.8.0
```

**1. The new look.** Nothing to do: your city now has one real light, volume, mist and reflections. Wait for the rain,
a plane over you (its tag says the flight, route, height and speed) or a real launch (the rocket and its T+). To
compare, `polish = false` under `[pulse]` brings back the 1.7 look; remove it to go back.

**2. Two new worlds.** Under `[pulse]`:

```toml
scene = "mars"         # a colony on Mars: the dome is your PC, rovers are your AI agents, launch pads your repos
# scene = "monolith"   # a crystal over a mirror lake: your cores, your agents as shards, your repos as pylons
```

Click anything to read it; clicking the dome or the crystal also says how rich it is drawing on your PC (*light* to
*ultra*). Back to your city: `scene = "skyline"` (or the one you had).

**3. Your datacenter in the city.** If you have panels with a status (Proxmox, Docker, Uptime Kuma, Home Assistant,
from [community sources](#community-sources)), name them:

```toml
[datacenter]
panels = ["proxmox", "docker"]   # the names of your [[source]] panels in pulse.toml
```

Every VM or container becomes a hall in front of the city; when one goes down, lightning strikes it.
([section 77](docs/GUIDE.md#77-your-datacenter-is-a-district-of-the-city-18))

**4. Describe your world.** `omarchy-pulse scene editor` → **✨ Describe…** → write what you want ("a harbour where
every container is a ship"). With `[creator] agent = "claude -p"` (any AI command) it is drawn for you; without it,
**Copy the request**, paste it into any AI and paste the reply back. Then **Save** and **Try on wallpaper**.
([section 78](docs/GUIDE.md#78-describe-your-world-18))

**5. Racing, rockets, Starlink.** Nothing to set: they follow the real sky over you. `omarchy-pulse sky` says when the
next Starlink train passes and the next launch.

#### 6. A scene from the community: blackout-room

The first scene someone else made for Pulse: Anatolii ([tolixx](https://github.com/tolixx)) drew it for his homelab in
Kyiv, which rides out power cuts on battery stations. A server room on a street at night: with the grid the houses are
lit and current runs along the line from the pole; in a power cut the street goes dark, lightning strikes, "on
battery" appears and the rack keeps blinking on its batteries, greener when they are full, redder as they drain.

**Install it** (one line; or in the editor: **Community** → blackout-room → **Install**):

```bash
omarchy-pulse scene get blackout-room
```

and put it on the wallpaper, under `[pulse]` in `pulse.toml`:

```toml
scene = "blackout-room"
```

**Give it your data.** It follows three panels, each a number: `grid` (how much of your power comes from the grid,
0 to 100), `ups` (the battery, in %) and `watts` (what your homelab draws, the meter is full at 500 W). With no
panels yet every value is 0, so you see the street in a power cut. With one UPS on NUT:

```toml
[[source]]
name = "grid"
command = "upsc myups@localhost ups.status | grep -q OL && echo 100 || echo 0"
every = 30

[[source]]
name = "ups"
command = "upsc myups@localhost battery.charge"
every = 60

[[source]]
name = "watts"
command = "upsc myups@localhost ups.realpower"
every = 60
```

**No UPS? Try it anyway** with three fixed numbers: `command = "echo 100"` for `grid`, `"echo 96"` for `ups`,
`"echo 240"` for `watts`. Then change the grid to `"echo 0"`: within half a minute the power goes out. Put it back to
`"echo 100"`: a chime, "the grid is back". Click the room for the watts and the battery.

Back to your city: `scene = "skyline"` (or the one you had). Want yours in everyone's gallery too?
[How to send a scene, in four steps](docs/PACKAGES.md#4-put-your-scene-in-the-gallery).

## New in 1.7

**Creator: make anything, share it, install other people's.** The scene editor
([docs/EDITOR.md](docs/EDITOR.md)) now builds complete scenes:

- **Your pictures and backgrounds.** Import PNG, JPEG, WebP or SVG: put them on the stage as shapes (tinted with the
  theme or in their own colours, mirrored, moving with your data like any shape) or behind everything as the scene's
  background. A scene can have its own palette too.
- **Every value of the engine.** A shape can follow the Sun, the Moon, the planes overhead (one copy per plane), the
  ISS, Starlink, the aurora, the next launch, music, meetings, and **your own panels**: anything a `[[source]]`, a push
  or Home Assistant turns into a number. *A tower that grows with my homelab's power draw* is now three clicks.
- **Your homelab as a city.** One copy of a shape for every row of your panels: a building per Proxmox VM, a fish
  per Docker container, a lamp per light, green, amber or red with each one, and lightning when one goes down.
- **Animations.** Keyframes: move, size, fade, turn; loop them, or play them when something happens. Presets: jump,
  fly, fade, grow, shake, spin.
- **Actions.** *When* an agent waits for you, CI fails or passes, a commit lands, a plane passes, the ISS rises, a
  rocket launches, it rains, you click a shape, a value crosses a line... *do*: play an animation, flash, show or hide a
  shape, or ask Pulse for one of its effects (fireworks, a banner, lightning, a shooting star, a balloon with your
  text, a rocket, a solar flare, a chime). Pulse keeps them rare and quiet while you work.
- **.pulse files.** **Export .pulse**: one file with the scene, its pictures, animations and actions, a preview, your
  name and a licence. **Import…** (or drop it on the editor): open it, change it, install it.
  `omarchy-pulse scene export | import` do the same from a terminal.
- **The Community gallery.** The editor's **Community** button shows the scenes people shared in
  [community/scenes/](community/scenes/), with **Install** and **Install & edit** (`omarchy-pulse scene gallery`,
  `scene get NAME`). Five to start: *homelab tower*, *container aquarium*, *green build rocket*, *moonlit garden*,
  *agent bell*. **Made something? Send it**: one pull request with your `.pulse`
  file ([docs/PACKAGES.md](docs/PACKAGES.md#4-put-your-scene-in-the-gallery)).

Safe by design: a package is data, or code that runs in Pulse's sandbox (no network, no files, no token). It never
carries commands or `[[source]]` scripts; pictures are checked byte by byte; gallery packages must match the reviewed
index; scenes see only the numbers of your panels and the names and states of their rows, never a private panel.
([section 75](docs/GUIDE.md#75-creator-pictures-animations-actions-pulse-files-and-the-community-17))

## New in 1.6

**The scene editor.** Make your own wallpaper world with the mouse: `omarchy-pulse scene editor`. Add boxes, circles,
stars, text, gauges, bars, buildings, waves and particles on a stage that shows Pulse's sky and horizon; give each one
the theme's colours, a glow, a motion (pulse, float, sway, spin, blink, drift); make it **follow your data** (a
building as tall as your RAM, rain as heavy as your network, a colour from green to red with the CPU) and **repeat** it
once per AI agent, repo or CPU core, coloured by their state. A click on a shape on the wallpaper shows its line
(`{cpu}% CPU`, `{name} · {state}`). Start from a template (*Agent lanterns*, *Core skyline*, *Packet rain*, *Night
garden*), move the demo sliders to see it react, or watch it with your live data; undo, redo, duplicate; **Save**,
**Try on wallpaper** in one click, **Export** to share it. What it makes is a normal community scene, in the same
sandbox, with no code inside the design. **[The editor's guide](docs/EDITOR.md)**: your first scene in two minutes,
every shape and option, recipes, sharing.

**All your AI agents.** Every agent Omarchy 4.5 ships (OpenCode, Claude Code, Codex, Gemini CLI, Copilot CLI, Pi),
Grok Build and Hermes Agent fly as drones through their own hooks: `omarchy-pulse hook setup` prints what to paste,
`omarchy-pulse hook install opencode` (or `pi`) adds the plugin. Each one has its **health**: `ok 92%` on its tag, a
bigger drone when its runs end well, flickering lights when it fails often. Agents working **in the same repo are
linked by a line of light** with packets running between them. Claude Duet, Meta Muse and other cloud agents can join
through their desktop notifications. Only the event, the tool's name and the folder reach Pulse.
([section 62](docs/GUIDE.md#62-all-your-ai-agents-their-health-and-working-together-16))

**Your desktop, your way.** `omarchy-pulse layout edit`: drag the panels where you want them, hide the ones you do not
need, Save on screen. ([63](docs/GUIDE.md#63-move-and-hide-the-panels-16))

**Omarchy in the city.** Your **Hyprland workspaces as a tower**, a lit window for each app and a lift on the one you
are on; **ivy** climbs it while the PC stays on and flowers after a week. Now and then a **hot-air balloon** brings an
**Omarchy tip** from your own key bindings (`[tips] enabled`, `every`; never in a meeting or a focus). **Pending
updates pile up as parcels**, the kernel in red, and a road sign blinks when the **news** asks for manual intervention
before an update (`omarchy-pulse news`). ([64-66](docs/GUIDE.md#64-the-workspaces-tower-and-the-ivy-16))

**Colours.** `[music] colors = true`: while a song plays, the city takes the colours of its cover. `omarchy-pulse
theme from-wallpaper PICTURE NAME`: a new Omarchy theme made from a picture's colours, for you to review and apply.
([67-68](docs/GUIDE.md#67-the-colours-of-your-music-16))

**Your day and your week.** The day's video is now a **diary**: its date, your day, its numbers (commits, agent tasks,
focus, planes...), at midnight or at the hour you choose (`[timelapse] at = "18:00"`). On Sunday night, **your week in
a minute** (`omarchy-pulse recap`). ([69](docs/GUIDE.md#69-your-days-diary-and-your-week-16))

**A local AI, your LEDs, your news.** `[ai] ollama`: what the sphere does not know goes to a model on your PC, with what
is on your wallpaper (`omarchy-pulse ask "is everything fine?"`, or by voice). `[wled]`: your WLED strips glow with
the sphere, red on a critical alert, a rainbow for fireworks. `[[news.rule]]`: a headline with "CVE" strikes the city
with lightning. ([70-72](docs/GUIDE.md#70-ask-the-sphere-with-a-local-ai-16))

**The friends' neighbourhood.** With a crew code you exchanged by hand, your friends' cities stand on your horizon: a
building each, as tall as their week of commits; a floor lights up when they commit, the light on top is their CI. Only
numbers travel, and only if you both turn it on (`[crew] city = true`). ([74](docs/GUIDE.md#74-the-friends-neighbourhood-16))

**And more.** Backups are trucks that drive to the vault and back, smoking when they fail; a calmer second monitor
(`secondary = "simple"`); during a focus the towers come down and the drones rest; a new device pings on the hacker
radar. ([73](docs/GUIDE.md#73-backups-as-trucks-a-calmer-second-monitor-focus-and-the-radar-16))

Everything of 1.5 is still there and works as before. The sphere is the default scene, as the guide always said. Every
new thing turns off with one line in `pulse.toml`; the ones that talk to other devices or services (Ollama, WLED,
cloud agents, news rules) are off until you set them.

## New in 1.5

**Scenes anyone can make.** A whole new world for the wallpaper in one JavaScript file, drawn from your real data,
without touching the engine: Pulse keeps the sky, the weather and the panels. `omarchy-pulse scene new` starts one,
the **scene lab** shows it live with demo data you control, checks it and exports a PNG or a video. Scenes run in a
sandbox (no network, no token), so you can install them from anyone. The example: **pond**, where every AI agent is a
fish and every repo a water lily. ([the creators' guide](docs/SCENES.md), [section 47](docs/GUIDE.md#47-community-scenes-15))

**Noir.** `scene = "noir"`: a futuristic megacity at night, towers with thousands of windows, searchlights, neon signs,
cars in the air and rain that never stops; a big screen on the central tower shows everything the sphere shows,
your agent's face too. All original. ([section 55](docs/GUIDE.md#55-the-noir-scene-15))

**The holidays and the eclipses.** Christmas lights and a tree, snow when it is really cold, the New Year's countdown
on the sphere and fireworks at midnight on your clock. The eclipses of the Sun and the Moon you can see from your city,
at their real time: the Moon's disc on the Sun where it really is, the stars at totality, the Moon turning red.
([48](docs/GUIDE.md#48-the-holidays-15), [49](docs/GUIDE.md#49-eclipses-15))

**A construction site for your code.** Every commit lowers a new floor onto its tower, a crane works where an AI agent
works, scaffolding while there are changes not committed, fire and a fire engine while the CI is red.
([section 50](docs/GUIDE.md#50-the-code-construction-site-15))

**What expires, and the weather of your infrastructure.** Certificates, domains and licences as candles that burn
down (amber at 14 days, red and aloud at 3). A panel in trouble brings a cloud over the city, a storm when it is
critical; with Zabbix, a double click on the storm acknowledges it (if you turn it on). Home Assistant sensors with
the age of their last report, so a stale "all clear" never looks healthy; your home's energy as a river, golden from
the sun. ([51-54](docs/GUIDE.md#51-what-expires-15))

**Pulse Net (beta, read-only, off by default).** Your network becomes part of the city: a building for each switch and
a window for each port (SNMP, LLDP), the firewall as the city walls with attacks arriving from their real country
(FortiGate, OPNsense, pfSense), syslog and SNMP traps as bolts on the device, Wi-Fi clients walking around their
antennas (UniFi, MikroTik, OpenWrt), hosts and VMs as towers and floors with migrations as lifts (Proxmox, vCenter),
backups as night trucks (restic, Proxmox Backup Server, Veeam), intruders as red-eyed robots (CrowdSec, Wazuh). A
**hacker** scene with a radar on your network and a read-only terminal (`omarchy-pulse term`), and a **NOC mode** with
incidents until someone acknowledges them. ([56-58](docs/GUIDE.md#57-pulse-net-15-beta-read-only))

**Your own wallpaper (1.5.1).** `wallpaper = "~/Pictures/city.jpg"` (or `"omarchy"`, your theme's background) puts
your picture under the widgets, with Pulse's live weather, planes and fireworks on top, or the panels alone; with
`wallpaper_colors = "picture"` the panels take their colours from it.
([section 60](docs/GUIDE.md#60-your-own-wallpaper-151))

**Reduced motion.** `motion = "reduced"`: everything keeps still and still shows your data, community scenes too.
([section 59](docs/GUIDE.md#59-reduced-motion-15))

Everything of 1.4 is still there and works as before; what is new and visible without any setting (the holiday lights
in December, an eclipse when there is one, a cloud over a panel in trouble) turns off with one line in `pulse.toml`.

## New in 1.4

**The real night sky.** The stars you know and their constellations, Venus, Jupiter, Mars and Saturn where they
really are, the Moon with its real phase, more shooting stars in the nights of a meteor shower. Computed for your
city, right now. ([guide, section 37](docs/GUIDE.md#37-the-real-night-sky))

**Starlink trains.** A few days after a launch, the train of satellites crosses your sky when it really does, lit by
the Sun while you are in the dark, and Pulse tells you five minutes before. ([section 38](docs/GUIDE.md#38-starlink-trains))

**Every kind of weather.** The Sun, clouds as many as the real ones, rain as hard as it really rains, snow, hail, fog,
and thunderstorms from where they really are, with bolts on your city and the thunder after them.
([section 39](docs/GUIDE.md#39-every-kind-of-weather))

**The air you breathe.** A haze over the city when the air is poor, pollen floating in the light, the air quality in
the weather panel. ([section 40](docs/GUIDE.md#40-the-air-you-breathe))

**The sound of your city.** A quiet soundscape, if you want it: the real rain and wind, a plane over you, thunder,
the station. ([section 41](docs/GUIDE.md#41-the-sound-of-your-city))

**Your room joins in.** Your lights flash with the lightning, turn red for an alert, blink for the doorbell (Home
Assistant, Philips Hue, a Philips TV's Ambilight), and your Echo speaks with Pulse.
([section 42](docs/GUIDE.md#42-the-lights-of-your-room-and-alexa))

**Your phone.** A focus done, the doorbell, your car charged, a quake near you: on your phone with the ntfy app.
([section 43](docs/GUIDE.md#43-your-phone))

**Pulse Wrapped.** On 31 December, your year as a video: your city day by day and the numbers of your year.
([section 44](docs/GUIDE.md#44-pulse-wrapped))

**Robots on the floor.** Your Docker or Podman containers, and your AI bots, walk in the city as little robots; their
eyes tell how they are. ([section 45](docs/GUIDE.md#45-robots-on-the-floor))

**The banner plane.** Now and then a little plane tows a banner across your sky, "♥ OMARCHY · WE CAN FIX
EVERYTHING", and when a new Omarchy is out it flies to tell you. ([section 46](docs/GUIDE.md#46-the-banner-plane))

Also: plane labels keep clear of your panels and of the sphere.

```bash
omarchy-pulse sky            # tonight: the Moon, the planets, the meteor shower, the next Starlink train, the air
omarchy-pulse phone new      # notifications on your phone
omarchy-pulse lights pair    # your Philips Hue lights (or use Home Assistant)
omarchy-pulse wrapped        # your year so far, as a video
```

## New in 1.3

**The city that grows with you.** Every focus you finish builds a building in your district; a streak of days keeps
its lights on, every week in a row a monument rises. Your city is your work. ([guide, section 28](docs/GUIDE.md#28-the-city-that-grows-with-you))

**The planes over you.** The real planes over your city cross your sky with their lights: flight, aircraft, height,
and where they come from and go.
([section 29](docs/GUIDE.md#29-the-planes-over-you))

**Your day as a video.** At midnight Pulse makes a short video of your day, ready to share, with no names in it.
([section 30](docs/GUIDE.md#30-your-day-as-a-video))

**Your calendar is a station.** Your next meetings are trains of light coming in; five minutes before, the voice tells
you. ([section 31](docs/GUIDE.md#31-your-calendar-is-a-station))

**Your home.** The doorbell turns the sphere into the door camera; your lights are one click away (Home Assistant).
([section 32](docs/GUIDE.md#32-your-home))

**Earthquakes and tsunamis.** A real earthquake shakes the city, and waves come from where it happened; a tsunami
message swells the sea at its feet. ([section 33](docs/GUIDE.md#33-earthquakes-and-tsunamis))

**The wind.** The real wind blows in the scene: rain and snow slant with it, gusts cross the sky, the sphere shows it.
([section 34](docs/GUIDE.md#34-the-wind))

**Your electric car.** Your car parked in the city, its battery filling with light while it charges; the sphere and the
voice tell you when it is charged, a panel starts the charge or the climate (Tessie or Home Assistant).
([section 35](docs/GUIDE.md#35-your-electric-car))

**Your language.** Everything Pulse says by itself can be in Italian, Spanish, French or German (`language` in
`[voice]`), with a voice of that language. ([section 21](docs/GUIDE.md#21-the-voice))

Also: the scene is now drawn at your screen's full resolution when the PC can (`quality`, half on battery, on screens
above 2560x1440 or when slow), the weather on the sphere follows the real sky (it was always a sun), old notifications leave the wallpaper,
`omarchy-pulse perf --probe` measures the real frame times, and a crash with `dmabuf = "on"` turns it off by itself.

Try 1.3 (each one is set up in `~/.config/omarchy-pulse/pulse.toml`: `[city]`, `[sky]`, `[timelapse]`, `[calendar]`,
`[home]`, `[car]`, `[quakes]`):

```bash
omarchy-pulse focus 25 "write the guide"   # finish it: a new building in your city
omarchy-pulse city                         # buildings, streak, monuments
omarchy-pulse calendar                     # the next trains (add your iCal address in [calendar])
omarchy-pulse car                          # your car's battery (Tessie token or Home Assistant in [car])
omarchy-pulse doorbell                     # the sphere becomes the door
omarchy-pulse timelapse today              # your day so far, as a video
```

Every command, every "hey sphere" phrase and the API are in one place: [guide, section 36](docs/GUIDE.md#36-every-command).

## New in 1.2

**Your real city.** Behind the towers stands the skyline of your city, with its landmarks: the Colosseum in Rome,
the Duomo in Milan, the One World Trade Center in New York, the Eiffel Tower, Big Ben, the Burj Khalifa, the Petronas
Towers and 90+ cities from every continent. It follows the city of your weather, or pick one: `skyline = "tokyo"`.
([guide, section 22](docs/GUIDE.md#22-your-real-city))

**The Git city.** Your repositories are towers: as tall as this week's commits, with a green, red or amber beacon
for their CI. A push lights the tower up with fireworks. ([section 23](docs/GUIDE.md#23-the-git-city))

**Space over your city.** When the ISS really passes over you, you see it cross your sky. Every SpaceX launch
(Starlink, ISS crews, Starship) counts down in the ticker and on the sphere, then lifts off from your horizon. A
geomagnetic storm paints a green aurora, a solar flare flashes the sky. ([section 24](docs/GUIDE.md#24-space-over-your-city))

**Your heartbeat.** Connect your watch (Home Assistant, a script) and the horizon beats with your heart, never leaving
your PC. ([section 25](docs/GUIDE.md#25-your-heartbeat))

**The crew.** Share a code with friends: their agents fly in your sky, yours in theirs, and you celebrate each other's
finished tasks, end-to-end sealed with the code. ([section 26](docs/GUIDE.md#26-the-crew))

**Hey sphere.** Hold a key and talk: "focus 25 minutes", "how are my servers?", "what did I miss?". Offline speech to
text (whisper), only safe commands, the answer out loud, in English or Italian. ([section 27](docs/GUIDE.md#27-hey-sphere))

Try them in a minute:

```
sed -i 's/^skyline = .*/skyline = "tokyo"/' ~/.config/omarchy-pulse/pulse.toml   # your city (no line yet? add skyline = "tokyo" under [pulse])
omarchy-pulse listen                  # say "hey sphere, how are my servers?" (needs whisper.cpp or faster-whisper)
omarchy-pulse crew new                # a code for your friends
omarchy-pulse fireworks "1.2 shipped"
```

**Faster updates, smoother frames.** `omarchy-pulse perf` shows how smooth every monitor is; on NVIDIA with 4K or two
monitors, `dmabuf = "on"` (guide section 12).

## Install

```
omarchy-theme-install https://github.com/piermark/omarchy-pulse-theme
~/.config/omarchy/themes/pulse/extras/install-live
```

The first line installs the **pulse** theme (colours, borders, btop, two still wallpapers). The second makes the
wallpaper live: it installs what it needs (`python-gobject`, `gtk-layer-shell`, `webkit2gtk-4.1`, and `ffmpeg` /
`playerctl` if missing), asks your city for the weather and starts Pulse (optional: `sudo pacman -S cava fping
espeak-ng` for the music, every device on your network and the voice). From then on it starts at every login while
the theme is **pulse**. Run it from a terminal on the PC itself, not over SSH.

- Live with **every** theme (Pulse takes each theme's colours): `extras/install-live --always`
- Back to the pulse theme only: `extras/install-live --theme`
- Remove it (your sources stay): `extras/install-live --remove`
- Check what is missing: `omarchy-pulse doctor` · logs: `journalctl --user -u omarchy-pulse -f`

**The full guide** (every source, the AI brief, cameras, a cookbook for Zabbix, Home Assistant, Proxmox, GitHub
Actions, MQTT and more, privacy, tuning, troubleshooting): [docs/GUIDE.md](docs/GUIDE.md).

## Scenes and wallpapers

Six live scenes: **sphere** (the LED sphere in the city, the default from 1.1), **skyline** (the city of your CPU
cores around a wireframe core), **orbit** (the cores become moons around a larger core), **calm** (a still grid,
lighter on the PC), **noir** (a futuristic megacity at night, a big screen instead of the sphere; 1.5) and **hacker**
(phosphor green or neon noir, a radar on your network; 1.5). Pick one with `scene` in
`~/.config/omarchy-pulse/pulse.toml`, or in one line:

```
sed -i 's/^scene = .*/scene = "noir"/' ~/.config/omarchy-pulse/pulse.toml    # sphere | skyline | orbit | calm | noir | hacker
```

Or a **community scene**, a whole world someone made in one file: `pond`, `monolith` and `mars` (1.8) come with Pulse, `omarchy-pulse scene list`
shows the ones you have. **Make your own with the mouse** in the scene editor (1.6: `omarchy-pulse scene editor`,
[docs/EDITOR.md](docs/EDITOR.md)), or in code ([docs/SCENES.md](docs/SCENES.md)). **Install other people's** from
the editor's Community tab, or share yours as a `.pulse` file (1.7, [docs/PACKAGES.md](docs/PACKAGES.md)).

Or **your own picture** under the widgets (1.5.1): `wallpaper = "~/Pictures/city.jpg"`, or `"omarchy"` for your theme's
background; `wallpaper_mode = "live"` keeps the real weather, planes and fireworks on top, `"panels"` shows the panels
only, and `wallpaper_colors = "picture"` gives the widgets the picture's colours ([section 60](docs/GUIDE.md#60-your-own-wallpaper-151)).

The wallpaper switches within about 3 seconds (`omarchy-pulse reload` to do it now).

The theme also has three still wallpapers, `01-pulse-skyline`, `02-pulse-orbit` and `03-pulse-sphere`, switched with Omarchy's
next-background key. You see them while Pulse is off or starting; they do not change the live scene.

## New in 1.1

**The LED sphere.** Thousands of LEDs on a turning globe. It shows the clock, the weather, your CPU, the latest news,
a camera, and takes over when it matters: red with an alert, a countdown in focus mode, the spectrum of your music,
the face of your agent while it works. Its light falls on the city and on the wet plaza. Two looks: `sphere = "real"`
(one continuous image) or `"led"` (thousands of separate lights).

**AI agents as drones.** Every agent at work is a drone flying over the city with its tag ("claude · omarchytv ·
editing · 4m"). A working drone scans the city with its beam, a waiting one blinks amber (it needs you), a finished
one lands green with fireworks, a failed one red with an alert. The agent's face lives in the sphere (or in the core).

```
omarchy-pulse agent grok working "reading the issue"     # any agent, any script
omarchy-pulse agent grok done
omarchy-pulse watch build -- make release                 # a drone while it runs, green or red at the end
omarchy-pulse hook claude                                 # Claude Code hooks (docs/GUIDE.md, section 16)
```

Only the agent, the tool it uses and the project folder reach the wallpaper: never your prompts or your code.

**The city dances with your music** (with [cava](https://github.com/karlstav/cava) installed): the towers, the horizon,
the floor and the sphere move with what you play, only while something plays.

**Fireworks** when an agent finishes, a focus session ends, an alert goes back to green, or whenever you want:
`omarchy-pulse fireworks "v1.1 shipped"` (`git push && omarchy-pulse fireworks "pushed"`).

**Focus mode**: `omarchy-pulse focus 25 "write the guide"` dims every panel, puts the countdown on the sphere and
under the clock, and ends with fireworks and a notification. `omarchy-pulse focus off` to stop.

**Your colours**: `[colors] preset = "synthwave" | "matrix" | "amber" | "ice" | "vapor" | "inferno"`, or `"theme"`
(the Omarchy theme in use, the default), and any single colour on top (`accent = "#39e1ff"`).

**It sees you.** The eyes of the face follow your pointer; move it on an empty desktop and the face wakes up and
looks at you. Lock the screen (or leave: hypridle can tell Pulse) and it falls asleep, z z z. When you are back it
smiles and shows **while you were away**: the alerts, the agents that finished, the new devices, the news, how many
notifications. ([guide, section 19](docs/GUIDE.md#19-the-face-that-sees-you-and-while-you-were-away))

**Your network on the grid.** Every device on your Wi-Fi is a car of light driving on the grid, with its name. A
device never seen before blinks amber and raises an alert: you know at once when something new joins your network.
Nothing leaves the PC. ([section 20](docs/GUIDE.md#20-your-network-on-the-grid))

**A voice** (off by default, offline with [piper](https://github.com/rhasspy/piper) or espeak-ng): "claude finished",
"grok needs you", "welcome back, while you were away...". Never while you share the screen, never at night.
([section 21](docs/GUIDE.md#21-the-voice))

**News**: a panel with the Omarchy releases and DHH's posts, on by default (`pulse-feed` reads any RSS/Atom feed).
**Click** a panel to run its `click = "..."` command. **The sky follows the sun**: dawn, day, dusk and night from
your weather. **Shooting stars** cross the sky when a notification or a news item arrives.

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
Actions, Docker, Hacker News, UniFi and more. Each one is a folder with the script, the `[[source]]` block
to paste in your `pulse.toml` and a short README.

Install one:

    cp community/zabbix/pulse-zabbix ~/.config/omarchy-pulse/sources/
    chmod +x ~/.config/omarchy-pulse/sources/pulse-zabbix

then paste `community/zabbix/source.toml` into `~/.config/omarchy-pulse/pulse.toml`.

**Made a source for your homelab, your printer, your smart home?** Open a pull request: see
[CONTRIBUTING.md](CONTRIBUTING.md). Sources run on other people's PCs, so they are read-only,
keep secrets out of the files and talk only to the service they say. Wanted: Pi-hole,
OctoPrint/Klipper, Uptime Kuma, Grafana, TrueNAS, Synology, Wazuh, Jellyfin, Tailscale.

**Made a scene?** We want it. Export it as a `.pulse` file and send it as a pull request to
[community/scenes/](community/scenes/): it appears in everyone's Community gallery
([docs/PACKAGES.md](docs/PACKAGES.md#4-put-your-scene-in-the-gallery)).

## Built in

System (CPU, every core, RAM, GPU, temperature, disk, battery), network, weather (Open-Meteo, no key), now
playing (playerctl), your last notifications (Omarchy 4.5 or mako), GitHub (releases and commits of the repos you follow, plus your
notifications after `gh auth login`), clock and a ticker. Turn any of them off in `[builtin]`.

## Privacy

When you share or record the screen, Pulse notices (Hyprland tells it) and hides cameras, notifications, your IP and
every `private = true` panel until you stop. By hand: `omarchy-pulse privacy on | off | auto | toggle` (bind it to a
key). The hub listens only on `127.0.0.1`, every request needs the token in `~/.config/omarchy-pulse/token`
(mode 600), and camera addresses and passwords never reach the page.

1.6: the agents' hooks send only the event, the tool's name and the folder (never prompts or code); agents in the same
repo are matched by a hash of the folder, never its path. Ollama and WLED are reached only on this PC or your network.
Your crew sees your city only if you set `[crew] city = true`, and then only numbers (commits this week, CI green or
red). Pulse reads the update news from archlinux.org every few hours (`[updates] news = false` to stop it) and runs
`checkupdates`, which never installs anything.

## Light on the PC

The scene runs at 20 frames a second only while you look at an empty desktop; with windows open it drops to 8, with
something full screen it stops, on battery it stays at 12 at most, and a PC that cannot keep up gets fewer frames by
itself. Panels refresh once a second. Tune it in `[pulse]`: `fps`, `scene = "calm"` (still grid), `scene = "orbit"`
(the cores become moons around the core), and `gpu = "never"` on a PC without a working GPU driver. It uses
WebKitGTK: about 400 MB of memory with one monitor. With two or more, `secondary = "simple"` (1.6) keeps the other
monitors calm and light (12 frames a second, no drones, planes or robots).

## Files

- `colors.toml`, `hyprland.conf` (cyan to magenta borders), `btop.theme`, `icons.theme`
- `backgrounds/01-pulse-skyline.png`, `02-pulse-orbit.png`, `03-pulse-sphere.png`: the stills, under the live wallpaper
- `extras/install-live`, `extras/pulse-live` (theme-set hook and login start), `extras/pulse/` (the engine:
  `omarchy-pulse`, the page in `web/`, `pulse.toml`, `sources/` with `pulse-ai`, `pulse-feed`, `pulse-ping`,
  `pulse-systemd`, `pulse-table`; 1.6: the scene editor `web/editor.html` and its engine `web/scene-design.js`, the
  OpenCode and Pi plugins in `agents/`)
- `docs/GUIDE.md`: the full guide · `docs/EDITOR.md`: the scene editor's guide · `docs/SCENES.md`: scenes in code ·
  `docs/PACKAGES.md`: .pulse files and the Community gallery · `docs/REVIEWING.md`: how community pull requests are
  reviewed
- `community/`: sources and, in `community/scenes/`, the Community gallery (`index.json`, written by
  `tools/scene-index.py`)

## Support

Pulse is free and MIT-licensed. If you find it useful and want to support development, you can [buy me a coffee](https://buymeacoffee.com/piermark) or sponsor me on [GitHub Sponsors](https://github.com/sponsors/piermark) — totally optional.

## License

MIT, © 2026 Marco Porro ([x.com/MarcoPorro](https://x.com/MarcoPorro)). Weather data by
[Open-Meteo](https://open-meteo.com) (CC BY 4.0). Satellite orbits by [CelesTrak](https://celestrak.org). The Omarchy icon on the
banner plane comes from [Dashboard Icons](https://dashboardicons.com/icons/omarchy) (Apache-2.0); Omarchy is a
project by Basecamp, and Pulse is a community theme, not an official one.
