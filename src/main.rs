use std::{f32::consts::PI, time::Duration};

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::bounding::{Aabb2d, RayCast2d};
use bevy::render::camera::RenderTarget;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};

use map::{Map, MapPos, Mob, TILE_SIZE, Tile};

mod edge;
mod map;
mod mapgen;
// mod performance_ui;
mod renderer;
mod sdf;

const CAMERA_DECAY_RATE: f32 = 2.;
const PLAYER_MOVE_DELAY: Duration = Duration::from_millis(100);
const PLAYER_SHOOT_DELAY: Duration = Duration::from_millis(500);
const PLAYER_START: IVec2 = IVec2::new(100, 0);

fn on_resize(mut resize_reader: EventReader<bevy::window::WindowResized>) {
    for _e in resize_reader.read() {}
}

#[derive(Component)]
struct Player;

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
    ));

    /*match camera_query.get_single() {
        Ok((camera_entity, _)) => {
            commands
                .entity(camera_entity)
                .remove::<Camera>()
                .insert(camera);
        }
        Err(_) => {
            commands.spawn((
                Camera2d,
                camera,
                RenderLayers::layer(1),
                PrimaryCamera,
                Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
            ));
        }
    };*/

    for e in resize_reader.read() {
        println!("Resize happened {:?}", e);
    }
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<PrimaryCamera>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<PrimaryCamera>)>,
    time: Res<Time>,
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
}

fn setup(mut commands: Commands, mut window: Query<&mut Window>, assets: Res<map::WorldAssets>) {
    window.single_mut().resizable = true;
    let player_start_translation = Vec3::new(PLAYER_START.x as f32, PLAYER_START.y as f32, 1.0);
    commands.spawn((
        Player,
        map::MapPos(PLAYER_START),
        Mesh2d(assets.circle.clone()),
        MeshMaterial2d(assets.red.clone()),
        Transform::from_translation(player_start_translation),
        RenderLayers::layer(1),
    ));
    commands.insert_resource(MoveTimer(Timer::new(PLAYER_MOVE_DELAY, TimerMode::Once)));
    commands.insert_resource(MouseWorldCoords(player_start_translation.truncate()));
    commands.insert_resource(ShootState {
        timer: Timer::new(PLAYER_SHOOT_DELAY, TimerMode::Once),
        player_last_position: player_start_translation.truncate(),
        focus: 0.0,
    });
}

#[derive(Resource)]
struct MouseWorldCoords(Vec2);

fn update_mouse_coords(
    mut mouse_world_coords: ResMut<MouseWorldCoords>,
    query_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, With<PrimaryCamera>)>,
) {
    let (camera, camera_transform) = query_camera.single();
    if let Ok(window) = query_window.get_single() {
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
        {
            mouse_world_coords.0 = world_position;
        }
    }
}

#[derive(Debug, Resource)]
struct ShootState {
    timer: Timer,
    player_last_position: Vec2,
    focus: f32,
}

