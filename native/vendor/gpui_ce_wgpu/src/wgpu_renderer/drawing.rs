use gpui::{
    AtlasTextureId, MonochromeSprite, Path, PolychromeSprite, Quad, ScaledPixels, Shadow,
    SubpixelSprite, Underline,
};

use crate::WgpuTextureInfo;
use gpui_render::shaders::interface::{self as shader_interface, BufferData};

use super::{
    WgpuRenderer,
    buffers::{InstanceSlice, InstanceUpload},
    frame, path_types, pipelines,
};

impl WgpuRenderer {
    pub(super) fn draw_quads(
        &self,
        quads: &[Quad],
        smoothed: bool,
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        let pipelines = &self.resources().pipelines;
        let pipeline = if smoothed {
            &pipelines.smoothed_quads
        } else {
            &pipelines.quads
        };
        self.draw_instances(quads, pipeline, instances, pass)
    }

    pub(super) fn draw_shadows(
        &self,
        shadows: &[Shadow],
        smoothed: bool,
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        let pipelines = &self.resources().pipelines;
        let pipeline = if smoothed {
            &pipelines.smoothed_shadows
        } else {
            &pipelines.shadows
        };
        self.draw_instances(shadows, pipeline, instances, pass)
    }

    pub(super) fn draw_underlines(
        &self,
        underlines: &[Underline],
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        self.draw_instances(
            underlines,
            &self.resources().pipelines.underlines,
            instances,
            pass,
        )
    }

    pub(super) fn draw_monochrome_sprites(
        &self,
        sprites: &[MonochromeSprite],
        texture_id: AtlasTextureId,
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        if sprites.is_empty() {
            return Ok(());
        }
        let Some(texture) = self.atlas.get_texture_info(texture_id) else {
            return Ok(());
        };
        self.draw_instances_with_texture(
            sprites,
            texture_id,
            &texture,
            &self.resources().pipelines.monochrome_sprites,
            instances,
            pass,
        )
    }

    pub(super) fn draw_subpixel_sprites(
        &self,
        sprites: &[SubpixelSprite],
        texture_id: AtlasTextureId,
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        if sprites.is_empty() {
            return Ok(());
        }
        let Some(texture) = self.atlas.get_texture_info(texture_id) else {
            return Ok(());
        };
        let resources = self.resources();
        let pipeline = resources
            .pipelines
            .subpixel_sprites
            .as_ref()
            .unwrap_or(&resources.pipelines.monochrome_sprites);
        self.draw_instances_with_texture(sprites, texture_id, &texture, pipeline, instances, pass)
    }

    pub(super) fn draw_polychrome_sprites(
        &self,
        sprites: &[PolychromeSprite],
        texture_id: AtlasTextureId,
        smoothed: bool,
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        if sprites.is_empty() {
            return Ok(());
        }
        let Some(texture) = self.atlas.get_texture_info(texture_id) else {
            return Ok(());
        };
        let pipelines = &self.resources().pipelines;
        let pipeline = if smoothed {
            &pipelines.smoothed_polychrome_sprites
        } else {
            &pipelines.polychrome_sprites
        };
        self.draw_instances_with_texture(sprites, texture_id, &texture, pipeline, instances, pass)
    }

    fn draw_instances<T: BufferData>(
        &self,
        values: &[T],
        pipeline: &pipelines::WgpuRenderPipeline,
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        if values.is_empty() {
            return Ok(());
        }
        let resources = self.resources();
        self.draw_bound_instances(
            values,
            pipeline,
            resources.instances.bind_group(),
            instances,
            pass,
        )
    }

    fn draw_bound_instances<T: BufferData>(
        &self,
        values: &[T],
        pipeline: &pipelines::WgpuRenderPipeline,
        bind_group: &wgpu::BindGroup,
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        let Some(slice) = instances.write(values) else {
            return Err(frame::DrawError::CapacityPlanningInvariant);
        };
        self.draw_bound_slice(pipeline, bind_group, &slice, pass);
        Ok(())
    }

    fn draw_bound_slice<T>(
        &self,
        pipeline: &pipelines::WgpuRenderPipeline,
        bind_group: &wgpu::BindGroup,
        slice: &InstanceSlice<T>,
        pass: &mut wgpu::RenderPass<'_>,
    ) {
        pass.set_pipeline(pipeline);
        slice.set_data_bind_group(pass, bind_group);
        pass.draw(0..pipeline.fixed_vertex_count(), slice.range());
    }

