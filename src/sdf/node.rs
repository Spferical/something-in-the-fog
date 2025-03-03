use bevy::ecs::system::lifetimeless::Read;
use bevy::prelude::*;
use bevy::render::render_graph::ViewNode;
use bevy::render::render_resource::{
    BindGroupEntries, Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
};
use bevy::render::view::{ViewTarget, ViewUniformOffset, ViewUniforms};

use super::SdfTexture;
use super::{AllSdfSettings, SdfPipeline};

#[derive(Default)]
pub struct SdfNode;

const SDF_PASS: &str = "sdf_pass";
const SDF_BIND_GROUP: &str = "sdf_bind_group";

impl ViewNode for SdfNode {
    type ViewQuery = (Read<ViewTarget>, Read<ViewUniformOffset>, Read<SdfTexture>);

    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        (view_target, view_offset, sdf_texture): bevy::ecs::query::QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let sdf_pipeline = world.resource::<SdfPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let all_sdf_settings = world.resource::<AllSdfSettings>();

        let (Some(pipeline), Some(view_uniform_binding)) = (
            pipeline_cache.get_render_pipeline(sdf_pipeline.pipeline_id),
            world.resource::<ViewUniforms>().uniforms.binding(),
        ) else {
            return Ok(());
        };

        let screen_size = view_target.main_texture().size();
        let larger_dim = screen_size.width.max(screen_size.height);
        let num_passes = (larger_dim as f32).log2().ceil() as usize;

        for i in 0..num_passes {
            let Some(sdf_settings_binding) = all_sdf_settings.all[i].binding() else {
                println!("early exit");
                return Ok(());
            };

            let (ping, pong) = if (i % 2) == 0 {
                (&sdf_texture.ping, &sdf_texture.pong)
            } else {
                (&sdf_texture.pong, &sdf_texture.ping)
            };

            let post_process = view_target.post_process_write();

            let output = if i == num_passes - 1 {
                post_process.destination
            } else {
                &pong.default_view
            };

            let bind_group = render_context.render_device().create_bind_group(
                SDF_BIND_GROUP,
                &sdf_pipeline.layout,
                &BindGroupEntries::sequential((
                    post_process.source,
                    &ping.default_view,
                    &sdf_pipeline.sampler,
                    view_uniform_binding.clone(),
                    sdf_settings_binding.clone(),
                )),
            );

            let mut sdf_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some(SDF_PASS),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: output,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                ..default()
            });

            sdf_pass.set_render_pipeline(pipeline);
            // are we missing a dynamic index here?
            sdf_pass.set_bind_group(0, &bind_group, &[view_offset.offset]);
            sdf_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}
