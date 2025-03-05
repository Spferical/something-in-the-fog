use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::{asset::Handle, ecs::component::Component, image::Image};

mod mat;
mod prepare;

use crate::renderer::OccluderTextureCpu;
pub use mat::EdgeMaterial;
pub use prepare::{on_resize_edge_texture, prepare_edge_texture};

#[derive(Component)]
pub struct EdgeTexture(pub Handle<Image>);

const EDGE_ORDER_OFFSET: isize = 1;
const EDGE_LAYER: usize = 2;

pub fn setup_edge_pass(
    window: Single<&Window>,
    mut commands: Commands,
    edge_texture_query: Query<&EdgeTexture>,
    occluder_texture_query: Query<&OccluderTextureCpu>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<EdgeMaterial>>,
) {
    let (width, height) = (
        window.resolution.physical_width() as f32,
        window.resolution.physical_height() as f32,
    );

    let Ok(edge_texture) = edge_texture_query.get_single() else {
        return;
    };
    let Ok(occluder_texture) = occluder_texture_query.get_single() else {
        return;
    };

    let fullscreen_mesh = meshes.add(Rectangle::new(width, height));

    let camera_postprocess = Camera {
        clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
        target: edge_texture.0.clone().into(),
        order: EDGE_ORDER_OFFSET,
        ..default()
    };
    commands.spawn((
        Camera2d,
        camera_postprocess,
        RenderLayers::layer(EDGE_LAYER),
    ));

    commands.spawn((
        Mesh2d(fullscreen_mesh.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)),
        MeshMaterial2d(materials.add(EdgeMaterial {
            screen_texture: Some(occluder_texture.0.clone()),
        })),
        RenderLayers::layer(EDGE_LAYER),
    ));
}
