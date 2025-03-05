use bevy::prelude::*;
use bevy::render::render_graph::RenderLabel;
use bevy::render::texture::CachedTexture;

mod mat;
mod node;
mod prepare;

pub use crate::renderer::OccluderTextureCpu;
use bevy::render::view::RenderLayers;
pub use mat::SdfMaterial;
pub use prepare::prepare_sdf_texture;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SdfPass;

#[derive(Component)]
pub struct SdfTexture {
    pub iters: Vec<Handle<Image>>,
}

const SDF_ORDER_OFFSET: usize = 1;
const SDF_START_LAYER: usize = 31;

pub fn setup_sdf_pass(
    window: Single<&Window>,
    mut commands: Commands,
    sdf_texture_query: Query<&SdfTexture>,
    occluder_texture_query: Query<&OccluderTextureCpu>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SdfMaterial>>,
) {
    let (width, height) = (
        window.resolution.physical_width() as f32,
        window.resolution.physical_height() as f32,
    );

    let larger_dim = width.max(height);
    let num_passes = (larger_dim as f32).log2().ceil() as usize;

    let Ok(sdf_texture) = sdf_texture_query.get_single() else {
        return;
    };
    let Ok(occluder_texture) = occluder_texture_query.get_single() else {
        return;
    };

    let fullscreen_mesh = meshes.add(Rectangle::new(width, height));

    for i in 0..num_passes {
        let ping_it = SDF_START_LAYER - i;
        let ping_image = sdf_texture.iters[i].clone();
        let pong_image = sdf_texture.iters[i + 1].clone();

        if i == num_passes - 1 {
            let camera_postprocess = Camera {
                clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
                ..default()
            };
            commands.spawn((Camera2d, camera_postprocess, RenderLayers::layer(ping_it)));
        } else {
            let camera_postprocess = Camera {
                clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
                target: pong_image.clone().into(),
                order: (SDF_ORDER_OFFSET + i) as isize,
                ..default()
            };
            commands.spawn((Camera2d, camera_postprocess, RenderLayers::layer(ping_it)));
        }

        commands.spawn((
            Mesh2d(fullscreen_mesh.clone()),
            MeshMaterial2d(materials.add(SdfMaterial {
                screen_texture: Some(occluder_texture.0.clone()),
                sdf: Some(ping_image.clone()),
                iteration: i as i32,
            })),
            RenderLayers::layer(ping_it),
        ));
    }
}
