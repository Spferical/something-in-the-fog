use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use bevy::render::render_graph::RenderLabel;
use mat::{Light, LightBundle, LightingSettings};

mod mat;

use crate::edge::EdgeTexture;
use crate::map::TILE_SIZE;
use crate::renderer::OccluderTextureCpu;
use crate::sdf::SdfTexture;
use bevy::render::view::RenderLayers;
pub use mat::LightingMaterial;

const LIGHTING_ORDER_OFFSET: isize = 20;
const LIGHTING_LAYER: usize = 4;

/*pub fn alter_fov(
    mut commands: Commands,
    mut entity: Query<Entity, With<PerspectiveProjection>>,
    time: Res<Time>,
) {
    let Ok(entity) = entity.get_single_mut() else {
        return;
    };

    let fov = ((time.elapsed().as_millis() as f32) / 1000.0).cos() * 40. + 60.;
    let y = ((time.elapsed().as_millis() as f32) / 1000.0).cos() * 0.1 + 0.5;
    println!("got here, fov {:?}", fov);
    let proj = PerspectiveProjection {
        fov: fov.to_radians(),
        ..default()
    };
    // commands.entity(entity).insert(proj);
    commands
        .entity(entity)
        //.insert(Transform::from_xyz(0.0, y, 0.0).looking_at(Vec3::ZERO, -Vec3::Z))
        .insert(proj);
}*/

pub fn setup_lighting_pass(
    window: Single<&Window>,
    mut commands: Commands,
    sdf_texture_query: Query<&SdfTexture>,
    occluder_texture_query: Query<&OccluderTextureCpu>,
    edge_texture_query: Query<&EdgeTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LightingMaterial>>,
    // mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let (width, height) = (
        (window.resolution.physical_width()) as f32,
        (window.resolution.physical_height()) as f32,
    );

    let Ok(sdf_texture) = sdf_texture_query.get_single() else {
        return;
    };
    let Ok(occluder_texture) = occluder_texture_query.get_single() else {
        return;
    };
    let Ok(edge_texture) = edge_texture_query.get_single() else {
        return;
    };

    let settings = LightingSettings {
        tile_size: TILE_SIZE as i32,
    };
    let lights = [Light::default(); 8];

    let plane = meshes.add(Plane3d::default().mesh().size(1.0, height / width));
    commands.spawn((
        Mesh3d(plane),
        // MeshMaterial3d(standard_materials.add(Color::srgb(0.3, 0.5, 0.3))),
        MeshMaterial3d(materials.add(LightingMaterial {
            screen_texture: Some(occluder_texture.0.clone()),
            edge_texture: Some(edge_texture.0.clone()),
            seed_texture: Some(sdf_texture.iters[0].clone()),
            lighting_settings: settings,
            lights: LightBundle { lights },
            num_lights: 0,
        })),
        RenderLayers::layer(LIGHTING_LAYER),
    ));

    commands.spawn((
        Camera3d::default(),
        PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            ..default()
        },
        // Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Off,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
            hdr: true,
            order: LIGHTING_ORDER_OFFSET,
            ..default()
        },
        Transform::from_xyz(0.0, 0.5, 0.0).looking_at(Vec3::ZERO, -Vec3::Z),
        RenderLayers::layer(LIGHTING_LAYER),
    ));
}
