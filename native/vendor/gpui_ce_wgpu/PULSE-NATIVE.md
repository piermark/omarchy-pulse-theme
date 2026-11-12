# gpui_ce_wgpu 0.3.0, with one change for pulse-native

This is the published crate [gpui_ce_wgpu 0.3.0](https://crates.io/crates/gpui_ce_wgpu) (Apache-2.0, see
LICENSE-APACHE), the wgpu renderer of gpui-ce, used through `[patch.crates-io]` in `../../Cargo.toml`. Everything
is as published except three files in `src/wgpu_renderer/`, each change marked `pulse-native`:

- `drawing.rs`: a batch of paths (the shapes GPUI tessellates into triangles) is drawn in a small target only as big
  as the batch, with the same multisampling and the same resolve, and then copied into the full-size intermediate
  texture where the paths are. Paths far apart in one batch are packed side by side in that target. Upstream draws
  every batch into a multisampled texture as big as the screen and resolves all of it: a scene with sixty batches a
  frame then moves sixty screens of memory (four samples each) per frame, which binds an older GPU (an Intel Haswell
  at 1536×864; four times more in 4K). Here a batch moves about a sixteenth of the screen on average.
- `resources.rs`: the small targets, kept and reused (four at most), and one copy of the path globals per batch (its
  own viewport); the full-screen multisampled texture is made only when the upstream way is asked for.
- `frame.rs`: the per-batch slots start again every frame; `PULSE_BATCHES=1` prints the batches and what a frame
  costs here.

`GPUI_FULL_PATH_PASSES=1` draws the upstream way, to compare.
