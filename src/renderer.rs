use bevy::render::render_resource::{Texture, TextureView};
use bevy::render::texture::GpuImage;
use bevy::render::view::RenderLayers;
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

use crate::sdf::{
    prepare_sdf_settings, prepare_sdf_texture, AllSdfSettings, SdfNode, SdfPass, SdfPipeline,
};

#[derive(Component)]
pub struct OccluderTextureCpu(pub Handle<Image>);

#[derive(Component)]
pub struct OccluderTexture {
    pub handle: TextureView,
}

pub struct Renderer;

fn prepare_occluder_texture(
    mut commands: Commands,
    occluders: Query<(Entity, &OccluderTextureCpu)>,
    images: Res<RenderAssets<GpuImage>>,
) {
    println!("before preparing occluder texture");
    let Ok((entity_id, texture_cpu)) = occluders.get_single() else {
        return;
    };

    println!("preparing occluder texture");
    let image = images.get(&texture_cpu.0).unwrap().clone();
    let occluder_texture = OccluderTexture {
        handle: image.texture_view,
    };
    commands.entity(entity_id).insert(occluder_texture);

    /*match &camera.target {
        RenderTarget::Image(image) => {
            let image = images.get(image).unwrap().clone();
            let occluder_texture = OccluderTexture {
                handle: image.texture_view,
            };
            commands.entity(camera_id).insert(occluder_texture);
        }
        _ => {
            println!("in here");
            return;
        }
    };*/
}

impl Plugin for Renderer {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // set up render systems here
        render_app
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
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<SdfPipeline>();
    }
}
