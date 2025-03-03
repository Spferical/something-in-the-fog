use bevy::{
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::*,
    render::{
        render_graph::{RenderGraphApp, ViewNodeRunner},
        view::prepare_view_targets,
        Render, RenderApp,
    },
};

use crate::sdf::{prepare_sdf_texture, SdfNode, SdfPass, SdfPipeline};

pub struct Renderer;

impl Plugin for Renderer {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // set up render systems here
        render_app
            .add_systems(Render, prepare_sdf_texture.after(prepare_view_targets))
            .add_render_graph_node::<ViewNodeRunner<SdfNode>>(Core2d, SdfPass)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::EndMainPass,
                    SdfPass,
                    Node2d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<SdfPipeline>();
    }
}