    fn draw_instances_with_texture<T: BufferData>(
        &self,
        values: &[T],
        texture_id: AtlasTextureId,
        texture: &WgpuTextureInfo,
        pipeline: &pipelines::WgpuRenderPipeline,
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        if values.is_empty() {
            return Ok(());
        }
        let resources = self.resources();
        let bind_group = resources.instances.textured_bind_group(
            &resources.device,
            &resources.bind_group_layouts,
            pipeline.data_layout(),
            texture_id,
            texture.identity,
            &texture.view,
            &resources.atlas_sampler,
        );
        self.draw_bound_instances(values, pipeline, &bind_group, instances, pass)
    }

    pub(super) fn draw_paths_from_intermediate(
        &self,
        paths: &[Path<ScaledPixels>],
        instances: &mut InstanceUpload,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> frame::DrawResult {
        let sprite_count = path_types::sprite_count(paths);
        let Some(sprite_slice) = instances.write_iter(sprite_count, path_types::sprites(paths))
        else {
            return Err(frame::DrawError::CapacityPlanningInvariant);
        };
        let resources = self.resources();
        let Some(path_intermediate_view) = resources.path_intermediate_view.as_ref() else {
            return Err(frame::DrawError::MissingIntermediateTarget);
        };
        let bind_group = resources.instances.path_bind_group(
            &resources.device,
            &resources.bind_group_layouts,
            resources.pipelines.paths.data_layout(),
            path_intermediate_view,
            &resources.atlas_sampler,
        );
        self.draw_bound_slice(&resources.pipelines.paths, &bind_group, &sprite_slice, pass);
        Ok(())
    }

    pub(super) fn draw_paths_to_intermediate(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        paths: &[Path<ScaledPixels>],
        instances: &mut InstanceUpload,
    ) -> frame::DrawResult {
        if std::env::var_os("GPUI_FULL_PATH_PASSES").is_none() {
            if let Some(result) = self.draw_paths_to_tile(encoder, paths, instances) {
                return result;
            }
        }
        self.draw_paths_to_intermediate_full(encoder, paths, instances)
    }

    /// pulse-native: the batch is drawn in a target only as big as its paths, and that piece is copied into the
    /// intermediate texture where the paths are. The same multisampling, the same resolve, the same picture: only
    /// the memory a batch touches shrinks from the whole screen to the batch. None when it cannot (no slot left).
    fn draw_paths_to_tile(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        paths: &[Path<ScaledPixels>],
        instances: &mut InstanceUpload,
    ) -> Option<frame::DrawResult> {
        let vertex_count = path_types::rasterization_vertex_count(paths);
        if vertex_count == 0 {
            return Some(Ok(()));
        }
        let (tw, th) = (self.target.width(), self.target.height());
        // where each path is, in whole device pixels, a pixel wider for the antialiased edges
        let rect = |path: &Path<ScaledPixels>| -> Option<[u32; 4]> {
            let b = path.clipped_bounds();
            let x0 = (b.origin.x.0.floor() as i64 - 1).clamp(0, tw as i64) as u32;
            let y0 = (b.origin.y.0.floor() as i64 - 1).clamp(0, th as i64) as u32;
            let x1 = ((b.origin.x.0 + b.size.width.0).ceil() as i64 + 1).clamp(0, tw as i64) as u32;
            let y1 = ((b.origin.y.0 + b.size.height.0).ceil() as i64 + 1).clamp(0, th as i64) as u32;
            (x1 > x0 && y1 > y0).then_some([x0, y0, x1, y1])
        };
        // paths close together are drawn as one piece; pieces far apart are packed side by side in the tile, so
        // a batch with a plane on the left and a drone on the right costs two small pieces, not the screen between.
        // Pieces never overlap on the screen (each is copied back whole).
        let area = |r: &[u32; 4]| (r[2] - r[0]) as u64 * (r[3] - r[1]) as u64;
        let union = |a: &[u32; 4], b: &[u32; 4]| [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])];
        let touch = |a: &[u32; 4], b: &[u32; 4]| a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3];
        let mut pieces: Vec<[u32; 4]> = Vec::new();
        let mut of_path: Vec<usize> = Vec::with_capacity(paths.len());
        for path in paths {
            let Some(r) = rect(path) else {
                of_path.push(usize::MAX);
                continue;
            };
            let near = pieces.iter().position(|p| touch(p, &r) || area(&union(p, &r)) <= (area(p) + area(&r)) * 3 / 2 + 4096);
            match near {
                Some(i) => {
                    pieces[i] = union(&pieces[i], &r);
                    of_path.push(i);
                }
                None => {
                    pieces.push(r);
                    of_path.push(pieces.len() - 1);
                }
            }
        }
        if pieces.is_empty() {
            return Some(Ok(()));
        }
        // a piece that grew over another: one piece (until none overlap)
        let mut joined: Vec<usize> = (0..pieces.len()).collect();
        loop {
            let mut merged = false;
            'outer: for i in 0..pieces.len() {
                if joined[i] != i {
                    continue;
                }
                for j in i + 1..pieces.len() {
                    if joined[j] == j && touch(&pieces[i], &pieces[j]) {
                        pieces[i] = union(&pieces[i], &pieces[j]);
                        joined[j] = i;
                        merged = true;
                        break 'outer;
                    }
                }
            }
            if !merged {
                break;
            }
        }
        let root = |mut i: usize| {
            while joined[i] != i {
                i = joined[i];
            }
            i
        };
        let live: Vec<usize> = (0..pieces.len()).filter(|&i| joined[i] == i).collect();
        // shelf packing: the tallest first, rows as wide as the widest piece or the square root of the whole
        let mut order = live.clone();
        order.sort_by_key(|&i| std::cmp::Reverse(pieces[i][3] - pieces[i][1]));
        let total: u64 = live.iter().map(|&i| area(&pieces[i])).sum();
        let widest = live.iter().map(|&i| pieces[i][2] - pieces[i][0]).max().unwrap_or(1);
        let row_w = widest.max(((total as f64).sqrt() * 1.2) as u32).min(tw.max(widest));
        let mut place = vec![(0u32, 0u32); pieces.len()];
        let (mut x, mut y, mut row_h, mut rw, mut rh) = (0u32, 0u32, 0u32, 0u32, 0u32);
        for &i in &order {
            let (w, h) = (pieces[i][2] - pieces[i][0], pieces[i][3] - pieces[i][1]);
            if x > 0 && x + w > row_w {
                y += row_h;
                x = 0;
                row_h = 0;
            }
            place[i] = (x, y);
            x += w;
            row_h = row_h.max(h);
            rw = rw.max(x);
            rh = rh.max(y + row_h);
        }
        if rw > tw.max(1) * 2 || rh > th.max(1) * 2 || rw > 8192 || rh > 8192 {
            return None;
        }
        if std::env::var_os("PULSE_BATCHES").is_some() {
            eprintln!("path batch {} paths {} pieces {}x{} of {}x{}", paths.len(), live.len(), rw, rh, tw, th);
        }
        let resources = self.resources();
        let slot = resources.path_batch_globals.next.get();
        if slot >= super::resources::PATH_BATCH_SLOTS {
            return None;
        }
        resources.path_batch_globals.next.set(slot + 1);
        let intermediate = resources.path_intermediate_texture.as_ref()?;

        // a tile at least as big: the smallest that fits, or a new one (sizes rounded up, so few are made)
        let mut tiles = resources.path_tiles.borrow_mut();
        let stamp = tiles.iter().map(|t| t.used).max().unwrap_or(0) + 1;
        let fit = tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.width >= rw && t.height >= rh)
            .min_by_key(|(_, t)| t.width as u64 * t.height as u64)
            .map(|(i, _)| i);
        let index = match fit {
            Some(i) => i,
            None => {
                let round = |v: u32, _max: u32| v.div_ceil(128) * 128;
                if tiles.len() >= 4 {
                    // forget the one used longest ago
                    if let Some((oldest, _)) = tiles.iter().enumerate().min_by_key(|(_, t)| t.used) {
                        tiles.swap_remove(oldest);
                    }
                }
                tiles.push(super::resources::PathTile::new(
                    &resources.device,
                    self.target.format(),
                    round(rw, tw),
                    round(rh, th),
                    self.rendering_params.path_sample_count,
                ));
                tiles.len() - 1
            }
        };
        let tile = &mut tiles[index];
        tile.used = stamp;

        // the batch's own viewport: its tile; its vertices and clips moved by where the batch is
        let globals = gpui_render::shaders::common::GlobalUniforms {
            viewport_size: wgsl_rs::std::vec2f(tile.width as f32, tile.height as f32),
            premultiplied_alpha: gpui_render::shaders::common::ShaderBool::Disabled,
            padding: 0,
        };
        resources.path_batch_globals.write(&resources.queue, slot, &globals);
        // every path moved from its place on the screen to its piece's place in the tile
        let shift: Vec<(ScaledPixels, ScaledPixels)> = of_path
            .iter()
            .map(|&k| {
                if k == usize::MAX {
                    return (ScaledPixels(-1.0e6), ScaledPixels(-1.0e6)); // nothing of it shows: out of the tile
                }
                let r = root(k);
                (ScaledPixels(pieces[r][0] as f32 - place[r].0 as f32), ScaledPixels(pieces[r][1] as f32 - place[r].1 as f32))
            })
            .collect();
        let shifted = paths.iter().zip(shift.iter()).flat_map(|(path, &(ox, oy))| {
            let mut bounds = path.clipped_bounds();
            bounds.origin.x = bounds.origin.x - ox;
            bounds.origin.y = bounds.origin.y - oy;
            path.vertices.iter().map(move |vertex| path_types::PathRasterizationVertex {
                xy_position: gpui::Point { x: vertex.xy_position.x - ox, y: vertex.xy_position.y - oy },
                curve_position: vertex.st_position,
                color: path.color,
                bounds,
            })
        });
        let Some(vertex_slice) = instances.write_iter(vertex_count, shifted) else {
            return Some(Err(frame::DrawError::CapacityPlanningInvariant));
        };
        let (target_view, resolve_target) = match &tile.msaa {
            Some((_, msaa_view)) => (msaa_view, Some(&tile.resolve.1)),
            None => (&tile.resolve.1, None),
        };
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("path_tile_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: if resolve_target.is_some() { wgpu::StoreOp::Discard } else { wgpu::StoreOp::Store },
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&resources.pipelines.path_rasterization);
            pass.set_bind_group(shader_interface::GLOBAL_BIND_GROUP, &resources.path_batch_globals.groups[slot], &[]);
            vertex_slice.set_data_bind_group(&mut pass, resources.instances.bind_group());
            pass.draw(vertex_slice.range(), 0..1);
        }
        for &i in &live {
            let (w, h) = (pieces[i][2] - pieces[i][0], pieces[i][3] - pieces[i][1]);
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo { texture: &tile.resolve.0, mip_level: 0, origin: wgpu::Origin3d { x: place[i].0, y: place[i].1, z: 0 }, aspect: wgpu::TextureAspect::All },
                wgpu::TexelCopyTextureInfo { texture: intermediate, mip_level: 0, origin: wgpu::Origin3d { x: pieces[i][0], y: pieces[i][1], z: 0 }, aspect: wgpu::TextureAspect::All },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
        }
        Some(Ok(()))
    }

    fn draw_paths_to_intermediate_full(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        paths: &[Path<ScaledPixels>],
        instances: &mut InstanceUpload,
    ) -> frame::DrawResult {
        let vertex_count = path_types::rasterization_vertex_count(paths);
        if vertex_count == 0 {
            return Ok(());
        }

        let Some(vertex_slice) =
            instances.write_iter(vertex_count, path_types::rasterization_vertices(paths))
        else {
            return Err(frame::DrawError::CapacityPlanningInvariant);
        };
        let resources = self.resources();
        let Some(path_intermediate_view) = resources.path_intermediate_view.as_ref() else {
            return Err(frame::DrawError::MissingIntermediateTarget);
        };
        let (target_view, resolve_target) = if let Some(msaa_view) = &resources.path_msaa_view {
            (msaa_view, Some(path_intermediate_view))
        } else {
            (path_intermediate_view, None)
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("path_rasterization_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    // Only the resolved texture is sampled later. The MSAA
                    // attachment is transient and cannot use StoreOp::Store.
                    store: if resolve_target.is_some() {
                        wgpu::StoreOp::Discard
                    } else {
                        wgpu::StoreOp::Store
                    },
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        pass.set_pipeline(&resources.pipelines.path_rasterization);
        pass.set_bind_group(
            shader_interface::GLOBAL_BIND_GROUP,
            &resources.path_globals_bind_group,
            &[],
        );
        vertex_slice.set_data_bind_group(&mut pass, resources.instances.bind_group());
        pass.draw(vertex_slice.range(), 0..1);
        Ok(())
    }
}
