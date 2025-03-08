use bevy::core_pipeline::tonemapping::DebandDither;
use bevy::render::render_graph::RenderLabel;
use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};

mod mat;
mod prepare;

use crate::edge::EdgeTexture;
use crate::map::TILE_SIZE;
use crate::renderer::OccluderTextureCpu;
use bevy::render::view::RenderLayers;
pub use mat::SdfMaterial;
pub use prepare::{on_resize_sdf_texture, prepare_sdf_texture};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SdfPass;

#[derive(Component)]
pub struct SdfTexture {
    pub iters: Vec<Handle<Image>>,
}

const SDF_ORDER_OFFSET: usize = 2;
const SDF_START_LAYER: usize = 31;

pub fn setup_sdf_pass(
    window: Single<&Window>,
    mut commands: Commands,
    sdf_texture_query: Query<&SdfTexture>,
    occluder_texture_query: Query<&OccluderTextureCpu>,
    edge_texture_query: Query<&EdgeTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SdfMaterial>>,
) {
    let (width, height) = (
        (window.resolution.physical_width()) as f32,
        (window.resolution.physical_height()) as f32,
    );

    let larger_dim = width.max(height);
    let num_passes = (larger_dim as f32).log2().ceil() as usize;
    let endpoint = num_passes;

    let Ok(sdf_texture) = sdf_texture_query.get_single() else {
        return;
    };
    let Ok(occluder_texture) = occluder_texture_query.get_single() else {
        return;
    };
    let Ok(edge_texture) = edge_texture_query.get_single() else {
        return;
    };

    let fullscreen_mesh = meshes.add(Rectangle::new(width, height));
    let proj = OrthographicProjection {
        scale: 1.0,
        ..OrthographicProjection::default_2d()
    };
    for i in 0..endpoint {
        let j = endpoint - i;
        let ping_it = SDF_START_LAYER - i;
        let ping_image = sdf_texture.iters[j].clone();
        let pong_image = sdf_texture.iters[j - 1].clone();

        /*if i == endpoint - 1 {
            let camera_postprocess = Camera {
                clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
                hdr: true,
                order: (SDF_ORDER_OFFSET + i) as isize,
                ..default()
            };
            commands.spawn((
                Camera2d,
                proj.clone(),
                camera_postprocess,
                RenderLayers::layer(ping_it),
            ));
        } else {*/
        let camera_postprocess = Camera {
            clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
            target: pong_image.clone().into(),
            hdr: true,
            order: (SDF_ORDER_OFFSET + i) as isize,
            ..default()
        };
        commands.spawn((
            Camera2d,
            proj.clone(),
            Tonemapping::None,
            Msaa::Off,
            DebandDither::Disabled,
            camera_postprocess,
            RenderLayers::layer(ping_it),
        ));
        // }
        commands.spawn((
            Mesh2d(fullscreen_mesh.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)),
            MeshMaterial2d(materials.add(SdfMaterial {
                screen_texture: Some(occluder_texture.0.clone()),
                edge_texture: Some(edge_texture.0.clone()),
                seed_texture: Some(ping_image.clone()),
                iteration: i as i32,
                probe_size: 1 << (num_passes - i - 1),
                tile_size: TILE_SIZE as i32,
            })),
            RenderLayers::layer(ping_it),
        ));
    }
}
