use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::*;
use mat::{Light, LightBundle, LightingSettings};

mod mat;

use crate::animation::{MuzzleFlash, WobbleEffects};
use crate::edge::EdgeTexture;
use crate::map::{MapPos, Zones};
use crate::player::{FlashlightInfo, Player};
use crate::renderer::{NonOccluderTexture, OccluderTexture, PlaneMouseMovedEvent};
use crate::sdf::SdfTexture;
use crate::ui::UiSettings;
use crate::PrimaryCamera;
use bevy::render::view::RenderLayers;
pub use mat::LightingMaterial;

const LIGHTING_ORDER_OFFSET: isize = 20;
pub const LIGHTING_LAYER: usize = 4;
pub const UI_LAYER: usize = 4;
pub const FOG: [f32; 6] = [130.0, 50.0, 130.0, 60.0, 80.0, 100.0];

#[derive(Component)]
pub struct RenderPlane;

pub fn get_mouse_location(
    pointers: Query<&PointerInteraction>,
    mut mouse_writer: EventWriter<PlaneMouseMovedEvent>,
) {
    for point in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position)
    {
        let pt = Vec2::new(point.x, point.z);
        mouse_writer.send(PlaneMouseMovedEvent(pt));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_lighting_pass(
    mut commands: Commands,
    query: Query<&MeshMaterial3d<LightingMaterial>, With<RenderPlane>>,
    mut materials: ResMut<Assets<LightingMaterial>>,
    mut mouse_reader: EventReader<PlaneMouseMovedEvent>,
    mut muzzle_flash: Query<(Entity, &mut MuzzleFlash)>,
    mut player_injury: Query<&mut WobbleEffects, With<Player>>,
    player_location: Query<&MapPos, With<Player>>,
    zones: Res<Zones>,
    primary_camera_query: Query<&Transform, With<PrimaryCamera>>,
    settings: ResMut<UiSettings>,
    flashlight_info: Res<FlashlightInfo>,
    time: Res<Time>,
) {
    let Ok(mat) = query.get_single() else {
        return;
    };

    let Ok(camera_2d_transform) = primary_camera_query.get_single() else {
        return;
    };
    let world_origin = (camera_2d_transform.translation) / crate::SDF_RES as f32;

    let mut mouse_position: Vec2 = Vec2::ZERO;
    for ev in mouse_reader.read() {
        mouse_position = ev.0 + 0.5;
    }

    let flashlight_center = Vec4::new(0.5, 0.5, 0.11, 0.0);
    let delta = (Vec2::new(0.5, 0.5) - mouse_position).normalize();
    let battery_curve = EasingCurve::new(0.0, 1.0, EaseFunction::CircularIn);
    let flashlight = Light {
        color: Vec4::new(
            1.0f32.lerp(0.8, flashlight_info.focus_factor),
            1.0f32.lerp(0.8, flashlight_info.focus_factor),
            1.0,
            1.0,
        ),
        intensity: 5000.0.lerp(20000.0, flashlight_info.focus_factor)
            * battery_curve.sample(flashlight_info.battery).unwrap_or(0.0),
        center: flashlight_center,
        direction: Vec4::new(delta.x, delta.y, 0.3, 0.0).normalize(),
        focus: 50f32.lerp(20.0, flashlight_info.focus_factor).to_radians(),
        attenuation: 10f32.lerp(1.0, flashlight_info.focus_factor),
        flicker: 1,
        ..default()
    };
    let player_light_center = Vec4::new(0.5, 0.5, 0.11, 0.0);

    let fog_density: f32 = if let Ok(pos) = player_location.get_single() {
        let mut fog = 130.0;
        for (i, zone) in zones.0.iter().enumerate() {
            if zone.contains(pos.0) {
                let fog_i = FOG[i];
                let fog_prev = if i > 0 { FOG[i - 1] } else { FOG[i] };
                let alpha = ((pos.0.x - zone.min.x) as f32) / ((zone.max.x - zone.min.x) as f32);
                fog = alpha * fog_i + (1.0 - alpha) * fog_prev;
            }
        }
        fog
    } else {
        130.0
    };

    let player_light_color = if let Some(injury) = player_injury
        .get_single_mut()
        .ok()
        .and_then(|p| p.effects.get(0).cloned())
    {
        (Vec4::new(injury.timer.fraction(), 0.0, 0.0, 1.0) * 10.0 + Vec4::new(1.0, 1.0, 1.0, 1.0))
            .normalize()
    } else {
        Vec4::new(1.0, 1.0, 1.0, 1.0)
    };
    let player_light = Light {
        color: player_light_color,
        intensity: 1.0,
        center: player_light_center,
        direction: Vec4::new(0.0, 0.0, 0.0, 0.0),
        focus: 0.0,
        attenuation: 5.0,
        ..default()
    };

    if let Some(mat) = materials.get_mut(mat) {
        mat.lights.lights[0] = flashlight;
        mat.lights.lights[1] = player_light;

        mat.lighting_settings.fog_density = fog_density;
        mat.lighting_settings.num_lights = 2;
        mat.lighting_settings.toggle_2d = settings.toggle_2d as i32;
        mat.lighting_settings.world_origin = world_origin;

        if settings.low_graphics {
            mat.lighting_settings.light_trace_samples = 4;
            mat.lighting_settings.ray_trace_samples = 12;
            mat.lighting_settings.fog_trace_samples = 4;
        } else {
            mat.lighting_settings.light_trace_samples = 16;
            mat.lighting_settings.ray_trace_samples = 16;
            mat.lighting_settings.fog_trace_samples = 8;
        }

        if let Ok((entity, mut flash)) = muzzle_flash.get_single_mut() {
            flash.timer.tick(time.delta());
            let intensity_scalar = flash.ease.sample_clamped(flash.timer.fraction());

            let muzzle_flash_light = Light {
                color: Vec4::new(1.0, 1.0, 0.7, 1.0),
                intensity: flash.info.muzzle_flash_max_intensity * intensity_scalar,
                center: player_light_center,
                direction: Vec4::new(delta.x, delta.y, 0.0, 0.0),
                focus: flash.info.muzzle_flash_focus,
                attenuation: flash.info.muzzle_flash_attenuation,
                ..default()
            };
            mat.lights.lights[2] = muzzle_flash_light;
            mat.lighting_settings.num_lights += 1;

            if flash.timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup_lighting_pass(
    mut commands: Commands,
    sdf_texture_query: Query<&SdfTexture>,
    occluder_texture_query: Query<&OccluderTexture>,
    edge_texture_query: Query<&EdgeTexture>,
    ui_texture_query: Query<&NonOccluderTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LightingMaterial>>,
    // mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let aspect_ratio = 1.0;

    let Ok(sdf_texture) = sdf_texture_query.get_single() else {
        return;
    };
    let Ok(occluder_texture) = occluder_texture_query.get_single() else {
        return;
    };
    let Ok(edge_texture) = edge_texture_query.get_single() else {
        return;
    };
    let Ok(ui_texture) = ui_texture_query.get_single() else {
        return;
    };

    let lights = [Light::default(); 8];

    let settings = LightingSettings {
        toggle_2d: 0,
        num_lights: 1,
        ..default()
    };

    let plane = meshes.add(Plane3d::default().mesh().size(1.0, aspect_ratio));
    commands.spawn((
        Mesh3d(plane),
        // MeshMaterial3d(standard_materials.add(Color::srgb(0.3, 0.5, 0.3))),
        MeshMaterial3d(materials.add(LightingMaterial {
            screen_texture: Some(occluder_texture.0.clone()),
            edge_texture: Some(edge_texture.0.clone()),
            seed_texture: Some(sdf_texture.iters[0].clone()),
            ui_texture: Some(ui_texture.0.clone()),
            lighting_settings: settings,
            lights: LightBundle { lights },
        })),
        RenderLayers::layer(LIGHTING_LAYER),
        RenderPlane,
    ));

    commands.spawn((
        Camera3d::default(),
        PerspectiveProjection {
            fov: 100.0_f32.to_radians(),
            ..default()
        },
        // Tonemapping::None,
        Tonemapping::TonyMcMapface,
        DebandDither::Disabled,
        Msaa::Off,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
            hdr: true,
            order: LIGHTING_ORDER_OFFSET,
            ..default()
        },
        Transform::from_xyz(0.0, 1.2, 0.0).looking_at(Vec3::ZERO, -Vec3::Z),
        RenderLayers::layer(LIGHTING_LAYER),
    ));
}
