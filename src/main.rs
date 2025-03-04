use std::time::Duration;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        renderer::RenderDevice,
        texture::TextureCache,
        view::RenderLayers,
    },
};

mod edge;
mod performance_ui;
mod renderer;
mod sdf;
mod world;

const CAMERA_DECAY_RATE: f32 = 2.;

fn on_resize(mut resize_reader: EventReader<bevy::window::WindowResized>) {
    for _e in resize_reader.read() {}
}

#[derive(Resource)]
struct MoveTimer(Timer);

impl Default for MoveTimer {
    fn default() -> Self {
        MoveTimer(Timer::new(Duration::from_secs_f64(0.25), TimerMode::Once))
    }
}

#[derive(Component)]
struct Player;

fn recreate_camera(
    window: Single<&Window>,
    mut commands: Commands,
    camera_query: Query<(Entity, &Camera)>,
    mut resize_reader: EventReader<bevy::window::WindowResized>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!(
        "dims {:?} {:?}",
        window.resolution.physical_width(),
        window.resolution.physical_height()
    );
    let mut image = Image::new_fill(
        Extent3d {
            width: window.resolution.physical_width(), // does this work?
            height: window.resolution.physical_height(),
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    let camera = Camera {
        target: image_handle.clone().into(),
        clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
        hdr: true,
        ..default()
    };

    let texture_cpu = renderer::OccluderTextureCpu(image_handle);

    match camera_query.get_single() {
        Ok((camera_entity, _)) => {
            println!("got here OK");
            commands
                .entity(camera_entity)
                .remove::<Camera>()
                .insert(camera);
        }
        Err(_) => {
            println!("got here Err");
            commands.spawn((Camera2d, camera, RenderLayers::layer(1)));
        }
    };

    let camera_postprocess = Camera {
        clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
        hdr: true,
        ..default()
    };
    commands.spawn((Camera2d, camera_postprocess, RenderLayers::layer(2)));

    let fullscreen_mesh = meshes.add(bevy::math::primitives::Rectangle::new(
        window.resolution.physical_width() as f32,
        window.resolution.physical_height() as f32,
    ));
    commands.spawn((
        Mesh2d(fullscreen_mesh),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.2, 0.3))),
        RenderLayers::layer(2),
        texture_cpu,
    ));

    for e in resize_reader.read() {
        println!("Resize happened {:?}", e);
    }
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    window.single_mut().resizable = true;
    commands.spawn((
        Player,
        world::WorldPos(IVec2::new(0, 0)),
        Mesh2d(meshes.add(Rectangle::new(24.0, 24.0))),
        MeshMaterial2d(materials.add(Color::LinearRgba(LinearRgba::RED))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        RenderLayers::layer(1),
    ));
    commands.init_resource::<MoveTimer>();
}

#[derive(Default)]
struct MovePlayerState {
    last_move_direction: IVec2,
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut world::WorldPos), With<Player>>,
    blocked_query: Query<&mut world::WorldPos, (With<world::BlocksMovement>, Without<Player>)>,
    mut timer: ResMut<MoveTimer>,
    mut local_state: Local<MovePlayerState>,
    tile_map: Res<world::TileMap>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        if let Ok((mut transform, mut world_pos)) = player_query.get_single_mut() {
            let mut movement = IVec2::ZERO;
            for (key, dir) in [
                (KeyCode::KeyW, IVec2::new(0, 1)),
                (KeyCode::KeyA, IVec2::new(-1, 0)),
                (KeyCode::KeyS, IVec2::new(0, -1)),
                (KeyCode::KeyD, IVec2::new(1, 0)),
            ] {
                if keyboard_input.pressed(key) {
                    movement += dir;
                }
            }
            let x_move = IVec2::new(movement.x, 0);
            let y_move = IVec2::new(0, movement.y);
            let x_dest = world_pos.0 + x_move;
            let y_dest = world_pos.0 + y_move;
            let x_valid = movement.x != 0
                && blocked_query
                    .iter_many(tile_map.0.get(&x_dest).unwrap_or(&vec![]))
                    .next()
                    .is_none();
            let y_valid = movement.y != 0
                && blocked_query
                    .iter_many(tile_map.0.get(&y_dest).unwrap_or(&vec![]))
                    .next()
                    .is_none();
            if !x_valid {
                movement.x = 0;
            }
            if !y_valid {
                movement.y = 0;
            }
            if x_valid && y_valid {
                // alternate
                if local_state.last_move_direction.x == x_move.x {
                    movement.x = 0;
                } else {
                    movement.y = 0;
                }
            }
            world_pos.0 += movement;
            transform.translation = world_pos.to_vec3(transform.translation.z);
            if movement != IVec2::ZERO {
                local_state.last_move_direction = movement;
                timer.0.reset();
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(renderer::Renderer)
        .add_plugins(performance_ui::PerformanceUiPlugin)
        .add_plugins(world::WorldPlugin)
        .add_systems(Startup, (recreate_camera, setup))
        .add_systems(Update, (update_camera, on_resize))
        .add_systems(FixedUpdate, move_player)
        .run();
}
