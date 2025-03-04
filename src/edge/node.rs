use bevy::ecs::system::lifetimeless::Read;
use bevy::prelude::*;
use bevy::render::render_graph::ViewNode;
use bevy::render::render_resource::{
    BindGroupEntries, Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
};
use bevy::render::view::{ViewTarget, ViewUniformOffset, ViewUniforms};

use super::EdgePipeline;
use super::EdgeTexture;

#[derive(Default)]
pub struct EdgeNode;

const EDGE_PASS: &str = "edge_pass";
const EDGE_BIND_GROUP: &str = "edge_bind_group";

impl ViewNode for EdgeNode {
    type ViewQuery = (Read<ViewTarget>, Read<EdgeTexture>);

    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        (view_target, edge_texture): bevy::ecs::query::QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let sdf_pipeline = world.resource::<EdgePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let (Some(pipeline), Some(view_uniform_binding)) = (
            pipeline_cache.get_render_pipeline(sdf_pipeline.pipeline_id),
            world.resource::<ViewUniforms>().uniforms.binding(),
        ) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let output = edge_texture.edges.default_view.clone();

        let bind_group = render_context.render_device().create_bind_group(
            EDGE_BIND_GROUP,
            &sdf_pipeline.layout,
            &BindGroupEntries::sequential((post_process.source, &sdf_pipeline.sampler)),
        );

        let mut edge_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some(EDGE_PASS),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &output,
                resolve_target: None,
                ops: Operations::default(),
            })],
            ..default()
        });

        edge_pass.set_render_pipeline(pipeline);
        // are we missing a dynamic index here?
        edge_pass.set_bind_group(0, &bind_group, &[]);
        edge_pass.draw(0..3, 0..1);

        Ok(())
    }
}
