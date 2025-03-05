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
    pub ping: Handle<Image>,
    pub pong: Handle<Image>,
}

const SDF_ORDER_OFFSET: usize = 1;
const SDF_PING_LAYER: usize = 14;
const SDF_PONG_LAYER: usize = 15;

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
    let num_passes = 1;
    println!("num passes {:?}", num_passes);

    let Ok(sdf_texture) = sdf_texture_query.get_single() else {
        return;
    };
    let Ok(occluder_texture) = occluder_texture_query.get_single() else {
        return;
    };

    let fullscreen_mesh = meshes.add(Rectangle::new(width, height));

    for i in 0..num_passes {
        let (ping_layer, sdf_texture, output_texture) = match i % 2 {
            0 => (SDF_PING_LAYER, &sdf_texture.ping, &sdf_texture.pong),
            _ => (SDF_PONG_LAYER, &sdf_texture.pong, &sdf_texture.ping),
        };

        if i == num_passes - 1 {
            let camera_postprocess = Camera {
                // clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
                // hdr: true,
                // order: (SDF_ORDER_OFFSET + i) as isize,
                //order: 0,
                ..default()
            };
            commands.spawn((
                Camera2d,
                camera_postprocess,
                RenderLayers::layer(ping_layer),
            ));
        } else {
            let camera_postprocess = Camera {
                // clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
                // hdr: true,
                target: output_texture.clone().into(),
                order: (SDF_ORDER_OFFSET + i) as isize,
                ..default()
            };
            commands.spawn((
                Camera2d,
                camera_postprocess,
                RenderLayers::layer(ping_layer),
            ));
        }

        commands.spawn((
                Mesh2d(fullscreen_mesh.clone()),
                MeshMaterial2d(materials.add(SdfMaterial {
                    screen_texture: Some(occluder_texture.0.clone()),
                    sdf: Some(sdf_texture.clone()),
                    iteration: i as i32,
                })),
                RenderLayers::layer(ping_layer),
        ));
    }
}