#[allow(clippy::complexity)]
fn update_shooting(
    player_query: Query<&Transform, With<Player>>,
    mut shoot_state: ResMut<ShootState>,
    mouse_world_coords: Res<MouseWorldCoords>,
    time: Res<Time>,
    mut gizmos: Gizmos,
    mouse_button: Res<ButtonInput<MouseButton>>,
    map: Res<Map>,
    mobs: Query<(&Mob, &Transform), Without<Player>>,
    tiles: Query<(&Tile, &Transform), (Without<Mob>, Without<Player>)>,
) {
    let player_pos = player_query.single();

    shoot_state.timer.tick(time.delta());
    if shoot_state.player_last_position != player_pos.translation.truncate() {
        shoot_state.player_last_position = player_pos.translation.truncate();
        shoot_state.focus -= 1.0;
    } else {
        shoot_state.focus += time.delta().as_secs_f32();
    }
    shoot_state.focus = shoot_state.focus.clamp(0.0, 2.5);

    let mouse_offset = mouse_world_coords.0 - player_pos.translation.truncate();
    let aim_angle_degrees = (30.0 - shoot_state.focus * 12.0).max(0.0);
    let left_angle = Vec2::from_angle(aim_angle_degrees / 2.0 * PI / 180.0);
    let right_angle = Vec2::from_angle(-aim_angle_degrees / 2.0 * PI / 180.0);
    let left_ray = left_angle.rotate(mouse_offset.normalize());
    let right_ray = right_angle.rotate(mouse_offset.normalize());
    gizmos.line_gradient_2d(
        player_pos.translation.truncate() + left_ray * 60.0,
        player_pos.translation.truncate() + left_ray * 240.0,
        bevy::color::palettes::basic::YELLOW.with_alpha(0.5),
        Color::NONE.into(),
    );
    gizmos.line_gradient_2d(
        player_pos.translation.truncate() + right_ray * 60.0,
        player_pos.translation.truncate() + right_ray * 240.0,
        bevy::color::palettes::basic::YELLOW.with_alpha(0.5),
        Color::NONE.into(),
    );

    if mouse_button.just_pressed(MouseButton::Left) {
        // shoot
        // let angle_degrees = rand::thread_rng().gen_range(left_angle..right_angle);
        let line_start = player_pos.translation.truncate();
        shoot_state.focus -= 1.0;
        let mut collisions = vec![];
        let player_pos_ivec2 = MapPos::from_vec3(player_pos.translation).0;
        for x in player_pos_ivec2.x - 100..player_pos_ivec2.x + 100 {
            for y in player_pos_ivec2.y - 100..player_pos_ivec2.y + 100 {
                if let Some(entities) = map.0.get(&IVec2 { x, y }) {
                    for (_mob, transform) in mobs.iter_many(entities) {
                        collisions.push(Aabb2d::new(
                            transform.translation.truncate(),
                            Vec2::splat(TILE_SIZE / 2.0),
                        ));
                    }
                    for (tile, transform) in tiles.iter_many(entities) {
                        if tile.0.blocks_movement() {
                            collisions.push(Aabb2d::new(
                                transform.translation.truncate(),
                                Vec2::splat(TILE_SIZE / 2.0),
                            ));
                        }
                    }
                }
            }
        }
        let dist_to_player = |point| player_pos.translation.truncate().distance_squared(point);
        collisions.sort_by(|a, b| {
            dist_to_player((a.min + a.max) / 2.0)
                .partial_cmp(&dist_to_player((b.min + b.max) / 2.0))
                .unwrap()
        });
        if let Ok(dir) = Dir2::new(mouse_offset) {
            let ray = RayCast2d::new(line_start, dir, 4000.0);
            let mut collided = false;
            for c in collisions {
                if let Some(distance) = ray.aabb_intersection_at(&c) {
                    gizmos.line_2d(
                        line_start,
                        line_start + dir * distance,
                        bevy::color::palettes::basic::YELLOW,
                    );
                    collided = true;
                    break;
                }
            }
            if !collided {
                gizmos.line_2d(
                    line_start,
                    line_start + dir * 4000.0,
                    bevy::color::palettes::basic::YELLOW,
                );
            }
        }
    }
}

#[derive(Resource)]
struct MoveTimer(Timer);

#[derive(Default)]
struct MovePlayerState {
    last_move_direction: IVec2,
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut map::MapPos), With<Player>>,
    blocked_query: Query<&mut map::MapPos, (With<map::BlocksMovement>, Without<Player>)>,
    mut timer: ResMut<MoveTimer>,
    mut local_state: Local<MovePlayerState>,
    tile_map: Res<map::Map>,
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
        // .add_plugins(performance_ui::PerformanceUiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(map::WorldPlugin)
        .add_plugins(renderer::Renderer)
        .add_systems(Startup, (create_camera, setup))
        .add_systems(
            Update,
            (
                update_camera,
                on_resize,
                update_mouse_coords,
                update_shooting,
            )
                .chain(),
        )
        .add_systems(FixedUpdate, move_player)
        .run();
}
