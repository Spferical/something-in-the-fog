use bevy::render::render_resource::{Texture, TextureView};
use bevy::render::texture::GpuImage;
use bevy::render::view::RenderLayers;
use bevy::sprite::Material2dPlugin;
use bevy::{
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::*,
    render::{
        camera::RenderTarget,
        render_asset::{RenderAsset, RenderAssets},
        render_graph::{RenderGraphApp, ViewNodeRunner},
        renderer::RenderDevice,
        texture::{CachedTexture, TextureCache},
        view::prepare_view_targets,
        Render, RenderApp, RenderSet,
    },
};

/*use crate::sdf::{
    prepare_sdf_settings, prepare_sdf_texture, AllSdfSettings, SdfNode, SdfPass, SdfPipeline,
};*/
use crate::sdf::{prepare_sdf_texture, setup_sdf_pass, SdfMaterial};

#[derive(Component, Clone)]
pub struct OccluderTextureCpu(pub Handle<Image>);

#[derive(Component)]
pub struct OccluderTexture {
    pub handle: TextureView,
}

pub struct Renderer;

fn debug_render_targets(q: Query<&Camera>) {
    for camera in &q {
        match &camera.target {
            RenderTarget::Window(wid) => {
                eprintln!("Camera renders to window with id: {:?}", wid);
            }
            RenderTarget::Image(handle) => {
                eprintln!("Camera renders to image asset with id: {:?}", handle);
            }
            RenderTarget::TextureView(_) => {
                eprintln!("This is a special camera that outputs to something outside of Bevy.");
            }
        }
    }
}

impl Plugin for Renderer {
    fn build(&self, app: &mut App) {
        /*let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };*/
        app.add_plugins(Material2dPlugin::<SdfMaterial>::default())
            .add_systems(PreStartup, prepare_sdf_texture)
            .add_systems(PostStartup, setup_sdf_pass)
            .add_systems(Update, debug_render_targets);

        // set up render systems here
        /*render_app
        .init_resource::<AllSdfSettings>()
        .add_systems(
            Render,
            (
                prepare_sdf_settings.in_set(RenderSet::Prepare),
                prepare_occluder_texture.in_set(RenderSet::Prepare),
                prepare_sdf_texture
                    .after(prepare_view_targets)
                    .in_set(RenderSet::ManageViews),
            ),
        )
        .add_render_graph_node::<ViewNodeRunner<SdfNode>>(Core2d, SdfPass)
        .add_render_graph_edges(
            Core2d,
            (
                Node2d::EndMainPass,
                SdfPass,
                Node2d::EndMainPassPostProcessing,
            ),
        );*/
    }

    /*fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // render_app.init_resource::<SdfPipeline>();
    }*/
}
