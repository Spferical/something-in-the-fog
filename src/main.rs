use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};
use map::Zones;
use player::Player;
use ui::{UiEvent, UiSettings};

mod animation;
mod assets;
mod despawn_after;
mod edge;
mod map;
mod mapgen;
mod mob;
mod player;
mod renderer;
mod sdf;
mod spawn;
mod ui;

const CAMERA_DECAY_RATE: f32 = 2.;

// Z-coordinates for everything in the game world.
const Z_PLAYER: f32 = 1.0;
const Z_TILES: f32 = 2.0;
const Z_ITEMS: f32 = 3.0;
const Z_MOBS: f32 = 4.0;

fn on_resize(mut resize_reader: EventReader<bevy::window::WindowResized>) {
    for _e in resize_reader.read() {}
}

#[derive(Component)]
struct PrimaryCamera;

fn create_camera(
    mut window: Single<&mut Window>,
    mut commands: Commands,
    // camera_query: Query<(Entity, &Camera), With<PrimaryCamera>>,
    mut resize_reader: EventReader<bevy::window::WindowResized>,
    mut images: ResMut<Assets<Image>>,
) {
    window.resizable = true;
    let mut image = Image::new_fill(
        Extent3d {
            width: window.resolution.physical_width(), // does this work?
            height: window.resolution.physical_height(),
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    let camera = Camera {
        clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
        hdr: true,
        order: 0,
        ..default()
    };

    let texture_cpu = renderer::OccluderTextureCpu(image_handle);
    commands.spawn(texture_cpu.clone());

    commands.spawn((
        Camera2d,
        camera,
        RenderLayers::layer(1),
        PrimaryCamera,
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
        OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        },
    ));

    for e in resize_reader.read() {
        println!("Resize happened {:?}", e);
    }
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<PrimaryCamera>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<PrimaryCamera>)>,
    time: Res<Time>,
    mut ev_scroll: EventReader<MouseWheel>,
    ui_settings: Res<UiSettings>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());

    for event in ev_scroll.read() {
        if ui_settings.debug_scroll {
            let factor = match event.unit {
                MouseScrollUnit::Line => 0.2,
                MouseScrollUnit::Pixel => 0.01,
            };
            camera.scale -= event.y * factor;
        }
    }
}

fn setup(mut window: Query<&mut Window>) {
    window.single_mut().resizable = true;
}

fn handle_ui_event(
    mut ev: EventReader<UiEvent>,
    zones: Res<Zones>,
    mut player_query: Query<(&mut Transform, &mut map::MapPos), With<Player>>,
) {
    for ev in ev.read() {
        match ev {
            UiEvent::TeleportPlayer(zone_idx) => {
                if let Some(zone) = zones.0.get(*zone_idx) {
                    let (mut transform, mut map_pos) = player_query.single_mut();
                    let dest = zone.center();
                    map_pos.0 = dest;
                    transform.translation = map_pos.to_vec2().extend(transform.translation.z);
                }
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            ui::UiPlugin,
            ui::performance::PerformanceUiPlugin,
            LogDiagnosticsPlugin::default(),
            assets::AssetsPlugin,
            map::WorldPlugin,
            animation::AnimatePlugin,
            renderer::Renderer,
            spawn::SpawnPlugin,
            player::PlayerPlugin,
            mob::MobPlugin,
            despawn_after::DespawnAfterPlugin,
        ))
        .add_systems(Startup, (create_camera, setup))
        .add_systems(Update, (handle_ui_event, update_camera, on_resize).chain())
        .run();
}
