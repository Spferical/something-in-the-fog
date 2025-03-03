use bevy::ecs::system::lifetimeless::Read;
use bevy::prelude::*;
use bevy::render::render_graph::ViewNode;
use bevy::render::render_resource::{
    BindGroupEntries, Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
};
use bevy::render::view::{ViewTarget, ViewUniformOffset, ViewUniforms};

use super::SdfPipeline;
use super::SdfTexture;

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

        let (Some(pipeline), Some(view_uniform_binding)) = (
            pipeline_cache.get_render_pipeline(sdf_pipeline.pipeline_id),
            world.resource::<ViewUniforms>().uniforms.binding(),
        ) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let bind_group = render_context.render_device().create_bind_group(
            SDF_BIND_GROUP,
            &sdf_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &sdf_pipeline.sampler,
                view_uniform_binding.clone(),
            )),
        );

        let mut sdf_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some(SDF_PASS),
            color_attachments: &[Some(RenderPassColorAttachment {
                // view: &sdf_texture.sdf.default_view,
                view: &post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            ..default()
        });

        sdf_pass.set_render_pipeline(pipeline);
        sdf_pass.set_bind_group(0, &bind_group, &[view_offset.offset]);
        sdf_pass.draw(0..3, 0..1);

        Ok(())
    }
}
